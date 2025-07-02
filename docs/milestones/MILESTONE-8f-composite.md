# Milestone 8f: Composite Tag Infrastructure - Completion Guide

## Overview

Milestone 8f established the infrastructure for composite tags in exif-oxide. The infrastructure is complete and working, but only a subset of composite tag computations have been implemented. This document provides context for completing the remaining implementations.

## Current Status

### ✅ Completed Infrastructure

1. **Code Generation** (codegen/extract_tables.pl)
   - Successfully extracts 63 composite tag definitions from ExifTool
   - Generates `src/generated/composite_tags.rs` with complete `CompositeTagDef` structures
   - Includes require/desire dependencies, value_conv logic references, and print_conv references

2. **Core Infrastructure** (src/exif.rs)
   - `build_composite_tags()` method with single-pass dependency resolution
   - `composite_tags` HashMap storage
   - Integration with `get_all_tags()` using "Composite:" prefix
   - `apply_composite_conversions()` for PrintConv support
   - Enhanced debug logging showing available vs missing dependencies

3. **Working Implementations**
   - ImageSize - Combines width × height
   - GPSAltitude - Altitude with sea level reference
   - PreviewImageSize - Preview dimensions
   - ShutterSpeed - From multiple sources with fallback
   - Aperture - FNumber or ApertureValue fallback
   - DateTimeOriginal - Date/time field combination
   - FocalLength35efl - Focal length × scale factor
   - ScaleFactor35efl - Placeholder (returns 1.0 when FocalLength present)
   - SubSecDateTimeOriginal - DateTime with subseconds/timezone
   - CircleOfConfusion - 35mm diagonal / (scale × 1440)

### ❌ Remaining Work

1. **Composite Tag Implementations**
   - ~53 composite tags still show "not yet implemented" in trace logs
   - Each requires translating Perl ValueConv expressions to Rust
   - Some depend on complex functions like `Image::ExifTool::Exif::RedBlueBalance`
   - See `COMPOSITE_TAGS` in `src/generated/composite_tags.rs` for full list

2. **Complex Dependencies**
   - ScaleFactor35efl needs full `CalcScaleFactor35efl` implementation
   - LightValue needs `CalculateLV` function
   - DOF (Depth of Field) has complex multi-value calculations
   - Several tags depend on other composite tags (needs Milestone 11.5)

3. **Compatibility Test Updates**
   - Test snapshots were created before "Composite:" prefix was added
   - Need to update snapshots or filter composite tags in tests
   - Currently causing 2 test failures in exiftool_compatibility_tests

## Implementation Guide

### Adding a New Composite Tag

1. **Find the Definition**
   ```rust
   // In src/generated/composite_tags.rs, find your tag:
   CompositeTagDef {
       name: "YourTag",
       value_conv: Some("$val[0] + $val[1]"), // Perl expression
       require: &[(0, "RequiredTag")],
       desire: &[(1, "OptionalTag")],
       ...
   }
   ```

2. **Add to compute_composite_tag Match**
   ```rust
   // In src/exif.rs around line 905
   match composite_def.name {
       // ... existing tags ...
       "YourTag" => self.compute_your_tag(available_tags),
       _ => { /* ... */ }
   }
   ```

3. **Implement Computation Method**
   ```rust
   /// Compute YourTag composite
   /// ExifTool: lib/Image/ExifTool/Composite.pm line XXX
   fn compute_your_tag(&self, available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
       // Translate Perl ValueConv expression verbatim
       // Example: "$val[0] + $val[1]" becomes:
       let val0 = available_tags.get("RequiredTag")?.as_f64()?;
       let val1 = available_tags.get("OptionalTag")?.as_f64().unwrap_or(0.0);
       Some(TagValue::F64(val0 + val1))
   }
   ```

### Common Perl → Rust Patterns

| Perl Expression | Rust Translation |
|----------------|------------------|
| `$val[0] \|\| $val[1]` | First non-None value fallback |
| `"$val[0] $val[1]"` | `format!("{} {}", val0, val1)` |
| `$val[0] * $val[1]` | Multiplication with as_f64() |
| `$val =~ /pattern/` | Regex or string matching |
| `defined($val)` | `Option::is_some()` |
| `IsFloat($val)` | `value.as_f64().is_some()` |

### Testing Your Implementation

1. **Unit Test** (if complex logic)
   ```rust
   #[test]
   fn test_your_tag_computation() {
       let mut tags = HashMap::new();
       tags.insert("RequiredTag".to_string(), TagValue::F64(10.0));
       let result = compute_your_tag(&tags);
       assert_eq!(result, Some(TagValue::F64(10.0)));
   }
   ```

2. **Integration Test**
   ```bash
   # Test with real image
   RUST_LOG=trace cargo run -- test-images/your-test.jpg 2>&1 | grep YourTag
   
   # Compare with ExifTool
   exiftool -Composite:YourTag test-images/your-test.jpg
   ```

## Priority Tags to Implement

Based on frequency in real images:

1. **RedBalance/BlueBalance** - White balance calculations
2. **LensID** - Lens identification from multiple sources
3. **Megapixels** - Calculate from ImageSize
4. **GPSPosition** - Combine lat/lon into single field
5. **FOV** (Field of View) - Complex trigonometry
6. **HyperfocalDistance** - Photography calculations

## Known Issues

1. **Single-Pass Limitation**
   - Tags depending on other composites fail (e.g., CircleOfConfusion needs ScaleFactor35efl)
   - Will be fixed in Milestone 11.5 with multi-pass support

2. **Missing PrintConv**
   - Many composite tags reference PrintConv functions that aren't implemented
   - These fall back to raw value display

3. **Complex Functions**
   - `CalcScaleFactor35efl` - Complex sensor size calculations
   - `RedBlueBalance` - White balance array processing
   - `CalculateLV` - Light value calculations
   - These need full manual implementation

## References

- ExifTool Source: `lib/Image/ExifTool/Composite.pm`
- Composite Documentation: `third-party/exiftool/doc/concepts/COMPOSITE_TAGS.md`
- Tag Definitions: `src/generated/composite_tags.rs`
- Test Images: `test-images/` directory

## Next Steps

1. Implement high-priority composite tags based on real-world usage
2. Update compatibility test snapshots to include "Composite:" prefix
3. Add PrintConv implementations for composite tags
4. Document any complex calculations with ExifTool source references
5. Consider implementing Milestone 11.5 for multi-pass support if many tags are blocked

Remember: Trust ExifTool - translate ValueConv expressions verbatim, don't optimize or simplify!