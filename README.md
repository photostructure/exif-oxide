# exif-oxide

A high-performance Rust implementation of portions of ExifTool, providing fast metadata extraction from 26+ image and media file formats.

## Goals

- **Performance**: 10-20x faster than Perl ExifTool for common operations
- **Multi-Format**: Support for JPEG, RAW (CR2, NEF, ARW, DNG), PNG, HEIF, WebP, MP4, and more
- **Compatibility**: Maintain ExifTool tag naming and structure (with Rust identifier constraints)
- **Safety**: Memory-safe handling of untrusted files
- **Features**: Embedded image extraction, XMP support

## Why exif-oxide?

While ExifTool is the gold standard for metadata extraction, its Perl implementation can be slow for high-volume applications. exif-oxide aims to provide:

1. Sub-10ms metadata extraction for typical JPEG files
2. Native embedded preview/thumbnail extraction
3. ExifTool-compatible output for easy migration

## Design

See [DESIGN.md](DESIGN.md) for architectural details and [ALTERNATIVES.md](ALTERNATIVES.md) for why we chose this approach.

## Quick Start

### Library Usage

```rust
use exif_oxide::{read_basic_exif, extract_xmp_properties};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Extract basic EXIF metadata
    let exif = read_basic_exif("photo.jpg")?;

    println!("Camera: {} {}",
        exif.make.as_deref().unwrap_or("Unknown"),
        exif.model.as_deref().unwrap_or("Unknown")
    );

    if let Some(orientation) = exif.orientation {
        println!("Orientation: {}", orientation);
    }

    // Read XMP metadata (hierarchical properties)
    let xmp_props = extract_xmp_properties("photo.jpg")?;
    for (key, value) in &xmp_props {
        println!("XMP {}: {}", key, value);
    }

    Ok(())
}
```

## Development

This project includes `exiftool` as a submodule: use `git clone --recursive`.

## Tag Naming Compatibility

ExifTool tag names are preserved when possible, but tags with invalid Rust identifier characters are modified:

- **ExifTool**: `"NikonActiveD-Lighting"`
- **exif-oxide**: `"NikonActiveD_Lighting"`

Hyphens are converted to underscores to maintain valid Rust identifiers while preserving semantic meaning.

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
- The Rust community for excellent binary parsing libraries
