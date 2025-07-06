# Milestone 17c: Olympus RAW Support

**Duration**: 2 weeks  
**Goal**: Implement Olympus ORF format with its complex multi-section structure

## Overview

This milestone adds support for Olympus ORF (Olympus Raw Format), which represents a significant step up in complexity:

- 4,235 lines in ExifTool (vs 956 for Panasonic)
- 15 distinct ProcessBinaryData sections
- Complex IFD structure with multiple data blocks
- Extensive camera-specific settings

## Background

**ORF Characteristics**:

- TIFF-based container with Olympus-specific IFDs
- Multiple data sections for different feature sets
- Equipment data stored separately from image data
- Focus information in dedicated blocks
- Raw development parameters in maker notes

**ProcessBinaryData Sections** (from Olympus.pm):

1. CameraSettings
2. FocusInfo
3. Equipment
4. CameraSettings2
5. RawDevelopment
6. RawDevelopment2
7. ImageProcessing
8. FocusDistance
9. RawInfo
10. CameraSettings3
11. SpecialMode
12. WhiteBalance
13. BlackLevel
14. ColorMatrix
15. WB_RBLevels

## Implementation Strategy

### Phase 1: Multi-Section Architecture (Week 1)

**Enhanced Handler Structure**:

```rust
// src/raw/formats/olympus.rs
use std::collections::HashMap;
use crate::implementations::binary_data::{ProcessBinaryData, BinaryDataEntry};

pub struct OlympusRawHandler {
    // 15 different ProcessBinaryData sections!
    processors: HashMap<OlympusDataType, ProcessBinaryData>,
    // Olympus-specific IFD navigation
    ifd_navigator: OlympusIfdNavigator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OlympusDataType {
    CameraSettings,
    FocusInfo,
    Equipment,
    CameraSettings2,
    RawDevelopment,
    RawDevelopment2,
    ImageProcessing,
    FocusDistance,
    RawInfo,
    CameraSettings3,
    SpecialMode,
    WhiteBalance,
    BlackLevel,
    ColorMatrix,
    WBRBLevels,
}

impl OlympusRawHandler {
    pub fn new() -> Self {
        let mut processors = HashMap::new();

        // Initialize all 15 processors
        // ExifTool: lib/Image/ExifTool/Olympus.pm
        processors.insert(
            OlympusDataType::CameraSettings,
            Self::create_camera_settings_processor()
        );
        processors.insert(
            OlympusDataType::FocusInfo,
            Self::create_focus_info_processor()
        );
        processors.insert(
            OlympusDataType::Equipment,
            Self::create_equipment_processor()
        );
        // ... continue for all 15 types

        Self {
            processors,
            ifd_navigator: OlympusIfdNavigator::new(),
        }
    }

    fn create_camera_settings_processor() -> ProcessBinaryData {
        // ExifTool: %Image::ExifTool::Olympus::CameraSettings
        ProcessBinaryData::new(vec![
            BinaryDataEntry::new(0x100, 2, "ExposureMode", None),
            BinaryDataEntry::new(0x102, 2, "MeteringMode", None),
            BinaryDataEntry::new(0x104, 2, "FocusMode", None),
            // ... many more entries
        ])
    }

    fn create_focus_info_processor() -> ProcessBinaryData {
        // ExifTool: %Image::ExifTool::Olympus::FocusInfo
        ProcessBinaryData::new(vec![
            BinaryDataEntry::new(0x000, 2, "FocusDistance", None),
            BinaryDataEntry::new(0x002, 2, "MacroMode", None),
            BinaryDataEntry::new(0x004, 2, "FocusRange", None),
            BinaryDataEntry::new(0x006, 2, "FocusStepCount", None),
            // ... more focus-related entries
        ])
    }
}
```

**Olympus IFD Navigation**:

```rust
// src/raw/formats/olympus/navigator.rs
pub struct OlympusIfdNavigator {
    // Olympus uses specific tag IDs to point to data sections
    section_tags: HashMap<u16, OlympusDataType>,
}

impl OlympusIfdNavigator {
    pub fn new() -> Self {
        let mut section_tags = HashMap::new();

        // Tag IDs that contain pointers to data sections
        // From Olympus.pm tag definitions
        section_tags.insert(0x2010, OlympusDataType::Equipment);
        section_tags.insert(0x2020, OlympusDataType::CameraSettings);
        section_tags.insert(0x2030, OlympusDataType::RawDevelopment);
        section_tags.insert(0x2040, OlympusDataType::ImageProcessing);
        section_tags.insert(0x2050, OlympusDataType::FocusInfo);
        // ... more mappings

        Self { section_tags }
    }

    pub fn find_data_sections(&self, reader: &ExifReader) -> Result<Vec<DataSection>> {
        let mut sections = Vec::new();

        // Navigate Olympus maker note IFD
        let maker_note_entries = reader.get_maker_note_entries()?;

        for entry in maker_note_entries {
            if let Some(data_type) = self.section_tags.get(&entry.tag) {
                sections.push(DataSection {
                    data_type: *data_type,
                    offset: entry.value_offset as u64,
                    size: entry.count as usize,
                });
            }
        }

        Ok(sections)
    }
}
```

