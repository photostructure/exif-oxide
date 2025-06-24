# Engineering Learnings & Tribal Knowledge

This document captures critical discoveries, gotchas, and hard-won knowledge for engineers working on exif-oxide. It's organized by topic for quick reference during development.

## 🚨 Critical Gotchas & Common Pitfalls

### Camera Standards Are Chaos

- Camera manufacturers have played fast and loose with standards
- ExifTool has figured it out -- always find the relevant code in `$REPO_ROOT/vendored/exiftool` and ensure we're following their algorithms
- If we invent any heuristics, they **most likely won't be correct**
- What seems like a bug might be a workaround for a 2003 camera model
- **Never invent parsing heuristics** - defer to ExifTool's algorithms as verbatim as possible

### Binary Parsing Safety

- **ALWAYS** bounds-check before reading - use `get()` not indexing
- Offsets are from TIFF header start, not file start
- Next IFD offset of 0xFFFFFFFF means "no next IFD"
- Value fits inline if `size × count ≤ 4 bytes`

### String Handling Traps

- EXIF strings are null-terminated BUT buffer may contain garbage after null
- Some manufacturers pad with spaces instead of nulls
- UTF-8 not guaranteed - may need charset detection
- Always scan for null terminator or use full count length

### JPEG Segment Parsing

- **Segment length includes itself** - subtract 2 for actual data size
- APP1 segments limited to 64KB (65533 bytes after length field)
- Markers can be padded with unlimited 0xFF bytes
- Must check for "Exif\0\0" signature, not just APP1 presence

### Performance Land Mines

- Don't load entire files into memory - use streaming
- Skip malformed entries rather than failing entirely
- Bounds check all offsets before accessing data
- Pre-compile regex patterns with `lazy_static!`

## 📊 Format-Specific Knowledge

### JPEG Structure

- **Byte Order**: TIFF header ~12 bytes into file after "Exif\0\0" marker
- **Multiple Segments**: Can have both EXIF (APP1) and XMP (APP1) segments
- **MPF Support**: Modern cameras use APP2 segments for Multi-Picture Format
- **EOI vs SOS**: 0xD9 = End of Image, 0xDA = Start of Scan (no more metadata)
- **Padding**: Files often have padding after EOI marker - search last 32 bytes

### TIFF/RAW Structure

- **Endianness Detection**: "II" = little-endian, "MM" = big-endian
- **Magic Number**: Always 42 (0x002A or 0x2A00)
- **IFD Chains**: Next IFD offset of 0xFFFFFFFF = end of chain
- **Offset Calculation**: TIFF header at byte 0, all offsets relative to it
- **Memory Modes**: Full file vs metadata-only (90% memory savings)

### PNG Structure

- **eXIf Chunks**: Contains raw TIFF/EXIF data (no wrapper)
- **Parsing Optimization**: Stop at IDAT (image data) chunks
- **Chunk Validation**: Length + CRC validation required

### Container Formats

- **RIFF (WebP/AVI)**: Little-endian, word-aligned chunks, 4-byte WebP padding
- **QuickTime/MP4**: Big-endian atoms, size 0 = "to end of file"
- **Atom Structure**: 32-bit or 64-bit sizes, hierarchical nesting

## 🏗️ Architecture Patterns

### Table-Driven Design

**Success Pattern**: ExifTool Perl → Generated Rust tables

- Parse ExifTool's Perl modules at build time (build.rs)
- Generate static lookup tables for O(1) tag access
- 496 EXIF tags + manufacturer-specific tables
- Zero runtime overhead, cache-friendly linear search

### Format Detection & Dispatch

**Central Pattern**: Single `find_metadata_segment()` function

```rust
match format {
    FileType::JPEG => jpeg::find_exif_segment(reader)?,
    FileType::PNG => png::find_exif_chunk(reader)?,
    FileType::TIFF | FileType::CR2 => tiff::find_ifd_data(reader)?,
}
```

### Error Handling Philosophy

**Graceful Degradation**: Continue parsing on errors

- Return `Option<T>` instead of failing hard
- Collect warnings, don't stop execution
- Skip malformed entries, continue with rest
- Matches ExifTool's robust approach

### Value Type System

**Type Safety**: Enum-based values with array support

