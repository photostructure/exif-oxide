# TPP: Fuzzing Infrastructure

## Summary

exif-oxide parses untrusted, adversarial input by design — PhotoStructure
ingests arbitrary user photo/video files, and every format parser here
(JPEG segment scanning, TIFF/EXIF IFD walking, PNG chunks, RAW
manufacturer formats, XMP/IPTC) does byte-level offset arithmetic on
attacker-influenced data. **There is no fuzzing anywhere in this repo
today** (verified below). This TPP adds `cargo-fuzz` targets over the
real parser entry points, seeds them from the existing test-image
corpus, wires a time-boxed CI job, and establishes a crash-triage
workflow that follows the project's existing TDD process (every crash
becomes a failing regression test before it becomes a fix).

## Current phase
- [x] Research & Planning
- [x] Write breaking tests
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation
- [x] Review & Refinement
- [x] Final Integration

## Required reading
- [TDD.md](../docs/TDD.md) — a crash IS a bug; same workflow: reproduce
  as a test, confirm it fails/panics for the right reason, fix root
  cause, confirm the test passes, run `cargo t` for regressions
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md) — when a fuzz crash reveals
  a case ExifTool also handles specially (even if "handles" means "warns
  and continues"), match ExifTool's behavior, don't invent stricter
  validation it doesn't have
- [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md) — don't "fix" a crash by adding
  a defensive guard that silently swallows the case; propagate errors,
  don't paper over them (this is also called out in the user's global
  "No bogus guardrails" rule)

## Description

**Problem**: no fuzz targets, no fuzzing CI job, no corpus-driven crash
testing exist. Confirmed:

```bash
rg -l fuzz /home/mrm/src/exif-oxide --glob '!third-party' --glob '!src/generated'
# docs/design/NORMALIZATION-DECISION.md   (says "fuzzy matching" - unrelated)
# scripts/analyze-required-expressions.py (says "fuzzy match" - unrelated)
# -> zero real matches
cargo fuzz --version
# error: no such command: `fuzz`  (cargo-fuzz isn't even installed locally)
```

**Why it matters**: this is untrusted-input parsing territory. A
malformed JPEG/TIFF/RAW file with a corrupted IFD offset, a negative
length, or a truncated chunk should produce an `Err`, never a panic, OOM,
or (in unsafe code, though this crate should be safe-Rust-only —
confirm with `rg unsafe src/ --glob '!generated'`) memory corruption.
PhotoStructure runs this against files it doesn't control the origin of.

**Success test**: `cargo +nightly fuzz run fuzz_tiff -- -max_total_time=60`
runs clean (no crash) after seeding from the corpus; a deliberately
introduced off-by-one in an offset calculation gets caught within the
time budget.

## Tribal knowledge

### Real parser entry points (verified 2026-07-01)

These are the actual `pub fn` boundaries that take raw bytes or a
`Read + Seek` and do untrusted-input parsing — fuzz these directly
rather than going through the CLI/file-path layer, since in-memory
`Cursor<&[u8]>` targets are what cargo-fuzz wants:

| Format | Function | File |
|---|---|---|
| Format sniffing | `detect_file_format<R: Read+Seek>` | `src/formats/detection.rs:16` |
| JPEG segments | `scan_jpeg_segments<R>`, `extract_jpeg_exif<R>`, `extract_jpeg_xmp<R>`, `extract_jpeg_iptc<R>`, `hash_jpeg_scan_data<R>` | `src/formats/jpeg.rs` |
| TIFF/EXIF | `extract_tiff_exif<R>`, `extract_tiff_xmp`, `get_tiff_endianness`, `validate_tiff_format` | `src/formats/tiff.rs` |
| PNG | `parse_png_ihdr(data: &[u8])`, `hash_png_image_data` | `src/formats/png.rs` |
| AVIF/HEIC (ISO-BMFF boxes) | `parse_box_header`, `find_box_by_type`, `parse_ispe_box`, `parse_pitm_box`, `parse_iinf_box`, `parse_ipma_box`, `extract_heic_dimensions_primary_item`, `extract_avif_dimensions` | `src/formats/avif.rs` |
| GIF | `parse_gif_screen_descriptor(data: &[u8])` | `src/formats/gif.rs:67` |
| IPTC | `parse_iptc_metadata(data: &[u8])`, `parse_iptc_from_app13` | `src/formats/iptc.rs:213,220` |
| XMP | `XmpProcessor::process_xmp_data(&mut self, data: &[u8])`, `process_xmp_data_individual` | `src/xmp/processor.rs:57,87` |
| Core IFD walker (the one everything above eventually feeds into) | `ExifReader::parse_exif_data(&mut self, exif_data: &[u8])` | `src/exif/mod.rs:172` — called from `src/formats/mod.rs:504,677,954` and `src/raw/formats/minolta.rs:392` |
| RAW magic-byte validators | `validate_kyocera_magic`, `validate_minolta_mrw_magic`, `validate_panasonic_rw2_magic`, `validate_olympus_orf_magic`, `validate_sony_arw_magic/sr2/srf` | `src/raw/detector.rs:116-215` |
| Whole-file, highest-level | `extract_metadata(path: &Path, ...)` | `src/formats/mod.rs:63` — takes a `Path`, not bytes; fuzz harness must write the fuzz input to a tempfile first if you want this level of coverage (dispatch + detection + format-specific parsing all in one) |

