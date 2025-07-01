# exif-oxide Implementation Milestones

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

1. Remove the completed milestone section from this document.
2. Concisely summarize the completed work in docs/MILESTONES-DONE.md

## Core Principles

1. **Always Working**: Every milestone produces runnable code with graceful fallbacks
2. **No Panics**: Missing implementations return raw values, never crash
3. **Demand-Driven**: Only implement what's needed for real test images
4. **Manual Excellence**: Complex logic is manually ported with ExifTool references
5. **Transparent Progress**: Runtime tracking shows exactly what's missing

---

## Milestone 8b: Basic ValueConv (2 weeks)

**Before starting -- review the current code and future pending milestones -- is this still relevant work?**

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

## Milestone 11.5: Multi-Pass Composite Building (1 week)

**Goal**: Enhance composite tag infrastructure to support composite-on-composite dependencies

**Context**: Milestone 8f implements single-pass composite building which works for simple composites. This milestone adds multi-pass support for advanced composites that depend on other composites (e.g., FocalLength35efl depending on ScaleFactor35efl).

**Deliverables**:

1. **Multi-Pass Algorithm**

   - Track deferred composites that couldn't be built due to missing dependencies
   - Implement iterative passes until no new composites can be built
   - Add circular dependency detection with proper error reporting

2. **Dependency Tracking**

   - Maintain `notBuilt` set of composite tags not yet computed
   - Track which composites depend on other composites
   - Implement dependency graph for debugging

3. **Advanced Composite Examples**

   - FocalLength35efl: Depends on FocalLength and ScaleFactor35efl (itself composite)
   - DOF (Depth of Field): Complex calculation with multiple dependencies
   - LensID: May depend on other derived lens information

4. **Performance Optimizations**
   - Only process deferred tags in subsequent passes
   - Short-circuit when no progress made
   - Cache dependency lookups

**Success Criteria**:

- Composite tags with composite dependencies work correctly
- Circular dependencies detected and reported
- No performance regression for simple composites
- All existing composite tags continue working

**Algorithm** (based on ExifTool's BuildCompositeTags):

```rust
loop {
    let mut progress_made = false;
    let mut deferred = Vec::new();

    for composite in pending_composites {
        if all_dependencies_available(composite) {
            build_composite(composite);
            progress_made = true;
        } else {
            deferred.push(composite);
        }
    }

    if !progress_made {
        if deferred.is_empty() {
            break; // All done
        } else {
            // Circular dependency detected
            warn!("Circular dependency in {} composite tags", deferred.len());
            break;
        }
    }

    pending_composites = deferred;
}
```

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
