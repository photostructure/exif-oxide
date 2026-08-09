# TPP: Compatibility Oracle v2 — Common-Media Contract and Matrix

## Summary

The current compatibility gate is a useful conservative ratchet, but its
headline is easy to overread. The verified baseline is 23 of 191 configured
and observed tag names matching ExifTool in every exercised file. It is not
23 of all 272 configured tags, and it does not measure the rest of ExifTool.
One mismatch in any file makes the entire tag fail, while the report assigns
that tag to only the first failing file. Real improvements can therefore be
hidden or attributed to the wrong format.

Build an explicit common-media contract and a committed file x tag
observation matrix. Preserve the existing works-everywhere ratchet, add
format/manufacturer/required-tag views, make missing mandatory corpus inputs
fail, and use the pinned ExifTool 13.59 executable for every oracle path.

Success means each Tier 1 claim is supported by an exercised observation or
is explicitly delegated to real ExifTool. No percentage may silently shrink
its denominator when corpus files are absent.

## Current phase

- [x] Research & Planning
- [x] Design alternatives
- [x] Task breakdown
- [ ] Write breaking tests
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Required reading

- [AGENTS.md](../AGENTS.md)
- [TPP-GUIDE.md](../docs/TPP-GUIDE.md)
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- [TDD.md](../docs/TDD.md)
- [MILESTONES.md](../docs/MILESTONES.md)
- [snapshot oracle TPP](../_done/20260701-P1-snapshot-oracle-integrity.md)

## Ground truth

- Pinned oracle: `third-party/exiftool/exiftool`, version 13.59.
- Committed snapshots: 379; six are hard-excluded; 373 are exercised when
  every referenced source file is present.
- Current aggregate: 23/191 observed configured tags work in every exercised
  file; 168 gaps are reviewed and allowlisted.
- Configured supported tags: 272. Required tags: 120. Their overlap is 99;
  21 required tags are outside the supported list.
- Only 72 required keys occur in the snapshot set. Another 27 are configured
  as supported but unobserved.
- Corpus skew: 224 JPEG snapshots, 22 DNG, 21 NEF, 20 CR3, five MOV, four
  HEIC, and zero MP4, AVI, or standalone XMP snapshots. Local test assets do
  contain MP4 and AVI samples, but snapshot generation omits them.
- Missing source files currently reduce observations and disable only the
  stale-gap check. CI tracks no `test-images` files, so this is a live gap.
- MIME and binary comparison paths invoke PATH `exiftool`; snapshot creation
  correctly invokes the pinned executable.
- The comprehensive binary extraction test is a dashboard: it does not fail
  for mismatched payloads, low success, or zero discovered binary tags.

## Contract

Track four disjoint claims:

1. Tier 1 common-media formats and required metadata.
2. Native supported and exercised observations.
3. Explicit fallback to real ExifTool.
4. Explicitly deferred behavior with a reason.

The initial common-media format set is JPEG, TIFF, PNG, GIF, WebP,
HEIC/HEIF, AVIF, MOV, MP4, AVI, and standalone XMP, plus the major RAW
families already represented. Group names must be canonical ExifTool `-G`
names; aliases belong in an explicit tested mapping, not in denominators.

## Design

### Observation record

Emit one deterministic record per expected observation containing:

- corpus item identifier and extension;
- manufacturer/model when applicable;
- canonical group and tag;
- required/supported/fallback/deferred classification;
- ExifTool value, exif-oxide value, and normalized comparison outcome;
- mismatch category and reviewed-gap identifier when non-matching;
- pinned ExifTool version and snapshot provenance.

Derive the legacy works-everywhere view, per-format view, per-manufacturer
view, required-tag coverage, and unexpected-difference report from the same
records. Do not maintain competing comparison implementations.

### Corpus manifest

Commit a manifest of mandatory and optional inputs with path, format, tier,
provenance/license, expected snapshot, exclusion reason, and ExifTool
version. Full mode fails when a mandatory item is absent. A small licensing-
safe PR corpus may coexist with a provisioned full corpus, but each mode has
an explicit denominator and must say which one it ran.

### Ratchets

- Preserve every currently working observation.
- Keep the works-everywhere tag ratchet for continuity.
- Fail on new file x tag mismatches even when the global tag was already red.
- Fail on stale reviewed gaps at the observation/category level.
- Make numeric tolerances and omissions tag-specific and documented.
- Turn binary and MIME comparison into pinned-oracle ratchets with explicit
  minimum coverage.

## Alternatives considered

### Keep only the aggregate

Rejected. It conceals format-local progress and attributes a multi-format
failure to one lexicographically first file.

### Replace the aggregate outright

Rejected. The 23/191 works-everywhere number is conservative and provides
continuity. The matrix should explain it, not erase it.

### Commit the entire current corpus

Not assumed. Licensing and repository size need evidence. A manifest plus a
small mandatory PR subset and provisioned full set can enforce honest modes.

## Tasks

1. [ ] Write failing tests for missing mandatory corpus items, canonical group
   aliases, a format-local improvement hidden by a global failure, and a new
   observation mismatch under an already-allowlisted tag.
2. [ ] Reconcile `docs/required-tags.json` and
   `config/supported_tags.json`; classify every Tier 1 requirement as native,
   fallback, or deferred.
3. [ ] Add the corpus manifest and include existing MP4/AVI assets plus the
   standalone XMP suite in snapshot discovery. Record every exclusion.
4. [ ] Refactor comparison into one pinned-oracle observation pipeline and
   derive JSON and human reports from it.
5. [ ] Add file x tag, format, manufacturer, required-tag, and legacy
   works-everywhere ratchets.
6. [ ] Pin MIME and binary paths to vendored ExifTool; make their assertions
   meaningful and require minimum exercised counts.
7. [ ] Provision the mandatory corpus in CI or add a licensing-safe committed
   subset. Make the selected mode and denominator visible in CI output.
8. [ ] Regenerate the baseline once, review every classification change, run
   `make verify`, and update only claims supported by the new reports.

## Acceptance gates

- All oracle subprocesses use `third-party/exiftool/exiftool` 13.59.
- Missing mandatory data fails rather than shrinking coverage.
- MP4, AVI, and standalone XMP have exercised observations.
- Every Tier 1 required tag has exactly one native/fallback/deferred class.
- The old 23/191 metric is reproducible from the matrix.
- Reports show per-format and per-manufacturer denominators.
- A new mismatch under an existing red tag fails the gate.
- `make verify` passes with the declared corpus mode.

## Out of scope

- Implementing the missing format parsers themselves.
- Expanding Tier 1 based only on available fixtures.
- Replacing the permanent real-ExifTool fallback.
- Claiming full ExifTool parity from configured-tag percentages.

## Files likely involved

- `tests/exiftool_compatibility_tests.rs`
- `src/compat/`
- `tools/generate_exiftool_json.sh`
- `config/supported_tags.json`
- `config/compat_known_gaps.json`
- `docs/required-tags.json`
- new corpus manifest and generated observation report
- CI workflow and Make targets that select the explicit corpus mode
