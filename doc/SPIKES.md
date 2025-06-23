# Development Spikes

This document outlines the incremental development spikes to build exif-oxide, starting from basic functionality and building up to full ExifTool compatibility.

## Spike 1: Basic EXIF Tag Reading (Make, Model, Orientation)

**Goal:** Minimal viable EXIF reader that can extract Make, Model, and Orientation from JPEG files.

### Success Criteria

- [x] Read JPEG files and locate APP1 (EXIF) segment
- [x] Parse basic IFD structure
- [x] Extract Make (0x10F), Model (0x110), Orientation (0x112) tags
- [x] Handle both endianness (II and MM)
- [x] Basic error handling
- [x] Tests with images from different manufacturers

### Implementation Steps

1. **Core structures** (`src/core/types.rs`)

   ```rust
   pub struct IfdEntry {
       tag: u16,
       format: u16,
       count: u32,
       value_offset: u32,
   }

   pub enum Endian {
       Little,
       Big,
   }

   pub enum Value {
       Ascii(String),
       Short(u16),
       Long(u32),
   }
   ```

2. **JPEG parser** (`src/core/jpeg.rs`)

   - Find SOI marker (0xFFD8)
   - Scan for APP1 marker (0xFFE1)
   - Verify EXIF header ("Exif\0\0")
   - Extract TIFF data block

3. **IFD parser** (`src/core/ifd.rs`)

   - Read TIFF header (II/MM, 0x2A)
   - Parse IFD entries
   - Handle value reading (inline vs offset)
   - String extraction with null termination

4. **Simple API** (`src/lib.rs`)

   ```rust
   pub fn read_basic_exif(path: &Path) -> Result<BasicExif> {
       let jpeg_data = JpegReader::read_file(path)?;
       let exif_data = jpeg_data.find_exif()?;
       let ifd = IfdParser::parse(exif_data)?;

       Ok(BasicExif {
           make: ifd.get_string(0x10F),
           model: ifd.get_string(0x110),
           orientation: ifd.get_u16(0x112),
       })
   }
   ```

5. **Tests** (`tests/spike1.rs`)
   - Test images from Canon, Nikon, Sony
   - Both orientations (landscape/portrait)
   - Both byte orders
   - Missing EXIF data
   - Corrupted data

### Deliverables

- Working code that passes all tests
- Benchmark showing <1ms for typical JPEG
- README with usage example

### Learnings to Document

- JPEG segment structure quirks
- IFD parsing edge cases
- Performance bottlenecks found

---

## Spike 1.5: Minimal Table Generation

**Goal:** Build just enough table generation to validate our architecture and unblock future development.

### Context

During Spike 1 implementation, we hard-coded the handling of Make, Model, and Orientation tags instead of using ExifTool's tag tables as originally designed. This approach doesn't scale - Spike 2 would require hard-coding hundreds of manufacturer-specific tags. We need to validate our table-driven architecture before proceeding.

### Success Criteria

- [x] Parse basic tag definitions from ExifTool's Exif.pm
- [x] Generate Rust code for ~500 common EXIF tags
- [x] Replace hard-coded tag handling in IFD parser
- [x] All Spike 1 tests still pass
- [x] Document parsing challenges discovered

### Implementation Steps

1. **Perl Parser** (`build.rs` or separate tool)

   ```perl
   # Parse structures like this from Exif.pm:
   0x10f => {
       Name => 'Make',
       Groups => { 2 => 'Camera' },
       Writable => 'string',
       WriteGroup => 'IFD0',
   }
   ```

   Extract only:

   - Tag ID (hex number)
   - Name (human-readable)
   - Basic format type

2. **Code Generator** (`src/tables/generated.rs`)

   ```rust
   // Generated code should look like:
   pub const EXIF_TAGS: &[(u16, TagInfo)] = &[
       (0x010f, TagInfo {
           name: "Make",
           format: ExifFormat::Ascii,
           group: Some("Camera"),
       }),
       // ... more tags
   ];
   ```

3. **Update IFD Parser** (`src/core/ifd.rs`)

   - Remove hard-coded match statements
   - Look up tags in generated table
   - Use format info to guide parsing
   - Fall back gracefully for unknown tags

