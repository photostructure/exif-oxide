# Milestone 17b: Simple TIFF-Based RAW Formats

**Duration**: 2 weeks  
**Goal**: Implement Minolta MRW and Panasonic RW2 RAW formats

## Overview

Building on the foundation from Milestone 17a, this milestone adds support for two straightforward TIFF-based RAW formats:

- **Minolta MRW** (537 lines) - Clean TIFF structure with simple maker notes
- **Panasonic RW2** (956 lines) - TIFF with entry-based offset handling

These formats introduce slightly more complexity while remaining manageable, setting the stage for more complex manufacturers.

## Background

**Format Characteristics**:

- **MRW**: Minolta's RAW format, very clean TIFF implementation
- **RW2**: Panasonic's modern RAW format, introduces entry-based offsets
- Both use standard TIFF containers with manufacturer-specific IFDs

**Complexity Progression**:

- Kyocera (17a): 173 lines, basic ProcessBinaryData
- Minolta: 537 lines, multiple data blocks
- Panasonic: 956 lines, entry-based offsets
- Next (Olympus): 4,235 lines, 15 ProcessBinaryData sections

## Implementation Strategy

### Phase 1: Minolta MRW Support (Week 1)

**Format Detection Update**:

```rust
// src/raw/detector.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawFormat {
    Kyocera,
    Minolta,    // New
    Panasonic,  // New
    Unknown,
}

pub fn detect_raw_format(file_type: FileType, make: &str) -> RawFormat {
    match (file_type, make) {
        (FileType::RAW, "Kyocera") => RawFormat::Kyocera,
        (FileType::MRW, _) => RawFormat::Minolta,
        (FileType::RW2, _) => RawFormat::Panasonic,
        (FileType::RAW, make) if make.contains("MINOLTA") => RawFormat::Minolta,
        (FileType::RAW, "Panasonic") => RawFormat::Panasonic,
        _ => RawFormat::Unknown,
    }
}
```

**Minolta Handler Implementation**:

```rust
// src/raw/formats/minolta.rs
use crate::implementations::binary_data::ProcessBinaryData;

pub struct MinoltaRawHandler {
    // ExifTool: lib/Image/ExifTool/MinoltaRaw.pm
    mrw_processor: ProcessBinaryData,
    prd_processor: ProcessBinaryData,  // PRD block
    ttw_processor: ProcessBinaryData,  // TTW block
    wbg_processor: ProcessBinaryData,  // WBG block
}

impl MinoltaRawHandler {
    pub fn new() -> Self {
        // From MinoltaRaw.pm - multiple ProcessBinaryData blocks
        Self {
            mrw_processor: Self::create_mrw_processor(),
            prd_processor: Self::create_prd_processor(),
            ttw_processor: Self::create_ttw_processor(),
            wbg_processor: Self::create_wbg_processor(),
        }
    }

    fn create_mrw_processor() -> ProcessBinaryData {
        // Main MRW data block
        // ExifTool: %Image::ExifTool::MinoltaRaw::Main
        ProcessBinaryData::new(vec![
            // Entries from MinoltaRaw.pm Main table
            BinaryDataEntry::new(0x00, 4, "PRD", None),  // Pointer to PRD
            BinaryDataEntry::new(0x04, 4, "TTW", None),  // Pointer to TTW
            BinaryDataEntry::new(0x08, 4, "WBG", None),  // Pointer to WBG
            // ... rest of entries
        ])
    }
}

impl RawFormatHandler for MinoltaRawHandler {
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()> {
        // Process MRW header to find data blocks
        let header = self.parse_mrw_header(data)?;

        // Process each data block based on type
        for block in header.blocks {
            match block.block_type.as_str() {
                "PRD" => self.prd_processor.process(reader, &block.data, block.offset, "MinoltaPRD")?,
                "TTW" => self.ttw_processor.process(reader, &block.data, block.offset, "MinoltaTTW")?,
                "WBG" => self.wbg_processor.process(reader, &block.data, block.offset, "MinoltaWBG")?,
                _ => {} // Unknown block, skip
            }
        }

        Ok(())
    }

    fn parse_mrw_header(&self, data: &[u8]) -> Result<MrwHeader> {
        // Parse MRW file structure
        // ExifTool: Image::ExifTool::MinoltaRaw::ProcessMRW
        // MRW format has multiple data blocks at different offsets
        todo!("Parse MRW header structure")
    }
}
```

