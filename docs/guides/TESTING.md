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

## Avoid mocks and byte array snippets

- Avoid mocks and stubs where possible.

- Whenever possible, use integration tests that load actual files from
`$REPO_ROOT/test-images`. 

## Always validate our output with `exiftool`

- Group names, tag names, ValueConf and PrintConf results should match VERBATIM. 

  It's only by testing agains



