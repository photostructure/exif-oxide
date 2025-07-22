# Technical Project Plan: Sony Required Tags Implementation

## Project Overview

- **Goal**: Implement support for all Sony-specific required tags from PhotoStructure's tag-metadata.json
- **Problem**: Need comprehensive support for Sony MakerNotes tags marked as required, including encrypted data

## Background & Context

- PhotoStructure requires proper Sony tag extraction for photo management
- Sony uses complex encrypted MakerNotes requiring special handling
- Many standard tags have Sony-specific variants with additional precision/data
- Current implementation extracts ~40 tags but missing critical required ones

## Technical Foundation

- **Key files**:
  - `src/implementations/sony/mod.rs` - Main Sony processor
  - `src/implementations/sony/encryption.rs` - Decryption logic
  - `src/generated/Sony_pm/` - Generated lookup tables
  - `third-party/exiftool/lib/Image/ExifTool/Sony.pm` - ExifTool source
- **Documentation**:
  - `docs/todo/MILESTONE-17e-Sony-RAW.md` - Sony implementation status
  - `third-party/exiftool/doc/modules/Sony.md` - Sony module overview

## Work Completed

- ✅ Basic Sony MakerNotes detection
- ✅ Encryption/decryption infrastructure (framework exists, needs implementation)
- ✅ Some binary data sections parsing (7 of 139 ProcessBinaryData sections)
  - ✅ SonyAFInfoProcessor - Autofocus data (tag 0x940e)
  - ✅ SonyCameraInfoProcessor - Camera info (tag 0x0010)
  - ✅ SonyCameraSettingsProcessor - Basic settings (tag 0x0114)
  - ✅ SonyShotInfoProcessor - Shot metadata (tag 0x3000)
  - ⚠️ SonyTag9050Processor - Stub only, needs decryption
  - ⚠️ SonyTag2010Processor - Stub only, needs decryption
- ✅ Generated lookup tables for various Sony tables (322 PrintConv entries)
- ✅ Sony tag naming system working (shows "Sony:AFType" not "EXIF:Tag_927C")
- ✅ ProcessBinaryData integration complete in RAW handler
- ⚠️ Standard EXIF tags extracting (ISO, ExposureTime, FNumber) but not Sony-specific variants

## Remaining Tasks

### High Priority - Sony-Specific Required Tags (3 tags)

1. **SonyExposureTime** (freq 0.010)
   - Located in encrypted MakerNotes, specifically in:
     - **Tag9050a** (Sony.pm lines 7568-7574) - offset 0x003a, int16u format
     - **Tag9050b** (Sony.pm lines 7850-7856) - offset 0x0046, int16u format  
     - **Tag9050c** (Sony.pm lines 8163-8169) - offset 0x0046, int16u format
     - **Tag9050d** (Sony.pm lines 8245-8251) - offset 0x001a, int16u format
   - Format: `ValueConv => '$val ? 2 ** (16 - $val/256) : 0'`
   - May have higher precision than standard ExposureTime
   - Requires Tag9050 processor implementation with decryption

2. **SonyFNumber** (freq 0.011)
   - Sony's proprietary F-number format, found in:
     - **Tag9050a** (Sony.pm lines 7576-7582) - offset 0x003c, int16u format
     - **Tag9050b** (Sony.pm lines 7858-7864) - offset 0x0048, int16u format
     - **Tag9050c** (Sony.pm lines 8171-8177) - offset 0x0048, int16u format
     - **Tag9050d** (Sony.pm lines 8253-8259) - offset 0x001c, int16u format
     - **Tag9416** (Sony.pm lines 8919-8925) - offset 0x0010, int16u format (for DSC models excluded)
   - Format: `ValueConv => '2 ** (($val/256 - 16) / 2)'`
   - May include lens-specific corrections
   - Requires Tag9050/Tag9416 processor implementation

3. **SonyISO** (freq 0.022)
   - Located in various positions depending on model:
     - **Tag2010b** (Sony.pm lines 6466-6471) - offset 0x1218, int16u format
     - **Tag2010c** (Sony.pm lines 6536-6541) - offset 0x11f4, int16u format
     - **Tag2010d** (Sony.pm lines 6600-6605) - offset 0x1270, int16u format
     - **Tag2010e** (Sony.pm lines 6669-6674) - offset 0x1254, int16u format (specific models)
     - **Tag2010e** (Sony.pm lines 6678-6683) - offset 0x1258, int16u format (DSC-RX1/RX1R)
     - **Tag2010f** (Sony.pm lines 6715-6720) - offset 0x1280, int16u format
     - **Tag2010g** (Sony.pm lines 6869-6874) - offset 0x113c, int16u format
     - **Tag2010h** (Sony.pm lines 6951-6956) - offset 0x0344, int16u format
     - **Tag2010i** (Sony.pm lines 7092-7097) - offset 0x0346, int16u format
     - **Tag2010j** (Sony.pm lines 7241-7246) - offset 0x0320, int16u format
     - **Tag9416** (Sony.pm lines 8887-8892) - offset 0x0004, int16u format
   - Format: `ValueConv => '100 * 2**(16 - $val/256)'`
   - Requires Tag2010/Tag9416 processor implementation with model detection

