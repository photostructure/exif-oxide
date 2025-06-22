# Implementation Learnings

This document captures important discoveries and gotchas encountered during development.

## Spike 1 Learnings

### JPEG Parsing

1. **Segment Length Includes Itself**: JPEG segment lengths include the 2 bytes used for the length field itself. Always subtract 2 from the segment length to get the actual data size.

2. **EOI vs SOS**: End of Image (0xD9) and Start of Scan (0xDA) both indicate no more metadata follows, but handle them differently:

   - EOI: End of file
   - SOS: Image data follows (no more metadata)

3. **Marker Padding**: JPEG markers can be padded with unlimited 0xFF bytes. The parser must consume all 0xFF bytes until finding a non-0xFF marker byte.

4. **APP1 Size Limit**: APP1 segments have a 64KB size limit (65533 bytes of data after the length field).

### EXIF/TIFF Structure

1. **Byte Order Markers**:

   - "II" (0x4949) = Little-endian (Intel)
   - "MM" (0x4D4D) = Big-endian (Motorola)
   - Magic number is always 42 (0x002A or 0x2A00 depending on endianness)

2. **IFD Entry Values**:

   - If value size ≤ 4 bytes, it's stored inline in the offset field
   - If value size > 4 bytes, the offset field contains a pointer to the data
   - This is why we need to check `format.size() * count` to determine storage location

3. **String Handling**:
   - EXIF strings are null-terminated but the buffer may contain garbage after the null
   - Some cameras pad with spaces instead of nulls
   - Always scan for null terminator or use the entire count length

### Testing Discoveries

1. **ExifTool's Test Images**: The file `ExifTool.jpg` actually contains FUJIFILM EXIF data, not Canon data as ExifTool reports. This suggests ExifTool may use additional metadata sources or override mechanisms beyond just the EXIF data.

2. **IFD0 vs IFD1**:
   - IFD0 contains main image metadata
   - IFD1 contains thumbnail metadata
   - Some images may have contradictory data between IFDs

### Rust-Specific Learnings

1. **Error Handling**: Using `thiserror` makes error types much cleaner and automatically implements std::error::Error.

2. **Binary Parsing Options**:

   - `byteorder` crate is simple and efficient for basic endian handling
   - `nom` provides powerful parser combinators but may be overkill for simple binary formats
   - For Spike 1, direct parsing was sufficient and more transparent

3. **Testing Patterns**:
   - Unit tests with constructed binary data help verify parsing logic
   - Integration tests with real files catch edge cases
   - Keep test data minimal but representative

### Performance Considerations

1. **Memory Usage**:

   - Don't load entire files into memory - use streaming where possible
   - For EXIF, the APP1 segment is typically small (<64KB) so loading it is fine

2. **Bounds Checking**:
   - Always verify offsets before accessing data
   - Skip malformed entries rather than failing entirely
   - This matches ExifTool's robust parsing approach

## Tag Table Strategy

### Original Design vs Implementation

The original design called for leveraging ExifTool's tag tables, but Spike 1 hard-coded the few tags needed. This was a shortcut that doesn't scale.

### Proper Table-Driven Approach

1. **Parse ExifTool's Perl modules** to extract tag definitions
2. **Generate Rust code** with tag tables at build time
3. **Use lookup tables** instead of hard-coded matches

Example tag definition from ExifTool:

```perl
0x10f => {
    Name => 'Make',
    Groups => { 2 => 'Camera' },
    Writable => 'string',
    WriteGroup => 'IFD0',
    DataMember => 'Make',
}
```

This contains valuable metadata:

- Tag ID (0x10f)
- Human-readable name
- Group/category information
- Data type and writability
- Special handling flags

### Benefits of Table-Driven Approach

1. **Maintainability**: Update tables when ExifTool adds new tags
2. **Completeness**: Access to 25 years of camera quirks
3. **Consistency**: Same tag names as ExifTool
4. **Extensibility**: Easy to add manufacturer-specific tables

### ExifTool Source Navigation

Key files in the ExifTool source (`exiftool/lib/Image/ExifTool/`):

- `Exif.pm` - Standard EXIF tags (IFD0, IFD1, ExifIFD, etc.)
- `Canon.pm`, `Nikon.pm`, etc. - Manufacturer-specific tags
- `GPS.pm` - GPS IFD tags
- `ExifTool.pm` - Core functionality including ProcessBinaryData

