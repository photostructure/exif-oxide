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
- [x] Write breaking tests
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation (2026-07-03, opus subagent + orchestrator fixes)
- [x] Review & Refinement (adversarial review; 5 findings, all resolved)
- [x] Final Integration (gate green: 23 working / 168 allowlisted / 0 unexpected)

## Review outcome (2026-07-03) — what the gate looks like now

- **Option A shipped**: `test_exiftool_compatibility` is a hard gate.
  Every non-Working tag (all 5 categories) must be in
  `config/compat_known_gaps.json` (grouped: reason + reference + tags;
  duplicate tag across groups = config error); every allowlisted tag must
  still reproduce (stale-entry ratchet) or the test fails telling you to
  remove it. `TAGS_FILTER` skips assertions (debug mode);
  `COMPAT_DUMP_GAPS=1` emits the machine-readable gap list.
- **Headline metric changed meaning — do not read 23/191 as a regression
  from 94/191.** The old aggregation kept the *first-seen* per-tag state
  over unsorted read_dir order, so a tag Working in one early file masked
  failures everywhere else. Now a tag is Working only if it matches in
  every file that carries it. Same binary, same snapshots: 94/191 under
  the old (broken) counting = 23/191 under honest counting, 168 gap tags,
  each allowlisted with a root cause. Newly *visible* (not newly broken):
  HEIC EXIF extraction gap (IMG_9757.heic: 0 of 32 supported EXIF tags
  extracted — 33 allowlisted tags), Nikon Z-series NEF misdetected as NRW
  (z_5_2/z_6_3/z_8 report FileType NRW; D-series fine).
- **Version-skew guard**: generator hard-codes the vendored
  `third-party/exiftool/exiftool`, writes the committed
  `generated/exiftool-json/.exiftool-version` marker ONLY on a fully
  successful `--force` run, and incremental runs abort on marker mismatch
  ("Run: make compat-gen-force"). `test_snapshot_exiftool_version_matches_submodule`
  asserts marker == `ExifTool.pm` `$VERSION`. Proven: marker→13.53 fails
  both the script (exit 1) and the test with actionable messages.
- **Adversarial review findings (all verified empirically, then fixed)**:
  (F1, HIGH) marker was written on every run — an incremental
  `make compat-gen`/`make compat` after a bump would stamp the new version
  over stale snapshots, silently defeating the guard → fixed as above;
  (F2) marker wasn't committed → fresh clones would panic → committed;
  (F3) machines without the B2 `test-images/` corpus (160/168 allowlist
  entries anchor there) would false-fire the stale ratchet with advice to
  delete correct entries → ratchet now skipped with a "run
  `make pull-test-images`" note when any source image is missing (the
  unexpected-gap check still runs — missing files can only under-report);
  (F4) Composite:LensID "A or B" ordering for tied Nikon lens candidates
  is genuinely nondeterministic (same exiftool, 8 runs, both orders — Perl
  hash-order feeding `join ' or '`), so the 3 churned d3500 snapshots were
  reverted; harmless to the gate (tag is an allowlisted Missing gap, value
  never compared) but expect random churn on future `compat-gen-force`;
  (F5) HEIC group reason overstated the count (43→"0 of 32") → corrected.
- **Not done, deliberate**: CI does not run `make compat-test` (no
  workflow references it) — wiring it into CI needs the corpus story
  solved first (B2 sync in CI or t/images-only mode).

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

## Session log (2026-07-03, design pass)

- **Task 2 DONE**: `make compat-test` exits **0** while printing
  "Tracking 92 compatibility gaps: 87 missing, 0 dependency failures,
  5 type mismatches" at 94/191 (49%) — contradiction confirmed on
  current HEAD (`727c5ecc`). Plus 5 only-in-exif-oxide → 97 gap tags.
