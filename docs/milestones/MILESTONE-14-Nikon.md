# Milestone 14: Second Manufacturer - Nikon

**Duration**: 4 weeks  
**Goal**: Prove architecture with encrypted maker notes and complex manufacturer implementation

## Overview

Nikon represents the most sophisticated manufacturer implementation in ExifTool, featuring advanced encryption, comprehensive lens databases, and model-specific processing. This milestone establishes the patterns for handling complex manufacturer implementations while proving our architecture can scale beyond Canon's relatively straightforward design.

## Background

From ExifTool's Nikon.pm analysis:

- **14,191 lines** vs Canon's 10,639 lines (33% larger)
- **135 tag tables** vs Canon's 107 (26% more complexity)
- **Advanced encryption system** using serial number and shutter count as keys
- **618 lens database entries** vs Canon's ~400 (55% larger database)
- **Multiple format versions** for maker note structure
- **Model-specific subdirectories** for each camera generation
- **Complex AF grid systems** vs Canon's point-based approach

## Key Concepts

### Nikon Maker Note Format Detection

```perl
# ExifTool: MakerNotes.pm:152-163
# Multiple signature patterns for different Nikon generations
if ($make =~ /^NIKON CORPORATION$/i) {
    # Modern format with TIFF header at offset 10
    if (substr($val, 0, 4) eq "\x02\x10\x00\x00") {
        return "NikonFormat3";
    }
}
```

### Encryption Key System

```perl
# ExifTool: Nikon.pm:1234-1267
# Pre-scan phase to extract encryption keys
if ($tagID == 0x001d) {        # SerialNumber
    $$et{NikonSerialKey} = $val;
} elsif ($tagID == 0x00a7) {   # ShutterCount  
    $$et{NikonCountKey} = $val;
}
```

### Offset Scheme Complexity

```perl
# ExifTool: Nikon.pm:890-934
# Multiple offset schemes based on format version
if ($format eq 'Format3') {
    $base = $dataPos + 0x0a;  # TIFF header at 0x0a
} elsif ($format eq 'Format2') {
    $base = $dataPos + 0x08;  # Different base offset
}
```

## Implementation Phases

### Phase 1: Foundation Infrastructure (Week 1)

**Goal**: Establish basic Nikon detection and module structure

#### 1.1 Module Structure Setup

Following Canon's proven pattern:

```
src/implementations/nikon/
├── mod.rs                    # Main coordinator module  
├── detection.rs              # Multi-format maker note detection
├── offset_schemes.rs         # Format-specific offset calculations
├── encryption.rs             # Encryption key management (skeleton)
├── tags.rs                   # Primary tag ID mappings
├── lens_database.rs          # Lens ID lookup tables
└── tests.rs                  # Unit tests
```

#### 1.2 Format Detection System

```rust
// src/implementations/nikon/detection.rs

#[derive(Debug, Clone, PartialEq)]
pub enum NikonFormat {
    Format1,    // Early Nikon format
    Format2,    // Mid-generation format  
    Format3,    // Modern format with TIFF header
}

pub fn detect_nikon_format(data: &[u8]) -> Option<NikonFormat> {
    // ExifTool: MakerNotes.pm:152-163
    if data.len() >= 4 {
        match &data[0..4] {
            b"\x02\x10\x00\x00" => Some(NikonFormat::Format3),
            b"\x02\x00\x00\x00" => Some(NikonFormat::Format2),
            _ => Some(NikonFormat::Format1), // Default fallback
        }
    } else {
        None
    }
}

pub fn detect_nikon_signature(make: &str) -> bool {
    // ExifTool: MakerNotes.pm:152
    make == "NIKON CORPORATION" || make == "NIKON"
}
```

#### 1.3 Offset Scheme Implementation

```rust
// src/implementations/nikon/offset_schemes.rs

pub fn calculate_nikon_base_offset(
    format: NikonFormat,
    data_pos: usize,
) -> usize {
    match format {
        // ExifTool: Nikon.pm:890-934
        NikonFormat::Format3 => data_pos + 0x0a,  // TIFF header at 0x0a
        NikonFormat::Format2 => data_pos + 0x08,  // Mid-generation offset
        NikonFormat::Format1 => data_pos + 0x06,  // Early format offset
    }
}
```

### Phase 2: Encryption System Foundation (Week 2)

**Goal**: Implement encryption key management and ProcessNikonEncrypted skeleton

#### 2.1 Encryption Key Management

