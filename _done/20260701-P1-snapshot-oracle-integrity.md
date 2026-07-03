# TPP: Snapshot Oracle Integrity

## Summary

`test_exiftool_compatibility` (`tests/exiftool_compatibility_tests.rs`)
is supposed to be our ExifTool-oracle regression gate, but it **never
fails** — it prints a report and returns `Ok` regardless of how many
tags mismatch, are missing, or have the wrong value. That means
"supported per `config/supported_tags.json`" and "actually correct" are
completely decoupled today, and nothing in CI would notice if either
regressed further. This TPP: (1) proves that with a currently-real bug,
(2) makes the test assertive with an explicit, reviewed known-failures
allowlist so legitimate divergences don't just get silently permitted
again, and (3) ties snapshot regeneration to the pinned submodule
version so snapshots can't silently drift from what they claim to test
against.

## Current phase
- [x] Research & Planning
- [ ] Write breaking tests
- [ ] Design alternatives
- [ ] Task breakdown
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Session log (2026-07-02)

- **State changed since authoring**: the version-catchup TPP is DONE
  (commit `f2bdb304`) — snapshots are now regenerated with **13.59**
  (all 379 carry `"ExifToolVersion": 13.59`), and compat stands at
  **43% (83/191): 87 missing, 16 type mismatches, 5 only-in-exif-oxide**
  → Task 5's triage surface is ~104 gaps, expect the allowlist to lean
  on category-level reasons (video-blocked tags, binary-extraction,
  etc.) with references, not 104 hand-written essays.
- Expected snapshot-churn items from the 13.59 bump (from catchup TPP
  triage): FujiFilm `RAFVersion`→`FirmwareVersion`, Pentax
  `AFInfo`→`AFInfoK3III`, Nikon burst-tag rework, composite
  `FocalLength35efl` Description change.
- **Task 1 + the GPSPosition fix are being executed together** (the
  breaking test doubles as program work-item #4's fix gate; fix
  delegated 2026-07-02). The runbook `docs/guides/EXIFTOOL-UPGRADE.md`
  §8–9 already documents regen-with-submodule-exiftool + baseline
  comparison (Task 7's cross-reference target).
- **GPSPosition fix LANDED (review-gated)**: byte-exact vs snapshots on
  both acceptance images; compat 83→84 working, type mismatches 16→15.
  Three root causes: (a) unsigned EXIF coords fed to GPSPosition because
  the signed GPS-module composites lose a name-keyed `COMPOSITE_TAGS`
  registry collision (GPS vs Sony vs QuickTime — follow-up in program
  TPP); (b) Rust default float formatting vs Perl `%.15g`
  (`format_perl_number` added); (c) ExifTool rounds every rational read
  to 10 sig-figs (`GetRational64u`→`RoundFloat`; `round_float` ported).
  Breaking test lives in `tests/snapshot_oracle_tests.rs` (this TPP's
  Task 1 — extend that file for the assertive-oracle work).
- **Review verdicts (codex + orchestrator ground-truth check)**: both
  findings ACCEPTED — (1) zero-denominator rationals must invalidate the
  whole coordinate (`GPS.pm:585` returns `''` on `inf|undef`;
  `GetRational64u` yields those strings), we wrongly treated them as 0.0;
  (2) non-finite Perl casing is `Inf`/`NaN`, not `inf`/`nan`. Fixes +
  pinning tests applied before commit.

## Required reading
- [TDD.md](../docs/TDD.md) — start with a failing test, this TPP's Task 1 IS that test
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md) — section 5 "Allowed deviations" is the
  only legitimate reason a known-failure entry should exist
- [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md) — no bogus guardrails; don't paper over
  real bugs with the allowlist instead of fixing them
- `_todo/P03-implementation-backlog.md` — tracks the actual composite-tag bugs;
  this TPP is about making tests catch bugs like these, not fixing them

## Description

**Bug**: The primary compatibility test compares our output against
ExifTool-generated JSON snapshots, computes categorized differences,
prints them... and then always returns success.

**Reproducing evidence** (verified 2026-07-01, current `HEAD`):

```rust
// tests/exiftool_compatibility_tests.rs:441-457
// Report critical issues but don't fail the test - this is for tracking progress
let critical_failures = ...;
if critical_failures > 0 || dependency_failures > 0 {
    println!("...tracking N compatibility gaps...");
    // no assert!, no panic!, no process::exit — the #[test] fn just returns
} else {
    println!("\n🎉 Perfect ExifTool compatibility achieved!");
}
```

**Why it matters**: `make tests` and `make compat` both run this test
via `compat-test` (Makefile:171). A regression that breaks a previously
working tag produces console noise, not a red CI run. Combined with
snapshots that (by design, correctly) always reflect real ExifTool
truth, this means we can ship known-wrong output indefinitely as long
as nobody reads the test's stdout.