Tag definitions start around line 700-800 in most files, after the initial documentation.

## Spike 1.5 Learnings

### Table Generation Architecture

**Key Discovery:** ExifTool's Perl source is highly parseable with regex-based approaches.

1. **Perl Hash Structure Patterns**
   ```perl
   # Pattern 1: Simple assignment
   0x10f => 'Make',
   
   # Pattern 2: Hash with metadata
   0x10f => {
       Name => 'Make',
       Groups => { 2 => 'Camera' },
       Writable => 'string',
   }
   ```

2. **Regex Strategy That Works**
   ```rust
   // Multi-line tag definitions
   let tag_re = Regex::new(r"(?s)(0x[0-9a-fA-F]+)\s*=>\s*\{([^}]+)\}").unwrap();
   
   // Simple string assignments  
   let simple_tag_re = Regex::new(r"(0x[0-9a-fA-F]+)\s*=>\s*'([^']+)',").unwrap();
   ```

3. **Build Script Integration**
   - Use `build.rs` to run parsing at compile time
   - Generate code in `$OUT_DIR/generated_tags.rs`
   - Include generated code with `include!(concat!(env!("OUT_DIR"), "/generated_tags.rs"))`

### ExifTool Format Mapping

**Critical Insight:** ExifTool's format names map predictably to EXIF specification formats:

```rust
// ExifTool → Our enum mapping
"string"       → ExifFormat::Ascii
"int8u"        → ExifFormat::U8
"int16u"       → ExifFormat::U16
"int32u"       → ExifFormat::U32
"int16s"       → ExifFormat::I16
"int32s"       → ExifFormat::I32
"rational64u"  → ExifFormat::Rational
"rational64s"  → ExifFormat::SignedRational
"undef"        → ExifFormat::Undefined
```

### Value Type System Design

**Rust Pattern:** Use enums for type-safe value representation with array support:

```rust
pub enum ExifValue {
    // Single values
    Ascii(String),
    U16(u16), U32(u32),
    Rational(u32, u32),  // (numerator, denominator)
    
    // Array values  
    U16Array(Vec<u16>),
    RationalArray(Vec<(u32, u32)>),
    
    // Fallback
    Undefined(Vec<u8>),
}
```

**Design Decision:** Separate single values from arrays in enum variants rather than using `Vec<T>` everywhere. This makes common single-value access more ergonomic.

### Parsing Edge Cases

1. **Rational Number Validation**
   - Always check for zero denominators
   - ExifTool data is generally well-formed, but validation is still important

2. **Group Hierarchy**
   - ExifTool uses 3-level grouping: family (0), specific (1), category (2)
   - Category (group 2) is most useful for user organization
   - Examples: "Camera", "Time", "Author", "Location"

3. **Missing Information Handling**
   - Many tags have no explicit `Writable` field → default to unknown format
   - Unknown formats get parsed as raw bytes (Undefined)
   - Parser continues gracefully rather than failing

### Development Tooling

**Best Practice:** Create debugging tools alongside core functionality:

```rust
// Development binary for exploring generated tables
cargo run --bin parse_exiftool_tags

// Output:
// === Camera Tags (33) ===
//   0x010F - Make                      Format: string
//   0x0110 - Model                     Format: string
```

This tool proved invaluable for:
- Debugging regex patterns
- Understanding ExifTool's structure  
- Validating format mappings
- Exploring tag organization

### Rust Ownership Patterns

**Common Pattern:** Collect statistics before consuming data structures:

```rust
// Collect count before consuming in for loop
let group_count = by_group.len();

// Now safe to consume by_group
for (group, mut group_tags) in by_group {
    // Process groups...
}

// Can still use group_count here
println!("Groups: {}", group_count);
```

**Iterator Chaining:** Use iterator methods for data processing:

```rust
tags.sort_by_key(|t| t.tag_id);       // Sort by tag ID
tags.dedup_by_key(|t| t.tag_id);      // Remove duplicates  
group_tags.take(10)                   // Show first 10
values.any(|v| matches!(v, ExifValue::Undefined(_)))  // Check for any
```

### Testing Strategy Evolution

**Lesson:** Layer tests from unit → integration → real-world:

1. **Unit Tests** - Synthetic binary data for edge cases
2. **Table Tests** - Verify generated lookup tables  
3. **Integration Tests** - Real ExifTool test images
4. **Format Tests** - All EXIF value types with real data

