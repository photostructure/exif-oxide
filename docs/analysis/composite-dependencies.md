# Composite Tag Dependency Analysis

## Executive Summary

This analysis identifies missing base tag dependencies for composite tags defined in `config/supported_tags.json`. Of the 27 composite tags currently supported, several have missing base tag dependencies that will prevent proper calculation.

## Key Findings

### Critical Missing Dependencies

#### ❌ High Priority Missing Base Tags

1. **`Composite:Aperture`** - Missing core base tags:
   - `EXIF:FNumber` ✅ (supported)
   - `EXIF:ApertureValue` ❌ (missing)

2. **`Composite:ShutterSpeed`** - Missing base tags:
   - `EXIF:ExposureTime` ✅ (supported)
   - `EXIF:ShutterSpeedValue` ❌ (missing)
   - `BulbDuration` ❌ (missing)

3. **`Composite:ISO`** - Base tag availability unclear:
   - `EXIF:ISO` ✅ (supported)
   - `EXIF:ISOSpeed` ✅ (supported)
   - Various manufacturer-specific ISO tags ❌ (likely missing)

#### ✅ Well-Supported Composites

1. **`Composite:ImageSize`** - All dependencies available:
   - **Required:** `ImageWidth`, `ImageHeight` ✅
   - **Desired:** `EXIF:ExifImageWidth`, `EXIF:ExifImageHeight` ✅

2. **`Composite:Megapixels`** - Dependencies available:
   - **Required:** `Composite:ImageSize` ✅ (supported)

3. **GPS Composites** - Most dependencies available:
   - `Composite:GPSLatitude` → `EXIF:GPSLatitude`, `EXIF:GPSLatitudeRef` ✅
   - `Composite:GPSLongitude` → `EXIF:GPSLongitude`, `EXIF:GPSLongitudeRef` ✅  
   - `Composite:GPSPosition` → Uses above GPS composites ✅
   - `Composite:GPSDateTime` → `EXIF:GPSDateStamp`, `EXIF:GPSTimeStamp` ✅

### Dependency Pattern Analysis

Based on ExifTool source analysis, composite tags use three dependency mechanisms:

#### 1. **Require** - Mandatory Dependencies
Tags that **must** be present for calculation:
```perl
ImageSize => {
    Require => {
        0 => 'ImageWidth',
        1 => 'ImageHeight',
    },
}
```

#### 2. **Desire** - Optional Dependencies  
Tags that are preferred but not mandatory (provides fallback options):
```perl
Aperture => {
    Desire => {
        0 => 'FNumber',
        1 => 'ApertureValue',
    },
}
```

#### 3. **Inhibit** - Conditional Exclusions
Prevents calculation when certain conditions are met:
```perl
Lens => {
    Inhibit => {
        4 => 'Composite:LensID',
    },
}
```

## Detailed Dependency Breakdown

### Camera/Exposure Composites

| Composite Tag | Base Tag Dependencies | Status |
|---------------|----------------------|---------|
| `Composite:Aperture` | `EXIF:FNumber` ✅, `EXIF:ApertureValue` ❌ | **Partial** |
| `Composite:ShutterSpeed` | `EXIF:ExposureTime` ✅, `EXIF:ShutterSpeedValue` ❌, `BulbDuration` ❌ | **Partial** |
| `Composite:ISO` | `EXIF:ISO` ✅, `EXIF:ISOSpeed` ✅, various MakerNotes ISO tags | **Partial** |

### Image Dimension Composites

| Composite Tag | Base Tag Dependencies | Status |
|---------------|----------------------|---------|
| `Composite:ImageSize` | **Require:** `ImageWidth` ✅, `ImageHeight` ✅<br>**Desire:** `EXIF:ExifImageWidth` ✅, `EXIF:ExifImageHeight` ✅ | **Complete** |
| `Composite:Megapixels` | **Require:** `Composite:ImageSize` ✅ | **Complete** |

### GPS Location Composites

