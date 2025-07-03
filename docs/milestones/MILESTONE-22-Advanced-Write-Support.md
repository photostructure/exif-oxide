# Milestone 22: Advanced Write Support

**Duration**: 3-4 weeks  
**Goal**: Implement complex metadata writing including MakerNote preservation and multi-format operations

## Overview

Advanced write support extends the basic write foundation to handle the most challenging aspects of metadata modification: preserving manufacturer-specific data, complex format synchronization, and specialized binary data handling. This milestone makes exif-oxide capable of professional metadata workflows.

## Background: Advanced Write Complexity

**MakerNote preservation is ExifTool's most complex write operation**:
- **Manufacturer-specific formats**: Each brand has unique binary structures
- **Offset fixup systems**: Complex pointer recalculation when data moves
- **Encryption handling**: Some manufacturers encrypt sections (Nikon, Pentax)
- **Binary preservation**: Must maintain exact byte sequences for compatibility

**Key Challenge**: Modifying metadata while preserving unknown binary data requires sophisticated offset management and format-specific knowledge.

## Implementation Strategy

### Phase 1: MakerNote Preservation Infrastructure (Week 1)

**MakerNote Handler System**:
```rust
pub struct MakerNoteHandler {
    preservation_strategies: HashMap<String, Box<dyn MakerNotePreserver>>,
    offset_fixup_engine: OffsetFixupEngine,
    encryption_detector: EncryptionDetector,
}

pub trait MakerNotePreserver: Send + Sync {
    fn can_preserve(&self, make: &str, model: &str, maker_note_data: &[u8]) -> bool;
    fn preserve_maker_note(&self, 
        original_data: &[u8], 
        changes: &[TagChange], 
        context: &PreservationContext
    ) -> Result<PreservedMakerNote>;
    fn requires_offset_fixup(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct PreservedMakerNote {
    pub data: Vec<u8>,
    pub offset_map: HashMap<usize, usize>, // old_offset -> new_offset
    pub preserved_sections: Vec<PreservedSection>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreservedSection {
    pub name: String,
    pub original_offset: usize,
    pub new_offset: usize,
    pub size: usize,
    pub preservation_method: PreservationMethod,
}

#[derive(Debug, Clone)]
pub enum PreservationMethod {
    ExactCopy,           // Binary copy with no modifications
    OffsetFixup,         // Copy with pointer adjustments
    EncryptedPreserve,   // Maintain encryption, warn user
    Reconstructed,       // Rebuilt from tag values
}
```

**Offset Fixup Engine**:
```rust
pub struct OffsetFixupEngine {
    pointer_tables: HashMap<String, PointerTable>, // Per-manufacturer pointer locations
    fixup_strategies: Vec<Box<dyn OffsetFixupStrategy>>,
}

pub trait OffsetFixupStrategy: Send + Sync {
    fn detect_pointers(&self, data: &[u8], manufacturer: &str) -> Vec<PointerLocation>;
    fn fixup_pointer(&self, data: &mut [u8], location: &PointerLocation, offset_delta: i64) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PointerLocation {
    pub offset_in_data: usize,
    pub pointer_size: usize,    // 2, 4, or 8 bytes
    pub base_offset: usize,     // What the pointer is relative to
    pub endianness: ByteOrder,
}

impl OffsetFixupEngine {
    // ExifTool's complex offset fixup logic
    pub fn fixup_offsets(&mut self, 
        manufacturer: &str, 
        original_data: &[u8], 
        new_data: &mut [u8],
        data_shift: i64
    ) -> Result<Vec<String>> {
        
        let mut warnings = Vec::new();
        let pointer_locations = self.detect_manufacturer_pointers(manufacturer, original_data)?;
        
        for location in pointer_locations {
            match self.fixup_single_pointer(new_data, &location, data_shift) {
                Ok(()) => {},
                Err(e) => {
                    warnings.push(format!("Failed to fix offset at 0x{:x}: {}", location.offset_in_data, e));
                }
            }
        }
        
        Ok(warnings)
    }
}
```

### Phase 2: Canon MakerNote Preservation (Week 1-2)

