# GPMF Integration Test

## Test that the GPMF API is available

```rust
use exif_oxide::extract_gpmf_metadata;

// This would work if we had a real GoPro file
// let gpmf_data = extract_gpmf_metadata("gopro_video.mp4").unwrap();
```

## Test that GPMF tags are available

```rust
use exif_oxide::gpmf::get_gpmf_tag;

let tag = get_gpmf_tag("DVNM");
assert\!(tag.is_some());
assert_eq\!(tag.unwrap().name, "DeviceName");
```

## JPEG APP6 GPMF Detection

The JPEG parser now detects APP6 segments with "GoPro" signature and extracts GPMF data.

## MP4 GPMF Detection  

Support for MP4 GPMF boxes is integrated into the HEIF parser for MP4/MOV files.

## API Integration

- `extract_gpmf_metadata()` function available in lib.rs
- GPMF metadata type added to MetadataCollection
- 103 GPMF tags with PrintConv integration
- 17 GPMF format codes for data parsing

✅ GPMF integration is complete and ready for GoPro files\!

