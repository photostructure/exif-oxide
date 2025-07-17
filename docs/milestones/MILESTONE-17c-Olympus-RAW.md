# Milestone 17c: Olympus RAW Support

**Duration**: 6-8 hours (Revised from 2 weeks)  
**Goal**: Implement Olympus ORF format leveraging existing RAW infrastructure and generated lookup tables

## Overview

This milestone adds support for Olympus ORF (Olympus Raw Format). The implementation is significantly **simplified** from the original estimate due to:

- **Existing RAW infrastructure**: TIFF-based processing from Panasonic handler
- **Generated lookup tables**: `olympuscameratypes.rs`, `olympuslenstypes.rs`, `filters.rs` already available
- **Dual-mode processing**: ExifTool's pattern supports both binary data AND IFD processing
- **TIFF foundation**: ORF files are TIFF-based, leveraging existing `TiffHeader` and IFD processing

**Key Insight**: The 4,235 lines in ExifTool include extensive lookup tables and multiple format variants. Our codegen infrastructure has already generated the lookup tables, dramatically reducing implementation complexity.

## Background

**ORF Characteristics**:

- **TIFF-based container** with Olympus-specific maker note IFDs
- **Dual processing modes**: Each section can be processed as binary data OR IFD (ExifTool pattern)
- **Multiple data sections** for different feature sets
- **Generated lookup tables**: Camera types, lens types, and filters already available

**Core Data Sections** (9 primary sections from ExifTool analysis):

1. **Equipment (0x2010)** - Camera hardware, lens data, serial numbers
2. **CameraSettings (0x2020)** - Core camera settings, exposure mode, white balance  
3. **RawDevelopment (0x2030)** - RAW processing parameters
4. **RawDev2 (0x2031)** - Additional RAW development parameters
5. **ImageProcessing (0x2040)** - Image processing settings, art filters
6. **FocusInfo (0x2050)** - Autofocus information, AF points
7. **RawInfo (0x3000)** - RAW file specific information
8. **MainInfo (0x4000)** - Main Olympus tag table (primary maker notes)
9. **UnknownInfo (0x5000)** - Unknown/experimental data section

**Additional FE Model Sections** (optional, for FE camera models):
- **0x2100-0x2900** - FE model-specific tags (can be implemented later)

**Available Generated Infrastructure**:
- `src/generated/Olympus_pm/olympuscameratypes.rs` - 303 camera model mappings
- `src/generated/Olympus_pm/olympuslenstypes.rs` - 138 lens definitions  
- `src/generated/Olympus_pm/filters.rs` - Art filter definitions

## Implementation Strategy

### Phase 1: Detection and Integration (30 minutes)

**Add Olympus to RAW Detection**:

1. **Update `src/raw/detector.rs`**:
   - Add `Olympus` variant to `RawFormat` enum
   - Add ORF detection logic to `detect_raw_format()`
   - Add `validate_olympus_orf_magic()` function

2. **Update `src/raw/processor.rs`**:
   - Register Olympus handler in `RawProcessor::new()`

3. **Update `src/raw/formats/mod.rs`**:
   - Add `pub mod olympus;` declaration

### Phase 2: Core Handler Implementation (2-3 hours)

**TIFF-based Handler Structure** (following Panasonic pattern):

```rust
// src/raw/formats/olympus.rs
use crate::exif::ExifReader;
use crate::raw::RawFormatHandler;
use crate::tiff_types::TiffHeader;
use crate::types::{DirectoryInfo, Result};
use crate::generated::Olympus_pm::{lookup_olympus_camera_types, lookup_olympus_lens_types};

/// Olympus ORF format handler
/// ExifTool: lib/Image/ExifTool/Olympus.pm - TIFF-based with dual processing modes
pub struct OlympusRawHandler {
    /// Track which sections we support
    supported_sections: std::collections::HashMap<u16, &'static str>,
}

impl OlympusRawHandler {
    pub fn new() -> Self {
        let mut supported_sections = std::collections::HashMap::new();
        
        // Core data sections (tag ID -> section name)
        supported_sections.insert(0x2010, "Equipment");
        supported_sections.insert(0x2020, "CameraSettings"); 
        supported_sections.insert(0x2030, "RawDevelopment");
        supported_sections.insert(0x2031, "RawDev2");
        supported_sections.insert(0x2040, "ImageProcessing");
        supported_sections.insert(0x2050, "FocusInfo");
        supported_sections.insert(0x3000, "RawInfo");
        supported_sections.insert(0x4000, "MainInfo");
        supported_sections.insert(0x5000, "UnknownInfo");
        
        Self { supported_sections }
    }
}

impl RawFormatHandler for OlympusRawHandler {
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // ORF files are TIFF-based, parse TIFF header first
        let header = TiffHeader::parse(data)?;
        reader.set_test_header(header.clone());
        reader.set_test_data(data.to_vec());

        // Process main IFD using existing TIFF infrastructure  
        let dir_info = DirectoryInfo {
            name: "Olympus".to_string(),
            dir_start: header.ifd0_offset as usize,
            dir_len: 0,
            base: 0,
            data_pos: 0,
            allow_reprocess: false,
        };

        reader.process_subdirectory(&dir_info)?;
        
        // Apply Olympus-specific processing for known sections
        self.process_olympus_sections(reader, data)?;
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Olympus"
    }

    fn validate_format(&self, data: &[u8]) -> bool {
        // ORF files are TIFF-based
        super::super::detector::validate_olympus_orf_magic(data)
    }
}
```

