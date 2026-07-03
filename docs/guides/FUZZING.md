# Fuzzing exif-oxide

exif-oxide parses untrusted, adversarial input by design: PhotoStructure feeds
it arbitrary user photo/video files, and every format parser does byte-level
offset arithmetic on attacker-influenced data. A malformed JPEG/TIFF/RAW with a
corrupted IFD offset, a negative length, or a truncated chunk must produce an
`Err` — never a panic, an out-of-bounds index, an integer-overflow abort, or an
unbounded allocation.

This crate is 100% safe Rust (`rg "unsafe" src/ --glob '!generated'` is empty),
so a fuzz crash can never be memory corruption. It is always one of:

- **a panic** — `unwrap`/`expect`/`slice[i]` out of bounds / arithmetic overflow
  in a debug build. This is a correctness bug: the parser should have returned
  `Err`. **Fix by returning an error, not by adding a silent guard that swallows
  the case** (see [ANTI-PATTERNS.md](../ANTI-PATTERNS.md) and the "No bogus
  guardrails" rule).
- **an allocation bomb / OOM / timeout** — an attacker-controlled length or
  count field drove an unbounded `Vec::with_capacity`, read, or loop. These are
  the most security-relevant class; bound the allocation against the remaining
  input size.

## Prerequisites

```bash
rustup toolchain install nightly     # libFuzzer requires the nightly toolchain
cargo install cargo-fuzz --locked    # provides `cargo fuzz`
```

## Targets

The `fuzz/` crate is a **separate workspace** (it has its own `[workspace]` in
`fuzz/Cargo.toml`) so it never affects `make lint`/`make t` on stable. It
follows Option A from the TPP: one target per format entry point, calling
exactly one parser boundary on the raw fuzz input via an in-memory
`Cursor<&[u8]>` (fast — no filesystem per iteration), plus one slower
whole-file target for the detection/dispatch layer.

| Target | Entry point(s) | Notes |
|---|---|---|
| `fuzz_exif_ifd` | `ExifReader::parse_exif_data` | Highest value: the shared TIFF/EXIF IFD walker that JPEG, TIFF, DNG and every RAW format funnel into. |
| `fuzz_jpeg` | `scan_jpeg_segments`, `extract_jpeg_{exif,xmp,iptc}` | JPEG FF-marker segment chain + APP extractors. |
| `fuzz_tiff` | `validate_tiff_format`, `get_tiff_endianness`, `extract_tiff_xmp`, `extract_tiff_exif` | TIFF container header/offset math. |
| `fuzz_png` | `parse_png_ihdr` | PNG signature + IHDR chunk. |
| `fuzz_avif` | `parse_box_header`, `extract_avif_dimensions`, `extract_heic_dimensions_primary_item` | ISO-BMFF box tree (AVIF/HEIC). |
| `fuzz_gif` | `parse_gif_screen_descriptor` | GIF logical screen descriptor. |
| `fuzz_iptc` | `parse_iptc_from_app13`, `parse_iptc_metadata` | Photoshop 8BIM wrapper + IIM records. |
| `fuzz_xmp` | `XmpProcessor::process_xmp_data{,_individual}` | RDF/XML packet parsing. |
| `fuzz_whole_file` | `extract_metadata` (Option B) | Secondary, slow: writes input to a tempfile, exercises detection + dispatch. Small CI budget so it never starves the fast targets. |

```bash
cargo +nightly fuzz list   # shows all targets
```

## Seeding the corpus

`fuzz/corpus/` is gitignored (it can be large, and libFuzzer mutates it in
place). Seed it from the project's real test-image corpora:

```bash
fuzz/seed-corpus.sh
```

This **copies** (never symlinks — libFuzzer rewrites corpus files) format-
relevant subsets of `test-images/` and `third-party/exiftool/t/images/` into
`fuzz/corpus/<target>/`. The ExifTool `t/images` set is 25 years of
deliberately-malformed regression samples, which makes an excellent adversarial
seed set. Re-running the script is safe: it refreshes the seeds it manages and
leaves any fuzzer-discovered inputs in place.

Note: `test-images/` only exists locally (it is gitignored and synced from
Backblaze B2 via `make pull-test-images`, which needs credentials). In CI the
script silently skips it, so CI corpora are seeded from `t/images` alone —
local runs fuzz a substantially richer corpus than the nightly job.

## Running a target locally

```bash
fuzz/seed-corpus.sh                                             # once
cargo +nightly fuzz run fuzz_exif_ifd -- -max_total_time=120    # run for 120s
cargo +nightly fuzz run fuzz_tiff     -- -max_total_time=60
```

Useful libFuzzer flags (everything after `--`):

- `-max_total_time=<seconds>` — stop after N seconds (what CI uses).
- `-runs=<n>` — stop after N inputs instead of by time.
- `-rss_limit_mb=<mb>` — abort if RSS exceeds this (catches allocation bombs;
  the default 2048 is usually fine).
- `-max_len=<bytes>` — cap generated input size.

A clean run exits 0. A crash writes the offending input to
`fuzz/artifacts/<target>/crash-<sha1>` and prints the panic/stack.

## Crash triage — a crash IS a bug (follow [TDD.md](../TDD.md))

When `cargo fuzz run` finds a crash, do **not** patch it by adding a defensive
guard that quietly returns early. Follow the same test-driven workflow as any
other bug:

1. **Reproduce and minimize.** cargo-fuzz prints the crashing input path. Shrink
   it to the minimal reproducer:

   ```bash
   cargo +nightly fuzz tmin fuzz_exif_ifd fuzz/artifacts/fuzz_exif_ifd/crash-<sha1>
   ```

   Re-run the minimized file to confirm it still crashes:

   ```bash
   cargo +nightly fuzz run fuzz_exif_ifd fuzz/artifacts/fuzz_exif_ifd/minimized-from-<sha1>
   ```

2. **Write a breaking regression test.** Commit the minimized bytes (they are
   tiny after `tmin`) and add a `#[test]` that feeds them to the same entry
   point and asserts it returns `Err` rather than panics. Small inputs can be
   inlined as a byte literal; larger ones go under `tests/fixtures/`. Put the
   test next to the parser it covers (e.g. a `#[cfg(test)]` module in
   `src/exif/mod.rs`) or in the crate's integration tests.

3. **Confirm it fails for the right reason.** Run the test and verify it panics
   with the exact message/location the fuzzer reported — not a different bug or
   a test-setup mistake.

4. **Fix at the root cause, following Trust ExifTool.** Check what real ExifTool
   does with the same malformed input (`exiftool <file>`): ExifTool is almost
   always *more* lenient than an unwrap-and-panic — it warns and continues, or
   skips the bad directory. Match that behavior (return `Err`, or skip the
   malformed sub-structure exactly as ExifTool does). Do not invent stricter
   validation ExifTool doesn't have, and do not silently swallow the error.

5. **Confirm green + no regressions.** The regression test passes, and
   `cargo t` (or `make t`) shows no new failures.

6. **Commit the artifact** alongside the fix: `fuzz/artifacts/` is tracked in
   git (see `fuzz/.gitignore`) because the raw crash inputs are the regression
   corpus for the fixes — some crashes (e.g. a multi-GB allocation bomb) cannot
   be reproduced safely by a unit test, so the artifact is the only executable
   evidence. Delete only superseded intermediates (e.g. pre-`tmin` copies of an
   input whose minimized form is kept).

If a crash lands in code another agent/PR currently owns, save the reproducer
under `fuzz/artifacts/<target>/` and hand it off rather than editing the owned
code.

## Bugs found so far (worked examples)

The first validation runs of these harnesses found five pre-existing crashes,
each fixed via the workflow above (breaking test → fix per Trust ExifTool →
`cargo t`):

| Target | Crash | Root cause | Fix |
|---|---|---|---|
| `fuzz_exif_ifd` | OOM, `malloc(8.58 GB)` | `extract_short_array_value`/`extract_long_array` called `Vec::with_capacity(entry.count)` with an attacker-controlled IFD count (`0xFFFFFFFF`) **before** the offset/bounds check | Validate the byte range against `data` first (in u64, safe for 32-bit targets), then allocate (`src/value_extraction.rs`) |
| `fuzz_jpeg`, `fuzz_iptc` | panic: multiply overflow | extended-IPTC length accumulated up to 8 bytes into a `u16`; ExifTool (IPTC.pm:1152) uses an unbounded scalar | widen the accumulator to `u64`, bounds-check in u64 like ExifTool's unbounded compare (`src/formats/iptc.rs`) |
| `fuzz_avif` | panic: add overflow | `data_start + data_size` overflowed `usize` for a near-`u64::MAX` extended box size before the length check | bounds-check the box content in u64 before deriving `data_end` (`src/formats/avif.rs`) |
| `fuzz_jpeg` | panic: subtract overflow | APP1 scanners computed `length - 8`/`length - 31`/`length - 77` from the declared segment length after matching EXIF/XMP identifiers read from the *stream*, so a segment declaring fewer bytes than its own header underflowed `u16`; ExifTool matches identifiers only against the declared segment data (`$$segDataPt`) and warns on short extended-XMP segments (ExifTool.pm:7840-7858) | gate each identifier match on the declared length; warn + skip short extended-XMP segments (`src/formats/jpeg.rs`) |
| `fuzz_exif_ifd` | stack overflow | a MakerNotes IFD containing another MakerNotes tag (0x927C) pointing back at itself recursed forever: the manufacturer dispatch bypasses `process_subdirectory`'s PROCESSED tracking (ExifTool catches this in ProcessDirectory) | apply the same PROCESSED guard in `process_maker_notes_with_signature_detection` (`src/exif/ifd.rs`) |

All are the "panic / alloc-bomb, never memory unsafety" classes expected from a
safe-Rust codebase. Regression tests live next to each parser; the OOM's
definitive reproducer is the fuzz artifact (a unit test cannot safely provoke an
8 GB allocation, so the committed test guards the validate-before-allocate
ordering and the fuzzer confirms the allocation is gone under `-rss_limit_mb`).

## CI

`.github/workflows/fuzz.yml` runs each target as a parallel matrix job on a
**nightly `schedule`** and on **`workflow_dispatch`** (300s/target by default;
the dispatch input `max_total_time` overrides it). It is deliberately **not** a
per-PR blocking gate: fuzzing is non-deterministic and readily surfaces
pre-existing latent bugs unrelated to whatever a PR changes, which makes a flaky
merge signal. Once the seed corpus has run clean across several nightly cycles,
promote it to a short per-PR smoke (add a `pull_request` trigger with a ~60s
budget).

Any crash fails that target's job and uploads the crashing input from
`fuzz/artifacts/` as a build artifact for offline triage via the workflow above.
Running the targets as a matrix keeps wall-clock time at roughly the per-target
budget rather than the sum.
