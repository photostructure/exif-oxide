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

## Supported Formats

**Image Formats**: JPEG, TIFF, PNG, HEIF/HEIC, WebP  
**RAW Formats**: CR2, NEF, ARW, DNG, PEF, ORF, RAF, RW2, SRW, 3FR, IIQ, MEF, MOS, MRW, CRW, SR2, NRW  
**Video Formats**: MP4, M4V, MOV, 3GP, 3G2, AVI

Total: 26 formats with metadata extraction support

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

### Command Line

```bash
# Build the project
cargo build --release

# Extract all EXIF data as JSON
cargo run --bin exif-oxide -- photo.jpg

# Extract specific tags
cargo run --bin exif-oxide -- -Make -Model photo.jpg

# Extract binary data (e.g., thumbnail)
cargo run --bin exif-oxide -- -b -ThumbnailImage photo.jpg > thumb.jpg

# Extract with group names
cargo run --bin exif-oxide -- -G photo.jpg
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

### Major Milestones Complete! ✅

**Phase 0 - Synchronization Infrastructure**: Complete
- Auto-generated maker note detection for all 10 manufacturers
- ProcessBinaryData table extraction with 530+ tags  
- Table-driven PrintConv system achieving 96% code reduction
- Smooth regeneration system with zero manual maintenance

**Phase 1 - Multi-Format Support**: Complete  
- 26 file formats supported (JPEG, RAW, PNG, HEIF, MP4, etc.)
- ExifTool-compatible metadata extraction
- Embedded binary extraction (thumbnails, previews)

**EXIF Migration - Revolutionary Improvement**: Complete ✅
- **28x improvement**: 643 EXIF tags extracted vs previous ~23
- **87% coverage gap eliminated**: Now extracting comprehensive EXIF data
- **Table-driven architecture**: Leveraging proven sync extractor pattern
- **Zero regressions**: All 123 tests passing with full backward compatibility

See:
- [PHASE1-COMPLETE.md](doc/PHASE1-COMPLETE.md) for Phase 1 summary
- [EXIFTOOL-SYNC.md](doc/EXIFTOOL-SYNC.md) for synchronization infrastructure
- [TODO-MIGRATE-EXIF-TO-SYNC.md](doc/TODO-MIGRATE-EXIF-TO-SYNC.md) for EXIF migration details

### Next Steps (Phase 2-4):
- Additional manufacturer maker notes (Ricoh, PhaseOne, Qualcomm, Red expansion)
- Metadata writing capabilities with safety guarantees  
- Performance optimizations (SIMD, parallel processing)
- Async API and plugin system

## License

This project is licensed under the same terms as ExifTool: **GNU General Public License v3 (GPL-3.0)**

ExifTool is free and open-source software that allows personal, commercial, and any other use without separate licensing fees.

## Tag Naming Compatibility

ExifTool tag names are preserved when possible, but tags with invalid Rust identifier characters are modified:

- **ExifTool**: `"NikonActiveD-Lighting"`
- **exif-oxide**: `"NikonActiveD_Lighting"`

Hyphens are converted to underscores to maintain valid Rust identifiers while preserving semantic meaning.

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
- The Rust community for excellent binary parsing libraries