| Composite Tag | Base Tag Dependencies | Status |
|---------------|----------------------|---------|
| `Composite:GPSLatitude` | **Require:** `EXIF:GPSLatitude` ✅, `EXIF:GPSLatitudeRef` ✅ | **Complete** |
| `Composite:GPSLongitude` | **Require:** `EXIF:GPSLongitude` ✅, `EXIF:GPSLongitudeRef` ✅ | **Complete** |
| `Composite:GPSPosition` | **Require:** `Composite:GPSLatitude` ✅, `Composite:GPSLongitude` ✅ | **Complete** |
| `Composite:GPSAltitude` | **Desire:** `EXIF:GPSAltitude` ✅, `EXIF:GPSAltitudeRef` ✅, `XMP:GPSAltitude` ✅, `XMP:GPSAltitudeRef` ✅ | **Complete** |
| `Composite:GPSDateTime` | **Require:** `EXIF:GPSDateStamp` ✅, `EXIF:GPSTimeStamp` ✅ | **Complete** |

### DateTime Composites

| Composite Tag | Base Tag Dependencies | Status |
|---------------|----------------------|---------|
| `Composite:DateTimeCreated` | **Require:** `IPTC:DateCreated`, `IPTC:TimeCreated` | **Needs Review** |
| `Composite:DateTimeOriginal` | **Require:** Various date/time sources | **Needs Review** |
| `Composite:SubSecCreateDate` | **Require:** `EXIF:CreateDate`, `EXIF:SubSecTime` ✅ | **Needs Review** |
| `Composite:SubSecDateTimeOriginal` | **Require:** `EXIF:DateTimeOriginal` ✅, `EXIF:SubSecTimeOriginal` ✅ | **Complete** |
| `Composite:SubSecModifyDate` | **Require:** `EXIF:ModifyDate` ✅, `EXIF:SubSecTimeDigitized` ✅ | **Complete** |

### Lens/Camera Equipment Composites

| Composite Tag | Base Tag Dependencies | Status |
|---------------|----------------------|---------|
| `Composite:Lens` | Complex logic using various `LensModel`, `LensInfo`, `LensSpec` tags | **Needs Review** |
| `Composite:LensID` | Manufacturer-specific lens ID mappings | **Needs Review** |
| `Composite:LensSpec` | `EXIF:LensInfo`, `EXIF:DNGLensInfo`, various MakerNotes | **Partial** |
| `Composite:LensType` | Manufacturer-specific type mappings | **Needs Review** |

## Missing Base Tags Analysis

### Priority 1: Critical Missing Tags

These base tags should be added to enable proper composite calculation:

```json
[
  "EXIF:ApertureValue",
  "EXIF:ShutterSpeedValue"
]
```

**Impact:** Without these, `Composite:Aperture` and `Composite:ShutterSpeed` will have reduced functionality.

### Priority 2: Enhanced Functionality Tags

These tags would improve composite tag coverage but aren't strictly required:

```json
[
  "IPTC:DateCreated", 
  "IPTC:TimeCreated",
  "MakerNotes:BulbDuration"
]
```

### Priority 3: Manufacturer-Specific Extensions

Manufacturer-specific tags that provide additional data sources for composites:

```json
[
  "Canon:ApertureValue",
  "Nikon:ShutterSpeedValue", 
  "Sony:ISO",
  "Canon:LensType",
  "Nikon:LensID"
]
```

## Mitigation Strategies

### 1. Immediate Actions

**Add Critical Missing Base Tags:**
- Add `EXIF:ApertureValue` to supported_tags.json
- Add `EXIF:ShutterSpeedValue` to supported_tags.json
- Verify these tags are properly extracted in the EXIF processor

### 2. Graceful Degradation

**Implement Fallback Logic:**
- For `Composite:Aperture`: Use `EXIF:FNumber` when `EXIF:ApertureValue` unavailable
- For `Composite:ShutterSpeed`: Use `EXIF:ExposureTime` when `EXIF:ShutterSpeedValue` unavailable
- Document which base tags are available vs. missing for each composite

### 3. Validation Framework

**Composite Tag Validation:**
- Create test cases that verify composite tags work with available base tags
- Add logging when composite calculation fails due to missing dependencies
- Implement dependency checking before attempting composite calculation

### 4. Future Enhancements

**Manufacturer-Specific Support:**
- Gradually add manufacturer-specific base tags based on usage frequency
- Prioritize Canon, Nikon, Sony tags as they're most common
- Use tag frequency analysis from `docs/tag-metadata.json` to guide priorities

