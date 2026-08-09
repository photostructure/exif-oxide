# TPP: Faithful Read IR and Mechanical ExifTool Processor Dispatch

## Summary

The code generator currently recognizes much of ExifTool's behavioral table
metadata and then discards it. Runtime `TagInfo` keeps mainly name, format,
conversions, and offset status. Missing conditions, subdirectories, hooks,
processor identifiers, priorities, counts, DataMembers, defaults, and table
flags are reconstructed through manual tables and heuristic processor
selection. This is both harder to reason about and less faithful than the
source being ported.

Introduce a lossless read-side intermediate representation for selected
ExifTool modules and mechanically port the explicit/table/default dispatch
chain plus `ProcessDirectory` and `ProcessBinaryData`. Keep working readers
and generated conversion functions as adapters during migration. Switch each
path only after shadow-mode equality with the old implementation and the
pinned ExifTool oracle.

This is a staged strangler migration, not a clean-sheet rewrite. Manual Rust
remains appropriate only for cited ExifTool code references and procedural
exceptions.

## Current phase

- [x] Research & Planning
- [x] Design alternatives
- [x] Task breakdown
- [ ] Write characterization tests
- [ ] IR implementation
- [ ] Processor implementation
- [ ] Shadow migration
- [ ] Review & Refinement
- [ ] Final Integration

## Required reading

- [AGENTS.md](../AGENTS.md)
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md)
- [CODEGEN.md](../docs/CODEGEN.md)
- [PROCESSOR-DISPATCH.md](../docs/guides/PROCESSOR-DISPATCH.md)
- [ExifTool PROCESS_PROC](../third-party/exiftool/doc/concepts/PROCESS_PROC.md)
- [ExifTool module overview](../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md)
- [compatibility oracle v2](20260808-P0-compatibility-oracle-v2.md)

## Verified failure in the current shape

- `TagInfo` retains name, format, PrintConv, ValueConv, offset status, and
  little else.
- Codegen recognizes but skips table behavior including `PROCESS_PROC`,
  `GROUPS`, `FORMAT`, `FIRST_ENTRY`, `PRIORITY`, `VARS`, and `DATAMEMBER`.
- Tag-level `RawConv`, `Condition`, `SubDirectory`, `Hook`, `DataMember`,
  priority, count, byte order, and flags are not generally represented.
- Conditional definitions are reduced to heuristics, losing source order,
  predicates, and side effects.
- Runtime processor dispatch scores all registered processors by invented
  capabilities. ExifTool uses an explicit processor, then table
  `PROCESS_PROC`, then `ProcessExif`.
- An invented Canon `SerialDataMkII` processor is selected by model even
  though no corresponding ExifTool processor exists. `CanonMainProcessor`
  is registered but returns no tags.
- The generic binary-data implementation uses cumulative offsets. ExifTool
  uses `index * default-format-size + varSize`, supports sparse/negative
  indices, and records DataMembers.
- Binary scalar reads can ignore the supplied slice and read the full reader
  buffer through a Canon helper.
- Hand-transcribed Canon and Panasonic tables duplicate generated source data.
- Unsupported generated conversions can silently return identity values.
  The permanent real-ExifTool fallback is not yet invoked inside this crate.

## Principles

1. Generated data preserves source order and provenance.
2. Every read-relevant ExifTool field is represented or explicitly classified
   unsupported; no silent drops.
3. Dispatch follows ExifTool's explicit/table/default chain.
4. Unsupported behavior returns a typed `FallbackNeeded`, never a plausible
   identity result.
5. Existing working output is a regression floor.
6. Old and new paths coexist only behind testable adapters and shadow mode.
7. Delete speculative/manual systems only after reachability and parity proof.

## Read IR scope

The first IR must preserve:

- table name/module/provenance and source order;
- table processor, groups, default format, byte order, first-entry behavior,
  priority, flags, variables, and DataMembers;
- ordered tag definitions including numeric/string/fractional identifiers;
- name, format, count, RawConv, ValueConv, PrintConv, Condition, Hook,
  SubDirectory, DataMember, priority, flags, and unknown-tag policy;
- named code/procedure references as stable identifiers;
- an explicit unsupported construct with source location and fallback reason.

