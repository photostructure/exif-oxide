# Technical Project Plan: EXIF Required Tags Implementation

## Project Overview

- **Goal**: Ensure all standard EXIF tags marked as required in PhotoStructure are properly extracted
- **Problem**: Need comprehensive support for 36 EXIF tags that are marked as required in tag-metadata.json

## Background & Context

- EXIF tags are standardized across all camera manufacturers
- These 18 tags form the foundation of image metadata
- Most are already partially implemented but need verification

## Technical Foundation

- **Key files**:
  - `src/exif/tags.rs` - Tag definitions and creation
  - `src/exif/ifd.rs` - IFD parsing
  - `src/value_extraction.rs` - Value extraction from binary data
  - `src/generated/tags/` - Generated tag constants
- **Standards**: EXIF 2.32 specification

## Required EXIF Tags (36 total)

### Core Camera Settings (8 tags)
- **ApertureValue** (0x9202) - APEX aperture value - freq 0.390
- **ExposureTime** (0x829A) - Shutter speed in seconds - freq 0.990
  - Note: Sony cameras also write SonyExposureTime with potentially higher precision
- **FNumber** (0x829D) - F-stop value - freq 0.970
  - Note: Sony cameras also write SonyFNumber with lens corrections
- **FocalLength** (0x920A) - Lens focal length in mm - freq 0.950
- **ISO** (0x8827) - ISO sensitivity - freq 0.890
  - Note: Sony cameras also write SonyISO with extended range info
- **ISOSpeed** (0x8833) - ISO speed ratings - freq 0.002
- **ShutterSpeedValue** (0x9201) - APEX shutter speed - freq 0.380
- **MaxApertureValue** (0x9205) - Smallest F number of lens - freq 0.380

### Timestamps (7 tags)
- **CreateDate** (0x9004) - When image was created - freq 0.970
- **DateTimeOriginal** (0x9003) - When photo was taken - freq 0.970
- **DateTimeDigitized** (0x9004) - When digitized - freq 0.004
- **ModifyDate** (0x0132) - File modification time - freq 0.890
- **SubSecTime** (0x9290) - Subsecond timestamps - freq 0.083
- **SubSecTimeDigitized** (0x9292) - Subsecond for digitized - freq 0.084
- **DateTime** (0x0132) - File change date/time - freq 0.000

### Image Properties (6 tags)
- **ImageWidth** (0x0100) - Image dimensions - freq 1.000
- **ImageHeight** (0x0101) - Image dimensions - freq 1.000
- **Orientation** (0x0112) - Rotation/flip info - freq 0.920
- **ImageDescription** (0x010E) - User description - freq 0.430
- **ExifImageWidth** (0xA002) - Valid image width - freq 0.980
- **ExifImageHeight** (0xA003) - Valid image height - freq 0.980

### Camera/Lens Information (6 tags)
- **Make** (0x010F) - Camera manufacturer - freq 1.000
- **Model** (0x0110) - Camera model - freq 1.000
- **Software** (0x0131) - Processing software - freq 0.600
- **LensInfo** (0xA432) - Lens specification - freq 0.086
- **LensMake** (0xA433) - Lens manufacturer - freq 0.022
- **LensModel** (0xA434) - Lens model name - freq 0.100

### GPS Information (5 tags)
- **GPSLatitude** (0x0002) - Latitude - freq 0.079
- **GPSLongitude** (0x0004) - Longitude - freq 0.079
- **GPSAltitude** (0x0006) - Altitude - freq 0.061
- **GPSLatitudeRef** (0x0001) - North or South - freq 0.039
- **GPSLongitudeRef** (0x0003) - East or West - freq 0.040

### Other Required Tags (4 tags)
- **Copyright** (0x8298) - Copyright string - freq 0.200
- **Artist** (0x013B) - Person who created the image
- **UserComment** (0x9286) - User comments
- **ExifVersion** (0x9000) - EXIF version

## Work Completed

- ✅ Basic IFD parsing infrastructure
- ✅ Value extraction for common types
- ✅ Tag namespace assignment
- ✅ Some tags already extracting (Make, Model, DateTime)

