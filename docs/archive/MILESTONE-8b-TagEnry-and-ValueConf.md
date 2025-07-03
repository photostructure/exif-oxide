## Important Steps Before Coding

1. Be sure to study $REPO_ROOT/CLAUDE.md, $REPO_ROOT/docs/ARCHITECTURE.md, $REPO_ROOT/third-party/exiftool/CLAUDE.md, and all relevant related documentation before starting any work. With this project, **everything** is more complicated than you'd expect.

2. Be sure to follow the `Trust ExifTool` and `Ask clarifying questions` sections of ../CLAUDE.md

## Important Steps During Implementation

1. Be sure to follow the `Trust ExifTool` and `Code smell` sections of ../CLAUDE.md

## Important Milestone Validation Steps

After you think you're done implementing a milestone:

1. **Update Supported Tags Configuration**: If your milestone adds working PrintConv implementations, update the `MILESTONE_COMPLETIONS` configuration in `codegen/src/main.rs` to include your new supported tags, then run `cargo run -p codegen` to regenerate the supported tags JSON.

2. **Compatibility Testing**: Re-run `make compat` and iterate until all tests pass. The regenerated supported tags list will automatically be used by the compatibility tests.

3. **Code Quality**: Run `make precommit` and fix linting, compilation, and test errors.

4. **DON'T DELETE YOUR STUBS**: if clippy is complaining about unused code, don't delete your stub! Instead add a TODO with the milestone that will replace the stub with a real implementation.

## Important Steps After Completing a Milestone

1. Remove the completed milestone section from this document.
2. Concisely summarize the completed work in docs/archive/DONE-MILESTONES.md

## Core Principles

1. **Always Working**: Every milestone produces runnable code with graceful fallbacks
2. **No Panics**: Missing implementations return raw values, never crash
3. **Demand-Driven**: Only implement what's needed for real test images
4. **Manual Excellence**: Complex logic is manually ported with ExifTool references
5. **Transparent Progress**: Runtime tracking shows exactly what's missing

## Milestone 8b: TagEntry API & Basic ValueConv (COMPLETED)

**Goal**: Restructure API to return both value and print fields, implement basic ValueConv

**Context**: Our current architecture forces PrintConv to return strings, breaking ExifTool compatibility for numeric values like FNumber. This milestone fixes the architecture and implements minimal ValueConv support.

**Deliverables**:

1. **New TagEntry API Structure**

   - [x] Create TagEntry struct with {group, name, value, print} fields
   - [x] Update ExifReader to build TagEntry objects
   - [x] Modify apply_conversions to return (value, print) tuple
   - [x] Update all tag extraction to use new structure

   ````rust
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
   ````

2. **Basic ValueConv Implementation**

   - [x] ValueConv registry (same pattern as PrintConv)
   - [x] Rational to float conversion (for FNumber, ExposureTime, FocalLength)
   - [x] Update codegen to extract ValueConv references (with temporary workaround)
   - [x] Wire ValueConv into conversion pipeline

3. **CLI -# Flag Support**

   - [x] Parse -TagName# syntax
   - [x] Track which tags should use value vs print
   - [x] Update JSON output to match ExifTool behavior
   - [x] Update extract_metadata_json to handle -# flag mode
   - [x] Add compatibility tests for -# flag behavior

   **Testing Infrastructure Updates**:

   - [x] Modify `tests/compatibility.rs` to test both normal and -# modes
   - [x] Update `extract_metadata_json` to accept flag configuration
   - [x] Create test cases comparing:
     - Normal output: `exiftool -j image.jpg`
     - Numeric output: `exiftool -j -FNumber# -ExposureTime# image.jpg`
   - [x] Ensure our JSON matches ExifTool's exactly (numeric vs string types)

4. **Fix PrintConv Implementations**
   - [x] Update fnumber_print_conv to match ExifTool exactly
   - [x] Fix exposuretime_print_conv comparison logic
   - [x] Ensure all PrintConv outputs match ExifTool

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

## Completion Summary

**Status**: COMPLETED

All deliverables for Milestone 8b have been successfully implemented:

1. **TagEntry API** - Complete with value/print fields supporting ExifTool's -# flag functionality
2. **ValueConv Registry** - Implemented with rational-to-float conversions for FNumber, ExposureTime, and FocalLength
3. **CLI Argument Parsing** - Fixed to handle mixed positional arguments (files and -TagName# flags in any order)
4. **JSON Serialization** - Correctly switches between value and print representations based on -# flags
5. **Composite Tags** - Fixed to ensure they are included in JSON output
6. **Unit Tests** - Added comprehensive tests for argument parsing patterns

**Key Implementation Details**:

- **parse_mixed_args function** in main.rs separates files from numeric tags based on prefix patterns
- **ValueConv workaround** in codegen/src/main.rs manually maps the three needed tags until extraction script is fixed
- **formats.rs enhancement** ensures all tag_entries (including composite tags) are added to legacy_tags HashMap

**Validation Results**:

All CLI invocation patterns now work correctly:
- `./exif-oxide image.jpg -FNumber# -ExposureTime#` ✓
- `./exif-oxide -FNumber# image.jpg -ExposureTime#` ✓  
- `./exif-oxide -FNumber# -ExposureTime# image.jpg` ✓

Numeric tags show value field, normal output shows print field with correct JSON types.