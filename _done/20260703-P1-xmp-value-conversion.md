# TPP: XMP Value Conversion — Type-Mismatch Burndown

## Summary

**Problem**: XMP tags are emitted as raw XML text — `XMP:FNumber` = `"8/1"`
instead of `8.0`, `XMP:DateTimeOriginal` = `"2005-06-08T12:05:36+01:00"`
instead of `"2005:06:08 12:05:36+01:00"`.
**Why it matters**: 8 of the 13 type mismatches in the compat report
(45%, 86/191) share this one root cause; it's the largest single
accuracy win currently triaged.
**Solution**: port ExifTool's two-layer XMP conversion: (1) parse-time
format conversion keyed off `Writable` (`rational`/`date`,
XMP.pm:3673-3687), (2) per-tag ValueConv/PrintConv from the XMP exif
table (XMP.pm:2040-2115).
**Success test**: `cargo run --bin compare-with-exiftool -- test-images/canon/eos_1ds_mark_ii.jpg XMP:`
shows no diffs for FNumber, ApertureValue, ShutterSpeedValue,
FocalLength, DateTimeOriginal.
**Key constraint**: Trust ExifTool — translate XMP.pm exactly,
including `inf`/`undef` on zero denominators.

## Current phase

- [x] Research & Planning (2026-07-03, verified against XMP.pm v13.59 source)
- [x] Write breaking tests
- [x] Task breakdown review
- [x] Implementation
- [x] Review & Refinement
- [x] Final Integration (2026-07-03: adversarial review FIX-FIRST →
      2 missing PrintConv arms + a latent `print_fraction` sign bug
      fixed; 972 tests green; compat 94/191; committed)

## Implementation results (2026-07-03)

**Compat delta: 86 → 94 / 191 working (+8); type mismatches 13 → 5.**
The five remaining type mismatches are exactly the out-of-scope items below
(XMP:Source apostrophe, MakerNotes:Categories, IPTC:Keywords,
EXIF:GPSProcessingMethod, EXIF:XPKeywords) — no XMP-conversion regressions.
`compare-with-exiftool -g XMP:` is 100% (16/16) on the Canon file; the iPhone
file's only remaining XMP misses are GPSLatitude/GPSLongitude (DMS→decimal
`ToDegrees`, a separate conversion, pre-existing, untouched here).

**Files changed (diff is these 5 only; no generated-code drift):**
- `src/xmp/value_conversion.rs` (new) — Layer 1 ports (`convert_rational`,
  `convert_xmp_date`) + Layer 2 dispatch (`apply_exif_photo_conv`) + the two
  XMP-specific `sprintf` PrintConvs, with inline unit tests.
- `src/xmp/processor.rs` — apply Layer 1 then Layer 2 in `flatten_xmp_structure`.
- `src/xmp/xmp_lookup.rs` — DRY the namespace match into `namespace_table()`;
  add `lookup_xmp_tag_by_name` (see gotcha below).
- `src/xmp/mod.rs` — register the module.
- `tests/xmp_value_conv_tests.rs` (new) — image-level breaking tests for both
  files (Canon cluster + dates, iPhone GPS + fractional-seconds date).

**Deviation from Task 3 (codegen vs hand-registry):** chose the explicit
registry (`apply_exif_photo_conv`, keyed on the exif-namespace display name) over
extending the codegen `xmp_tag` strategy + an `XmpTagInfo.value_conv` field. The
codegen path for *expression*-valued XMP ValueConv/PrintConv does not exist today
(the strategy only extracts hash-map PrintConvs), so adding it would be a large,
higher-risk generator + impl_registry change that could shift output for many
other XMP tags — disproportionate for a 6-tag cluster. The registry only *wires*
already-ported EXIF functions (`apex_shutter_speed_value_conv`,
`apex_aperture_value_conv`, `fnumber_print_conv`, `exposuretime_print_conv`) plus
two tiny `sprintf` formatters — it transcribes no ExifTool data, so it does not
violate the no-manual-porting rule. `XmpTagInfo` got **no** `value_conv` field
(would have been shelf-ware under this approach). If XMP expression conversions
proliferate, revisit the codegen route.

**Gotcha discovered:** the parsed XMP structure is keyed by the *resolved display
name*, so renamed tags (exif:GPSTimeStamp is stored as `GPSDateTime`,
XMP.pm:2350) miss the property-name `lookup_xmp_tag` in `flatten_xmp_structure`
and lost their `Writable` format. Fixed with `lookup_xmp_tag_by_name` (a by-`.name`
scan of the namespace table), used **only** to recover the format for Layer 1 —
tag-name/PrintConv resolution is unchanged, so no behavior shift for other tags.

