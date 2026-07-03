# TPP: ExifTool Version Catch-up (Fire Drill) + Upgrade Runbook

## Summary

The vendored ExifTool submodule is 16 releases behind upstream (v13.43 →
v13.59, Dec 2025 → May 2026). There is no documented process for bumping
it. This TPP is a deliberate "fire drill": do the bump once, by hand,
and write down what it teaches as `docs/guides/EXIFTOOL-UPGRADE.md` so
the next bump (and eventually a CI job) isn't archaeology. Upstream
shipped 4 security-related releases (13.50, 13.53, 13.54, 13.59)
in this window, which is itself a reason not to defer this further.

## Current phase
- [x] Research & Planning
- [x] Write breaking tests (n/a — compat suite + build are the oracle)
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation (in progress 2026-07-02)
- [x] Review & Refinement
- [x] Final Integration

## Session log (2026-07-02)

- **Baseline (Task 1)**: `make compat-test` on 13.43 snapshots: **42%
  (82/191 tags), 87 missing, 17 type mismatches, 5 only-in-exif-oxide**;
  full report saved at session scratchpad `compat-baseline-13.43.txt`.
  Submodule was clean; parent gitlink lagged one fork commit (`eb2279b9`,
  IMAGE_DATA_HASH.md doc — Matthew's, fold into this TPP's commit).
- **DEVIATION from Option A**: bumped straight to 13.59 (Option B) instead
  of 4-release hops. Rationale: (1) the fork tracks upstream via *merges*
  on its `docs` branch, so each hop = a merge commit + fork push — 4× the
  ceremony; (2) triage is per-module over the full range anyway (subagent
  fan-out), and per-hop `git diff` remains available for bisection via
  upstream tags; (3) read-only scope eliminates the two scariest
  procedural diffs (Geotag.pm interpolation, Writer.pl) outright.
- **Bump mechanics that worked**: submodule sits on fork branch `docs`;
  `git merge 13.59 -m "Update to 13.59"` conflicts massively because
  older fork updates were squash-style (ancient merge-base). Resolution:
  `git checkout --theirs -- . && git add -A && git commit` — then VERIFY
  with `git diff --stat 13.59 HEAD` = fork-only files (doc/, CLAUDE.md,
  .claude/) and `$VERSION` grep = 13.59.
- **Perl env rot**: `make codegen` "succeeded" (exit 0!) while every
  field_extractor.pl invocation died on missing JSON::XS — XS modules
  were wiped by a system perl upgrade; pure-Perl survived. Fixed via
  `~/perl5/bin/cpanm -l ~/perl5 --notest JSON::XS Params::Util`; cpanfile
  updated to declare JSON::XS + PPI. **Runbook items**: (a) check codegen
  log for WARN lines — exit code is useless; (b) verify perl deps first:
  `perl -I ~/perl5/lib/perl5 -MJSON::XS -MPPI -e1`.
- **Real codegen churn**: 114 generated files, +2241/−1428.
- Codegen's submodule patches are `my`→`our` PLUS glob-alias blocks
  ("Ensure the field_extractor can see our exported fields") — both
  mechanical. Discard with `git -C third-party/exiftool checkout -- .`
  only AFTER all codegen runs are done and BEFORE final snapshot regen.
- **Two codegen bugs found by the bump** (fix in codegen/src, delegated):
  1. Exif.pm 13.59 added `%use` hash (C2PA AI-usage) → emitted
     `pub mod use;` — `to_snake_case` (codegen/src/strategies/
     output_locations.rs:91) lacks a Rust-keyword guard.
  2. Nikon.pm 13.59 `FirmwareVersion56` ValueConv
     (`$val =~ s/(\d{2})/$1./; $val`) → tag_kit emitted a reference to
     `ast_value_37b97a54731266fa` that the function generator never wrote
     (E0432). Contract violation: tags must only reference successfully
     generated functions; degrade to None like other untranslatable
     expressions.
