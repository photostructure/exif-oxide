# Phase 2: Maker Note Parser Expansion

**Goal**: Port all major manufacturer maker note parsers from ExifTool, following the Canon implementation pattern.

**Duration**: 3-4 weeks

**Dependencies**: ✅ Phase 1 (multi-format support), ProcessBinaryData framework

**Prerequisites**: Understanding of EXIF IFD structure, binary data parsing, Rust trait systems

## What Are Maker Notes?

Maker notes are proprietary EXIF data sections where camera manufacturers store camera-specific metadata that doesn't fit standard EXIF tags. Each manufacturer uses different:

- **Data structures** (some IFD-based, others binary blobs)
- **Tag ID schemes** (often overlapping between manufacturers)
- **Binary formats** (fixed-length records, variable-length, encrypted sections)
- **Versioning** (different camera generations use different layouts)

**Why they're complex**: No standards, reverse-engineered from camera behavior, model-specific variations, sometimes encrypted/obfuscated.

## Phase 1 Learnings Applied to Phase 2

From our successful Phase 1 multi-format work, key patterns to apply:

### 1. Table-Driven Architecture Works

- Generated tag tables from ExifTool Perl sources (like we did in `build.rs`)
- Static lookup tables for O(1) tag access
- Consistent trait-based architecture across manufacturers

### 2. Memory-Efficient Parsing

- Don't load entire maker note sections unless needed
- Stream-based parsing for large binary structures
- Early termination when finding specific tags

### 3. Robust Error Handling

- Graceful degradation when maker note parsing fails
- Continue extracting standard EXIF even if maker notes fail
- Comprehensive bounds checking (maker notes often have invalid offsets)

### 4. ExifTool Compatibility Testing

- Use the `tests/exiftool_compatibility.rs` pattern we established
- Compare output with ExifTool for validation
- Test with real-world files from `exiftool/t/images/`

## Reference Implementation Study Guide

**Essential files to understand** before starting any manufacturer:

### `src/maker/canon.rs` - The Template

```rust
// Key patterns to follow:
1. MakerNoteParser trait implementation
2. Manufacturer detection via Make tag
3. Tag ID prefixing system (0xC000 + original_tag_id)
4. Binary data extraction with bounds checking
5. Error handling that allows fallback to standard EXIF
```

### `src/maker/mod.rs` - The Dispatch System

```rust
// Study how Canon is integrated:
1. MakerNoteParser trait definition
2. Manufacturer detection and routing
3. Tag prefix management to avoid ID conflicts
4. Integration with main EXIF parsing flow
```

### `build.rs` - Table Generation

```rust
// Canon tag table generation pattern:
1. Perl module parsing with regex
2. Tag ID extraction and validation
3. Static table generation for runtime lookup
4. Format and group information preservation
```

**Critical Tribal Knowledge**: The Canon implementation works because it follows ExifTool's exact logic. **Don't try to "improve" the approach** - compatibility is more important than elegance.

### Key Insights from Canon Implementation

**1. Maker Note Offset Handling**

```rust
// Canon stores maker notes with relative offsets
// ExifTool has complex logic for this - study MakerNoteOffset in Canon.pm
// Our implementation handles this in detect_canon_maker_note()
```

**2. Binary Data Extraction Pattern**

```rust
// Canon binary tags (like CameraSettings) use this pattern:
// 1. Find tag pointing to binary data
// 2. Extract binary blob with bounds checking
// 3. Apply ProcessBinaryData-style extraction
// 4. Generate multiple tags from single binary blob
```

**3. Error Recovery Strategy**

```rust
// Critical pattern: never let maker note parsing break main EXIF
if let Err(e) = parse_maker_note(&data) {
    warn!("Maker note parsing failed: {}", e);
    // Continue with standard EXIF tags
}
```

**4. Table Generation Lessons**

- Perl module parsing is complex but worth it for ExifTool compatibility
- Tag format information is critical (u16, u32, string, binary)
- Group information helps with tag organization
- Conditional tags (model-specific) need special handling

## IMMEDIATE (High-impact manufacturers - 2 weeks)

### 1. Nikon Maker Notes (1 week)

