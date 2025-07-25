# Milestone 21: Basic Write Support

**Duration**: 3-4 weeks  
**Goal**: Implement fundamental metadata writing capabilities with safety validation

## Overview

Write support transforms exif-oxide from a read-only metadata tool into a complete metadata management solution. This milestone implements the foundation for safe metadata modification, focusing on common use cases while establishing the validation and safety infrastructure required for all write operations.

## Background: ExifTool Write Complexity

**Write is 2-3x more complex than read support**:
- **Core write system**: Writer.pl alone is 7,409 lines
- **18,026 total lines** across all write modules  
- **3-layer validation system**: Value, tag-specific, and structural validation
- **Atomic write operations**: Safety-first approach with temp files

**Key Insight**: ExifTool's write reliability comes from extensive pre-validation and conservative safety mechanisms.

## Implementation Strategy

### Phase 1: Write Infrastructure Foundation (Week 1)

**Core Write System**:
```rust
pub struct MetadataWriter {
    validation_engine: ValidationEngine,
    format_writers: HashMap<FileType, Box<dyn FormatWriter>>,
    safety_options: WriteSafetyOptions,
    pending_changes: HashMap<String, TagChange>,
}

#[derive(Debug, Clone)]
pub struct TagChange {
    pub tag_name: String,
    pub old_value: Option<TagValue>,
    pub new_value: Option<TagValue>, // None = delete tag
    pub format: WriteFormat,
    pub validation_passed: bool,
}

#[derive(Debug, Clone)]
pub struct WriteSafetyOptions {
    pub create_backup: bool,
    pub backup_suffix: String,      // Default: "_original"
    pub atomic_writes: bool,        // Use temp files
    pub validate_before_write: bool,
    pub preserve_file_structure: bool,
}

impl MetadataWriter {
    pub fn set_tag_value(&mut self, tag_name: &str, value: TagValue) -> Result<()> {
        // Validate the tag change
        let change = TagChange {
            tag_name: tag_name.to_string(),
            old_value: self.get_current_value(tag_name),
            new_value: Some(value.clone()),
            format: self.determine_write_format(tag_name)?,
            validation_passed: false,
        };
        
        // Run validation
        self.validation_engine.validate_tag_change(&change)?;
        
        // Store pending change
        self.pending_changes.insert(tag_name.to_string(), change);
        Ok(())
    }
    
    pub fn write_changes(&mut self, input_path: &Path, output_path: &Path) -> Result<WriteResult> {
        // Create backup if requested
        if self.safety_options.create_backup {
            self.create_backup(input_path)?;
        }
        
        // Validate all pending changes
        self.validate_all_changes()?;
        
        // Perform atomic write operation
        self.atomic_write(input_path, output_path)
    }
}
```

