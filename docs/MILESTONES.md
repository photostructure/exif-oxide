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

4. **DON'T DELETE YOUR STUBS**: if clippy is complaining about unused code, don't delete your stub! Instead add a TODO with the milestone that will replace the stub with a real implementation.

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

## Milestone 8b: TagEntry API & Basic ValueConv (1 week)

**Goal**: Restructure API to return both value and print fields, implement basic ValueConv

**Context**: Our current architecture forces PrintConv to return strings, breaking ExifTool compatibility for numeric values like FNumber. This milestone fixes the architecture and implements minimal ValueConv support.

**Deliverables**:

1. **New TagEntry API Structure**
   - [ ] Create TagEntry struct with {group, name, value, print} fields
   - [ ] Update ExifReader to build TagEntry objects
   - [ ] Modify apply_conversions to return (value, print) tuple
   - [ ] Update all tag extraction to use new structure

   ```rust
   /// A single extracted metadata tag with both its converted value and display string.
   /// 
   /// This structure provides access to both the logical value (after ValueConv)
   /// and the human-readable display string (after PrintConv), allowing consumers
   /// to choose the most appropriate representation.
   /// 
   /// # Examples
   /// 
   /// ```
   /// // A typical EXIF tag entry
   /// TagEntry {
   ///     group: "EXIF".to_string(),
   ///     name: "FNumber".to_string(),
   ///     value: TagValue::F64(4.0),      // Post-ValueConv: 4/1 → 4.0
   ///     print: "4.0".to_string(),       // Post-PrintConv: formatted for display
   /// }
   /// 
   /// // A tag with units in the display string
   /// TagEntry {
   ///     group: "EXIF".to_string(),
   ///     name: "FocalLength".to_string(),
   ///     value: TagValue::F64(24.0),     // Numeric value
   ///     print: "24.0 mm".to_string(),   // Human-readable with units
   /// }
   /// ```
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct TagEntry {
       /// Tag group name (e.g., "EXIF", "GPS", "Canon", "MakerNotes")
       /// 
       /// Groups follow ExifTool's naming conventions:
       /// - Main IFDs: "EXIF", "GPS", "IFD0", "IFD1"
       /// - Manufacturer: "Canon", "Nikon", "Sony", etc.
       /// - Sub-groups: "Canon::CameraSettings", etc.
       pub group: String,
       
       /// Tag name without group prefix (e.g., "FNumber", "ExposureTime")
       /// 
       /// Names match ExifTool's tag naming exactly for compatibility.
       pub name: String,
       
       /// The logical value after ValueConv processing.
       /// 
       /// This is the value you get with ExifTool's -# flag:
       /// - Rational values converted to floats (4/1 → 4.0)
       /// - APEX values converted to real units
       /// - Raw value if no ValueConv exists
       /// 
       /// # Examples
       /// 
       /// - FNumber: `TagValue::F64(4.0)` (from rational 4/1)
       /// - ExposureTime: `TagValue::F64(0.0005)` (from rational 1/2000)
       /// - Make: `TagValue::String("Canon")` (no ValueConv needed)
       pub value: TagValue,
       
       /// The display string after PrintConv processing.
       /// 
       /// This is the human-readable representation:
       /// - Numbers may be formatted ("4.0" not "4")
       /// - Units may be added ("24.0 mm")
       /// - Coded values decoded ("Rotate 90 CW" not "6")
       /// 
       /// If no PrintConv exists, this equals `value.to_string()`.
       /// 
       /// # ExifTool JSON Compatibility
       /// 
       /// When serializing to JSON, some numeric PrintConv results
       /// (like FNumber's "4.0") are encoded as JSON numbers, not strings.
       /// The CLI handles this compatibility layer.
       pub print: String,
   }
   ```

2. **Basic ValueConv Implementation**
   - [ ] ValueConv registry (same pattern as PrintConv)
   - [ ] Rational to float conversion (for FNumber, ExposureTime, FocalLength)
   - [ ] Update codegen to extract ValueConv references
   - [ ] Wire ValueConv into conversion pipeline

3. **CLI -# Flag Support**
   - [ ] Parse -TagName# syntax
   - [ ] Track which tags should use value vs print
   - [ ] Update JSON output to match ExifTool behavior
   - [ ] Update extract_metadata_json to handle -# flag mode
   - [ ] Add compatibility tests for -# flag behavior

   **Testing Infrastructure Updates**:
   - [ ] Modify `tests/compatibility.rs` to test both normal and -# modes
   - [ ] Update `extract_metadata_json` to accept flag configuration
   - [ ] Create test cases comparing:
     - Normal output: `exiftool -j image.jpg`
     - Numeric output: `exiftool -j -FNumber# -ExposureTime# image.jpg`
   - [ ] Ensure our JSON matches ExifTool's exactly (numeric vs string types)

