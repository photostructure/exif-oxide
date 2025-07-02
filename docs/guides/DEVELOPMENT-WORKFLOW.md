# exif-oxide Development Workflow

This guide describes the day-to-day development workflow for implementing new features in exif-oxide.

## The Extract-Generate-Implement Cycle

Development follows a demand-driven approach: only implement what real images actually need.

```
Extract → Generate → Discover → Implement → Validate
   ↑                                            ↓
   └────────────────────────────────────────────┘
```

## Step 1: Extract Phase

Extract tag definitions from ExifTool source:

```bash
perl codegen/extract_tables.pl > codegen/generated/tag_tables.json
```

This parses ExifTool's Perl modules and extracts:

- Tag definitions
- PrintConv/ValueConv references
- Processor specifications
- Format patterns

## Step 2: Generate Phase

Run code generation to create Rust code:

```bash
cargo run -p codegen
```

Expected output:

```
Generated: 823 mainstream tag definitions (from 15,234 total)
Simple conversions implemented: 156
Complex conversions referenced: 234 (no stubs generated)
Custom processors identified: 47

Code is ready to compile and run!
Use --show-missing on actual images to see what's needed.
```

Generated files in `src/generated/`:

- `tags.rs` - Tag table definitions
- `conversion_refs.rs` - Required conversion lists
- `supported_tags.rs` - Currently supported tags
- `composite_tags.rs` - Composite tag definitions

## Step 3: Discover Missing Implementations

### Using --show-missing

Run on actual test images to discover what's needed:

```bash
cargo run -p exif-oxide -- test-images/Canon/Canon_T3i.jpg --show-missing
```

Output:

```
Missing Implementations for Canon_T3i.jpg
=========================================
HIGH PRIORITY (blocks common tags):
- orientation_lookup (PrintConv)
  Used by: EXIF:Orientation

MEDIUM PRIORITY:
- canon_ev_format (PrintConv)
  Used by: Canon:ExposureCompensation
- canon_wb_lookup (PrintConv)
  Used by: Canon:WhiteBalance
```

### Prioritizing Work

Focus on:

1. **HIGH PRIORITY**: Tags that appear in most images
2. **Test Coverage**: Implementations needed by test suite
3. **User Requests**: Specific tags requested by users

## Step 4: Implement What's Needed

### 4.1 Find ExifTool Source

Locate the implementation in ExifTool:

```bash
# Search for PrintConv
grep -r "orientation_lookup\|Orientation.*PrintConv" third-party/exiftool/lib/

# Find in specific module
less third-party/exiftool/lib/Image/ExifTool/Exif.pm
/Orientation
```

### 4.2 Create Implementation

Add to appropriate file:

```rust
// In src/implementations/print_conv.rs

/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:4912
pub fn orientation_print_conv(val: &TagValue) -> String {
    match val {
        TagValue::Integer(1) => "Horizontal (normal)".to_string(),
        TagValue::Integer(2) => "Mirror horizontal".to_string(),
        TagValue::Integer(3) => "Rotate 180".to_string(),
        TagValue::Integer(4) => "Mirror vertical".to_string(),
        TagValue::Integer(5) => "Mirror horizontal and rotate 270 CW".to_string(),
        TagValue::Integer(6) => "Rotate 90 CW".to_string(),
        TagValue::Integer(7) => "Mirror horizontal and rotate 90 CW".to_string(),
        TagValue::Integer(8) => "Rotate 270 CW".to_string(),
        _ => format!("{}", val),
    }
}
```

### 4.3 Register Implementation

Add to registry:

```rust
// In registry initialization
registry.register_print_conv(
    "exif_orientation_lookup",
    orientation_print_conv
);
```

## Step 5: Validate

### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
make test

# Compatibility tests
make compat
```

### Compare with ExifTool

```bash
# Generate reference output
exiftool -j test-images/Canon/Canon_T3i.jpg > expected.json

# Generate our output
cargo run -- test-images/Canon/Canon_T3i.jpg > actual.json

# Compare
diff expected.json actual.json
```

### Update Supported Tags

If implementation is complete:

1. Add to `MILESTONE_COMPLETIONS` in `codegen/src/main.rs`
2. Regenerate: `cargo run -p codegen`
3. Verify in compatibility tests

## Common Development Tasks

### Adding a PrintConv

1. Find in ExifTool source
2. Add function to `src/implementations/print_conv.rs`
3. Register in `init_print_conv_registry()`
4. Test against real images

### Adding a ValueConv

1. Find in ExifTool source
2. Add function to `src/implementations/value_conv.rs`
3. Register in `init_value_conv_registry()`
4. Ensure PrintConv still works with converted value

### Adding a Manufacturer Processor

1. Create new file: `src/implementations/{manufacturer}.rs`
2. Implement processor function
3. Add module to `src/implementations/mod.rs`
4. Register in appropriate registry

### Debugging Tips

```bash
# Enable trace logging
RUST_LOG=trace cargo run -- test.jpg

# Use ExifTool verbose mode for comparison
exiftool -v3 test.jpg

# Check specific tag extraction
cargo run -- test.jpg | jq '.["EXIF:Orientation"]'
```

## Generator Stubs (Future Feature)

Generate implementation stubs for specific images:

```bash
cargo run -p exif-oxide -- --generate-stubs Canon_T3i.jpg
```

Creates: `implementations/stubs/canon_t3i_stubs.rs` with skeleton functions.

## Best Practices

1. **Always Reference ExifTool**: Include file and line numbers
2. **Test Edge Cases**: Include 0, negative, and invalid values
3. **Match Behavior Exactly**: Don't "improve" ExifTool's logic
4. **Document Quirks**: Explain any non-obvious behavior
5. **Use Real Images**: Test against actual camera files

## Performance Considerations

- **Lazy Implementation**: Only implement what's actually used
- **Batch Discovery**: Run --show-missing on multiple images
- **Profile First**: Don't optimize without measurement

## Related Documentation

- [CODEGEN-STRATEGY.md](../design/CODEGEN-STRATEGY.md) - Code generation details
- [IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md) - Implementation patterns
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Background and concepts
- [EXIFTOOL-UPDATE-WORKFLOW.md](EXIFTOOL-UPDATE-WORKFLOW.md) - Updating ExifTool
