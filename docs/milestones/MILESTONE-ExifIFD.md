# Milestone: ExifIFD Compliance

**Duration**: 1-2 weeks  
**Goal**: Implement proper ExifIFD support per ExifTool specification for correct group assignment and context-aware processing

## Overview

ExifIFD (EXIF Image File Directory) is the core subdirectory within TIFF/EXIF structures containing camera-specific metadata. While exif-oxide currently recognizes ExifIFD as a subdirectory (tag 0x8769), it lacks proper ExifTool-compliant group assignment and context-aware processing, causing tags to be misidentified as coming from the main EXIF IFD rather than the ExifIFD subdirectory.

This milestone corrects the group assignment system to match ExifTool's three-level group hierarchy and ensures ExifIFD tags are properly distinguished from main IFD tags for API compatibility.

## Codebase Analysis Findings

**During ExifIFD research, analysis identified 30+ primitive lookup tables suitable for the simple table extraction framework. These represent 500+ manual lookup entries that could be automatically generated.**

### Manual Lookup Tables Found

**Print Conversion Tables** (`src/implementations/print_conv.rs`):

- 11 primitive tables with simple `key → string` mappings
- Examples: `orientation_print_conv`, `flash_print_conv`, `colorspace_print_conv`
- All follow pattern `match value { key => "string", ... }`

**Nikon Tables** (`src/implementations/nikon/tags.rs`):

- 18+ primitive HashMap constructions
- Examples: `nikon_quality_conv`, `nikon_af_area_mode_conv`
- All follow pattern `HashMap<i32, &str>`

**Canon Tables** (`src/implementations/canon/tags.rs`):

- Tag ID mapping table with 70+ entries
- Simple `u16 → String` pattern

### Simple Table Extraction Opportunity

These tables are excellent candidates for [MILESTONE-CODEGEN-SIMPLE-TABLES.md](MILESTONE-CODEGEN-SIMPLE-TABLES.md):

- All use primitive key-value mappings
- No complex Perl logic or conditionals
- Direct ExifTool source references available
- Would eliminate manual maintenance

## Background from ExifTool Analysis

From `third-party/exiftool/doc/concepts/EXIFIFD.md` analysis:

### ExifTool's ExifIFD Architecture

- **Group Assignment**: ExifIFD tags have `Group1 = "ExifIFD"` (not "EXIF")
- **Shared Table**: Uses same tag table (`Image::ExifTool::Exif::Main`) as main IFD but with different context
- **Same Processor**: `ProcessExif` function handles both main IFD and ExifIFD, but with context awareness
- **Write Group**: ExifIFD tags have default `WRITE_GROUP => 'ExifIFD'`
- **Offset Inheritance**: ExifIFD inherits base offset calculations from main IFD

### Current Implementation Problems

**1. Incorrect Group Assignment** (`src/exif/tags.rs:65`):

```rust
// WRONG - All ExifIFD tags assigned "EXIF" group
let namespace = match ifd_name {
    "ExifIFD" => "EXIF",  // Should be "ExifIFD"
    // ...
};
```

**2. Missing Group1 Field** (`src/types/metadata.rs:41-85`):

```rust
// TagEntry lacks group1 field for ExifTool's 3-level hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    pub group: String,     // Group0 only
    // Missing: pub group1: String,
    pub name: String,
    pub value: TagValue,
    pub print: String,
}
```

**3. Missing Context Awareness**: Processor correctly identifies ExifIFD subdirectories but doesn't maintain proper context during tag assignment.

**4. API Incompatibility**: Tags extracted from ExifIFD appear to come from main EXIF IFD, breaking ExifTool compatibility for group-based tag access.

## Implementation Strategy

### Phase 1: Group Assignment Correction (Week 1)

**Fix TagSourceInfo Generation**:

```rust
// src/exif/tags.rs - Updated create_tag_source_info
pub(crate) fn create_tag_source_info(&self, ifd_name: &str) -> TagSourceInfo {
    // ExifTool: lib/Image/ExifTool/Exif.pm group mappings
    let namespace = match ifd_name {
        "Root" | "IFD0" | "IFD1" => "EXIF",
        "GPS" => "EXIF",        // GPS tags belong to EXIF group
        "ExifIFD" => "ExifIFD", // FIX: Proper ExifIFD group assignment
        "InteropIFD" => "EXIF",
        "MakerNotes" => "MakerNotes",
        _ => "EXIF",
    };

    // ExifTool: Group hierarchy system
    let group1 = match ifd_name {
        "ExifIFD" => "ExifIFD",
        "GPS" => "GPS",
        "InteropIFD" => "InteropIFD",
        "MakerNotes" => "MakerNotes",
        _ => "IFD0", // Main IFD default
    };

    TagSourceInfo {
        namespace: namespace.to_string(),
        group1: group1.to_string(),
        processor_name: self.get_processor_name_for_ifd(ifd_name),
        directory_path: self.path.clone(),
    }
}
```

**Add Context Tracking**:

```rust
// src/exif/mod.rs - Enhanced context tracking
#[derive(Debug)]
pub struct IfdContext {
    pub name: String,
    pub group1: String,      // ExifTool Group1 assignment
    pub is_exif_ifd: bool,   // Special ExifIFD handling flag
    pub base_offset: u64,    // Inherited offset for subdirectories
}

impl ExifReader {
    fn enter_ifd_context(&mut self, ifd_name: &str, offset: u64) {
        let context = IfdContext {
            name: ifd_name.to_string(),
            group1: self.get_group1_for_ifd(ifd_name),
            is_exif_ifd: ifd_name == "ExifIFD",
            base_offset: offset,
        };

        self.ifd_context_stack.push(context);
    }

    fn get_group1_for_ifd(&self, ifd_name: &str) -> String {
        // ExifTool: Groups => { 1 => 'ExifIFD' } specification
        match ifd_name {
            "ExifIFD" => "ExifIFD".to_string(),
            "GPS" => "GPS".to_string(),
            "InteropIFD" => "InteropIFD".to_string(),
            "MakerNotes" => "MakerNotes".to_string(),
            _ => "IFD0".to_string(),
        }
    }
}
```

### Phase 2: Context-Aware Processing (Week 2)

**ExifIFD-Specific Validation**:

```rust
// src/exif/validation.rs - ExifIFD validation rules
impl ExifReader {
    fn validate_exif_ifd_entry(&self, tag_id: u16, value: &TagValue) -> Result<()> {
        // ExifTool: ExifIFD requires ExifVersion tag (0x9000)
        if self.current_ifd_context().is_exif_ifd {
            match tag_id {
                0x9000 => {
                    // ExifVersion is mandatory for valid ExifIFD
                    if let TagValue::String(version) = value {
                        if !version.starts_with("0") {
                            self.warnings.push(format!(
                                "Invalid ExifVersion: {}", version
                            ));
                        }
                    }
                }
                0xA000 => {
                    // FlashpixVersion validation
                    self.validate_flashpix_version(value)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
```

**Offset Management for ExifIFD**:

```rust
// src/exif/processors.rs - Enhanced subdirectory processing
pub(crate) fn process_subdirectory_tag(
    &mut self,
    tag_id: u16,
    offset: u32,
    tag_name: &str,
    size: Option<usize>,
) -> Result<()> {
    let subdir_name = match tag_id {
        0x8769 => "ExifIFD",
        0x8825 => "GPS",
        0xA005 => "InteropIFD",
        0x927C => "MakerNotes",
        _ => return Ok(()),
    };

    // ExifTool: ExifIFD inherits base offset from current context
    let inherited_base = if subdir_name == "ExifIFD" {
        self.base // ExifIFD uses main IFD base offset
    } else {
        self.base + offset as u64 // Other subdirs may have different schemes
    };

    // Create subdirectory context with proper offset inheritance
    let dir_info = DirectoryInfo {
        name: subdir_name.to_string(),
        dir_start: offset as usize,
        dir_len: size.unwrap_or(0),
        base: inherited_base,
        data_pos: 0,
        allow_reprocess: false,
    };

    // Enter ExifIFD context before processing
    self.enter_ifd_context(subdir_name, inherited_base);
    let result = self.process_subdirectory(&dir_info);
    self.exit_ifd_context();

    result
}
```