- **Diff triage (Tasks 5-6) COMPLETE — zero PORT-REQUIRED items.** Three
  subagents classified every hunk in every module codegen processes
  (reports: session scratchpad `triage-{core,mfr,fmt}.md`; spot-verified).
  All procedural read-path changes fall on unimplemented paths (Canon
  PrintLensID/ColorData dispatch, Sony Tag94xx stubs, Nikon Z6III
  AlwaysDecrypt, QuickTime/Matroska/DJI/MacOS — all fall to ExifTool
  fallback tier) or are neutralized by our infrastructure (ExifTool's
  UCS2→UTF16 migration: we already use surrogate-aware
  `String::from_utf16`, src/implementations/raw_conv.rs:74; ExifTool's
  XMP CDATA scanner fixes: we use quick-xml NsReader,
  src/xmp/processor.rs:381). Watch items for the snapshot-oracle TPP:
  tag renames FujiFilm `RAFVersion`→`FirmwareVersion`, Pentax
  `AFInfo`→`AFInfoK3III`, Nikon burst-tag rework, and composite
  `FocalLength35efl` Description change.

## Required reading
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md) — translate exactly, cite file:line
- [TDD.md](../docs/TDD.md) — this isn't a bug fix, but "Final Integration" still requires `cargo t` clean
- [CODEGEN.md](../docs/CODEGEN.md) — how `make codegen` extracts tables
- [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md) — never manually port table data; use codegen
- `docs/EXCLUDED-TAGS.md`, `docs/reference/SUPPORTED-FORMATS.md` — scope of what must keep working

## Description

`third-party/exiftool` is a git submodule with two remotes: `origin`
(`git@github.com:photostructure/exiftool.git`, our fork) and `upstream`
(`https://github.com/exiftool/exiftool.git`, the real Phil Harvey repo).
Bumping means: fast-forward the submodule to a newer upstream tag,
re-run `make codegen` to regenerate `src/generated/**`, then manually
port any *procedural* (non-tabular) logic changes that codegen can't
capture — new parsing algorithms, new heuristics, new special-case
branches. The output of this TPP is both the ported logic and a runbook
so this becomes routine instead of a project.

## Tribal knowledge

### The version-gap number is easy to get wrong — verify it yourself

`git submodule status` and plain `git describe` report **`11.18-265-g3a79a582`**,
which looks like we're on ancient v11.18. **This is wrong and misleading.**
Verified 2026-07-01:

```bash
# git describe (default) only matches ANNOTATED tags. 11.18 is the only
# annotated tag in this repo's history; every 12.x/13.x tag is lightweight.
git -C third-party/exiftool cat-file -t 11.18   # -> "tag"    (annotated)
git -C third-party/exiftool cat-file -t 13.43   # -> "commit" (lightweight)

# --tags considers lightweight tags too, and gets much closer:
git -C third-party/exiftool describe --tags     # -> 13.38-38-g3a79a582

# Ground truth is the $VERSION string in the vendored source itself:
grep -n '^\$VERSION' third-party/exiftool/lib/Image/ExifTool.pm
# $VERSION = '13.43';
```

So the pinned submodule is **actually v13.43** (released Dec 4, 2025 per
`third-party/exiftool/Changes`), not v13.10 and not v11.18. **Always read
`$VERSION` from `ExifTool.pm` to check the real version — never trust
`git describe`/`git submodule status` output for this repo.** Put this
gotcha in the runbook; it will confuse the next engineer too.

### The actual gap, verified via the RSS feed and a real diff

```bash
# Latest tags on the true upstream (not our photostructure fork):
git -C third-party/exiftool ls-remote --tags upstream \
  | grep -oE 'refs/tags/13\.[0-9]+$' | sed 's#refs/tags/##' | sort -t. -k2 -n | tail -5
# 13.56 13.57 13.58 13.59  <- 13.59 is latest as of 2026-07-01
```

