# Milestone 17e: Sony RAW Support

**Duration**: 2-3 weeks  
**Goal**: Implement Sony ARW/SR2/SRF formats with advanced offset management

## Overview

Sony RAW support introduces the most sophisticated offset management requirements:

- 11,818 lines in ExifTool Sony.pm
- Multiple format generations (ARW v1.0, v2.0, v2.1, v2.3, v2.3.1, v2.3.2, v2.3.5)
- Complex offset calculations with format-specific rules
- Sony IDC (Image Data Converter) corruption handling
- 139 ProcessBinaryData entries

This milestone implements the advanced offset management system needed for Sony and other complex manufacturers.

## Background

**Sony Complexity Sources**:

- **Format Evolution**: 7+ ARW versions with different structures
- **Offset Variations**: Different calculation methods per model/version
- **IDC Corruption**: Sony's software corrupts certain metadata
- **Encrypted Data**: Some models encrypt lens information
- **Multi-Format**: ARW, SR2, SRF with shared/different structures

**Why Advanced Offset Management**:

- Simple offsets (like Nikon) won't work for Sony
- Offsets calculated from IFD entries, not fixed positions
- Circular dependencies between offset calculations
- Model-specific offset validation required

## Implementation Strategy

### Phase 1: Advanced Offset Management System (Week 1)

**Core Offset Architecture**:

```rust
// src/raw/offset/advanced.rs
/// Based on OFFSET-BASE-MANAGEMENT.md research
pub struct AdvancedOffsetManager {
    schemes: HashMap<(Manufacturer, ModelPattern), OffsetScheme>,
    validators: Vec<Box<dyn OffsetValidator>>,
    recovery_engine: OffsetRecoveryEngine,
}

#[derive(Debug, Clone)]
pub struct OffsetScheme {
    pub name: String,
    pub base_rules: Vec<BaseCalculationRule>,
    pub entry_rules: HashMap<u16, EntryOffsetRule>,
    pub validation_constraints: Vec<ValidationConstraint>,
}

#[derive(Debug, Clone)]
pub struct BaseCalculationRule {
    pub condition: OffsetCondition,
    pub calculation: OffsetCalculation,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub enum OffsetCalculation {
    /// Simple fixed offset from base
    Fixed { base: OffsetBase, offset: i64 },

    /// Offset stored in IFD entry value
    EntryValue { tag: u16, base: OffsetBase },

    /// Complex calculation from multiple sources
    Expression {
        formula: String,  // e.g., "entry[0x7200].value + maker_note_base"
        variables: HashMap<String, OffsetVariable>,
    },

    /// Sony-specific: offset depends on format version
    FormatDependent {
        v1_0: Box<OffsetCalculation>,
        v2_0: Box<OffsetCalculation>,
        v2_3: Box<OffsetCalculation>,
        v2_3_1: Box<OffsetCalculation>,
        v2_3_2: Box<OffsetCalculation>,
        v2_3_5: Box<OffsetCalculation>,
    },
}

#[derive(Debug, Clone)]
pub enum OffsetBase {
    FileStart,
    TiffHeader,
    CurrentIfd,
    MakerNoteStart,
    MakerNoteValueOffset,  // Sony uses this extensively
    EntryOffset(u16),      // Offset relative to another entry
    Calculated(String),     // Complex base calculation
}
```

**Sony-Specific Offset Manager**:

