# P03: Implementation Backlog

**Prerequisites**:

- P01 (Fix the Build) must be complete - `make precommit` passing
- P02 (Required Tags Audit) complete - gap analysis generated

---

## Overview

This backlog prioritizes implementation work based on the gap analysis in [required-tags-gap-analysis.json](../analysis/required-tags-gap-analysis.json).

**Key Finding**: 99.2% of required tags are in `supported_tags.json`, but "supported" doesn't mean "working correctly". The real gaps are:

| Category                | Tags | Effort   | Status                |
| ----------------------- | ---- | -------- | --------------------- |
| PrintConv not applied   | ~15  | Low      | **Start Here**        |
| Format differences      | ~5   | Low      | Quick wins            |
| Composite tags missing  | ~12  | Medium   | Infrastructure needed |
| Blocked on Milestone 18 | 22   | Blocked  | Skip                  |
| Research needed         | 20   | Research | Identify sources      |

---

## Quick Wins (P03a) - PrintConv Application

**Problem**: Tags show raw numeric values instead of human-readable strings.

**Why it matters**: Users expect "Multi-segment" not "5" for MeteringMode.

**Solution**: Apply existing PrintConv lookup tables during tag output.

**Tags affected**:

| Tag                   | Current Output | Expected Output     |
| --------------------- | -------------- | ------------------- |
| EXIF:MeteringMode     | 5              | Multi-segment       |
| EXIF:ResolutionUnit   | 2              | inches              |
| EXIF:Orientation      | 1              | Horizontal (normal) |
| EXIF:Flash            | 0              | No Flash            |
| EXIF:YCbCrPositioning | 1              | Centered            |
| EXIF:ExposureProgram  | 2              | Program AE          |
| EXIF:GPSLatitudeRef   | N              | North               |
| EXIF:GPSLongitudeRef  | W              | West                |
| EXIF:GPSAltitudeRef   | 0              | Above Sea Level     |

**Implementation approach**:

1. These tags already have PrintConv defined in `src/generated/`
2. The issue is PrintConv is not being applied in the output pipeline
3. Check `src/compat/` for normalization logic
4. Pattern to follow: search for tags where PrintConv IS being applied

**Verification**:

```bash
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep -E "MeteringMode|ResolutionUnit"
# Should show matching values after fix
```

---

## Quick Wins (P03b) - Format Fixes

**Problem**: Minor formatting differences between exif-oxide and ExifTool output.

**Tags affected**:

| Tag                    | exif-oxide | ExifTool    | Fix                           |
| ---------------------- | ---------- | ----------- | ----------------------------- |
| File:FileTypeExtension | jpeg       | jpg         | Use canonical short extension |
| MakerNotes:FileNumber  | 1181861    | 118-1861    | Apply PrintConv regex         |
| EXIF:ShutterSpeedValue | 4.3        | 1/20        | Apply PrintExposureTime       |
| EXIF:GPSTimeStamp      | 17:17:58   | 17:17:58.65 | Preserve fractional seconds   |

**Implementation approach**:

1. FileTypeExtension: Check `src/file/` for extension logic
2. FileNumber: Verify PrintConv `$_=$val;s/(\d+)(\d{4})/$1-$2/;$_` is applied
3. ShutterSpeedValue: Use `PrintExposureTime` function
4. GPSTimeStamp: Check rational-to-string conversion preserves decimals

---

## Medium Effort (P03c) - Composite Tags

**Problem**: Composite tags (calculated from other tag values) are listed as supported but not being generated.

**Why it matters**: Many commonly-used tags like Aperture, ShutterSpeed, Megapixels, ImageSize, and GPS coordinates are Composite.

**Tags affected** (from [composite-dependencies.json](../analysis/expressions/composite-dependencies.json)):

