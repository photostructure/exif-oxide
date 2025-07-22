# Technical Project Plan: Unified Tag Definition Codegen

## Project Overview

**Goal**: Generate complete tag definitions from ExifTool source, including tag IDs, names, formats, AND PrintConv implementations in a single unified structure.

**Problem**: Manual implementations of EXIF tag PrintConvs are error-prone, particularly around tag ID offsets. Even simple 2-3 entry lookups can have offset bugs that are hard to spot in PR review, leading to expensive runtime errors.

## Background & Context

### Why This Work is Needed

- **Offset errors**: Manual tag implementations require matching tag IDs (e.g., 0x0128) with their PrintConv logic, creating opportunities for offset mistakes
- **PR review difficulty**: Reviewers struggle to verify that tag IDs match their implementations correctly
- **Maintenance burden**: Even stable EXIF tags require careful manual translation from ExifTool source
- **Existing solutions insufficient**: Current inline_printconv extractor expects named hashes, but EXIF uses inline anonymous PrintConvs

### Related Documentation

- [CODEGEN.md](../CODEGEN.md) - Code generation framework
- [20250721-migrate-all-manual-lookup-tables-to-codegen.md](./20250721-migrate-all-manual-lookup-tables-to-codegen.md) - Parent task tracking manual lookup migrations
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core principle of exact ExifTool translation

## Technical Foundation

### Key Systems

- **Extraction**: `codegen/extractors/` - Perl scripts that parse ExifTool modules
- **Generation**: `codegen/src/generators/` - Rust code that generates lookup tables
- **Expression System**: `src/expressions/` - Runtime expression evaluator for simple Perl expressions
- **Manual Registry**: `src/registry.rs` - Function lookup for complex PrintConvs

### Current Architecture

```
ExifTool Tag Definition
    ↓
Manual Implementation (error-prone)
    ↓
src/implementations/print_conv.rs
```

### Target Architecture

```
ExifTool Tag Definition
    ↓
Unified Extraction (tag ID + PrintConv together)
    ↓
Generated Tag Tables with PrintConv
    ↓
Runtime Dispatcher (simple/expression/manual)
```

## Work Completed

This is a new initiative - no work completed yet. The concept emerged from recognizing that inline PrintConvs are fundamentally part of tag definitions, not separate lookup tables.

### Related Existing Code to Study

- `codegen/extractors/tag_definitions.pl` - Current tag extractor (without PrintConv)
- `codegen/src/generators/tags.rs` - Current tag generator 
- `src/implementations/print_conv.rs` - Manual implementations we want to replace
- `src/registry.rs` - Function registry pattern for complex cases

## Remaining Tasks

### Phase 1: Prototype Extractor (High Confidence)

1. **Create tag definition extractor** (`codegen/extractors/tag_definitions_complete.pl`)
   - Extract tag ID, name, format, groups, and PrintConv together
   - Classify PrintConv types: Simple hash, Expression, or Manual
   - Output unified JSON structure
   - Base on existing `tag_definitions.pl` but add PrintConv extraction

2. **Create manual PrintConv registry** (`codegen/config/Exif_pm/manual_printconv.json`)
   - Map complex PrintConvs to manual function names
   - Document why each needs manual implementation
   - Start empty - only add as needed

3. **Implement generator** (`codegen/src/generators/tag_definitions.rs`)
   - Generate `TagDefinition` structs with embedded PrintConv
   - Generate runtime dispatcher for different PrintConv types
   - Consider extending existing `tags.rs` generator

### Phase 2: EXIF Basic Tags Implementation

4. **Extract EXIF basic tags**
   - ResolutionUnit (0x0128) - 3 entries
   - YCbCrPositioning (0x0213) - 2 entries  
   - ColorSpace (0xa001) - 5 entries
   - WhiteBalance (0xa403) - 2 entries
   - MeteringMode (0x9207) - 8 entries
   - ExposureProgram (0x8822) - 10 entries

5. **Update print_conv.rs** to use generated definitions
   - Replace manual implementations with generated lookups
   - Verify identical behavior with tests

### Phase 3: GPS Tags with Expressions (Research Needed)

6. **Extract GPS tags** 
   - Simple: GPSAltitudeRef, GPSLatitudeRef, GPSLongitudeRef
   - Expression: GPSAltitude (has `$val =~ /^(inf|undef)$/ ? $val : "$val m"`)

7. **Enhance expression translator**
   - Research: Can our expression DSL handle the GPS PrintConvs?
   - Implement Perl→Rust expression translation for common patterns