Do not force every Perl expression into Rust source. The existing compiled
PPI conversions remain valid adapters. A later runtime scalar-expression IR
may be adopted only after differential evaluation against Perl proves the
reachable supported slice.

## Initial processor slice

Start with Canon CameraSettings/AFInfo and Panasonic RAW because current code
contains both manual duplicates and generic processing claims. Mechanically
port:

1. processor resolution;
2. ordered conditional table selection;
3. `ProcessDirectory` recursion and base/offset propagation;
4. `ProcessBinaryData` index, format, count, DataMember, condition, and
   conversion behavior;
5. typed unsupported/fallback propagation.

Compare numeric (`-n`) and printed output across several camera models. Once
stable, use the same IR to evolve the existing QuickTime walker into a
table-driven ISO-BMFF route for MOV/MP4, then HEIC and CR3. Do not copy the
manual QuickTime architecture into those formats first.

## Alternatives considered

### Keep adding format/manufacturer special cases

Rejected. It duplicates source tables, invents routing rules, and throws away
the bug history encoded in ExifTool metadata.

### Rewrite everything around a runtime Perl-expression interpreter

Rejected as the first step. It still requires exact Perl coercion, context,
regex, mutation, and callout semantics. Characterize a narrow evaluator later
without discarding the working compiled conversions.

### Direct-port only currently required tags

Useful for named procedures, but not as the general architecture. It scales
poorly across table-driven MakerNotes and future ExifTool releases.

## Tasks

1. [ ] Use oracle-v2 observations to freeze Canon/Panasonic numeric and
   printed characterization fixtures, including sparse binary indices,
   conditions, subdirectories, DataMembers, and unsupported constructs.
2. [ ] Inventory every read-relevant table/tag field in the selected ExifTool
   modules. Add a generator test that fails for every silently dropped field.
3. [ ] Define the neutral read IR and generated provenance/unsupported types.
   Generate it beside current tables without changing runtime behavior.
4. [ ] Add direct `ProcId` resolution matching ExifTool's
   explicit/table/default order. Characterize and then bypass capability
   scoring for the selected slice.
5. [ ] Mechanically port `ProcessDirectory` and `ProcessBinaryData` from the
   pinned source with cited source locations and byte/offset tests.
6. [ ] Run old and new selected paths in test-only shadow mode. Require equal
   native output plus equality with ExifTool before switching.
7. [ ] Replace the selected manual Canon/Panasonic tables with generated IR.
   Prove no observation regresses, then remove only the now-unreachable code.
8. [ ] Introduce `FallbackNeeded` at conversion/processor boundaries and
   characterize how the public/integration layer delegates it to real
   ExifTool.
9. [ ] Extend the proven IR to QuickTime `ProcessMOV` tables in shadow mode;
   migrate MOV/MP4 before HEIC and CR3 branches.
10. [ ] Generate a reachability report for identity placeholders, invented
    processors, and manual tables. Quarantine or delete each only with proof.
11. [ ] Run focused tests, oracle matrix, fuzz targets, and `make verify` for
    each switched slice. Keep commits small and one migration slice each.

## Acceptance gates

- Selected modules have zero silently dropped read fields.
- Processor resolution and binary offsets match cited ExifTool behavior.
- Numeric and printed Canon/Panasonic observations match ExifTool.
- Unsupported reachable behavior returns `FallbackNeeded`.
- No currently working oracle observation regresses.
- Selected manual tables and invented dispatch are removed only after their
  replacement is active and parity-proven.
- The first ISO-BMFF migration matches both the existing QuickTime reader and
  ExifTool in shadow mode.
- `make verify` passes after every switched slice.

## Out of scope

- Write support.
- A big-bang replacement of all readers or the PPI transpiler.
- Hand transcription of ExifTool tables.
- Optimizing or simplifying ExifTool behavior before parity.
- Extending the native claim when fallback ownership is undefined.

## Files likely involved

- `codegen/src/strategies/`
- new codegen read-IR types and field-coverage tests
- `src/types/tag_info.rs` or a successor generated-runtime representation
- `src/processor_registry/`
- `src/exif/binary_data.rs`
- `src/implementations/canon/`
- `src/raw/formats/panasonic.rs`
- test-only shadow adapters and oracle matrix fixtures