```rust
pub enum ExifValue {
    Ascii(String), U16(u16), U32(u32),
    Rational(u32, u32),  // (numerator, denominator)
    U16Array(Vec<u16>), RationalArray(Vec<(u32, u32)>),
    Undefined(Vec<u8>),  // Fallback
}
```

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

## Spike 4 Learnings

### XMP Architecture and Parsing

**Key Discovery:** XMP requires fundamentally different parsing approach than binary EXIF data.

1. **XMP Packet Detection in JPEG**

   - XMP stored in APP1 segments with signature `"http://ns.adobe.com/xap/1.0/\0"`
   - Different from EXIF APP1 segments (`"Exif\0\0"`)
   - Can coexist with EXIF in same JPEG file
   - Segments can exceed 64KB (Extended XMP support needed)

2. **XML Parser Selection**

   - **quick-xml** chosen for performance and streaming capability
   - Event-driven parsing allows handling very large XMP packets
   - Memory efficient - doesn't load entire DOM
   - Handles malformed XML gracefully

3. **UTF-16 Encoding Discovery**
   ```rust
   // Automatic encoding detection
   if data.len() >= 2 && data[0] == 0x00 { // UTF-16 BE
   } else if data.len() >= 2 && data[1] == 0x00 { // UTF-16 LE
   } else { // UTF-8
   }
   ```

### Hierarchical Data Model

**Architecture Decision:** Use enum-based value system for type safety.

1. **XmpValue Design**

   ```rust
   pub enum XmpValue {
       Simple(String),                      // Basic properties
       Array(XmpArray),                     // RDF containers
       Struct(HashMap<String, XmpValue>),   // Nested properties
   }
   ```

2. **RDF Container Support**

   - **rdf:Seq** (Ordered) - Sequence matters (e.g., creation workflow steps)
   - **rdf:Bag** (Unordered) - Set of values (e.g., keywords)
   - **rdf:Alt** (Alternative) - Language alternatives (e.g., title in multiple languages)

3. **Language Alternative Handling**
   ```rust
   pub struct LanguageAlternative {
       pub language: Option<String>,  // xml:lang attribute
       pub value: XmpValue,
   }
   ```

### Namespace Registry System

**Key Insight:** Dynamic namespace handling essential for real-world XMP.

1. **Common Namespace Registry**

   ```rust
   // Built-in namespaces
   "dc" → "http://purl.org/dc/elements/1.1/"
   "xmp" → "http://ns.adobe.com/xap/1.0/"
   "tiff" → "http://ns.adobe.com/tiff/1.0/"
   "exif" → "http://ns.adobe.com/exif/1.0/"
   ```

2. **Dynamic Expansion**
   - Parse xmlns declarations at runtime
   - Expand prefixed properties to full URIs
   - Handle custom namespaces from various applications

### Parsing State Management

**Challenge:** XML parsing requires complex state tracking for nested structures.

1. **Element Context Stack**

   ```rust
   struct ElementContext {
       name: String,
       namespace: Option<String>,
       current_array: Option<XmpArray>,
       current_struct: HashMap<String, XmpValue>,
   }
   ```

2. **RDF Parsing Patterns**
   - **Attribute-based**: `<rdf:Description dc:title="Photo Title">`
   - **Element-based**: `<dc:title>Photo Title</dc:title>`
   - **Mixed content**: Combination of both in same document

### Error Handling Strategy

**Design Decision:** Graceful degradation for malformed XMP.

1. **Continue on Errors**

   - Skip malformed properties but continue parsing
   - Collect warnings for debugging
   - Return partial results rather than failing entirely

2. **Recursive Depth Limiting**

   ```rust
   const MAX_DEPTH: usize = 100;
   // Prevent stack overflow from deeply nested structures
   ```

3. **UTF-8 Validation**
   - Handle invalid UTF-8 sequences in XML
   - Use `String::from_utf8_lossy` for recovery

### Real-world Testing Insights

**Comprehensive Test Coverage:** 39 tests covering edge cases.

1. **International Content**

   - UTF-16 encoded XMP from some applications
   - Language alternatives with xml:lang
   - Special characters in property values

2. **Malformed Data Handling**

   - Missing namespace declarations
   - Unclosed tags
   - Mixed content types
   - Very long property values

3. **Application-specific Variations**
   - Adobe Photoshop XMP structure
   - IPTC metadata in XMP
   - Custom application namespaces

### Performance Characteristics

**Achievement:** Sub-10ms parsing for typical XMP packets.

