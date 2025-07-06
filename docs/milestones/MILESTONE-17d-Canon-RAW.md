# Milestone 17d: Canon RAW Support

**Duration**: 2-3 weeks  
**Goal**: Implement Canon RAW formats (CR2 required, CRW/CR3 optional)

## Overview

Canon RAW support represents a major complexity milestone:

- 10,648 lines in ExifTool Canon.pm
- 169 ProcessBinaryData entries
- Multiple format variants (CR2, CRW, CR3)
- Complex offset schemes (4/6/16/28 byte variants)
- Extensive lens database

This milestone focuses on CR2 (TIFF-based) as the primary target, with CRW (legacy) and CR3 (MOV-based) as stretch goals.

## Background

**Canon Format Evolution**:

- **CRW**: Legacy format, custom structure (pre-2004)
- **CR2**: Current TIFF-based format (2004-2018)
- **CR3**: New MOV/MP4-based format (2018+)

**Complexity Sources**:

- Multiple camera generations with different data layouts
- Proprietary lens coding system
- Custom color processing information
- AF micro-adjustment data
- Dual Pixel RAW information

## Implementation Strategy

### Phase 1: Canon Infrastructure (Week 1)

**Format Detection and Routing**:

```rust
// src/raw/formats/canon/mod.rs
pub mod cr2;
pub mod crw;  // Optional
pub mod cr3;  // Optional
pub mod lens_database;
pub mod offset_schemes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanonFormat {
    CR2,    // Primary target
    CRW,    // Legacy (optional)
    CR3,    // Modern (optional)
}

pub struct CanonRawHandler {
    // 169 ProcessBinaryData sections!
    processors: HashMap<CanonDataType, ProcessBinaryData>,
    lens_db: CanonLensDatabase,
    offset_manager: CanonOffsetManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonDataType {
    CameraSettings,
    FocalLength,
    ShotInfo,
    Panorama,
    ImageType,
    FirmwareVersion,
    FileNumber,
    OwnerName,
    CameraInfo,       // Variable size!
    ModelID,
    AFInfo,           // Multiple versions
    ThumbnailImageValidArea,
    SerialNumberFormat,
    SuperMacro,
    DateStampMode,
    MyColors,
    FirmwareRevision,
    Categories,
    FaceDetect1,
    FaceDetect2,
    AFInfo2,          // Different structure
    ContrastInfo,
    ImageUniqueID,
    WBInfo,
    FaceDetect3,
    TimeInfo,
    BatteryType,
    AFInfo3,          // Yet another version
    // ... and 140+ more!
}
```

**Canon Offset Management**:

```rust
// src/raw/formats/canon/offset_schemes.rs
/// Canon uses different offset schemes based on camera generation
/// ExifTool: Canon.pm lines 8439-8498
pub struct CanonOffsetManager {
    schemes: HashMap<u32, OffsetScheme>,
}

#[derive(Debug, Clone)]
pub struct OffsetScheme {
    pub base_offset: OffsetBase,
    pub pointer_size: PointerSize,
    pub endianness: Endianness,
}

#[derive(Debug, Clone, Copy)]
pub enum PointerSize {
    Bytes4,   // Older cameras
    Bytes6,   // Some PowerShot models
    Bytes16,  // Newer DSLRs
    Bytes28,  // Latest models
}

impl CanonOffsetManager {
    pub fn detect_offset_scheme(&self, reader: &ExifReader) -> Result<OffsetScheme> {
        // Canon offset detection logic
        // ExifTool: ProcessCanon() offset detection
        let model_id = reader.get_tag_value("Canon:ModelID")?;

        match model_id {
            0x80000001..=0x80000100 => Ok(OffsetScheme {
                base_offset: OffsetBase::IfdStart,
                pointer_size: PointerSize::Bytes4,
                endianness: Endianness::Big,
            }),
            0x80000200..=0x80000300 => Ok(OffsetScheme {
                base_offset: OffsetBase::ValueOffset,
                pointer_size: PointerSize::Bytes6,
                endianness: Endianness::Little,
            }),
            // ... more model ranges
            _ => Ok(OffsetScheme::default()),
        }
    }
}
```

### Phase 2: CR2 Implementation (Week 1-2)

**CR2 Processor**:

```rust
// src/raw/formats/canon/cr2.rs
pub struct CR2Processor {
    handler: CanonRawHandler,
}

impl CR2Processor {
    pub fn process(&mut self, reader: &mut ExifReader) -> Result<()> {
        // CR2 is TIFF-based with Canon maker notes
        // Process standard TIFF structure first
        self.process_tiff_structure(reader)?;

        // Find Canon maker notes
        let maker_note_offset = reader.get_maker_note_offset()?;
        let maker_data = reader.get_maker_note_data()?;

        // Detect offset scheme for this camera
        let scheme = self.handler.offset_manager.detect_offset_scheme(reader)?;

        // Process Canon-specific IFDs
        self.process_canon_ifds(reader, &maker_data, maker_note_offset, &scheme)?;

        // Handle Canon color data
        self.process_canon_color_data(reader)?;

        Ok(())
    }

    fn process_canon_ifds(&mut self, reader: &mut ExifReader, data: &[u8], offset: u64, scheme: &OffsetScheme) -> Result<()> {
        // Canon stores data in custom IFD structure
        // Each tag ID maps to a specific data processor

        let entries = parse_canon_ifd(data, scheme)?;

        for entry in entries {
            match entry.tag {
                0x0001 => self.process_camera_settings(reader, entry)?,
                0x0002 => self.process_focal_length(reader, entry)?,
                0x0003 => self.process_shot_info(reader, entry)?,
                0x0004 => self.process_panorama(reader, entry)?,
                0x0026 => self.process_af_info2(reader, entry)?,
                // ... handle all 169 types
                _ => {} // Unknown tag
            }
        }

        Ok(())
    }
}
```

**Canon Lens Database**:

```rust
// src/raw/formats/canon/lens_database.rs
/// Canon encodes lens information in a complex way
/// ExifTool: Canon.pm %canonLensTypes (2000+ entries!)
pub struct CanonLensDatabase {
    lens_types: HashMap<u16, LensInfo>,
    lens_decoder: CanonLensDecoder,
}

#[derive(Debug, Clone)]
pub struct LensInfo {
    pub name: String,
    pub focal_range: Option<(f32, f32)>,
    pub max_aperture: Option<f32>,
    pub is_teleconverter: bool,
}

impl CanonLensDatabase {
    pub fn decode_lens(&self, lens_type: u16, focal_length: f32, aperture: f32) -> Option<&LensInfo> {
        // Canon lens matching is complex
        // Some lens IDs are reused for different lenses!
        // Must match against focal length and aperture

        if let Some(candidates) = self.get_lens_candidates(lens_type) {
            // Filter by focal length and aperture
            for lens in candidates {
                if self.matches_lens_params(lens, focal_length, aperture) {
                    return Some(lens);
                }
            }
        }

        None
    }
}
```

### Phase 3: Complex Data Types (Week 2)

**AF Info Processing** (3 versions!):

```rust
// Canon has evolved AF info through multiple versions
fn create_af_info_processor() -> ProcessBinaryData {
    // ExifTool: %Image::ExifTool::Canon::AFInfo
    ProcessBinaryData::new(vec![
        BinaryDataEntry::new(0x00, 2, "NumAFPoints", None),
        BinaryDataEntry::new(0x02, 2, "ValidAFPoints", None),
        BinaryDataEntry::new(0x04, 2, "CanonImageWidth", None),
        BinaryDataEntry::new(0x06, 2, "CanonImageHeight", None),
        BinaryDataEntry::new(0x08, 2, "AFImageWidth", None),
        BinaryDataEntry::new(0x0a, 2, "AFImageHeight", None),
        BinaryDataEntry::new(0x0c, 2, "AFAreaWidths", None),
        BinaryDataEntry::new(0x0e, 2, "AFAreaHeights", None),
        BinaryDataEntry::new(0x10, 4, "AFAreaXPositions", None),
        BinaryDataEntry::new(0x14, 4, "AFAreaYPositions", None),
        // Variable length AF point data follows
    ])
}

fn process_af_info(&mut self, reader: &mut ExifReader, entry: CanonIfdEntry) -> Result<()> {
    // AF info has variable length based on number of AF points
    let data = entry.get_data(reader)?;

    // First read fixed portion
    let num_points = u16::from_be_bytes([data[0], data[1]]);

    // Then read variable AF point data
    let point_size = 4; // bytes per AF point
    let expected_size = 0x18 + (num_points as usize * point_size);

    if data.len() >= expected_size {
        // Process AF points
        for i in 0..num_points {
            let offset = 0x18 + (i as usize * point_size);
            let point_data = &data[offset..offset + point_size];
            self.process_af_point(reader, i, point_data)?;
        }
    }

    Ok(())
}
```

**Canon Color Data**:

```rust
// Canon stores extensive color processing information
fn process_canon_color_data(&mut self, reader: &mut ExifReader) -> Result<()> {
    // Color data location varies by model
    // ExifTool: Canon.pm ProcessCanonColorData()

    if let Some(color_data_offset) = self.find_color_data_offset(reader)? {
        let color_data = reader.read_at_offset(color_data_offset, 0x4000)?; // Up to 16KB!

        // Color data format depends on camera generation
        let model_id = reader.get_tag_value("Canon:ModelID")?;

        match self.detect_color_data_version(model_id) {
            ColorDataVersion::V1 => self.process_color_data_v1(&color_data)?,
            ColorDataVersion::V2 => self.process_color_data_v2(&color_data)?,
            ColorDataVersion::V3 => self.process_color_data_v3(&color_data)?,
            // ... up to V8 currently
        }
    }

    Ok(())
}
```

### Phase 4: Testing and Optional Formats (Week 2-3)

