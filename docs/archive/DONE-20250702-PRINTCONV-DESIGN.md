● PrintConv Output Format Mismatch Issue [RESOLVED]

Solution Implemented: TagEntry API with Separate Value/Print Fields

See Milestone 8b for implementation details.

Problem Summary

Our PrintConv implementation has a fundamental architecture mismatch with ExifTool's output expectations. The registry system assumes all PrintConv functions
return strings, but ExifTool expects some tags to remain numeric after "PrintConv" processing.

Current State Analysis

What's Working ✅

- Most PrintConv functions work correctly (ColorSpace, Flash, MeteringMode, Orientation, etc.)
- Basic PrintConv registry and application system is functional
- Canon MakerNote integration is successful with 27+ tags being extracted

What's Broken ❌

- FNumber: ExifTool expects 4.0 (numeric), we output [4, 1] (raw rational array)
- ExposureTime: ExifTool expects "1/2000" (string), we output [1, 2000] (raw rational array)

Root Cause

The issue occurs in /src/exif.rs in the apply_conversions() method:

if let Some(print_conv_ref) = tag_def.print_conv_ref {
let converted_string = registry::apply_print_conv(print_conv_ref, &value);

      // Only use the converted string if it's different from the raw value
      if converted_string != value.to_string() {
          return TagValue::String(converted_string);  // ← Always returns STRING
      }

}

Registry signature forces string returns:
pub type PrintConvFn = fn(&TagValue) -> String;

But ExifTool expects mixed output types:

- FNumber: 4.0 (numeric f64)
- ExposureTime: "1/2000" (formatted string)
- FocalLength: "24.0 mm" (formatted string)

Architecture Conflict

Our current PrintConv functions work correctly:
// fnumber_print_conv returns "4"
// exposuretime_print_conv returns "1/2000"
// focallength_print_conv returns "24.0 mm"

But the registry system converts ALL PrintConv results to TagValue::String, while ExifTool expects:

- FNumber to remain numeric (no PrintConv, just ValueConv)
- ExposureTime to be string formatted
- FocalLength to be string formatted

Evidence from Compatibility Tests

ExifTool Reference Output:
{
"EXIF:FNumber": 4.0, // ← numeric
"EXIF:ExposureTime": "1/2000", // ← string
"EXIF:FocalLength": "24.0 mm" // ← string
}

Our Current Output:
{
"EXIF:FNumber": [4, 1], // ← raw rational (no conversion applied)
"EXIF:ExposureTime": [1, 2000], // ← raw rational (no conversion applied)  
 "EXIF:FocalLength": "24.0 mm" // ← works correctly
}

Why FocalLength works but FNumber/ExposureTime don't:
The apply_conversions() method has this condition:
if converted_string != value.to_string() {
return TagValue::String(converted_string);
}

- FocalLength: TagValue::Rational(24,1).to_string() = "24/1" ≠ "24.0 mm" → conversion applied ✅
- FNumber: TagValue::Rational(4,1).to_string() = "4/1" ≠ "4" → should apply conversion but needs numeric output
- ExposureTime: TagValue::Rational(1,2000).to_string() = "1/2000" = "1/2000" → conversion skipped ❌

Possible Solution Paths

Solution 1: Tag-Specific Output Type Configuration

Approach: Extend tag definitions to specify expected output type.

pub struct TagDef {
// ... existing fields
pub print_conv_ref: Option<&'static str>,
pub print_conv_output_type: Option<PrintConvOutputType>, // ← NEW
}

pub enum PrintConvOutputType {
String, // Default - convert to string
Numeric, // Parse back to number  
 Passthrough, // Use ValueConv only, skip PrintConv
}

Implementation:
fn apply_conversions(&self, raw_value: &TagValue, tag_def: Option<&TagDef>) -> TagValue {
let mut value = raw_value.clone();

      // Apply ValueConv first
      if let Some(value_conv_ref) = tag_def.value_conv_ref {
          value = registry::apply_value_conv(value_conv_ref, &value);
      }

      // Apply PrintConv based on expected output type
      if let Some(print_conv_ref) = tag_def.print_conv_ref {
          let converted_string = registry::apply_print_conv(print_conv_ref, &value);

          match tag_def.print_conv_output_type {
              Some(PrintConvOutputType::Numeric) => {
                  if let Ok(num) = converted_string.parse::<f64>() {
                      return TagValue::F64(num);
                  }
              }
              Some(PrintConvOutputType::Passthrough) => {
                  return value; // Skip PrintConv
              }
              _ => {
                  return TagValue::String(converted_string);
              }
          }
      }

      value

}