1. **Memory Efficiency**

   - Streaming parser - no DOM allocation
   - Zero-copy where possible for string values
   - UTF-16 conversion only when detected

2. **Parse Speed**
   - Event-driven parsing faster than DOM
   - Namespace lookup optimized with HashMap
   - Recursive depth limiting prevents runaway parsing

### Integration with EXIF

**Design Pattern:** Parallel metadata streams.

1. **Unified Extraction API**

   ```rust
   // Both EXIF and XMP from same file
   let exif = read_basic_exif("photo.jpg")?;
   let xmp = extract_xmp_properties("photo.jpg")?;
   ```

2. **Namespace Separation**
   - EXIF data: Numeric tag IDs (0x10F, 0x110)
   - XMP data: String properties ("dc:title", "xmp:CreateDate")
   - No conflicts between systems

### Future XMP Enhancements Identified

1. **Extended XMP Support**

   - Multi-segment reassembly for >64KB packets
   - GUID-based chunk linking
   - MD5 validation

2. **XMP Writing**

   - XML serialization with proper formatting
   - Namespace declaration management
   - Atomic JPEG updates

3. **Advanced RDF Features**
   - Complex nested structures
   - RDF resource references
   - Typed literal values

### Testing Strategy Evolution

**Pattern:** Comprehensive edge case coverage from day one.

1. **Phase 1 Tests (5 tests)**

   - Basic XMP detection and simple properties
   - Multiple namespace support

2. **Phase 2 Tests (14 tests)**

   - Hierarchical structures (arrays, structs)
   - UTF-16 encoding support
   - Real-world XMP from JPEG files

3. **Error Handling Tests (14 tests)**

   - Malformed XML recovery
   - Invalid UTF-8 handling
   - Depth limiting

4. **Enhanced Tests (6 tests)**
   - Advanced RDF features
   - IPTC and Photoshop namespaces
   - Boolean and numeric values

### Architecture Validation

**Success:** XMP system integrates cleanly with existing EXIF infrastructure.

1. **Modular Design**

   - `src/xmp/reader.rs` - JPEG XMP extraction
   - `src/xmp/parser.rs` - XML parsing logic
   - `src/xmp/types.rs` - Data structures
   - `src/xmp/namespace.rs` - Namespace management

2. **Error Handling Consistency**

   - Same Result<T> pattern as EXIF code
   - XmpError type with detailed context
   - Graceful degradation philosophy

3. **Performance Integration**
   - No measurable impact on EXIF parsing speed
   - XMP parsing remains under 10ms
   - Memory usage stays efficient

## Phase 1 Multi-Format Support Learnings

### Format Detection and Dispatch Architecture

**Key Discovery:** Central format dispatch pattern scales well across 26+ formats.

1. **Unified MetadataSegment Type**

   ```rust
   pub struct MetadataSegment {
       pub data: Vec<u8>,        // Raw EXIF/IFD data
       pub offset: u64,          // File offset
       pub source_format: FileType,  // Track origin
   }
   ```

   - Single type works for JPEG APP1, PNG chunks, TIFF IFDs, container atoms
   - Source format tracking helps with format-specific quirks

2. **Reader Trait Pattern**
   ```rust
   pub fn find_metadata_segment_from_reader<R: Read + Seek>(
       reader: &mut R,
   ) -> Result<Option<MetadataSegment>>
   ```
   - Allows both file paths and already-open streams
   - Critical for testing and memory-mapped files

### TIFF/RAW Format Insights

**Lesson:** TIFF-based formats require different memory strategies than JPEG.

1. **Full File vs. Metadata-Only Modes**

   ```rust
   pub enum TiffParseMode {
       FullFile,      // For binary extraction (thumbnails)
       MetadataOnly,  // For tag reading (90% less memory)
   }
   ```

   - IFD offsets can reference anywhere in file
   - Metadata-only mode reads just IFD chain (~64KB max)
   - Full file mode needed when extracting binary data

2. **Endianness Handling**

   ```rust
   const TIFF_LITTLE_ENDIAN: [u8; 4] = [0x49, 0x49, 0x2a, 0x00]; // "II*\0"
   const TIFF_BIG_ENDIAN: [u8; 4] = [0x4d, 0x4d, 0x00, 0x2a];    // "MM\0*"
   ```

   - Must detect early and propagate through parsing
   - Canon CR2 adds "CR" marker at offset 8 after TIFF header

