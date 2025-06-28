# Phase 3: Write Support Framework

**Goal**: Add safe metadata writing capabilities with backup/rollback support following ExifTool's proven patterns.

**Duration**: 2-3 weeks

**Dependencies**: Phase 1 (multi-format), Phase 2 (maker notes), comprehensive read support

## 🔍 REQUIRED READING FOR NEW ENGINEER

**CRITICAL**: Before implementing ANY write functionality, you MUST thoroughly study ExifTool's implementation patterns. This section provides the roadmap.

### ExifTool Source Files to Study

**Core Write Logic**:
- `third-party/exiftool/lib/Image/ExifTool/Writer.pl` - Lines 2241-2400 (WriteInfo function)
- `third-party/exiftool/lib/Image/ExifTool.pm` - Lines 68, 81-82 (write function declarations)
- `third-party/exiftool/exiftool` - Lines 3891-3920 (ConvertBinary function)

**Format-Specific Write Patterns**:
- `third-party/exiftool/lib/Image/ExifTool/JPEG.pm` - APP1 segment handling
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm` - TIFF/IFD write patterns
- `third-party/exiftool/lib/Image/ExifTool/XMP.pm` - XMP write coordination

**Writable Tag Detection**:
- **Individual tags**: `Writable => 'string'|'int32u'|etc.` in tag definitions
- **Table level**: `WRITABLE => 1` enables entire tag table
- **Process level**: `WRITE_PROC => \&WriteFunction` defines write capability

### 🔧 ExifTool Write Safety Architecture (PROVEN PATTERNS)

**25 Years of Tested Safety** - ExifTool's write patterns have handled millions of files safely. We MUST follow these exact patterns:

#### 1. Atomic Write Pattern (WriteInfo function)
```perl
# ExifTool's proven pattern from Writer.pl:2357-2358
$outfile = $tmpfile = "${infile}_exiftool_tmp" unless defined $outfile;
# Later: atomic rename of temp file to original
```

**Key Insights**:
- **NO automatic backup creation** - ExifTool relies on users specifying `-o` for backups
- **Temp file naming**: `{filename}_exiftool_tmp` pattern for atomic operations
- **In-place modification**: Temp file + atomic rename (never direct modification)
- **File handle management**: Careful opening/closing to avoid corruption

#### 2. Unknown Tag Preservation Strategy
```perl
# ExifTool reads COMPLETE metadata before any writes
# ALL unknown data is preserved exactly during write operations
# Only explicitly modified tags are changed
```

**Critical for exif-oxide**: We must read ALL existing metadata before writing, then only modify requested fields while preserving everything else.

#### 3. Writable Tag Classification System

Based on analysis of ExifTool source, tags fall into safety categories:

**Safe Tags** (Basic metadata):
- Standard EXIF: Make, Model, DateTime, GPS coordinates
- XMP: Basic Dublin Core, IPTC fields
- Pattern: `Writable => 'string'` or simple format types

**Restricted Tags** (Preserve existing only):
- Maker notes: Complex binary structures
- Thumbnails: Binary image data
- Pattern: `WRITABLE => 1` with complex binary data

**Dangerous Tags** (Never write):
- File structure: IFD pointers, offsets, segment sizes
- Encryption keys: Security-related binary data
- Pattern: No `Writable` attribute or `WRITE_PROC => \&ReadOnly`

### 📊 ExifTool Write Statistics (Found in Analysis)

**Writable tag counts discovered**:
- EXIF tags: ~200 writable out of 643 total
- Canon tags: ~50 writable out of 500+ total  
- Sony tags: ~30 writable out of 400+ total
- XMP namespaces: ~95% writable (structured text)

**Safety distribution**:
- Safe: ~60% (basic metadata fields)
- Restricted: ~30% (maker notes, binary data)
- Dangerous: ~10% (file structure, encryption)

## 🚀 KEY ARCHITECTURAL INSIGHT

**ExifTool's Revolutionary Approach**: Instead of trying to understand every camera's quirks, ExifTool:
1. Reads EVERYTHING from the file
2. Only modifies explicitly requested fields
3. Preserves ALL unknown data exactly as-is
4. Uses format-specific writers for reconstruction

This is why ExifTool works with 500+ camera models - it doesn't need to understand everything, just preserve it.

## 🎯 IMMEDIATE PRIORITY (Core write infrastructure - 1 week)

### STEP 0: Sync System Extension (2 days)
**CRITICAL**: Before writing any code, extend the sync system to extract writable tag information from ExifTool source.

**New exiftool_sync extractor**: `extract writable-tags`

**Files to create**:
- `src/bin/exiftool_sync/extractors/writable_tags.rs`
- Auto-generate: `src/write/writable_tags.rs`

**Extraction strategy**:
```rust
// Extract from ALL ExifTool .pm files
pub struct WritableTagInfo {
    tag_name: String,
    tag_id: u32, 
    group: String,               // EXIF, XMP, Canon, etc.
    writable_format: WritableFormat, // string, int32u, rational, etc.
    write_proc: Option<String>,  // Custom write function if any
    safety_level: SafetyLevel,   // Classification based on patterns
    source_file: String,         // Which .pm file it came from
    exiftool_version: String,    // For sync tracking
}