```rust
// src/implementations/nikon/encryption.rs

#[derive(Debug, Clone)]
pub struct NikonEncryptionKeys {
    pub serial_number: Option<String>,
    pub shutter_count: Option<u32>,
    pub camera_model: String,
}

impl NikonEncryptionKeys {
    pub fn new(model: String) -> Self {
        Self {
            serial_number: None,
            shutter_count: None,
            camera_model: model,
        }
    }

    pub fn store_serial_key(&mut self, serial: String) {
        // ExifTool: Nikon.pm:1234
        self.serial_number = Some(serial);
    }

    pub fn store_count_key(&mut self, count: u32) {
        // ExifTool: Nikon.pm:1267
        self.shutter_count = Some(count);
    }

    pub fn has_required_keys(&self) -> bool {
        // Most Nikon encryption requires both keys
        self.serial_number.is_some() && self.shutter_count.is_some()
    }
}
```

#### 2.2 ProcessNikonEncrypted Skeleton

```rust
// src/implementations/nikon/encryption.rs

pub fn process_nikon_encrypted(
    reader: &mut ExifReader,
    data: &[u8],
    keys: &NikonEncryptionKeys,
) -> Result<()> {
    // ExifTool: Nikon.pm:ProcessNikonEncrypted
    
    if !keys.has_required_keys() {
        // Log encryption detection without keys
        reader.add_tag(
            "Nikon:EncryptedData", 
            TagValue::String("Encrypted (keys required)".to_string())
        );
        return Ok(());
    }

    // TODO: Implement actual decryption in future milestone
    // For now, detect and log encrypted sections
    reader.add_tag(
        "Nikon:EncryptedData",
        TagValue::String("Encrypted (decryption not implemented)".to_string())
    );
    
    Ok(())
}
```

#### 2.3 Key Extraction Pre-scan

```rust
// src/implementations/nikon/mod.rs

pub fn process_nikon_makernotes(
    reader: &mut ExifReader,
    data: &[u8],
    _offset: usize,
) -> Result<()> {
    let mut encryption_keys = NikonEncryptionKeys::new(
        reader.get_tag_value("Make")
            .unwrap_or("Unknown Nikon".to_string())
    );

    // Phase 1: Pre-scan for encryption keys
    // ExifTool: Nikon.pm:1234-1267
    prescan_for_keys(data, &mut encryption_keys)?;

    // Phase 2: Process standard tags
    process_standard_tags(reader, data, &encryption_keys)?;

    // Phase 3: Process encrypted sections (skeleton)
    process_encrypted_sections(reader, data, &encryption_keys)?;

    Ok(())
}

fn prescan_for_keys(
    data: &[u8], 
    keys: &mut NikonEncryptionKeys
) -> Result<()> {
    // Scan for tag 0x001d (SerialNumber) and 0x00a7 (ShutterCount)
    // ExifTool: Nikon.pm pre-scan logic
    // TODO: Implement EXIF directory scanning for key extraction
    Ok(())
}
```

### Phase 3: Core Tag Processing & PrintConv (Week 3-3.5)

**Goal**: Implement mainstream Nikon tag processing and conversion functions

#### 3.1 Lens Database Implementation

```rust
// src/implementations/nikon/lens_database.rs

#[derive(Debug, Clone)]
pub struct NikonLensEntry {
    pub id_pattern: String,  // "50 1 0C 00 02 00 14 02"
    pub description: String, // "AF-S DX Nikkor 18-55mm f/3.5-5.6G VR"
}

lazy_static! {
    // ExifTool: Nikon.pm %nikonLensIDs - 618 entries
    static ref NIKON_LENS_DATABASE: Vec<NikonLensEntry> = vec![
        NikonLensEntry {
            id_pattern: "06 00 00 07 00 00 00 01".to_string(),
            description: "AF Nikkor 50mm f/1.8".to_string(),
        },
        // ... 618 total entries from ExifTool extraction
    ];
}

pub fn lookup_nikon_lens(lens_data: &[u8]) -> Option<String> {
    if lens_data.len() < 8 {
        return None;
    }

    // ExifTool: Nikon.pm LensIDConv function
    let id = format!("{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        lens_data[0], lens_data[1], lens_data[2], lens_data[3],
        lens_data[4], lens_data[5], lens_data[6], lens_data[7]);

    NIKON_LENS_DATABASE.iter()
        .find(|entry| entry.id_pattern == id)
        .map(|entry| entry.description.clone())
}
```

#### 3.2 Model-Specific Tag Tables

```rust
// src/implementations/nikon/tags.rs

pub struct NikonTagTable {
    pub name: &'static str,
    pub model_condition: Option<&'static str>,
    pub tags: &'static [(u16, &'static str, Option<PrintConvFunc>)],
}

// ExifTool: Nikon.pm model-specific tables
pub static NIKON_Z9_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoZ9",
    model_condition: Some("NIKON Z 9"),
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        // ... Z9-specific tags
    ],
};

pub static NIKON_MAIN_TAGS: NikonTagTable = NikonTagTable {
    name: "Nikon::Main", 
    model_condition: None,
    tags: &[
        (0x0001, "MakerNoteVersion", None),
        (0x0002, "ISO", Some(nikon_iso_conv)),
        (0x0003, "ColorMode", Some(nikon_color_mode_conv)),
        (0x0004, "Quality", Some(nikon_quality_conv)),
        (0x0005, "WhiteBalance", Some(nikon_white_balance_conv)),
        // ... 200+ mainstream tags
    ],
};
```

