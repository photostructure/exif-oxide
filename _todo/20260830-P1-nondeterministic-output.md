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
- [ ] Write breaking tests
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Confirmed repros (2026-08-30, 5-run samples)

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