Release dates (from `https://exiftool.org/rss.xml`): 13.59 = May 27,
2026; our pinned 13.43 = Dec 4, 2025. **That's 16 releases and ~5.8
months behind**, not 42 releases / 11 months — the larger number in
earlier planning docs came from the same `git describe` confusion above
(mistaking `11.18-265` for a real version distance). Security-relevant
releases in the gap: 13.50 ("Security update, MacOS only"), 13.53
("Security update, Windows only"), 13.54 ("Security update"), 13.59
("Security update") — changelog entries give no CVE/technical detail,
but that's still 4 releases we're missing security fixes from.

### Confirmed: real procedural-logic changes exist in the gap, not just tables

The submodule already has an `upstream` remote configured — fetch just
the tags you need (does not touch the working tree, does not move
`HEAD`, safe to run):

```bash
git -C third-party/exiftool fetch upstream tag 13.59 --no-tags
git -C third-party/exiftool diff --stat HEAD..13.59 -- \
  lib/Image/ExifTool/Canon.pm lib/Image/ExifTool/QuickTime.pm lib/Image/ExifTool/Geotag.pm
#  Canon.pm     | 272 +++++++++++++++++++++++++++++++--------
#  Geotag.pm    | 229 +++++++++++++++++++++++++++------
#  QuickTime.pm | 118 +++++++++++++----
```

Verified concrete example — a new Canon teleconverter ("extender")
detection regex was added to `Canon.pm` in this window:

```bash
git -C third-party/exiftool diff HEAD..13.59 -- lib/Image/ExifTool/Canon.pm | grep -i extender
# +        @tcs = ( $3 ) if $lensModel =~ / \+ ((EXTENDER )?RF)?(\d+(\.\d*)?)x\b/;
```

Geotag.pm's diff touches the linear-interpolation-around-fixes logic
(the algorithm that estimates GPS position between track-log fixes,
`Geotag.pm` around line 1104-1140 today). In just the Canon.pm diff,
roughly 152 of 223 added lines (~68%) are `key => value` table-data
lines (`grep -c '^+.*=>'`); the rest is procedural. Don't assume this
68/~80% split holds file-by-file — QuickTime.pm and Geotag.pm are
logic-heavy modules and will skew lower. **Do this same `git diff
--stat` + targeted grep exercise for every changed file under
`config/exiftool_modules.json`'s module list before assuming "codegen
handles it."**

**IMPORTANT — clean up after yourself**: the `git fetch upstream tag
...` command above only adds refs/tags to the submodule's local git
objects; it's safe and doesn't dirty `git status` in the parent repo.
Do NOT run `git checkout`/`reset`/`add` inside `third-party/exiftool` —
per `CLAUDE.md` it's a submodule and those commands must be coordinated
with the user.

### Pre-existing landmine found during this research (not caused by this TPP)

As of 2026-07-01, `third-party/exiftool` already has an uncommitted
dirty working tree (`git -C third-party/exiftool status --porcelain`
shows ~20+ modified `.pm` files, e.g. `Exif.pm`, `Canon.pm`, `GPS.pm`).
CLAUDE.md notes codegen "may temporarily patch ExifTool files, but
these changes should be reverted automatically" — this looks like a
revert that didn't happen (possibly from a concurrent session). **Before
starting Task 1, check `git -C third-party/exiftool status --porcelain`
is clean and ask the user how to handle it if not** — don't silently
blow away someone else's in-progress patch, and don't silently proceed
on top of a dirty submodule either.

### Codegen mechanics you'll rely on

- `make codegen` runs `codegen/` which invokes `field_extractor.pl`
  against the modules listed in `config/exiftool_modules.json` and
  regenerates everything under `src/generated/`.
- `src/generated/**` is 100% generated — never hand-edit it (instant PR
  rejection per `CLAUDE.md`). If codegen output looks wrong after the
  bump, fix `codegen/src/` or the module list, not the generated file.
- `make compat-gen-force` regenerates `generated/exiftool-json/*.json`
  snapshots using whatever `exiftool` binary is first on `$PATH` — this
  is your **system** ExifTool, which is unrelated to the submodule
  version. For this TPP, snapshots must be regenerated using the
  **newly bumped submodule's** `exiftool` script (or a system install of
  the same version) — see `_todo/20260701-P1-snapshot-oracle-integrity.md`
  for the general invariant this depends on.

