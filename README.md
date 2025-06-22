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

## Development

Currently working on [Spike 1](SPIKES.md#spike-1-basic-exif-tag-reading-make-model-orientation): Basic EXIF tag reading.

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

## License

This project is licensed under the same terms as ExifTool:
- Free for personal, educational, and non-profit use
- Commercial use requires licensing

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
- The exiftool-vendored project for datetime parsing heuristics
- The Rust community for excellent binary parsing libraries