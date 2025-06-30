# exif-oxide Implementation Milestones - Take 5

This document outlines the incremental development milestones for exif-oxide, incorporating the ARCHITECTURE-TAKE-5 design with runtime fallback TODO system, stateful reader architecture, hybrid processor dispatch, and layered offset management.

## Core Principles

1. **Always Working**: Every milestone produces runnable code with graceful fallbacks
2. **No Panics**: Missing implementations return raw values, never crash
3. **Demand-Driven**: Only implement what's needed for real test images
4. **Manual Excellence**: Complex logic is manually ported with ExifTool references
5. **Transparent Progress**: Runtime tracking shows exactly what's missing

---

## Milestone 0: Foundation & Tooling (2 weeks)

**Goal**: Establish the build pipeline with runtime fallback system

**Deliverables**:

- [ ] Minimal Perl extraction script (`extract_tables.pl`)
  - Extract EXIF IFD0 tags from `lib/Image/ExifTool/Exif.pm`
  - Include PrintConv/ValueConv as string references
  - Output clean JSON with all tag metadata
- [ ] Rust codegen tool with new TODO UX
  - Generate tag tables with string references (no stubs!)
  - Create runtime registry structure
  - Generate ProcessorType enum (~50 variants)
- [ ] Implementation registry skeleton
  - Runtime lookup for PrintConv/ValueConv
  - Missing implementation tracking system
  - `--show-missing` and `--generate-stubs` CLI support
- [ ] Test harness with graceful fallback
  - Compare output with ExifTool
  - Track missing implementations per image

**Success Criteria**:

- Generated code compiles and runs on JPEG with missing implementations
- `--show-missing` correctly identifies what needs implementation
- Raw values displayed when PrintConv missing (no panics!)

---

## Milestone 1: Stateful Reader & Basic JPEG (2-3 weeks)

**Goal**: Implement stateful ExifReader and JPEG segment parsing

**Deliverables**:

- [ ] ExifReader state management (from STATE-MANAGEMENT.md)
  - PROCESSED hash for recursion prevention
  - VALUES hash for extracted tags
  - Directory context (path, base, byte order)
- [ ] JPEG segment parser
  - Find APP1 (EXIF) segments
  - Handle segment markers with streaming approach
  - Basic error recovery for truncated files
- [ ] File type detection
  - Magic number patterns for JPEG, TIFF
  - MIME type matching ExifTool
- [ ] Integration test suite
  - Test against t/images/ExifTool.jpg
  - Show missing implementations

**Success Criteria**:

- Stateful reader maintains context through directory traversal
- JPEG parser finds EXIF data in all test images
- `--show-missing` reveals needed PrintConv implementations

**Manual Implementations Needed**: None yet (using raw values)

---

## Milestone 2: ProcessExif & Basic IFD Structure (3-4 weeks)

**Goal**: Implement ProcessExif with basic offset management

**Deliverables**:

- [ ] ProcessExif implementation
  - IFD entry parsing
  - TIFF header and endianness handling
  - Basic offset calculations (no quirks yet)
- [ ] DirectoryContext (from OFFSET-BASE-MANAGEMENT.md)
  - Core offset formula: `absolute = base + data_pos + relative`
  - Standard TIFF offset scheme only
- [ ] Basic tag extraction
  - String tags (Make, Model, DateTime)
  - Numeric tags (raw values only)
  - Format support: BYTE, ASCII, SHORT, LONG
- [ ] Hybrid processor dispatch foundation
  - ProcessorType::Exif variant
  - Basic dispatch without conditions

**Success Criteria**:

- Extract Make, Model, DateTime from test images
- Correct offset calculations for all standard EXIF tags
- Handle both endianness correctly

**Manual Implementations**:

- `process::exif::process_exif` - core IFD parser

---

## Milestone 3: First PrintConv Implementations (2 weeks)

**Goal**: Implement high-frequency PrintConv based on real images

**Deliverables**:

- [ ] Run on 100 test images to find common missing PrintConv
- [ ] Implement top 10 by frequency:
  - EXIF:Orientation lookup (very common)
  - EXIF:Flash BITMASK operation
  - EXIF:ExposureProgram lookup
  - DateTime formatting
  - (Others based on actual usage)
- [ ] PrintConv registry integration
  - Register implementations
  - Runtime lookup working
- [ ] Metrics collection
  - Track hit rate of implemented conversions
  - Show coverage improvement

**Success Criteria**:

- 10 most common PrintConv implementations working
- Coverage jumps from 0% to ~40% on test images
- `--show-missing` shows reduced missing count

**Manual Implementations**:

- `print_conv::orientation` - 8 value lookup
- `print_conv::flash` - BITMASK with 7 bits
- (8 more based on frequency analysis)

---

## Milestone 4: SubDirectory & ExifIFD (3 weeks)

**Goal**: Handle nested IFDs with recursion prevention

**Deliverables**:

- [ ] SubDirectory tag support
  - Follow pointers to ExifIFD
  - Implement PROCESSED tracking
  - PATH stack management
- [ ] Processor dispatch with overrides
  - Table-level PROCESS_PROC
  - SubDirectory ProcessProc override
  - Still no conditions (unconditional dispatch)
- [ ] Additional IFD support
  - ExifIFD tags (exposure, camera settings)
  - IFD0 → ExifIFD chain
  - GPS IFD (simpler structure)

**Success Criteria**:

- Extract nested EXIF data correctly
- No infinite loops on circular references
- GPS coordinates extract when present

**Manual Implementations**:

- Recursion prevention logic
- Path tracking through directory tree

---

## Milestone 5: RATIONAL & Basic ValueConv (2-3 weeks)

**Goal**: Implement RATIONAL types and high-frequency ValueConv

**Deliverables**:

- [ ] RATIONAL/SRATIONAL format support
  - Parse correctly with zero handling
  - Efficient fraction representation
- [ ] Run analysis to find common ValueConv patterns
- [ ] Implement top conversions:
  - ShutterSpeedValue (2^-x formula)
  - ApertureValue (2^(x/2) formula)
  - Simple mathematical conversions
- [ ] ValueConv registry integration
  - Same pattern as PrintConv
  - Graceful fallback to raw

**Success Criteria**:

- Shutter speed shows "1/250" not raw APEX
- Aperture shows "f/2.8" not raw value
- ~30% of ValueConv covered by frequency

**Manual Implementations**:

- `value_conv::apex_shutter_speed`
- `value_conv::apex_aperture`
- (Others based on frequency)

---

## Milestone 6: ProcessBinaryData Simple Patterns (4 weeks)

**Goal**: Core ProcessBinaryData with fixed formats only

**Deliverables**:

- [ ] ProcessBinaryData for fixed formats
  - int8u, int16u, int32u arrays
  - Fixed string[N] formats
  - Basic binary extraction
- [ ] Canon CameraSettings as test case
  - Fixed format binary data
  - Many discrete values
  - Good PrintConv coverage opportunity
- [ ] Format pattern registry
  - Parse "int16u[10]" patterns
  - Generate extraction code
- [ ] Bit-level extraction
  - Mask support for packed bits
  - BitShift operations

**Success Criteria**:

- Canon CameraSettings fully decoded
- All fixed-format patterns working
- Discover variable-format needs

**Manual Implementations**:

- `process::binary_data::process_binary_data` (fixed formats only)
- `formats::fixed::parse_fixed_array`

---

## Milestone 7: First MakerNote - Canon (4-5 weeks)

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

## Milestone 8: Conditional Dispatch (3 weeks)

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

## Milestone 9: Variable ProcessBinaryData (3-4 weeks)

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

## Milestone 10: Error Classification System (2-3 weeks)

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

## Milestone 11: Second Manufacturer - Nikon (4-5 weeks)

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

## Milestone 12: Performance & Coverage Analysis (2 weeks)

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
20. **Complete Coverage** - Remaining conversions

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