- **Codegen bug fixes landed** (subagent, verified by me):
  1. Keyword guard in `to_snake_case`
     (codegen/src/strategies/output_locations.rs) — `use` → `use_`, full
     strict+reserved keyword list, unit + doc tests.
  2. `SimpleTableStrategy::can_handle` claiming rule fixed
     (codegen/src/strategies/simple_table.rs:243): fractional keys no
     longer override tag-table markers. Verified mechanism (fixer's
     report, spot-checked): MakerNotes0x56's new `4.1` keys flipped it
     TagKit→SimpleTable; TagKit stopped regenerating the tags file, and
     since **codegen never deletes stale output**, the committed
     `maker_notes0x56_tags.rs` kept importing a function that no longer
     got emitted → E0432. The same bug had 37 ProcessBinaryData tables
     (FujiFilm/H264/Nikon FlashInfo*/PanasonicRaw/Pentax/QuickTime/RIFF/
     Sony) shipping as GARBAGE simple-table lookups (only
     CHECK_PROC/WRITE_PROC/NOTES strings, zero real tags — verified via
     `git show HEAD:src/generated/Nikon_pm/flash_info0103.rs`). All 37
     now regenerate as real `*_tags.rs` tag kits; old orphans deleted;
     none were referenced by hand-written code. The FirmwareVersion56
     ValueConv lands as the standard `missing_value_conv` passthrough
     placeholder... which the paren-bug fix below may upgrade to a real
     implementation.
- **Follow-up candidate (fixer recommendation, endorsed): codegen should
  clean/prune `src/generated/`** — stale files from reclassified or
  removed tables are what turned both bugs into build breaks. Without
  cleanup, every future strategy change risks the same. Small TPP-worthy.

## Post-review fixes (review gate, 2026-07-02)

Reviewers: codex (read-only sandbox) + Claude subagent, identical scope;
orchestrator as third reader. Verdicts, with evidence:

- **VETOED — codex High "tag_kit drops fractional subtag keys (Nikon
  4.1–4.4 Burst tags)"**: true observation, wrong severity. tag_kit
  (codegen/src/strategies/tag_kit.rs:450) has never parsed fractional
  keys for ANY ProcessBinaryData table; the Burst subtags are new in
  13.59 (never extracted at baseline → no regression), absent from
  `config/supported_tags.json` (verified grep = 0 hits), and served by
  the ExifTool fallback tier. Known limitation, recorded here; do not
  re-litigate. If bit-field subtags ever enter supported scope, tag_kit
  needs fractional-key support.
- codex verified clean: no tables claimed by neither strategy; deleted
  generated files all have `*_tags.rs` replacements + consistent mod.rs;
  cpanfile; runbook commands/paths; keyword-guard cross-references.
- Claude reviewer: **NO FINDINGS** (empirically verified: single
  choke-point name derivation incl. live `use_` case end-to-end;
  truth-tabled the claiming change — only mixed-key+marker tables moved,
  all 37 paired with replacements, zero dangling mod.rs refs; none of
  the deleted modules were ever consumed by hand-written src/; cpanfile
  deps genuine; runbook paths/targets all exist; generated JSON diffs are
  version bumps only). Cosmetic note only: `USE__DATA` double underscore
  (valid, no collision — left as-is). Keyword list over-broad on
  contextual keywords (`union`/`gen`/`try`) — harmless, kept for safety.
- Fourth reviewer signal: clippy caught the PPI paren-drop (see above).
  **Fix landed** (codegen/src/ppi/rust_generator/visitor.rs): `visit_list`
  unwrapped single-element groupings and infix branches rendered bare, so
  explicit source parens vanished. Fix re-wraps a `PPI::Structure::List`
  operand in the arithmetic/numeric-comparison branches when its single
  child is a compound infix construct; skips atoms and self-delimiting
  power()/concat() children (no redundant parens). 6 new full-pipeline
  tests (codegen/tests/paren_precedence_test.rs, TDD red→green). Regen
  healed **17 miscompiled functions** (Canon ExposureTime/FlashOutput,
  Sony BatteryTemperature, PanasonicRaw Gamma, ...) — every diff a pure
  paren-addition. eq_op/identity_op eliminated with zero new clippy
  errors (delta proven exactly −3 via buggy/fixed A-B run). String-op
  (`eq`/`lt`) and `&&`/`||` infix branches NOT extended — zero real-corpus
  cases; same helper applies trivially if one appears.
