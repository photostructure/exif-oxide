# Binary Tag Extraction in exif-oxide

This document explains how exif-oxide implements binary data extraction for embedded images (thumbnails, previews, etc.) while maintaining compatibility with ExifTool's behavior.

## Overview

Binary extraction in ExifTool is triggered by the `-b` flag, which outputs raw binary data instead of encoded strings. The key binary image tags we need to support are:

- **ThumbnailImage** - Standard EXIF thumbnail (typically in IFD1)
- **ThumbnailTIFF** - TIFF-format thumbnails
- **PreviewImage** - Larger preview images
- **PreviewTIFF** - TIFF-format previews
- **JpgFromRaw** - Embedded JPEG in RAW files (Canon, others)
- **JpgFromRaw2** - Alternate embedded JPEG

## ExifTool's Binary Extraction Mechanism

### 1. The -b Flag

When `-b` is specified in ExifTool:
- Sets `$binaryOutput = 1` in the exiftool script
- Binary values are returned as SCALAR references
- The `ConvertBinary()` function handles output formatting
- For JSON output with `-b`, binary data is base64-encoded with "base64:" prefix

### 2. Binary Tag Storage

ExifTool stores binary data as SCALAR references:
```perl
# In ExifTool
$val = \$binaryData;  # SCALAR ref indicates binary
```

### 3. Tag Types

Binary image tags fall into two categories:

#### Direct Tags
Tags that directly contain binary data:
- Found in maker notes (e.g., Canon's JpgFromRaw at 0x2007)
- Stored as complete binary blobs

#### Composite Tags
Tags constructed from offset/length pairs:
- ThumbnailImage (from ThumbnailOffset + ThumbnailLength)
- PreviewImage (from PreviewImageStart + PreviewImageLength)
- Require both components to extract

## Implementation in exif-oxide

### Phase 1: Core Binary Extraction (Current Priority)

1. **Binary Data Representation**
   ```rust
   pub enum TagValue {
       // ... other variants
       Binary(Vec<u8>),  // Raw binary data
   }
   ```

2. **Extraction Logic**
   ```rust
   // For composite tags like ThumbnailImage
   pub fn extract_thumbnail_image(ifd: &IFD) -> Option<Vec<u8>> {
       let offset = ifd.get_u32(0x0201)?;  // ThumbnailOffset
       let length = ifd.get_u32(0x0202)?;  // ThumbnailLength
       
       // Extract binary data from file at offset
       extract_binary_segment(offset, length)
   }
   ```

3. **Output Formatting**
   - Binary mode: Write raw bytes to stdout
   - JSON mode: Base64-encode with "base64:" prefix
   - Regular mode: Show placeholder like "[Binary data 123456 bytes]"

### Phase 2: Maker-Specific Binary Tags

Each manufacturer has unique binary tags:

#### Canon (CanonRaw.pm)
- **0x2007**: JpgFromRaw - Full-resolution JPEG
- **0x2008**: ThumbnailImage - Small thumbnail
- Uses RawConv validation: `$self->ValidateImage(\$val,$tag)`

#### Nikon
- Various preview formats in maker notes
- Multiple preview sizes (Preview1, Preview2, etc.)

#### Sony
- Embedded previews in ARW files
- Special handling for encrypted data

### Phase 3: Composite Tag System

Implement ExifTool's Composite tag mechanism:

```rust
pub struct CompositeTag {
    name: &'static str,
    requires: Vec<TagId>,
    extract_fn: fn(&HashMap<TagId, TagValue>) -> Option<TagValue>,
}

// Example: ThumbnailImage composite
CompositeTag {
    name: "ThumbnailImage",
    requires: vec![0x0201, 0x0202],  // Offset, Length
    extract_fn: |tags| {
        let offset = tags.get(&0x0201)?.as_u32()?;
        let length = tags.get(&0x0202)?.as_u32()?;
        Some(TagValue::Binary(extract_at_offset(offset, length)?))
    },
}
```

## ExifTool Synchronization Strategy

### Source Files to Track

Add to `exiftool-sync.toml`:

```toml
[binary_extraction]
composite_tags = { source = "lib/Image/ExifTool/Exif.pm:4858-4877", version = "12.65" }
canon_raw = { source = "lib/Image/ExifTool/CanonRaw.pm:345-361", version = "12.65" }
convert_binary = { source = "exiftool:3891-3920", version = "12.65" }

[module_sources]
"src/binary/extraction.rs" = ["lib/Image/ExifTool/Exif.pm", "exiftool"]
"src/maker/canon_binary.rs" = ["lib/Image/ExifTool/CanonRaw.pm", "lib/Image/ExifTool/Canon.pm"]
```

### Attribution in Code

```rust
//! Binary data extraction for embedded images

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm (Composite tags)"]
#![doc = "EXIFTOOL-SOURCE: exiftool (ConvertBinary function)"]
#![doc = "EXIFTOOL-VERSION: 12.65"]

// Implementation...
```

### Monthly Sync Process

1. **Check for Binary Tag Changes**:
   ```bash
   cargo run --bin exiftool_sync diff 12.65 12.66 --focus binary
   ```

2. **Key Areas to Monitor**:
   - New composite tag definitions in Exif.pm
   - Changes to RawConv validation in maker modules
   - New binary tags in manufacturer modules
   - Updates to ConvertBinary() in main script

3. **Update Extraction Logic**:
   - Regenerate offset/length mappings
   - Add new manufacturer-specific tags
   - Update validation routines

## Testing Strategy

### 1. Direct Comparison Tests

```rust
#[test]
fn test_thumbnail_extraction_canon() {
    let image = include_bytes!("../test-images/canon/Canon_40D.JPG");
    let our_thumb = extract_thumbnail(image).unwrap();
    
    // Compare with ExifTool output
    let exiftool_thumb = std::fs::read("test-images/canon/Canon_40D_thumb_exiftool.jpg").unwrap();
    assert_eq!(our_thumb, exiftool_thumb);
}
```

### 2. Validation Tests

```rust
#[test]
fn test_binary_validation() {
    // Test that we properly validate JPEG data
    let invalid = vec![0xFF, 0xD8, 0x00, 0x00];  // Invalid JPEG
    assert!(validate_jpeg(&invalid).is_err());
}
```

### 3. Edge Cases

- Zero-length binary tags
- Missing offset or length
- Offsets beyond file size
- Encrypted/obfuscated data

## Implementation Checklist

- [ ] Core binary data type (`TagValue::Binary`)
- [ ] Basic thumbnail extraction (IFD1)
- [ ] Binary output mode (`-b` equivalent)
- [ ] JSON base64 encoding
- [ ] Composite tag framework
- [ ] Canon JpgFromRaw extraction
- [ ] Nikon preview extraction
- [ ] Sony preview extraction
- [ ] Binary validation (JPEG, TIFF)
- [ ] Offset bounds checking
- [ ] ExifTool compatibility tests
- [ ] Sync tooling for binary tags

## Performance Considerations

1. **Memory Efficiency**:
   - Stream large binaries instead of loading into memory
   - Use memory-mapped files for large RAW files

2. **Lazy Extraction**:
   - Only extract binary data when requested
   - Cache extracted binaries for multiple accesses

3. **Validation**:
   - Quick magic number check before full validation
   - Skip validation in fast mode

## Security Considerations

1. **Bounds Checking**:
   - Always validate offset + length <= file_size
   - Prevent integer overflow in calculations

2. **Resource Limits**:
   - Maximum binary size limits
   - Timeout for extraction operations

3. **Validation**:
   - Validate image headers before returning
   - Option to skip validation for trusted sources

## References

- ExifTool Composite tags: `lib/Image/ExifTool/Exif.pm:4858+`
- Canon binary tags: `lib/Image/ExifTool/CanonRaw.pm:345+`
- Binary conversion: `exiftool:3891-3920 (ConvertBinary)`
- Validation: Various `ValidateImage()` calls in maker modules

## Future Enhancements

1. **Streaming API**: For extracting large previews without full file load
2. **Multi-preview**: Extract all available previews in one pass
3. **Format conversion**: On-the-fly TIFF to JPEG conversion
4. **Thumbnail generation**: Create thumbnails if missing