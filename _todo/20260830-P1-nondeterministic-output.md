# TPP: Nondeterministic tag output across identical runs

## Summary

Repeated runs of the *same binary* on the *same file* return different
values. Discovered 2026-08-30 while adjudicating the float-division fix
(the flips predate it — the pre-fix binary flips identically, 4/1 split
over 5 runs). This makes the compat oracle noisy and would surface as
unstable metadata in PhotoStructure, so it blocks trusting any
snapshot-based ratchet.

## Current phase

- [x] Research & Planning (initial repro only)
- [x] Write breaking tests (tests/value_determinism_tests.rs — validated failing pre-fix)
- [x] Implementation (three distinct root causes, all fixed)
- [x] Review & Refinement (Codex review done; R765-A/B fixed, R765-C deferred below)
- [x] Final Integration (`make verify` passed; 330-file sweep x6 runs: zero value flips)

## Root causes found (2026-08-30)

1. **Composite build order** — `resolve_and_compute_composites` iterated
   `COMPOSITE_TAGS.values()` (HashMap, per-process seed) and never deferred a
   composite whose Desire names an unbuilt composite. Fixed in
   src/composite_tags/orchestration.rs by mirroring BuildCompositeTags:
   alphabetical "Module-Name" order (ExifTool.pm:3984, 5793) plus notBuilt
   deferral (ExifTool.pm:4044-4053, 4074-4078) and the allBuilt retry
   (ExifTool.pm:4150-4158). This was the cause of the FocalLength35efl flips
   (26 vs 5.7 on apple/iphone_13_pro.jpg) and affected ~190 of 330 corpus files.
2. **Canon FileNumber** — `apply_canon_main_table_print_conv` fabricated a
   FileNumber from CanonCameraSettings (tag 0x1) entry 8 and stored it under
   tag 0x8, racing with the real FileNumber. Canon.pm:1226-1232 shows 0x1 is a
   SubDirectory with no FileNumber; branch deleted
   (src/implementations/canon/mod.rs).
3. **XMP duplicate names** — `flatten_xmp_structure` iterated nested HashMaps,
   so exif:NativeDigest vs tiff:NativeDigest (both output "NativeDigest")
   raced. Repro file: test-images/canon/eos_1ds_mark_ii.jpg. Fixed by
   preserving document order in parse and resolving collisions with ExifTool's
   FoundTag rule (ExifTool.pm:9514-9585): statically resolved priority chain
   per-tag Priority -> table PRIORITY -> 0 for Avoid'd tags (:9469-9473, table
   AVOID propagation :9250-9251), existing 0-priority promoted to 1 before the
   `>=` comparison (:9544-9551, :9564). Both NativeDigest tables carry
   PRIORITY => 0 (XMP.pm:1900, 1992) so the FIRST property in document order
   wins — verified against vendored exiftool in both document orders. Required
   `priority: Option<i8>` in generated XmpTagInfo
   (codegen/src/strategies/xmp_tag.rs + `make codegen`). The table lookup uses
   the raw property ID (not the display-name storage key) so metadata for tags
   like dc:source (key "source", stored "Source", Avoid per XMP.pm:1034)
   is not lost.

## Codex review verdicts (2026-08-30)

- R765-A (Avoid-only priority reduction wrong; tiff-first must keep TIFF):
  accepted — reproduced with vendored exiftool, fixed via the full priority
  chain above; both document orders pinned in tests.
- R765-B (priority metadata lost when property key != display name, e.g.
  dc:source): accepted — reproduced (photoshop-first fixture returned DCSOURCE
  pre-fix); fixed by carrying the raw property ID in the document-order
  records; regression test test-resources/source-collision.xmp.
- R765-C (property_order keeps first-occurrence position while the parse map
  keeps the last value, so INTERLEAVED repeats of the same output name across
  namespaces — e.g. photoshop:History, xmpMM:History, photoshop:History again
  — can pick a different winner than ExifTool): accepted as a real edge-case
  divergence, deferred. It is deterministic (not a value flip), requires a
  same-name property repeated AFTER the colliding namespace's property, and
  reproducing ExifTool exactly needs a per-occurrence event stream through the
  Bag/Alt merging logic. No corpus file trips it (330-file sweep is clean).

## Scope (Matthew, 2026-08-30)

VALUE nondeterminism only. Field/tag *ordering* differences between runs
are explicitly out of scope — do not spend effort stabilizing output
order. All three repros below are value flips, so all are in scope.

## Original repros (2026-08-30, 5-run samples — all fixed, see Root causes)

1. `Composite:FocalLength35efl` flips between `26` and `5.7` (2/3 split).
   Suspect: `ScaleFactor35efl` sometimes is not resolved before its
   dependent composite — composite dependency resolution order is not
   deterministic.
2. `MakerNotes:FileNumber` flips between `130-0112` and `6-5535` on
   `test-images/canon/powershot_s110.jpg`. Suspect: conditional tag
   variants racing — two candidate interpretations of the same tag ID and
   the winner varies per run.
3. `XMP:NativeDigest` flips between two different source tags (two XMP
   properties map to the same output name; which one wins varies).

## Likely mechanism

All three smell like iteration over a `HashMap`/`HashSet` (randomized
seed per process) deciding evaluation or precedence order somewhere it
must not. ExifTool's equivalents are ordered (arrays, sorted keys, or
documented precedence).

## Approach

Per docs/TDD.md: for each repro, write a test that runs extraction N
times in-process and asserts identical output, validate it fails, then
find the unordered collection and impose ExifTool's order (Trust
ExifTool — find what order ExifTool actually uses, don't invent one).
A cheap first sweep: `rg 'HashMap|HashSet' src/composites src/exif` for
iteration sites feeding resolution or precedence.

## Constraints

- Fix the ordering at the source; do NOT paper over with sorting the
  final output.
- The three repros above may share one root cause or be three distinct
  ones — verify each independently.