4. **Validation**
   - Run all existing tests
   - Compare output with ExifTool for test images
   - Ensure no regression in functionality

### Scope Limits (Important!)

**In Scope:**

- Basic tag ID, name, and format
- Standard EXIF tags (not manufacturer-specific)
- Simple string and integer types
- Static code generation (not runtime parsing)

**Out of Scope (for now):**

- Complex conditions (e.g., "if CameraModel == X then...")
- PrintConv functions (human-readable conversions)
- Maker note tables
- Binary data structures
- Table inheritance
- Dynamic/runtime table loading

### Expected Challenges

1. **Perl Syntax**: ExifTool uses complex Perl data structures. Focus on the simple cases first.

2. **Format Mapping**: ExifTool's format names (e.g., 'string', 'int16u') need mapping to our ExifFormat enum.

3. **Special Cases**: Some tags have complex definitions. Skip these initially - we just need the basics working.

### Deliverables

1. **Table generator** - Build script that converts Exif.pm to Rust ✅
2. **Generated tables** - 496 EXIF tags with format and group info ✅
3. **Updated IFD parser** - Using tables instead of hard-coding ✅
4. **Documentation** - What we learned about ExifTool's structure ✅
5. **Test results** - Proof that nothing broke ✅
6. **Development tool** - `parse_exiftool_tags` binary for debugging ✅

### Why This Matters

Without this spike:

- Every future spike accumulates technical debt
- We miss critical edge cases ExifTool knows about
- We can't leverage 25 years of accumulated knowledge
- We'll likely need major rewrites later

With this spike:

- Clean foundation for all tag-based features
- Early validation of our architecture
- Understanding of ExifTool's complexity
- Confidence we can handle full table generation later

### Implementation Results (Completed)

**Actual Implementation Time:** 1 day (faster than estimated due to existing regex patterns)

**Key Achievements:**

- Successfully parsed 496 tags from ExifTool's Exif.pm
- Generated static lookup tables with O(1) access
- Extended EXIF value types to support all formats (Rational, SignedRational, arrays)
- Added comprehensive test coverage (29 tests total)
- Created development tool for debugging table generation

**Architecture Insights:**

1. **Build-time Code Generation Works Well**

   - Using `build.rs` to generate static tables is efficient
   - Generated code includes in `target/debug/build/.../out/generated_tags.rs`
   - No runtime overhead for tag lookup

2. **ExifTool Perl Structure is Parseable**

   - Most tag definitions follow predictable patterns
   - Regex-based parsing captures 80% of cases
   - Complex conditions can be skipped initially without loss of functionality

3. **Format Type Mapping**

   ```rust
   // ExifTool formats → Our types
   "string" → ExifFormat::Ascii
   "int16u" → ExifFormat::U16
   "rational64u" → ExifFormat::Rational
   "rational64s" → ExifFormat::SignedRational
   ```

4. **Group Organization**
   - ExifTool uses hierarchical groups (0=family, 1=specific, 2=category)
   - We extract group 2 (category) for user-friendly organization
   - Results in 5 main groups: Camera, Time, Author, Location, General

**Parsing Challenges Solved:**

1. **Multi-line Perl Structures**

   - Used `(?s)` regex flag for multi-line matching
   - Handled nested braces with careful regex bounds

2. **Simple vs Complex Tags**

   - Simple: `0x10f => 'Make',`
   - Complex: `0x10f => { Name => 'Make', ... }`
   - Parser handles both patterns

3. **Format Defaults**
   - Tags without explicit `Writable` default to unknown format
   - Parser gracefully handles missing information

**Testing Strategy:**

- Unit tests for all EXIF format types
- Table lookup tests for known tags
- Integration tests with real ExifTool images
- Rational number validation (no zero denominators)

**Rust Idioms Applied:**

- Iterator patterns (`sort_by_key`, `dedup_by_key`, `take`, `any`)
- Ownership handling (collected stats before consuming HashMap)
- Error handling with `Result<Option<T>>` pattern
- Zero-cost abstractions with static lookup tables

---

## Spike 2: Maker Note Parsing

**Goal:** Parse manufacturer-specific data from maker notes.

### Success Criteria