| Tag                    | Dependencies                                  | Expression                |
| ---------------------- | --------------------------------------------- | ------------------------- |
| Composite:Aperture     | FNumber, ApertureValue                        | `$val[0] \|\| $val[1]`    |
| Composite:ShutterSpeed | ExposureTime, ShutterSpeedValue, BulbDuration | Ternary selection         |
| Composite:Megapixels   | ImageSize                                     | `$d[0] * $d[1] / 1000000` |
| Composite:ImageSize    | ImageWidth, ImageHeight, ExifImage\*          | Complex selection         |
| Composite:ISO          | CameraISO, BaseISO, AutoISO                   | Canon-specific calc       |
| Composite:Lens         | MinFocalLength, MaxFocalLength                | PrintFocalRange           |
| Composite:GPSLatitude  | GPSLatitude, GPSLatitudeRef                   | Sign based on ref         |
| Composite:GPSLongitude | GPSLongitude, GPSLongitudeRef                 | Sign based on ref         |
| Composite:GPSPosition  | GPSLatitude, GPSLongitude                     | String concat             |
| Composite:GPSDateTime  | GPSDateStamp, GPSTimeStamp                    | Concat with Z             |

**Implementation approach**:

1. Check existing composite infrastructure in `src/composite/` or `src/implementations/`
2. Composite tags need access to previously-extracted tag values
3. Follow pattern from `docs/analysis/expressions/composite-dependencies.json`
4. Consider: are we generating composite tags but not outputting them?

**Key investigation**:

```bash
rg "Composite" src/ --type rust | head -20
rg "composite" src/implementations/ --type rust
```

---

## Blocked on Milestone 18 (22 tags) - DO NOT IMPLEMENT

**QuickTime** (18 tags): CompressorName, CreateDate, CreationDate, Duration, HandlerDescription, ImageHeight, ImageWidth, Make, MediaCreateDate, MediaDuration, MediaModifyDate, Model, ModifyDate, Software, TrackCreateDate, TrackDuration, TrackModifyDate, Rotation

**RIFF** (4 tags): DateTimeOriginal, Duration, ImageHeight, ImageWidth, Software

**Status**: Blocked until video format support is implemented. Skip these entirely.

---

## Research Needed (P03d) - Unknown Tags

**Problem**: 20 required tags are not in `tag-metadata.json` - need to identify which ExifTool module provides them.

**Tags**:

```
AttributionName, AttributionURL, CameraModelName, DNGLensInfo, FileVersion,
HierarchicalKeywords, HistoryWhen, ImageDataHash, Jurisdiction, KeywordInfo,
License, People, Permits, PersonInImageName, PersonInImageWDetails, Prohibits,
RegionList, Requires, UseGuidelines
```

**Likely sources** (based on naming patterns):

| Source Module                | Tags                                                                                                |
| ---------------------------- | --------------------------------------------------------------------------------------------------- |
| XMP-cc (Creative Commons)    | AttributionName, AttributionURL, License, Permits, Prohibits, Requires, UseGuidelines, Jurisdiction |
| XMP-mwg-rs (Regions)         | RegionList, PersonInImageWDetails                                                                   |
| XMP-MP (Microsoft Photo)     | PersonInImageName                                                                                   |
| XMP-xmpMM (Media Management) | HistoryWhen                                                                                         |
| XMP-lr (Lightroom)           | HierarchicalKeywords                                                                                |
| XMP-crs (Camera Raw)         | CameraModelName                                                                                     |
| EXIF/DNG                     | DNGLensInfo                                                                                         |
| File module                  | ImageDataHash                                                                                       |

**Research approach**:

```bash
# Search ExifTool source for tag definitions
rg "AttributionName|HierarchicalKeywords" third-party/exiftool/lib/Image/ExifTool/
```

---

## Implementation Order

1. **P03a**: PrintConv application (highest impact, lowest effort)
2. **P03b**: Format fixes (quick wins)
3. **P03c**: Composite tag infrastructure (enables many tags)
4. **P03d**: Research unknown tags (parallel work)

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