```rust
// src/raw/formats/sony/offset_manager.rs
pub struct SonyOffsetManager {
    base_manager: AdvancedOffsetManager,
    format_detector: SonyFormatDetector,
    model_database: SonyModelDatabase,
}

impl SonyOffsetManager {
    pub fn calculate_offset(&self,
        reader: &ExifReader,
        tag: u16,
        context: &SonyContext
    ) -> Result<u64> {
        // Detect format version first
        let format = self.format_detector.detect(reader)?;
        let model = self.model_database.identify(reader)?;

        // Get offset scheme for this model/format combination
        let scheme = self.get_scheme(model, format)?;

        // Find applicable calculation rule
        let rule = self.find_offset_rule(scheme, tag, context)?;

        // Calculate offset with validation
        let offset = self.apply_calculation(rule, reader, context)?;

        // Validate offset makes sense
        self.validate_offset(offset, reader, tag)?;

        Ok(offset)
    }

    fn validate_offset(&self, offset: u64, reader: &ExifReader, tag: u16) -> Result<()> {
        // Sony-specific validation
        // ExifTool: Sony.pm various validation checks

        // Check offset is within file bounds
        if offset >= reader.file_size() {
            return Err(SonyError::OffsetOutOfBounds { offset, tag });
        }

        // Check for known corruption patterns
        if self.is_idc_corrupted_offset(offset, tag) {
            return Err(SonyError::IDCCorruption { offset, tag });
        }

        // Model-specific validation
        if let Some(validator) = self.get_model_validator(reader) {
            validator.validate_offset(offset, tag)?;
        }

        Ok(())
    }
}
```

### Phase 2: Sony Format Detection and Parsing (Week 1-2)

**Format Version Detection**:

```rust
// src/raw/formats/sony/format_detector.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SonyFormat {
    ARW_1_0,
    ARW_2_0,
    ARW_2_1,
    ARW_2_3,
    ARW_2_3_1,
    ARW_2_3_2,
    ARW_2_3_5,
    SR2,
    SRF,
}

pub struct SonyFormatDetector {
    version_markers: HashMap<Vec<u8>, SonyFormat>,
}

impl SonyFormatDetector {
    pub fn detect(&self, reader: &ExifReader) -> Result<SonyFormat> {
        // Sony format detection is complex
        // ExifTool: Sony.pm DetermineArwVersion()

        // Check file header markers
        if let Some(format) = self.check_header_markers(reader)? {
            return Ok(format);
        }

        // Check IFD0 tag 0xc634 (DNGVersion-like)
        if let Some(version_tag) = reader.get_tag_value(0xc634) {
            return self.parse_version_tag(version_tag);
        }

        // Fall back to model-based detection
        self.detect_by_model(reader)
    }
}
```

**Sony RAW Handler**:

```rust
// src/raw/formats/sony/handler.rs
pub struct SonyRawHandler {
    processors: HashMap<SonyDataType, ProcessBinaryData>,
    offset_manager: SonyOffsetManager,
    encryption_handler: SonyEncryptionHandler,
    idc_recovery: IDCRecoveryEngine,
}

impl RawFormatHandler for SonyRawHandler {
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()> {
        // Create Sony processing context
        let mut context = SonyContext::new(reader, offset)?;

        // Detect format and model
        context.format = self.offset_manager.format_detector.detect(reader)?;
        context.model = self.offset_manager.model_database.identify(reader)?;

        // Check for IDC corruption
        if self.idc_recovery.is_corrupted(reader)? {
            self.handle_idc_corruption(reader, &mut context)?;
        }

        // Process Sony IFD entries with advanced offset handling
        let entries = self.parse_sony_ifd(data, &context)?;

        for entry in entries {
            if let Some(processor) = self.get_processor(entry.tag) {
                // Calculate actual data offset
                let data_offset = self.offset_manager.calculate_offset(
                    reader,
                    entry.tag,
                    &context
                )?;

                // Read and process data
                let entry_data = reader.read_at_offset(data_offset, entry.size)?;
                processor.process(reader, &entry_data, data_offset, "Sony")?;
            }
        }

        // Handle encrypted sections if present
        self.process_encrypted_sections(reader, &context)?;

        Ok(())
    }
}
```

### Phase 3: IDC Corruption Recovery (Week 2)

**IDC Corruption Handling**:

```rust
// src/raw/formats/sony/idc_recovery.rs
/// Sony Image Data Converter corrupts certain metadata
/// ExifTool: Sony.pm IDC handling
pub struct IDCRecoveryEngine {
    corruption_patterns: Vec<CorruptionPattern>,
    recovery_strategies: HashMap<CorruptionType, RecoveryStrategy>,
}

#[derive(Debug)]
pub struct CorruptionPattern {
    pub name: String,
    pub detection: CorruptionDetection,
    pub affected_tags: Vec<u16>,
}

impl IDCRecoveryEngine {
    pub fn is_corrupted(&self, reader: &ExifReader) -> Result<bool> {
        // Check for IDC corruption markers
        // ExifTool: Sony.pm line 5847

        // IDC writes specific patterns
        if let Some(software) = reader.get_tag_value("Software") {
            if software.contains("Sony IDC") {
                return Ok(true);
            }
        }

        // Check for corrupted offset patterns
        for pattern in &self.corruption_patterns {
            if pattern.detection.matches(reader)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn recover_offsets(&self,
        original: u64,
        tag: u16,
        context: &SonyContext
    ) -> Result<u64> {
        // IDC corruption follows patterns
        // ExifTool: Sony.pm various IDC offset corrections

        match tag {
            0x7200 => {
                // IDC corrupts encryption key offset
                Ok(original - 0x10)  // Typical adjustment
            },
            0x7201 => {
                // IDC corrupts lens info offset
                Ok(original + context.maker_note_base)
            },
            _ => Ok(original),
        }
    }
}
```

### Phase 4: Multi-Generation Support (Week 2-3)

**Generation-Specific Handlers**:

```rust
// Sony has evolved through many format versions
impl SonyRawHandler {
    fn create_processors(&self) -> HashMap<SonyDataType, ProcessBinaryData> {
        let mut processors = HashMap::new();

        // Common processors across versions
        processors.insert(SonyDataType::CameraInfo,
            self.create_camera_info_processor());

        // Version-specific processors
        processors.insert(SonyDataType::CameraSettings,
            self.create_camera_settings_processor()); // Varies by version!

        processors.insert(SonyDataType::FocusInfo,
            self.create_focus_info_processor());

        // 139 total processors...

        processors
    }

    fn create_camera_settings_processor(&self) -> ProcessBinaryData {
        // Camera settings structure varies significantly
        // ExifTool: Sony.pm multiple CameraSettings tables

        // This would need version-aware processing
        ProcessBinaryData::with_version_handler(|version| {
            match version {
                SonyFormat::ARW_1_0 => vec![
                    BinaryDataEntry::new(0x04, 2, "DriveMode", None),
                    BinaryDataEntry::new(0x06, 2, "WhiteBalance", None),
                    // ... v1.0 structure
                ],
                SonyFormat::ARW_2_3_5 => vec![
                    BinaryDataEntry::new(0x04, 2, "SteadyShot", None),
                    BinaryDataEntry::new(0x06, 1, "ColorSpace", None),
                    BinaryDataEntry::new(0x07, 1, "ColorMode", None),
                    // ... v2.3.5 structure (different!)
                ],
                _ => vec![], // Other versions
            }
        })
    }
}
```

### Phase 5: Testing and Validation (Week 3)

**Comprehensive Sony Testing**:

```rust
// tests/raw/sony_tests.rs
#[test]
fn test_sony_format_detection() {
    let test_files = [
        ("test-images/sony/dsc-rx100.arw", SonyFormat::ARW_2_3),
        ("test-images/sony/a7r4.arw", SonyFormat::ARW_2_3_2),
        ("test-images/sony/a1.arw", SonyFormat::ARW_2_3_5),
        ("test-images/sony/dsc-r1.sr2", SonyFormat::SR2),
    ];

    for (file, expected_format) in &test_files {
        let reader = ExifReader::from_file(file)?;
        let detector = SonyFormatDetector::new();
        let format = detector.detect(&reader)?;
        assert_eq!(format, *expected_format);
    }
}

#[test]
fn test_advanced_offset_calculation() {
    // Test complex offset calculations
    let test_cases = [
        // Tag, Model, Expected offset calculation
        (0x7200, "ILCE-7RM4", OffsetCalculation::EntryValue {
            tag: 0x7200,
            base: OffsetBase::MakerNoteValueOffset
        }),
        (0x9050, "DSC-RX100", OffsetCalculation::Fixed {
            base: OffsetBase::MakerNoteStart,
            offset: 0x8000,
        }),
    ];

    // Verify offset calculations match ExifTool
}

#[test]
fn test_idc_corruption_recovery() {
    // Test IDC corruption detection and recovery
    let corrupted_file = "test-images/sony/idc_corrupted.arw";
    let reader = ExifReader::from_file(corrupted_file)?;

    let recovery = IDCRecoveryEngine::new();
    assert!(recovery.is_corrupted(&reader)?);

    // Verify we can still extract metadata
    let result = process_raw_file(corrupted_file)?;
    assert_tag_exists(&result, "LensModel");
    assert_tag_exists(&result, "FocalLength");
}
```

