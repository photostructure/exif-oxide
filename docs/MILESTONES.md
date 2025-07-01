# exif-oxide Implementation Milestones - Take 5

This document outlines the incremental development milestones for exif-oxide.

## Important Steps Before Coding

1. Be sure to study $REPO_ROOT/CLAUDE.md, $REPO_ROOT/docs/ARCHITECTURE.md, $REPO_ROOT/third-party/exiftool/CLAUDE.md, and all relevant related documentation before starting any work. With this project, **everything** is more complicated than you'd expect.

2. Be sure to follow the `ExifTool is Gospel` and `Ask clarifying questions` sections of ../CLAUDE.md

## Important Steps During Implementation

1. Be sure to follow the `ExifTool is Gospel` and `Code smell` sections of ../CLAUDE.md

## Important Milestone Validation Steps

After you think you're done implementing a milestone:

1. **Update Supported Tags Configuration**: If your milestone adds working PrintConv implementations, update the `MILESTONE_COMPLETIONS` configuration in `codegen/src/main.rs` to include your new supported tags, then run `cargo run -p codegen` to regenerate the supported tags JSON.

2. **Compatibility Testing**: Re-run `make compat` and iterate until all tests pass. The regenerated supported tags list will automatically be used by the compatibility tests.

3. **Code Quality**: Run `make precommit` and fix linting, compilation, and test errors.

## Important Steps After Completing a Milestone

1. Edit this document and replace the milestone section that you completed with a summary of the task.

## Core Principles

1. **Always Working**: Every milestone produces runnable code with graceful fallbacks
2. **No Panics**: Missing implementations return raw values, never crash
3. **Demand-Driven**: Only implement what's needed for real test images
4. **Manual Excellence**: Complex logic is manually ported with ExifTool references
5. **Transparent Progress**: Runtime tracking shows exactly what's missing

---

## ✅ Milestone 0a: Minimal CLI & Registry (COMPLETED)

**Goal**: Create testable foundation with basic CLI that outputs JSON

**Implementation Summary**:

- Created CLI with clap argument parsing (src/main.rs)
- JSON output matching ExifTool structure with array format
- Registry system with PrintConv/ValueConv runtime lookup (src/registry.rs)
- Graceful fallback to raw values for missing implementations
- Integration test framework comparing with ExifTool output
- `--show-missing` flag showing development progress

**Key Files**: `src/main.rs`, `src/registry.rs`, `src/types.rs`, `tests/integration_tests.rs`

---

## ✅ Milestone 0b: Code Generation Pipeline (COMPLETED)

**Goal**: Extract ExifTool tables and generate Rust code

**Implementation Summary**:

- Created Perl extraction script analyzing ExifTool's tag tables
- Generated `TagMetadata.json` with mainstream tags filtered by usage frequency
- Rust codegen producing `src/generated.rs` with tag constants and conversion references
- Registry initialization loading generated definitions
- CLI integration showing real ExifTool tag names

**Key Files**: `tools/extract_tag_metadata.pl`, `src/generated.rs`, `third-party/exiftool/doc/TagMetadata.json`

**Future Enhancement Needed**: The static GPS tag table added in Milestone 6 (`src/generated/tags.rs`) demonstrates the need for enhanced code generation to extract GPS and other subdirectory tables from ExifTool automatically, rather than manual static definitions.

---

## ✅ Milestone 1: File I/O & JPEG Detection (COMPLETED)

**Goal**: Read real files and detect JPEG format

**Implementation Summary**:

- File I/O with buffered reading using Read + Seek traits
- Magic byte detection for JPEG (0xFFD8) and TIFF (II/MM) files
- Complete JPEG segment scanner finding APP1 (EXIF) segments
- Proper EXIF data location after "Exif\0\0" marker
- Real file metadata extraction (size, modification date, etc.)
- Graceful error handling for unsupported formats

**Key Files**: `src/formats.rs` (detect_file_format, scan_jpeg_segments, extract_metadata)