**Context**: Nikon is #2 camera manufacturer globally. Their maker notes are ExifTool's most complex implementation (3000+ lines).

**ExifTool source**: `lib/Image/ExifTool/Nikon.pm` (study the MAIN_NIKON, NIKON_V1, NIKON_V2 tables)

**Why Nikon is challenging**:

- **Version chaos**: V1 (D1), V2 (D70-D200), V3 (D300+) use completely different layouts
- **Encryption**: ShutterData and other sections use proprietary encryption
- **Endianness**: Some sections switch endianness mid-stream
- **Model dependencies**: Same tag ID means different things on different cameras

**Recommended Implementation Strategy**:

**Step 1: Version Detection (Days 1-2)**

```rust
// Nikon maker notes start with version signature
// Follow this pattern from Nikon.pm lines 200-250
fn detect_nikon_version(data: &[u8]) -> Result<NikonVersion> {
    if data.starts_with(b"Nikon\x00\x01\x00") {
        NikonVersion::V1
    } else if data.starts_with(b"Nikon\x00\x02") {
        NikonVersion::V2
    } else if data.len() >= 10 && &data[0..5] == b"Nikon" {
        NikonVersion::V3  // Most modern cameras
    } else {
        NikonVersion::Unknown
    }
}
```

**Step 2: Table Generation (Days 2-3)**

- Start with V2/V3 tags only (V1 is legacy)
- Focus on MAIN_NIKON table from lines 400-800 in Nikon.pm
- Use tag prefix `0x4E00` (N in hex = 4E) + original tag ID

**Step 3: IFD Parsing with Nikon Quirks (Days 3-4)**

