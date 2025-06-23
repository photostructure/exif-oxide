# Contributing to exif-oxide

Thank you for your interest in contributing to exif-oxide! This guide will help you get started.

## Code of Conduct

Please be respectful and constructive in all interactions. We're building on 25+ years of Phil Harvey's ExifTool work and aim to maintain that spirit of collaboration.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/exif-oxide.git`
3. Initialize the ExifTool submodule: `git submodule update --init`
4. Create a feature branch: `git checkout -b feature/your-feature-name`
5. Make your changes
6. Run tests: `cargo test`
7. Submit a pull request

## ExifTool Attribution Requirements

**IMPORTANT**: exif-oxide leverages knowledge from Phil Harvey's ExifTool. Simple attribution is required for all ExifTool-derived implementations.

### How to Add Attribution

When implementing features from ExifTool, add doc attributes at the top of your Rust file:

```rust
//! Canon maker note parsing implementation

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/CanonRaw.pm"]

// rest of implementation...
```

For auto-generated files, the build system adds attribution automatically.

### Why This Approach?

1. **Simple** - Just doc attributes at the top of each file
2. **Greppable** - Easy to find all files affected by a Perl module change
3. **Self-documenting** - Shows up in rustdoc
4. **No separate files** - Attribution travels with the code

### Updating exiftool-sync.toml

After a sync with new ExifTool version:

1. Update the version number
2. Update last_sync date
3. Add to sync_history when fully incorporated

## Development Guidelines

### Code Style

- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow Rust naming conventions
- Write idiomatic Rust (not Perl-in-Rust)

### Error Handling

- Use `thiserror` for error types
- Provide context in error messages
- Handle malformed data gracefully
- Never panic in library code

### Performance

- Benchmark changes that might impact performance
- Use zero-copy operations where possible
- Avoid unnecessary allocations
- Profile before optimizing

### Testing

- Write tests for new functionality
- Test with ExifTool's test images when applicable
- Validate against ExifTool output
- Test error cases and malformed data

## Pull Request Process

1. **Update Documentation**
   - Add/update code comments
   - Update module EXIFTOOL_ATTRIBUTION.md if needed
   - Update README if adding features

2. **Ensure Tests Pass**
   - `cargo test`
   - `cargo test --features exiftool-compat` (if applicable)

3. **Update Attribution**
   - Add EXIFTOOL-SOURCE doc attributes for new files
   - Update exiftool-sync.toml version if syncing
   - Run `cargo run --bin exiftool_sync scan` to verify

4. **PR Description**
   - Describe what changed and why
   - Note any ExifTool features incorporated
   - List any breaking changes

## Sync with ExifTool Updates

To incorporate new ExifTool versions:

1. Check current status: `cargo run --bin exiftool_sync status`
2. Review changes: `cargo run --bin exiftool_sync diff 13.26 13.27`
3. See what's impacted - the tool will show which Rust files need updating
4. Update implementations as needed
5. Update version in exiftool-sync.toml
6. Regenerate auto-generated files: `cargo build`
7. Run tests to verify

## Questions?

- Check doc/EXIFTOOL-SYNC.md for the full synchronization guide
- Review existing code for examples
- Open an issue for clarification
- Discuss in PR comments

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).