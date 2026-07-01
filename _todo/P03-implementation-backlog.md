# P03: Implementation Backlog

**Verified 2026-07-01**: Of the 3 "Known Bugs" listed below, 2 are already fixed and 1 remains open:
- Megapixels sprintf: FIXED. `cargo run -- third-party/exiftool/t/images/Canon.jpg` and 3 other test images all show numeric values matching `exiftool -Megapixels`, not literal `"%.*f"`.
- ShutterSpeed fallback: FIXED. Checked against 4 test images (Canon.jpg, Nikon.jpg, Sony.jpg, iPhone samples) - all match `exiftool -ShutterSpeed` exactly (e.g. "1/332", "1/213"), not "1/1".
- GPSPosition sign: STILL BROKEN. `cargo run -- test-images/apple/IMG_3755.JPG` shows `"Composite:GPSPosition": "37.52583939995588 122.45673460002102"` (positive longitude) vs ExifTool's `"37.5258393999556 -122.456734600022"` (negative, correct for West). Root cause: `composite_gps_position()` in `src/core/composite_fallbacks.rs:391-408` concatenates `vals[0]`/`vals[1]` directly with `format!("{} {}", lat_val, lon_val)`, but the values it receives are not sign-adjusted for hemisphere (N/S, E/W) the way ExifTool's `Composite:GPSLatitude`/`Composite:GPSLongitude` are before being fed into `Composite:GPSPosition` (Exif.pm:5165-5196).

**Prerequisites** (both long since satisfied - P01/P02 never had standalone files):

- P01 (Fix the Build) - `cargo build --release` succeeds (verified 2026-07-01)
- P02 (Required Tags Audit) - `docs/analysis/required-tags-gap-analysis.json` exists

---

## Overview

This backlog prioritizes implementation work based on the gap analysis in [required-tags-gap-analysis.json](../docs/analysis/required-tags-gap-analysis.json).

**Key Finding**: 99.2% of required tags are in `supported_tags.json`, but "supported" doesn't mean "working correctly". The real gaps are:

| Category                | Tags | Effort   | Status                                    |
| ----------------------- | ---- | -------- | ------------------------------------------ |
| PrintConv not applied   | ~15  | Low      | ✅ **COMPLETE** (P03a)                     |
| Format differences      | ~5   | Low      | ✅ **COMPLETE** (P03b)                     |
| Composite tags missing  | ~12  | Medium   | ⚠️ **~99% COMPLETE** - 1 bug remains (P03c) |
| Blocked on Milestone 18 | 22   | Blocked  | Skip                                       |
| Research needed         | 20   | Research | ✅ **COMPLETE** (P03d)                     |
| DNGLensInfo             | 1    | Low      | ✅ **COMPLETE** (P03e)                     |
| XMP namespace tags      | 15   | Medium   | ✅ **COMPLETE** (P03f/P03g)                |

---

## Quick Wins (P03a) - PrintConv Application

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

PrintConv lookup tables are now applied correctly during tag output.

**Verified working**:

| Tag                   | Output              |
| --------------------- | ------------------- |
| EXIF:MeteringMode     | Average             |
| EXIF:ResolutionUnit   | inches              |
| EXIF:Orientation      | Horizontal (normal) |
| EXIF:YCbCrPositioning | Centered            |

**Verification**:

```bash
cargo run -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep -E "MeteringMode|ResolutionUnit|Orientation"
# Shows human-readable values, not numeric codes
```

---

## Quick Wins (P03b) - Format Fixes

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

Format fixes for basic EXIF tags are working. Remaining format issues are composite-related (see P03c).

**Verified working**:

| Tag                    | Output |
| ---------------------- | ------ |
| File:FileTypeExtension | jpg    |

**Note**: ShutterSpeedValue formatting for composite tags is tracked in P03c Task 6.

---

## Medium Effort (P03c) - Composite Tags

**Status**: ⚠️ **~99% COMPLETE** - Tasks 0-6 done, 2 of 3 bugs fixed, 1 remains (verified 2026-07-01, see note at top of this document)

**Details**: P03c never had a standalone file - full plan is inline below.

### Infrastructure Complete (Tasks 0-6)

- ✅ 46 composite functions generated via PPI pipeline
- ✅ 29 value_conv function pointers set
- ✅ 16 print_conv function pointers set
- ✅ Runtime orchestration enabled
- ✅ 31 fallback functions in `src/core/composite_fallbacks.rs`
- ✅ Legacy files deleted (`implementations.rs`, `dispatch.rs`)
- ✅ `make lint` passes

**Working composite tags**:
- SubSecCreateDate, SubSecDateTimeOriginal, SubSecModifyDate
- Aperture, ISO, Lens, LensID (Canon.jpg)
- GPSLatitude, GPSLongitude (numeric values)

### Known Bugs

| Tag          | Issue                                                   | Status (2026-07-01)                                                                                     |
| ------------ | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Megapixels   | Was: shows `"%.*f"` instead of number                     | ✅ FIXED - verified against 4 test images, matches ExifTool exactly                                          |
| ShutterSpeed | Was: shows "1/1" instead of correct value                 | ✅ FIXED - verified against 4 test images, matches ExifTool exactly                                          |
| GPSPosition  | Wrong sign on longitude (missing hemisphere sign flip)    | 🔴 OPEN - see verification note above; fix in `composite_gps_position()` (`src/core/composite_fallbacks.rs:391`) |

---

## Blocked on Milestone 18 (22 tags) - DO NOT IMPLEMENT

**QuickTime** (18 tags): CompressorName, CreateDate, CreationDate, Duration, HandlerDescription, ImageHeight, ImageWidth, Make, MediaCreateDate, MediaDuration, MediaModifyDate, Model, ModifyDate, Software, TrackCreateDate, TrackDuration, TrackModifyDate, Rotation