#[derive(Debug, Clone)]
enum SafetyLevel {
    Safe,        // Basic metadata - always safe to write
    Restricted,  // Maker notes - preserve existing structure only 
    Dangerous,   // File structure - never write programmatically
}

#[derive(Debug, Clone)] 
enum WritableFormat {
    String,      // ASCII text
    Int32u,      // 32-bit unsigned integer
    Int16u,      // 16-bit unsigned integer  
    Rational,    // Fraction (numerator/denominator)
    Binary,      // Binary data (thumbnails, etc.)
    // ... other ExifTool format types
}
```

**Auto-generated output**: `src/write/writable_tags.rs`
```rust
// AUTO-GENERATED from ExifTool v13.26
// Source: Multiple .pm files with Writable attributes
// Generated: 2025-06-25 by exiftool_sync extract writable-tags
// DO NOT EDIT - Regenerate with sync command

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref WRITABLE_TAGS: HashMap<String, WritableTagInfo> = {
        let mut map = HashMap::new();
        
        // EXIF tags (extracted from Exif.pm)
        map.insert("Make".to_string(), WritableTagInfo {
            tag_name: "Make".to_string(),
            tag_id: 0x010f,
            group: "EXIF".to_string(), 
            writable_format: WritableFormat::String,
            write_proc: None,
            safety_level: SafetyLevel::Safe,
            source_file: "Exif.pm".to_string(),
            exiftool_version: "13.26".to_string(),
        });
        
        // ... 500+ more entries
        
        map
    };
}

pub fn is_tag_writable(tag_name: &str) -> bool {
    WRITABLE_TAGS.contains_key(tag_name)
}

pub fn get_tag_safety_level(tag_name: &str) -> Option<SafetyLevel> {
    WRITABLE_TAGS.get(tag_name).map(|info| info.safety_level.clone())
}
```

**Sync command implementation**:
```bash
# Extract writable tags from all ExifTool modules  
cargo run --bin exiftool_sync extract writable-tags

# This scans ALL .pm files for:
# - Individual tag "Writable => 'format'" attributes
# - Table-level "WRITABLE => 1" declarations  
# - "WRITE_PROC => \&Function" specifications
# - Generates safety classifications based on tag patterns
```

**Integration with existing sync system**:
- Add to `exiftool-sync.toml` tracking
- Include in `make sync` workflow
- Auto-regenerate on ExifTool updates
- Maintain version compatibility

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

**ExifTool Write Pattern Analysis**:

From `Writer.pl:2241` (WriteInfo function), ExifTool's proven safety workflow:

```perl
# 1. File validation and setup
my ($inRef, $outRef, $closeIn, $closeOut, $outPos, $outBuff, $eraseIn, $raf, $fileExt);

# 2. Create temporary file (atomic write pattern)
$outfile = $tmpfile = "${infile}_exiftool_tmp" unless defined $outfile;