---

## ✅ Milestone 2: Minimal EXIF Parser (COMPLETED)

**Goal**: Extract first real tags from EXIF data

**Implementation Summary**:

- Complete TIFF/EXIF header parser with endianness detection (II/MM)
- IFD (Image File Directory) parser handling 12-byte entries
- Tag extraction supporting ASCII, SHORT, and LONG formats
- ExifReader struct maintaining state and extracted values
- Real tag extraction: Make="Canon", Model="Canon EOS REBEL T3i", Orientation=8
- Proper null-termination handling for ASCII strings
- Translation of ExifTool's ProcessExif function (Exif.pm:6172-7128)

**Key Files**: `src/exif.rs` (ExifReader, TiffHeader, IfdEntry parsing)

---

## ✅ ExifTool Compatibility Testing (COMPLETED)

**Goal**: Systematic compatibility testing against ExifTool reference output

**Implementation Summary**:

- ExifTool reference JSON generation (`tools/generate_exiftool_json.sh`)
- Compatibility test suite comparing exif-oxide vs ExifTool output (`tests/exiftool_compatibility_tests.rs`)
- JSON object comparison with field-order independence using `similar` crate
- Problematic file exclusion system for graceful handling
- Makefile integration: `make compat-gen`, `make compat-test`, `make compat`
- 58 reference files testing 53 successfully with proper exclusions

**Key Files**: `tools/generate_exiftool_json.sh`, `tests/exiftool_compatibility_tests.rs`, `generated/exiftool-json/`

---

## ✅ Milestone 3: More EXIF Formats (COMPLETED)

**Goal**: Support common numeric formats with PrintConv conversions

**Implementation Summary**:

- Enhanced EXIF format support for BYTE (uint8), SHORT (uint16), LONG (uint32) with proper endianness handling
- Fixed offset-based value extraction for data that doesn't fit in 4-byte inline storage
- Added ImageWidth, ImageHeight, XResolution, YResolution tag extraction from real images
- Implemented PrintConv conversion system with runtime registry for human-readable output
- Manual PrintConv implementations for Orientation ("Rotate 270 CW"), ResolutionUnit ("inches"), YCbCrPositioning ("Co-sited")
- Hex display format for unknown tags (Tag_8769, Tag_0132) matching ExifTool
- 51/51 compatibility tests now pass with exact ExifTool output matching
- Excluded 7 problematic files (thermal imaging, specialized formats)

**Key Files**: `src/exif.rs` (enhanced format extraction), `src/implementations/print_conv.rs` (manual conversions), `src/registry.rs` (conversion registry)

---

## ✅ Milestone 4: First PrintConv - Orientation (COMPLETED)

**Goal**: Implement first human-readable conversion

**Implementation Summary**:

- PrintConv registry system with runtime lookup and graceful fallback (src/registry.rs)
- Orientation PrintConv with complete 8-value lookup table (src/implementations/print_conv.rs)
- ResolutionUnit and YCbCrPositioning PrintConv implementations
- Automatic registration of PrintConv functions during library initialization
- JSON output showing converted values matching ExifTool default behavior
- Coverage metrics tracking PrintConv hit/miss rates for development guidance
- Integration tests verifying exact ExifTool compatibility

**Success Criteria Met**:

- ✅ Orientation shows "Rotate 270 CW" not "8" for Canon T3i test image
- ✅ Matches ExifTool JSON output exactly (`exiftool -j`)
- ✅ Other tags (Make/Model) show appropriate values (strings vs converted)
- ✅ PrintConv registry with runtime lookup and fallback
- ✅ Coverage metrics available via --show-missing flag

**Key Files**: `src/registry.rs`, `src/implementations/print_conv.rs`, `src/implementations/mod.rs`, `src/exif.rs` (apply_conversions)

---

## ✅ Milestone 5: SubDirectory & Stateful Reader (COMPLETED)

**Goal**: Handle nested IFDs with recursion prevention

**Implementation Summary**:

- Enhanced ExifReader with stateful processing features including PROCESSED tracking for recursion prevention and PATH stack management for directory hierarchy
- Implemented comprehensive processor dispatch system with 3-level fallback: SubDirectory overrides → Directory-specific defaults → Table-level processor → Final Exif fallback
- Added DirectoryInfo and DataMemberValue types for subdirectory context management and tag dependencies
- Created ProcessorType enum supporting Exif, BinaryData, GPS, Canon, Nikon, and Generic processors with manufacturer-specific variants
- Implemented SubDirectory tag detection (ExifIFD 0x8769, GPS 0x8825, InteropIFD 0xA005, MakerNotes 0x927C) with proper pointer following
- Added stateful reader architecture maintaining extracted values and processing state across nested IFD traversal
- All existing functionality preserved with 51/51 compatibility tests passing and 21 comprehensive unit tests

**Success Criteria Met**:

- ✅ SubDirectory processing infrastructure complete with recursion prevention
- ✅ Processor dispatch system with table-level PROCESS_PROC and SubDirectory overrides
- ✅ No infinite loops on circular references through PROCESSED tracking
- ✅ Extensible architecture for future IFD types and manufacturer-specific processors

**Key Files**: `src/exif.rs` (stateful ExifReader, processor dispatch), `src/types.rs` (DirectoryInfo, ProcessorType, DataMemberValue), enhanced architecture for Milestone 6 RATIONAL format and GPS coordinate extraction

**Note**: This milestone established the _infrastructure_ for subdirectory processing. Actual extraction of specific tags from ExifIFD and GPS subdirectories will be implemented in Milestone 6 along with RATIONAL format support and ValueConv/PrintConv for camera settings.

---

## ✅ Milestone 6: RATIONAL Format & GPS (COMPLETED)

**Goal**: Implement RATIONAL types and GPS coordinate extraction

**Implementation Summary**:

- ✅ RATIONAL/SRATIONAL format support (formats 5 & 10)
  - Parse 2x uint32/int32 correctly with proper endianness
  - Handle zero denominators gracefully
  - Display as "num/den" with special cases for /1 and /0
  - Support both single rational and rational arrays
- ✅ GPS IFD subdirectory processing
  - Follow GPS IFD pointer (tag 0x8825) to GPS directory
  - Extract GPS tags with proper table lookup (GPS_TAG_BY_ID)
  - Tag source tracking for correct name resolution
  - GPSLatitude/GPSLongitude as rational arrays by default
- ✅ Enhanced stateful reader architecture
  - PROCESSED hash for recursion prevention working
  - Tag source tracking (tag_sources HashMap)
  - PATH tracking operational from Milestone 5
  - Processor dispatch system handles GPS vs EXIF tables
- ✅ GPS coordinate extraction
  - Raw rational arrays: [[54,1], [5938,100], [0,1]] format
  - GPSLatitudeRef/GPSLongitudeRef as ASCII strings
  - GPSTimeStamp as 3-element rational array
  - GPSMapDatum string extraction

**Success Criteria Met**:

- ✅ GPS coordinates extracted correctly from test file
- ✅ Rationals display as proper arrays with zero-denominator handling
- ✅ No infinite loops - recursion prevention working
- ✅ Raw rational arrays extracted correctly from EXIF data

**Key Files**: `src/exif.rs` (RATIONAL extraction), `src/types.rs` (TagValue variants), `src/generated/tags.rs` (GPS table), enhanced architecture supporting GPS-specific processing

**Note**: This milestone provides the foundation for decimal GPS coordinate conversion, which requires PrintConv/ValueConv support or CLI flags (future milestone). The static GPS tag table added here demonstrates the pattern needed for future code generation improvements.

---

## ✅ Milestone 7: More PrintConv Implementations (COMPLETED)

**Goal**: Implement common PrintConv patterns

**Implementation Summary**:

- ✅ Analyzed test images and identified 5 critical PrintConv implementations needed by mainstream tags
- ✅ Discovered Flash uses direct hash lookup (NOT bitmask) in ExifTool - implemented exact translation with all 26 Flash values
- ✅ Implemented 5 core PrintConv functions with exact ExifTool compatibility:
  - Flash PrintConv with complete value set including red-eye reduction combinations
  - ColorSpace PrintConv with Sony-specific non-standard values support
  - WhiteBalance PrintConv with standard Auto/Manual values
  - MeteringMode PrintConv with 8 standard metering modes
  - ExposureProgram PrintConv with Canon-specific "Bulb" mode support
- ✅ All implementations include proper hex formatting for unknown values matching ExifTool's PrintHex behavior
- ✅ Comprehensive test suite with 35+ test cases covering standard, edge case, and manufacturer-specific values
- ✅ Registry integration with automatic function registration during library initialization
- ✅ Full validation against ExifTool output - all 51 compatibility tests pass with exact matches

**Success Criteria Met**:

- ✅ Common tags now show human-readable values: "Auto" instead of 0, "sRGB" instead of 1, "Multi-segment" instead of 5
- ✅ Flash implemented correctly as direct lookup (corrected assumption about bitmask)
- ✅ Coverage significantly improved with 5 new PrintConv implementations supporting mainstream camera tags
- ✅ All output matches ExifTool exactly including Sony and Canon manufacturer-specific edge cases

**Key Files**: `src/implementations/print_conv.rs` (5 new functions), `src/implementations/mod.rs` (registry integration), extensive test coverage with exact ExifTool source references (lib/Image/ExifTool/Exif.pm lines 164-197, 2082-2097, 2357-2371, 2620-2638, 2809-2821)

---

## Milestone 8a: Enhanced Code Generation & DRY Cleanup

**Goal**: Eliminate manual maintenance of conversion references and supported tag lists through enhanced code generation

**Architectural Decision**: Based on analysis of implementation options, we've chosen the "Codegen-Generated Requirements" approach (documented in docs/ARCHITECTURE.md Code Generation Strategy section 4). This aligns with the project's "Simple Codegen" principle and future-proofs for manufacturer-specific tables in Milestones 9+.

**Deliverables**:

1. **Enhanced Conversion Reference Generation**

   - ✅ Modified `codegen/extract_tables.pl` to extract PrintConv/ValueConv references from EXIF and GPS tables
   - ✅ Updated `codegen/src/main.rs` to generate both `tags.rs` AND `conversion_refs.rs` from same JSON source
   - ✅ Eliminated manual maintenance - now auto-generates 33 PrintConv + 7 ValueConv references
   - ✅ Single source of truth established for all conversion requirements

2. **Configuration-Driven Supported Tags System**
   - ✅ Implemented milestone-based configuration in `codegen/src/main.rs` with `MILESTONE_COMPLETIONS` array
   - ✅ Auto-generates `config/supported_tags.json` for shell script and `supported_tags.rs` for Rust tests
   - ✅ Quality-controlled approach: only tags with working implementations and passing tests are included
   - ✅ Future-proof: simply uncomment milestone lines as new PrintConv implementations complete

**Success Criteria**:

- ✅ Zero manual maintenance of conversion reference lists - auto-generated from tag definitions
- ✅ Single source of truth for supported tags across test and generation scripts
- ✅ All existing functionality preserved with 51/51 compatibility tests passing
- ✅ Future manufacturer-specific tables automatically include their conversion references

**Configuration-Driven Approach**:
The supported tags system uses a milestone-based configuration in the codegen that ensures quality control while eliminating maintenance burden. To add new supported tags as milestones complete:

```rust
// In codegen/src/main.rs, uncomment when ready:
("Milestone 8b", &["GPSLatitude", "GPSLongitude"]),
("Milestone 9", &["MeteringMode", "WhiteBalance"]),
```

This generates both the JSON config for shell tools and Rust constants for tests, maintaining DRY principles while providing explicit quality gates.