**RIFF** (4 tags): DateTimeOriginal, Duration, ImageHeight, ImageWidth, Software

**Status**: Blocked until video format support is implemented. Skip these entirely.

---

## Research Complete (P03d) - Unknown Tags

**Status**: COMPLETE - moved to [../_done/P03d-unknown-tags-research.md](../_done/P03d-unknown-tags-research.md) 2026-07-01

**Outcome**: 16 tags researched with ExifTool source references. Full details in [unknown-tags-research.md](../docs/analysis/unknown-tags-research.md).

**Follow-up TPPs** (none of P03e/f/g ever had standalone files - inline below):
- **P03e**: DNGLensInfo (EXIF tag 0xc630) - ✅ **COMPLETE** (already in codegen)
- **P03f**: XMP Namespace Tags - ✅ **COMPLETE** (40 tables, 719 tags generated)
- **P03g**: XMP2.pl + MWG.pm Integration - ✅ **COMPLETE** (verified 2026-07-01: `src/generated/XMP_pm/cc_tags.rs` and `src/generated/MWG_pm/` exist; 15 tags unblocked - see status below, this line previously said "NOT STARTED" which was stale)

---

## DNGLensInfo (P03e) - EXIF Tag

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

DNGLensInfo (tag ID 0xc630 / 50736) was already implemented through codegen with `lensinfo_print_conv` PrintConv.

**Verification**:
```bash
# ExifTool output:
exiftool -DNGLensInfo third-party/exiftool/t/images/DNG.dng
# DNG Lens Info : 18-55mm f/?

# exif-oxide output (matches exactly):
cargo run -- third-party/exiftool/t/images/DNG.dng 2>/dev/null | grep DNGLensInfo
# "EXIF:DNGLensInfo": "18-55mm f/?"
```

**Implementation details**:
- Tag ID: 0xc630 (50736 decimal)
- Source: [Exif_pm/main_tags.rs](../src/generated/Exif_pm/main_tags.rs) (generated)
- PrintConv: `lensinfo_print_conv` in [print_conv.rs](../src/implementations/print_conv.rs)

---

## XMP Namespace Tags (P03f)

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

XmpTagStrategy created, 40 XMP namespace tables generated with 719 total tags.

**Completed**:
- XmpTagInfo type (`src/core/xmp_tag_info.rs`)
- XmpTagStrategy (`codegen/src/strategies/xmp_tag.rs`)
- 40 namespace tables in `src/generated/XMP_pm/`
- xmp_lookup.rs wired to generated tables

**Details**: P03f never had a standalone file - full plan is inline above.

---

## XMP2.pl + MWG.pm Codegen (P03g)

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

**Problem**: P03f generated XMP tables from XMP.pm, but XMP2.pl tables (cc, MediaPro, iptcExt) are lazily loaded and not captured. MWG.pm not in module list.

**Root Cause**: ExifTool.pm:8900 loads XMP2.pl on-demand via GetTagTable, not during require.

**Solution**:
1. Modify field_extractor.pl to explicitly require XMP2.pl when processing XMP.pm
2. Add MWG.pm to exiftool_modules.json
3. Update xmp_lookup.rs with new namespace routing

**Tags unblocked** (15, codegen now generates their tables - extraction correctness for some is still tracked separately in `_done/P03d-unknown-tags-research.md`'s ignored tests):
- **XMP-cc** (8): License, AttributionName, AttributionURL, UseGuidelines, Permits, Requires, Prohibits, Jurisdiction
- **XMP-mwg-rs** (1): RegionList
- **XMP-mwg-kw** (2): KeywordInfo, HierarchicalKeywords
- **XMP-Iptc4xmpExt** (2): PersonInImageWDetails, PersonInImageName
- **XMP-mediapro** (1): People

**Details**: P03g never had a standalone file - full plan is inline above.

---

## Implementation Order

1. ✅ **P03a**: PrintConv application - **COMPLETE**
2. ✅ **P03b**: Format fixes - **COMPLETE**
3. ⚠️ **P03c**: Composite tag infrastructure - **~99% COMPLETE** (1 of 3 bugs remains - GPSPosition sign)
4. ✅ **P03d**: Research unknown tags - **COMPLETE** (moved to `_done/P03d-unknown-tags-research.md`)
5. ✅ **P03e**: DNGLensInfo - **COMPLETE** (already in codegen)
6. ✅ **P03f**: XMP Namespace Tags - **COMPLETE** (40 tables, 719 tags)
7. ✅ **P03g**: XMP2.pl + MWG.pm Integration - **COMPLETE** (15 tags unblocked)

Note: P03a/b/c/e/f/g never existed as separate files in `_todo/` or `docs/todo/` - their details are inline in this document. Only P03d had (and P03 itself has) a standalone file.

### Next Steps

1. **GPSPosition sign bug**: Fix `composite_gps_position()` in `src/core/composite_fallbacks.rs:391` to apply hemisphere sign (N/S, E/W) before concatenating lat/lon, matching ExifTool's Composite:GPSLatitude/GPSLongitude semantics (Exif.pm:5165-5196). Megapixels and ShutterSpeed bugs are already fixed - no action needed.

---

## Quality Checklist

- [ ] `make precommit` passes after each PR
- [ ] Each fix verified with `compare-with-exiftool` tool
- [ ] No regressions in existing compatibility tests
- [ ] Follow Trust ExifTool principle - copy behavior exactly

---

## Emergency Recovery

```bash
# If something breaks
git status  # Check what changed
git diff docs/  # Review doc changes
git checkout HEAD -- src/  # Revert source changes if needed

# Validate before declaring success
make precommit
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg
```