### Phase 3: Section Processing Implementation (3-4 hours)

**Dual-Mode Processing** (following ExifTool's pattern):

```rust
// Each section supports both binary data and IFD processing modes
impl OlympusRawHandler {
    fn process_olympus_sections(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // Process sections found in the maker notes
        for (tag_id, section_name) in &self.supported_sections {
            if let Some(tag_value) = reader.get_extracted_tag(*tag_id) {
                match section_name {
                    "Equipment" => self.process_equipment_section(reader, &tag_value, data)?,
                    "CameraSettings" => self.process_camera_settings_section(reader, &tag_value, data)?,
                    "FocusInfo" => self.process_focus_info_section(reader, &tag_value, data)?,
                    "RawDevelopment" => self.process_raw_development_section(reader, &tag_value, data)?,
                    "ImageProcessing" => self.process_image_processing_section(reader, &tag_value, data)?,
                    _ => {
                        // Basic section processing for unknown sections
                        tracing::debug!("Found Olympus section: {} at tag {:#x}", section_name, tag_id);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn process_equipment_section(&self, reader: &mut ExifReader, tag_value: &TagValue, data: &[u8]) -> Result<()> {
        // ExifTool: Equipment section processing
        // Use generated lookup tables for camera and lens identification
        
        // Example: Extract camera type and convert using generated table
        if let Some(camera_code) = self.extract_camera_type(tag_value, data) {
            if let Some(camera_name) = lookup_olympus_camera_types(&camera_code) {
                reader.add_tag(0x0110, TagValue::String(camera_name.to_string()), "Camera", "Equipment");
            }
        }
        
        // Example: Extract lens type and convert using generated table  
        if let Some(lens_code) = self.extract_lens_type(tag_value, data) {
            if let Some(lens_name) = lookup_olympus_lens_types(&lens_code) {
                reader.add_tag(0x0111, TagValue::String(lens_name.to_string()), "Lens", "Equipment");
            }
        }
        
        Ok(())
    }
}
```

**Key Implementation Notes**:

1. **Leverage existing TIFF infrastructure** - ORF files are TIFF-based
2. **Use generated lookup tables** - Camera types, lens types, filters already available
3. **Follow Panasonic pattern** - Similar TIFF-based RAW format processing
4. **Dual processing modes** - Each section can be binary data OR IFD (ExifTool pattern)
5. **Incremental implementation** - Start with core sections, add FE sections later

### Phase 4: Testing and Integration (1-2 hours)

**Test File Availability**: Check for Olympus ORF test files in `test-images/olympus/`.

**Basic Integration Test**:

```rust
// tests/raw/olympus_tests.rs  
#[test]
fn test_olympus_orf_basic_processing() {
    let test_file = "test-images/olympus/test.orf"; // If available
    
    if !std::path::Path::new(test_file).exists() {
        eprintln!("Skipping Olympus test - no test file available");
        return;
    }
    
    let result = process_raw_file(test_file).expect("Should process ORF file");
    
    // Verify basic EXIF tags are extracted
    assert!(result.contains_key("Make"));
    assert!(result.contains_key("Model"));
    
    // Verify Olympus-specific processing 
    // (exact tags depend on what sections we implement)
}

#[test] 
fn test_olympus_handler_registration() {
    let processor = RawProcessor::new();
    let supported = processor.supported_formats();
    assert!(supported.contains(&RawFormat::Olympus));
}
```

**CLI Integration**: Once handler is implemented, `exif-oxide file.orf` should work automatically.

## Critical Implementation Knowledge

### ExifTool Dual Processing Pattern

**Key Insight**: ExifTool processes each Olympus section with **conditional logic**:

```perl
# From Olympus.pm - each section has dual processing
0x2010 => [
    {
        Name => 'Equipment',
        Condition => '$format ne "ifd" and $format ne "int32u"',
        SubDirectory => { TagTable => 'Image::ExifTool::Olympus::Equipment', ByteOrder => 'Unknown' },
    },
    {
        Name => 'EquipmentIFD', 
        Flags => 'SubIFD',
        FixFormat => 'ifd',
        SubDirectory => { TagTable => 'Image::ExifTool::Olympus::Equipment', Start => '$val' },
    },
],
```

**Translation**: Each section can be processed as:
1. **Binary data** (when `$format ne "ifd"`) - use binary data parsing
2. **IFD** (when format is `ifd`) - use standard IFD processing

### Existing Infrastructure to Leverage

1. **TIFF Processing**: `TiffHeader::parse()`, `process_subdirectory()`
2. **Generated Tables**: Use `lookup_olympus_camera_types()`, `lookup_olympus_lens_types()`
3. **Pattern**: Follow `PanasonicRawHandler` structure exactly
4. **Binary Data**: Can leverage existing binary data processing utilities

### Special Processing Functions to Implement Later

- **PrintLensInfo()**: Custom lens information formatting
- **ExtenderStatus()**: Lens extender detection logic  
- **PrintAFAreas()**: AF point visualization
- **ProcessORF()**: Main ORF processor (delegates to `ProcessTIFF()`)

### Tag ID to Section Mapping (Critical)

```rust
// Tag IDs that point to data sections in Olympus maker notes
const OLYMPUS_SECTION_TAGS: &[(u16, &str)] = &[
    (0x2010, "Equipment"),        // Camera/lens hardware info
    (0x2020, "CameraSettings"),   // Core camera settings
    (0x2030, "RawDevelopment"),   // RAW processing parameters
    (0x2031, "RawDev2"),          // Additional RAW parameters
    (0x2040, "ImageProcessing"),  // Image processing, art filters
    (0x2050, "FocusInfo"),        // Autofocus information
    (0x3000, "RawInfo"),          // RAW file specific info
    (0x4000, "MainInfo"),         // Main Olympus tag table
    (0x5000, "UnknownInfo"),      // Unknown/experimental data
    // FE model sections (0x2100-0x2900) can be added later
];
```

## Success Criteria

### Core Requirements (Revised)

- [ ] **ORF Format Detection**: Add Olympus to `RawFormat` enum and detection logic
- [ ] **Handler Registration**: Register `OlympusRawHandler` in `RawProcessor`
- [ ] **TIFF-based Processing**: ORF files processed using existing TIFF infrastructure
- [ ] **Core Section Support**: Process 9 primary data sections (Equipment, CameraSettings, etc.)
- [ ] **Generated Table Usage**: Use `lookup_olympus_camera_types()` and `lookup_olympus_lens_types()`
- [ ] **CLI Integration**: `exif-oxide file.orf` works correctly

### Validation Tests (Realistic Scope)

- [ ] **Handler Creation**: `OlympusRawHandler::new()` works
- [ ] **Format Validation**: `validate_olympus_orf_magic()` correctly identifies ORF files
- [ ] **Basic Processing**: Extract standard EXIF fields (Make, Model, etc.)
- [ ] **Generated Tables**: Camera and lens identification using generated lookups
- [ ] **ExifTool Comparison**: Compare basic output with `exiftool -j file.orf` (if test files available)

### Optional Extensions (Future)

- [ ] **FE Model Sections**: Add 0x2100-0x2900 section processing
- [ ] **Special Functions**: Implement PrintLensInfo, ExtenderStatus, PrintAFAreas
- [ ] **Complex PrintConv**: Art filter modes, focus area visualization
- [ ] **Advanced Validation**: Multiple camera model testing

## Implementation Boundaries

### Goals (Milestone 17c - Revised)

- **Basic ORF metadata extraction** using existing TIFF infrastructure
- **Core section processing** for 9 primary data sections
- **Generated table integration** for camera and lens identification
- **CLI integration** for ORF file processing
- **Foundation for future enhancements** (FE sections, special functions)

### Non-Goals (Unchanged)

- Preview extraction (Milestone 19)
- ORF file writing
- RAW image decoding
- Art Filter rendering
- **Complex PrintConv functions** (can be added incrementally later)
- **All 15+ sections** (focus on 9 core sections first)

## Dependencies and Prerequisites

**✅ Already Available**:
- **RAW Processing Infrastructure**: `RawFormatHandler` trait, `RawProcessor` dispatcher
- **TIFF Processing**: `TiffHeader::parse()`, IFD processing
- **Generated Lookup Tables**: `olympuscameratypes.rs`, `olympuslenstypes.rs`, `filters.rs`
- **Pattern to Follow**: `PanasonicRawHandler` provides exact template

**📋 Requirements**:
- **Basic Rust knowledge**: Pattern matching, HashMap usage, trait implementation
- **ExifTool Understanding**: Dual processing modes (binary data vs IFD)
- **TIFF Format Knowledge**: IFD structure, tag processing (already in codebase)

**🔧 Optional for Testing**:
- **Olympus ORF test files**: Check `test-images/olympus/` directory
- **ExifTool installation**: For output comparison validation

## Technical Notes

### ORF Structure (Simplified)

```
ORF File (TIFF-based):
├── TIFF Header (MM/II + magic)
├── Main IFD (standard TIFF)
│   ├── Standard EXIF tags
│   └── Maker Note IFD (Olympus-specific)
│       ├── 0x2010 → Equipment section
│       ├── 0x2020 → CameraSettings section  
│       ├── 0x2030 → RawDevelopment section
│       └── ... (6 more core sections)
└── Image Data
```

### Processing Strategy

1. **Parse as TIFF**: Use existing `TiffHeader::parse()` and IFD processing
2. **Process Maker Notes**: Extract Olympus-specific sections from maker note IFD
3. **Dual Mode Processing**: Each section can be binary data OR IFD (follow ExifTool pattern)
4. **Use Generated Tables**: Camera/lens identification via existing lookup functions

### Trust ExifTool Compliance

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

- **Preserve dual processing logic**: Binary data vs IFD modes
- **Use exact tag IDs**: 0x2010-0x5000 for core sections
- **Leverage existing patterns**: Follow `PanasonicRawHandler` structure
- **Generated tables**: Use codegen infrastructure instead of manual lookups

### Implementation Shortcuts (Allowed)

- **Start with core sections**: Implement 9 primary sections, add FE sections later
- **Basic processing first**: Focus on tag extraction, add complex PrintConv later
- **Leverage infrastructure**: Use existing TIFF and RAW processing patterns

## Risk Mitigation (Revised)

### Implementation Complexity

- **Risk**: Overengineering the initial implementation
- **Mitigation**: Start with basic TIFF processing, add sections incrementally
- **Pattern**: Follow existing `PanasonicRawHandler` exactly

### Missing Test Files

- **Risk**: No Olympus ORF test files available for validation
- **Mitigation**: Implement handler structure first, test with available files later
- **Alternative**: Create minimal validation tests that don't require specific files

### Generated Table Integration

- **Risk**: Incorrect usage of generated lookup tables
- **Mitigation**: Study existing usage in `olympuscameratypes.rs` and `olympuslenstypes.rs`
- **Validation**: Test lookup functions with known values

### TIFF Processing Complexity

- **Risk**: ORF files may have TIFF variations not handled by existing code
- **Mitigation**: Start with standard TIFF processing, add ORF-specific handling as needed
- **Fallback**: Basic EXIF extraction should work even if Olympus sections fail

## Next Steps

After successful completion:

1. **Incremental Enhancement**: Add FE model sections (0x2100-0x2900) if needed
2. **Special Functions**: Implement PrintLensInfo, ExtenderStatus, PrintAFAreas
3. **Milestone 17d**: Canon CR2 (using similar TIFF-based pattern)
4. **Milestone 17e**: Sony ARW (advanced offset management)

## Summary (Revised)

Olympus ORF implementation is **significantly simplified** from the original estimate due to:

- **Existing RAW infrastructure** handles TIFF-based processing
- **Generated lookup tables** eliminate manual table maintenance  
- **Proven patterns** from `PanasonicRawHandler` provide exact template
- **Incremental approach** allows core functionality first, enhancements later

This milestone validates that our RAW processing architecture can handle manufacturer-specific formats efficiently while leveraging existing infrastructure and generated code.

## Engineer Onboarding Notes

**For a new engineer taking this work**:

1. **Study Existing Code**: Read `src/raw/formats/panasonic.rs` first - it's the exact pattern to follow
2. **Understand Generated Tables**: Examine `src/generated/Olympus_pm/` to see available lookup functions
3. **Start Simple**: Focus on detection and basic TIFF processing before adding section-specific logic
4. **Trust ExifTool**: When in doubt, check ExifTool's Olympus.pm for exact processing logic
5. **Test Incrementally**: Build and test each phase before moving to the next

**Time Estimate**: 6-8 hours for a engineer familiar with the codebase, 1-2 days for a new engineer.