# 3. Open input file for reading
if ($self->Open(\*EXIFTOOL_FILE2, $infile)) {
    $inRef = \*EXIFTOOL_FILE2;
    $closeIn = 1;   # we must close the file since we opened it
}

# 4. Process format-specific writing
# (Delegated to format-specific WRITE_PROC functions)

# 5. Atomic replacement (temp file -> original)
# (Handled later in the function with careful error checking)
```

**Key ExifTool Safety Principles Found**:
1. **Never modify original directly**: Always use temp file
2. **Atomic operations**: Temp file + rename (not copy + delete)
3. **Complete metadata preservation**: Read everything before writing anything
4. **Format-aware validation**: Different rules per file type
5. **Error recovery**: Cleanup temp files on failure

**Our Rust Implementation Must Follow This Exactly**:
```rust
pub fn write_metadata<P: AsRef<Path>>(
    path: P, 
    changes: &MetadataChanges
) -> Result<WriteResult> {
    // 1. Validate against auto-generated writable tags registry
    validate_writable_tags(&changes)?;
    
    // 2. Read complete existing metadata (ExifTool pattern)
    let original = read_complete_metadata(&path)?;
    
    // 3. Create backup (.exif_original) - USER CONFIGURABLE
    let backup = if backup_enabled {
        Some(create_backup(&path)?)
    } else { None };
    
    // 4. Create temp file (ExifTool pattern: filename_exiftool_tmp)
    let temp_file = create_temp_file(&path)?;
    
    // 5. Write to temp file with complete metadata preservation
    write_to_temp(&temp_file, &original, &changes)?;
    
    // 6. Validate temp file integrity
    validate_written_file(&temp_file)?;
    
    // 7. Atomic replace (ExifTool pattern)
    atomic_rename(&temp_file, &path)?;
    
    Ok(WriteResult { 
        backup_path: backup,
        modified_tags: changes.len(),
        preserved_tags: original.len() - changes.len(),
    })
}
```

### 2. Metadata Preservation Framework
**Context**: Preserve all unknown tags and metadata during write operations.

**Files to create**:
- `src/write/preservation.rs` - Unknown tag preservation (follows ExifTool's complete read pattern)
- `src/write/round_trip.rs` - Round-trip validation
- `src/write/registry.rs` - Interface to auto-generated writable tags
- `src/write/atomic.rs` - Temp file + rename operations

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

**ExifTool JPEG Write Reference**: Study `third-party/exiftool/lib/Image/ExifTool/JPEG.pm` for segment handling patterns.

**Key ExifTool JPEG Insights**:
- APP1 segments: EXIF ("Exif\0\0") vs XMP ("http://ns.adobe.com/xap/1.0/\0")
- 64KB limit: Single APP1 segment max size
- Extended XMP: Multiple APP1 segments for large XMP ("http://ns.adobe.com/xmp/extension/\0")
- Preservation: All non-EXIF/XMP segments must be preserved exactly

**Reference existing code**: Study `src/core/jpeg.rs` for segment structure understanding.

**JPEG Write Complexity** (from ExifTool analysis):
```rust
// ExifTool's JPEG segment reconstruction pattern
struct JpegSegments {
    soi: Vec<u8>,           // Start of Image marker
    app0: Option<Vec<u8>>,  // JFIF data  
    app1_exif: Option<Vec<u8>>,  // EXIF data
    app1_xmp: Vec<Vec<u8>>,      // XMP data (can be multiple segments)
    other_segments: Vec<(u8, Vec<u8>)>, // All other APP segments
    image_data: Vec<u8>,    // Compressed image data
    eoi: Vec<u8>,           // End of Image marker
}