#### 3.3 PrintConv Implementation

```rust
// src/implementations/nikon/print_conv.rs

pub fn nikon_quality_conv(value: &TagValue) -> Result<String> {
    // ExifTool: Nikon.pm Quality PrintConv
    let quality_map: HashMap<i32, &str> = [
        (1, "VGA Basic"),
        (2, "VGA Normal"), 
        (3, "VGA Fine"),
        (4, "SXGA Basic"),
        (5, "SXGA Normal"),
        (6, "SXGA Fine"),
        // ... complete mapping from ExifTool
    ].iter().cloned().collect();

    if let Some(val) = value.as_i32() {
        Ok(quality_map.get(&val)
            .unwrap_or(&"Unknown")
            .to_string())
    } else {
        Ok(format!("Unknown ({})", value))
    }
}

pub fn nikon_white_balance_conv(value: &TagValue) -> Result<String> {
    // ExifTool: Nikon.pm WhiteBalance PrintConv
    let wb_map: HashMap<i32, &str> = [
        (0, "Auto"),
        (1, "Preset"),
        (2, "Daylight"),
        (3, "Incandescent"),
        (4, "Fluorescent"),
        (5, "Cloudy"),
        (6, "Speedlight"),
        // ... complete WB mapping
    ].iter().cloned().collect();

    // Similar implementation pattern
    if let Some(val) = value.as_i32() {
        Ok(wb_map.get(&val)
            .unwrap_or(&"Unknown")
            .to_string())
    } else {
        Ok(format!("Unknown ({})", value))
    }
}
```

#### 3.4 AF System Processing

```rust
// src/implementations/nikon/af_processing.rs

pub fn process_nikon_af_info(
    reader: &mut ExifReader,
    data: &[u8],
) -> Result<()> {
    // ExifTool: Nikon.pm AFInfo processing
    
    if data.len() < 4 {
        return Err(ExifError::insufficient_data("AFInfo", 4, data.len()));
    }

    // AF Info version detection
    let version = u16::from_be_bytes([data[0], data[1]]);
    reader.add_tag("Nikon:AFInfoVersion", TagValue::Integer(version as i64));

    match version {
        0x0100 => process_af_info_v0100(reader, data),
        0x0102 => process_af_info_v0102(reader, data), 
        0x0103 => process_af_info_v0103(reader, data),
        _ => {
            reader.add_tag(
                "Nikon:AFInfo",
                TagValue::String(format!("Unknown version 0x{:04x}", version))
            );
            Ok(())
        }
    }
}

fn process_af_info_v0100(
    reader: &mut ExifReader,
    data: &[u8],
) -> Result<()> {
    // ExifTool: Nikon.pm AFInfo version 0100 processing
    // Handle legacy AF point mapping
    Ok(())
}
```

### Phase 4: Testing & Polish (Week 4)

**Goal**: Comprehensive testing and integration validation

#### 4.1 Test Strategy

```rust
// src/implementations/nikon/tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nikon_format_detection() {
        // Test Format3 detection
        let format3_data = b"\x02\x10\x00\x00extra_data";
        assert_eq!(
            detect_nikon_format(format3_data),
            Some(NikonFormat::Format3)
        );

        // Test Format2 detection  
        let format2_data = b"\x02\x00\x00\x00extra_data";
        assert_eq!(
            detect_nikon_format(format2_data),
            Some(NikonFormat::Format2)
        );
    }

    #[test]
    fn test_nikon_signature_detection() {
        assert!(detect_nikon_signature("NIKON CORPORATION"));
        assert!(detect_nikon_signature("NIKON"));
        assert!(!detect_nikon_signature("Canon"));
    }

    #[test]
    fn test_offset_calculation() {
        let base = calculate_nikon_base_offset(NikonFormat::Format3, 100);
        assert_eq!(base, 110); // 100 + 0x0a
    }

    #[test]
    fn test_lens_database_lookup() {
        let lens_data = [0x06, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x01];
        let result = lookup_nikon_lens(&lens_data);
        assert!(result.is_some());
        assert!(result.unwrap().contains("50mm"));
    }

    #[test]
    fn test_encryption_key_management() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        assert!(!keys.has_required_keys());

        keys.store_serial_key("12345".to_string());
        assert!(!keys.has_required_keys());

        keys.store_count_key(1000);
        assert!(keys.has_required_keys());
    }
}
```