`ExifReader::parse_exif_data` is the highest-value single target: JPEG,
TIFF, DNG, and every RAW format in `src/raw/formats/` (canon, sony,
olympus, panasonic, minolta, kyocera) all funnel into it after their own
format-specific header/offset handling. A fuzz target there alone
exercises the IFD-chain-walking, offset-adjustment
(`apply_is_offset_adjustment`, see `_todo/P1-BINARY-EXTRACTION-ALL-FORMATS.md`
for the offset-handling landmines), and tag-value-parsing code shared by
every format.

### Seed corpus

```bash
find test-images -type f | wc -l                        # 328
find third-party/exiftool/t/images -type f | wc -l       # 193
```

Both directories are real camera output across many manufacturers —
much better seeds than synthetic files. `third-party/exiftool/t/images`
in particular is ExifTool's own 25-year accumulation of edge-case and
malformed-file regression samples (deliberately weird files that once
broke ExifTool) — an excellent adversarial corpus. `cargo-fuzz` corpora
are just directories of files; point `fuzz/corpus/<target>/` at (copies
of, not symlinks into — fuzzing mutates corpus files in place)
format-relevant subsets of these two directories.

### cargo-fuzz is not installed or configured anywhere

`cargo fuzz --version` fails locally (`no such command: fuzz`). Task 1
must `cargo install cargo-fuzz` and `cargo fuzz init` — this creates a
`fuzz/` member crate at the repo root using `libFuzzer` (requires
`+nightly`). Add `fuzz/` to `.gitignore`'s exclusions carefully — the
harnesses (`fuzz/fuzz_targets/*.rs`) must be committed, but
`fuzz/target/` and `fuzz/corpus/` should not be (corpus can be large;
regenerate or fetch separately, similar to how `test-images/` is
handled via `make pull-test-images`).

### Safe-Rust check before assuming "crash = bug in our code"

```bash
rg "unsafe" src/ --glob '!generated' --glob '!*/tests.rs'
```

If this is empty (verify — don't assume), every crash found is
reachable via 100% safe Rust, meaning any crash is either a panic
(unwrap/expect/index-out-of-bounds/integer overflow in debug builds) or
a resource exhaustion (unbounded allocation from an attacker-controlled
length field) — never actual memory unsafety. That changes triage
priority: panics are correctness bugs (should return `Err` instead),
allocation bombs are the more security-relevant class to prioritize.

## Solutions

### Option A (preferred): One fuzz target per format entry point, not one mega-target

Separate `fuzz_targets/fuzz_jpeg.rs`, `fuzz_tiff.rs`, `fuzz_png.rs`,
`fuzz_avif.rs`, `fuzz_gif.rs`, `fuzz_iptc.rs`, `fuzz_xmp.rs`,
`fuzz_exif_ifd.rs` (the `ExifReader::parse_exif_data` one), each calling
exactly one entry point from the table above on the raw fuzz input.

**Pros**: crashes are automatically attributed to a format/module;
seed corpora stay small and relevant per-target (faster fuzzing); can
run targets independently in CI with separate time budgets; matches how
`third-party/exiftool/t/images` is already organized by manufacturer/format.
**Cons**: more `fuzz_targets/*.rs` boilerplate files to maintain; won't
catch bugs in the dispatch/detection layer that only manifest when
format detection picks the "wrong" parser for ambiguous bytes.

### Option B: Single whole-file fuzz target through `extract_metadata`

One target, writes fuzz bytes to a tempfile, calls
`formats::extract_metadata(path, false, false, None)`.

**Why Option A is better as the primary approach**: `extract_metadata`
requires filesystem I/O per fuzz iteration (tempfile create/write/read),
which is dramatically slower than in-memory `Cursor<&[u8]>` targets —
fewer iterations per second means fewer bugs found per CPU-hour. Keep
Option B as a **secondary, lower-frequency** target specifically to
catch format-detection/dispatch bugs that Option A's per-format targets
can't reach (e.g., a file that's detected as PNG but actually triggers
TIFF parsing due to embedded magic bytes).