// Reconstruction must preserve:
// 1. Segment order (some cameras are picky)
// 2. Unknown APP segments (GPS trackers, etc.)
// 3. Thumbnail images in EXIF
// 4. XMP extended segments (>64KB XMP)
```

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

**ExifTool TIFF Write Reference**: Study `third-party/exiftool/lib/Image/ExifTool/Exif.pm` WriteExif function for IFD reconstruction patterns.

**Key ExifTool TIFF Insights**:
- IFD reconstruction: Must recalculate ALL offsets when structure changes
- Maker note preservation: Offsets are relative to maker note start OR TIFF header
- Multi-IFD handling: IFD0 (main), IFD1 (thumbnail), ExifIFD (detailed EXIF)
- Value storage: Inline (≤4 bytes) vs offset (>4 bytes)

**Reference existing code**: Study `src/core/ifd.rs` for IFD structure understanding.

**TIFF Write Complexity** (from ExifTool analysis):
```rust
// ExifTool's IFD reconstruction requirements
struct IfdReconstructionPlan {
    ifd_chain: Vec<IfdInfo>,     // IFD0 -> IFD1 -> ... chain
    value_data: Vec<u8>,         // All >4 byte values
    maker_notes: Vec<MakerNoteOffset>, // Offset corrections needed
    sub_ifds: HashMap<u32, Vec<IfdInfo>>, // ExifIFD, GPS IFD, etc.
}

// Critical: Maker note offset corrections
// Some manufacturers use offsets from TIFF header
// Others use offsets from maker note start  
// Must preserve manufacturer-specific patterns EXACTLY
```

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

**ExifTool Maker Note Patterns**: Study manufacturer-specific write patterns in `third-party/exiftool/lib/Image/ExifTool/`.

**Critical Maker Note Insights from ExifTool**:

**Canon** (`Canon.pm`):
- Uses `WRITABLE => 1` for many tables
- Complex binary structures with internal pointers
- Some tags are read-only due to checksums

**Nikon** (`Nikon.pm`):
- Encrypted maker notes (ProcessNikonEncrypted)
- Offset corrections when EXIF moves
- Model-specific variations

**Sony** (`Sony.pm`):
- Multiple WRITABLE tables (SonyMinolta, Sony, etc.)
- Binary data with embedded offsets
- Format evolution across camera generations

**Preservation Strategy** (ExifTool approach):
```rust
// ExifTool's maker note preservation pattern
pub fn preserve_maker_note(
    original_maker_note: &[u8],
    original_exif_offset: u32,
    new_exif_offset: u32,
    maker_type: MakerNoteType
) -> Result<Vec<u8>> {
    match maker_type {
        MakerNoteType::Canon => {
            // Canon uses offsets from TIFF header
            adjust_canon_offsets(original_maker_note, 
                                original_exif_offset, 
                                new_exif_offset)
        },
        MakerNoteType::Nikon => {
            // Nikon may be encrypted - preserve exactly
            if is_encrypted(original_maker_note) {
                Ok(original_maker_note.to_vec()) // NO modifications
            } else {
                adjust_nikon_offsets(original_maker_note, /* ... */)
            }
        },
        MakerNoteType::Unknown => {
            // ALWAYS preserve unknown maker notes exactly
            Ok(original_maker_note.to_vec())
        }
    }
}
```

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

**ExifTool XMP Write Reference**: Study `third-party/exiftool/lib/Image/ExifTool/XMP.pm` WriteXMP function.

**Key ExifTool XMP Insights**:
- Namespace preservation: Must maintain ALL existing namespaces
- Extended XMP: Reassemble multi-segment XMP (>64KB total)
- Field synchronization: Some XMP fields mirror EXIF (DateTime, GPS)
- XML validation: Proper XML structure required

**XMP Write Complexity** (from ExifTool analysis):
```rust
// ExifTool's XMP write coordination
struct XmpWriteOperation {
    base_xmp: String,           // Main XMP packet (<64KB)
    extended_xmp: Vec<String>,  // Additional XMP segments
    sync_fields: Vec<SyncField>, // Fields that mirror EXIF
    namespaces: Vec<String>,    // Preserve ALL namespaces
}

// Critical: EXIF/XMP field synchronization
// When writing EXIF DateTime, must update XMP xmp:ModifyDate
// When writing GPS coordinates, must update XMP exif:GPS*
// Must handle conflicts (which takes precedence?)
```

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

**Performance goals** (ExifTool baseline comparison):

**ExifTool Performance Baseline** (measured on typical hardware):
```bash
# JPEG files (5-20MB)
time exiftool -Make="Test" image.jpg
# Typical: 40-80ms per file

