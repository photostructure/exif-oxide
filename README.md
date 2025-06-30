# exif-oxide

A high-performance Rust implementation of portions of ExifTool, providing fast
metadata extraction from 26+ image and media file formats.

## Goals

- **Performance**: 10-20x faster than Perl ExifTool for common operations
- **Multi-Format**: Support for JPEG, RAW (CR2, NEF, ARW, DNG), PNG, HEIF, WebP, MP4, and more
- **Compatibility**: Maintain ExifTool tag naming and structure (with Rust identifier constraints)
- **Safety**: Memory-safe handling of untrusted files
- **Features**: Embedded image extraction, XMP support

## Why exif-oxide?

While ExifTool is the gold standard for metadata extraction, the Perl runtime
that it relies on can grind against Windows and macOS security systems.

By translating to rust, applications using exif-oxide can be a single, "normal"
binary application that can "play nicely" with Defender and Gatekeeper, as well
as providing substantive performance gains over an approach that `fork`s
`exiftool` as a child process.

## Design

See [ARCHITECTURE.md](ARCHITECTURE.md) for architectural details

## Licensing

This program and code is offered under a commercial and under the GNU Affero
General Public License. See [LICENSE](./LICENSE) for details.

## Development

This project includes a branch of `exiftool` as a submodule: use `git clone
--recursive`. This branch has a number of documents to bolster comprehension of
ExifTool, which has a **ton** of surprising bits going on.

The production code paths are mostly produced via automatic code generation that
reads from the ExifTool source directly. 

## Tag Naming Compatibility

ExifTool tag names are preserved when possible, but tags with invalid Rust
identifier characters are modified:

- **ExifTool**: `"NikonActiveD-Lighting"`
- **exif-oxide**: `"NikonActiveD_Lighting"`

Hyphens (and any other invalid characters) are converted to underscores to
ensure that they are valid Rust identifiers, while mostly preserving semantic
meaning.

## Acknowledgments

- Phil Harvey for creating and maintaining ExifTool for 25 years