## Tasks

- [x] **Task 0: Breaking-test-first for the fuzzing gap itself.** Before
      writing infra, confirm there's currently no signal at all: run
      `rg "unsafe" src/ --glob '!generated'` and record the result in
      this TPP; if any `unsafe` exists, list the file:line — those
      blocks are the highest-priority fuzz targets since they're where
      real memory-safety bugs (not just panics) could live.

- [x] **Task 1: Install and scaffold.** `cargo install cargo-fuzz`,
      `cargo +nightly fuzz init` at repo root. Confirm the generated
      `fuzz/Cargo.toml` depends on the local `exif-oxide` crate path.
      **Proof**: `cargo +nightly fuzz list` shows the scaffold with zero
      targets; `fuzz/` exists with correct `.gitignore` entries for
      `fuzz/target/` and `fuzz/corpus/`.

- [x] **Task 2: Write the `ExifReader::parse_exif_data` target first**
      (highest coverage-per-target per the tribal-knowledge table
      above). `fuzz_targets/fuzz_exif_ifd.rs` calls
      `ExifReader::new().parse_exif_data(data)`. Seed
      `fuzz/corpus/fuzz_exif_ifd/` from EXIF blobs extracted out of a
      sample of `test-images/` and `third-party/exiftool/t/images/`
      files (extract via `extract_jpeg_exif`/`extract_tiff_exif` once,
      save the raw bytes — don't seed with whole JPEG files here, this
      target takes raw EXIF/TIFF bytes not a JPEG container).
      **Proof**: `cargo +nightly fuzz run fuzz_exif_ifd -- -max_total_time=120`
      completes with no crash; corpus dir is non-empty.

- [x] **Task 3: Write remaining per-format targets** (JPEG, TIFF, PNG,
      AVIF/HEIC, GIF, IPTC, XMP) per the Option A table. Seed each from
      the relevant subset of `test-images/` /
      `third-party/exiftool/t/images/` (e.g., `fuzz_jpeg` seeds from
      `*.jpg`/`*.jpeg` files directly, since `scan_jpeg_segments` takes
      a whole JPEG reader).
      **Proof**: each target runs `-max_total_time=60` clean; list of
      targets and their corpus sizes in this TPP's handoff notes.

- [x] **Task 4: Secondary whole-file target (Option B).** Add
      `fuzz_whole_file` using `extract_metadata` via a tempfile, low
      time-budget in CI (this one is slow per-iteration by design —
      don't let it starve the per-format targets' CI time).
      **Proof**: runs clean for a short smoke duration.

- [x] **Task 5: CI job.** Add a bounded-time fuzzing job (nightly or
      per-PR-with-time-cap — decide based on existing CI runtime
      budget, check `.github/workflows/` for current job durations
      first) that runs every target for e.g. 60-120s each and fails the
      build on any crash, uploading the crashing input as an artifact.
      **Proof**: a deliberately introduced bug (e.g., temporarily
      `unwrap()` an `Option` that can be `None` on malformed input) gets
      caught by the CI job in a test run, then revert the deliberate bug.

- [x] **Task 6: Crash-triage workflow (per TDD.md).** Document in this
      TPP (or a short `docs/guides/FUZZING.md`): when `cargo fuzz run`
      finds a crash, (a) minimize with `cargo fuzz tmin`, (b) turn the
      minimized input into a `#[test]` regression test with the raw
      bytes committed under `tests/fixtures/` or similar, (c) confirm
      the test fails for the expected reason, (d) fix following Trust
      ExifTool (check what real ExifTool does with the same malformed
      input — `exiftool` should be more lenient than an unwrap-and-panic,
      rarely stricter), (e) confirm the regression test passes and
      `cargo t` has no regressions.
      **Proof**: workflow doc exists; if any crash was found during
      Tasks 2-4, it went through this exact workflow as a worked example.