3. **IFD Chain Depth Limiting**
   ```rust
   const MAX_IFD_DEPTH: usize = 10;
   ```
   - Prevents infinite loops from circular references
   - Most files have 2-3 IFDs max

### PNG eXIf Chunk Discoveries

**Key Learning:** PNG metadata is much simpler than expected.

1. **Chunk Structure**

   - PNG uses length-prefixed chunks with CRC
   - eXIf chunk contains raw TIFF/EXIF data (no wrapper)
   - Must validate PNG signature before chunk parsing

2. **Parsing Optimization**
   ```rust
   // Stop at critical chunks
   fn is_critical_data_chunk(chunk_type: &[u8; 4]) -> bool {
       matches!(chunk_type, b"IDAT" | b"IEND")
   }
   ```
   - No metadata after IDAT (image data) chunks
   - Allows early termination of parsing

### Container Format Patterns

**Discovery:** RIFF and QuickTime containers share similar patterns.

1. **RIFF Container (WebP, AVI)**

   ```rust
   // WebP EXIF has 4-byte padding before TIFF header
   let mut padding = [0u8; 4];
   reader.read_exact(&mut padding)?;
   ```

   - RIFF uses little-endian chunk sizes
   - Chunks must be word-aligned (pad if odd size)
   - WebP stores both EXIF and XMP chunks

2. **QuickTime/MP4 Atoms**

   ```rust
   // Atom sizes can be 32-bit or 64-bit
   let size = if size32 == 1 {
       // Extended 64-bit size follows
       u64::from_be_bytes(extended_size)
   } else {
       size32 as u64
   };
   ```

   - QuickTime uses big-endian exclusively
   - Size 0 means "to end of file"
   - ftyp atom validates file format via brand codes

3. **Metadata Location Patterns**
   - **RIFF**: Metadata in top-level chunks
   - **QuickTime**: Metadata in moov/meta or moov/udta atoms
   - **UUID atoms**: Can contain EXIF/XMP with specific UUIDs

### Binary Data Extraction Evolution

**Critical Fix:** Offset calculation for thumbnails in multi-format context.

1. **Format-Agnostic Offset Handling**

   ```rust
   // JPEG: Offset from TIFF header in APP1 segment
   // TIFF/RAW: Offset from file start
   let tiff_offset = if is_jpeg {
       find_exif_marker_position() + 6  // After "Exif\0\0"
   } else {
       0  // TIFF formats start at file beginning
   };
   ```

2. **Flexible Tag Value Parsing**
   ```rust
   // Tags can be stored as different types
   pub fn get_numeric_u32(&self, tag_id: u16) -> Option<u32> {
       match self.entries.get(&tag_id)? {
           ExifValue::U32(v) => Some(*v),
           ExifValue::U16(v) => Some(*v as u32),
           ExifValue::Undefined(data) if data.len() >= 4 => {
               Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
           }
           _ => None,
       }
   }
   ```

### Performance Optimization Insights

**Achievement:** No JPEG performance regression with 26-format support.

1. **Benchmark Results**

   - JPEG: ~8-9 microseconds (same as before)
   - TIFF: ~5-6 microseconds (faster due to no segment search)
   - PNG: ~7 microseconds
   - Container formats: ~8-10 microseconds

2. **Memory Optimization Patterns**

   ```rust
   // Pre-allocate with reasonable capacity
   let mut optimized_data = Vec::with_capacity(MAX_IFD_SIZE);

   // Streaming for containers
   while let Ok(chunk) = read_chunk_header(reader) {
       // Process without loading entire file
   }
   ```

3. **Early Termination Strategies**
   - PNG: Stop at IDAT chunks
   - RIFF: Sanity check at 100MB
   - QuickTime: Limit atom search depth

### Testing and Compatibility Insights

**Pattern:** ExifTool's test suite provides excellent real-world coverage.

1. **Format Detection Validation**

   - 60/190 files detected (31%) - correct as many are non-image
   - 41/60 detected files have metadata (68%)
   - Unknown formats handled gracefully

2. **Cross-Format Thumbnail Extraction**

   ```rust
   // Same logic works across all formats
   extract_offset_based_tag(ifd, 0x1201, 0x1202, original_data)
   ```

   - IFD1 structure standardized across manufacturers
   - Offset interpretation varies by container format