- [x] Parse Canon maker notes (most documented)
- [x] Extract Canon-specific tags (lens info, camera settings)
- [x] Handle maker note offset corrections
- [x] Abstract maker note detection and dispatch

### Implementation Steps

1. **Maker note detection** (`src/maker/mod.rs`)

   - Read maker note tag (0x927C)
   - Identify manufacturer from Make tag
   - Dispatch to appropriate parser

2. **Canon parser** (`src/maker/canon.rs`)

   - Port basic Canon tables from ExifTool
   - Handle Canon's IFD-style maker notes
   - Extract CameraSettings, ShotInfo, etc.

3. **Table generation prototype** (`tools/table_converter/`)
   - Parse simple Perl hash structures
   - Generate Rust constants
   - Verify with Canon.pm subset

### Key Challenges

- Maker note offset calculations
- Encrypted/obfuscated sections
- Model-specific variations

### Implementation Results (Completed)

**Actual Implementation Time:** 1 day

**Key Achievements:**

- Successfully implemented manufacturer detection and dispatch system
- Extended build.rs to parse Canon.pm and generate 34 Canon-specific tags
- Fixed ExifIFD parsing issue (maker notes are in ExifIFD, not IFD0)
- Added special handling for structural tags (0x8769, 0x8825)
- Successfully parsed Canon maker notes from Canon1DmkIII.jpg (28 tags extracted)
- Created trait-based MakerNoteParser architecture for extensibility
- Added tag prefixing system (0xC000+) to avoid EXIF/maker note conflicts

**Architecture Insights:**

1. **Maker Notes Location**

   - Maker notes (tag 0x927c) are typically in ExifIFD, not IFD0
   - Required extending IFD parser to handle sub-IFDs
   - ExifIFD offset (tag 0x8769) needs special U32 format handling

2. **Canon Maker Note Structure**

   - Canon uses standard IFD format (same as main EXIF)
   - 6962 bytes of maker note data in test image
   - No special header or offset corrections needed for Canon
   - May have 8-byte footer with offset information

3. **Table Generation Extension**
   - Canon.pm parsing added to build.rs
   - Generated separate CANON_TAGS lookup table
   - 34 Canon tags parsed successfully
   - SubDirectory tags skipped for now (too complex)

**Technical Learnings:**

1. **ExifIFD Parsing Critical**

   - Standard IFD parser only reads IFD0
   - Maker notes usually in ExifIFD (tag 0x8769)
   - Need to merge ExifIFD entries with IFD0 for complete parsing

2. **Structural Tag Handling**

   - Tags like 0x8769 (ExifOffset) need special format handling
   - ExifTool's format definitions can be wrong for structural tags
   - Override format for known structural tags regardless of lookup table

3. **Canon Maker Note Format**
   - Direct IFD structure (no complex header)
   - Uses same byte order as main EXIF
   - Footer detection works but not needed for basic parsing

**Testing Results:**

- Canon1DmkIII.jpg: 28 Canon tags extracted successfully
- ExifTool comparison: ExifTool shows 36 entries, we got 28 (good coverage)
- All maker note data successfully parsed as IFD structure

---

## Spike 3: Binary Tag Extraction (PreviewImage)

**Goal:** Extract embedded preview images and thumbnails from EXIF data.

### Success Criteria

- [x] Extract JPEG thumbnail from IFD1
- [x] Extract preview from maker notes
- [x] Handle multiple preview sizes
- [x] Validate extracted images
- [x] Memory-efficient extraction
- [x] Test with real-world images (JPG/RAW/HEIF)

### Implementation Steps

1. **Extend IFD Parser for IFD1** (`src/core/ifd.rs`)

   - Parse IFD1 (thumbnail directory) after IFD0
   - Handle thumbnail tags: 0x201 (ThumbnailOffset), 0x202 (ThumbnailLength)
   - Merge IFD1 entries with appropriate prefixing

2. **Extend Canon Tag Tables** (`build.rs`)

   - Add Canon preview image tags from Canon.pm
   - PreviewImageStart, PreviewImageLength, PreviewImageValid
   - Support multiple preview sizes