**Validation Engine**:
```rust
pub struct ValidationEngine {
    type_validators: HashMap<String, Box<dyn TypeValidator>>,
    format_validators: HashMap<FileType, Box<dyn FormatValidator>>,
    cross_tag_rules: Vec<Box<dyn CrossTagRule>>,
}

pub trait TypeValidator: Send + Sync {
    fn validate_value(&self, value: &TagValue, constraints: &TagConstraints) -> Result<()>;
}

pub trait FormatValidator: Send + Sync {
    fn validate_write_operation(&self, changes: &[TagChange], file_data: &[u8]) -> Result<()>;
}

pub trait CrossTagRule: Send + Sync {
    fn validate_cross_dependencies(&self, changes: &HashMap<String, TagChange>) -> Result<()>;
}

// ExifTool's 3-layer validation system
impl ValidationEngine {
    // Layer 1: Value Validation
    pub fn validate_value(&self, tag_name: &str, value: &TagValue) -> Result<()> {
        let constraints = self.get_tag_constraints(tag_name)?;
        
        // Type checking
        if !constraints.allowed_types.contains(&value.get_type()) {
            return Err(WriteError::InvalidType {
                tag: tag_name.to_string(),
                expected: constraints.allowed_types.clone(),
                actual: value.get_type(),
            });
        }
        
        // Range validation
        match value {
            TagValue::Integer(i) => {
                if let Some(range) = &constraints.integer_range {
                    if *i < range.min || *i > range.max {
                        return Err(WriteError::ValueOutOfRange {
                            tag: tag_name.to_string(),
                            value: *i,
                            range: range.clone(),
                        });
                    }
                }
            },
            TagValue::Float(f) => {
                if let Some(range) = &constraints.float_range {
                    if *f < range.min || *f > range.max {
                        return Err(WriteError::ValueOutOfRange {
                            tag: tag_name.to_string(), 
                            value: *f,
                            range: range.clone(),
                        });
                    }
                }
            },
            TagValue::String(s) => {
                if let Some(max_len) = constraints.max_string_length {
                    if s.len() > max_len {
                        return Err(WriteError::StringTooLong {
                            tag: tag_name.to_string(),
                            length: s.len(),
                            max_length: max_len,
                        });
                    }
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    // Layer 2: Tag-Specific Validation
    pub fn validate_tag_rules(&self, tag_name: &str, value: &TagValue, all_changes: &HashMap<String, TagChange>) -> Result<()> {
        match tag_name {
            "Orientation" => {
                // Orientation must be 1-8
                if let TagValue::Integer(i) = value {
                    if *i < 1 || *i > 8 {
                        return Err(WriteError::InvalidOrientation(*i));
                    }
                }
            },
            "DateTime" | "DateTimeOriginal" | "DateTimeDigitized" => {
                // Validate datetime format: YYYY:MM:DD HH:MM:SS
                if let TagValue::String(dt) = value {
                    self.validate_datetime_format(dt)?;
                }
            },
            "GPSLatitude" | "GPSLongitude" => {
                // GPS coordinates must be rational arrays
                self.validate_gps_coordinate(tag_name, value)?;
            },
            _ => {} // No special rules for this tag
        }
        
        Ok(())
    }
    
    // Layer 3: Structural Validation
    pub fn validate_file_structure(&self, file_type: FileType, changes: &HashMap<String, TagChange>) -> Result<()> {
        match file_type {
            FileType::JPEG => {
                // Ensure EXIF version is present if writing EXIF tags
                if changes.keys().any(|k| k.starts_with("EXIF:")) {
                    if !changes.contains_key("ExifVersion") {
                        return Err(WriteError::MissingMandatoryTag("ExifVersion".to_string()));
                    }
                }
            },
            FileType::TIFF => {
                // TIFF requires certain baseline tags
                for required_tag in ["ImageWidth", "ImageLength", "BitsPerSample"] {
                    if changes.keys().any(|k| k.starts_with("EXIF:")) && !changes.contains_key(required_tag) {
                        return Err(WriteError::MissingMandatoryTag(required_tag.to_string()));
                    }
                }
            },
            _ => {} // No special structural requirements
        }
        
        Ok(())
    }
}
```

### Phase 2: EXIF Write Implementation (Week 1-2)

**EXIF Writer**:
```rust
pub struct ExifWriter {
    endian: ByteOrder,
    ifd_builders: HashMap<String, IfdBuilder>,
}

impl FormatWriter for ExifWriter {
    fn write_tags(&mut self, changes: &[TagChange], reader: &ExifReader) -> Result<Vec<u8>> {
        // Build new EXIF structure with changes
        let mut exif_data = Vec::new();
        
        // Write TIFF header
        self.write_tiff_header(&mut exif_data)?;
        
        // Build IFD0 with changes
        let ifd0_offset = self.build_ifd_with_changes("IFD0", changes, reader, &mut exif_data)?;
        
        // Update TIFF header with IFD0 offset
        exif_data[4..8].copy_from_slice(&(ifd0_offset as u32).to_le_bytes());
        
        // Build sub-IFDs (ExifIFD, GPS, etc.)
        self.build_sub_ifds(changes, reader, &mut exif_data)?;
        
        Ok(exif_data)
    }
}

impl ExifWriter {
    fn build_ifd_with_changes(
        &mut self,
        ifd_name: &str,
        changes: &[TagChange],
        reader: &ExifReader,
        data: &mut Vec<u8>,
    ) -> Result<usize> {
        
        let ifd_offset = data.len();
        let mut entries = Vec::new();
        
        // Get existing tags for this IFD
        let existing_tags = reader.get_ifd_tags(ifd_name)?;
        
        // Merge existing tags with changes
        for tag in existing_tags {
            if let Some(change) = changes.iter().find(|c| c.tag_name == tag.name) {
                if let Some(new_value) = &change.new_value {
                    // Modified tag
                    entries.push(self.build_tag_entry(&tag.name, new_value)?);
                }
                // If new_value is None, tag is deleted (omitted from entries)
            } else {
                // Unchanged tag - preserve as-is
                entries.push(self.build_tag_entry(&tag.name, &tag.value)?);
            }
        }
        
        // Add new tags that don't exist in original file
        for change in changes {
            if change.tag_name.starts_with(&format!("{}:", ifd_name)) {
                if !existing_tags.iter().any(|t| t.name == change.tag_name) {
                    if let Some(new_value) = &change.new_value {
                        entries.push(self.build_tag_entry(&change.tag_name, new_value)?);
                    }
                }
            }
        }
        
        // Sort entries by tag ID
        entries.sort_by_key(|e| e.tag_id);
        
        // Write IFD structure
        data.extend(&(entries.len() as u16).to_le_bytes()); // Entry count
        
        for entry in entries {
            data.extend(&entry.tag_id.to_le_bytes());
            data.extend(&entry.format.to_le_bytes());
            data.extend(&entry.count.to_le_bytes());
            data.extend(&entry.value_or_offset.to_le_bytes());
        }
        
        data.extend(&0u32.to_le_bytes()); // Next IFD offset (0 = end)
        
        Ok(ifd_offset)
    }
}
```

