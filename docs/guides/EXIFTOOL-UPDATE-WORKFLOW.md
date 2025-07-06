# ExifTool Update Workflow

This guide describes the step-by-step process for updating exif-oxide when a new ExifTool version is released.

## Overview

ExifTool releases monthly updates with new parsers, file types, and bugfixes. This workflow ensures exif-oxide stays current with minimal manual effort.

## Prerequisites

- ExifTool submodule at `third-party/exiftool/`
- Perl installed for extraction script
- Rust toolchain for building

## Update Process

### 1. Update ExifTool Submodule

```bash
cd third-party/exiftool
git fetch origin
git checkout v12.77  # new version tag
cd ../..
```

### 2. Regenerate Tag Definitions

```bash
# Extract updated tag definitions from ExifTool
perl codegen/extract_tables.pl > codegen/generated/tag_tables.json

# Run code generation
cargo run -p codegen
```

### 3. Review Changes

The codegen output will show what's new:

```
New in ExifTool 12.77:
- 3 new mainstream tags requiring implementation
- 1 new Canon processor variant
- 47 non-mainstream tags (ignored)

Missing implementations (priority order):
1. canon_new_lens_type (PrintConv) - 15 test images
2. nikon_z9_af_mode (PrintConv) - 8 test images
3. ProcessCanonCR3 (Processor) - 5 test images
```

### 4. Assess Implementation Requirements

#### For Minor Updates (tags only)

If the update only adds new tags within existing processors:
- No manual implementation needed
- Proceed directly to testing

#### For Major Updates (new processors/conversions)

If new processors or complex conversions are needed:
- Implement missing pieces manually
- Reference ExifTool source code
- Test against provided images

### 5. Implement Missing Pieces

For each missing implementation:

1. **Locate in ExifTool source**
   ```bash
   # Search for the implementation
   grep -r "canon_new_lens_type" third-party/exiftool/lib/
   ```

2. **Port to Rust**
   - Add to appropriate file in `src/implementations/`
   - Include ExifTool source reference
   - Match behavior exactly

3. **Register implementation**
   - Add to registry in implementation file
   - Update MILESTONE_COMPLETIONS if needed

### 6. Update Supported Tags

If you've added new working implementations:

```bash
# Update MILESTONE_COMPLETIONS in codegen/src/main.rs
# Then regenerate to update supported tags list
cargo run -p codegen
```

### 7. Test Compatibility

```bash
# Run full test suite
make test

# Run compatibility tests
make compat

# Test specific new features
cargo run -p exif-oxide -- test-images/canon/new_model.jpg
```

### 8. Commit Changes

```bash
# Stage generated code
git add src/generated/

# Stage implementations (if any)
git add src/implementations/

# Update ExifTool submodule reference
git add third-party/exiftool

# Commit with descriptive message
git commit -m "feat: update to ExifTool v12.77

- Add support for Canon new lens type
- Add Nikon Z9 AF mode support
- Update generated tag definitions"
```

## Common Scenarios

### Scenario 1: Tags-Only Update

Most common case - ExifTool adds tags but no new logic:

```bash
# Update submodule
cd third-party/exiftool && git checkout v12.77 && cd ../..

# Regenerate
perl codegen/extract_tables.pl > codegen/generated/tag_tables.json
cargo run -p codegen

# Test and ship
make test
git add -A && git commit -m "chore: update to ExifTool v12.77"
```

### Scenario 2: New Manufacturer Support

When ExifTool adds a new camera manufacturer:

1. Check if it's mainstream (>80% frequency)
2. If yes, implement core processors
3. Start with basic tag extraction
4. Add PrintConv/ValueConv as needed

### Scenario 3: New File Format

When ExifTool adds support for a new file format:

1. Check if format is in scope (image/video metadata)
2. Implement file detection
3. Add basic processor
4. Iterate based on test files

## Troubleshooting

### Extraction Script Fails

```bash
# Ensure you're in the right directory
pwd  # Should show /path/to/exif-oxide

# Check Perl modules
perl -e 'use lib "third-party/exiftool/lib"; use Image::ExifTool; print "OK\n"'
```

### Codegen Errors

```bash
# Clean and rebuild
rm -rf codegen/target
cargo clean -p codegen
cargo build -p codegen
```

### Test Failures

```bash
# Run specific test
cargo test canon_new_lens --no-fail-fast -- --nocapture

# Compare with ExifTool
exiftool -j test.jpg > expected.json
cargo run -- test.jpg > actual.json
diff expected.json actual.json
```

## Best Practices

1. **Review ExifTool Changes**: Read ExifTool's Changes file for breaking changes
2. **Test Incrementally**: Test each new feature separately
3. **Document Issues**: Note any problematic tags in code comments
4. **Prioritize Mainstream**: Focus on tags with >80% usage frequency

## Related Documentation

- [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md) - Unified code generation and implementation guide
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - General development guide