### Phase 2: Complex Data Processing (Week 1-2)

**Main Handler Implementation**:

```rust
impl RawFormatHandler for OlympusRawHandler {
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()> {
        // Find all data sections in the maker notes
        let sections = self.ifd_navigator.find_data_sections(reader)?;

        // Process each section with appropriate processor
        for section in sections {
            if let Some(processor) = self.processors.get(&section.data_type) {
                let section_data = reader.read_at_offset(section.offset, section.size)?;

                // Use specific group name for each section type
                let group_name = format!("Olympus{:?}", section.data_type);
                processor.process(reader, &section_data, section.offset, &group_name)?;
            }
        }

        // Handle special Olympus quirks
        self.apply_olympus_quirks(reader)?;

        Ok(())
    }

    fn apply_olympus_quirks(&self, reader: &mut ExifReader) -> Result<()> {
        // Olympus-specific adjustments
        // ExifTool: various special cases in Olympus.pm

        // Example: Olympus stores some values with specific scaling
        if let Some(mut iso) = reader.get_tag_value_mut("ISO") {
            // Olympus ISO values need adjustment
            // From Olympus.pm ISO handling
            iso.apply_scaling(100.0);
        }

        Ok(())
    }
}
```

**Equipment Data Handling**:

```rust
// Olympus stores lens and body information separately
fn create_equipment_processor() -> ProcessBinaryData {
    // ExifTool: %Image::ExifTool::Olympus::Equipment
    ProcessBinaryData::new(vec![
        BinaryDataEntry::new(0x000, 2, "EquipmentVersion", None),
        BinaryDataEntry::new(0x100, 6, "CameraType", None),
        BinaryDataEntry::new(0x102, 6, "SerialNumber", None),
        BinaryDataEntry::new(0x104, 32, "InternalSerialNumber", None),
        BinaryDataEntry::new(0x201, 6, "LensType", None),
        BinaryDataEntry::new(0x202, 32, "LensSerialNumber", None),
        BinaryDataEntry::new(0x204, 4, "LensFirmwareVersion", None),
        BinaryDataEntry::new(0x208, 2, "MaxApertureValue", None),
        BinaryDataEntry::new(0x20a, 2, "MinFocalLength", None),
        BinaryDataEntry::new(0x20c, 2, "MaxFocalLength", None),
        // ... more equipment entries
    ])
}
```

### Phase 3: Tag Resolution and PrintConv (Week 2)

**Olympus-Specific PrintConv**:

```rust
// src/implementations/olympus/print_conv.rs
pub fn register_olympus_print_conv() {
    let registry = get_global_registry();

    // Olympus has many specific value conversions
    registry.register_print_conv("olympus_exposure_mode", |val| {
        // ExifTool: Olympus.pm ExposureMode PrintConv
        match val.as_u16() {
            Some(1) => TagValue::String("Manual".to_string()),
            Some(2) => TagValue::String("Program".to_string()),
            Some(3) => TagValue::String("Aperture Priority".to_string()),
            Some(4) => TagValue::String("Shutter Priority".to_string()),
            Some(5) => TagValue::String("Program Creative".to_string()),
            _ => val.clone(),
        }
    });

    registry.register_print_conv("olympus_focus_mode", |val| {
        // Complex focus mode with bit fields
        if let Some(focus_val) = val.as_u16() {
            let mut modes = Vec::new();

            if focus_val & 0x01 != 0 { modes.push("Single AF"); }
            if focus_val & 0x02 != 0 { modes.push("Sequential AF"); }
            if focus_val & 0x04 != 0 { modes.push("Continuous AF"); }
            if focus_val & 0x10 != 0 { modes.push("Multi AF"); }

            TagValue::String(modes.join(", "))
        } else {
            val.clone()
        }
    });
}
```

### Phase 4: Testing and Validation (Week 2)