### Phase 3: Common Write Operations (Week 2-3)

**DateTime Handling**:
```rust
impl MetadataWriter {
    pub fn set_datetime(&mut self, tag: DateTimeTag, datetime: DateTime<Utc>) -> Result<()> {
        let formatted = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
        
        match tag {
            DateTimeTag::DateTime => {
                self.set_tag_value("DateTime", TagValue::String(formatted.clone()))?;
                // Also update XMP if present
                self.set_tag_value("XMP:ModifyDate", TagValue::String(formatted))?;
            },
            DateTimeTag::DateTimeOriginal => {
                self.set_tag_value("DateTimeOriginal", TagValue::String(formatted.clone()))?;
                self.set_tag_value("XMP:DateTimeOriginal", TagValue::String(formatted))?;
            },
            DateTimeTag::DateTimeDigitized => {
                self.set_tag_value("DateTimeDigitized", TagValue::String(formatted.clone()))?;
                self.set_tag_value("XMP:CreateDate", TagValue::String(formatted))?;
            },
        }
        
        Ok(())
    }
    
    // ExifTool's -AllDates functionality
    pub fn set_all_dates(&mut self, datetime: DateTime<Utc>) -> Result<()> {
        self.set_datetime(DateTimeTag::DateTime, datetime)?;
        self.set_datetime(DateTimeTag::DateTimeOriginal, datetime)?;
        self.set_datetime(DateTimeTag::DateTimeDigitized, datetime)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum DateTimeTag {
    DateTime,
    DateTimeOriginal,
    DateTimeDigitized,
}
```

**Basic Metadata Operations**:
```rust
impl MetadataWriter {
    pub fn set_artist(&mut self, artist: &str) -> Result<()> {
        self.set_tag_value("Artist", TagValue::String(artist.to_string()))?;
        self.set_tag_value("XMP:Creator", TagValue::String(artist.to_string()))?;
        Ok(())
    }
    
    pub fn set_copyright(&mut self, copyright: &str) -> Result<()> {
        self.set_tag_value("Copyright", TagValue::String(copyright.to_string()))?;
        self.set_tag_value("XMP:Rights", TagValue::String(copyright.to_string()))?;
        Ok(())
    }
    
    pub fn set_orientation(&mut self, orientation: u8) -> Result<()> {
        if orientation < 1 || orientation > 8 {
            return Err(WriteError::InvalidOrientation(orientation));
        }
        
        self.set_tag_value("Orientation", TagValue::Integer(orientation as i64))?;
        self.set_tag_value("XMP:Orientation", TagValue::Integer(orientation as i64))?;
        Ok(())
    }
    
    pub fn set_description(&mut self, description: &str) -> Result<()> {
        self.set_tag_value("ImageDescription", TagValue::String(description.to_string()))?;
        self.set_tag_value("XMP:Description", TagValue::String(description.to_string()))?;
        Ok(())
    }
}
```

### Phase 4: Safety and Atomic Operations (Week 3-4)

