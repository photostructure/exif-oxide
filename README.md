# exif-oxide

A high-performance Rust implementation of ExifTool, providing fast metadata extraction and manipulation for images and media files.

## Status

**✅ Core Features Complete** - All major metadata extraction capabilities implemented. See [SPIKES.md](doc/SPIKES.md) for development details.

## Goals

- **Performance**: 10-20x faster than Perl ExifTool for common operations
- **Compatibility**: Maintain ExifTool tag naming and structure
- **Safety**: Memory-safe handling of untrusted files
- **Features**: Embedded image extraction, datetime intelligence, XMP support

## Why exif-oxide?

While ExifTool is the gold standard for metadata extraction, its Perl implementation can be slow for high-volume applications. exif-oxide aims to provide:

1. Sub-10ms metadata extraction for typical JPEG files
2. Native embedded preview/thumbnail extraction
3. Intelligent datetime parsing with timezone inference
4. ExifTool-compatible output for easy migration

## Design

See [DESIGN.md](DESIGN.md) for architectural details and [ALTERNATIVES.md](ALTERNATIVES.md) for why we chose this approach.

## Current Status

✅ **Spike 1 Complete**: Basic EXIF reading (Make, Model, Orientation) from JPEG files
✅ **Spike 1.5 Complete**: Table generation from ExifTool Perl modules (530 tags)
✅ **Spike 2 Complete**: Maker notes parsing (Canon with 34 tags)
✅ **Spike 3 Complete**: Binary tag extraction (thumbnails and previews)
✅ **Spike 4 Complete**: XMP reading with hierarchical data structures

### Available Features

- **EXIF Metadata**: Complete IFD parsing with 496 standard tags
- **Maker Notes**: Canon-specific tags (34 supported)
- **Binary Extraction**: Thumbnails and preview images from all manufacturers
- **XMP Support**: Full XML parsing with arrays, structs, and language alternatives
- **Performance**: Sub-10ms parsing for typical JPEG files
- **Cross-Platform**: Tested on Canon, Nikon, Sony, Fujifilm, Panasonic

## Quick Start

### Library Usage

```rust
use exif_oxide::{read_basic_exif, extract_thumbnail, extract_xmp_properties};

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
    
    // Extract embedded thumbnail
    if let Some(thumbnail) = extract_thumbnail("photo.jpg")? {
        std::fs::write("thumbnail.jpg", thumbnail)?;
        println!("Thumbnail extracted!");
    }
    
    // Extract largest preview image (Canon preview or EXIF thumbnail)
    if let Some(preview) = exif_oxide::extract_largest_preview("photo.jpg")? {
        std::fs::write("preview.jpg", preview)?;
        println!("Preview extracted!");
    }
    
    // Read XMP metadata (hierarchical properties)
    let xmp_props = extract_xmp_properties("photo.jpg")?;
    for (key, value) in &xmp_props {
        println!("XMP {}: {}", key, value);
    }
    
    Ok(())
}
```

### Command Line

```bash
# Build the project
cargo build --release

# Extract basic EXIF data
cargo run --bin exif-oxide -- photo.jpg

# Example output:
# EXIF Data for: photo.jpg
#   Make: Canon
#   Model: Canon EOS DIGITAL REBEL
#   Orientation: 1 (Normal)
```

### Testing

```bash
# Run all tests
cargo test

# Run specific feature tests
cargo test --test spike1   # Basic EXIF reading
cargo test --test spike2   # Canon maker notes  
cargo test --test spike3   # Thumbnail extraction
cargo test --test spike4_xmp  # XMP parsing

# Test with real images
cargo run --example debug_exif exiftool/t/images/Canon.jpg
cargo run --example debug_xmp_extraction test-images/canon/Canon_T3i.JPG
```

## Development

Core functionality complete! See [SPIKES.md](doc/SPIKES.md) for completed development phases.

Future enhancements:
- Additional manufacturer maker notes (Nikon, Sony, Fujifilm)
- XMP writing capabilities
- RAW format support (CR2, NEF, ARW, DNG)
- Advanced datetime heuristics

## License

This project is licensed under the same terms as ExifTool: **GNU General Public License v3 (GPL-3.0)**

ExifTool is free and open-source software that allows personal, commercial, and any other use without separate licensing fees.

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
- The exiftool-vendored project for datetime parsing heuristics
- The Rust community for excellent binary parsing libraries