### Phase 2: Panasonic RW2 Support (Week 1-2)

**Entry-Based Offset Introduction**:

```rust
// src/raw/offset/entry_based.rs
/// Panasonic uses offsets stored in IFD entries
/// rather than fixed offsets
pub struct EntryBasedOffsetProcessor {
    /// Map of tag ID to offset extraction rules
    offset_rules: HashMap<u16, OffsetExtractionRule>,
}

#[derive(Debug, Clone)]
pub struct OffsetExtractionRule {
    pub tag_id: u16,
    pub offset_field: OffsetField,
    pub base: OffsetBase,
}

#[derive(Debug, Clone)]
pub enum OffsetField {
    ValueOffset,    // Use the value offset field
    ActualValue,    // Use the value itself as offset
}

#[derive(Debug, Clone)]
pub enum OffsetBase {
    FileStart,
    IfdStart,
    MakerNoteStart,
}
```

**Panasonic Handler**:

```rust
// src/raw/formats/panasonic.rs
pub struct PanasonicRawHandler {
    binary_processor: ProcessBinaryData,
    offset_processor: EntryBasedOffsetProcessor,
}

impl PanasonicRawHandler {
    pub fn new() -> Self {
        // ExifTool: lib/Image/ExifTool/PanasonicRaw.pm
        // Uses entry-based offsets for some data

        let mut offset_rules = HashMap::new();

        // From PanasonicRaw.pm - entry-based offset tags
        offset_rules.insert(0x002e, OffsetExtractionRule {
            tag_id: 0x002e,
            offset_field: OffsetField::ValueOffset,
            base: OffsetBase::MakerNoteStart,
        });

        Self {
            binary_processor: Self::create_binary_processor(),
            offset_processor: EntryBasedOffsetProcessor::new(offset_rules),
        }
    }

    fn create_binary_processor() -> ProcessBinaryData {
        // Main Panasonic maker note data
        ProcessBinaryData::new(vec![
            // From PanasonicRaw.pm Main table
            BinaryDataEntry::new(0x04, 2, "ImageWidth", None),
            BinaryDataEntry::new(0x06, 2, "ImageHeight", None),
            BinaryDataEntry::new(0x18, 2, "ISO", None),
            // ... more entries
        ])
    }
}

impl RawFormatHandler for PanasonicRawHandler {
    fn process_maker_notes(&self, reader: &mut ExifReader, data: &[u8], offset: u64) -> Result<()> {
        // First, process standard binary data
        self.binary_processor.process(reader, data, offset, "Panasonic")?;

        // Then handle entry-based offsets
        // This is unique to Panasonic and some other manufacturers
        let entries = reader.get_current_ifd_entries()?;

        for entry in entries {
            if let Some(rule) = self.offset_processor.get_rule(entry.tag) {
                let data_offset = self.calculate_offset(entry, rule)?;
                let data = reader.read_at_offset(data_offset, entry.count as usize)?;

                // Process the data found at the calculated offset
                self.process_offset_data(reader, entry.tag, &data)?;
            }
        }

        Ok(())
    }
}
```

### Phase 3: Shared Infrastructure Improvements (Week 2)

**Enhanced TIFF Processing**:

```rust
// src/tiff/raw_extensions.rs
/// Extensions to TIFF processing for RAW formats
pub trait RawTiffExtensions {
    /// Get maker note IFD with proper offset handling
    fn get_raw_maker_note_ifd(&self) -> Result<Option<IfdInfo>>;

    /// Handle manufacturer-specific TIFF quirks
    fn apply_manufacturer_quirks(&mut self, make: &str) -> Result<()>;
}

impl RawTiffExtensions for TiffProcessor {
    fn get_raw_maker_note_ifd(&self) -> Result<Option<IfdInfo>> {
        // RAW files often have maker notes in non-standard locations
        // Handle various manufacturer patterns
        todo!()
    }

    fn apply_manufacturer_quirks(&mut self, make: &str) -> Result<()> {
        match make {
            "MINOLTA" => {
                // Minolta-specific TIFF handling
                self.set_byte_order_detection(ByteOrderDetection::MinoltaPattern);
            }
            "Panasonic" => {
                // Panasonic RW2 adjustments
                self.enable_entry_based_offsets();
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Phase 4: Testing and Validation (Week 2)

**Comprehensive Test Suite**:

```rust
// tests/raw/simple_tiff_tests.rs
#[test]
fn test_minolta_mrw_metadata() {
    let test_file = "test-images/minolta/sample.mrw";
    compare_with_exiftool(test_file, &[
        "Make", "Model", "ISO", "ShutterSpeed",
        "FNumber", "FocalLength", "WhiteBalance"
    ]);
}