4. **Fix PrintConv Implementations**
   - [ ] Update fnumber_print_conv to match ExifTool exactly
   - [ ] Fix exposuretime_print_conv comparison logic
   - [ ] Ensure all PrintConv outputs match ExifTool

**Success Criteria**:

- ExifTool compatibility tests pass for FNumber, ExposureTime, FocalLength
- API returns both value and print for all tags
- CLI -# flag works like ExifTool
- JSON output types match ExifTool exactly
- Compatibility tests pass for both normal and -# flag modes
- extract_metadata_json correctly switches between value/print based on -# flags

**Implementation Notes**:

1. **Edge Cases**:
   - Tags with no PrintConv: `print` field equals `format_tag_value(&value)`
   - Tags with no ValueConv: `value` field contains raw extracted data
   - Array values: Both fields handle arrays appropriately
   - Binary data: `value` contains `TagValue::Binary(BinaryRef)`, `print` shows size/format info

2. **Group Name Mapping**:
   ```rust
   // IFD name to group mapping
   match ifd_name {
       "IFD0" => "IFD0",
       "ExifIFD" => "EXIF",
       "GPS" => "GPS",
       "InteropIFD" => "Interop",
       "MakerNotes::Canon" => "Canon",
       "MakerNotes::Canon::CameraSettings" => "Canon",
       // etc.
   }
   ```

3. **CLI JSON Serialization**:
   ```rust
   // When outputting JSON, preserve ExifTool's type quirks
   match (tag_name, &entry.print) {
       ("FNumber", s) if s.parse::<f64>().is_ok() => {
           // Output as JSON number, not string
           json!(s.parse::<f64>().unwrap())
       }
       _ => json!(entry.print)  // Normal string output
   }
   ```

4. **extract_metadata_json Updates**:
   ```rust
   pub fn extract_metadata_json(
       path: &Path,
       numeric_tags: Option<HashSet<String>>, // NEW: tags to show as numeric
   ) -> Result<serde_json::Value> {
       let metadata = extract_metadata(path)?;
       
       // Build JSON based on numeric_tags configuration
       let mut output = json!({});
       for entry in metadata.tags {
           let key = format!("{}: {}", entry.group, entry.name);
           let value = if numeric_tags.as_ref().map_or(false, |set| set.contains(&entry.name)) {
               // Use value field for -# tags
               to_json_value(&entry.value)
           } else {
               // Use print field normally, with type preservation
               match entry.name.as_str() {
                   "FNumber" if entry.print.parse::<f64>().is_ok() => {
                       json!(entry.print.parse::<f64>().unwrap())
                   }
                   _ => json!(entry.print)
               }
           };
           output[key] = value;
       }
       Ok(output)
   }
   ```

**Future ValueConv Work** (separate milestone):
- APEX conversions (ShutterSpeedValue, ApertureValue)
- GPS coordinate conversion
- Complex mathematical conversions


## Milestone 8c: Full ValueConv Implementation (2 weeks)

**Goal**: Complete ValueConv system with all mathematical conversions

**Deliverables**:

- [ ] APEX conversions
  - ShutterSpeedValue (2^-x)
  - ApertureValue (2^(x/2))
  - ExposureCompensation
- [ ] GPS coordinate conversion
  - Degrees/minutes/seconds to decimal
  - Handle hemisphere references
- [ ] Date/time conversions
- [ ] Complex mathematical conversions from ExifTool

**Success Criteria**:

- All ValueConv tests from ExifTool pass
- GPS coordinates show decimal degrees with -#
- APEX values converted correctly

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