## Remaining Tasks

### High Priority - Core Value Extraction

1. **APEX Value Conversions**
   - ApertureValue → FNumber conversion (APEX: FNumber = 2^(ApertureValue/2))
   - ShutterSpeedValue → ExposureTime conversion (APEX: ExposureTime = 2^(-ShutterSpeedValue))
   - MaxApertureValue conversion
   - Implement APEX formulas per EXIF spec section 4.6.5

2. **Rational Value Handling**
   - Ensure RATIONAL/SRATIONAL types properly extracted
   - Handle edge cases (0 denominator, overflow)
   - FocalLength, ExposureTime, FNumber all use RATIONAL -- 

3. **SubSecTime Processing**
   - Extract subsecond precision (ASCII string)
   - Combine with main timestamps for high precision
   - Handle SubSecTimeDigitized separately

4. **GPS Coordinate Processing**
   - Convert GPS rational degrees/minutes/seconds to decimal
   - Handle GPSLatitudeRef/GPSLongitudeRef (N/S, E/W)
   - Process GPSAltitude with GPSAltitudeRef (above/below sea level)

### Medium Priority - Missing Tag Implementation

1. **Lens Information Tags**
   - LensInfo (0xA432) - Min/max focal length and aperture
   - LensMake (0xA433) - ASCII string
   - LensModel (0xA434) - ASCII string

2. **Additional Timestamps**
   - Proper handling of CreateDate vs DateTimeOriginal
   - ModifyDate extraction and formatting

3. **Standard Metadata**
   - Artist (0x013B) - Creator name
   - UserComment (0x9286) - May have character code prefix
   - ExifVersion (0x9000) - Usually "0230"

### Low Priority - String Encoding & Validation

1. **Copyright/Description Strings**
   - Handle various character encodings
   - Strip null terminators
   - Validate UTF-8

2. **GPS Processing Method**
   - GPSProcessingMethod tag support
   - Handle various encoding markers

3. **ISO Value Normalization**
   - Handle both ISO (0x8827) and ISOSpeed (0x8833)
   - Some cameras use ISOSpeedRatings array

## Prerequisites

- Verify RATIONAL type extraction working correctly
- Ensure all standard EXIF IFDs are being processed

## Testing Strategy

- Test with images from multiple manufacturers
- Verify APEX conversions match ExifTool output
- Check edge cases (missing values, invalid data)

## Success Criteria

- All 36 EXIF required tags extracting correctly
- Values match ExifTool output format
- Proper error handling for missing/invalid data
- PrintConv producing human-readable values where applicable
- GPS coordinates properly converted to decimal degrees
- Timestamps include subsecond precision where available

## Gotchas & Tribal Knowledge

### APEX Values
- APEX values use logarithmic scale (base 2)
- FNumber = 2^(ApertureValue/2)
- ExposureTime = 2^(-ShutterSpeedValue)
- Some cameras write both FNumber AND ApertureValue (prefer direct values)

### GPS Coordinates
- Stored as 3 RATIONAL values: degrees, minutes, seconds
- Decimal conversion: degrees + minutes/60 + seconds/3600
- Apply negative for South latitude or West longitude
- GPSAltitude can be negative (below sea level) based on GPSAltitudeRef

### Timestamp Handling
- DateTime format: "YYYY:MM:DD HH:MM:SS"
- SubSecTime is ASCII string, not numeric (e.g., "123" = 0.123 seconds)
- CreateDate (0x9004) is actually DateTimeDigitized in EXIF spec
- Some cameras don't set all timestamp fields

### Tag Location Priority
- Tags can appear in multiple IFDs (IFD0, ExifIFD, GPS IFD)
- Use first found, but GPS tags only valid in GPS IFD
- ImageWidth/Height in IFD0 may differ from ExifImageWidth/Height

### Character Encodings
- UserComment may start with encoding marker (e.g., "ASCII\0\0\0")
- Most strings are ASCII null-terminated
- Some cameras use UTF-8 without proper markers