**Comprehensive ORF Testing**:

```rust
// tests/raw/olympus_tests.rs
#[test]
fn test_olympus_orf_all_sections() {
    let test_file = "test-images/olympus/e-m1.orf";

    // Verify all 15 data sections are found and processed
    let result = process_raw_file(test_file)?;

    // Check camera settings
    assert_tag_exists(&result, "ExposureMode");
    assert_tag_exists(&result, "MeteringMode");

    // Check focus info
    assert_tag_exists(&result, "FocusDistance");
    assert_tag_exists(&result, "FocusStepCount");

    // Check equipment
    assert_tag_exists(&result, "LensType");
    assert_tag_exists(&result, "LensSerialNumber");

    // Compare with ExifTool
    compare_with_exiftool(test_file, &[
        "Make", "Model", "ISO", "FNumber", "ExposureTime",
        "LensModel", "FocusDistance", "ColorSpace"
    ]);
}

#[test]
fn test_olympus_special_modes() {
    // Olympus has various special shooting modes
    let test_files = [
        ("test-images/olympus/art_filter.orf", "Art Filter Mode"),
        ("test-images/olympus/scene_mode.orf", "Scene Mode"),
        ("test-images/olympus/panorama.orf", "Panorama Mode"),
    ];

    for (file, expected_mode) in &test_files {
        let result = process_raw_file(file)?;
        assert_tag_value(&result, "SpecialMode", expected_mode);
    }
}
```

## Success Criteria

### Core Requirements

- [ ] **All 15 ProcessBinaryData sections**: Complete implementation
- [ ] **Section Navigation**: Correctly find and process all data blocks
- [ ] **Olympus Tags**: Camera settings, focus info, equipment data
- [ ] **PrintConv Functions**: Olympus-specific value conversions
- [ ] **CLI Integration**: `exif-oxide olympus.orf` works correctly

### Validation Tests

- [ ] Process multiple ORF samples (E-M1, E-M5, PEN series)
- [ ] Extract all standard EXIF fields
- [ ] Extract Olympus-specific fields (Art Filter, Scene Mode, etc.)
- [ ] Verify against `exiftool -j olympus.orf`
- [ ] Handle older and newer ORF format variations

## Implementation Boundaries

### Goals (Milestone 17c)

- Complete Olympus ORF metadata extraction
- Handle all 15 data sections properly
- Support equipment and lens information
- Implement Olympus-specific conversions

### Non-Goals

- Preview extraction (Milestone 19)
- ORF file writing
- RAW image decoding
- Art Filter rendering

## Dependencies and Prerequisites

- Completed Milestones 17a and 17b
- Multi-section ProcessBinaryData support
- Complex IFD navigation capability
- Olympus ORF test images

## Technical Notes

### ORF Structure Overview

```
ORF File:
├── TIFF Header
├── Main IFD
│   ├── EXIF IFD
│   └── Maker Note IFD
│       ├── Equipment (0x2010)
│       ├── Camera Settings (0x2020)
│       ├── Raw Development (0x2030)
│       ├── Image Processing (0x2040)
│       ├── Focus Info (0x2050)
│       └── ... (10 more sections)
└── Image Data
```

### Data Section Complexity

Each of the 15 sections has different:

- Entry counts (from 10 to 200+ entries)
- Data types (bytes, shorts, longs, ASCII)
- Offset calculations
- Value interpretations

### Trust ExifTool

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

- Implement all 15 sections exactly as in Olympus.pm
- Preserve quirky value calculations
- Don't skip "redundant" seeming sections
- Match ExifTool's tag naming exactly

## Risk Mitigation

### Section Discovery

- **Risk**: Missing data sections due to format variations
- **Mitigation**: Implement robust section discovery
- **Validation**: Check against multiple camera models

### Processing Order

- **Risk**: Some sections depend on others
- **Mitigation**: Process in ExifTool's order
- **Testing**: Verify interdependent values

### Memory Usage

- **Risk**: 15 processors could use significant memory
- **Mitigation**: Lazy initialization of processors
- **Alternative**: Load processors on-demand

## Next Steps

After successful completion:

1. Milestone 17d: Canon CR2 (even more complex)
2. Milestone 17e: Sony ARW (requires advanced offsets)
3. Build on multi-section pattern for other manufacturers

## Summary

Olympus ORF represents a significant complexity increase with its 15 distinct data sections. Successfully implementing this format validates our architecture can handle complex, multi-section manufacturer formats while maintaining clean code organization.