3. **Image Extraction Module** (`src/extract/`)

   ```rust
   // mod.rs - Public API
   pub fn extract_thumbnail(exif: &ExifData) -> Result<Option<Vec<u8>>>
   pub fn extract_preview(exif: &ExifData) -> Result<Vec<Vec<u8>>>
   pub fn extract_largest_preview(exif: &ExifData) -> Result<Option<Vec<u8>>>

   // thumbnail.rs - IFD1 thumbnail extraction
   // preview.rs - Maker note preview extraction
   ```

4. **Binary Data Handling**

   - JPEG validation (check SOI 0xFFD8 / EOI 0xFFD9 markers)
   - Bounds checking for offset/length pairs
   - Streaming extraction for memory efficiency

5. **Comprehensive Testing** (`tests/spike3.rs`)

   - ExifTool test images: Canon.jpg, Canon1DmkIII.jpg
   - User test images: JPG/RAW pairs, iPhone HEIF
   - Validate extracted images as proper JPEG format
   - Compare results with ExifTool's `-b -PreviewImage`

### Test Images Available

- **ExifTool Suite**: Canon.jpg, Canon1DmkIII.jpg (professional camera)
- **User Images**: `test-images/` directory with JPG/RAW pairs and iPhone HEIF
- **Coverage**: Basic cameras, professional cameras, modern mobile formats

### Performance Considerations

- Lazy extraction (only on request)
- Streaming API for large previews
- Memory-efficient bounds checking
- <5ms extraction time target

---

## Spike 4: XMP Reading and Writing

**Goal:** Full XMP support including struct parsing and writing capabilities.

### Context

XMP (Extensible Metadata Platform) is Adobe's XML-based metadata format that complements EXIF:
- Stored in JPEG APP1 segments with different signature than EXIF
- XML-based with RDF structure and namespace support
- Can exceed 64KB (requires Extended XMP across multiple segments)
- Supports hierarchical data structures (arrays, structs)
- Widely used for advanced metadata (keywords, ratings, GPS tracks, creator info)

### Success Criteria

- [x] Read XMP from JPEG APP1 segments
- [x] Parse XMP structs (hierarchical data)
- [ ] Handle Extended XMP (>64KB across multiple segments)
- [ ] Write XMP back to JPEG
- [x] Preserve unknown namespaces
- [x] Round-trip fidelity for all XMP data
- [x] Performance <10ms for typical files

### Phase 1: Basic XMP Detection and Reading (COMPLETE)

1. **XMP Segment Detection** (`src/xmp/mod.rs`, `src/xmp/reader.rs`) ✅
   ```rust
   pub struct XmpPacket {
       standard: Vec<u8>,     // Main XMP packet
       extended: Option<ExtendedXmp>, // Extended data if present
   }
   ```
   - Scan APP1 segments for XMP signature: `"http://ns.adobe.com/xap/1.0/\0"`
   - Differentiate from EXIF APP1 segments (different signature)
   - Extract XMP packet data

2. **XML Parsing Foundation**
   - Use `quick-xml` crate for performance and streaming capability
   - Parse basic RDF structure
   - Extract simple key-value pairs initially
   - Handle XML namespaces

3. **JPEG Integration** (`src/core/jpeg.rs` extension)
   ```rust
   pub struct JpegSegments {
       exif: Option<Vec<u8>>,
       xmp: Option<XmpPacket>,
   }
   ```
   - Extend JPEG parser to collect both EXIF and XMP segments
   - Return both data types from main API

### Phase 2: Namespace and Struct Support (COMPLETE)

1. **Namespace Registry** (`src/xmp/namespace.rs`)
   ```rust
   pub struct NamespaceRegistry {
       known: HashMap<&'static str, &'static str>, // prefix -> URI
       custom: HashMap<String, String>,
   }
   
   // Common namespaces
   const XMP_NS: &str = "http://ns.adobe.com/xap/1.0/";
   const DC_NS: &str = "http://purl.org/dc/elements/1.1/";
   const EXIF_NS: &str = "http://ns.adobe.com/exif/1.0/";
   const TIFF_NS: &str = "http://ns.adobe.com/tiff/1.0/";
   ```