**Canon MakerNote Handler**:
```rust
pub struct CanonMakerNotePreserver;

impl MakerNotePreserver for CanonMakerNotePreserver {
    fn preserve_maker_note(&self, 
        original_data: &[u8], 
        changes: &[TagChange], 
        context: &PreservationContext
    ) -> Result<PreservedMakerNote> {
        
        // Canon maker notes use standard TIFF structure
        // Can modify known tags while preserving unknown sections
        
        let mut preserved = PreservedMakerNote {
            data: Vec::new(),
            offset_map: HashMap::new(),
            preserved_sections: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Parse Canon TIFF structure
        let ifd_parser = TiffIfdParser::new(original_data, ByteOrder::LittleEndian)?;
        let ifds = ifd_parser.parse_all_ifds()?;
        
        // Rebuild IFDs with changes
        for ifd in ifds {
            let rebuilt_ifd = self.rebuild_canon_ifd(&ifd, changes, &mut preserved)?;
            preserved.data.extend(rebuilt_ifd);
        }
        
        // Preserve unknown binary sections
        self.preserve_canon_binary_sections(original_data, &mut preserved)?;
        
        Ok(preserved)
    }
    
    fn requires_offset_fixup(&self) -> bool {
        true // Canon maker notes contain pointers that need adjustment
    }
}

impl CanonMakerNotePreserver {
    fn rebuild_canon_ifd(&self, 
        original_ifd: &TiffIfd, 
        changes: &[TagChange], 
        preserved: &mut PreservedMakerNote
    ) -> Result<Vec<u8>> {
        
        let mut new_ifd_data = Vec::new();
        let mut entries = Vec::new();
        
        // Process each IFD entry
        for entry in &original_ifd.entries {
            let tag_name = self.canon_tag_id_to_name(entry.tag_id);
            
            if let Some(change) = changes.iter().find(|c| c.tag_name == tag_name) {
                if let Some(new_value) = &change.new_value {
                    // Modified tag - rebuild entry with new value
                    let new_entry = self.build_canon_tag_entry(entry.tag_id, new_value)?;
                    entries.push(new_entry);
                    
                    preserved.preserved_sections.push(PreservedSection {
                        name: tag_name.clone(),
                        original_offset: entry.value_offset,
                        new_offset: new_ifd_data.len(),
                        size: new_entry.data.len(),
                        preservation_method: PreservationMethod::Reconstructed,
                    });
                }
                // If new_value is None, tag is deleted (omitted)
            } else {
                // Unchanged tag - preserve exactly
                entries.push(entry.clone());
                
                preserved.preserved_sections.push(PreservedSection {
                    name: tag_name,
                    original_offset: entry.value_offset,
                    new_offset: new_ifd_data.len(),
                    size: entry.data.len(),
                    preservation_method: PreservationMethod::ExactCopy,
                });
            }
        }
        
        // Build new IFD structure
        new_ifd_data.extend(&(entries.len() as u16).to_le_bytes());
        for entry in entries {
            new_ifd_data.extend(entry.to_bytes());
        }
        new_ifd_data.extend(&0u32.to_le_bytes()); // Next IFD = 0
        
        Ok(new_ifd_data)
    }
}
```

### Phase 3: Multi-Format Write Coordination (Week 2-3)