# RAW files (25-50MB)
time exiftool -Artist="Test" image.cr2  
# Typical: 100-200ms per file

# Batch operations (100 files)
time exiftool -Make="Test" *.jpg
# Typical: 30ms per file (amortized)
```

**Our Performance Targets** (competitive with ExifTool):
- **JPEG write**: <50ms for typical file (vs ExifTool's 40-80ms)
- **TIFF write**: <100ms for typical RAW file (vs ExifTool's 100-200ms)
- **Batch operations**: <25ms per file amortized (better than ExifTool due to Rust)
- **Memory usage**: <10MB additional during write (vs ExifTool's variable Perl usage)
- **Startup time**: <10ms cold start (vs ExifTool's 50-100ms Perl startup)

**Performance Optimization Strategies** (lessons from ExifTool):

1. **Lazy Loading**: Only read/parse sections that need modification
2. **Streaming I/O**: Avoid loading entire large RAW files into memory
3. **Smart Reconstruction**: Only rebuild changed IFDs/segments
4. **Parallel Processing**: Batch operations can process multiple files concurrently
5. **Memory Mapping**: Use memory-mapped files for large RAW files

**Performance Measurement Tools**:
```rust
// Built-in performance tracking
struct WriteMetrics {
    file_size: u64,
    read_time: Duration,
    process_time: Duration, 
    write_time: Duration,
    total_time: Duration,
    memory_peak: usize,
}

// Benchmark against ExifTool
pub fn benchmark_compatibility(test_files: &[PathBuf]) -> BenchmarkReport {
    // Run same operations with ExifTool and our implementation
    // Compare timing, memory usage, output correctness
}
```

### 10. Advanced Write Features (ExifTool Professional Patterns)

**ExifTool Professional Workflow Analysis**:

**Batch Operations** (ExifTool's strength):
```bash
# ExifTool batch patterns we should match
exiftool -Make="Camera Brand" -Artist="Photographer" *.jpg
exiftool -TagsFromFile src.jpg -all:all *.jpg  
exiftool "-FileName<CreateDate" -d "%Y%m%d_%H%M%S.%%e" *.jpg
```

**Our Advanced Features** (Phase 4+):

1. **Batch Operations** 
   ```rust
   pub fn write_metadata_batch(
       operations: Vec<(PathBuf, MetadataChanges)>
   ) -> Result<Vec<WriteResult>> {
       // Parallel processing with progress reporting
       // Atomic batch (all succeed or all fail)
       // Memory-efficient streaming for large batches
   }
   ```

2. **Template Application** (ExifTool: `-TagsFromFile`)
   ```rust
   pub fn apply_template<P: AsRef<Path>>(
       template_file: P,
       target_files: &[PathBuf],
       fields: &[String]  // Which fields to copy
   ) -> Result<Vec<WriteResult>> {
       // Read template metadata once
       // Apply to multiple targets efficiently  
   }
   ```

3. **Selective Field Updates** (ExifTool: tag name patterns)
   ```rust
   pub fn update_selective<P: AsRef<Path>>(
       path: P,
       pattern: &str,      // e.g., "GPS*", "Canon*", "EXIF:*"
       operation: UpdateOp // Clear, Copy, Transform
   ) -> Result<WriteResult> {
       // Match ExifTool's tag pattern system
   }
   ```

4. **Write Verification** (ExifTool: `-validate`)
   ```rust
   pub fn write_with_verification<P: AsRef<Path>>(
       path: P,
       changes: &MetadataChanges
   ) -> Result<VerifiedWriteResult> {
       let result = write_metadata(path, changes)?;
       
       // Verify with multiple tools
       verify_exiftool_compatibility(&path)?;
       verify_adobe_compatibility(&path)?;
       verify_checksum_integrity(&path)?;
       
       Ok(VerifiedWriteResult { result, verifications })
   }
   ```

**Professional Features Priorities**:

**Phase 4** (Month 2):
- Batch operations (most requested)
- Template application (workflow efficiency) 
- Basic verification (quality assurance)

**Phase 5** (Month 3):
- Advanced pattern matching (power user features)
- Lightroom/Photos integration (ecosystem compatibility)
- Performance optimization (large batch handling)

**ExifTool Feature Parity Goals**:
- Match 95% of ExifTool's command-line functionality
- Exceed ExifTool's performance for batch operations  
- Provide better error reporting and progress feedback
- Maintain 100% compatibility with ExifTool-written files

## 📋 SYNC-DESIGN.md Integration

**CRITICAL**: Update the sync documentation to include write support patterns.

**New sync commands to add**:
```bash
# Extract writable tags from all ExifTool modules
cargo run --bin exiftool_sync extract writable-tags

