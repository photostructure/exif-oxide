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
- Achieve comparable or better performance, especially on Windows where file I/O is expensive
- Support the most commonly used metadata formats and manufacturers

## Project Non-Goals

- We will not port over the `geolocation` functionality
- We will not port over the "custom" configuration support that `exiftool` supports
- No filesystem recursion: we will not support batch-update operations via the CLI
- We will not support tag value updates that are not static values (no pattern-match replacements)

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

### Key Design Decisions

1. **Hybrid Approach**: Generate static tag tables at compile time, manually implement complex logic
2. **Minimal Perl**: Use Perl only to extract tag definitions to JSON, then process with Rust
3. **Implementation Library**: Manual Rust implementations indexed by Perl snippet signatures
4. **Always Compilable**: Codegen produces working code even with missing implementations
5. **Incremental Scope**: Start with basic JPEG/EXIF, then one manufacturer at a time

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