**Cross-Format Synchronization**:
```rust
pub struct MultiFormatWriter {
    format_writers: HashMap<FormatType, Box<dyn FormatWriter>>,
    sync_rules: Vec<Box<dyn SyncRule>>,
    write_order: Vec<FormatType>,
}

pub trait SyncRule: Send + Sync {
    fn get_affected_formats(&self) -> Vec<FormatType>;
    fn synchronize_change(&self, change: &TagChange, target_format: FormatType) -> Vec<TagChange>;
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FormatType {
    EXIF,
    XMP,
    IPTC,
    MakerNotes,
}

impl MultiFormatWriter {
    pub fn write_synchronized_changes(&mut self, 
        changes: &[TagChange], 
        reader: &ExifReader
    ) -> Result<MultiFormatResult> {
        
        // Generate synchronized changes for all formats
        let mut all_changes = self.generate_synchronized_changes(changes)?;
        
        // Write in dependency order (EXIF first, then XMP, then IPTC)
        let mut results = HashMap::new();
        
        for format in &self.write_order {
            if let Some(format_changes) = all_changes.remove(format) {
                let writer = self.format_writers.get_mut(format)
                    .ok_or_else(|| WriteError::UnsupportedFormat(*format))?;
                
                let result = writer.write_tags(&format_changes, reader)?;
                results.insert(*format, result);
            }
        }
        
        Ok(MultiFormatResult { format_results: results })
    }
    
    fn generate_synchronized_changes(&self, changes: &[TagChange]) -> Result<HashMap<FormatType, Vec<TagChange>>> {
        let mut format_changes: HashMap<FormatType, Vec<TagChange>> = HashMap::new();
        
        for change in changes {
            // Determine primary format for this tag
            let primary_format = self.determine_primary_format(&change.tag_name)?;
            format_changes.entry(primary_format).or_default().push(change.clone());
            
            // Apply synchronization rules
            for sync_rule in &self.sync_rules {
                if sync_rule.applies_to_tag(&change.tag_name) {
                    for target_format in sync_rule.get_affected_formats() {
                        if target_format != primary_format {
                            let synced_changes = sync_rule.synchronize_change(change, target_format);
                            format_changes.entry(target_format).or_default().extend(synced_changes);
                        }
                    }
                }
            }
        }
        
        Ok(format_changes)
    }
}

// Example synchronization rule
pub struct DateTimeSyncRule;
impl SyncRule for DateTimeSyncRule {
    fn get_affected_formats(&self) -> Vec<FormatType> {
        vec![FormatType::EXIF, FormatType::XMP, FormatType::IPTC]
    }
    
    fn synchronize_change(&self, change: &TagChange, target_format: FormatType) -> Vec<TagChange> {
        match (&change.tag_name.as_str(), target_format) {
            ("DateTime", FormatType::XMP) => {
                vec![TagChange {
                    tag_name: "XMP:ModifyDate".to_string(),
                    old_value: None,
                    new_value: change.new_value.clone(),
                    format: WriteFormat::XMP,
                    validation_passed: true,
                }]
            },
            ("DateTimeOriginal", FormatType::XMP) => {
                vec![TagChange {
                    tag_name: "XMP:DateTimeOriginal".to_string(),
                    old_value: None,
                    new_value: change.new_value.clone(),
                    format: WriteFormat::XMP,
                    validation_passed: true,
                }]
            },
            _ => vec![]
        }
    }
}
```

### Phase 4: Specialized Write Operations (Week 3-4)

**Thumbnail Handling**:
```rust
pub struct ThumbnailWriter;

impl ThumbnailWriter {
    pub fn write_thumbnail(&mut self, 
        thumbnail_data: &[u8], 
        writer: &mut MetadataWriter
    ) -> Result<()> {
        
        // Validate thumbnail is valid JPEG
        if !thumbnail_data.starts_with(&[0xff, 0xd8, 0xff]) {
            return Err(WriteError::InvalidThumbnail("Not a valid JPEG".to_string()));
        }
        
        // Set thumbnail-related tags
        writer.set_tag_value("ThumbnailImage", TagValue::Binary(thumbnail_data.to_vec()))?;
        writer.set_tag_value("ThumbnailOffset", TagValue::Integer(0))?; // Will be calculated during write
        writer.set_tag_value("ThumbnailLength", TagValue::Integer(thumbnail_data.len() as i64))?;
        writer.set_tag_value("ThumbnailCompression", TagValue::Integer(6))?; // JPEG compression
        
        Ok(())
    }
    
    pub fn generate_thumbnail_from_preview(&self, preview_data: &[u8], max_size: (u32, u32)) -> Result<Vec<u8>> {
        // Simple thumbnail generation (in practice, would use image processing library)
        // For now, just validate the preview is JPEG and return it if small enough
        
        if !preview_data.starts_with(&[0xff, 0xd8, 0xff]) {
            return Err(WriteError::InvalidPreview("Not a valid JPEG".to_string()));
        }
        
        // In a real implementation, would resize the image to max_size
        // For milestone purposes, just return the original if it's reasonably sized
        if preview_data.len() > 64 * 1024 {
            return Err(WriteError::PreviewTooLarge(preview_data.len()));
        }
        
        Ok(preview_data.to_vec())
    }
}
```

