# Milestone 17a: RAW Foundation & Kyocera Format

**Status**: ✅ COMPLETED - July 2025  
**Duration**: 1-2 weeks  
**Goal**: Build core RAW processing infrastructure with the simplest format (Kyocera)

## Overview

This milestone establishes the foundation for all RAW format support by implementing core infrastructure and validating it with the simplest possible format: KyoceraRaw (173 lines in ExifTool). This provides a minimal viable RAW implementation while building the architecture needed for more complex formats.

## Background

**Why Start with Kyocera?**

- **Simplest format**: Only 173 lines in ExifTool (vs 14K+ for Nikon)
- **Basic ProcessBinaryData**: Straightforward binary data parsing
- **No complex offsets**: Simple, predictable structure
- **TIFF-based**: Uses standard TIFF container
- **Rare but valid**: Good test of architecture without complexity

## Implementation Strategy

### Phase 1: Core RAW Infrastructure (Week 1)

**RAW Format Detection**:

```rust
// src/raw/mod.rs
pub mod detector;
pub mod processor;
pub mod router;

use crate::file_type::FileType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawFormat {
    Kyocera,     // Starting simple
    // Future formats will be added in subsequent milestones
    Unknown,
}

/// Detect RAW format from file type and metadata
/// ExifTool: various manufacturer modules
pub fn detect_raw_format(file_type: FileType, make: &str) -> RawFormat {
    match (file_type, make) {
        (FileType::RAW, "Kyocera") => RawFormat::Kyocera,
        _ => RawFormat::Unknown,
    }
}
```

**RAW Processor Foundation**:

```rust
// src/raw/processor.rs
use crate::exif::ExifReader;
use crate::tiff::TiffProcessor;
use crate::error::Result;

pub struct RawProcessor {
    tiff_processor: TiffProcessor,
    format_handlers: HashMap<RawFormat, Box<dyn RawFormatHandler>>,
}

/// Trait for manufacturer-specific RAW handlers
pub trait RawFormatHandler: Send + Sync {
    /// Process manufacturer-specific maker notes
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()>;

    /// Get handler name for debugging
    fn name(&self) -> &'static str;
}

impl RawProcessor {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();

        // Register Kyocera handler
        handlers.insert(RawFormat::Kyocera,
            Box::new(KyoceraRawHandler::new()) as Box<dyn RawFormatHandler>);

        Self {
            tiff_processor: TiffProcessor::new(),
            format_handlers: handlers,
        }
    }

    pub fn process_raw(&mut self, reader: &mut ExifReader) -> Result<()> {
        // 1. Detect format
        let make = reader.get_tag_value("Make").unwrap_or_default();
        let file_type = reader.file_type();
        let format = detect_raw_format(file_type, &make);

        // 2. Process TIFF structure (common to most RAW)
        self.tiff_processor.process_tiff(reader)?;

        // 3. Process manufacturer-specific data
        if let Some(handler) = self.format_handlers.get(&format) {
            if let Some(maker_note_offset) = reader.get_maker_note_offset() {
                let data = reader.get_maker_note_data()?;
                handler.process_maker_notes(reader, &data, maker_note_offset)?;
            }
        }

        Ok(())
    }
}
```

### Phase 2: Kyocera Implementation (Week 1)

**Kyocera Handler**:

```rust
// src/raw/formats/kyocera.rs
use crate::implementations::binary_data::{ProcessBinaryData, BinaryDataEntry};

pub struct KyoceraRawHandler {
    binary_processor: ProcessBinaryData,
}

impl KyoceraRawHandler {
    pub fn new() -> Self {
        // ExifTool: lib/Image/ExifTool/KyoceraRaw.pm
        // Simple ProcessBinaryData with fixed offsets
        let entries = vec![
            BinaryDataEntry::new(0x00, 4, "Make", None),
            BinaryDataEntry::new(0x04, 4, "Model", None),
            BinaryDataEntry::new(0x08, 4, "DateTime", None),
            // ... rest of entries from KyoceraRaw.pm
        ];

        Self {
            binary_processor: ProcessBinaryData::new(entries),
        }
    }
}

impl RawFormatHandler for KyoceraRawHandler {
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()> {
        // ExifTool: ProcessBinaryData(\$dirInfo, \$tagTablePtr)
        self.binary_processor.process(reader, data, offset, "Kyocera")
    }

    fn name(&self) -> &'static str {
        "KyoceraRaw"
    }
}
```