**Atomic Write Implementation**:
```rust
impl MetadataWriter {
    pub fn atomic_write(&mut self, input_path: &Path, output_path: &Path) -> Result<WriteResult> {
        // Create temporary file
        let temp_file = self.create_temp_file(output_path)?;
        
        // Perform the write operation to temp file
        let write_result = match self.write_to_file(input_path, &temp_file) {
            Ok(result) => result,
            Err(error) => {
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_file);
                return Err(error);
            }
        };
        
        // Validate the written file
        self.validate_written_file(&temp_file)?;
        
        // Atomic rename (move temp file to final location)
        fs::rename(&temp_file, output_path).map_err(|e| WriteError::FileOperationFailed {
            operation: "rename".to_string(),
            path: output_path.to_path_buf(),
            source: e,
        })?;
        
        Ok(write_result)
    }
    
    fn create_backup(&self, original_path: &Path) -> Result<PathBuf> {
        let backup_path = original_path.with_extension(
            format!("{}{}", 
                original_path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                &self.safety_options.backup_suffix
            )
        );
        
        fs::copy(original_path, &backup_path).map_err(|e| WriteError::BackupFailed {
            original: original_path.to_path_buf(),
            backup: backup_path.clone(),
            source: e,
        })?;
        
        Ok(backup_path)
    }
    
    fn validate_written_file(&self, file_path: &Path) -> Result<()> {
        // Read back the written file and verify it's valid
        let reader = ExifReader::from_file(file_path)?;
        
        // Basic sanity checks
        if reader.get_file_type() == FileType::Unknown {
            return Err(WriteError::CorruptedOutput);
        }
        
        // Verify critical tags are still readable
        for change in self.pending_changes.values() {
            if let Some(new_value) = &change.new_value {
                if let Some(written_value) = reader.get_tag_value(&change.tag_name) {
                    if !self.values_equivalent(&written_value, new_value) {
                        return Err(WriteError::ValueMismatch {
                            tag: change.tag_name.clone(),
                            expected: new_value.clone(),
                            actual: written_value,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## Success Criteria

### Core Requirements
- [ ] **Basic Tag Writing**: DateTime, Orientation, Artist, Copyright, Description
- [ ] **Validation System**: 3-layer validation prevents invalid writes
- [ ] **Safety Mechanisms**: Atomic writes with backup creation
- [ ] **JPEG/TIFF Support**: Core format write capability
- [ ] **Error Handling**: Graceful failure with meaningful error messages

### Validation Tests
- Write basic metadata to JPEG and verify with ExifTool
- Test validation rejection of invalid values
- Verify atomic write behavior (no partial writes on failure)
- Test backup creation and restoration
- Validate cross-format synchronization (EXIF ↔ XMP)

## Implementation Boundaries

### Goals (Milestone 21)
- Foundation for all metadata writing operations
- Common metadata modifications for photo management
- Safety-first approach with validation and backups
- JPEG and TIFF format support

### Non-Goals (Future Milestones)
- **XMP writing**: Complex RDF/XML structure writing
- **MakerNote modification**: Manufacturer-specific data preservation
- **RAW file writing**: Complex format-specific write operations
- **Advanced validation**: Complete metadata standard compliance

## Dependencies and Prerequisites

### Milestone Prerequisites
- **All read infrastructure**: Core tag extraction and format parsing
- **Error handling system**: Write operations need robust error classification

### Technical Dependencies
- **File I/O**: Atomic write operations and backup management
- **Validation**: Type checking and format constraint validation
- **Cross-format sync**: Understanding tag relationships across formats

## Risk Mitigation

### Data Safety Risk
- **Risk**: Write operations could corrupt or destroy files
- **Mitigation**: Atomic writes, mandatory validation, backup creation
- **Testing**: Extensive testing with corrupted/edge case files

### Validation Complexity Risk
- **Risk**: Overly strict validation prevents legitimate writes
- **Mitigation**: Layered validation with user override options
- **Strategy**: Start conservative, relax based on user feedback

### Performance Risk: Validation Overhead
- **Risk**: Extensive validation slows write operations
- **Mitigation**: Configurable validation levels, optimization for common cases
- **Monitoring**: Profile validation performance on large files

## Related Documentation

### Required Reading
- **ExifTool Writer.pl**: Core write system architecture and validation patterns
- **EXIF specification**: Understanding tag constraints and requirements
- **Safety documentation**: File handling best practices

### Implementation References
- **ExifTool write patterns**: SetNewValue/GetNewValue infrastructure
- **Validation examples**: Type checking and cross-tag dependency rules
- **Atomic write patterns**: Temp file creation and validation strategies

This milestone establishes the foundation for metadata writing while prioritizing data safety and validation. The conservative approach ensures reliability while building toward more advanced write capabilities in future milestones.