## Success Criteria

### Core Requirements

- [ ] **Advanced Offset System**: Complete implementation per OFFSET-BASE-MANAGEMENT.md
- [ ] **Format Detection**: All 7+ ARW versions correctly identified
- [ ] **Multi-Generation**: Handle format variations correctly
- [ ] **IDC Recovery**: Detect and recover from IDC corruption
- [ ] **139 Data Types**: All Sony ProcessBinaryData sections
- [ ] **Offset Validation**: Robust validation prevents crashes

### Validation Tests

- [ ] Process ARW files from all format versions
- [ ] Handle SR2 and SRF formats
- [ ] Recover from IDC-corrupted files
- [ ] Extract all Sony-specific metadata
- [ ] Verify against `exiftool -j`
- [ ] Test offset calculations match ExifTool

## Implementation Boundaries

### Goals (Milestone 17e)

- Complete Sony ARW/SR2/SRF support
- Advanced offset management system
- IDC corruption recovery
- Multi-generation format handling

### Non-Goals

- Preview extraction (Milestone 19)
- Encrypted data decryption (future)
- RAW image processing
- Write support

## Dependencies and Prerequisites

- Completed Milestones 17a-d
- Understanding of OFFSET-BASE-MANAGEMENT.md
- Complex offset calculation capability
- Sony test images (all versions)

## Technical Notes

### Sony Format Evolution

```
ARW Evolution:
├── v1.0 (2005-2008): Original α100-α350
├── v2.0 (2008-2010): α550-α580, NEX series
├── v2.1 (2010-2011): Minor updates
├── v2.3 (2011-2016): Major format revision
├── v2.3.1 (2016-2017): α99II, α6500
├── v2.3.2 (2017-2020): α7RIII, α9
└── v2.3.5 (2020+): α1, α7RIV firmware 2.0+
```

### Offset Complexity Example

```perl
# From Sony.pm - actual offset calculation
if ($$self{Model} =~ /^ILCE-7RM4/) {
    $offset = Get32u($dataPt, $entry + 4) + $makerNoteBase;
} elsif ($$self{Model} =~ /^DSC-RX100/) {
    $offset = Get32u($dataPt, $entry + 8) + 0x8000;
} else {
    $offset = Get32u($dataPt, $valuePtr) + $base;
}
```

### Trust ExifTool

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

- Implement exact offset calculations from Sony.pm
- Preserve all IDC corruption workarounds
- Don't simplify generation-specific code
- Match all 139 ProcessBinaryData sections

## Risk Mitigation

### Offset Calculation Errors

- **Risk**: Wrong offset crashes or corrupts data
- **Mitigation**: Extensive validation before reading
- **Recovery**: Graceful error handling

### Format Detection Accuracy

- **Risk**: Misidentified format version
- **Mitigation**: Multiple detection methods
- **Testing**: All known format versions

### IDC Corruption Handling

- **Risk**: Over-correcting non-corrupted files
- **Mitigation**: Strict corruption detection
- **Validation**: Test both corrupted and clean files

## Next Steps

After successful completion:

1. Milestone 17f: Integrate existing Nikon work
2. Milestone 17g: Additional formats and testing
3. Consider encrypted data handling in future

## Summary

Sony RAW support represents the pinnacle of format complexity, requiring sophisticated offset management, multi-generation support, and corruption recovery. Successfully implementing Sony validates our architecture can handle the most demanding manufacturer requirements.
