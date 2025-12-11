# exif-oxide: Rust translation of ExifTool

> **⚠️ Under Heavy Development** — This project is not yet ready for production use. APIs may change without notice, and many features are incomplete. See [Current Status](#current-status) for details.

High-performance Rust implementation of ExifTool's metadata extraction, translating ExifTool's battle-tested logic to provide a single binary solution with 10-100x faster performance.

## Why exif-oxide?

ExifTool is the industry standard for metadata extraction, but modern antivirus software often flags Perl interpreters as suspicious, causing significant performance degradation and false-positive security warnings.

`exif-oxide` provides a single, native binary that eliminates these antivirus conflicts while delivering 10x faster batch processing and 30-100x faster single-file operations.

## Current Status

### ✅ **Working Now**

- **JPEG metadata extraction** - Complete EXIF, XMP, GPS, and maker notes
- **Multiple RAW formats** - Kyocera, Minolta MRW, Panasonic RW2/RWL (100% compatibility achieved)
- **File type detection** - 122 formats with 100% ExifTool-compatible MIME types
- **Canon cameras** - Full maker note support with ProcessBinaryData
- **Sony cameras** - Complete maker note integration with human-readable tag names
- **Nikon cameras** - Maker note support with encryption detection and lens database
- **GPS coordinates** - Decimal degree conversion and composite tags
- **Composite tags** - Advanced calculations like ShutterSpeed
- **CLI compatibility** - JSON output with -TagName# numeric mode
- **PPI→AST→Rust pipeline** - Production-ready code generation with precedence climbing

### 🚧 **In Progress**

- **Olympus ORF** - 90% complete, resolving tag ID conflicts
- **Canon CR2/CR3** - Implementing SHORT array extraction for binary data
- **Sony ARW** - Tag naming complete, expanding ProcessBinaryData coverage
- **Video metadata** - MP4, MOV, and other video formats (Milestone 18)

### 📋 **Planned**

- **Write support** - Tag modification and file updates (Milestones 21-22)
- **Binary data extraction** - Thumbnails and embedded images (Milestone 19)
- **Error classification** - Detailed error reporting (Milestone 20)

## Project Goals

- **Trust ExifTool**: Translate ExifTool's logic exactly—no "improvements" or novel parsing
- **Mainstream focus**: Support 500-1000 most common tags (>80% frequency) vs ExifTool's 15,000+
- **Performance**: 8-15x faster batch processing, 30x faster single-file operations
- **Compatibility**: Maintain behavioral compatibility with ExifTool output
- **Maintainability**: Use code generation to track ExifTool's monthly updates

## Differences from ExifTool

- **JSON-only output**: Always outputs JSON (no text mode)
- **Mainstream tags**: Focuses on ~500-1000 most common tags vs ExifTool's 15,000+
- **No geolocation**: Excludes ExifTool's GPS coordinate lookup features
- **File type detection**: Trusts file extensions for NEF/NRW distinction (ExifTool uses content analysis)
- **No write patterns**: No pattern-match replacements for tag updates
- **No custom configuration**: ExifTool has a rich featureset for custom tag extraction and rendering. We're not porting that over.
- **Unknown tags omitted**: Tags marked as "Unknown" in ExifTool are filtered out by default (ExifTool shows these with the `-u` flag)

## How It Works

exif-oxide uses a sophisticated hybrid approach combining automated code generation with manual translation:

### Code Generation Pipeline

The project features a production-ready **PPI→AST→Rust** code generation pipeline that automatically extracts and translates ExifTool's Perl code:

1. **PPI Parser** - Parses ExifTool's Perl source into an abstract syntax tree
2. **AST Normalization** - Multi-pass normalization using precedence climbing for correct operator handling
3. **Strategy Pattern** - 9 competing strategies automatically discover and claim ExifTool symbols
4. **Rust Generation** - Produces type-safe Rust code with full ExifTool compatibility

This pipeline automatically extracts:
- 3,000+ lookup tables (camera models, lens IDs, white balance settings)
- Tag definitions with metadata
- File type detection patterns
- PrintConv/ValueConv expressions

### Manual Excellence

Complex processing logic is carefully hand-ported with ExifTool source references:
- Manufacturer-specific quirks (Canon/Nikon/Sony offset calculations)
- Binary data extraction algorithms
- Encryption handling
- Composite tag calculations

This hybrid approach ensures perfect compatibility with ExifTool's 25+ years of camera-specific quirks while maintaining a maintainable codebase that can track ExifTool's monthly updates. See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for technical details.

## Installation

### Pre-built binaries (recommended)

Download from [GitHub Releases](https://github.com/photostructure/exif-oxide/releases) - binaries available for:

| Platform | Architecture |
|----------|--------------|
| Linux    | x86_64, ARM64 |
| macOS    | Intel, Apple Silicon |
| Windows  | x86_64, ARM64 |

### Using cargo-binstall

```bash
cargo binstall exif-oxide
```

### From source

```bash
cargo install exif-oxide
```

## Usage

```bash
# Extract all metadata as JSON
exif-oxide photo.jpg

# Multiple files
exif-oxide photo1.jpg photo2.jpg

# Show numeric values with -TagName# syntax
exif-oxide photo.jpg -FNumber# -ExposureTime#

# Show implementation status
exif-oxide --show-missing photo.jpg
```

## Library Usage

```rust
use exif_oxide::formats::extract_metadata;
use std::path::Path;

let metadata = extract_metadata(Path::new("photo.jpg"), false, false)?;
for tag in &metadata.tags {
    println!("{}: {}", tag.tag_name, tag.print_value);
}
```

## Licensing

Dual-licensed under commercial license and GNU Affero General Public License v3.0+. See [LICENSE](./LICENSE) for details.

## Development

> **Note**: This repository was rebuilt in September 2025 to remove git LFS due to excessive storage costs from our large test image corpus. The previous repository was archived and made private to stem the bleeding. Test images are now stored separately and excluded from version control.

### Quick Start

1. Clone with submodules:

   ```bash
   git clone --recursive https://github.com/photostructure/exif-oxide
   cd exif-oxide
   ```

2. Build and test:

   ```bash
   make precommit  # Build, test, and lint
   # Or run tests directly:
   cargo t         # Run tests (use 'cargo t' not 'cargo test')
   ```

3. Run on test images:
   ```bash
   cargo run test-images/canon/Canon_T3i.JPG
   ```

### Architecture Highlights

- **PPI→AST→Rust pipeline**: Production-ready code generation with precedence climbing parser
- **Strategy pattern**: 9 competing strategies auto-discover ExifTool symbols (no config needed)
- **Hybrid approach**: Generated static data (3,000+ tables) + manual complex logic
- **Runtime registries**: PrintConv/ValueConv implementations avoid code bloat
- **Trust ExifTool**: Every implementation cites ExifTool source line numbers

### Essential Reading

- [GETTING-STARTED.md](docs/GETTING-STARTED.md) - Start here for new contributors
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design and philosophy
- [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) - **Critical**: Our #1 development rule
- [MILESTONES.md](docs/MILESTONES.md) - Current development roadmap
- [SECURITY.md](SECURITY.md) - Security policy and vulnerability reporting

## Tag Compatibility

ExifTool tag names are preserved exactly. Group-prefixed output matches `exiftool -j -G`:

```json
{
  "EXIF:Make": "Canon",
  "EXIF:Model": "Canon EOS T3i",
  "GPS:GPSLatitude": 40.7589,
  "Composite:ShutterSpeed": "1/60"
}
```

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool, the definitive metadata extraction tool, for over 25 years