**Key Insight:** Testing with ExifTool's own test images (`exiftool/t/images/`) provides excellent coverage of real-world edge cases.

### Performance Characteristics

**Zero-Cost Abstractions Achieved:**
- Static lookup tables with no runtime overhead
- O(1) tag lookup via linear search over ~500 items (cache-friendly)
- No dynamic allocation for table access
- Generated code optimizes well

**Memory Usage:**
- Static table: ~40KB for 496 tags  
- No heap allocation for lookups
- Parsed values only allocated on demand

### Format Type Mapping

ExifTool format types → Our ExifFormat enum:

- `'string'` → `ExifFormat::Ascii`
- `'int16u'` → `ExifFormat::U16`
- `'int32u'` → `ExifFormat::U32`
- `'rational64u'` → `ExifFormat::Rational`
- `'int16s'` → `ExifFormat::I16`
- `'int32s'` → `ExifFormat::I32`
- `'rational64s'` → `ExifFormat::SignedRational`
- `'undef'` → `ExifFormat::Undefined`
- `'binary'` → `ExifFormat::Undefined`

Note: Some tags don't specify a format - these default to the format found in the actual file.

### Complex Tag Features (Skip in Spike 1.5)

These features should be documented but skipped initially:

1. **Conditional Tags**:

   ```perl
   Condition => '$$self{Make} =~ /Canon/',
   ```

2. **PrintConv Functions**:

   ```perl
   PrintConv => {
       1 => 'Horizontal',
       3 => 'Rotate 180',
       6 => 'Rotate 90 CW',
       8 => 'Rotate 270 CW',
   },
   ```

3. **Binary Data Structures**:

   ```perl
   SubDirectory => {
       TagTable => 'Image::ExifTool::Canon::CameraSettings',
       ProcessProc => \&ProcessBinaryData,
   },
   ```

4. **Dynamic Tags**:
   ```perl
   ValueConv => '$val / 1000',
   ValueConvInv => '$val * 1000',
   ```

## Spike 2 Learnings

### Maker Note Architecture

**Key Discovery:** Maker notes are typically stored in the ExifIFD (tag 0x8769), not in IFD0.

1. **ExifIFD Sub-Directory Structure**
   ```
   IFD0 → contains tag 0x8769 (ExifOffset) pointing to ExifIFD
   ExifIFD → contains tag 0x927c (MakerNotes) with manufacturer data
   ```

2. **Canon Maker Note Format**
   - Uses standard IFD structure (same as main EXIF)
   - No complex header or signature required
   - Uses same byte order as main EXIF data
   - May have 8-byte footer with offset information

3. **Structural Tag Issues**
   - Tag 0x8769 (ExifOffset) incorrectly defined as Ascii in ExifTool
   - Must override to U32 format for proper parsing
   - Other structural tags: 0x8825 (GPSOffset), 0x014A (SubIFDs)

### Table Generation Extension

**Pattern:** Canon.pm follows similar structure to Exif.pm but with fewer main tags.

1. **Canon Tag Parsing**
   ```perl
   # Canon Main table starts around line 1183
   %Image::ExifTool::Canon::Main = (
       0x1 => { Name => 'CanonCameraSettings', SubDirectory => {...} },
       0x6 => { Name => 'CanonImageType', Writable => 'string' },
       # ...
   );
   ```

2. **Build System Integration**
   - Extended build.rs to parse both Exif.pm and Canon.pm
   - Generated separate CANON_TAGS lookup table
   - 34 Canon tags successfully parsed vs 496 EXIF tags
   - Skipped SubDirectory tags (too complex for initial implementation)

3. **Lookup Function Pattern**
   ```rust
   pub fn lookup_canon_tag(tag_id: u16) -> Option<&'static TagInfo>
   pub fn lookup_tag(tag_id: u16) -> Option<&'static TagInfo>  // EXIF
   ```

### IFD Parser Enhancements

**Critical Fix:** Extended IFD parser to handle sub-directories.

1. **ExifIFD Integration**
   ```rust
   // Parse IFD0 first
   let mut ifd0 = Self::parse_ifd(&data, &header, ifd_offset)?;
   
   // Check for ExifIFD and merge entries
   if let Some(ExifValue::U32(exif_ifd_offset)) = ifd0.entries.get(&0x8769) {
       let exif_ifd = Self::parse_ifd(&data, &header, exif_ifd_offset)?;
       // Merge ExifIFD entries into IFD0
   }
   ```

