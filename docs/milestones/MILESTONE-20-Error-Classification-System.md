# Milestone 20: Error Classification System

**Duration**: 2-3 weeks  
**Goal**: Implement ExifTool's sophisticated error handling and manufacturer quirk management

## Overview

Robust error handling is what separates ExifTool from other metadata tools - it successfully extracts data from files that cause other tools to fail completely. This milestone implements ExifTool's multi-level error classification system, manufacturer-specific workarounds, and graceful degradation strategies.

## Background: ExifTool's Error Sophistication

**Error Classification Levels**:
- **Level 0 (Major Error)**: Critical failures that stop processing
- **Level 1 (Minor Error)**: Recoverable errors (with `-m` flag)  
- **Level 2 ([Minor] Warning)**: Non-critical issues with `[Minor]` prefix
- **Level 3 (Validation Warning)**: Only shown in validation mode

**Key Insight**: ExifTool's robustness comes from treating manufacturer-specific data as "non-essential" - maker notes can be corrupted without preventing basic metadata extraction.

## Implementation Strategy

### Phase 1: Core Error Classification Infrastructure (Week 1)

**Error Type System**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorLevel {
    Fatal,      // Stop processing immediately
    Minor,      // Recoverable with ignore_minor_errors flag
    Warning,    // Continue processing, report issue
    Validation, // Only show in validation mode
}

#[derive(Debug, Clone)]
pub struct ExifError {
    pub level: ErrorLevel,
    pub message: String,
    pub context: ErrorContext,
    pub ignorable: bool,
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file_path: Option<PathBuf>,
    pub format: Option<String>,
    pub location: ErrorLocation,
    pub manufacturer: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    FileHeader,
    IFDDirectory { ifd_name: String, entry_index: Option<usize> },
    MakerNotes { manufacturer: String, tag_id: Option<u16> },
    BinaryData { processor: String, offset: usize },
    TagProcessing { tag_name: String },
}
```

**Error Handler Implementation**:
```rust
pub struct ErrorHandler {
    warnings: Vec<ExifError>,
    errors: Vec<ExifError>,
    options: ErrorHandlingOptions,
}

#[derive(Debug, Clone)]
pub struct ErrorHandlingOptions {
    pub ignore_minor_errors: bool,     // -m flag equivalent
    pub strict_validation: bool,       // Validation mode
    pub demote_errors: bool,           // Convert errors to warnings
    pub max_warnings: usize,           // Prevent warning spam
}

impl ErrorHandler {
    pub fn error(&mut self, message: String, level: ErrorLevel, context: ErrorContext) -> Result<()> {
        let error = ExifError {
            level: level.clone(),
            message: message.clone(),
            context,
            ignorable: level != ErrorLevel::Fatal,
        };
        
        match level {
            ErrorLevel::Fatal => {
                self.errors.push(error);
                Err(ExifError::Fatal { message })
            },
            ErrorLevel::Minor if self.options.ignore_minor_errors => {
                self.warn(format!("[minor] {}", message), ErrorLevel::Warning, error.context);
                Ok(())
            },
            ErrorLevel::Minor => {
                self.errors.push(error);
                Err(ExifError::Minor { message })
            },
            ErrorLevel::Warning => {
                self.warnings.push(error);
                Ok(())
            },
            ErrorLevel::Validation if !self.options.strict_validation => {
                // Hide validation warnings unless in validation mode
                Ok(())
            },
            ErrorLevel::Validation => {
                self.warnings.push(error);
                Ok(())
            },
        }
    }
    
    pub fn warn(&mut self, message: String, level: ErrorLevel, context: ErrorContext) {
        let prefixed_message = match level {
            ErrorLevel::Warning => format!("[Minor] {}", message),
            ErrorLevel::Validation => format!("[Validation] {}", message),
            _ => message,
        };
        
        if self.warnings.len() < self.options.max_warnings {
            self.warnings.push(ExifError {
                level,
                message: prefixed_message,
                context,
                ignorable: true,
            });
        }
    }
}
```

### Phase 2: Manufacturer-Specific Quirk Handling (Week 1-2)

**Quirk Registry System**:
```rust
pub struct ManufacturerQuirkHandler {
    quirks: HashMap<String, Vec<Box<dyn ManufacturerQuirk>>>,
}

pub trait ManufacturerQuirk: Send + Sync {
    fn applies_to(&self, make: &str, model: &str, firmware: Option<&str>) -> bool;
    fn apply_quirk(&self, data: &mut [u8], context: &QuirkContext) -> Result<QuirkResult>;
    fn description(&self) -> &str;
}