**GPS Coordinate Writing**:
```rust
impl MetadataWriter {
    pub fn set_gps_coordinates(&mut self, latitude: f64, longitude: f64, altitude: Option<f64>) -> Result<()> {
        // Convert decimal degrees to degree/minute/second rationals
        let (lat_deg, lat_min, lat_sec) = self.decimal_to_dms(latitude.abs());
        let (lon_deg, lon_min, lon_sec) = self.decimal_to_dms(longitude.abs());
        
        // GPS latitude
        self.set_tag_value("GPSLatitude", TagValue::RationalArray(vec![
            Rational::new(lat_deg as i64, 1),
            Rational::new(lat_min as i64, 1), 
            Rational::new((lat_sec * 1000.0) as i64, 1000),
        ]))?;
        self.set_tag_value("GPSLatitudeRef", TagValue::String(
            if latitude >= 0.0 { "N" } else { "S" }.to_string()
        ))?;
        
        // GPS longitude
        self.set_tag_value("GPSLongitude", TagValue::RationalArray(vec![
            Rational::new(lon_deg as i64, 1),
            Rational::new(lon_min as i64, 1),
            Rational::new((lon_sec * 1000.0) as i64, 1000),
        ]))?;
        self.set_tag_value("GPSLongitudeRef", TagValue::String(
            if longitude >= 0.0 { "E" } else { "W" }.to_string()
        ))?;
        
        // GPS altitude (optional)
        if let Some(alt) = altitude {
            self.set_tag_value("GPSAltitude", TagValue::Rational(
                Rational::new((alt.abs() * 1000.0) as i64, 1000)
            ))?;
            self.set_tag_value("GPSAltitudeRef", TagValue::Integer(
                if alt >= 0.0 { 0 } else { 1 }
            ))?;
        }
        
        Ok(())
    }
    
    fn decimal_to_dms(&self, decimal: f64) -> (u32, u32, f64) {
        let degrees = decimal.floor() as u32;
        let minutes_float = (decimal - degrees as f64) * 60.0;
        let minutes = minutes_float.floor() as u32;
        let seconds = (minutes_float - minutes as f64) * 60.0;
        
        (degrees, minutes, seconds)
    }
}
```

## Success Criteria

### Core Requirements
- [ ] **MakerNote Preservation**: Modify EXIF while preserving manufacturer data
- [ ] **Cross-Format Sync**: Changes propagate correctly across EXIF/XMP/IPTC
- [ ] **Thumbnail Writing**: Generate and embed thumbnail images
- [ ] **GPS Coordinate Writing**: Proper GPS tag formatting and validation
- [ ] **Offset Fixup**: Handle pointer adjustments in complex maker notes

### Validation Tests
- Modify Canon CR2 files and verify maker notes remain intact
- Test cross-format synchronization (EXIF DateTime ↔ XMP ModifyDate)
- Write GPS coordinates and verify with mapping software
- Generate thumbnails and verify they display correctly
- Test with encrypted Nikon maker notes (preservation only)

## Implementation Boundaries

### Goals (Milestone 22)
- Professional-grade metadata modification capabilities
- Safe handling of manufacturer-specific data
- Multi-format synchronization for photo management workflows
- Complex tag writing (GPS, thumbnails, structured data)

### Non-Goals (Future Milestones)
- **Full encryption support**: Nikon decryption/re-encryption
- **RAW file writing**: Complex proprietary format modifications
- **Lens correction data**: Manufacturer-specific binary data modification
- **Video metadata writing**: Complex video format modifications

## Dependencies and Prerequisites

### Milestone Prerequisites
- **Milestone 21**: Basic write support foundation and validation system
- **Format support**: EXIF, XMP, and manufacturer processor infrastructure

### Technical Dependencies
- **Offset calculation**: Complex pointer arithmetic for maker note fixups
- **Format synchronization**: Understanding cross-format tag relationships
- **Binary data handling**: Thumbnail generation and GPS coordinate encoding

## Risk Mitigation

### MakerNote Corruption Risk
- **Risk**: Offset fixup errors could corrupt manufacturer-specific data
- **Mitigation**: Conservative preservation strategy with extensive validation
- **Strategy**: Start with read-only preservation, add modification incrementally

### Cross-Format Consistency Risk
- **Risk**: Synchronization rules might create conflicting metadata
- **Mitigation**: Prioritized format hierarchy with conflict resolution
- **Testing**: Comprehensive testing of synchronization edge cases

### Performance Risk: Complex Operations
- **Risk**: Offset fixup and multi-format writes could be slow
- **Mitigation**: Optimize common operations, provide progress feedback
- **Monitoring**: Profile performance on large files with complex maker notes

## Related Documentation

### Required Reading
- **ExifTool Write modules**: Complex write operation patterns
- **MakerNote documentation**: Manufacturer-specific format details
- **Offset fixup patterns**: Pointer adjustment strategies

### Implementation References
- **Canon/Nikon processors**: Existing maker note parsing infrastructure
- **Cross-format sync examples**: EXIF/XMP/IPTC tag relationships
- **ExifTool preservation patterns**: Binary data preservation strategies

This milestone completes the write support capabilities by handling the most challenging aspects of metadata modification while maintaining compatibility with manufacturer-specific features and professional workflow requirements.