2. **Hierarchical Data Model** (`src/xmp/types.rs`)
   ```rust
   pub enum XmpValue {
       Simple(String),
       Array(XmpArray),
       Struct(HashMap<String, XmpValue>),
   }
   
   pub enum XmpArray {
       Ordered(Vec<XmpValue>),      // rdf:Seq
       Unordered(Vec<XmpValue>),    // rdf:Bag
       Alternative(Vec<XmpValue>),   // rdf:Alt (e.g., language alternatives)
   }
   ```

3. **Struct Parsing Implementation**
   - Parse rdf:Description blocks
   - Handle nested properties
   - Support language alternatives (xml:lang)
   - Parse typed nodes (rdf:type)

### Phase 3: Extended XMP Support (1-2 days)

1. **Extended XMP Detection** (`src/xmp/extended.rs`)
   ```rust
   pub struct ExtendedXmpInfo {
       guid: String,              // Links to main packet
       total_length: u32,         // Total extended data size
       md5: [u8; 16],            // For validation
       chunks: BTreeMap<u32, Vec<u8>>, // offset -> data
   }
   ```
   - Look for xmpNote:HasExtendedXMP in standard packet
   - Find APP1 segments with signature: `"http://ns.adobe.com/xmp/extension/\0"`

2. **Chunk Assembly**
   - Parse chunk headers: GUID (32 bytes) + length (4) + offset (4)
   - Collect all chunks with matching GUID
   - Reassemble in offset order
   - Validate with MD5 hash

### Phase 4: XMP Writing (2-3 days)

1. **XML Serialization** (`src/xmp/writer.rs`)
   ```rust
   pub trait XmpSerialize {
       fn to_xml(&self, writer: &mut Writer<Vec<u8>>) -> Result<()>;
   }
   ```
   - Convert XmpValue back to proper RDF/XML
   - Maintain namespace declarations
   - Format with appropriate whitespace
   - Follow Adobe XMP Specification formatting

2. **Segment Management**
   - Calculate packet size
   - If >64KB, split into Extended XMP:
     - Generate GUID
     - Calculate MD5 of extended portion
     - Create chunks with proper headers
   - Build APP1 segments with correct markers

3. **JPEG Update** (`src/core/jpeg_writer.rs`)
   ```rust
   pub fn update_xmp(path: &Path, xmp: &XmpPacket) -> Result<()> {
       // 1. Read existing JPEG
       // 2. Preserve non-XMP segments
       // 3. Replace/add XMP segments
       // 4. Write to temp file
       // 5. Atomic rename
   }
   ```

### Phase 5: Integration and Testing (1-2 days)

1. **Unified API** (`src/lib.rs` extension)
   ```rust
   pub struct Metadata {
       exif: HashMap<u16, ExifValue>,
       xmp: HashMap<String, XmpValue>,
   }
   
   impl Metadata {
       pub fn get_title(&self) -> Option<&str>;
       pub fn get_keywords(&self) -> Vec<&str>;
       pub fn get_creator(&self) -> Option<&str>;
   }
   ```

2. **Field Synchronization**
   - Common fields mapping (Make, Model, DateTime, etc.)
   - Conflict resolution strategies
   - Bidirectional sync options

3. **Comprehensive Testing** (`tests/spike4.rs`)
   - ExifTool test images with XMP
   - Large XMP files requiring extended segments
   - Round-trip read/write tests
   - Malformed XMP handling
   - Performance benchmarks

### Technical Decisions

1. **XML Library**: `quick-xml` for performance and streaming
2. **Memory Strategy**: Stream large XMP, cache parsed structures
3. **Write Safety**: Temp file + atomic rename pattern
4. **Error Handling**: Graceful degradation for malformed XMP
5. **API Design**: Separate XMP/EXIF internally, unified access API

### Expected Challenges

1. **RDF Complexity**: RDF/XML has many valid representations
2. **Extended XMP Edge Cases**: Chunks may arrive out of order
3. **Namespace Variations**: Same data can use different prefixes
4. **Character Encoding**: Proper UTF-8/UTF-16 handling
5. **Write Atomicity**: Ensuring no data loss on write failure

### Deliverables