- **Pre-existing lint rot (NOT this TPP)**: `make lint` at HEAD already
  fails with 75 clippy errors (proven in a pristine HEAD worktree —
  toolchain upgrade added `from_ref`/`is_multiple_of` lints; plus dead
  consolidated normalizer structs). Fixed as an immediate follow-up
  commit after this TPP's, so the gate is green again.
- **Snapshots regenerated with 13.59** (`PATH` prepend trick, 379 files,
  `ExifTool version: 13.59` in generator header).
- **Post-bump compat (Task 9): 43% (83/191) vs 42% (82/191) baseline** —
  no regression, one type mismatch resolved (17→16).
- **Fork pushed**: `origin/docs` now carries the "Update to 13.59" merge.
- **THIRD codegen bug — PPI transpiler drops parentheses** (found by
  `make lint`: clippy `eq_op`/`identity_op` fired on degenerate literal
  arithmetic): `1 / (1 + $val/32768)` (PanasonicRaw DistortionScale) →
  `1/1 + val/32768`, and `($val - 1) * 3` (H264 Gain) → `val - 1*3`.
  Pre-existing flattening bug, newly *exercised* because the
  simple_table rerouting fix made these tables' ValueConvs transpile for
  the first time. Extent unknown beyond the clippy-detectable degenerate
  cases — fix delegated (minimal, TDD, maintenance-mode rules). Lesson
  for the runbook: clippy on generated code is a real semantic oracle —
  do NOT blanket-allow `eq_op`/`identity_op`.
- **Runbook written**: `docs/guides/EXIFTOOL-UPGRADE.md` (~210 lines,
  from this session's actual experience, incl. both codegen bugs and the
  exit-0 gotchas).

## Solutions

### Option A (preferred): Stage the bump in ~4-release increments

Bump 13.43 → 13.47 → 13.51 → 13.55 → 13.59 (or similar), running
`make codegen && cargo build && make compat-test` after each hop. If a
hop's diff is small (mostly tables), fold it into the next; if a hop
introduces a large procedural change (like the Canon extender regex),
stop and port it before continuing. This keeps any single diff you're
triaging small enough to review carefully, and gives you a bisectable
trail if something regresses.

**Pros**: small, reviewable diffs; a bad hop is easy to isolate; partial
progress is committable even if later hops stall.
**Cons**: more `make codegen` + `cargo build` + `make compat-test` cycles
(slower wall-clock); tag-data-only hops feel like overhead.

### Option B: One big jump straight to 13.59

Bump directly, run codegen once, triage the entire accumulated diff at
once.

**Why Option A is better**: 369 files changed across the full gap (`git
diff --stat HEAD..13.59 --shortstat` inside the submodule after
fetching the tag). Triaging that in one pass risks missing procedural
changes buried in a huge diff — exactly the kind of "PPI/generated code
looks fine so I didn't check the .pm diff" mistake `ANTI-PATTERNS.md`
warns about. Only use Option B if Option A's first hop shows the whole
gap is overwhelmingly tabular.

## Tasks

All tasks complete 2026-07-02 (see Session log for proofs; Tasks 2-7
executed as a single 13.59 jump per the DEVIATION note):

- [x] **Task 1: Baseline.** Record current compat pass rate before
      touching anything: `make compat-test 2>&1 | tail -30` (note
      working/differences counts from the printed report). Confirm
      submodule working tree is clean (see landmine note above).
      **Proof**: baseline numbers pasted into this TPP's handoff notes.

