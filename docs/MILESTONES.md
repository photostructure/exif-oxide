# exif-oxide Implementation Milestones - Take 5

This document outlines the incremental development milestones for exif-oxide.

## Important Preequisite Steps

1. Be sure to study $REPO_ROOT/CLAUDE.md, $REPO_ROOT/docs/ARCHITECTURE.md, $REPO_ROOT/third-party/exiftool/CLAUDE.md, and all relevant related documentation before starting any work. With this project, **everything** is more complicated than you'd expect.

2. Reread ../CLAUDE.md's `ExifTool is Gospel` section.

## Important Milestone Validation Steps

After you think you're done implementing a milestone:

1. Read $REPO_ROOT/tests/exiftool_compatibility_tests.rs and $REPO_ROOT/tools/generate_exiftool_json.sh

2. Your task should allow more tags to be include-listed in exiftool_compatibility_tests.rs, more file types to be included by generate_exiftool_json.sh, or remove excluded files from exiftool_compatibility_tests.rs. Edit the above test files accordingly.

3. Re-run `make compat` and iterate until all tests pass.

4. Run `make precommit` and fix linting, compilation, and test errors

5. After completing a milestone, edit this document and replace the milestone section that you completed with a summary of the task.

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

**Note**: This milestone established the *infrastructure* for subdirectory processing. Actual extraction of specific tags from ExifIFD and GPS subdirectories will be implemented in Milestone 6 along with RATIONAL format support and ValueConv/PrintConv for camera settings.

---

## Milestone 6: RATIONAL Format & GPS (2 weeks)

**Goal**: Implement RATIONAL types and high-frequency ValueConv

**Deliverables**:

- [ ] RATIONAL/SRATIONAL format
  - Parse 2x uint32 correctly
  - Handle zero denominators
  - Display as "num/den"
- [ ] GPS IFD support
  - Follow GPS IFD pointer
  - Extract GPS tags
  - GPSLatitude/Longitude as rationals
- [ ] Stateful reader completion
  - PROCESSED hash for recursion
  - VALUES hash storage
  - PATH tracking
- [ ] Basic coordinate display
  - Show raw rational arrays
  - No conversion yet

**Success Criteria**:

- GPS coordinates extracted
- Rationals display correctly
- No infinite loops on bad data

**Test Commands**:

```bash
# Files with GPS data
exif-oxide t/images/GPS.jpg | jq .GPSLatitude
# Should show [deg/1, min/1, sec/100] format
```

---

## Milestone 7: More PrintConv Implementations (1 week)

**Goal**: Implement common PrintConv patterns

**Deliverables**:

- [ ] Analyze test images for needed PrintConv
- [ ] Implement top 5-10:
  - Flash (BITMASK)
  - ExposureProgram
  - MeteringMode
  - WhiteBalance
  - ColorSpace
- [ ] BITMASK support
  - Bit flag parsing
  - Comma-separated output
- [ ] Simple lookups
  - Hash table conversions

**Success Criteria**:

- Common tags show readable values
- BITMASK works for Flash
- Coverage improves significantly

---

## Milestone 8: Basic ValueConv (2 weeks)

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

13. **XMP/XML Support** - Major format addition
14. **Advanced Nikon Encryption** - Complete crypto port/implementation
15. **Sony & Olympus** - Additional manufacturers
16. **Write Support Foundation** - Basic tag updates
17. **Video Metadata** - QuickTime/MP4 atoms
18. **Advanced Write** - MakerNote preservation
19. **RAW Formats** - CR2, NEF, ARW support
20. **Async Support** - AsyncRead/AsyncSeek wrappers
21. **Complete Coverage** - Remaining mainstream conversions

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
