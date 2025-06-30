# exif-oxide: A translation of ExifTool to Rust

This directory contains the exif-oxide project, which ports ExifTool's metadata extraction functionality to Rust using a hybrid approach of code generation and manual implementations.

## Why exif-oxide?

While ExifTool is the gold standard for metadata extraction, the Perl runtime
that it relies on can grind against Windows and macOS security systems.

By translating to rust, applications using exif-oxide can be a single, "normal"
binary application that can "play nicely" with Defender and Gatekeeper, as well
as providing substantive performance gains over an approach that `fork`s
`exiftool` as a child process.

## Project Goals

- Leverage ExifTool's invaluable camera-specific quirks and edge cases, and not introduce any novel parsing or heuristics--ExifTool has figured everything out already.
- Create a Rust library with behavioral compatibility with ExifTool for reading metadata
- Use code generation for static tag definitions while manually implementing complex logic
- Maintain the ability to track ExifTool's monthly updates through regeneration
- Provide streaming I/O to handle large files efficiently without loading them into memory
- Support mainstream metadata tags (>80% frequency) for the most common formats and manufacturers

## Project Non-Goals

- We will not provide 100% tag compatibility for reading nor writing. We focus on mainstream tags (frequency >80% in TagMetadata.json or "mainstream: true"), translating approximately 500-1000 tags out of ExifTool's 15,000+.
- We will not port over the `geolocation` functionality
- We will not port over ExifTool's custom tag definitions and user-defined tags
- No filesystem recursion: we will not support batch-update operations via the CLI
- We will not support tag value updates that are not static values (no pattern-match replacements)

## Design

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for architectural details.

For engineers starting on the project, see [ENGINEER-GUIDE.md](docs/ENGINEER-GUIDE.md) for practical guidance on understanding ExifTool and implementing features.

## Licensing

This program and code is offered under a commercial and under the GNU Affero
General Public License. See [LICENSE](./LICENSE) for details.

## Development

This project includes a branch of `exiftool` as a submodule: use `git clone
--recursive`. This branch has a number of documents to bolster comprehension of
ExifTool, which has a **ton** of surprising bits going on.

The production code paths are mostly produced via automatic code generation that
reads from the ExifTool source directly.

### Quick Start

1. Clone with submodules: `git clone --recursive https://github.com/yourusername/exif-oxide`
2. Read [ENGINEER-GUIDE.md](docs/ENGINEER-GUIDE.md) to understand ExifTool concepts
3. Check [MILESTONES.md](docs/MILESTONES.md) for the development roadmap
4. Run codegen: `cargo run -p codegen`
5. Test with: `cargo test`

### Expected CLI (coming soon)

```sh
$ exif-oxide --help

Extracts EXIF data from image files

Usage: exif-oxide [OPTIONS] [ARGS]...

Arguments:
  [ARGS]...  All remaining arguments (files and -TagName patterns)

Options:
  -G, --groups   Include group names in output
  -n, --numeric  Show numeric/raw values (no PrintConv)
  -b, --binary   Extract raw binary data (requires single tag and single file)
      --api      API mode: show both raw and formatted values with full type information
  -h, --help     Show help information

EXAMPLES:
  exif-oxide photo.jpg                          # All tags as JSON
  exif-oxide photo1.jpg photo2.jpg              # Multiple files
  exif-oxide -G photo.jpg                       # All tags with groups
  exif-oxide -Make -Model photo.jpg             # Only return Make and Model tag values
  exif-oxide -b -ThumbnailImage photo.jpg > thumb.jpg  # Save thumbnail
```

### Expected Usage (Future API)

```rust
use exif_oxide::ExifReader;
use std::fs::File;

let mut reader = ExifReader::new();
let file = File::open("photo.jpg")?;

// Extract all mainstream metadata
let metadata = reader.read_metadata_stream(file, Default::default())?;
println!("Camera: {} {}", metadata.tags["Make"], metadata.tags["Model"]);

// Stream large binary data (thumbnails) without loading into memory
if let Some(thumbnail_ref) = metadata.get_binary_ref("ThumbnailImage") {
    let file = File::open("photo.jpg")?;
    let mut thumbnail_reader = reader.stream_binary_tag(file, thumbnail_ref)?;
    let mut thumbnail_file = File::create("thumbnail.jpg")?;
    std::io::copy(&mut thumbnail_reader, &mut thumbnail_file)?;
}
```

### Key Design Decisions

1. **Hybrid Approach**: Generate static tag tables at compile time, manually implement complex logic
2. **Minimal Perl**: Use Perl only to extract tag definitions to JSON, then process with Rust
3. **Implementation Library**: Manual Rust implementations indexed by Perl snippet signatures
4. **Always Compilable**: Codegen produces working code even with missing implementations
5. **Incremental Scope**: Start with basic JPEG/EXIF, then one manufacturer at a time
6. **Streaming First**: Handle large files efficiently without requiring full file loads

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