- [x] **Task 7: Final validation.** `make codegen fmt lint t` clean;
      all fuzz targets run their smoke-duration clean; CI job wired and
      green. Move to `_done/`. (2026-07-03: gate ran clean — 695 tests, 0
      failed; all 6 committed reproducers re-run clean; CI "green" pending
      the first real nightly run since the workflow lands with this commit.)

## Session log

### 2026-07-02 — implementation + first crash triage

**Built (Option A + secondary Option B, as designed):**
- `fuzz/` cargo-fuzz crate, detached from the root workspace via an empty
  `[workspace]` table in `fuzz/Cargo.toml` so `make lint`/`make t` (stable,
  `--all-features`) never try to build the libFuzzer targets. Verified: both
  stay green.
- 9 targets, all shown by `cargo +nightly fuzz list`: `fuzz_exif_ifd`,
  `fuzz_jpeg`, `fuzz_tiff`, `fuzz_png`, `fuzz_avif`, `fuzz_gif`, `fuzz_iptc`,
  `fuzz_xmp`, `fuzz_whole_file`.
- `fuzz/seed-corpus.sh` — copies (never symlinks) format-relevant subsets of
  `test-images/` + `third-party/exiftool/t/images/`. Seed counts:
  fuzz_jpeg 104, fuzz_iptc 104, fuzz_whole_file 137, fuzz_xmp 12, fuzz_tiff 7,
  fuzz_exif_ifd 7, fuzz_avif 5, fuzz_png 2, fuzz_gif 2 (5 MB/file cap — see
  gotchas).
- `.github/workflows/fuzz.yml` — matrix (one job/target), nightly `schedule` +
  `workflow_dispatch` (see CI decision below).
- `docs/guides/FUZZING.md` — targets, corpus, run instructions, crash-triage
  workflow, worked examples.

**Task 0 finding:** `rg "unsafe" src/ --glob '!generated' --glob '!*/tests.rs'`
is empty. Every reachable crash is a panic or an allocation bomb, never memory
unsafety — which held for all three bugs found.

**Crashes found on first validation (all in non-excluded files) and fixed via
TDD (breaking test → fix per Trust ExifTool → cargo t):**
1. `fuzz_exif_ifd` — OOM `malloc(8.58 GB)`. `extract_short_array_value` /
   `extract_long_array` in `src/value_extraction.rs` pre-allocated
   `Vec::with_capacity(entry.count)` (count up to `0xFFFFFFFF`) *before* the
   bounds check. Fix: validate the byte range against `data`, then allocate.
   Regression tests `test_{short,long}_array_huge_count_does_not_oom`.
2. `fuzz_jpeg` / `fuzz_iptc` — panic "multiply with overflow" at
   `src/formats/iptc.rs:120`. Extended-IPTC length accumulated up to 8 bytes
   into a `u16`; ExifTool (IPTC.pm:1152) uses an unbounded scalar. Fix: `u64`
   accumulator + overflow-safe bounds check. Regression test
   `test_extended_iptc_length_does_not_overflow`.
3. `fuzz_avif` — panic "add with overflow" at `src/formats/avif.rs:121`.
   `data_start + data_size` overflowed `usize` for a near-`u64::MAX` extended
   box size. Fix: `checked_add`. Regression test
   `test_box_header_huge_size_does_not_overflow`.

Reproducers saved under `fuzz/artifacts/{fuzz_exif_ifd,fuzz_jpeg,fuzz_avif}/`.
Clean targets: `fuzz_tiff` (27.8M runs), `fuzz_png` (50.2M), `fuzz_gif` (45.5M).

**CI decision:** nightly `schedule` + `workflow_dispatch`, NOT a per-PR gate.
Because fuzzing is non-deterministic and surfaces pre-existing latent bugs
unrelated to a given PR (as these three demonstrate), a blocking PR check would
be flaky. Promote to a per-PR smoke once the corpus runs clean across several
nightly cycles. The three real crashes above already prove "CI catches bugs,"
so the synthetic deliberate-bug demo (Task 5 proof) was unnecessary and skipped.

**Gotchas for the next session:**
- `cargo fuzz run` intermittently triggers a full ASan recompile (~4.5 min) of
  `exif-oxide` — appears when another `cargo` invocation ran recently. It caused
  false `rc=124` timeouts (`fuzz_iptc`, `fuzz_xmp`, `fuzz_whole_file`) in the
  validation harness — those were killed mid-recompile, NOT real hangs/bugs. Do
  a standalone `cargo +nightly fuzz build` first and use a generous outer
  timeout when scripting runs.