- [x] **Task 2: Bump submodule (first increment).** From repo root:
      `git -C third-party/exiftool fetch upstream tag <target>`, then
      checkout that tag as a detached ref and hard-set the submodule
      gitlink — coordinate exact commands with the user first since
      this touches the submodule pointer (CLAUDE.md restriction).
      **Proof**: `git -C third-party/exiftool describe --tags` (or the
      `$VERSION` grep) shows the new target version.

- [x] **Task 3: Regenerate.** `make codegen && cargo build 2>&1 | tail -50`.
      **Proof**: build succeeds; `git status src/generated/` shows the
      expected churn (new tables, renamed lookups) and nothing manually
      edited.

- [x] **Task 4: Regenerate compat snapshots with the NEW exiftool.**
      Use the just-bumped submodule's own `third-party/exiftool/exiftool`
      script (not whatever's on `$PATH`) to run
      `tools/generate_exiftool_json.sh --force`, or update `$PATH`
      first — confirm which the script actually invokes
      (`command -v exiftool` inside the script).
      **Proof**: `generated/exiftool-json/*.json` files' timestamps
      update; spot-check one file's `ExifToolVersion` field matches the
      new version.

- [x] **Task 5: Triage the diff.** For each `.pm` file that changed
      (`git -C third-party/exiftool diff --stat <old>..<new>`), classify
      as table-data-only (codegen handles it, no action) vs procedural
      (needs a manual port). Use the grep-for-`=>`-density trick above
      as a fast first pass, then read the actual diff for anything with
      a low ratio. **Proof**: a checklist in this TPP (or a follow-up
      doc) listing every changed `.pm` file with its classification.

- [x] **Task 6: Port required logic.** For each procedural change
      identified in Task 5, port it into the relevant `src/` module
      (never into `src/generated/`) with an ExifTool file:line citation
      per `TRUST-EXIFTOOL.md`. Write/extend a unit or compat test per
      `TDD.md` before considering a port done.
      **Proof**: `cargo t <relevant test>` passes; `compare-with-exiftool`
      shows no new regressions on affected files.

- [x] **Task 7: Repeat Tasks 2-6 for remaining increments** until at
      v13.59 (or the then-current latest — re-check
      `ls-remote --tags upstream` since more releases may have landed
      during this work).

- [x] **Task 8: Write the runbook.** `docs/guides/EXIFTOOL-UPGRADE.md`:
      step-by-step version bump process, the `git describe` gotcha, the
      table-vs-procedural triage method, and a sketch of future CI
      automation (nightly check of `ls-remote --tags upstream` for new
      releases → auto-bump → `make codegen` → `cargo t` →
      auto-PR if green, or a triage report issue if red).
      **Proof**: doc exists, under 400 lines, reviewed against this
      TPP's actual experience (not aspirational).

- [x] **Task 9: Final validation.** `make codegen fmt lint t` clean;
      `make compat-test` pass rate is >= baseline from Task 1 (a
      regression here means Task 5/6 missed something).
      **Proof**: command output pasted into handoff notes; TPP moved to
      `_done/`.

## Files referenced

- `third-party/exiftool/lib/Image/ExifTool.pm:32` — `$VERSION` ground truth
- `third-party/exiftool/Changes` — human-readable release history (only
  current up to whatever's checked out; re-check via `exiftool.org/rss.xml`
  or `ls-remote --tags upstream` for anything newer)
- `third-party/exiftool/lib/Image/ExifTool/Canon.pm` — extender/teleconverter
  detection regex added upstream (see diff command above)
- `third-party/exiftool/lib/Image/ExifTool/Geotag.pm` (~line 1104-1140) —
  GPS track interpolation, touched upstream
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm` — large diff,
  not yet triaged line-by-line
- `config/exiftool_modules.json` — module list codegen processes
- `codegen/` (invoked via `make codegen`) — regenerates `src/generated/`
- `tools/generate_exiftool_json.sh` — compat snapshot generator
- `docs/CODEGEN.md`, `docs/guides/EXIFTOOL-GUIDE.md` — existing codegen docs
- `docs/guides/EXIFTOOL-UPGRADE.md` — **to be created** by this TPP
