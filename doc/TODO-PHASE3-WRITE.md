# Phase 3: Write Support Framework

**Goal**: Add safe metadata writing capabilities with backup/rollback support.

**Duration**: 2-3 weeks

**Dependencies**: Phase 1 (multi-format), Phase 2 (maker notes), comprehensive read support

## IMMEDIATE (Core write infrastructure - 1 week)

### 1. Safe Write Architecture
**Context**: Implement ExifTool's approach to safe metadata writing with backup and atomic operations.

**Files to create**:
- `src/write/mod.rs` - Main write API
- `src/write/safety.rs` - Backup and rollback mechanisms
- `src/write/validation.rs` - Pre-write validation

**Safety pattern to implement**:
```rust
pub fn write_metadata<P: AsRef<Path>>(path: P, metadata: &Metadata) -> Result<WriteResult> {
    // 1. Create backup (.exif_original)
    let backup = create_backup(&path)?;
    
    // 2. Write to temporary file
    let temp_file = write_to_temp(&path, metadata)?;
    
    // 3. Validate temp file
    validate_written_metadata(&temp_file)?;
    
    // 4. Atomic rename
    atomic_replace(&path, temp_file)?;
    
    Ok(WriteResult { backup_path: backup })
}
```

**Reference ExifTool approach**: Study how ExifTool handles write safety, preserves unknown tags, maintains file integrity.

### 2. Metadata Preservation Framework
**Context**: Preserve all unknown tags and metadata during write operations.

**Files to create**:
- `src/write/preservation.rs` - Unknown tag preservation
- `src/write/round_trip.rs` - Round-trip validation

**Preservation strategy**:
- Read complete metadata before writing
- Preserve unknown EXIF tags, maker notes, XMP sections
- Only modify explicitly requested fields
- Maintain all binary data structures

**Implementation approach**:
```rust
pub struct WriteOperation {
    original_metadata: CompleteMetadata,  // Everything from file
    modifications: HashMap<String, Value>, // Only what user wants to change
    preservation_mask: PreservationMask,   // What to keep unchanged
}
```

## SHORT-TERM (Format-specific writers - 2 weeks)

### 3. JPEG Write Support (1 week)
**Context**: JPEG is the most common format, needs both EXIF and XMP coordination.

**Reference existing code**: Study `src/core/jpeg.rs` for segment structure understanding.

**Files to create**:
- `src/write/jpeg.rs` - JPEG-specific writing
- `src/write/segments.rs` - APP1 segment management

**JPEG-specific challenges**:
- Multiple APP1 segments (EXIF + XMP)
- Segment size limits (64KB)
- Extended XMP handling (multiple segments)
- Thumbnail preservation

**Implementation pattern**:
```rust
impl Writer for JpegWriter {
    fn write_metadata(&self, file: &mut File, metadata: &Metadata) -> Result<()> {
        // 1. Parse existing segments
        let segments = parse_jpeg_segments(file)?;
        
        // 2. Update EXIF segment
        let new_exif = update_exif_segment(&segments.exif, &metadata.exif)?;
        
        // 3. Update XMP segment  
        let new_xmp = update_xmp_segment(&segments.xmp, &metadata.xmp)?;
        
        // 4. Reconstruct JPEG with new segments
        reconstruct_jpeg(file, segments, new_exif, new_xmp)?;
    }
}
```

### 4. TIFF Write Support (1 week)
**Context**: TIFF is foundation for many RAW formats, complex IFD reconstruction required.

**Reference existing code**: Study `src/core/ifd.rs` for IFD structure understanding.

**Files to create**:
- `src/write/tiff.rs` - TIFF-specific writing
- `src/write/ifd_builder.rs` - IFD reconstruction

**TIFF-specific challenges**:
- IFD chain reconstruction
- Offset recalculation after changes
- Maker note preservation and offset correction
- Multiple IFD support (IFD0, IFD1, ExifIFD)

**Key considerations**:
- Preserve maker note byte order and offsets
- Handle SubIFD structures (ExifIFD, GPS IFD)
- Maintain thumbnail data in IFD1
- Calculate correct offsets for variable-length data

## MEDIUM-TERM (Advanced write features - 1 week)