#### 4.2 Integration Testing

```rust
// tests/integration/nikon_integration.rs

#[test]
fn test_nikon_d850_integration() {
    // Test with real Nikon D850 file
    let test_file = "tests/fixtures/nikon_d850_sample.nef";
    let reader = ExifReader::from_file(test_file).unwrap();
    
    // Verify basic Nikon detection
    assert_eq!(reader.get_tag("Make").unwrap(), "NIKON CORPORATION");
    
    // Verify format detection worked
    assert!(reader.has_tag("Nikon:MakerNoteVersion"));
    
    // Check encryption detection
    if reader.has_tag("Nikon:EncryptedData") {
        // Encryption detected but not decrypted (expected in this milestone)
        let encrypted_value = reader.get_tag("Nikon:EncryptedData").unwrap();
        assert!(encrypted_value.contains("Encrypted"));
    }
}
```

## Success Criteria

- [ ] **Format Detection**: All three Nikon maker note formats correctly identified
- [ ] **Signature Detection**: Proper detection of "NIKON CORPORATION" and "NIKON" signatures  
- [ ] **Offset Calculation**: Correct base offset calculation for each format version
- [ ] **Encryption Detection**: Encrypted sections identified (keys detected if present)
- [ ] **Basic Tag Extraction**: 50+ mainstream Nikon tags extracted with raw values
- [ ] **PrintConv Functions**: 20+ conversion functions for common tags (Quality, WhiteBalance, etc.)
- [ ] **Lens Database**: 618-entry lens lookup functional for mainstream lenses
- [ ] **Model Detection**: Model-specific table selection working for 5+ camera models
- [ ] **Test Coverage**: 95%+ code coverage with comprehensive unit tests
- [ ] **Integration Validation**: Real Nikon files processed without panicking

## Testing Strategy

### 1. Unit Testing
- Format detection with various data patterns
- Offset calculation accuracy
- Encryption key management
- Lens database lookups
- PrintConv function accuracy

### 2. Integration Testing  
- Real Nikon NEF files from multiple camera models
- Different maker note format versions
- Files with and without encryption
- Boundary conditions and malformed data

### 3. Compatibility Testing
- Compare output with ExifTool for same files
- Validate mainstream tag extraction
- Verify graceful handling of encrypted data

### 4. Performance Testing
- Large lens database lookup performance
- Multi-format processing overhead
- Memory usage with complex tag structures

## Implementation Dependencies

### ExifTool Source References
- **Nikon.pm** (14,191 lines) - Primary module implementation
- **MakerNotes.pm** - Format detection logic
- **PrintConv tables** - 135 tag table definitions

### Project Dependencies
- Encryption system relies on successful key extraction
- Lens database requires mainstream tag filtering
- Model-specific tables need camera detection logic
- PrintConv functions depend on tag value parsing

## Risk Mitigation

### Encryption Complexity
- **Risk**: Encryption implementation is extremely complex
- **Mitigation**: Phase 2 implements skeleton with detection only. Actual decryption deferred to future milestone.

### Database Size
- **Risk**: 618-entry lens database may impact performance
- **Mitigation**: Use lazy_static for one-time initialization and efficient HashMap lookups.

### Format Variations
- **Risk**: Three different maker note formats increase complexity
- **Mitigation**: Clear separation of format detection and processing logic.

### Model-Specific Tables  
- **Risk**: 30+ model-specific tables may cause code explosion
- **Mitigation**: Use conditional table selection and shared processing functions.

## Future Milestone Dependencies

### Milestone 15: Advanced Nikon Encryption
- Full ProcessNikonEncrypted implementation
- Serial number and shutter count decryption algorithms
- Model-specific encryption variations
- Write support with re-encryption

### Milestone 16: Complete Nikon Coverage
- Remaining non-mainstream tags
- Video metadata processing (AVI/MOV)
- Nikon Capture NX edit history
- Advanced AF grid processing

## Related Documentation

- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Critical implementation principles
- [Nikon.md](../../third-party/exiftool/doc/modules/Nikon.md) - Complete Nikon module analysis
- [ENCRYPTION.md](../../third-party/exiftool/doc/concepts/ENCRYPTION.md) - ExifTool encryption concepts
- [LENS_DATABASE.md](../../third-party/exiftool/doc/concepts/LENS_DATABASE.md) - Lens identification system
- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - Managing encryption keys and model state
- [IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md) - Manual implementation patterns

This milestone establishes Nikon as the second major manufacturer while proving our architecture can handle significantly more complex implementations than Canon. The encryption system foundation will enable future milestones to add full decryption capabilities.