**API Group Access**:

```rust
// src/types/metadata.rs - Enhanced TagEntry with group hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    /// ExifTool Group0 (format family)
    pub group: String,

    /// ExifTool Group1 (subdirectory location) - NEW
    pub group1: String,

    /// Tag name
    pub name: String,

    /// Converted value (post-ValueConv)
    pub value: TagValue,

    /// Display string (post-PrintConv)
    pub print: String,
}

// Usage examples for group-based access
impl ExifData {
    /// Get all ExifIFD tags specifically
    pub fn get_exif_ifd_tags(&self) -> Vec<&TagEntry> {
        self.tags.iter()
            .filter(|tag| tag.group1 == "ExifIFD")
            .collect()
    }

    /// ExifTool compatibility: get tag by group-qualified name
    pub fn get_tag_by_group(&self, group_name: &str, tag_name: &str) -> Option<&TagEntry> {
        self.tags.iter()
            .find(|tag| tag.group1 == group_name && tag.name == tag_name)
    }
}
```

## Success Criteria

### Core Requirements

- [ ] **Group1 Assignment**: ExifIFD tags correctly assigned `group1 = "ExifIFD"`
- [ ] **Context Tracking**: Processor knows when processing ExifIFD vs main IFD
- [ ] **API Compatibility**: `TagEntry` includes both `group` and `group1` fields
- [ ] **Offset Inheritance**: ExifIFD inherits proper base offset from main IFD
- [ ] **Validation Rules**: ExifIFD-specific validation (ExifVersion requirement)

### Validation Tests

**Test Group Assignment**:

```rust
#[test]
fn test_exif_ifd_group_assignment() {
    let exif_data = process_file("t/images/Canon.jpg").unwrap();

    // Find a tag that should be in ExifIFD (like ExposureTime)
    let exposure_time = exif_data.get_tag_by_name("ExposureTime").unwrap();
    assert_eq!(exposure_time.group1, "ExifIFD");
    assert_eq!(exposure_time.group, "EXIF");

    // Verify main IFD tags are still assigned correctly
    let image_width = exif_data.get_tag_by_name("ImageWidth").unwrap();
    assert_eq!(image_width.group1, "IFD0");
}
```

**Test Context-Aware Processing**:

```rust
#[test]
fn test_exif_ifd_context_awareness() {
    let exif_data = process_file("t/images/Canon-ExifIFD.jpg").unwrap();

    // Verify ExifIFD-specific validation occurred
    let exif_version = exif_data.get_tag_by_group("ExifIFD", "ExifVersion").unwrap();
    assert!(exif_version.value.as_string().is_some());

    // Verify no warnings about missing ExifVersion
    assert!(exif_data.warnings.iter()
        .find(|w| w.contains("ExifVersion"))
        .is_none());
}
```

**Test ExifTool Compatibility**:

```rust
#[test]
fn test_exiftool_compatibility() {
    let exif_data = process_file("t/images/Nikon-D70.jpg").unwrap();

    // Should be able to access ExifIFD tags by group
    let exif_ifd_tags = exif_data.get_exif_ifd_tags();
    assert!(!exif_ifd_tags.is_empty());

    // Verify mainstream ExifIFD tags are present
    let required_tags = ["ExposureTime", "FNumber", "ISO", "ExifVersion"];
    for tag_name in required_tags {
        assert!(exif_ifd_tags.iter()
            .any(|tag| tag.name == tag_name),
            "Missing ExifIFD tag: {}", tag_name);
    }
}
```

## Implementation Boundaries

### Goals (This Milestone)

- Correct ExifIFD group assignment (`group1 = "ExifIFD"`)
- Context-aware IFD processing with stack management
- API enhancement for group-based tag access
- ExifIFD-specific validation rules
- Proper offset inheritance for subdirectories

### Non-Goals (Future Work)

- **Write Group Management**: ExifIFD write support (Milestone 21)
- **Advanced Group Features**: Group2 assignments, complex group hierarchies
- **Performance Optimization**: Group-based indexing or lookup acceleration
- **Extended Validation**: Full EXIF 2.32/3.0 specification compliance