### Medium Priority - Standard MakerNotes Tags Sony Populates

**High Frequency Tags (>50%):**
- **ExposureTime** (freq 0.990) - Standard location + Sony variants
- **FNumber** (freq 0.970) - Multiple locations in Sony data
- **FocalLength** (freq 0.950) - Including lens corrections
- **ISO** (freq 0.890) - Consolidate from multiple sources
- **ShutterSpeed** (freq 0.860) - Format from APEX values
- **Aperture** (freq 0.850) - Calculate from FNumber

**Lens Information:**
- **LensID** (freq 0.200) - Complex Sony lens identification
- **LensType** (freq 0.180) - From various binary sections
- **Lens** (freq 0.150) - Full lens description
- **LensModel** (freq 0.100) - E-mount and A-mount lenses
- **LensInfo** (freq 0.086) - Min/max specifications
- **LensSpec** (freq 0.039) - Formatted specification

**Other Required Tags:**
- **SerialNumber** (freq 0.130) - Camera body serial
- **InternalSerialNumber** (freq 0.150) - Internal ID
- **FileNumber** (freq 0.130) - Image counter
- **CameraID** (freq 0.068) - Model-specific ID
- **DateTimeUTC** (freq 0.007) - From Sony timestamps
- **Software** (freq 0.600) - Firmware version

### Low Priority - Location and Metadata

- **Categories** (freq 0.051) - If supported by model
- **Title** (freq 0.021) - User-defined title
- **City** (freq 0.010) - GPS location data
- **Country** (freq 0.010) - GPS location data

## Prerequisites

- Complete encryption/decryption for all Sony formats
  - Simple substitution cipher for 0x94xx tags (Decipher function)
  - LFSR-based encryption for SR2SubIFD (Decrypt function)
- Model detection for camera-specific processing
  - Already have model detection in place
  - Need to map models to specific Tag2010 variants (a-j)
- ~~Fix namespace assignment for Sony tags~~ ✅ COMPLETED

## Testing Strategy

- Test with multiple Sony camera models (A7III, A7RIV, RX100, etc.)
- Verify encryption/decryption working correctly
- Compare with ExifTool output using compare tool
- Check both JPEG and ARW (RAW) files

## Success Criteria

- All 3 Sony-specific required tags extracting
- Standard required tags populated correctly
- Encryption/decryption working for all models
- PrintConv producing human-readable values
- Namespace correctly set to "MakerNotes:"

## Implementation Details

### ProcessBinaryData Sections Needed

For the 3 Sony-specific required tags, we need these processors:

1. **Tag2010 Processor Enhancement**
   - Current stub needs full implementation with decryption
   - Multiple variants (a-j) based on camera model
   - Contains SonyISO at different offsets per variant
   - Model detection logic from Sony.pm lines 1055-1289

2. **Tag9050 Processor Enhancement**  
   - Current stub needs decryption support
   - Contains SonyExposureTime and SonyFNumber
   - 4 variants (a-d) with different offsets
   - Encrypted data requires ProcessEnciphered handling

3. **Tag9416 Processor** (new)
   - Contains alternate SonyISO, SonyFNumber locations
   - Used by newer models
   - Shares similar structure with Tag9050

### Decryption Implementation

From ExifTool Sony.pm (lines 11341-11379):

```perl
# Decipher (for 0x94xx tags)
# Simple substitution cipher based on offset
my $key = $start + $offset;
foreach (@vals) {
    $_ = ($_ - $key) & 0xff;
    $key = $_; 
}

# Decrypt (for SR2SubIFD)
# LFSR-based encryption with 128-bit key
# More complex, used for SR2 format
```

## Gotchas & Tribal Knowledge

### Sony Encryption
- **Multiple Algorithms**: Different models use different encryption
- **Key Generation**: Based on camera model, serial number, and other factors
- **Encrypted Sections**: Not all MakerNotes data is encrypted
- **Format Changes**: Encryption format changes between camera generations

### Tag Locations
- **Tag9400**: Common location for exposure data (encrypted)
- **Tag9404**: Alternative location in newer models
- **CameraSettings**: Unencrypted basic settings
- **Multiple Copies**: Same data may appear in multiple locations

### Lens Detection
- **E-mount vs A-mount**: Different ID schemes
- **Third-Party**: May not report correctly
- **Adapted Lenses**: Special handling needed
- **Sony Lens Database**: Much larger than Canon/Nikon

### Value Extraction
- **Byte Order**: Can vary within MakerNotes
- **Rational Values**: Often stored differently than standard EXIF
- **Model Dependencies**: Tag locations vary significantly by model
- **Firmware Versions**: Same model may have different layouts

### Special Processing
- **Focus Information**: Complex multi-point data
- **Color Information**: Model-specific formats
- **Video Metadata**: Different structure than stills
- **Panorama Data**: Special tags for sweep panorama