3. **Edge Cases Discovered**
   - Some JPEGs have no EXIF despite APP1 segments
   - PNG files rarely have eXIf chunks
   - Video formats may have metadata in multiple locations

### Architecture Validation

**Success:** Modular design scales to 26 formats without complexity explosion.

1. **Clean Separation of Concerns**

   ```
   core/
   ├── jpeg.rs         # Segment parsing
   ├── tiff.rs         # IFD parsing + dual mode
   ├── png.rs          # Chunk parsing
   ├── heif.rs         # Atom parsing
   └── containers/     # RIFF + QuickTime
       ├── riff.rs
       └── quicktime.rs
   ```

2. **Format Registration Pattern**

   ```rust
   match format {
       FileType::JPEG => jpeg::find_exif_segment(reader)?,
       FileType::PNG => png::find_exif_chunk(reader)?,
       FileType::TIFF | FileType::CR2 | ... => tiff::find_ifd_data(reader)?,
       FileType::WEBP | FileType::AVI => containers::riff::find_metadata(reader)?,
       // ... easily extensible
   }
   ```

3. **Error Handling Consistency**
   - All parsers return `Result<Option<Segment>>`
   - `None` = no metadata (not an error)
   - Graceful degradation for malformed data

### Integration Challenges and Solutions

1. **Hardcoded JPEG Calls**

   - Found 7 total (5 in lib.rs, 2 in main.rs)
   - Simple replacement with `find_metadata_segment`
   - No API breaking changes needed

2. **Rust Lifetime Management**

   ```rust
   // Can't return reference to data we just read
   // Solution: Return owned Vec<u8>
   pub data: Vec<u8>,  // Not &'a [u8]
   ```

3. **Compilation Time Impact**
   - Added ~3 seconds to release builds
   - Modular structure allows partial compilation
   - Worth it for 26-format support

### Future Format Considerations

**Patterns identified for remaining formats:**

1. **DNG (Digital Negative)**

   - TIFF-based but may have special IFD structures
   - Already supported via TIFF parser

2. **AVIF**

   - HEIF-based, should work with existing parser
   - May need additional atom types

3. **WebM**

   - Matroska container, different from RIFF/QuickTime
   - Would need new container parser

4. **RAW Formats Not Yet Tested**
   - Most use TIFF structure (should work)
   - Some have proprietary headers before TIFF data
   - May need format-specific offset adjustments

## Spike 6: DateTime Intelligence Learnings

### Chrono API Evolution

**Critical Discovery:** Chrono 0.4 deprecated several commonly-used APIs.

1. **Deprecated Pattern**

   ```rust
   // OLD - No longer works
   Utc.ymd_opt(2024, 3, 15).unwrap().and_hms_opt(14, 30, 0).unwrap()

   // NEW - Current API
   Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap()
   ```

2. **Import Requirements**

   ```rust
   use chrono::{TimeZone, Utc};     // TimeZone trait needed
   use chrono::Timelike;            // For hour(), minute(), etc.
   ```

3. **Struct Literal Syntax Changes**

   ```rust
   // OLD - with ..Default::default()
   let mut camera_info = CameraInfo::default();
   camera_info.make = Some("Canon".to_string());

   // NEW - Struct literal
   let camera_info = CameraInfo {
       make: Some("Canon".to_string()),
       ..Default::default()
   };
   ```

### Timezone Database Integration

**Key Learning:** GPS timezone inference requires proper timezone boundary database.

1. **tzf-rs Integration**

   ```rust
   use tzf_rs::DefaultFinder;
   lazy_static! {
       static ref FINDER: DefaultFinder = DefaultFinder::new();
   }
   ```

   - Provides accurate timezone boundaries globally
   - ~2MB memory overhead (acceptable)
   - Much more accurate than simple lat/lng lookup tables

2. **Chrono-tz for DST Handling**

   ```rust
   // tzf-rs returns timezone name
   let tz_name = FINDER.get_tz_name(lng, lat);

   // chrono-tz provides DST-aware offset calculation
   let tz: Tz = tz_name.parse()?;
   let offset_seconds = tz.offset_from_utc_datetime(&datetime.naive_utc())
       .format("%z")  // Workaround: format as string
       .to_string();
   ```

3. **GPS Coordinate Validation**
   ```rust
   // GPS (0,0) is invalid per exiftool-vendored
   if lat.abs() < 0.0001 && lng.abs() < 0.0001 {
       return false;  // Atlantic Ocean coordinates are placeholder
   }
   ```