#[derive(Debug)]
pub struct QuirkResult {
    pub applied: bool,
    pub warning_message: Option<String>,
    pub data_modified: bool,
}

#[derive(Debug)]
pub struct QuirkContext {
    pub file_type: String,
    pub processing_stage: ProcessingStage,
    pub ifd_context: Option<String>,
}

#[derive(Debug)]
pub enum ProcessingStage {
    FileHeader,
    IFDProcessing,
    MakerNoteProcessing,
    TagExtraction,
}
```

**Canon-Specific Quirks**:
```rust
// Canon EOS 40D firmware bug - incorrect directory counts
pub struct CanonEOS40DDirectoryCountQuirk;
impl ManufacturerQuirk for CanonEOS40DDirectoryCountQuirk {
    fn applies_to(&self, make: &str, model: &str, firmware: Option<&str>) -> bool {
        make == "Canon" && model == "Canon EOS 40D"
        // Could also check firmware version if needed
    }
    
    fn apply_quirk(&self, data: &mut [u8], context: &QuirkContext) -> Result<QuirkResult> {
        if let ProcessingStage::IFDProcessing = context.processing_stage {
            if context.ifd_context.as_deref() == Some("MakerNotes") {
                // Check if last directory entry has invalid format
                // ExifTool Canon.pm:6318 pattern
                let entries_count = u16::from_le_bytes([data[0], data[1]]);
                if entries_count > 0 {
                    let last_entry_offset = 2 + 12 * (entries_count as usize - 1);
                    if last_entry_offset + 4 < data.len() {
                        let format = u16::from_le_bytes([data[last_entry_offset + 2], data[last_entry_offset + 3]]);
                        if format < 1 || format > 13 {
                            // Fix the directory count
                            let corrected_count = entries_count - 1;
                            data[0..2].copy_from_slice(&corrected_count.to_le_bytes());
                            
                            return Ok(QuirkResult {
                                applied: true,
                                warning_message: Some(format!(
                                    "Fixed Canon EOS 40D directory count bug ({} -> {})", 
                                    entries_count, corrected_count
                                )),
                                data_modified: true,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(QuirkResult { applied: false, warning_message: None, data_modified: false })
    }
    
    fn description(&self) -> &str {
        "Canon EOS 40D firmware 1.0.4 directory count bug fix"
    }
}

// Sony double-encryption bug
pub struct SonyDoubleEncryptionQuirk;
impl ManufacturerQuirk for SonyDoubleEncryptionQuirk {
    fn applies_to(&self, make: &str, _model: &str, _firmware: Option<&str>) -> bool {
        make == "SONY"
    }
    
    fn apply_quirk(&self, data: &mut [u8], context: &QuirkContext) -> Result<QuirkResult> {
        if let ProcessingStage::MakerNoteProcessing = context.processing_stage {
            // Detect double-encrypted Sony data and fix it
            if self.is_double_encrypted(data) {
                self.decipher_data(data)?;
                return Ok(QuirkResult {
                    applied: true,
                    warning_message: Some("Fixed double-enciphered Sony metadata".to_string()),
                    data_modified: true,
                });
            }
        }
        
        Ok(QuirkResult { applied: false, warning_message: None, data_modified: false })
    }
    
    fn description(&self) -> &str {
        "Sony IDC utility double-encryption bug fix"
    }
}
```

### Phase 3: Error Recovery Mechanisms (Week 2)

**Corrupted Data Recovery**:
```rust
pub struct CorruptionRecoveryHandler;

impl CorruptionRecoveryHandler {
    // ExifTool pattern: scan for JPEG/TIFF signatures in unknown files
    pub fn attempt_embedded_format_recovery(
        &self, 
        data: &[u8], 
        error_handler: &mut ErrorHandler
    ) -> Result<Option<(FileType, usize)>> {
        
        // Scan for known magic signatures
        for (offset, window) in data.windows(4).enumerate() {
            match window {
                [0xff, 0xd8, 0xff, _] => {
                    error_handler.warn(
                        format!("Processing JPEG-like data after unknown {}-byte header", offset),
                        ErrorLevel::Warning,
                        ErrorContext {
                            file_path: None,
                            format: Some("JPEG".to_string()),
                            location: ErrorLocation::FileHeader,
                            manufacturer: None,
                        }
                    );
                    return Ok(Some((FileType::JPEG, offset)));
                },
                [b'M', b'M', 0x00, 0x2a] | [b'I', b'I', 0x2a, 0x00] => {
                    error_handler.warn(
                        format!("Processing TIFF-like data after unknown {}-byte header", offset),
                        ErrorLevel::Warning,
                        ErrorContext {
                            file_path: None,
                            format: Some("TIFF".to_string()),
                            location: ErrorLocation::FileHeader,
                            manufacturer: None,
                        }
                    );
                    return Ok(Some((FileType::TIFF, offset)));
                },
                _ => continue,
            }
        }
        
        Ok(None)
    }
    
    // Large array protection
    pub fn validate_array_size(
        &self,
        count: u32,
        format: &str,
        tag_name: &str,
        error_handler: &mut ErrorHandler
    ) -> Result<bool> {
        
        const MAX_REASONABLE_COUNT: u32 = 100_000;
        const MAX_EXTREME_COUNT: u32 = 2_000_000;
        
        if count > MAX_REASONABLE_COUNT && !matches!(format, "undef" | "string" | "binary") {
            let error_level = if count > MAX_EXTREME_COUNT {
                ErrorLevel::Minor // Major if extremely large
            } else {
                ErrorLevel::Warning
            };
            
            error_handler.error(
                format!("Ignoring {} with excessive count ({})", tag_name, count),
                error_level,
                ErrorContext {
                    file_path: None,
                    format: Some(format.to_string()),
                    location: ErrorLocation::TagProcessing { tag_name: tag_name.to_string() },
                    manufacturer: None,
                }
            )?;
            
            return Ok(false); // Skip processing this tag
        }
        
        Ok(true) // Continue processing
    }
}
```

**Progressive Fallback System**:
```rust
pub struct ProcessingFallbackHandler;

impl ProcessingFallbackHandler {
    pub fn process_with_fallback<T>(
        &self,
        primary_processor: impl FnOnce() -> Result<T>,
        fallback_processor: impl FnOnce() -> Result<T>,
        error_handler: &mut ErrorHandler,
        context: ErrorContext,
    ) -> Result<T> {
        
        match primary_processor() {
            Ok(result) => Ok(result),
            Err(primary_error) => {
                error_handler.warn(
                    format!("Primary processing failed: {}, attempting fallback", primary_error),
                    ErrorLevel::Warning,
                    context.clone()
                );
                
                fallback_processor().map_err(|fallback_error| {
                    error_handler.error(
                        format!("Both primary and fallback processing failed: {} / {}", 
                               primary_error, fallback_error),
                        ErrorLevel::Minor,
                        context
                    );
                    fallback_error
                })
            }
        }
    }
}
```

### Phase 4: MINOR_ERRORS Integration (Week 2-3)

**Tag Table Error Handling**:
```rust
#[derive(Debug, Clone)]
pub struct TagTableConfig {
    pub minor_errors: bool,  // ExifTool's VARS => { MINOR_ERRORS => 1 }
    pub ignore_unknown: bool,
    pub max_recursion_depth: usize,
}

impl TagProcessor {
    pub fn process_tag_table(
        &mut self,
        table: &TagTable,
        data: &[u8],
        error_handler: &mut ErrorHandler,
    ) -> Result<()> {
        
        let context = ErrorContext {
            file_path: self.file_path.clone(),
            format: Some(table.name.to_string()),
            location: ErrorLocation::IFDDirectory { 
                ifd_name: table.name.to_string(), 
                entry_index: None 
            },
            manufacturer: self.current_manufacturer.clone(),
        };
        
        // Check if this table has MINOR_ERRORS flag
        let demote_errors = table.config.minor_errors;
        
        for (index, entry) in self.parse_ifd_entries(data)?.iter().enumerate() {
            let mut entry_context = context.clone();
            if let ErrorLocation::IFDDirectory { ref mut entry_index, .. } = entry_context.location {
                *entry_index = Some(index);
            }
            
            match self.process_ifd_entry(entry, error_handler) {
                Ok(_) => continue,
                Err(entry_error) => {
                    let error_level = if demote_errors {
                        ErrorLevel::Warning  // Convert to warning if MINOR_ERRORS is set
                    } else {
                        ErrorLevel::Minor
                    };
                    
                    error_handler.error(
                        format!("Failed to process entry {}: {}", index, entry_error),
                        error_level,
                        entry_context
                    )?;
                }
            }
        }
        
        Ok(())
    }
}
```

**Integration with Existing Processors**:
```rust
// Update existing manufacturer processors to use error handling
impl NikonProcessor {
    pub fn process_with_error_handling(
        &mut self,
        reader: &mut ExifReader,
        error_handler: &mut ErrorHandler,
    ) -> Result<()> {
        
        // Apply Nikon-specific quirks first
        let quirk_handler = ManufacturerQuirkHandler::for_manufacturer("NIKON CORPORATION");
        let quirk_context = QuirkContext {
            file_type: "NEF".to_string(),
            processing_stage: ProcessingStage::MakerNoteProcessing,
            ifd_context: Some("Nikon".to_string()),
        };
        
        if let Some(mut maker_data) = reader.get_maker_note_data() {
            for quirk in quirk_handler.get_applicable_quirks(&reader.get_make(), &reader.get_model()) {
                let result = quirk.apply_quirk(&mut maker_data, &quirk_context)?;
                if result.applied {
                    if let Some(warning) = result.warning_message {
                        error_handler.warn(warning, ErrorLevel::Warning, ErrorContext {
                            file_path: reader.file_path.clone(),
                            format: Some("Nikon MakerNotes".to_string()),
                            location: ErrorLocation::MakerNotes { 
                                manufacturer: "Nikon".to_string(), 
                                tag_id: None 
                            },
                            manufacturer: Some("NIKON CORPORATION".to_string()),
                        });
                    }
                }
            }
        }
        
        // Process with error recovery
        self.process_nikon_makernotes_with_recovery(reader, error_handler)
    }
}
```

## Success Criteria

### Core Requirements
- [ ] **Error Classification**: Four-level error system (Fatal/Minor/Warning/Validation)
- [ ] **Manufacturer Quirks**: Handle Canon EOS 40D, Sony double-encryption, and other known issues
- [ ] **Graceful Degradation**: Continue processing when non-essential data is corrupted
- [ ] **Recovery Mechanisms**: Embedded format detection and fallback processing
- [ ] **MINOR_ERRORS Support**: Tag tables can mark themselves as non-essential

### Validation Tests
- Process 1000+ files without crashing on corrupted data
- Handle Canon EOS 40D maker note corruption gracefully
- Recover JPEG data from files with unknown headers
- Validate error messages are semantically similar to ExifTool's output
- Test with deliberately corrupted files to verify recovery

## Implementation Boundaries

### Goals (Milestone 20)
- Complete error classification and handling system
- Essential manufacturer quirk workarounds
- Robust corruption recovery mechanisms
- Graceful degradation for non-essential data

### Non-Goals (Future Milestones)
- **Verbatim error message replication**: Semantic similarity is sufficient
- **Complete quirk coverage**: Focus on most common/critical issues
- **Advanced validation**: Basic validation warnings, not comprehensive validation mode
- **Performance optimization**: Focus on correctness over speed

## Dependencies and Prerequisites

### Milestone Prerequisites
- **All format processors**: Error handling integrates with existing format support
- **Core infrastructure**: ExifReader and tag processing systems

### Integration Points
- **Manufacturer processors**: Canon, Nikon, Sony processors need error handling integration
- **Format parsers**: TIFF, JPEG, video parsers need corruption recovery
- **CLI interface**: Error/warning reporting in command-line output

## Risk Mitigation

### Complexity Risk: Manufacturer Quirks
- **Risk**: Each manufacturer has unique quirks requiring specialized handling
- **Mitigation**: Modular quirk system allows incremental addition of workarounds
- **Strategy**: Start with most common issues, add edge cases as needed

### Performance Risk: Error Checking Overhead
- **Risk**: Extensive error checking could slow processing
- **Mitigation**: Make error handling optional with configuration flags
- **Optimization**: Only apply quirks when manufacturer matches

### Compatibility Risk: Error Message Changes
- **Risk**: Different error messages might confuse users
- **Mitigation**: Focus on semantic equivalence, not verbatim reproduction
- **Documentation**: Clear documentation about error handling differences

## Related Documentation

### Required Reading
- **ExifTool.pm**: Core error handling patterns and classification system
- **Manufacturer modules**: Canon.pm, Nikon.pm, Sony.pm quirk examples
- **FILE_TYPES.md**: Error recovery patterns for format detection

### Implementation References
- **Error handling patterns**: ExifTool.pm lines 5522-5560 (Warn/Error functions)
- **Quirk examples**: Canon EOS 40D fix, Sony double-encryption handling
- **Recovery mechanisms**: Last-ditch JPEG/TIFF detection patterns

This milestone establishes exif-oxide as a robust metadata extraction tool that can handle real-world corrupted files with the same reliability as ExifTool, while providing clear feedback about data quality issues.