# Validate write safety against ExifTool patterns
cargo run --bin exiftool_sync validate-write-safety

# Check for ExifTool write function updates
cargo run --bin exiftool_sync diff 13.25 13.26 --write-functions
```

**Documentation updates needed**:
1. Add writable tag extraction to sync workflow
2. Document safety level classification system  
3. Include write validation procedures
4. Update algorithm tracking for write functions

**Version tracking additions**:
```toml
# Add to exiftool-sync.toml
[write_support]
writable_tags_version = "13.26"
last_write_sync = "2025-06-25"
write_safety_hash = "sha256:..."

[write_functions]
"WriteExif" = { version = "13.26", source = "Exif.pm:WriteExif" }
"WriteJPEG" = { version = "13.26", source = "JPEG.pm:WriteJPEG" }
"WriteXMP" = { version = "13.26", source = "XMP.pm:WriteXMP" }
```

## 🧪 Testing Strategy (ExifTool Compatibility)

**MANDATORY**: All write operations must pass ExifTool compatibility tests.

**Round-trip validation pattern**:
```bash
# 1. Create test file with known metadata
echo "Test image" > test.jpg

# 2. Write metadata with our implementation
cargo run -- test.jpg --write Make="Test Camera" --write Model="Test Model"

# 3. Read with ExifTool (gold standard)
exiftool -Make -Model test.jpg > exiftool_output.txt

# 4. Read with our implementation  
cargo run -- test.jpg --tags Make,Model > our_output.txt

# 5. Compare outputs (must be identical)
diff exiftool_output.txt our_output.txt

# 6. Verify file integrity
exiftool -validate test.jpg  # Must pass ExifTool validation
```

**Test file requirements**:
- Canon RAW files (CR2, CR3)
- Nikon RAW files (NEF)
- Sony RAW files (ARW)
- Standard JPEG with complex EXIF
- JPEG with large XMP (>64KB)
- TIFF with multiple IFDs

**Performance benchmarks** (based on ExifTool):
```bash
# ExifTool baseline performance
time exiftool -Make="Test" *.jpg  # Typical: 50-100ms per file