#[test]
fn test_panasonic_rw2_metadata() {
    let test_file = "test-images/panasonic/sample.rw2";
    compare_with_exiftool(test_file, &[
        "Make", "Model", "ISO", "ExposureTime",
        "FNumber", "LensModel", "ColorMode"
    ]);
}

#[test]
fn test_panasonic_entry_based_offsets() {
    // Verify entry-based offset calculation works correctly
    let handler = PanasonicRawHandler::new();
    let test_entry = IfdEntry {
        tag: 0x002e,
        value_offset: 0x1000,
        count: 100,
        // ...
    };

    let offset = handler.calculate_entry_offset(&test_entry)?;
    assert_eq!(offset, 0x1000); // Relative to maker note start
}
```

## Success Criteria

### Core Requirements

- [ ] **Minolta MRW**: Complete support for MRW format metadata
- [ ] **Panasonic RW2**: Complete support for RW2 format metadata
- [ ] **Entry-Based Offsets**: Working implementation for Panasonic
- [ ] **CLI Integration**: Both formats work via CLI
- [ ] **Test Coverage**: Compatibility tests pass vs ExifTool

### Validation Tests

- [ ] Process sample MRW and RW2 files
- [ ] Extract all standard EXIF fields
- [ ] Extract manufacturer-specific fields
- [ ] Verify against `exiftool -j` output
- [ ] Handle corrupted/unusual files gracefully

## Implementation Boundaries

### Goals (Milestone 17b)

- Add two more TIFF-based RAW formats
- Introduce entry-based offset handling
- Improve shared TIFF infrastructure
- Maintain compatibility with existing code

### Non-Goals

- Complex offset management (save for Sony milestone)
- Preview extraction (Milestone 19)
- Non-TIFF formats (RAF, CRW)
- Write support

## Dependencies and Prerequisites

- Completed Milestone 17a (RAW foundation)
- Working ProcessBinaryData implementation
- Basic TIFF IFD navigation
- Test images for both formats

## Technical Notes

### MRW Format Structure

```
MRW File:
├── MRW Header
├── PRD Block (Picture Raw Data)
├── TTW Block (Thumbnail)
├── WBG Block (White Balance Gain)
└── Image Data
```

### RW2 Entry-Based Offsets

Panasonic stores some data at offsets specified in IFD entries:

```
IFD Entry 0x002e:
  Tag: 0x002e
  Type: LONG
  Count: 1
  Value: 0x1234  <- This value is an offset to actual data
```

### Trust ExifTool

Following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

- Copy exact offset calculations from PanasonicRaw.pm
- Preserve MRW block parsing order from MinoltaRaw.pm
- Don't "optimize" entry-based offset lookups

## Risk Mitigation

### Entry-Based Offset Complexity

- **Risk**: Entry-based offsets are new pattern
- **Mitigation**: Start simple, test thoroughly
- **Validation**: Compare calculated offsets with ExifTool debug output

### Format Documentation

- **Risk**: MRW/RW2 specs may be incomplete
- **Mitigation**: Rely entirely on ExifTool implementation
- **Reference**: Use ExifTool source as specification

## Next Steps

After successful completion:

1. Milestone 17c: Olympus ORF (medium complexity)
2. Milestone 17d: Canon CR2 (high complexity)
3. Build on entry-based offset pattern for Sony

## Summary

This milestone adds two more RAW formats with increasing complexity. Minolta provides a clean multi-block structure, while Panasonic introduces the important entry-based offset pattern that will be crucial for more complex formats like Sony.