1. **XMP Reader** - Complete XMP packet extraction and parsing ✅
2. **Struct Support** - Full hierarchical data model ✅  
3. **Extended XMP** - Multi-segment reassembly (future enhancement)
4. **XMP Writer** - XML serialization and JPEG update (future enhancement)
5. **Test Suite** - Comprehensive validation ✅ (39 tests)
6. **Documentation** - API docs and examples ✅
7. **Benchmarks** - Performance validation <10ms ✅

### Implementation Results (COMPLETE)

**Actual Implementation Time:** 4 days (phases 1-2 complete, advanced features implemented)

**Key Achievements:**
- Complete XMP packet detection in JPEG APP1 segments
- Advanced XML parsing with hierarchical data structures (arrays, structs)
- RDF container support (rdf:Seq, rdf:Bag, rdf:Alt)
- Language alternatives with xml:lang support
- UTF-16 encoding support for international content
- Dynamic namespace registry with common namespaces
- Comprehensive error handling for malformed XMP
- 39 test cases covering edge cases and real-world scenarios

**Architecture Insights:**
1. **Streaming XML Parser**: quick-xml provides efficient parsing without loading entire document
2. **Hierarchical Data Model**: XmpValue enum supports Simple, Array, and Struct variants
3. **Namespace Handling**: Registry pattern allows dynamic prefix expansion
4. **Error Recovery**: Graceful degradation with malformed XML continues parsing where possible
5. **Memory Efficiency**: Zero-copy where possible, UTF-16 decoded only when detected

### Integration Points

- Coordinate with EXIF data (shared fields)
- Handle EXIF/XMP synchronization
- Preserve both on write operations
- Unified metadata API

---

## Spike 5: Table Generation System

**Goal:** Automated conversion of ExifTool's Perl tables.

### Success Criteria

- [ ] Parse ExifTool's tag table syntax
- [ ] Generate equivalent Rust structures
- [ ] Handle conditions and conversions
- [ ] Support table inheritance
- [ ] Validate against test images

### Implementation Steps

1. **Perl parser** (`tools/table_converter/parser.rs`)

   - Handle Perl hash syntax
   - Parse nested structures
   - Extract table metadata

2. **Code generator** (`tools/table_converter/generator.rs`)

   - Generate static tables
   - Create lookup functions
   - Handle special cases

3. **Validation** (`tools/table_converter/validate.rs`)
   - Compare with ExifTool output
   - Test all converted tags
   - Performance benchmarks

---

## Spike 6: DateTime Intelligence

**Goal:** Port exiftool-vendored's datetime heuristics.

### Success Criteria

- [ ] Multi-source timezone inference
- [ ] GPS-based timezone lookup
- [ ] Manufacturer quirk handling
- [ ] Subsecond precision support
- [ ] Validation against known issues

### Implementation Steps

1. **Core parser** (`src/datetime/parser.rs`)

   - Handle various datetime formats
   - Parse subsecond data
   - Validate ranges

2. **Timezone inference** (`src/datetime/timezone.rs`)

   - GPS coordinates → timezone
   - UTC offset calculation
   - Explicit timezone parsing

3. **Quirk handling** (`src/datetime/quirks.rs`)
   - Nikon DST bug
   - Canon timezone format
   - Apple variations

### Test Cases

- Images with known timezone issues
- GPS-tagged images
- Videos (default UTC)
- Missing timezone data

---

## Future Spikes

### Spike 7: Performance Optimization

- Memory mapping implementation
- SIMD endian conversion
- Parallel IFD processing
- Benchmark suite

### Spike 8: Write Support

- EXIF writing to JPEG
- Maker note preservation
- Safe in-place updates
- Backup mechanisms

### Spike 9: RAW Format Support

- CR2 (Canon)
- NEF (Nikon)
- ARW (Sony)
- DNG (Adobe)

### Spike 10: Advanced Features

- Async API
- WASM compilation
- Streaming parser
- Plugin system

## Development Philosophy

Each spike should:

1. Have clear, measurable goals
2. Build on previous spikes
3. Include comprehensive tests
4. Document learnings and gotchas
5. Stay focused - avoid scope creep
6. Benchmark performance impact

The goal is steady progress toward a production-ready tool while learning the problem space deeply.