**Root cause**: the test was written as a progress-tracking dashboard
during early development (comment literally says "this is for tracking
progress... failures are expected during development") and was never
converted to a gate once the project matured.

**Fix approach**: make it fail on any undocumented difference, using an
explicit, reviewed allowlist (generalizing the existing but
narrower `get_known_missing_tags`/`EXCLUDED_FILES` mechanism) for
differences that are genuinely acceptable per `TRUST-EXIFTOOL.md`'s
"Allowed deviations" (GPS decimal rendering, internal ExifTool
inconsistencies) or genuinely tracked as a known bug with its own
backlog entry.

## Tribal knowledge

### A currently-real bug to use as the breaking-test example

Composite `Megapixels`/`ShutterSpeed` bugs cited in earlier planning
(`P03-implementation-backlog.md`, dated 2025-12-11) **appear to already
be fixed** — verified 2026-07-01 with
`cargo run --bin compare-with-exiftool -- test-images/apple/apple_iphone_13_pro_11.dng`,
both values match the snapshot exactly. **Don't use them as your
breaking-test target — they'd fail to reproduce and burn time.**

`Composite:GPSPosition` is still broken right now and is a good target:

```bash
cargo run --bin compare-with-exiftool -- test-images/apple/iphone_13_pro.jpg
# 🔄 Different values (1):
#   Composite:GPSPosition: ExifTool=Some(String("40.5935972222222 -122.38015")),
#                          exif-oxide=Some(String("40.59359722222222 122.38015"))
```

Two bugs in one: exif-oxide drops the negative sign on
`GPSLongitude` inside the combined `GPSPosition` string, and renders one
extra digit of precision. `Composite:GPSPosition` is listed in
`config/supported_tags.json:31`, so `test_exiftool_compatibility`
already sees this mismatch every run — it just doesn't fail on it. This
is exactly the shape of bug this TPP's Task 1 test should catch.

### The test infrastructure already has half of a known-failures mechanism

`tests/exiftool_compatibility_tests.rs` has `EXCLUDED_FILES` (whole-file
skip list), and `get_known_missing_tags()` /
`remove_known_missing_tags()` (per-file, per-tag removal for
"documented missing features," currently only used for one Panasonic
RW2 case, and that one is actually dead — its condition starts with
`if 1 > 2 && ...` so it never fires; grep it yourself to confirm before
assuming it's live). **Don't build a parallel mechanism** — generalize
this one: extend it to (a) always fire when its condition is true, (b)
cover value-mismatches, not just missing-tags, (c) require a reason
string and a reference (backlog TPP or ExifTool file:line) per entry so
it can't become a silent dumping ground, and (d) make the test **fail**
when a real difference isn't in the allowlist.

### Snapshot/submodule version skew is a separate, currently-unguarded risk

`tools/generate_exiftool_json.sh` shells out to whatever `exiftool` is
first on `$PATH` (`command -v exiftool`, then `exiftool -ver` is only
echoed, never checked against anything). Nothing ties the 379 files in
`generated/exiftool-json/*.json` to a specific ExifTool version — if a
snapshot was generated with a stray system ExifTool install that
differs from the pinned submodule (`third-party/exiftool`), tests would
silently compare against the wrong oracle. Every generated JSON snapshot
already carries `"ExifToolVersion"` in its top-level object (from `-j`
output) — use that.

```bash
# ground truth for "what version should snapshots be generated with":
grep -n '^\$VERSION' third-party/exiftool/lib/Image/ExifTool.pm
# spot check an existing snapshot:
grep '"ExifToolVersion"' generated/exiftool-json/test_images_apple_IMG_3755_JPG.json
```

### Corpus size (verified 2026-07-01)

`tools/generate_exiftool_json.sh` scans `test-images/` (328 files
currently, `make pull-test-images` fetches from Backblaze B2) plus
`third-party/exiftool/t/images` (193 files), filtered by
`SUPPORTED_EXTENSIONS` and `config/supported_tags.json` (274 tags).
Current snapshot count: `find generated/exiftool-json -name '*.json' | wc -l` → 379.

## Solutions

### Option A (preferred): Make the test assertive with a structured allowlist

Add a `known_failures.json` (or extend the existing const arrays in the
test file — see DRY note below) keyed by `(tag, file_pattern)` with a
mandatory `reason` field. `test_exiftool_compatibility` fails
(`panic!`/`assert!`) if any difference isn't matched by an allowlist
entry, and *also* fails if an allowlist entry no longer reproduces
(stale entries rot silently otherwise — this is the same class of bug
as the dead `if 1 > 2` condition found above). Add a small `make`
target or a second test that asserts the pinned submodule's `$VERSION`
matches the `ExifToolVersion` recorded in every snapshot file, failing
loudly on skew.

**Pros**: turns a dashboard into a real gate; stale-entry detection
prevents the allowlist from becoming a new "known missing tags never
actually fires" trap; version-skew guard is cheap and catches an entire
class of oracle corruption.
**Cons**: first PR will have to triage every current real difference
(missing tags, value mismatches) into either "fix it" or "allowlist with
reason" — expect this to surface more than just the GPSPosition bug.

### Option B: Separate "strict" test that's allowed to fail, keep the dashboard test as-is

Add a new `#[test]` that asserts, leave `test_exiftool_compatibility`
untouched as a non-blocking report.

**Why Option A is better**: leaves the exact same landmine in place —
whoever reads test output has to know to look at the *other* test.
`SIMPLE-DESIGN.md` Rule 4 (fewest elements) argues for fixing the one
test in place rather than running two similar ones forever. Only fall
back to Option B if triaging the current real differences turns out to
be large enough to block unrelated work — in that case, ship Option B
temporarily with a tracking issue to consolidate.

## Tasks

- [ ] **Task 1: Write the breaking test.** Add a focused test (e.g. in
      `tests/exiftool_compatibility_tests.rs` or a new
      `tests/snapshot_oracle_tests.rs`) that runs
      `compare-with-exiftool`-equivalent logic against
      `test-images/apple/iphone_13_pro.jpg` and asserts
      `Composite:GPSPosition` matches ExifTool's snapshot value exactly.
      **Proof it fails for the right reason**: `cargo t
      <new_test_name>` fails with a message showing the sign/precision
      mismatch, not a setup error (missing file, etc).

- [ ] **Task 2: Confirm current state.** Run `make compat-test 2>&1 |
      tail -40` and confirm it exits 0 despite printed differences (the
      root-cause bug). **Proof**: paste the exit code and the "Perfect
      ExifTool compatibility" / "tracking N compatibility gaps" line
      showing the contradiction.

- [ ] **Task 3: Design the allowlist schema.** Decide JSON file vs.
      Rust const (lean toward JSON for reviewability + reason strings;
      check if `config/` already has a place for this). Include: tag
      name, file pattern (or "all"), reason, and a reference (this
      TPP, a `_todo/` backlog entry, or a `TRUST-EXIFTOOL.md` section).
      **Proof**: schema documented in this TPP or a short design note,
      reviewed against `TRUST-EXIFTOOL.md`'s allowed-deviations list so
      it doesn't become a bug dumping ground.

- [ ] **Task 4: Make `test_exiftool_compatibility` assertive.** Replace
      the `println!`-only branch (lines ~441-457) with logic that
      loads the allowlist, subtracts matched entries, and
      `panic!`s/`assert!`s on anything left over. Also assert every
      allowlist entry actually reproduced (stale-entry detection).
      **Proof**: `cargo t test_exiftool_compatibility` fails before the
      allowlist is populated with real current gaps, passes after.

- [ ] **Task 5: Triage current real differences into fix-or-allowlist.**
      Run the now-assertive test, get the full list of failures, and
      for each: either fix it (small ones), or add a justified allowlist
      entry citing `TRUST-EXIFTOOL.md` or a backlog TPP. The GPSPosition
      bug from Task 1 either gets fixed here or gets an allowlist entry
      pointing at `_todo/P03-implementation-backlog.md`.
      **Proof**: `cargo t test_exiftool_compatibility` passes cleanly.

- [ ] **Task 6: Version-skew guard.** Add a test or `make` target
      that reads `third-party/exiftool/lib/Image/ExifTool.pm`'s
      `$VERSION` and asserts it matches the `ExifToolVersion` field in
      every file under `generated/exiftool-json/`.
      **Proof**: passes today (both should be 13.43); manually edit one
      snapshot's version field and confirm the guard fails.

- [ ] **Task 7: Automate regeneration on submodule bump.** Wire
      snapshot regeneration into whatever process bumps the submodule
      (see `_todo/20260701-P0-exiftool-version-catchup.md`) — at
      minimum, document in `docs/guides/EXIFTOOL-UPGRADE.md` (created by
      that TPP) that `make compat-gen-force` must run and Task 6's guard
      must pass before merging a version bump.
      **Proof**: cross-reference exists in both TPPs.

- [ ] **Task 8: Final validation.** `make codegen fmt lint t` clean;
      `make compat` (compat-gen + compat-test + test-mime-compat)
      passes with exit code 0. Move to `_done/`.

## Files referenced

- `tests/exiftool_compatibility_tests.rs:22` — `EXCLUDED_FILES`
- `tests/exiftool_compatibility_tests.rs:196-215` — `get_known_missing_tags`
  (note the dead `if 1 > 2` condition)
- `tests/exiftool_compatibility_tests.rs:441-458` — the non-assertive
  report branch this TPP replaces
- `tests/enhanced_exiftool_group_compatibility_tests.rs` — a *different*
  test file that already uses real `assert!`s (for Group0/1/2 hierarchy,
  not value correctness) — good pattern reference for Task 4
- `src/compat/normalization.rs`, `src/compat/filtering.rs`,
  `src/compat/comparison.rs`, `src/compat/reporting.rs` — comparison
  infrastructure the assertive test will reuse
- `config/supported_tags.json:31` — `Composite:GPSPosition` entry
- `tools/generate_exiftool_json.sh` — snapshot generator (version-pin gap)
- `generated/exiftool-json/*.json` (379 files) — the snapshots themselves
- `Makefile:162-225` — `compat-gen`, `compat-test`, `compat`, `compat-full` targets
- `_todo/P03-implementation-backlog.md` — composite-tag bug tracking (do not duplicate)