### 5. Maker Note Write Support
**Context**: Preserve manufacturer-specific data during write operations.

**Reference existing maker notes**: Study Canon, Nikon, Sony implementations for preservation patterns.

**Files to create**:
- `src/write/maker_notes.rs` - Maker note preservation
- Integration with format-specific writers

**Maker note challenges**:
- Offset corrections when EXIF data moves
- Binary data preservation
- Encrypted sections (read-only, preserve as-is)
- Model-specific structures

**Preservation approach**:
```rust
pub fn preserve_maker_note(
    original: &[u8], 
    exif_offset_delta: i32
) -> Result<Vec<u8>> {
    // Adjust internal offsets while preserving binary data
    // Handle manufacturer-specific offset calculation
    // Preserve encrypted/unknown sections exactly
}
```

### 6. XMP Write Integration
**Context**: Coordinate EXIF and XMP updates, handle synchronization.

**Reference existing XMP**: Study `src/xmp/` modules for structure understanding.

**Files to create**:
- `src/write/xmp.rs` - XMP writing support
- `src/write/coordination.rs` - EXIF/XMP field synchronization

**XMP write challenges**:
- Extended XMP reassembly (>64KB)
- Namespace preservation
- Field synchronization with EXIF
- XML formatting and validation

### 7. Comprehensive Write API
**Context**: Unified API that works across all supported formats.

**Files to modify**:
- `src/lib.rs` - Add public write API
- `src/main.rs` - Add write command-line support

**Public API design**:
```rust
// High-level API
pub fn write_exif_field<P: AsRef<Path>>(path: P, field: &str, value: &str) -> Result<()>;

// Advanced API  
pub fn write_metadata<P: AsRef<Path>>(path: P, metadata: &Metadata) -> Result<WriteResult>;

// Batch API
pub fn write_metadata_batch(operations: Vec<WriteOperation>) -> Result<Vec<WriteResult>>;
```

## LONG-TERM (Production features - ongoing)

### 8. Write Validation & Testing
**Context**: Ensure written files are valid and readable by ExifTool and other software.

**Validation approach**:
- Round-trip testing (write then read back)
- ExifTool compatibility validation
- File integrity checking
- Thumbnail preservation validation

**Testing strategy**:
```bash
# Round-trip test
cargo run -- input.jpg --write-field Make "Test Camera"
exiftool input.jpg | grep "Camera Make"  # Should show "Test Camera"
cargo run -- input.jpg  # Should read back correctly
```

### 9. Performance Optimization
**Context**: Writing should be fast while maintaining safety.

**Optimization targets**:
- Minimize file I/O (single pass where possible)
- Efficient IFD reconstruction
- Smart segment management (only rewrite what changed)
- Memory-efficient handling of large files

**Performance goals**:
- JPEG write: <50ms for typical file
- TIFF write: <100ms for typical RAW file  
- Memory usage: <10MB additional during write

### 10. Advanced Write Features
**Context**: Professional workflow features.

**Advanced features**:
- Batch operations on multiple files
- Template application (apply metadata from one file to many)
- Selective field updates (only change specific tags)
- Write verification and checksum validation

## Technical Architecture

### Write Safety Principles
1. **Never modify original file directly**
2. **Always create backup before write**
3. **Use atomic operations (temp file + rename)**
4. **Validate written data before committing**
5. **Preserve all unknown metadata**

### Format Support Strategy  
**Priority order**:
1. **JPEG** (90% of use cases)
2. **TIFF/RAW** (professional workflows)
3. **HEIF/HEIC** (modern mobile formats)
4. **PNG** (web/graphics workflows)

### Error Handling Strategy
- **Preserve originals**: Never leave corrupted files
- **Clear error messages**: Explain what went wrong and how to recover
- **Rollback capability**: Automatic restoration from backup on failure
- **Validation feedback**: Report what was written vs what was requested

## Success Criteria
- [ ] Safe write operations with backup/rollback
- [ ] JPEG and TIFF write support implemented
- [ ] Unknown tag and maker note preservation
- [ ] Round-trip compatibility (write then read gives same result)
- [ ] ExifTool compatibility for written files
- [ ] Performance targets met (<100ms for typical operations)
- [ ] Comprehensive write validation and testing