## Dependencies and Integration

### Prerequisites

- None - builds on existing IFD processing infrastructure
- **Note**: Analysis identified 30+ lookup tables suitable for simple table extraction (see above)

### Enables Future Work

- **Milestone 21 (Write Support)**: Proper group assignment essential for correct write operations
- **API Consumers**: Group-based tag filtering and access patterns
- **Debugging Tools**: Better identification of tag source locations

### Integration Points

```rust
// ExifTool compatibility layer
impl ExifData {
    /// ExifTool-style group access: EXIF:ExposureTime vs ExifIFD:ExposureTime
    pub fn get_tag_exiftool_style(&self, qualified_name: &str) -> Option<&TagEntry> {
        if let Some((group, name)) = qualified_name.split_once(':') {
            self.get_tag_by_group(group, name)
        } else {
            self.get_tag_by_name(qualified_name)
        }
    }
}

// Example usage
let exposure_time = exif_data.get_tag_exiftool_style("ExifIFD:ExposureTime");
let gps_lat = exif_data.get_tag_exiftool_style("GPS:GPSLatitude");
```

## Risk Mitigation

### Breaking API Changes

- **Risk**: Adding `group1` field to `TagEntry` breaks existing consumers
- **Mitigation**: Mark as non-breaking addition, provide backwards compatibility methods

### Performance Impact

- **Risk**: Context stack management adds overhead
- **Mitigation**: Lightweight context objects, stack only used during processing

### ExifTool Compatibility

- **Risk**: Subtle differences in group assignment vs ExifTool behavior
- **Mitigation**: Comprehensive test suite against ExifTool's test images with exact output comparison

## Testing Strategy

### Unit Tests

- Group assignment logic for all IFD types
- Context stack push/pop behavior
- Offset inheritance calculations
- ExifIFD-specific validation rules

### Integration Tests

- Process real camera files with ExifIFD content
- Compare group assignments with ExifTool output
- Verify API access patterns work as expected

### Regression Tests

- Ensure existing functionality unchanged
- Verify main IFD processing still works correctly
- Check that non-ExifIFD subdirectories unaffected

## Related Documentation

### Required Reading

**ExifIFD Implementation**:

- [EXIFIFD.md](../../third-party/exiftool/doc/concepts/EXIFIFD.md) - Complete ExifIFD specification
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System overview and principles
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Implementation principles

**Related Codegen Opportunity**:

- [MILESTONE-CODEGEN-SIMPLE-TABLES.md](MILESTONE-CODEGEN-SIMPLE-TABLES.md) - Framework for converting manual lookup tables
- [CODEGEN.md](../design/CODEGEN.md) - Simple table extraction details

### ExifTool Source References

- **lib/Image/ExifTool/Exif.pm:3006-3013**: ExifIFD tag definition with group assignment
- **lib/Image/ExifTool/Exif.pm:6174-7128**: ProcessExif function that handles both main IFD and ExifIFD
- **lib/Image/ExifTool/Exif.pm:850**: WRITE_GROUP default assignment for ExifIFD

### Implementation Files Modified

**Core ExifIFD Changes**:

- `src/exif/tags.rs:65` - Fix group assignment bug
- `src/exif/processors.rs:228-234` - Context-aware subdirectory processing
- `src/types/metadata.rs:41-85` - Add `group1` field to TagEntry
- `src/exif/ifd.rs` - Context tracking during IFD processing

**Potential Codegen Candidates Found**:

- `src/implementations/print_conv.rs` - 11 primitive lookup tables
- `src/implementations/nikon/tags.rs` - 18+ primitive lookup tables
- `src/implementations/canon/tags.rs` - Tag mapping table

This milestone fixes a foundational issue in EXIF processing that affects all downstream consumers. Proper ExifIFD support ensures tags are correctly identified by their source location, enabling ExifTool-compatible group-based access patterns and maintaining API compatibility for advanced metadata workflows.

**Additional Context**: Research for this milestone identified significant opportunities for simple table extraction. Consider reviewing the discovered lookup tables for potential codegen conversion either before or after ExifIFD implementation.