## Verification Results

### ExifTool Testing with Real Images

**Test Image:** `test-images/oneplus/gm1917_01.jpg` (OnePlus camera with GPS)

**Composite Tags Successfully Generated:**
```
[Composite] Aperture                        : 2.4
[Composite] ImageSize                       : 3264x2448
[Composite] Megapixels                      : 8.0
[Composite] ScaleFactor35efl                : 10.6
[Composite] ShutterSpeed                    : 1/877
[Composite] SubSecCreateDate                : 2019:06:06 12:53:36.743048
[Composite] SubSecDateTimeOriginal          : 2019:06:06 12:53:36.743048
[Composite] SubSecModifyDate                : 2019:06:06 12:53:36.743048
[Composite] GPSAltitude                     : 6.9 m Below Sea Level
[Composite] GPSDateTime                     : 2019:06:06 19:53:34Z
[Composite] GPSLatitude                     : 37 deg 30' 15.60" N
[Composite] GPSLongitude                    : 122 deg 28' 34.36" W
[Composite] CircleOfConfusion               : 0.003 mm
[Composite] FOV                             : 27.3 deg
[Composite] FocalLength35efl                : 7.0 mm (35 mm equivalent: 74.0 mm)
[Composite] GPSPosition                     : 37 deg 30' 15.60" N, 122 deg 28' 34.36" W
[Composite] HyperfocalDistance              : 7.02 m
[Composite] LightValue                      : 12.4
```

**Base Tag Availability Confirmed:**
- ✅ `FNumber`: 2.4, `ApertureValue`: 2.4 → `Composite:Aperture` works
- ✅ `ExposureTime`: 1/877, `ShutterSpeedValue`: 1/877 → `Composite:ShutterSpeed` works  
- ✅ GPS base tags present → All GPS composites work
- ✅ DateTime base tags present → SubSec DateTime composites work

**Additional Composite Tags Found:**
The verification revealed several composite tags not currently in our `supported_tags.json`:
- `ScaleFactor35efl` - Camera-specific scaling factor
- `CircleOfConfusion` - Optical calculation for depth of field
- `FOV` - Field of view calculation
- `FocalLength35efl` - 35mm equivalent focal length
- `HyperfocalDistance` - Photography calculation
- `LightValue` - Exposure value calculation

### Fallback Behavior Verified

**Test with Olympus E1:** Only `FNumber` available (no `ApertureValue`), but `Composite:Aperture` still works, confirming ExifTool's fallback logic using the `Desire` mechanism.

### Testing Recommendations

```bash
# Test composite tags with real images
./third-party/exiftool/exiftool -G -s test-images/oneplus/gm1917_01.jpg | grep "^\[Composite\]"

# Compare our implementation vs ExifTool  
cargo run --bin compare-with-exiftool test-images/oneplus/gm1917_01.jpg Composite:

# Test specific dependencies
./third-party/exiftool/exiftool -EXIF:ApertureValue -EXIF:FNumber -Composite:Aperture test-images/oneplus/gm1917_01.jpg
```

### Expected Results

- ✅ `Composite:ImageSize` and `Composite:Megapixels` work consistently
- ✅ GPS composites work for images with GPS data  
- ✅ `Composite:Aperture` and `Composite:ShutterSpeed` work when base tags available
- ⚠️ Photography calculation composites (`LightValue`, `FOV`, `HyperfocalDistance`) require additional base tags

## Implementation Priority

1. **Phase 1:** Add `EXIF:ApertureValue` and `EXIF:ShutterSpeedValue` to supported tags
2. **Phase 2:** Implement graceful degradation for missing dependencies
3. **Phase 3:** Add comprehensive testing for all composite tag scenarios
4. **Phase 4:** Gradually expand manufacturer-specific base tag support

## Conclusion

While most GPS and image dimension composites are well-supported, key photography composites (`Aperture`, `ShutterSpeed`) have missing dependencies that should be addressed. The missing tags are easily identifiable and can be added to improve composite tag functionality significantly.

The analysis shows that exif-oxide's current approach of supporting core EXIF tags provides a solid foundation, but strategic addition of a few key missing base tags would dramatically improve composite tag coverage.