- **TPP premise correction (Task 6)**: snapshots do **not** carry
  `ExifToolVersion` — the jq filter in `tools/generate_exiftool_json.sh`
  strips everything not in `supported_tags.json`... and while
  `ExifTool:ExifToolVersion` is listed there, 0/379 snapshot files
  contain it (raw `-G` output key survives filtering only if emitted;
  verified by grep). Revised design: (a) generator must invoke the
  **vendored** `third-party/exiftool/exiftool`, not PATH lookup —
  Matthew's machine has 13.53 on PATH vs submodule 13.59, a live skew
  that would have poisoned any missing snapshot on today's run; (b)
  generator writes `generated/exiftool-json/.exiftool-version`; (c) a
  test in `tests/exiftool_compatibility_tests.rs` (so `make compat-test`
  runs it) asserts marker == `$VERSION` in
  `third-party/exiftool/lib/Image/ExifTool.pm:32`.
- **Task 3 design (allowlist)**: `config/compat_known_gaps.json`,
  grouped schema: `{"groups": [{"reason", "reference", "tags": [...]}]}`.
  Union of tags = allowlist; duplicate tag across groups = config error
  = test failure. Test fails on (1) any non-Working tag not allowlisted
  (regression/new gap), (2) any allowlisted tag now fully Working
  (stale entry — remove it, ratchet the number up). Assertions skipped
  when `TAGS_FILTER` is set (debugging mode).
- **Aggregation fix folded in**: current loop keeps the *first-seen*
  state per tag over unsorted `read_dir` order — a tag Working in an
  early file masks a regression in a later file, nondeterministically.
  Fix: sort snapshot list; non-Working state always beats Working.
  This may honestly reclassify some "working" tags (rate could dip
  below 94/191); allowlist absorbs them with reasons.

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

- [x] **Task 1: Write the breaking test.** DONE 2026-07-02 via the
      GPSPosition fix (`tests/snapshot_oracle_tests.rs`, commit `141c4167`).

- [x] **Task 2: Confirm current state.** DONE 2026-07-03: exit code 0
      alongside "⚠️ Tracking 92 compatibility gaps: 87 missing tags, 0
      dependency failures, 5 type mismatches" at 94/191.

- [x] **Task 3: Design the allowlist schema.** DONE — grouped JSON in
      `config/compat_known_gaps.json` (see design session log above);
      loader/validator in `src/compat/known_gaps.rs` with unit tests.

- [x] **Task 4: Make `test_exiftool_compatibility` assertive.** DONE.
      **Proof**: removing `EXIF:Make` from the allowlist fails with
      `❌ 1 tag(s) diverge … NOT in the allowlist`; adding a bogus tag
      fails with the stale-entry ratchet message; restored config passes.

- [x] **Task 5: Triage differences into fix-or-allowlist.** DONE — 168
      tags in 14 root-cause groups, each with reason + TPP reference; no
      untriaged catch-all needed. Spot-verified by orchestrator +
      adversarial reviewer (HEIC, NEF-as-NRW, video group, type
      mismatches). GPSPosition itself was fixed back in Task 1.

- [x] **Task 6: Version-skew guard.** DONE (revised design — snapshots
      never carried `ExifToolVersion`; see session log):
      `.exiftool-version` marker + `test_snapshot_exiftool_version_matches_submodule`
      + incremental-generation abort on mismatch. **Proof**: marker set
      to 13.53 → script exits 1, test fails with skew message; restored.

- [x] **Task 7: Regeneration on submodule bump.** DONE —
      `docs/guides/EXIFTOOL-UPGRADE.md` §8/§9 mandate `make
      compat-gen-force` per bump and describe both gates; the generator
      itself enforces it (incremental aborts on skew).

- [x] **Task 8: Final validation.** DONE 2026-07-03: `cargo fmt` +
      `make lint` clean; full `cargo t` green (50 suites, subagent) and
      compat + known_gaps suites re-run green after review fixes;
      `make compat-test` exit 0 (gate passed: 23/168/0). `make codegen`
      deliberately not run — no codegen inputs changed. Moved to `_done/`.

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
