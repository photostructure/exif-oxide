# P03: Implementation Backlog

**Prerequisites**:

- P01 (Fix the Build) must be complete - `make precommit` passing
- P02 (Required Tags Audit) complete - gap analysis generated

---

## Overview

This backlog prioritizes implementation work based on the gap analysis in [required-tags-gap-analysis.json](../analysis/required-tags-gap-analysis.json).

**Key Finding**: 99.2% of required tags are in `supported_tags.json`, but "supported" doesn't mean "working correctly". The real gaps are:

| Category                | Tags | Effort   | Status                              |
| ----------------------- | ---- | -------- | ----------------------------------- |
| PrintConv not applied   | ~15  | Low      | ✅ **COMPLETE** (P03a)              |
| Format differences      | ~5   | Low      | ✅ **COMPLETE** (P03b)              |
| Composite tags missing  | ~12  | Medium   | ⚠️ **95% COMPLETE** - bugs remain   |
| Blocked on Milestone 18 | 22   | Blocked  | Skip                                |
| Research needed         | 20   | Research | ✅ **COMPLETE** (P03d)              |
| DNGLensInfo             | 1    | Low      | ✅ **COMPLETE** (P03e)              |
| XMP namespace tags      | 15   | Medium   | 🔲 NOT STARTED (P03f) - plan ready  |

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

**Status**: ⚠️ **95% COMPLETE** - Tasks 0-6 done, but bugs remain (validated 2025-12-11)

**Details**: See [P03c-composite-tags.md](P03c-composite-tags.md) for full implementation plan.

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

### Known Bugs (Fix Required)

| Tag          | Issue                                               | Root Cause                        |
| ------------ | --------------------------------------------------- | --------------------------------- |
| Megapixels   | Shows `"%.*f"` instead of number                    | sprintf_perl missing value arg    |
| ShutterSpeed | Shows "1/1" instead of correct value                | Fallback implementation bug       |
| GPSPosition  | Wrong sign on longitude, precision differs          | Sign conversion in fallback       |

---

## Blocked on Milestone 18 (22 tags) - DO NOT IMPLEMENT

**QuickTime** (18 tags): CompressorName, CreateDate, CreationDate, Duration, HandlerDescription, ImageHeight, ImageWidth, Make, MediaCreateDate, MediaDuration, MediaModifyDate, Model, ModifyDate, Software, TrackCreateDate, TrackDuration, TrackModifyDate, Rotation

**RIFF** (4 tags): DateTimeOriginal, Duration, ImageHeight, ImageWidth, Software

**Status**: Blocked until video format support is implemented. Skip these entirely.

---

## Research Complete (P03d) - Unknown Tags

**Status**: COMPLETE - See [P03d-unknown-tags-research.md](P03d-unknown-tags-research.md)

**Outcome**: 16 tags researched with ExifTool source references. Full details in [unknown-tags-research.md](../analysis/unknown-tags-research.md).

**Follow-up TPPs**:
- **P03e**: DNGLensInfo (EXIF tag 0xc630) - ✅ **COMPLETE** (already in codegen)
- **[P03f](P03f-xmp-namespace-tags.md)**: XMP Namespace Tags - ✅ **COMPLETE** (40 tables, 719 tags generated)
- **[P03g](P03g-xmp2-mwg-codegen.md)**: XMP2.pl + MWG.pm Integration - 🔲 **NOT STARTED** (15 tags blocked)

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
- Source: [Exif_pm/main_tags.rs](../../src/generated/Exif_pm/main_tags.rs) (generated)
- PrintConv: `lensinfo_print_conv` in [print_conv.rs](../../src/implementations/print_conv.rs)

---

## XMP Namespace Tags (P03f)

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

XmpTagStrategy created, 40 XMP namespace tables generated with 719 total tags.

**Completed**:
- XmpTagInfo type (`src/core/xmp_tag_info.rs`)
- XmpTagStrategy (`codegen/src/strategies/xmp_tag.rs`)
- 40 namespace tables in `src/generated/XMP_pm/`
- xmp_lookup.rs wired to generated tables

**Details**: See [P03f-xmp-namespace-tags.md](P03f-xmp-namespace-tags.md)

---

## XMP2.pl + MWG.pm Codegen (P03g)

**Status**: ✅ **COMPLETE** (validated 2025-12-11)

**Problem**: P03f generated XMP tables from XMP.pm, but XMP2.pl tables (cc, MediaPro, iptcExt) are lazily loaded and not captured. MWG.pm not in module list.

**Root Cause**: ExifTool.pm:8900 loads XMP2.pl on-demand via GetTagTable, not during require.

**Solution**:
1. Modify field_extractor.pl to explicitly require XMP2.pl when processing XMP.pm
2. Add MWG.pm to exiftool_modules.json
3. Update xmp_lookup.rs with new namespace routing

**Tags blocked** (15):
- **XMP-cc** (8): License, AttributionName, AttributionURL, UseGuidelines, Permits, Requires, Prohibits, Jurisdiction
- **XMP-mwg-rs** (1): RegionList
- **XMP-mwg-kw** (2): KeywordInfo, HierarchicalKeywords
- **XMP-Iptc4xmpExt** (2): PersonInImageWDetails, PersonInImageName
- **XMP-mediapro** (1): People

**Details**: See [P03g-xmp2-mwg-codegen.md](P03g-xmp2-mwg-codegen.md)

---

## Implementation Order

1. ✅ **P03a**: PrintConv application - **COMPLETE**
2. ✅ **P03b**: Format fixes - **COMPLETE**
3. ⚠️ **P03c**: Composite tag infrastructure - **95% COMPLETE** (bugs remain)
4. ✅ **P03d**: Research unknown tags - **COMPLETE**
5. ✅ **P03e**: DNGLensInfo - **COMPLETE** (already in codegen)
6. ✅ **P03f**: XMP Namespace Tags - **COMPLETE** (40 tables, 719 tags)
7. ✅ **P03g**: XMP2.pl + MWG.pm Integration - **COMPLETE** (15 tags unblocked)

### Next Steps

1. **P03c bugs**: Fix Megapixels sprintf, ShutterSpeed fallback, GPSPosition sign

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