- Nikon V2+ stores IFD after "Nikon\x00\x02\x00\x00" header (10 bytes)
- Some cameras have maker note offset bugs (study ExifTool's FixBase logic)
- Handle encrypted sections gracefully - don't fail entire parsing

**Step 4: Testing & Validation (Days 5-7)**

- Test with D850, D780, Z6/Z7 files (modern V3 format)
- Compare tag extraction with ExifTool output
- Verify binary data sections are correctly identified

**Files to create**:

- `src/maker/nikon.rs` - Main parser (follow Canon's trait pattern exactly)
- `build.rs` extension - Parse MAIN_NIKON table from Nikon.pm
- `src/tables/nikon_tags.rs` (generated) - ~200 tags initially

**Tribal Knowledge**:

- **Don't implement encryption initially** - Mark as binary data, implement later if needed
- **Focus on popular cameras** - D850, D780, Z6, Z7, Z9 represent 80% of users
- **Endianness changes mid-stream** - Some Nikon sections switch from big-endian to little-endian
- **Offset calculations are tricky** - Nikon has more offset bugs than other manufacturers

**Testing with**:

- `exiftool/t/images/Nikon.nef` (if available)
- Priority: Find D850, Z6 sample files for testing modern V3 format

### 2. Sony Maker Notes (1 week)

**Context**: Sony/Sony Alpha is #3 manufacturer globally. Complex due to Minolta heritage and frequent format changes.

**ExifTool source**: `lib/Image/ExifTool/Sony.pm` (study MAIN_SONY table, lines 300-600)

**Why Sony is challenging**:

- **Minolta legacy**: Older cameras use Minolta maker note format
- **Model variations**: A7 series vs FX series use different binary structures
- **Generation gaps**: Mark I, II, III, IV cameras often incompatible
- **Mixed formats**: Some tags are IFD-based, others are binary blobs

**Recommended Implementation Strategy**:

**Step 1: Sony vs Minolta Detection (Days 1-2)**

```rust
// Sony maker notes identification
fn detect_sony_format(make: &str, data: &[u8]) -> SonyFormat {
    if make.contains("SONY") {
        if data.starts_with(b"SONY DSC") { SonyFormat::DSC }     // Point & shoot
        else if data.len() >= 12 { SonyFormat::Modern }         // Alpha series
        else { SonyFormat::Legacy }
    } else if make.contains("MINOLTA") {
        SonyFormat::Minolta  // Legacy Minolta cameras
    } else {
        SonyFormat::Unknown
    }
}
```

**Step 2: Table Generation Focus (Days 2-3)**

- Start with MAIN_SONY table (modern Alpha series)
- Skip Minolta tables initially (legacy support)
- Use tag prefix `0x534F` (SO = Sony in hex) + original tag ID
- Focus on high-value tags: ISO, WhiteBalance, LensInfo

**Step 3: Binary Data Handling (Days 3-5)**

- Sony heavily uses binary data for lens information, camera settings
- Many tags point to binary blobs that need ProcessBinaryData
- Study ExifTool's Sony::ProcessSonyLensInfo for reference pattern

**Step 4: ARW vs JPEG Differences (Days 5-6)**

- ARW files have additional maker note sections not in JPEG
- Some tags only appear in RAW files
- Test both formats to ensure compatibility

**Step 5: Testing & Validation (Days 6-7)**

- Test with A7R series, FX series sample files
- Verify lens detection works correctly (major Sony feature)
- Compare tag extraction rates with ExifTool

**Files to create**:

- `src/maker/sony.rs` - Main parser implementation
- `build.rs` extension - Parse MAIN_SONY table from Sony.pm
- `src/tables/sony_tags.rs` (generated) - ~150 tags initially

**Sony-specific tribal knowledge**:

- **Lens detection is critical** - Sony users care most about accurate lens identification
- **Binary data sections are large** - Don't try to load everything, use streaming
- **Model differences are extreme** - A7 vs FX6 vs RX100 are completely different cameras
- **Minolta compatibility** - Many users have old Minolta lenses on Sony bodies

**Common Sony tag priorities** (implement these first):

- `0x0114` - CameraSettings (binary data)
- `0x0115` - WhiteBalance
- `0x0116` - Focus mode
- `0x0117` - AFAreaMode
- `0xB028` - LensInfo (critical for lens identification)

**Testing with**:

- `exiftool/t/images/Sony.arw` (if available)
- Priority: Find A7R series sample files (most popular)

## SHORT-TERM (Secondary manufacturers - 2 weeks)

**Context**: These manufacturers are simpler to implement but still important for market coverage. Use them to refine patterns before tackling complex binary processing.

### 3. Olympus Maker Notes (2-3 days)

**Context**: Clean IFD structure, excellent learning case for the patterns. Market share: ~5% but loyal users.

**Why start with Olympus**:

- Standard IFD structure (no binary blobs)
- Predictable tag layout across camera models
- Good ExifTool test files available
- Fewer edge cases than Nikon/Sony

**ExifTool source**: `lib/Image/ExifTool/Olympus.pm` (focus on MAIN_OLYMPUS table)

**Implementation approach** (follow Canon exactly):

```rust
// Olympus maker notes are straightforward IFD
// Look for "OLYMPUS\x00II" or "OLYMPUS\x00MM" signature
// Then standard IFD parsing after 8-byte header
```

**Files to create**:

- `src/maker/olympus.rs` (150-200 lines, simpler than Canon)
- Tag prefix: `0x4F4C` (OL = Olympus)
- ~80 tags initially from MAIN_OLYMPUS table

**Tribal knowledge**: Olympus users care about image stabilization settings and art filters.

### 4. Pentax Maker Notes (2-3 days)

**Context**: Very standard implementation, almost identical to Canon pattern.

**Why Pentax is easy**:

- Pure IFD structure, no binary data
- Consistent across all Pentax/Ricoh cameras
- Limited tag set (~50 meaningful tags)

**ExifTool source**: `lib/Image/ExifTool/Pentax.pm` (MAIN_PENTAX table)

**Files to create**:

- `src/maker/pentax.rs` (simpler than Canon, ~120 lines)
- Tag prefix: `0x5045` (PE = Pentax)

**Implementation time**: Fastest of all manufacturers to implement.

### 5. Fujifilm Maker Notes (3-4 days)

**Context**: More complex due to X-Trans sensor specifics and RAF format integration.

**Why Fujifilm is moderate complexity**:

- Film simulation settings (unique to Fujifilm)
- X-Trans sensor-specific data
- RAF format has additional maker note sections

**ExifTool source**: `lib/Image/ExifTool/Fujifilm.pm` (MAIN_FUJIFILM table)

**Files to create**:

- `src/maker/fujifilm.rs`
- Tag prefix: `0x4655` (FU = Fujifilm)
- Focus on film simulation and face detection tags

**Tribal knowledge**: Fujifilm users obsess over film simulation settings and color profiles.

### 6. Panasonic Maker Notes (3-4 days)

**Context**: Binary data processing, good bridge to ProcessBinaryData framework.

**Why Panasonic is important**:

- Growing market share (especially video)
- RW2 format integration
- Some binary data sections (preparation for complex manufacturers)

**ExifTool source**: `lib/Image/ExifTool/Panasonic.pm` (MAIN_PANASONIC table)

**Files to create**:

- `src/maker/panasonic.rs`
- Tag prefix: `0x5041` (PA = Panasonic)
- Include basic binary data handling

**Focus areas**: Video settings, lens corrections, face detection.

## MEDIUM-TERM (ProcessBinaryData framework - 1 week)

### 7. ProcessBinaryData Implementation

**Context**: Critical infrastructure needed by Nikon, Sony, and others. This is ExifTool's secret weapon for handling proprietary binary formats.

**Why ProcessBinaryData is essential**:

- **Nikon**: Encrypted ShutterData, FocusDistance, FlashInfo
- **Sony**: LensInfo, CameraSettings, entire maker note sections
- **Canon**: Some advanced tags we haven't implemented yet
- **Others**: Every manufacturer has some binary data

**ExifTool source**: Study these carefully:

- `lib/Image/ExifTool.pm` lines 4000-4200 (ProcessBinaryData function)
- `lib/Image/ExifTool/Canon.pm` lines 2000+ (CanonCameraSettings table)
- `lib/Image/ExifTool/Nikon.pm` lines 1500+ (Nikon binary tables)

**What ProcessBinaryData does**:

1. Takes binary blob + format specification
2. Extracts values at specific byte offsets
3. Converts raw bytes to meaningful values
4. Handles endianness, signed/unsigned, bit fields
5. Supports conditional extraction based on camera model

**Implementation Strategy**:

**Step 1: Core Framework (Days 1-3)**

```rust
// Binary format specification
#[derive(Debug)]
pub struct BinaryFormat {
    pub entries: Vec<BinaryEntry>,
}

#[derive(Debug)]
pub struct BinaryEntry {
    pub offset: i32,        // Negative = count from end
    pub format: DataFormat, // u8, u16, u32, string, etc.
    pub count: usize,       // For arrays
    pub tag_id: u16,        // Output tag ID
    pub condition: Option<Condition>, // Model-specific
}

// Core processor
pub struct BinaryDataProcessor;

impl BinaryDataProcessor {
    pub fn process(&self, data: &[u8], format: &BinaryFormat) -> Result<Vec<Tag>> {
        // Extract values according to format specification
        // Handle endianness conversions
        // Apply model-specific conditions
    }
}
```

**Step 2: Format Definitions (Days 3-5)**

```rust
// Canon camera settings format (existing reference)
const CANON_CAMERA_SETTINGS: BinaryFormat = BinaryFormat {
    entries: vec![
        BinaryEntry { offset: 1, format: DataFormat::U16, tag_id: 0xC001, .. },
        BinaryEntry { offset: 2, format: DataFormat::U16, tag_id: 0xC002, .. },
        // ... hundreds more
    ]
};

// Nikon ShutterData format
const NIKON_SHUTTER_DATA: BinaryFormat = BinaryFormat {
    entries: vec![
        BinaryEntry { offset: 0, format: DataFormat::U32, tag_id: 0x4E01, .. },
        // ... Nikon-specific entries
    ]
};
```

**Step 3: Integration with Makers (Days 5-7)**

- Update Canon parser to use new framework
- Add binary processing to Nikon parser
- Add binary processing to Sony parser
- Verify existing Canon binary tags still work

**Files to create**:

- `src/binary/mod.rs` - Core ProcessBinaryData implementation
- `src/binary/formats.rs` - Format definitions for each manufacturer
- `src/binary/processor.rs` - Main processing logic
- Integration updates to `src/maker/canon.rs`, `nikon.rs`, `sony.rs`

**Critical features to implement**:

- **Negative indexing**: `offset: -4` means "4 bytes from end"
- **Conditional extraction**: Only extract if camera model matches
- **Endianness handling**: Binary data might be different endian than main IFD
- **Variable length**: Some records have dynamic length
- **Bit field extraction**: Extract specific bits from bytes

**Tribal knowledge from Phase 1**:

- **Memory efficiency matters**: Don't copy large binary blobs unnecessarily
- **Error recovery**: Binary parsing should never fail entire maker note parsing
- **Bounds checking**: Binary data often has invalid offsets
- **Model detection**: Use existing camera model detection for conditions

**Testing strategy**:

- Start with Canon binary data (we know it works)
- Add Nikon ShutterData (encrypted, but format is known)
- Add Sony LensInfo (critical for users)
- Compare extracted values with ExifTool output

**Performance target**: ProcessBinaryData should add <2ms to parsing time.

## LONG-TERM (Comprehensive coverage - ongoing)

### 8. Remaining Manufacturers

**Context**: Complete coverage for all manufacturers we detect.

**Follow Canon pattern for each**:

- Leica: `src/maker/leica.rs` (tag prefix 0x12000)
- Samsung: `src/maker/samsung.rs` (tag prefix 0x13000)
- Sigma: `src/maker/sigma.rs` (tag prefix 0x14000)
- Hasselblad: `src/maker/hasselblad.rs` (tag prefix 0x15000)
- Phase One: `src/maker/phaseone.rs` (tag prefix 0x16000)
- GoPro: `src/maker/gopro.rs` (tag prefix 0x17000)
- Others as needed

**Approach**: Same exact pattern as Canon, just different ExifTool source files.

### 9. Advanced Binary Processing

**Context**: Handle encrypted sections, model-specific variations, complex binary structures.

**Nikon encryption**: Implement ShutterData decryption (if legally permissible)
**Sony compression**: Handle compressed binary sections
**Model detection**: Extend binary processing based on camera model

### 10. Comprehensive Testing & Validation

**Context**: Ensure all manufacturer parsers work correctly and consistently.

**Test coverage**:

- All manufacturers with ExifTool test images
- Real-world RAW files from each manufacturer
- Performance benchmarks (should add <5ms per manufacturer)
- Compatibility validation against ExifTool output

**Validation approach**:

```bash
# For each manufacturer
exiftool -struct -json manufacturer_test.raw > exiftool.json
cargo run -- manufacturer_test.raw > ours.json
# Compare tag extraction coverage and values
```

## Technical Architecture

### Tag Prefixing System (Critical to Avoid Conflicts)

**Why prefixing is essential**: Manufacturers reuse tag IDs. Canon's tag 0x0001 means something completely different from Nikon's tag 0x0001. We solve this by adding manufacturer-specific prefixes.

**Established prefixes**:

- **Canon**: `0xC000` + tag_id (C = Canon, established in current implementation)
- **Nikon**: `0x4E00` + tag_id (4E = hex for 'N', fits in u16 range)
- **Sony**: `0x534F` + tag_id (534F = hex for 'SO')
- **Olympus**: `0x4F4C` + tag_id (4F4C = hex for 'OL')
- **Pentax**: `0x5045` + tag_id (5045 = hex for 'PE')
- **Fujifilm**: `0x4655` + tag_id (4655 = hex for 'FU')
- **Panasonic**: `0x5041` + tag_id (5041 = hex for 'PA')

**Implementation pattern**:

```rust
// In each manufacturer's parser
const TAG_PREFIX: u16 = 0x4E00;  // Nikon example

fn prefixed_tag_id(original_id: u16) -> u16 {
    TAG_PREFIX + original_id
}

// Usage
let nikon_iso_tag = prefixed_tag_id(0x0002);  // Results in 0x4E02
```

**Tag collision detection**: Build system should verify no conflicts between prefixed ranges.

### Consistent Implementation Pattern

**Follow Canon exactly**:

1. MakerNoteParser trait implementation
2. Manufacturer detection in main dispatch
3. Table generation in build.rs
4. Generated tag lookup tables
5. Same error handling approach
6. Same testing patterns

### Code Reuse Strategy

- **Table generation**: Extend existing build.rs Perl parsing
- **IFD parsing**: Reuse existing IFD parser from Canon
- **Binary extraction**: Follow Canon binary tag patterns
- **Error handling**: Use same Result<> patterns as Canon

## Development Methodology & Best Practices

### Implementation Order (Critical for Success)

1. **Start simple**: Olympus → Pentax → Fujifilm → Panasonic
2. **Learn patterns**: Use simple manufacturers to refine your approach
3. **Build infrastructure**: ProcessBinaryData framework
4. **Tackle complex**: Nikon → Sony with infrastructure in place

### Testing Strategy (Learned from Phase 1)

```bash
# For each manufacturer implementation:

# 1. Table generation test
cargo build  # Should generate tables without errors

# 2. Basic parsing test
cargo test maker_tests::test_nikon_basic_parsing

# 3. ExifTool comparison test
exiftool -json -g -n test_nikon.nef > expected.json
cargo run -- test_nikon.nef --json > actual.json
# Compare tag extraction coverage

# 4. Performance test
cargo test --test performance_validation -- test_nikon_performance
```

### Common Pitfalls to Avoid

**1. Don't optimize too early**

- Get tag extraction working first
- Worry about performance only after basic functionality works
- ProcessBinaryData framework can be optimized later

**2. Don't implement everything at once**

- Start with 20-30 most important tags per manufacturer
- Focus on tags users actually care about (ISO, lens info, white balance)
- Binary data can be marked as "binary" initially

**3. Don't break existing functionality**

- Always test Canon parsing still works after changes
- Run full Phase 1 test suite after major changes
- Maker note parsing should never break standard EXIF parsing

**4. Don't ignore ExifTool's quirks**

- If ExifTool has weird offset calculations, copy them exactly
- ExifTool's "bugs" are often workarounds for camera firmware bugs
- Compatibility matters more than "correct" implementation

### Phase 1 Integration Points

**Use existing infrastructure**:

- File format detection from Phase 1
- Error handling patterns from multi-format work
- Testing patterns from `tests/exiftool_compatibility.rs`
- Performance validation from `tests/performance_validation.rs`

**Memory efficiency patterns**:

- Use `TiffParseMode::MetadataOnly` for maker note extraction
- Stream binary data rather than loading entire sections
- Apply early termination patterns when possible

## Success Criteria

### Functionality Requirements

- ✅ **Canon**: Already complete (reference implementation)
- [ ] **Nikon**: Major maker note tags extracted, version detection works
- [ ] **Sony**: Lens detection and basic settings extraction works
- [ ] **Olympus**: Standard IFD tags extracted correctly
- [ ] **Pentax**: Complete tag coverage (simplest manufacturer)
- [ ] **Fujifilm**: Film simulation and basic settings work
- [ ] **Panasonic**: Video settings and lens corrections extracted

### Quality Metrics

- [ ] **Tag coverage**: 70%+ compared to ExifTool for each manufacturer
- [ ] **Performance**: <5ms additional per manufacturer
- [ ] **Compatibility**: No regressions in Phase 1 functionality
- [ ] **Error handling**: Graceful degradation when maker notes fail
- [ ] **Memory usage**: No significant increase in memory consumption

### Technical Requirements

- [ ] **ProcessBinaryData**: Framework handles Nikon, Sony, Canon binary data
- [ ] **Tag prefixing**: No conflicts between manufacturers
- [ ] **Code consistency**: All manufacturers follow Canon patterns
- [ ] **Testing**: Integration tests for all manufacturers
- [ ] **Documentation**: Each manufacturer has clear documentation

## Next Steps After Phase 2

**Immediate follow-up work**:

1. **Performance optimization**: SIMD, memory mapping for large binary sections
2. **Advanced binary processing**: Encrypted section handling, compression
3. **Remaining manufacturers**: Leica, Samsung, Sigma, Hasselblad, Phase One
4. **Enhanced testing**: More real-world sample files, edge case handling

**Phase 3 preparation**:

- Write support will need maker note preservation
- ProcessBinaryData framework enables write-back of binary data
- Tag modification needs to handle manufacturer-specific constraints