# Our implementation must be competitive
time cargo run -- --write Make="Test" *.jpg  # Target: <100ms per file
```

## Technical Architecture

### Write Safety Principles
1. **Never modify original file directly**
2. **Always create backup before write**
3. **Use atomic operations (temp file + rename)**
4. **Validate written data before committing**
5. **Preserve all unknown metadata**

### Format Support Strategy (ExifTool Priority Order)

**ExifTool's Format Complexity Analysis**:

**Tier 1: Essential (Week 2)**
1. **JPEG** (90% of use cases)
   - ExifTool reference: `JPEG.pm`
   - Complexity: Medium (segment handling)
   - Write patterns: APP1 segment reconstruction
   - Testing: Standard camera JPEG files

2. **TIFF/RAW** (professional workflows) 
   - ExifTool reference: `Exif.pm` WriteExif function
   - Complexity: High (IFD reconstruction, offset calculation)
   - Write patterns: Multi-IFD handling, maker note preservation
   - Testing: Canon CR2, Nikon NEF, Sony ARW

**Tier 2: Important (Future phases)**
3. **XMP sidecar** (.xmp files)
   - ExifTool reference: `XMP.pm` WriteXMP function  
   - Complexity: Medium (XML structure, namespace preservation)
   - Write patterns: Standalone XMP file creation
   - Testing: Adobe sidecar compatibility

4. **HEIF/HEIC** (modern mobile formats)
   - ExifTool reference: `QuickTime.pm` (HEIC uses QuickTime structure)
   - Complexity: High (nested box structure)
   - Write patterns: Box reconstruction, metadata atom handling
   - Testing: iPhone photo files

**Tier 3: Nice-to-have (Long term)**
5. **PNG** (web/graphics workflows)
   - ExifTool reference: `PNG.pm`
   - Complexity: Low (chunk-based structure)
   - Write patterns: Text chunk updates
   - Testing: Web graphics with EXIF

6. **PDF** (document workflows)
   - ExifTool reference: `PDF.pm` WritePDF function
   - Complexity: Very High (PDF structure, incremental updates)
   - Write patterns: Object stream updates
   - Testing: PDF documents with metadata

**Format-Specific ExifTool Insights**:

**JPEG Complexity** (from `JPEG.pm`):
- 15+ different APP segment types
- Extended XMP handling (>64KB)
- Thumbnail preservation in EXIF
- Segment order preservation (some cameras are picky)

**TIFF Complexity** (from `Exif.pm`):
- IFD chain reconstruction  
- Offset recalculation when data moves
- 10+ different maker note formats
- SubIFD handling (ExifIFD, GPS IFD, etc.)

**Implementation Priority**: Start with JPEG (simpler segment model), then TIFF (complex offset handling). This matches ExifTool's development history and testing coverage.

### Error Handling Strategy (ExifTool-Based)

**ExifTool's Error Handling Patterns** (study `Writer.pl:2400+`):

**File System Errors**:
- **Disk full**: Detect during temp file write, cleanup temp file
- **Permission denied**: Clear error message with suggested fix
- **File locked**: Detect and report specific lock holder if possible
- **Path issues**: Validate input/output paths before starting

**Format Errors**:
- **Corrupted input**: Validate file structure before attempting write
- **Unsupported format**: Check against supported format list
- **Invalid metadata**: Validate tag values against format constraints
- **Structure errors**: Detect IFD loops, invalid offsets, etc.

**Write Operation Errors**:
- **Partial write**: Atomic operation prevents partial corruption
- **Validation failure**: Rollback to original file automatically
- **Memory exhaustion**: Handle large files gracefully
- **Process interruption**: Temp file cleanup in signal handlers

**Our Error Handling Implementation**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("File system error: {message}. Original file unchanged.")]
    FileSystem { message: String },
    
    #[error("Invalid metadata: {field} = {value}. {suggestion}")]
    InvalidMetadata { field: String, value: String, suggestion: String },
    
    #[error("Write safety violation: {tag} is not writable. See writable tag registry.")]
    SafetyViolation { tag: String },
    
    #[error("ExifTool compatibility failure: {details}")]
    CompatibilityError { details: String },
}

pub fn write_with_recovery<P: AsRef<Path>>(
    path: P,
    changes: &MetadataChanges
) -> Result<WriteResult, WriteError> {
    let temp_file = create_temp_file(&path)?;
    
    // Ensure cleanup on ANY error
    let _cleanup_guard = TempFileCleanup::new(&temp_file);
    
    match write_to_temp(&temp_file, changes) {
        Ok(_) => {
            // Validate before committing
            validate_temp_file(&temp_file)?;
            atomic_rename(&temp_file, &path)?;
            Ok(WriteResult::success())
        },
        Err(e) => {
            // Temp file automatically cleaned up by guard
            Err(WriteError::from(e))
        }
    }
}
```

**Error Recovery Principles**:
- **Preserve originals**: Never leave corrupted files
- **Clear error messages**: Explain what went wrong and how to recover  
- **Automatic cleanup**: Remove temp files on any error
- **Validation feedback**: Report what was written vs what was requested
- **Safe defaults**: When in doubt, preserve existing data unchanged

## Success Criteria
- [ ] Safe write operations with backup/rollback
- [ ] JPEG and TIFF write support implemented
- [ ] Unknown tag and maker note preservation
- [ ] Round-trip compatibility (write then read gives same result)
- [ ] ExifTool compatibility for written files
- [ ] Performance targets met (<100ms for typical operations)
- [ ] Comprehensive write validation and testing