**Benefits for Future Milestones**:

- **Milestone 8b**: ValueConv references automatically included in generated requirements
- **Milestone 9+**: Manufacturer-specific PrintConv (Canon, Nikon) automatically tracked
- **Architecture Alignment**: Follows "Simple Codegen" principle for straightforward table translations

---

## Milestone 8b: Basic ValueConv (2 weeks)

**Goal**: Mathematical value conversions

**Deliverables**:

- [ ] ValueConv registry
  - Same pattern as PrintConv
  - Chain with PrintConv
- [ ] APEX conversions
  - ShutterSpeedValue (2^-x)
  - ApertureValue (2^(x/2))
  - ExposureCompensation
- [ ] GPS coordinate conversion
  - Degrees/minutes/seconds to decimal
  - Handle hemisphere references
- [ ] FNumber from APEX

**Success Criteria**:

- Shutter shows "1/250" not APEX value
- Aperture shows "f/2.8" not APEX
- GPS shows decimal degrees

**Manual Implementations**:

```rust
fn apex_shutter_speed(val: f64) -> f64 {
    (-val).exp2()  // 2^-val
}

```

## ✅ Milestone 8c: Group-Prefixed Tag Names (COMPLETED)

**Goal**: Always output tags with group prefixes (e.g., "EXIF:Make", "GPS:GPSLatitude") to avoid tag name collisions and match ExifTool's -G mode.

**Implementation Summary**:

- ✅ Updated `get_all_tags()` method to add group prefixes based on IFD source
  - Maps IFD names to ExifTool groups: Root/IFD0 → "EXIF", GPS → "GPS", ExifIFD → "EXIF"
  - All tags now output as "{Group}:{TagName}" format
- ✅ Updated file-level tags to use "File:" prefix (FileName, FileSize, Directory, etc.)
- ✅ Added "System:" prefix for system tags (ExifDetectionStatus, ExifByteOrder, etc.)
- ✅ Modified ExifTool reference generation script to use `-G` flag
- ✅ Enhanced compatibility tests to handle group-prefixed tag names
  - Updated filtering logic to extract tag names from group-prefixed format
  - Modified test assertions to expect group prefixes
- ✅ Updated integration tests to expect group-prefixed output
- ✅ All 51 compatibility tests passing with exact ExifTool output matching

**Benefits**:

- Eliminates tag name collisions (e.g., EXIF:ColorSpace vs MakerNotes:ColorSpace)
- Provides clear context about tag origin
- Maintains full compatibility with ExifTool's -G mode output
- Sets foundation for Composite tags which will use "Composite:" prefix

**Key Files Modified**:

- `src/exif.rs` (get_all_tags method)
- `src/formats.rs` (file and system tag prefixes)
- `tools/generate_exiftool_json.sh` (added -G flag and group filtering)
- `tests/exiftool_compatibility_tests.rs` (group-aware filtering)
- `tests/integration_tests.rs` (updated assertions)

---

## ✅ Milestone 8d: Basic Composite GPS Tags (COMPLETED)

**Goal**: Implement composite GPS tags that convert raw rational arrays to decimal degrees.

**Implementation Summary**:

- ✅ Created composite tag infrastructure in `src/implementations/composite.rs`
  - CompositeTag struct with Require/Desire/ValueConv fields
  - Composite tag registry with runtime lookup
- ✅ Implemented GPS composite definitions
  - GPSLatitude: Combines GPS:GPSLatitude + GPS:GPSLatitudeRef → decimal degrees
  - GPSLongitude: Combines GPS:GPSLongitude + GPS:GPSLongitudeRef → decimal degrees
  - GPSPosition: Combines lat/lon into single formatted string
  - GPSDateTime: Combines GPSDateStamp + GPSTimeStamp into ISO format
- ✅ Added two-phase tag processing
  - Phase 1: Extract raw tags from EXIF data
  - Phase 2: Build composite tags from extracted values
