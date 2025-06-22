# Development Spikes

This document outlines the incremental development spikes to build exif-oxide, starting from basic functionality and building up to full ExifTool compatibility.

## Spike 1: Basic EXIF Tag Reading (Make, Model, Orientation)

**Goal:** Minimal viable EXIF reader that can extract Make, Model, and Orientation from JPEG files.

### Success Criteria
- [ ] Read JPEG files and locate APP1 (EXIF) segment
- [ ] Parse basic IFD structure
- [ ] Extract Make (0x10F), Model (0x110), Orientation (0x112) tags
- [ ] Handle both endianness (II and MM)
- [ ] Basic error handling
- [ ] Tests with images from different manufacturers

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

## Spike 2: Maker Note Parsing

**Goal:** Parse manufacturer-specific data from maker notes.

### Success Criteria
- [ ] Parse Canon maker notes (most documented)
- [ ] Extract Canon-specific tags (lens info, camera settings)
- [ ] Handle maker note offset corrections
- [ ] Abstract maker note detection and dispatch

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

---

## Spike 3: Binary Tag Extraction (PreviewImage)

**Goal:** Extract embedded preview images from EXIF data.

### Success Criteria
- [ ] Extract JPEG thumbnail from IFD1
- [ ] Extract preview from maker notes
- [ ] Handle multiple preview sizes
- [ ] Validate extracted images
- [ ] Memory-efficient extraction

### Implementation Steps

1. **Thumbnail extraction** (`src/extract/thumbnail.rs`)
   ```rust
   pub fn extract_thumbnail(exif: &ExifData) -> Result<Vec<u8>> {
       let offset = exif.get_u32(0x201)?; // ThumbnailOffset
       let length = exif.get_u32(0x202)?; // ThumbnailLength
       exif.read_bytes(offset, length)
   }
   ```

2. **Preview extraction** (`src/extract/preview.rs`)
   - Canon: PreviewImageStart/Length in maker notes
   - Nikon: PreviewImage or JpgFromRaw
   - Handle compression (some are compressed)

3. **Validation** (`src/extract/validate.rs`)
   - Check JPEG SOI/EOI markers
   - Verify size matches tags
   - Option to decode and verify

### Performance Considerations
- Lazy extraction (only on request)
- Streaming API for large previews
- Memory mapping for efficiency

---

## Spike 4: XMP Reading and Writing

**Goal:** Full XMP support including struct parsing.

### Success Criteria
- [ ] Read XMP from JPEG APP1 segments
- [ ] Parse XMP structs (hierarchical data)
- [ ] Write XMP back to JPEG
- [ ] Preserve unknown namespaces
- [ ] Handle multiple XMP segments

### Implementation Steps

1. **XMP detection** (`src/xmp/reader.rs`)
   - Scan for XMP APP1 segments
   - Handle extended XMP (multiple segments)
   - Parse XML structure

2. **Struct support** (`src/xmp/struct.rs`)
   ```rust
   pub enum XmpValue {
       Simple(String),
       Array(Vec<XmpValue>),
       Struct(HashMap<String, XmpValue>),
   }
   ```

3. **Namespace handling** (`src/xmp/namespace.rs`)
   - Register known namespaces
   - Preserve unknown ones
   - Handle namespace prefixes

4. **XMP writing** (`src/xmp/writer.rs`)
   - Serialize XMP to XML
   - Split into segments if >64KB
   - Update JPEG segments

### Integration Points
- Coordinate with EXIF data
- Handle EXIF/XMP synchronization
- Preserve both on write

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