### DateTime Parsing Edge Cases

**Discovery:** Loose datetime formats require special handling.

1. **Weekday Parsing Issue**

   ```rust
   // "Thu Mar 15 14:30:00 2024" fails with chrono's %a format
   // Solution: Strip weekday prefix before parsing

   fn strip_weekday_prefix(input: &str) -> Option<String> {
       let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
       for weekday in &weekdays {
           if input.trim().starts_with(weekday) {
               return Some(input[weekday.len()..].trim().to_string());
           }
       }
       None
   }
   ```

2. **Format Priority Strategy**

   ```rust
   // Try formats in order of strictness
   1. Standard EXIF: "2024:03:15 14:30:00"
   2. ISO 8601: "2024-03-15T14:30:00Z"
   3. Loose formats: Various human-readable formats
   ```

3. **Subsecond Precision Handling**
   ```rust
   // Variable digit counts mean different precisions
   match digits.len() {
       1 => num * 100.0,    // tenths → ms
       2 => num * 10.0,     // hundredths → ms
       3 => num as f32,     // milliseconds
       6 => num / 1000.0,   // microseconds → ms
   }
   ```

### Manufacturer Quirks Implementation

**Insight:** Real manufacturer quirks are more nuanced than expected.

1. **Nikon DST Bug Pattern**

   ```rust
   // Only certain models affected
   let problematic_models = ["D3", "D300", "D700", "D3S", "D300S"];

   // Only apply during DST transition periods
   fn is_near_dst_transition(datetime: &DateTime<Utc>) -> bool {
       let month = datetime.month();
       let day = datetime.day();
       // Spring: March 8-15, Fall: Oct 25-31
   }
   ```

2. **Make String Normalization**
   ```rust
   match camera_info.make.as_deref().map(str::to_lowercase).as_deref() {
       Some("nikon") | Some("nikon corporation") => { /* handle */ }
       Some("canon") => { /* handle */ }
       // Case-insensitive, handle variations
   }
   ```

### Performance Optimizations

**Achievement:** 50x better than target (0.1ms vs 5ms).

1. **Lazy Static Regex Compilation**

   ```rust
   lazy_static! {
       static ref EXIF_REGEX: Regex = Regex::new(/* pattern */).unwrap();
   }
   // Compile once, use many times
   ```

2. **Efficient Timezone Lookups**

   - tzf-rs DefaultFinder is highly optimized
   - Single static instance via lazy_static
   - No repeated initialization overhead

3. **Minimal String Allocations**
   ```rust
   // Return references where possible
   tag_name: "GPS:GPSLongitude"  // &'static str, not String
   ```

### API Design Patterns

**Learning:** Extend existing types for backward compatibility.

1. **Non-Breaking API Extension**

   ```rust
   pub struct BasicExif {
       pub make: Option<String>,
       pub model: Option<String>,
       pub orientation: Option<u16>,
       pub resolved_datetime: Option<ResolvedDateTime>,  // NEW field
   }
   ```

2. **Standalone Function Addition**

   ```rust
   // New function doesn't break existing code
   pub fn extract_datetime_intelligence<P: AsRef<Path>>(
       path: P
   ) -> Result<Option<ResolvedDateTime>>
   ```

3. **Optional Integration**
   ```rust
   // Only compute datetime intelligence if needed
   let resolved = if extract_datetime {
       Some(extract_datetime_intelligence(&exif_data, &xmp_data)?)
   } else {
       None
   };
   ```

### Testing Strategies

**Pattern:** Layer integration tests for complex features.

1. **GPS Timezone Test**

   ```rust
   // Use known coordinates with expected timezones
   let nyc = (40.7128, -74.0060);  // America/New_York
   let tokyo = (35.6762, 139.6503); // Asia/Tokyo
   ```

2. **Performance Benchmarking**

   ```rust
   let start = std::time::Instant::now();
   let _ = intelligence.resolve_capture_datetime(&collection, &camera);
   let elapsed = start.elapsed();
   assert!(elapsed.as_millis() < 5);  // Target: <5ms
   ```

3. **Cross-Validation Testing**
   ```rust
   // Test warnings for inconsistent timestamps
   let collection = DateTimeCollection {
       datetime_original: /* 2024-03-15 */,
       modify_date: /* 2024-03-17 */,  // 2 days later
   };
   // Should generate InconsistentDatetimes warning
   ```