**Comprehensive Testing**:

```rust
// tests/raw/canon_tests.rs
#[test]
fn test_canon_cr2_all_data_types() {
    let test_files = [
        "test-images/canon/5d_mark_iv.cr2",
        "test-images/canon/80d.cr2",
        "test-images/canon/g7x.cr2",
    ];

    for file in &test_files {
        let result = process_raw_file(file)?;

        // Verify core Canon data
        assert_tag_exists(&result, "Canon:ModelID");
        assert_tag_exists(&result, "Canon:LensType");
        assert_tag_exists(&result, "Canon:FirmwareVersion");
        assert_tag_exists(&result, "Canon:OwnerName");

        // AF Info (version depends on model)
        assert!(
            tag_exists(&result, "Canon:NumAFPoints") ||
            tag_exists(&result, "Canon:AFPointsInFocus") ||
            tag_exists(&result, "Canon:AFPointsSelected")
        );

        // Compare with ExifTool
        compare_with_exiftool(file, &[
            "Make", "Model", "Canon:LensModel", "Canon:SerialNumber",
            "ISO", "FNumber", "ExposureTime", "Canon:InternalSerialNumber"
        ]);
    }
}

#[test]
fn test_canon_lens_identification() {
    // Canon lens ID requires complex matching
    let test_cases = [
        (61182, 24.0, 105.0, "Canon EF 24-105mm f/4L IS USM"),
        (61182, 24.0, 70.0, "Canon EF 24-70mm f/4L IS USM"), // Same ID!
    ];

    let lens_db = CanonLensDatabase::new();

    for (lens_id, focal, aperture, expected) in &test_cases {
        let result = lens_db.decode_lens(*lens_id, *focal, *aperture);
        assert_eq!(result.map(|l| &l.name), Some(expected));
    }
}
```

## Success Criteria

### Core Requirements (CR2)

- [ ] **CR2 Format Support**: Complete TIFF-based CR2 processing
- [ ] **169 Data Types**: All Canon-specific ProcessBinaryData sections
- [ ] **Offset Schemes**: Handle 4/6/16/28 byte pointer variants
- [ ] **Lens Database**: Accurate lens identification
- [ ] **AF Information**: All three AF info versions
- [ ] **Color Data**: Canon color processing information

### Optional Goals (CRW/CR3)

- [ ] **CRW Support**: Legacy format (if time permits)
- [ ] **CR3 Support**: MOV-based format (if time permits)

### Validation Tests

- [ ] Process CR2 files from multiple camera generations
- [ ] Verify lens identification accuracy
- [ ] Extract all Canon-specific fields
- [ ] Compare against `exiftool -j`
- [ ] Handle edge cases (custom firmware, adapted lenses)

## Implementation Boundaries

### Goals (Milestone 17d)

- Complete CR2 metadata extraction
- Canon maker note processing
- Lens database implementation
- AF and color data extraction

### Non-Goals

- Preview extraction (Milestone 19)
- CR2 file writing
- RAW image decoding
- Dual Pixel RAW processing

## Dependencies and Prerequisites

- Completed Milestones 17a-c
- Complex offset management capability
- Variable-length data handling
- Canon CR2 test images

## Technical Notes

### CR2 Structure

```
CR2 File:
├── TIFF Header
├── IFD0 (Main Image)
├── IFD1 (Thumbnail)
├── EXIF IFD
│   └── Maker Note IFD (Canon-specific)
│       ├── Camera Settings (0x0001)
│       ├── Focal Length (0x0002)
│       ├── Shot Info (0x0003)
│       ├── AF Info (0x0026)
│       └── ... (165+ more)
├── Canon Color Data
└── RAW Image Data
```

### Offset Scheme Detection

Canon cameras use different pointer sizes:

- 4 bytes: Original Digital Rebels
- 6 bytes: Some PowerShots
- 16 bytes: Modern DSLRs
- 28 bytes: Latest mirrorless

### Trust ExifTool

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

- Implement all 169 ProcessBinaryData sections
- Use exact lens matching algorithm
- Preserve offset scheme detection logic
- Don't simplify AF info variations

## Risk Mitigation

### Complexity Management

- **Risk**: 169 data types is overwhelming
- **Mitigation**: Implement incrementally, test continuously
- **Priority**: Focus on most common types first

### Lens Database Size

- **Risk**: 2000+ lens entries use significant memory
- **Mitigation**: Generate from ExifTool source
- **Optimization**: Lazy loading or lookup tables

### Format Variations

- **Risk**: CR2 format varies by camera
- **Mitigation**: Test multiple camera models
- **Validation**: Use ExifTool debug output

## Next Steps

After successful completion:

1. Milestone 17e: Sony ARW (advanced offset management)
2. Milestone 17f: Nikon integration
3. Consider CR3 support in future milestone

## Summary

Canon CR2 support represents the most complex manufacturer implementation so far, with 169 distinct data types and multiple offset schemes. Success here proves our architecture can handle the most demanding manufacturer formats.