- ✅ GPS coordinate conversion logic
  - Converts [[deg,1],[min,1],[sec,100]] to decimal degrees
  - Applies hemisphere reference (S/W make negative)
  - Handles missing refs with graceful fallback

**Key Files**: `src/implementations/composite.rs`, `src/exif.rs` (build_composite_tags), updated tag output with "Composite:" prefix

---

## Milestone 8e: Fix GPS ValueConv vs Composite Confusion (1 week)

**Goal**: Refactor GPS coordinate conversion from ValueConv to proper Composite tags, establishing clean architectural separation

**Context**: Currently, GPS decimal conversion is incorrectly implemented as ValueConv. This milestone fixes the conceptual confusion by moving these conversions to the Composite system where they belong.

**Deliverables**:

1. **Remove GPS ValueConv Functions**

   - Remove `gps_coordinate_value_conv` and wrapper functions from `value_conv.rs`
   - Update registry to stop registering GPS ValueConv functions
   - GPS:GPSLatitude should return raw rational arrays only

2. **Move GPS Conversions to Composite System**

   - Implement proper composite compute functions:
     ```rust
     pub fn gps_latitude_composite(
         lat: &TagValue,      // GPS:GPSLatitude (rational array)
         lat_ref: &TagValue,  // GPS:GPSLatitudeRef (N/S)
     ) -> Result<TagValue>
     ```
   - Register in composite registry, not ValueConv registry

3. **Fix GPS Tag Extraction**

   - Ensure GPS tags return raw values:
     - `GPS:GPSLatitude` → `[[54,1], [59,100], [38,1]]`
     - `GPS:GPSLatitudeRef` → `"N"`
   - Remove any ValueConv application for GPS coordinate tags

4. **Update Tests**

   - Move GPS conversion tests from `value_conv_tests.rs` to `composite_tests.rs`
   - Update integration tests to expect raw GPS values
   - Add tests for composite GPS tags returning decimals

5. **Command-line -G Flag Support**
   - `-G` or `-G0`: Show group prefixes (default)
   - No `-G` flag: Hide groups, composite tags take precedence
   - Implement tag resolution logic for no-group mode

**Success Criteria**:

- `GPS:GPSLatitude` returns raw rational array
- `Composite:GPSLatitude` returns decimal degrees
- Clear architectural separation between ValueConv and Composite
- All tests pass with proper tag values

---

## Milestone 8f: Composite Tag Codegen & Infrastructure (1 week)

**Goal**: Add code generation support for Composite tags and establish full infrastructure

**Deliverables**:

1. **Extract Composite Definitions from ExifTool**

   - Enhance `extract_tables.pl` to parse `%Image::ExifTool::Composite`
   - Extract mainstream composite tags (same frequency filter)
   - Generate JSON with composite definitions

2. **Generate Composite Tag Definitions**

   - Create `src/generated/composite_tags.rs`:
     ```rust
     pub static COMPOSITE_TAGS: &[CompositeTagDef] = &[
         CompositeTagDef {
             name: "GPSLatitude",
             group: "Composite",
             require: &["GPS:GPSLatitude", "GPS:GPSLatitudeRef"],
             compute_ref: "gps_latitude_composite",
         },
         // ... more tags
     ];
     ```

3. **Composite Building Infrastructure**

   - Add `composite_tags: HashMap<String, TagValue>` to ExifReader
   - Implement dependency resolution (single pass, no circular deps)
   - Add `get_all_tags()` logic to merge extracted + composite tags

4. **Additional Composite Tags**

   - ImageSize: Combine ImageWidth + ImageHeight
   - GPSAltitude: Combine GPSAltitude + GPSAltitudeRef
   - LensID: Derive from LensModel with PrintConv
   - ShutterSpeed: Format ExposureTime as "1/x" or "x""

5. **PrintConv for Composite Tags**
   - Some composites need PrintConv (e.g., GPSPosition formatting)
   - Chain composite compute → PrintConv if defined
   - Test complete pipeline

**Success Criteria**:

- Codegen extracts composite definitions from ExifTool
- Multiple composite tags working (not just GPS)
- Clean separation of concerns in architecture
- Foundation for future composite tags

---

## Milestone 9: ProcessBinaryData Introduction (3 weeks)

**Goal**: Core ProcessBinaryData with fixed formats only

**Deliverables**:

- [ ] ProcessBinaryData framework
  - Processor trait implementation
  - Dispatch integration
- [ ] Fixed format support
  - int16u, int16s formats
  - Fixed arrays like int16u[3]
- [ ] MakerNote detection
  - Detect Canon signature
  - Route to ProcessBinaryData
- [ ] Canon CameraSettings test
  - MacroMode (index 1)
  - FocusMode (index 7)
  - Just a few tags initially
- [ ] Index-based extraction
  - FIRST_ENTRY = 1 support

**Success Criteria**:

- Extract Canon MacroMode correctly
- ProcessBinaryData dispatch works
- Can add more tags incrementally

**Test with Canon files**:

```bash
exif-oxide t/images/Canon.jpg | jq .MacroMode
# Should show "Macro" or "Normal"
```

---

## Milestone 10: Canon MakerNote Expansion (3 weeks)

**Goal**: Complete Canon support with offset fixing

**Deliverables**:

- [ ] Canon MakerNote detection
  - Identify Canon signature
  - Detect offset scheme by model
- [ ] Canon offset management (from OFFSET-BASE-MANAGEMENT.md)
  - 4/6/16/28 byte offset schemes
  - Footer validation
  - Base adjustment
- [ ] Canon-specific processors
  - ProcessSerialData for AF info
  - Handle word-swapped values
- [ ] Canon PrintConv implementations
  - Based on actual usage in test images
  - Focus on high-frequency conversions

**Success Criteria**:

- Extract Canon maker notes from 5+ models
- AF point data decoded correctly
- Offset calculations verified correct

**Manual Implementations**:

- `canon::detect_offset_scheme`
- `canon::fix_maker_note_base`
- `process::canon::serial_data::process`
- Canon-specific PrintConv functions

---

## Milestone 11: Conditional Dispatch (2 weeks)

**Goal**: Runtime condition evaluation for processor selection

**Deliverables**:

- [ ] Condition expression types (from PROCESSOR-PROC-DISPATCH.md)
  - DataPattern(regex) for data content
  - ModelMatch(regex) for camera model
  - Simple boolean combinations
- [ ] Conditional processor dispatch
  - Evaluate conditions at runtime
  - Select appropriate processor
  - Pass parameters through HashMap
- [ ] Integration with existing processors
  - Canon model-specific tables
  - Future Nikon encryption dispatch

**Success Criteria**:

- Canon FileNumber works per model
- Correct processor selected by conditions
- No performance regression

**Manual Implementations**:

- Condition evaluation logic
- Model-specific dispatch rules

---

## Milestone 12: Variable ProcessBinaryData (3 weeks)

**Goal**: Handle variable-length formats with DataMember

**Deliverables**:

- [ ] DataMember dependency system
  - Two-phase extraction
  - Expression evaluation for `string[$val{3}]`
  - Sequential dependency resolution
- [ ] Variable format support
  - var_string with termination
  - Simple `[$val{N}]` references only
- [ ] Canon advanced formats
  - Serial data with NumAFPoints dependency
  - Variable-length arrays

**Success Criteria**:

- Variable-length formats extract correctly
- DataMember dependencies resolved
- Canon AF arrays sized dynamically

**Manual Implementations**:

- `formats::variable::parse_string_from_val`
- DataMember resolution logic

---

## Milestone 13: Error Classification System (2 weeks)

**Goal**: Port ExifTool's sophisticated error handling

**Deliverables**:

- [ ] Error classification (Fatal/Minor/Warning)
  - MINOR_ERRORS compatibility
  - Continue-on-error behavior
- [ ] Manufacturer quirk handling
  - Samsung entry count fix
  - Known corruptions
