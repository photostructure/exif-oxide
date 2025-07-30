# Troubleshooting Guide

**🚨 CRITICAL: All debugging approaches must respect [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - use ExifTool as ground truth.**

This guide helps you debug and troubleshoot issues in exif-oxide development and usage.

## Quick Debugging Workflow

### 1. Compare with ExifTool

Always start by confirming what ExifTool actually does:

```bash
# See exactly what ExifTool extracts
exiftool -v3 image.jpg > exiftool_verbose.txt

# Get JSON output for comparison
exiftool -j image.jpg > expected.json

# See hex dump of specific tag
exiftool -htmlDump image.jpg > dump.html
```

### 2. Enable Trace Logging

```bash
# Enable trace logging
RUST_LOG=trace cargo run -- test.jpg

# Use ExifTool verbose mode for comparison
exiftool -v3 test.jpg

# Check specific tag extraction
cargo run -- test.jpg | jq '.["EXIF:Orientation"]'
```

### 3. Use --show-missing

```bash
# Find what implementations are missing
cargo run -- image.jpg --show-missing
```

This tells you exactly what needs to be implemented.

## Common Issues and Solutions

### Issue: Tag Values Don't Match ExifTool

**Symptoms:**

- Output differs from `exiftool -j`
- Missing tags that ExifTool shows
- Wrong values for existing tags

**Debug Steps:**

1. **Check ExifTool verbose output:**

   ```bash
   exiftool -v3 image.jpg | grep -A5 -B5 "TagName"
   ```

2. **Verify raw binary data:**

   ```bash
   exiftool -htmlDump image.jpg > dump.html
   # Open dump.html in browser to see hex data
   ```

3. **Add debug logging:**
   ```rust
   trace!("Tag {:04x}: raw_value={:?}", tag_id, raw_value);
   trace!("After ValueConv: {:?}", converted_value);
   trace!("After PrintConv: {:?}", display_value);
   ```

**Common Causes:**

- Wrong byte order interpretation
- Incorrect offset calculations
- Missing ValueConv/PrintConv implementation
- Wrong format interpretation

### Issue: Wrong File Type / MIME Type Detection

**Symptoms:**

- File type doesn't match ExifTool
- Wrong MIME type reported
- NEF/NRW confusion

**Debug Steps:**

1. **Check ExifTool's detection:**

   ```bash
   # See what file type ExifTool detects
   exiftool -FileType -MIMEType image.nef
   
   # Check compression values in all IFDs
   exiftool -a -n -Compression -G1 image.nef
   
   # Check for NEFLinearizationTable
   exiftool -NEFLinearizationTable image.nef
   ```

2. **Understand detection approaches:**
   - ExifTool uses complex content analysis with multi-stage overrides
   - exif-oxide trusts file extensions for NEF/NRW files (by design)
   - Check IFD0 compression values to understand the file structure

3. **Design Decisions:**
   - **NEF/NRW**: We trust file extensions (predictable behavior)
   - This avoids false positives from incomplete content analysis
   - See [MANUFACTURER-FACTS.md](MANUFACTURER-FACTS.md#22-nef-vs-nrw-file-type-detection) for rationale

**Common Causes:**

- Relying only on file extension
- Not checking IFD0 compression
- Missing content-based overrides
- Can't access MakerNotes data for detection

### Issue: Offset Calculation Errors

**Symptoms:**

- Reading wrong data locations
- Crashes on bounds checking
- Corrupted tag values

**Debug Steps:**

1. **Verify offset calculations:**

   ```rust
   assert_eq!(
       calculated_offset,
       expected_offset,
       "Offset mismatch for tag {}: calculated {:#x} != expected {:#x}",
       tag_name, calculated_offset, expected_offset
   );
   ```

2. **Check manufacturer-specific offset schemes:**

   - Canon: 4, 6, 16, or 28 byte offsets
   - Nikon: TIFF header at offset 0x0a
   - Sony: Various detection patterns

3. **Validate against ExifTool:**
   ```bash
   exiftool -v3 image.jpg | grep -i "base\|offset"
   ```

**Common Causes:**

- Wrong base offset for manufacturer
- Missing TIFF footer validation
- Incorrect entry-based offset handling

### Issue: Binary Data Processing Fails

**Symptoms:**

- Canon/Nikon manufacturer-specific tags missing
- Binary lens data not extracted
- ProcessBinaryData errors

**Debug Steps:**

1. **Verify binary data extraction:**

   ```rust
   trace!("Binary data length: {}", binary_data.len());
   trace!("First 16 bytes: {:02x?}", &binary_data[..16]);
   ```

2. **Check table definitions:**

   - Verify FIRST_ENTRY offset
   - Check format overrides
   - Validate offset ranges

3. **Compare with ExifTool source:**
   ```bash
   grep -r "ProcessBinaryData" third-party/exiftool/lib/Image/ExifTool/Canon.pm
   ```

**Common Causes:**

- Wrong binary data tag identification
- Incorrect offset calculations within binary data
- Missing format override handling

### Issue: String Encoding Problems

**Symptoms:**

- Garbled text in string fields
- Incorrect character display
- UTF-8 encoding errors

**Debug Steps:**

1. **Check for double-encoding:**

   ```rust
   if looks_like_double_utf8(&string) {
       string = decode_utf8_twice(string);
   }
   ```

2. **Verify null termination:**

   ```rust
   // Scan for null terminator, don't assume clean strings
   let null_pos = string.bytes().position(|b| b == 0);
   ```

3. **Compare raw bytes:**
   ```bash
   exiftool -v3 image.jpg | grep -A3 "string field"
   ```

**Common Causes:**

- Sony double-UTF8 encoding
- Manufacturer-specific character encodings
- Garbage data after null terminators

### Issue: Performance Problems

**Symptoms:**

- Slow processing compared to ExifTool
- Memory usage issues
- Timeout errors

**Debug Steps:**

1. **Profile with perf:**

   ```bash
   cargo build --release
   perf record --call-graph=dwarf target/release/exif-oxide image.jpg
   perf report
   ```

2. **Check memory usage:**

   ```bash
   valgrind --tool=massif target/release/exif-oxide image.jpg
   ```

3. **Measure against ExifTool:**
   ```bash
   time exiftool image.jpg
   time target/release/exif-oxide image.jpg
   ```

**Common Causes:**

- Loading entire file instead of streaming
- Inefficient lookup table access
- Unnecessary memory allocations

### Issue: Test Failures

**Symptoms:**

- Compatibility tests fail
- Unit tests break after changes
- Clippy import errors

**Debug Steps:**

1. **Run specific test:**

   ```bash
   cargo test canon_lens --no-fail-fast -- --nocapture
   ```

2. **Check test data:**

   ```bash
   # Make sure test images exist
   ls test-images/Canon/

   # Verify ExifTool reference data (generates missing files)
   make compat-gen

   # Force regenerate all reference data if needed
   make compat-gen-force
   ```

3. **Fix clippy import issues:**

   ```rust
   // Use module-level cfg(test) imports
   #[cfg(test)]
   use crate::types::TagValue;

   #[test]
   fn my_test() {
       let value = TagValue::String("test".to_string());
   }
   ```

**Common Causes:**

- Missing test helper features
- Incorrect test file paths
- Clippy import analysis issues

## Build and Development Issues

### Issue: Codegen Fails

**Symptoms:**

- `cargo run -p codegen` errors
- Missing generated files
- Compilation errors in generated code

**Debug Steps:**

1. **Check Perl dependencies:**

   ```bash
   perl -e 'use lib "third-party/exiftool/lib"; use Image::ExifTool; print "OK\n"'
   ```

2. **Verify ExifTool submodule:**

   ```bash
   cd third-party/exiftool
   git status
   git log --oneline -5
   ```

3. **Clean and rebuild:**
   ```bash
   make clean
   cargo clean -p codegen
   make codegen
   ```

**Common Causes:**

- Outdated ExifTool submodule
- Missing Perl modules
- Syntax errors in extraction scripts

### Issue: Compilation Errors

**Symptoms:**

- Rust compilation fails
- Linking errors
- Missing dependencies

**Debug Steps:**

1. **Check Rust toolchain:**

   ```bash
   rustc --version
   cargo --version
   ```

2. **Verify dependencies:**

   ```bash
   cargo check
   cargo update
   ```

3. **Clean build:**
   ```bash
   cargo clean
   cargo build
   ```

**Common Causes:**

- Outdated Rust toolchain
- Missing system dependencies
- Incompatible dependency versions

## Debugging Specific Manufacturers

### Canon Issues

**Common Problems:**

- LensType not extracted
- Binary data not found
- Wrong offset schemes

**Debug with:**

```bash
# Check Canon-specific verbose output
exiftool -v3 -Canon:all image.jpg

# Look for binary data tags
exiftool -v3 image.jpg | grep -i "binary\|canon"
```

### Nikon Issues

**Common Problems:**

- Encrypted sections not handled
- Wrong TIFF header offset
- Version detection failures

**Debug with:**

```bash
# Check Nikon-specific data
exiftool -v3 -Nikon:all image.jpg

# Look for encryption indicators
exiftool -v3 image.jpg | grep -i "encrypt\|nikon"
```

### Sony Issues

**Common Problems:**

- Double-UTF8 encoding
- Multiple maker note formats
- Encryption on newer models

**Debug with:**

```bash
# Check Sony-specific data
exiftool -v3 -Sony:all image.jpg

# Look for encoding issues
exiftool -v3 image.jpg | grep -i "utf\|sony"
```

## Advanced Debugging Techniques

### Hex Dump Analysis

```bash
# Create detailed hex dump
xxd image.jpg | head -100

# Look for specific signatures
strings image.jpg | grep -i "canon\|nikon\|sony"

# Check TIFF structure
exiftool -v4 image.jpg | less
```

### Memory Debugging

```bash
# Check for memory leaks
valgrind --leak-check=full target/debug/exif-oxide image.jpg

# Profile memory usage
heaptrack target/debug/exif-oxide image.jpg
```

### Binary Data Investigation

```rust
// Add detailed binary logging
fn debug_binary_data(data: &[u8], tag_name: &str) {
    trace!("Binary data for {}: {} bytes", tag_name, data.len());
    trace!("First 32 bytes: {:02x?}", &data[..std::cmp::min(32, data.len())]);

    // Look for patterns
    for (i, &byte) in data.iter().enumerate() {
        if byte == 0xFF || byte == 0x00 {
            trace!("Special byte {} at offset {}", byte, i);
        }
    }
}
```

### Is this tag prevalent?

Tag "popularity" ranges widely. First check `docs/tag-metadata.json`, but if it's not there, use `exiftool` to do your own research! For the `ISOSpeed` tag, for example:

```
exiftool -j -struct -G -r -if '$ISOSpeed' -ISOSpeed test-images/ third-party/exiftool/t/images/ ../test-images/
```

Which shows it's only in the EXIF group, and in 20/10693 of our sample files.

## Getting Help

1. **Read ExifTool source** - The answer is usually there
2. **Check ExifTool verbose output** - Shows exactly what it's doing
3. **Use real test images** - Not synthetic data
4. **Ask specific questions** - Include ExifTool output and error messages
5. **Reference TRUST-EXIFTOOL.md** - When in doubt, copy ExifTool exactly

## Prevention Tips

1. **Always test against ExifTool** - Make it a habit
2. **Use real camera files** - Not minimal test cases
3. **Add comprehensive logging** - Trace every step
4. **Validate assumptions** - Check offset calculations
5. **Follow manufacturer quirks** - Don't try to "fix" odd behavior

Remember: If ExifTool does something weird, there's probably a camera somewhere that requires that exact behavior. Trust the accumulated knowledge of 25 years of reverse engineering!