**Source correction (GPSAltitude):** XMP.pm:2342 GPSAltitude *does* carry
`PrintConv => '$val =~ /^(inf|undef)$/ ? $val : "$val m"'`, contradicting the
research note "Layer 1 rational only". But the snapshots and live compat run GPS
in numeric mode (`-GPSAltitude#`, per tools/generate_exiftool_json.sh), so
XMP:GPSAltitude ground truth is `12.3` (number), not `"12.3 m"`. We deliberately
do **not** apply the `"$val m"` PrintConv — Layer 1 rational only — matching the
project's GPS-decimal policy. Net effect matches the research; the rationale is
the numeric-mode oracle, not the absence of a PrintConv.

## Required reading

- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- [TDD.md](../docs/TDD.md) — breaking test first
- `git show 1438c0c4` — the IPTC ValueConv fix; same shape of bug, and
  its `src/formats/iptc.rs` hook is the pattern to mirror
- `third-party/exiftool/lib/Image/ExifTool/XMP.pm:3660-3700` — FoundXMP
  conversion call site (the mechanism we're porting)

## The ExifTool mechanism (verified 2026-07-03, file:line from v13.59)

Two layers, both applied at read time:

### Layer 1 — structural, format-driven (parse time, FoundXMP)

`XMP.pm:3673-3687`: after UTF-8 decode, every tag with a `Writable`
format gets:

- `Writable => 'rational'` → `ConvertRational` (`XMP.pm:3400-3417`):
  `m{^(-?\d+)/(-?\d+)$}` → numeric quotient `$1/$2`; denominator 0 →
  string `'inf'` (numerator ≠ 0) or `'undef'` (0/0). `"8/1"` → `8`,
  `"123/10"` → `12.3`.
- `Writable => 'date'` → `ConvertXMPDate` (`XMP.pm:3383-3394`):
  `^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}:\d{2})(:\d{2})?\s*(\S*)$` →
  `"$1:$2:$3 $4$s$6"` (seconds optional, timezone kept verbatim).
  Fallback: if value starts `^\d{4}(-\d{2}){0,2}` → `tr/-/:/` (bare
  `2005-06-08` → `2005:06:08`).

Our generated tables **already carry the key**: `XmpTagInfo.writable`
is `Some("rational")` for 44 entries and `Some("date")` for 36 across
`src/generated/XMP_pm/` — Layer 1 needs **zero codegen changes**.

### Layer 2 — per-tag ValueConv/PrintConv (XMP exif namespace table)

| Tag | ValueConv | PrintConv | XMP.pm line |
|-----|-----------|-----------|------|
| ShutterSpeedValue | `abs($val)<100 ? 1/(2**$val) : 0` | `PrintExposureTime` | 2081-2087 |
| ApertureValue, MaxApertureValue | `sqrt(2) ** $val` | `sprintf("%.1f",$val)` | 2088-2094, 2103-2109 |
| FNumber | — | `PrintFNumber` | 2047-2051 |
| ExposureTime | — | `PrintExposureTime` | 2042-2046 |
| FocalLength | — | `sprintf("%.1f mm",$val)` | 2161-2166 |
| GPSAltitude | — (Layer 1 rational only) | — | 2342-2348 |
| DateTimeOriginal, GPSDateTime (=GPSTimeStamp), DateTime, CreateDate, ModifyDate… | — (Layer 1 date) | `ConvertDateTime` = identity w/o `-d` | %dateTimeInfo XMP.pm:236-243 |

All Rust counterparts already exist (`src/implementations/`):
`apex_shutter_speed_value_conv` + `apex_aperture_value_conv`
(value_conv.rs:65,91), `exposuretime_print_conv` +
`fnumber_print_conv` (print_conv.rs:307,267).

## exif-oxide gap (verified 2026-07-03)

- `XmpTagInfo` (`src/core/xmp_tag_info.rs:9`) has **no `value_conv`
  field**; only `print_conv: Option<PrintConv>`.
- Generated XMP exif table (`src/generated/XMP_pm/exif_tags.rs`) has
  `print_conv: None` for every tag in the cluster — the codegen
  xmp_tag strategy (`codegen/src/strategies/xmp_tag.rs`) doesn't map
  expression-valued ValueConv/PrintConv through the impl_registry the
  way the EXIF path does.
- `src/xmp/processor.rs:138-220` applies only
  `PrintConv::Simple` lookups; Function/Expression/Complex fall
  through unconverted. Nothing implements Layer 1 at all.

## Tribal knowledge

- The IPTC fix (1438c0c4) hit the identical shape: parser never applied
  per-tag conversion + the referenced conversions were stubs. Mirror
  its structure: conversion application at the parse site, exact
  Exif.pm/XMP.pm ports in `src/implementations/`.
- exiftool -j emits PrintConv'd `"8.0"` as JSON *number* 8.0 when it
  looks numeric; our compat normalization (`src/compat/`) already
  handles that class — check before "fixing" a diff that's only
  JSON-type cosmetics.
- ConvertRational's `'inf'`/`'undef'` strings are load-bearing
  (GPS.pm returns '' for whole coordinates containing them — that
  gotcha already bit us in the GPSPosition fix, see
  `_todo/20260701-P1-snapshot-oracle-integrity.md` session log).
- ExifTool applies Layer 1 **before** storing the value, so ValueConv
  (Layer 2) sees the *numeric* value (e.g. ShutterSpeedValue ValueConv
  gets `8.321928`, not `"8321928/1000000"`).
- `XMP:GPSDateTime` is XMP tag `GPSTimeStamp` renamed
  (XMP.pm:2350-2361) — the rename already works in our tables; only
  the date conversion is missing.
- Perl `sprintf("%.1f")` rounds half-to-even differently than Rust's
  `{:.1}` in some cases — the existing APEX/FNumber Rust ports
  already deal with this; reuse them, don't re-port.

## Out of scope (tracked elsewhere)

The other 5 type mismatches have distinct root causes — separate
follow-ups, not this TPP: IPTC:Keywords list accumulation,
MakerNotes:Categories (Canon), EXIF:XPKeywords UCS-2 decode,
EXIF:GPSProcessingMethod encoding-prefix strip, XMP:Source apostrophe
truncation (looks like an XML quote-parsing bug — file separately if
confirmed). Composite `Condition` support and the COMPOSITE_TAGS
name-collision stay in the program TPP follow-ups
(`_todo/20260701-P0-strategic-review-program.md`).

## Tasks

- [x] **Task 1: Breaking tests.** New `tests/xmp_value_conv_tests.rs`
      asserting, for `test-images/canon/eos_1ds_mark_ii.jpg` and
      `test-images/apple/iphone_x.jpg`, the exact ExifTool snapshot
      values for: XMP:FNumber, XMP:ApertureValue,
      XMP:ShutterSpeedValue, XMP:FocalLength, XMP:DateTimeOriginal,
      XMP:GPSAltitude, XMP:GPSDateTime. Unit tests for ConvertRational
      edge cases (`8/1`, `123/10`, `1/0`→inf, `0/0`→undef, `-1/2`) and
      ConvertXMPDate shapes (full ISO w/ tz, no seconds, bare
      `2005-06-08`, `2005-06` partial).
      **Proof**: `cargo t xmp_value_conv` fails on value mismatch (not
      setup errors).
      *Done: image tests in `tests/xmp_value_conv_tests.rs` (confirmed
      failing on `"8/1"` vs `8.0` etc. before the fix); the ConvertRational/
      ConvertXMPDate unit tests live inline in `src/xmp/value_conversion.rs`
      (they exercise module-internal fns).*
- [x] **Task 2: Layer 1 port.** `convert_rational` + `convert_xmp_date`
      (exact XMP.pm:3383-3417 ports) in `src/xmp/` or
      `src/implementations/value_conv.rs`; apply in
      `src/xmp/processor.rs` where `final_value` is built, keyed off
      `XmpTagInfo.writable` == `rational`/`date`, before PrintConv.
      **Proof**: GPSAltitude, DateTime\*, GPSDateTime diffs clear in
      `compare-with-exiftool`. *Done in `src/xmp/value_conversion.rs`
      (`apply_writable_conversion` + the two ports), applied in
      `flatten_xmp_structure`.*
- [x] **Task 3: Layer 2 wiring.** Add `value_conv` to `XmpTagInfo` +
      populate via codegen xmp_tag strategy through the impl_registry
      (preferred — check how `codegen/src/impl_registry/` maps
      `Image::ExifTool::Exif::PrintExposureTime($val)` etc. for the
      EXIF path), OR document why a hand-registry in `src/xmp/` is
      acceptable. Extend `apply_xmp_print_conv` to handle
      Function-type conversions. **Proof**: ShutterSpeedValue,
      ApertureValue, FNumber, FocalLength diffs clear. *Done via the
      explicit registry (`apply_exif_photo_conv`) instead of the codegen
      route — see "Deviation from Task 3" above for the rationale.*
- [x] **Task 4: Regression + compat delta.** `make codegen fmt lint t`
      clean; run compat report; expect ≥8 net tag improvement
      (86→~94/191). Record the actual delta here. Check no regression
      on XMP tags that were already working (Rating, Subject, etc. —
      Layer 1 must not touch `Writable => 'string'/'integer'/'real'`
      tags; note `real` and `integer` are NOT converted by
      ConvertRational — only `rational` is).
      *Done: `make codegen fmt lint t` all clean (no generated drift;
      submodule re-cleaned via exiftool-patcher-undo.sh). Compat 86→94/191.
      String/integer/lang-alt tags (Rating, Subject, CreatorTool, …) still
      working — Layer 1 only touches `rational`/`date`.*
- [x] **Task 5: Update trackers.** Program TPP table row + session
      state; move this TPP to `_done/`. *Done 2026-07-03.*

## Review outcome (2026-07-03, adversarial subagent review — FIX-FIRST, then fixed)

Accepted and fixed before commit (all verified against the vendored
exiftool on synthetic element-form .xmp files, byte-exact after fix):

1. **ExposureCompensation regression** (was blocker-adjacent):
   exif:ExposureBiasValue is `Writable rational` + `PrintFraction`
   (XMP.pm:2096-2101). Layer 1 turned `"-1/3"` into `-0.333…` with no
   PrintConv — worse than before the change. Fixed with a
   `"ExposureCompensation"` arm in `apply_exif_photo_conv`.
2. **SubjectDistance**: same shape; PrintConv
   `'$val =~ /^(inf|undef)$/ ? $val : "$val m"'` (XMP.pm:2110-2115) now
   ported (`subject_distance_print_conv`).
3. **Bonus find — `print_fraction` itself was an inexact port** (shared
   with the EXIF path): it used `.floor()`+`.abs()` where Perl `int()`
   truncates toward zero with *signed* ratio tests (Exif.pm:5524-5529),
   so every negative EV printed as a wrong whole number (`-1/3` → `"-1"`);
   also `{:+.3}` where Perl uses `%+.3g` (now via `sprintf_perl`). Fixed +
   pinned with negative-fraction unit tests. Latent in compat only because
   no snapshot exercises a negative EV.
4. `numeric_value` now emits U64 for whole quotients beyond i32
   (e.g. `4294967295/1` focus-distance sentinels).

Documented known gaps (deliberately NOT fixed here):

- **XMPAutoConv** (XMP.pm:3675, on by default): unknown/IsDefault tags
  with rational/date-looking values are auto-converted by ExifTool but
  left raw by us; only affects tags outside the generated tables.
- **`inf`/`undef` flowing into Layer 2**: a zero-denominator
  ShutterSpeedValue/ExposureCompensation reaches the APEX/PrintFraction
  fns as a string and passes through (`"inf"`) or prints `Unknown (inf)`,
  where Perl numifies to `Inf` and yields `0`/`+inf`. Pathological
  corrupt-file territory; revisit only if a real file hits it.
- `lookup_xmp_tag_by_name` scans `HashMap.values()` — nondeterministic if
  a namespace had two properties with the same display name (none do
  today), and benign since only `writable` is consumed from the result.
- Attribute-form RDF (`exif:FNumber="8/1"` as XML attributes) is not
  extracted at all in standalone .xmp files — pre-existing, unrelated
  (see `_todo/P10-RDF-RESOURCE-ATTRIBUTES.md`).

## If architecture changed

- No `XmpTagInfo.writable`? Re-check `src/generated/XMP_pm/*_tags.rs`
  for whatever field now carries ExifTool's `Writable`; the goal is
  unchanged (format-keyed conversion).
- Goal is byte-parity with `exiftool -j -G` on the listed tags —
  verify with `cargo run --bin compare-with-exiftool <img> XMP:`.

## Files referenced

- `third-party/exiftool/lib/Image/ExifTool/XMP.pm:3383,3400,3673` —
  ConvertXMPDate, ConvertRational, FoundXMP call site
- `third-party/exiftool/lib/Image/ExifTool/XMP.pm:2040-2115,2161,2342` —
  exif-namespace per-tag conversions
- `src/xmp/processor.rs:112-220` — emit + PrintConv site (the hook)
- `src/core/xmp_tag_info.rs:9` — XmpTagInfo (needs value_conv)
- `src/generated/XMP_pm/exif_tags.rs` — generated table (writable ✓)
- `codegen/src/strategies/xmp_tag.rs` — table generator
- `src/implementations/value_conv.rs:65,91`, `print_conv.rs:267,307` —
  existing APEX/exposure/fnumber ports to reuse
- `generated/exiftool-json/*.json` — oracle snapshots for tests