- [ ] Comprehensive validation
  - Offset bounds checking
  - Entry count validation
  - Format verification
- [ ] Error context tracking
  - Full path to error location
  - Offset information

**Success Criteria**:

- Process 1000 files without crashing
- Known problematic files handled gracefully
- Error messages match ExifTool's

**Manual Implementations**:

- Error classification system
- Manufacturer-specific workarounds

---

## Milestone 14: Second Manufacturer - Nikon (4 weeks)

**Goal**: Prove architecture with encrypted maker notes

**Deliverables**:

- [ ] Nikon MakerNote detection
  - Multiple format versions
  - Encryption detection
- [ ] Nikon offset schemes
  - TIFF header at 0x0a
  - Version-specific handling
- [ ] ProcessNikonEncrypted
  - Basic decryption (defer full crypto)
  - Conditional dispatch by version
- [ ] Nikon-specific implementations
  - High-frequency PrintConv
  - Essential ValueConv

**Success Criteria**:

- Basic Nikon data extraction
- Correct format version detection
- Encryption detected (if not decrypted)

**Manual Implementations**:

- `nikon::detect_format_version`
- `nikon::fix_nikon_base`
- `process::nikon::encrypted::process` (skeleton)

---

## Milestone 15: Performance & Coverage Analysis (2 weeks)

**Goal**: Optimize and assess implementation coverage

**Deliverables**:

- [ ] Performance profiling
  - Benchmark vs ExifTool
  - Identify bottlenecks
  - Memory usage analysis
- [ ] Coverage metrics
  - Tag coverage by manufacturer
  - PrintConv/ValueConv hit rates
  - Missing implementation reports
- [ ] Optimization pass
  - Lazy extraction where possible
  - Efficient registry lookups
  - Memory-mapped file option
- [ ] Priority report for next phase
  - Most-needed implementations
  - Cost/benefit analysis

**Success Criteria**:

- Performance within 10x of ExifTool
- Clear roadmap for continued development
- 60%+ coverage of common tags

---

## Future Milestones (Priority Order Based on Analysis)

1. **XMP/XML Support** - Major format addition
1. **RAW Formats** - CR2, CR3, NEF, ARW, RAF, RW2, MRW, ... support
1. **Sony & Olympus** - Additional manufacturers
1. **Write Support Foundation** - Basic tag updates
1. **Video Metadata** - QuickTime/MP4 atoms
1. **Advanced Write** - MakerNote preservation
1. **ImageDataHash** - See <https://exiftool.org/ExifTool.html#ImageHashType>
1. **MIE Support** - Major format addition
1. **Async Support** - AsyncRead/AsyncSeek wrappers
1. **Advanced Nikon Encryption** - Complete crypto port/implementation
1. **Complete Coverage** - Remaining mainstream conversions

## Development Strategy Updates

### Always Shippable

- Every milestone runs and extracts data
- Missing features show raw values, not errors
- Coverage grows incrementally

### Demand-Driven Implementation

- Use `--show-missing` to guide development
- Only implement what real images need
- Track frequency to prioritize work
- Focus on mainstream tags (>80% frequency)

### Manual Excellence

- Each complex feature manually implemented
- Always reference ExifTool source
- Build expertise through careful porting

### Transparent Progress

- Runtime metrics show coverage
- Missing implementation logs guide work
- Users see exactly what's not supported

## Risk Mitigation Updates

- **No Stub Explosion**: Runtime references prevent code bloat
- **No Panic Risk**: Fallback system ensures stability
- **Incremental Complexity**: Each milestone adds one hard thing
- **Real-World Focus**: Test images drive implementation priority
- **Clear Scope**: ~50 processors enumerable, not infinite
- **Mainstream Focus**: ~500-1000 tags instead of 15,000+

This milestone plan embraces the reality that we're building a complex system incrementally. By using runtime fallbacks and demand-driven development, we can ship useful functionality immediately while building toward complete ExifTool compatibility over time.