Pros:

- Explicit configuration per tag
- Maintains existing PrintConv function signatures
- Clear semantics for each tag's expected behavior
- Minimal changes to existing code

Cons:

- Requires updating tag definitions (codegen changes)
- Additional complexity in tag metadata
- Manual configuration required for each special case

Solution 2: Registry Function Signature Change

Approach: Change PrintConv functions to return TagValue instead of String.

pub type PrintConvFn = fn(&TagValue) -> TagValue; // ← Changed from String

Implementation:
// Updated PrintConv functions
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
match val.as_f64() {
Some(f_number) => TagValue::F64(f_number), // ← Return numeric
// ... fallback logic
}
}

pub fn exposuretime_print_conv(val: &TagValue) -> TagValue {
// ... existing string formatting logic
TagValue::String(formatted_string) // ← Return string
}

Pros:

- Most flexible - each function decides its output type
- No additional metadata required
- Natural semantics per function

Cons:

- Breaking change to all existing PrintConv functions
- Registry interface change affects all callers
- Function naming becomes misleading ("print" implies string output)

Solution 3: Dual Registry System

Approach: Separate registries for string formatting vs. value conversion.

pub type PrintConvFn = fn(&TagValue) -> String; // String formatting
pub type DisplayConvFn = fn(&TagValue) -> TagValue; // Mixed output types

// Tag definitions specify which type to use
pub struct TagDef {
pub print_conv_ref: Option<&'static str>, // For string output
pub display_conv_ref: Option<&'static str>, // For mixed output
}

Pros:

- Clear separation of concerns
- Backward compatible with existing PrintConv functions
- Explicit about output expectations

Cons:

- Increased complexity (two registry systems)
- Need to migrate some functions from PrintConv to DisplayConv
- Potential confusion about which to use when

Solution 4: ExifTool-Style Multi-Pass Processing

Approach: Follow ExifTool's exact semantics more closely.

Research needed: Examine ExifTool source to understand exactly when PrintConv is applied vs. skipped, and how numeric vs. string outputs are determined.

ExifTool Logic (needs verification):

- Some tags have ValueConv only (numeric conversion)
- Some tags have PrintConv only (string formatting)
- Some tags have both (ValueConv → PrintConv)
- Output format determined by tag metadata, not function return type

Pros:

- Perfect ExifTool compatibility
- Follows proven patterns
- Most correct long-term solution

Cons:

- Requires deep ExifTool research
- Potentially significant architecture changes
- May affect many existing working tags

Recommended Approach

Solution 1 (Tag-Specific Output Type Configuration) is recommended because:

1. Minimal Risk: Preserves existing working PrintConv functions
2. Surgical Fix: Only affects the problematic tags (FNumber, ExposureTime)
3. Clear Semantics: Explicit configuration makes behavior predictable
4. Incremental: Can be implemented for just the failing tags initially

Implementation Plan:

1. Add print_conv_output_type field to TagDef
2. Configure FNumber as PrintConvOutputType::Numeric
3. Configure ExposureTime as PrintConvOutputType::String (ensure conversion applies)
4. Update codegen to handle new field
5. Test compatibility extensively

Alternative: If Solution 1 proves too complex, Solution 4 (ExifTool research) should be pursued to ensure we're implementing the correct semantics long-term.

Current Milestone Status

Despite this issue, Milestone 10 (Canon MakerNote Expansion) is fundamentally complete:

- ✅ Canon MakerNote detection and processing
- ✅ Canon offset fixing with multiple schemes
- ✅ Canon ProcessSerialData for AF info extraction
- ✅ Canon-specific tag processing (27+ tags extracted)
- ✅ Integration with main EXIF processing flow
- ❌ PrintConv output format compatibility ← This issue affects all tags, not just Canon

The PrintConv issue is a system-wide architecture problem, not a Canon-specific implementation problem.

## Resolution Summary

After discussion, we decided to:

1. **Keep ValueConv and PrintConv results separate** in a new TagEntry API structure
2. **Return both values** to API consumers: `{group, name, value, print}`
3. **CLI emulates ExifTool -# flag** to choose between value and print output
4. **Match ExifTool JSON exactly** including numeric vs string types

This solution:

- Provides maximum flexibility to API consumers
- Maintains exact ExifTool compatibility
- Avoids trying to make PrintConv return mixed types
- Sets up proper foundation for full ValueConv implementation

See Milestone 8b in MILESTONES.md for implementation plan.
