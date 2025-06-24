# TODO: Enhance find_metadata_segment for Multi-Format Support

## Current Limitations

The current `find_metadata_segment` function in `src/core/mod.rs` only supports JPEG files and specifically only extracts EXIF data from APP1 segments. This causes several issues:

1. **Limited File Format Support**: Only JPEG files can have their metadata extracted, despite the detection system supporting 43+ formats
2. **No MPF Support**: Modern cameras (like Canon R50) store preview images in MPF (Multi-Picture Format) which uses APP2 segments
3. **No RAW Support**: CR2, NEF, ARW and other RAW formats are TIFF-based and need different parsing
4. **Single Segment Only**: Only the first APP1 segment is extracted, missing additional metadata in other segments

## Example Problems

### Canon R50 Preview Extraction Fails

```bash
$ target/debug/exif-oxide test-images/canon/canon_eos_r50v_01.jpg -PreviewImage -b > img.jpg
Error: "Unknown tag: PreviewImage"

$ exiftool test-images/canon/canon_eos_r50v_01.jpg -PreviewImage -b > img.jpg
# Works fine - extracts 620KB preview from MPF segment
```

### CR2 Files Not Supported

```bash
$ cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.CR2
Error: No EXIF data found in 'test-images/canon/Canon_T3i.CR2'
```

## Technical Requirements

### 1. Refactor find_metadata_segment to Support Multiple Formats

Current implementation (simplified):

```rust
pub fn find_metadata_segment<P: AsRef<Path>>(path: P) -> Result<Option<MetadataSegment>> {
    // Only handles JPEG APP1 segments
    if /* is JPEG */ {
        // Find APP1 with "Exif\0\0"
    }
    Ok(None) // Returns None for all non-JPEG files!
}
```

Needed:

```rust
pub fn find_metadata_segment<P: AsRef<Path>>(path: P) -> Result<Option<MetadataSegment>> {
    let file_info = detect_file_type_from_path(&path)?;

    match file_info.file_type {
        FileType::JPEG => extract_jpeg_metadata(&path)?,
        FileType::CR2 | FileType::NEF | FileType::ARW => extract_tiff_metadata(&path)?,
        FileType::HEIF | FileType::HEIC => extract_heif_metadata(&path)?,
        FileType::PNG => extract_png_metadata(&path)?,
        // ... other formats
        _ => Ok(None)
    }
}
```

### 2. Add MPF (Multi-Picture Format) Support

MPF is used by many modern cameras for storing multiple images (previews, thumbnails) in a single JPEG file.

Structure:

- Stored in APP2 segment (0xFFE2)
- Marker: "MPF\0" (similar to "Exif\0\0")
- Contains IFD structure similar to EXIF
- References multiple images within the file

Key tags in MPF:

- 0xB000: MPFVersion
- 0xB001: NumberOfImages
- 0xB002: MPEntry (array of image entries)
- 0xB003: ImageUIDList
- 0xB004: TotalFrames

Each MPEntry contains:

- Image attributes
- Size
- Data offset (from start of MPF marker)
- Dependent image entry numbers

### 3. Support Multiple Metadata Segments

Current code returns after finding first APP1. Need to:

- Collect all APP1 segments (EXIF, XMP)
- Collect APP2 segments (MPF, FlashPix)
- Collect APP13 segments (Photoshop/IPTC)
- Return a vector or combined metadata structure

### 4. Add TIFF-Based Format Support

For CR2, NEF, ARW, DNG files:

- These are TIFF-based formats
- Metadata starts at file beginning (no APP segments)
- IFD0 contains main image metadata
- SubIFDs contain previews/thumbnails
- Maker notes in ExifIFD

## Implementation Steps

1. **Refactor MetadataSegment struct** to handle different sources:

   ```rust
   pub enum MetadataSource {
       JpegApp1,
       JpegApp2Mpf,
       TiffIfd,
       HeifMeta,
       PngChunk,
   }

   pub struct MetadataSegment {
       pub data: Vec<u8>,
       pub offset: usize,
       pub source: MetadataSource,
   }
   ```

2. **Create format-specific extractors**:

   - `extract_jpeg_metadata()` - Handle APP1, APP2, etc.
   - `extract_tiff_metadata()` - Handle TIFF-based RAW formats
   - `extract_heif_metadata()` - Handle HEIF/HEIC
   - `extract_png_metadata()` - Handle PNG chunks

3. **Update IfdParser** to handle different starting contexts:

   - JPEG: TIFF header after "Exif\0\0"
   - TIFF-based: TIFF header at file start
   - MPF: Similar to EXIF but different tag definitions

4. **Extend binary extraction** to understand MPF offsets:
   - MPF offsets are from the MPF marker, not TIFF header
   - Need to track different offset bases

## Testing

Test files needed:

- Modern JPEG with MPF (Canon R50, Sony A7 IV, etc.)
- Various RAW formats (CR2, CR3, NEF, ARW, DNG)
- HEIF/HEIC files
- PNG with EXIF chunks

Expected outcomes:

- `exif-oxide -PreviewImage -b canon_r50.jpg` should extract the 620KB preview
- `exif-oxide canon_photo.cr2` should show all EXIF data
- `exif-oxide -ThumbnailImage -b photo.nef` should extract embedded thumbnail

## References

- ExifTool MPF implementation: `lib/Image/ExifTool/MPF.pm`
- MPF Specification: CIPA DC-007 (Multi-Picture Format)
- TIFF specification for understanding IFD structures
- ExifTool's format-specific modules for implementation patterns

## Priority

This is **HIGH PRIORITY** for Phase 1 because:

- Main.rs already detects 43 formats but can only read JPEG
- Users expect basic tag reading to work across formats
- Binary extraction (thumbnails/previews) is a key feature
- Current limitation makes the tool much less useful than ExifTool