- Seed cap is 5 MB/file, which excludes most RAW files, so `fuzz_tiff` /
  `fuzz_exif_ifd` only get 7 (small `.tif`) seeds. Fine for finding these bugs;
  raise `MAX_SEED_BYTES` in `seed-corpus.sh` if deeper IFD coverage is wanted.
- Added re-exports in `src/formats/mod.rs` (`parse_png_ihdr`, `IhdrData`,
  `extract_heic_dimensions_primary_item`, `parse_box_header`, `IsoBox`) so the
  fuzz crate can reach the PNG/AVIF box parsers past their private submodules.

**Open follow-ups (not blocking this TPP):**
- `src/formats/jpeg.rs` has several `vec![0u8; length - 2]` sites where `length`
  is a JPEG segment field; bounded to ~64 KB (u16) but a `length < 2` could
  underflow-panic. Let a re-run of `fuzz_jpeg` (post-iptc-fix) confirm/deny
  before touching. `jpeg.rs:699` `Vec::with_capacity(total_size)` in extended-
  XMP reassembly is another candidate.
- Once several clean nightly runs land, add the `pull_request` smoke trigger.

### 2026-07-03 — resume: review gate + two more crashes found & fixed

**Original 3 fixes verified**: all three saved reproducers re-ran clean, and
`fuzz_avif` ran a full 120s clean. But the 120s re-runs of `fuzz_exif_ifd` and
`fuzz_jpeg` each found a NEW pre-existing crash (both triaged via TDD:
breaking test → validated explosion → fix per ExifTool source → green):

4. `fuzz_jpeg` — panic "subtract with overflow" at `jpeg.rs:462`
   (`length - 77`): APP1 scanners matched EXIF/XMP/extended-XMP identifiers
   read from the *stream*, then subtracted fixed header sizes from the
   *declared* segment length, which can be smaller. Same class at `length - 8`
   (L261), `length - 31` (L276, L400) — exactly the open follow-up the
   2026-07-02 log predicted. ExifTool matches identifiers only against
   `$$segDataPt` (exactly the declared bytes) and warns "Invalid extended XMP
   segment" when `$length <= 75` (ExifTool.pm:7840-7858). Fix: gate each
   identifier match on the declared length; warn+skip short ext-XMP segments.
   4 regression tests in `jpeg.rs`; minimized 81-byte reproducer committed.
5. `fuzz_exif_ifd` — ASan stack overflow: MakerNotes IFD containing a 0x927C
   tag pointing back at itself recursed forever via parse_ifd_entry →
   process_maker_notes_with_signature_detection → process_canon_makernotes →
   parse_ifd. The PROCESSED guard only lived in process_subdirectory, which
   the manufacturer dispatch bypasses (ExifTool catches this in
   ProcessDirectory's `$$self{PROCESSED}`). Fix: same guard at the top of the
   makernote dispatch (`ifd.rs`). Regression test
   `tests/makernote_recursion_test.rs` (synthesized self-referential TIFF;
   validated pre-fix: "has overflowed its stack", SIGABRT).

**Review gate ran** (8-angle Claude /code-review fan-out + codex CLI).
Accepted & applied findings:
- `fuzz/.gitignore` was ignoring `fuzz/artifacts/` — the reproducers the
  handoff required committed. Fixed; FUZZING.md step 6 rewritten (it said
  "delete the artifact", contradicting the .gitignore comment and this TPP).
- `fuzz.yml`: `lfs: true` was a false claim (test-images/ is B2-synced, not
  git-LFS; CI corpus is t/images only — now documented in the workflow and
  FUZZING.md); concurrency comment described a nonexistent PR trigger and
  `cancel-in-progress: true` would let a nightly kill a long dispatch run
  (now queues instead); duplicated time-budget step inlined
  (`inputs.max_total_time || '300'`).
- 32-bit-target latent bugs in the new bounds math: `count * 2`/`count * 4`
  usize overflow in value_extraction (now u64 math); avif
  `(box_size - header_size) as usize` truncation (now u64 compare, which also
  subsumed the earlier checked_add and its duplicated error branch); avif
  `offset + BOX_HEADER_SIZE` wrap for the now-public parse_box_header (now
  saturating remaining-bytes check). iptc usize::MAX clamp sentinel replaced
  by comparing in u64 like ExifTool's unbounded scalar.
- Rejected/deferred (vetted, real, but pre-existing and non-crash — follow-ups
  below): iptc marker==0 zero-scan starts 4 bytes late vs IPTC.pm:1136
  (warn-parity only); avif rejects size-0 "box extends to EOF" boxes that
  ExifTool handles; both 64-bit-only OOM regression tests can't actually fail
  pre-fix under Linux overcommit (documented in their comments; the fuzz
  artifact is the definitive reproducer).

**codex review (second independent reviewer) caught a real HIGH bug in the
first version of the recursion guard**: it inserted the MakerNotes addr into
`processed` before dispatch, but the generic (non-Canon/Olympus/Sony) path
delegates to `process_subdirectory`, whose own PROCESSED check computes the
same addr when `base == 0` — so every generic-manufacturer MakerNotes
directory was wrongly skipped as a "circular reference" on FIRST visit.
`cargo t` (695 green) did not catch it: no test asserts on generic-path
makernote output. Fixed via TDD (breaking test
`test_generic_makernote_is_not_skipped_as_circular` failed with exactly the
predicted warning, then passed): the guard now only *checks* at entry and
pre-registers the addr solely in the Canon/Sony branches that recurse via
direct `parse_ifd`; the generic path keeps process_subdirectory's own
tracking. codex confirmed all other areas clean, including the exact
jpeg length-gate thresholds (8/31/37/78) against ExifTool.pm's `$length`
semantics. Lesson recorded: the codex + Claude double-review both earned
their keep — each found a distinct real bug class the other missed.

### COMPLETION STATE (2026-07-03)

All four resume-checklist items from the 2026-07-03 pause were done this
session: (1) `cargo t` green throughout (690 at pause + 4 jpeg underflow
tests + 2 recursion tests; final `make ... t` gate: 952 passed / 0 failed
across all suites); (2) all saved reproducers re-run clean, plus 120s re-runs —
which surfaced crashes #4 and #5 (jpeg length underflow, makernote recursion
stack overflow), both fixed via TDD; (3) review gate ran (8-angle Claude
fan-out + codex; codex caught a real HIGH bug in the first recursion-guard
version — see the session log above); (4) `make codegen fmt lint t` clean,
one coherent commit, TPP moved to `_done/`, program tracker updated.