## Prerequisites

- Understanding of current inline_printconv extraction (exists but has structural mismatch)
- Familiarity with ExifTool tag table structure
- Access to expression evaluation system (`src/expressions/`)
- Read [EXIFTOOL-GUIDE.md](../guides/EXIFTOOL-GUIDE.md) sections on tag tables and PrintConv

### Development Environment Setup

```bash
# Verify you can run extractors
cd codegen
perl extractors/simple_table.pl ../third-party/exiftool/lib/Image/ExifTool/Exif.pm %orientation

# Verify expression system works
cargo test -p exif-oxide expressions::tests
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_resolution_unit_printconv() {
    let tag_def = EXIF_TAGS.get(&0x0128).unwrap();
    assert_eq!(apply_print_conv(&tag_def, &TagValue::U16(2)), 
               TagValue::String("inches".to_string()));
}
```

### Integration Tests

- Generate test comparing our output with `exiftool -j` for test images
- Verify all extracted PrintConvs produce identical output to ExifTool

### Validation Script

```bash
# Compare generated PrintConvs with ExifTool
cargo run --bin validate-printconvs test-images/*.jpg
```

## Success Criteria & Quality Gates

- **Zero offset errors**: Tag IDs and PrintConvs extracted together, eliminating manual matching
- **ExifTool parity**: Generated PrintConvs produce identical output to ExifTool
- **PR reviewability**: Generated code clearly shows tag ID with its PrintConv
- **All EXIF basic tags**: Successfully migrate the 6 identified tags
- **GPS proof-of-concept**: At least one expression-based PrintConv working

## Gotchas & Tribal Knowledge

### PrintConv Complexity Levels

1. **Simple hashes**: Direct key→value mappings (most EXIF tags)
2. **Expressions**: Perl code we can translate (`sprintf`, simple conditionals)
3. **Complex**: References external hashes, complex logic → manual registry

### Why Not Extend inline_printconv?

The inline_printconv extractor expects named hashes (`%hashName`), but EXIF PrintConvs are anonymous hashes attached to tags. This structural mismatch makes it unsuitable.

### Expression Translation Examples

```perl
# Perl: '$val > 8 ? undef : $val'
# Rust: "if $val > 8 { None } else { Some($val) }"

# Perl: 'sprintf("%.1f", $val)' 
# Rust: "format!(\"{:.1}\", $val)"
```

### Implementation Order Matters

1. **Start with ResolutionUnit & YCbCrPositioning** - Simplest (2-3 entries each)
2. **Then ColorSpace & MeteringMode** - More entries but still simple
3. **GPS tags last** - They have expressions, good for validating that part

### Debugging Tips

- **Extractor issues**: Add `use Data::Dumper; print STDERR Dumper($tag_info);` in Perl
- **Generated code issues**: Check `codegen/generated/extract/*.json` for raw extraction data
- **Runtime issues**: Enable `RUST_LOG=trace` to see tag processing details

### Example Extraction Output

```json
{
  "tag_id": "0x0128",
  "name": "ResolutionUnit",
  "format": "int16u",
  "groups": { "0": "IFD0", "1": "IFD", "2": "Image" },
  "print_conv_type": "Simple",
  "print_conv_data": {
    "1": "None",
    "2": "inches",
    "3": "cm"
  }
}
```

### Future Extensibility

This approach scales to manufacturer modules (Canon, Nikon) where tag definitions also contain inline PrintConvs, potentially eliminating entire categories of manual implementation.

## Final Tasks: Codebase Retrofit & Documentation Update

### Retrofit Existing Code

After successful implementation, audit and migrate existing manual implementations:
- **Canon**: Review `src/implementations/canon/tags/` for inline PrintConvs in binary data tables
- **Nikon**: Check `src/implementations/nikon/tags/` for tag-specific PrintConvs
- **Sony/Olympus**: Scan for any manual tag+PrintConv implementations
- **Core EXIF**: Beyond the 6 prototype tags, find other EXIF tags with inline PrintConvs
- Create migration plan prioritizing high-value targets (many entries, complex offsets)

### Update Documentation

Review and update all `docs/todo/*.md` files to:
- Identify tasks that could benefit from Tag Definition Codegen instead of manual implementation
- Update migration strategies to prefer this approach for tag-specific PrintConvs
- Mark obsolete any tasks that planned manual tag implementations
- Add cross-references to this TPP where relevant