### Clippy Compliance

**Learning:** Clippy enforces idiomatic Rust patterns.

1. **Field Assignment Pattern**

   ```rust
   // Clippy warning: field_reassign_with_default
   // Instead of:
   let mut x = Foo::default();
   x.field = value;

   // Use:
   let x = Foo {
       field: value,
       ..Default::default()
   };
   ```

2. **Match Reference Patterns**

   ```rust
   // Clippy warning: needless borrow on both sides
   // Instead of:
   match &format {
       &WEBP_FORMAT => { }
   }

   // Use:
   match format {
       WEBP_FORMAT => { }
   }
   ```

### Confidence Scoring Design

**Insight:** Multi-tier confidence helps users understand reliability.

1. **Source-Based Base Scores**

   ```rust
   InferenceSource::ExplicitTag { .. } => 0.95,
   InferenceSource::GpsCoordinates { .. } => 0.80,
   InferenceSource::UtcDelta { .. } => 0.70,
   InferenceSource::ManufacturerQuirk { .. } => 0.60,
   InferenceSource::None => 0.10,
   ```

2. **Dynamic Adjustments**

   ```rust
   // Boost for validation
   if has_cross_validation { confidence += 0.05; }

   // Penalty for warnings
   confidence -= warnings.len() as f32 * 0.05;
   ```

### Error Handling Philosophy

**Pattern:** Graceful degradation over hard failures.

1. **Continue on Parsing Errors**

   ```rust
   // Don't fail entire datetime extraction for one bad format
   if let Ok(dt) = Self::parse_exif_standard(input) {
       return Ok(dt);
   }
   // Try next format...
   ```

2. **Collect Warnings, Don't Fail**

   ```rust
   pub struct ResolvedDateTime {
       pub datetime: ExifDateTime,
       pub warnings: Vec<DateTimeWarning>,  // Collect issues
   }
   ```

3. **Detailed Error Context**
   ```rust
   Error::InvalidDateTime(format!(
       "Could not parse datetime: '{}' (tried {} formats)",
       input, formats_tried
   ))
   ```

### Real-World Timezone Complexity

**Discovery:** Timezone inference has many edge cases.

1. **15/30 Minute Boundaries**

   ```rust
   // Most timezones align to 15 or 30 minute boundaries
   if offset_minutes % 15 != 0 && offset_minutes % 30 != 0 {
       // Suspicious offset, reduce confidence
   }
   ```

2. **DST Transition Handling**

   ```rust
   // Flag dates during DST transitions as potentially problematic
   if is_near_dst_transition(&datetime) {
       warnings.push(DateTimeWarning::DstTransition);
   }
   ```

3. **UTC Delta Validation**
   ```rust
   // Sanity check GPS time vs local time delta
   if delta_minutes.abs() > 14 * 60 {
       return None;  // Beyond valid timezone range
   }
   ```

### Architecture Decisions

**Success:** Modular design with clear separation of concerns.

1. **Module Organization**

   ```
   datetime/
   ├── mod.rs           # Public API
   ├── types.rs         # Core data structures
   ├── parser.rs        # String parsing
   ├── extractor.rs     # EXIF/XMP extraction
   ├── intelligence.rs  # Coordination engine
   ├── gps_timezone.rs  # GPS inference
   ├── utc_delta.rs     # Delta calculation
   └── quirks.rs        # Manufacturer handling
   ```

2. **Trait-Based Extension**

   ```rust
   // Could add more inference sources via traits
   trait TimezoneInferenceSource {
       fn infer(&self, collection: &DateTimeCollection) -> Option<InferenceSource>;
   }
   ```

3. **Priority-Based System**
   - Explicit tags (highest priority)
   - GPS coordinates
   - UTC timestamp delta
   - Manufacturer quirks (lowest priority)

### Integration Insights

**Learning:** DateTime intelligence integrates cleanly with existing systems.

1. **EXIF Data Flow**

   ```rust
   JPEG → EXIF Tags → DateTimeCollection → Intelligence Engine → ResolvedDateTime
   ```

2. **XMP Coordination**

   ```rust
   // XMP provides additional datetime sources
   if let Some(xmp_create) = xmp.get("xmp:CreateDate") {
       // Add to DateTimeCollection
   }
   ```