### Phase 3: CLI Integration (Week 2)

**File Type Detection Update**:

```rust
// Update src/file_type/detector.rs
impl FileTypeDetector {
    pub fn detect_from_extension(&self, path: &Path) -> Option<FileType> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            // Existing extensions...
            "raw" => Some(FileType::RAW),  // Generic RAW extension
            _ => None,
        }
    }
}
```

**CLI RAW Support**:

```rust
// Update src/main.rs processor routing
match file_type {
    FileType::JPEG => jpeg_processor.process(reader)?,
    FileType::TIFF => tiff_processor.process(reader)?,
    FileType::RAW => {
        // Use RAW processor for all RAW formats
        let mut raw_processor = RawProcessor::new();
        raw_processor.process_raw(reader)?;
    }
    // ... other formats
}
```

### Phase 4: Testing Infrastructure (Week 2)

**Kyocera Test Suite**:

```rust
// tests/raw/kyocera_tests.rs
#[test]
fn test_kyocera_raw_metadata() {
    let test_file = "test-images/kyocera/sample.raw";

    // Run exif-oxide
    let oxide_output = run_exif_oxide(test_file)?;

    // Run ExifTool
    let exiftool_output = run_exiftool_json(test_file)?;

    // Compare core tags
    assert_tag_match(&oxide_output, &exiftool_output, "Make");
    assert_tag_match(&oxide_output, &exiftool_output, "Model");
    assert_tag_match(&oxide_output, &exiftool_output, "DateTime");
    assert_tag_match(&oxide_output, &exiftool_output, "ISO");
    assert_tag_match(&oxide_output, &exiftool_output, "ExposureTime");
}

#[test]
fn test_raw_format_detection() {
    let detector = RawFormatDetector::new();

    // Test Kyocera detection
    assert_eq!(
        detector.detect(FileType::RAW, "Kyocera"),
        RawFormat::Kyocera
    );

    // Test unknown manufacturer
    assert_eq!(
        detector.detect(FileType::RAW, "Unknown"),
        RawFormat::Unknown
    );
}
```

## Success Criteria

### Core Requirements

- [x] **RAW Infrastructure**: Core RawProcessor, RawFormatHandler trait, format detection ✅
- [x] **Kyocera Support**: Full KyoceraRaw.pm implementation (173 lines) ✅
- [x] **CLI Integration**: `exif-oxide kyocera.raw` successfully extracts metadata ✅
- [x] **Test Coverage**: Compatibility tests pass against ExifTool output ✅
- [x] **No Binary Data**: Metadata only - no preview/thumbnail extraction ✅
- [x] **🔧 Compat Script Update**: Add "raw" to `SUPPORTED_EXTENSIONS` in `tools/generate_exiftool_json.sh` and regenerate reference files with `make compat-gen` ✅

### Validation Tests

- [x] Process Kyocera RAW sample files ✅
- [x] Extract basic EXIF data (Make, Model, DateTime, ISO, etc.) ✅
- [x] Verify output matches `exiftool -j kyocera.raw` ✅
- [x] Handle missing/corrupt Kyocera files gracefully ✅

## Implementation Boundaries

### Goals (Milestone 17a)

- Establish RAW processing architecture
- Implement simplest format (Kyocera) completely
- Validate infrastructure works end-to-end
- Set foundation for future formats

### Non-Goals

- Other RAW formats (future milestones)
- Preview/thumbnail extraction (Milestone 19)
- Advanced offset management (not needed for Kyocera)
- Write support

## Dependencies and Prerequisites

- Basic TIFF processing infrastructure (should already exist)
- ProcessBinaryData implementation
- File type detection system
- ExifTool Kyocera test images

## Technical Notes

### Trust ExifTool

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), we implement Kyocera support exactly as ExifTool does:

- Use same tag offsets from KyoceraRaw.pm
- Preserve any quirks or special handling
- No "improvements" or "optimizations"

### ProcessBinaryData Pattern

Kyocera uses ExifTool's ProcessBinaryData pattern:

```perl
# From KyoceraRaw.pm
%Image::ExifTool::KyoceraRaw::Main = (
    PROCESS_PROC => \&Image::ExifTool::ProcessBinaryData,
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    FIRST_ENTRY => 0,
    # Table entries with offsets...
);
```

We translate this pattern exactly to our Rust implementation.

## Risk Mitigation

### Sample File Availability

- **Risk**: Kyocera RAW files are rare
- **Mitigation**: Request samples from user if not in test-images/
- **Alternative**: Create synthetic test files based on format spec

### Architecture Validation

- **Risk**: Foundation might not scale to complex formats
- **Mitigation**: Design with Sony/Canon complexity in mind
- **Validation**: Review against most complex format requirements

## Next Steps

After successful completion:

1. Milestone 17b: Add Minolta/Panasonic (simple TIFF-based)
2. Milestone 17c: Add Olympus (medium complexity)
3. Continue building on proven foundation

## Summary

This milestone establishes RAW format support with the absolute simplest case, validating our architecture while delivering working functionality. The Kyocera implementation serves as both a proof of concept and the foundation for all future RAW format support.

---

## ✅ COMPLETION STATUS (July 2025)

**Successfully completed and validated** with all success criteria met:

### ✅ Core Requirements Validated
- **RAW Infrastructure**: Complete RawProcessor, RawFormatHandler trait, format detection implemented in `src/raw/` module ✅
- **Kyocera Support**: Full KyoceraRaw.pm implementation (173 lines) with 11 tag definitions in `src/raw/formats/kyocera.rs` ✅
- **CLI Integration**: `exif-oxide third-party/exiftool/t/images/KyoceraRaw.raw` successfully extracts all 18 metadata tags ✅
- **Test Coverage**: All 9 integration tests pass against real ExifTool test file `third-party/exiftool/t/images/KyoceraRaw.raw` ✅
- **No Binary Data**: Metadata-only extraction as specified (no preview/thumbnail functionality) ✅

### ✅ Validation Tests Passed (July 18, 2025)
- **Process Kyocera RAW sample files**: ✅ Successfully processed `KyoceraRaw.raw` (166 bytes)
- **Extract basic EXIF data**: ✅ Extracted Make: "KYOCERA", Model: "N DIGITAL", DateTime: "2005:07:16 18:14:30", ISO: 100
- **Verify output matches ExifTool**: ✅ All 11 Kyocera tags match ExifTool output exactly (FirmwareVersion, Model, Make, DateTimeOriginal, ISO, ExposureTime, WB_RGGBLevels, FNumber, MaxAperture, FocalLength, Lens)
- **Handle missing/corrupt files**: ✅ Graceful error handling with minimal test files and boundary conditions

### ✅ Current Test Results (Validated July 18, 2025)
```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

**CLI Output Validation**: Our implementation produces 18 total tags vs ExifTool's 30 tags, but all core Kyocera-specific metadata is correctly extracted with exact value matches.

### Key Implementation Details
- **Trust ExifTool**: Exact translation of KyoceraRaw.pm logic with source line references (lines 26-106)
- **Architecture**: Trait-based handler system enabling future manufacturer expansion
- **Binary Processing**: Big-endian data parsing with string reversal utilities matching ExifTool's ReverseString function
- **Integration**: Seamless integration with existing ExifReader and format detection systems
- **Testing**: Used real ExifTool test file instead of synthetic data (following CLAUDE.md guidance)
- **Format Detection**: Correctly identifies RAW files with Kyocera magic bytes "ARECOYK"
- **Value Conversion**: Implements all ExifTool conversions (exposure time, F-number, ISO lookup, string reversal)

### Files Created/Modified
- `src/raw/mod.rs` - Core RAW utilities and format detection
- `src/raw/detector.rs` - RAW format detection logic  
- `src/raw/processor.rs` - RawProcessor and handler trait system
- `src/raw/formats/kyocera.rs` - Complete KyoceraRawHandler implementation
- `src/formats/mod.rs` - RAW processing integration
- `tests/raw_integration_tests.rs` - Comprehensive test suite with 9 test cases

### Next Steps
Ready for Milestone 17b: Add Minolta/Panasonic (simple TIFF-based formats) using the proven architecture.

**Note**: Milestone 17b appears to already be implemented based on test results showing successful MRW and RW2 processing.
