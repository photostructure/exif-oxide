# Testing

## Test Images

Unfortunately, the test images in `$REPO_ROOT/third-party/exiftool/t/images` are
stripped of their data payload, so they test _many_ aspects of metadata parsing,
but whenever possible, we'd _prefer_ to test against full-size out-of-camera
example files. If you cannot find a full-sized example that your current task
demands in `$REPO_ROOT/test-images`, **ask the user** and we'll add it to the
repository.

Two test image directories serve different purposes:

1. **`test-images/`** - Full-size camera files with complete data payloads

   - Use these for development and feature testing
   - Real photos from actual cameras
   - Contains full metadata and image data

2. **`third-party/exiftool/t/images/`** - ExifTool test suite
   - Stripped of image data (metadata only)
   - Tests edge cases and problematic files
   - Use for compatibility testing

When implementing a manufacturer's support, test against both:

- Full files in `test-images/` for realistic testing
- Edge cases in `third-party/exiftool/t/images/` for robustness

## ExifTool Compatibility Testing

The `tests/exiftool_compatibility_tests.rs` provides automated validation against ExifTool's reference output:

### How It Works

1. **Reference Generation**: `tools/generate_exiftool_json.sh` creates snapshots of ExifTool's JSON output for all test images
2. **Comparison Testing**: Tests run exif-oxide against the same images and compare JSON outputs
3. **Normalization Layer**: Handles ExifTool's presentation inconsistencies without changing core parsing logic

### Normalization System

ExifTool has inconsistent PrintConv formatting across manufacturer modules. The test normalization layer standardizes these for API consistency:

**Examples:**

- **FocalLength**: `24` → `"24 mm"`, `1.8` → `"1.8 mm"`, `"24.0 mm"` → `"24 mm"`
- **ExposureTime**: Preserves ExifTool's varied formats - fractions stay strings (`"1/400"`), whole seconds stay numbers (`4`)
- **FNumber**: `14.0` → `14` (cleaning precision while preserving JSON number type)

### Adding New Normalization Rules

When you discover ExifTool inconsistencies, add rules to `get_normalization_rules()`:

```rust
// For unit-based tags
rules.insert("TagName", NormalizationRule::UnitFormat {
    unit: "mm",
    decimal_places: Some(1)
});

// For cleaning numeric precision
rules.insert("TagName", NormalizationRule::CleanNumericPrecision {
    max_places: 1
});
```

This approach follows the updated TRUST-EXIFTOOL principle: preserve core parsing logic while standardizing inconsistent presentation layers.

### Running Compatibility Tests

```bash
# Generate ExifTool reference snapshots
make compat-gen

# Run compatibility tests
make compat-test
```

## Avoid mocks and byte array snippets

- Avoid mocks and stubs where possible.

- Whenever possible, use integration tests that load actual files from
  `$REPO_ROOT/test-images`.
