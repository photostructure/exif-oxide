# exif-oxide

A high-performance Rust implementation of ExifTool, providing fast metadata extraction and manipulation for images and media files.

## Status

**🚧 Under Development** - This project is in early development. See [SPIKES.md](SPIKES.md) for the development roadmap.

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
✅ **Spike 1.5 Complete**: Table generation from ExifTool Perl modules
✅ **Spike 2 Complete**: Maker notes parsing (Canon, Nikon, Sony)
✅ **Spike 3 Complete**: Binary tag extraction (thumbnails and previews)
🚧 **Spike 4 In Progress**: XMP reading and writing (Phase 1 of 5 complete)

## Quick Start

### Library Usage

```rust
use exif_oxide::{read_basic_exif, extract_thumbnail, read_xmp};

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
    if let Ok(thumbnail) = extract_thumbnail("photo.jpg") {
        std::fs::write("thumbnail.jpg", thumbnail)?;
        println!("Thumbnail extracted!");
    }
    
    // Read XMP metadata (basic properties)
    if let Ok(xmp) = read_xmp("photo.jpg") {
        for (key, value) in &xmp.properties {
            println!("XMP {}: {}", key, value);
        }
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

# Run integration tests with ExifTool test images
cargo test --test spike1
cargo test --test spike3  # Thumbnail extraction tests
```

## Development

Next: [Spike 4](doc/SPIKES.md#spike-4-xmp-reading-and-writing) - XMP reading and writing support.

## License

This project is licensed under the same terms as ExifTool:
- Free for personal, educational, and non-profit use
- Commercial use requires licensing

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
- The exiftool-vendored project for datetime parsing heuristics
- The Rust community for excellent binary parsing libraries