Note: `fuzz/.gitignore` DID exclude `fuzz/artifacts/` at pause time — fixed;
all 6 reproducers (3 original + jpeg crash/minimized pair + recursion input)
are committed.

**Follow-ups (deferred, non-crash, all vetted as real but pre-existing):**
- iptc.rs marker==0 zero-padding scan starts 4 bytes late vs IPTC.pm:1136
  (warn-parity divergence only; both implementations stop parsing).
- avif.rs rejects `box_size == 0` ("box extends to EOF", legal ISO-BMFF,
  handled by ExifTool QuickTime.pm) — a valid-file functionality gap, not a
  crash.
- The value_extraction OOM regression tests cannot actually fail pre-fix
  under Linux overcommit (documented in their comments); the committed fuzz
  artifact + nightly `-rss_limit_mb` is the real guard.
- CI fuzzes a thinner corpus than local runs (no `test-images/` in CI — B2
  credentials); consider publishing a minimized corpus artifact if nightly
  coverage needs a boost.
- Once several nightly cycles run clean, add the `pull_request` smoke
  trigger (~60s/target) per the fuzz.yml header comment.

## Files referenced

- `src/formats/detection.rs:16` — `detect_file_format`
- `src/formats/jpeg.rs` — JPEG segment/EXIF/XMP/IPTC/hash extraction
- `src/formats/tiff.rs` — TIFF/EXIF extraction
- `src/formats/png.rs:86` — `parse_png_ihdr`
- `src/formats/avif.rs` — ISO-BMFF box parsing (AVIF/HEIC)
- `src/formats/gif.rs:67` — `parse_gif_screen_descriptor`
- `src/formats/iptc.rs:213,220` — IPTC parsing
- `src/xmp/processor.rs:57,87` — `XmpProcessor::process_xmp_data`
- `src/exif/mod.rs:172` — `ExifReader::parse_exif_data` (core IFD walker)
- `src/raw/detector.rs:116-215` — RAW magic-byte validators
- `src/raw/formats/*.rs` — per-manufacturer RAW handling feeding into `parse_exif_data`
- `src/formats/mod.rs:63` — `extract_metadata` (whole-file, Option B target)
- `test-images/` (328 files), `third-party/exiftool/t/images/` (193 files) — seed corpora
- `Makefile` — `pull-test-images` target for corpus refresh pattern to mirror