3. **Zero Performance Impact**
   - Only compute when requested
   - Lazy initialization of timezone database
   - No overhead for basic EXIF extraction

## Binary Extraction Refactoring

### API Simplification Journey

**Key Learning:** Simple, focused functions are better than complex multi-purpose helpers.

1. **Original Complex Design**

   ```rust
   // OLD: extract_thumbnail() and extract_largest_preview()
   // - Complex logic to find largest image
   // - Mixed concerns (finding vs extracting)
   // - Duplicate code for different tag pairs
   ```

2. **Simplified to Single Function**

   ```rust
   pub fn extract_binary_tag(
       ifd: &ParsedIfd,
       tag_id: u16,
       original_data: &[u8]
   ) -> Result<Option<Vec<u8>>>
   ```

   - Direct tag extraction by ID
   - Let caller decide which tag to extract
   - No opinions about "largest" or "best"

3. **Benefits Realized**
   - Reduced code size by ~60%
   - More flexible for different use cases
   - Easier to test and debug
   - Works for any offset-based binary data

### TIFF Header Offset Calculation

**Critical Discovery:** Offset calculation differs between formats.

1. **JPEG Files**

   ```rust
   // TIFF header is AFTER the APP1 marker header
   // Structure: FF E1 [length] "Exif\0\0" [TIFF header starts here]
   let tiff_offset = if original_data.len() > 12 && &original_data[0..2] == b"\xFF\xD8" {
       // Find "Exif\0\0" marker
       if let Some(exif_pos) = original_data.windows(6)
           .position(|window| window == b"Exif\x00\x00") {
           exif_pos + 6  // Skip past "Exif\0\0"
       } else {
           12  // Fallback typical offset
       }
   } else {
       0  // TIFF-based formats start at file beginning
   };
   ```

2. **Key Insight**

   - EXIF offsets are ALWAYS relative to TIFF header
   - TIFF header location varies by container format
   - Must calculate file offset as: tiff_offset + exif_offset

3. **Format-Specific Patterns**
   - **JPEG**: TIFF header ~12 bytes into file
   - **TIFF/RAW**: TIFF header at byte 0
   - **PNG**: Raw TIFF data in eXIf chunk
   - **WebP**: 4-byte padding before TIFF header

### JPEG Validation with Padding

**Discovery:** JPEG files often have padding after EOI marker.

1. **Original Assumption**

   ```rust
   // WRONG: Expected EOI at exact end
   data[data.len()-2..] == [0xFF, 0xD9]
   ```

2. **Real-World Pattern**

   ```rust
   // Search for EOI in last 32 bytes
   let search_start = if data.len() > 32 {
       data.len() - 32
   } else {
       2
   };
   let has_eoi = data[search_start..]
       .windows(2)
       .any(|window| window[0] == 0xFF && window[1] == 0xD9);
   ```

3. **Why Padding Exists**
   - Some cameras pad to sector boundaries
   - Editing software may append metadata
   - File systems may add alignment padding

### MPF (Multi-Picture Format) Discovery

**Key Learning:** Modern cameras use MPF for multiple images in one file.

1. **MPF Structure**

   - Stored in APP2 segments (0xFFE2)
   - Marker: "MPF\0" (similar to "Exif\0\0")
   - Contains IFD structure like EXIF
   - References multiple images (previews, 3D pairs, etc.)

2. **Canon R50 Example**

   ```
   JPEG file contains:
   - Main image
   - 160x120 thumbnail in EXIF (IFD1)
   - 1620x1080 preview in MPF (APP2)
   ```

3. **Why Current Code Failed**

   - `find_metadata_segment` only searches APP1
   - MPF requires APP2 segment parsing
   - Different offset calculation (from MPF marker, not TIFF)

4. **Implementation Requirements**
   - Extend segment search to include APP2
   - Add MPF IFD parser
   - Handle MPF-specific offset calculations
   - Support multiple metadata segments per file

### Lessons from Spike Simplification

1. **Start Simple, Extend Later**

   - Basic binary extraction first
   - Add format detection later
   - Preview selection logic can be external

2. **Test with Real Files Early**

   - Synthetic tests miss real-world quirks
   - Camera files have unexpected patterns
   - ExifTool compatibility is the gold standard

3. **Document Limitations Clearly**
   - Current: JPEG APP1 only
   - Missing: MPF, FlashPix, IPTC
   - Future: Multi-segment support needed