2. **Structural Tag Override**
   ```rust
   let actual_format = match tag {
       0x8769 => ExifFormat::U32, // ExifOffset
       0x8825 => ExifFormat::U32, // GPSOffset  
       0x014A => ExifFormat::U32, // SubIFDs
       _ => lookup_tag(tag).map(|t| t.format).unwrap_or(format)
   };
   ```

### Canon Maker Note Parsing

**Success:** Successfully parsed 28 Canon tags from Canon1DmkIII.jpg.

1. **Parsing Results**
   - Total maker note size: 6962 bytes
   - Tags extracted: 28 (vs ExifTool's 36)
   - All data stored as Undefined format initially
   - No parsing errors or crashes

2. **Manufacturer Detection**
   ```rust
   let manufacturer = Manufacturer::from_make("Canon");
   // Returns: Manufacturer::Canon
   
   let parser = manufacturer.parser(); 
   // Returns: Some(Box<dyn MakerNoteParser>)
   ```

3. **Tag Prefixing System**
   - Maker note tags prefixed with 0x8000 to avoid conflicts
   - Canon tag 0x0001 becomes 0x8001 in final output
   - Allows clean separation of EXIF vs maker note data

### Testing Strategy

**Approach:** Use ExifTool's test images for validation.

1. **Test Image Analysis**
   - Canon.jpg: Basic test image, no maker notes
   - Canon1DmkIII.jpg: Professional camera with full maker notes
   - Real-world validation against ExifTool verbose output

2. **Debug Methodology**
   ```rust
   eprintln!("DEBUG: IFD at offset {}: {} entries", offset, entry_count);
   eprintln!("DEBUG: Tag 0x{:04x}, format {}, count {}", tag, format_code, count);
   ```

3. **Comparison Results**
   - ExifTool: "MakerNoteCanon (SubDirectory) --> 36 entries"
   - Our parser: "Successfully parsed Canon maker notes! Found 28 tags"
   - 78% coverage achieved on first implementation

### Architecture Validation

**Confirmed:** Table-driven approach scales well to manufacturer-specific parsing.

1. **Modular Design Success**
   - Trait-based parser system works for multiple manufacturers
   - Clean separation: detection → dispatch → parsing
   - Easy to add new manufacturers (Nikon, Sony, etc.)

2. **Performance Characteristics**
   - No measurable performance impact from maker note parsing
   - Static table lookups remain O(1)
   - Memory usage stays minimal

### Next Steps Identified

1. **Enhanced Tag Parsing**
   - Parse binary data structures (ProcessBinaryData equivalent)
   - Handle Canon CameraSettings sub-structure
   - Add human-readable value conversions

2. **Other Manufacturers**
   - Nikon maker notes (different format, has signature)
   - Sony maker notes (encrypted sections)
   - Fujifilm, Olympus, Panasonic support

3. **Sub-IFD Support**
   - GPS IFD (tag 0x8825)
   - Interoperability IFD (tag 0xA005)
   - Thumbnail IFD (IFD1)

## Spike 3 Learnings

### Binary Tag Extraction (Preview Images & Thumbnails)

**Key Discovery:** Thumbnails are stored in IFD1 but with important parsing nuances.

1. **IFD1 Parsing Critical**
   - Thumbnails stored in IFD1 (next IFD after IFD0)
   - Tags 0x201 (ThumbnailOffset) and 0x202 (ThumbnailLength)
   - Must follow next IFD pointer from IFD0 to find IFD1

2. **Format Handling Issues**
   ```
   Expected: ThumbnailOffset as U32
   Reality: Often stored as Undefined format with raw bytes
   Solution: Flexible parsing (get_numeric_u32) handles multiple formats
   ```

3. **Offset Interpretation**
   - Parsed offset may point to data structure containing JPEG
   - Common pattern: 12-byte header before actual JPEG data  
   - Must search for JPEG SOI marker (0xFFD8) within offset area
   - Example: Parsed offset 8916 → actual JPEG at 8928 (+12 bytes)

4. **JPEG Boundary Detection**
   ```rust
   // Find JPEG start
   let jpeg_start = data.windows(2).position(|w| w == [0xFF, 0xD8]);
   
   // Find JPEG end (EOI marker)  
   let jpeg_end = data.windows(2).position(|w| w == [0xFF, 0xD9]);
   ```

5. **Real-world Validation Results**
   - **Canon T3i JPG**: 15,693 bytes extracted successfully
   - **Nikon Z8 JPG**: 11,855 bytes (matches ExifTool exactly)
   - **Sony A7C II JPG**: 10,857 bytes  
   - **Panasonic G9 II JPG**: 5,024 bytes
   - **Performance**: <8ms extraction time

6. **Cross-manufacturer Compatibility**
   - Same thumbnail extraction logic works across Canon, Nikon, Sony, Panasonic
   - IFD1 structure is standardized across manufacturers
   - Format variations handled by flexible numeric parsing

### Canon Maker Note Challenges

**Current Status:** Canon preview tags detected but not yet extracting valid data.

1. **Canon Preview Tag Detection**
   - Tags 0xB602 (PreviewImageLength), 0xB605 (PreviewImageStart) found
   - Generated lookup table includes Canon-specific preview tags
   - Build system successfully parses Canon.pm preview definitions

2. **Extraction Issues to Resolve**
   - Canon preview tags not yielding valid JPEG data
   - May require different offset interpretation than thumbnails
   - Possible sub-directory structure within maker notes

### Testing Strategy Validation

**Comprehensive Real-world Testing Approach:**

1. **Multi-manufacturer Coverage**
   - ExifTool canonical test images: Canon.jpg, Canon1DmkIII.jpg
   - User-provided real images: Canon, Nikon, Sony, Panasonic
   - Modern formats: R5 Mark II, Z8, A7C II

2. **Format Validation**
   ```rust
   // JPEG validation checks both SOI and EOI markers
   fn validate_jpeg(data: &[u8]) -> bool {
       has_soi && has_eoi // 0xFFD8 start, 0xFFD9 end
   }
   ```

3. **Performance Benchmarking**
   - Target: <5ms per extraction
   - Achieved: <8ms (acceptable for real-world usage)
   - Memory efficient: No loading entire files unnecessarily

### Architecture Insights

**Table-driven Approach Scales Well:**

1. **Flexible Parsing Strategy**
   ```rust
   pub fn get_numeric_u32(&self, tag: u16) -> Option<u32> {
       // Handles U32, U16, U8, Undefined formats
       // Critical for real-world format variations
   }
   ```

2. **Modular Design Success**
   - `src/extract/thumbnail.rs` - Standard EXIF thumbnails
   - `src/extract/preview.rs` - Manufacturer-specific previews  
   - `src/extract/mod.rs` - Unified API

3. **API Design**
   ```rust
   // High-level functions work with file paths
   pub fn extract_thumbnail(path) -> Result<Option<Vec<u8>>>
   pub fn extract_largest_preview(path) -> Result<Option<Vec<u8>>>
   ```

### Critical Implementation Details

1. **IFD1 Prefixing**
   - IFD1 tags prefixed with 0x1000 to avoid IFD0 conflicts
   - ThumbnailOffset becomes 0x1201, ThumbnailLength becomes 0x1202

2. **Byte Order Awareness**
   - Thumbnail offset/length must use file's byte order
   - Little-endian most common (Canon, Nikon)
   - get_numeric_u32 defaults to little-endian for Undefined data

3. **Error Handling Strategy**
   ```rust
   // Graceful degradation - return None rather than panic
   // Continue parsing other data if thumbnail extraction fails
   ```

## Future Considerations

### Next Steps for Canon Previews

1. **Canon Maker Note Sub-structure Analysis**
   - PreviewImageInfo (tag 0xB6) may contain sub-directory
   - Need to handle Canon's ProcessBinaryData equivalent
   - May require offset correction relative to maker note start

2. **Other Manufacturer Preview Support**
   - Nikon maker notes (different format, signatures)
   - Sony encrypted sections
   - Fujifilm, Olympus preview structures

### Sub-IFDs

- EXIF IFD (tag 0x8769) ✅ Implemented  
- GPS IFD (tag 0x8825)
- Interoperability IFD (tag 0xA005)

### Maker Notes

- Vendor-specific binary blobs ✅ Canon structure identified
- Often use relative offsets from maker note start
- May be encrypted or obfuscated
- Require manufacturer-specific parsing ✅ Architecture in place

### Extended Tag Support

- Rational types need numerator/denominator parsing ✅ Implemented
- Some tags have special PrintConv (print conversion) functions
- Tag conditions may depend on other tag values
