# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with exif-oxide.

## Project Overview

exif-oxide is a high-performance Rust implementation of Phil Harvey's [ExifTool](https://exiftool.org/):

- **Performance**: 10-50x faster than Perl
- **Memory safety**: Safe for untrusted files
- **Key features**: Binary extraction (thumbnails/previews), DateTime intelligence from [exiftool-vendored](https://github.com/photostructure/exiftool-vendored.js)

## Critical Development Principles

### 1. ExifTool is Gospel

- 25 years of camera-specific quirks and edge cases
- **How ExifTool handles it is the correct way** - no exceptions
- Maintain exact tag name and structure compatibility
- Do not invent any parsing heuristics. **ALWAYS** defer to ExifTool's algorithms, as verbatim as possible -- Chesterton's Fence applies here in a big way.

**⚠️ CRITICAL**: Never attempt to "improve" or "simplify" ExifTool's logic:
- If ExifTool checks for `0x41` before `0x42`, do it in that order
- If ExifTool has a weird offset calculation, copy it exactly
- If ExifTool special-cases "NIKON CORPORATION" vs "NIKON", there's a reason
- That 2003 Coolpix bug? Yeah, ExifTool knows about it. Trust the code.

Example violations to avoid:
```rust
// ❌ WRONG - "simplifying" ExifTool's logic
if manufacturer.starts_with("NIKON") { ... }

// ✅ CORRECT - matching ExifTool exactly
if manufacturer == "NIKON" || manufacturer == "NIKON CORPORATION" { ... }
```

### 2. Current Status (December 2024)

- **✅ COMPLETE**: Core spikes 1-6 (EXIF, maker notes, binary extraction, XMP, detection, datetime)
- **🔄 CURRENT**: Phase 1 - Multi-format support
- **⚠️ LIMITATION**: main.rs hardcoded to JPEG despite 43 format detection capability

## ⚠️ MANDATORY READING

Before implementing ANY feature or parsing logic:

1. **READ `doc/EXIFTOOL-SYNC.md`** - This document explains:
   - How to track ExifTool source code references
   - The synchronization workflow with ExifTool updates
   - Algorithm extraction tools and processes
   - Required attribution patterns

2. **CHECK ExifTool Implementation** - Never guess or invent:
   - Look in `third-party/exiftool/lib/Image/ExifTool/` for the canonical implementation
   - Use the exact same logic, quirks, and edge cases
   - If ExifTool does something weird, there's a reason (usually a 2003 camera bug)

3. **USE Source Attribution** - Every file implementing ExifTool logic MUST have:
   ```rust
   #![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
   ```

## Tribal Knowledge & Gotchas

### Binary Parsing

- **Always** bounds-check before reading (use `get()` not indexing)
- Offsets are from TIFF header start, not file start
- Next IFD offset -1 (0xFFFFFFFF) means "no next"
- Value fits inline if size × count ≤ 4 bytes

### String Handling Quirks

- EXIF strings are null-terminated BUT buffer may contain garbage after null
- Some makers pad with spaces instead of nulls
- UTF-8 not guaranteed - may need charset detection

### JPEG Gotchas

- APP1 segments limited to 64KB
- Multiple APP1 segments possible (EXIF + XMP)
- Segment length **includes** the length bytes
- Must check for "Exif\0\0" signature

### DateTime Pitfalls

- **Never trust single sources** - cross-reference multiple fields
- GPS 0,0 means "unset" not "off the coast of Africa"
- Video dates can be UTC unless explicitly specified
- Nikon DST bug - some models incorrectly handle daylight saving

### Maker Note Warnings

- Often use relative offsets from maker note start
- Some manufacturers encrypt/obfuscate data
- Model-specific variations common
- May have checksums - preserve unknown data for write-back

## Implementation Architecture

### Completed Components

```
core/
├── jpeg.rs         # APP1 segment extraction (EXIF/XMP)
├── ifd.rs          # IFD parsing with endian support
├── types.rs        # EXIF format definitions
└── endian.rs       # Byte order handling

tables/             # Generated from ExifTool (530 tags)
maker/              # Manufacturer-specific parsing
binary.rs           # Direct tag-based extraction
xmp/                # Full hierarchical XML parsing
detection/          # 43 formats, sub-1ms performance
datetime/           # Multi-source intelligence, GPS timezone
```

### Key Design Decisions

1. **Direct binary parsing** - No nom, transparent and efficient
2. **HashMap storage** - O(1) tag lookup by ID
3. **Table-driven** - Auto-generated from ExifTool Perl
4. **Zero-copy binary** - Memory efficient extraction

## Development Workflow

1. **FIRST: Read `doc/EXIFTOOL-SYNC.md`** - Understand the synchronization process and attribution requirements
2. **Check ExifTool's implementation** in `third-party/exiftool/lib/Image/ExifTool/`
3. **Look for existing source attributions** - Check if the functionality is already tracked:
   ```bash
   grep -r "EXIFTOOL-SOURCE" src/
   ```
4. **Write tests first** - Validate against ExifTool output
5. **Add source attribution** to your implementation:
   ```rust
   #![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
   ```
6. **Use sync tools** when needed:
   ```bash
   # Check what changed in ExifTool
   cargo run --bin exiftool_sync diff 12.65 12.66
   
   # Extract algorithms
   cargo run --bin exiftool_sync extract magic-numbers
   ```
7. **Benchmark and validate** - Must match ExifTool's behavior exactly

## Essential Commands

```bash
# Basic operations
cargo build && cargo test
cargo test test_canon_image  # Specific test

# Extract metadata
cargo run -- test.jpg                           # All tags as JSON
cargo run -- -Make -Model test.jpg             # Specific tags
cargo run -- -b -ThumbnailImage test.jpg > thumb.jpg  # Binary extraction

# Debug & compare
./exiftool/exiftool -struct -json test.jpg > exiftool.json
cargo run -- test.jpg > exif-oxide.json

# Development tools
cargo bench                                     # Performance testing
cargo run --example debug_jpeg_segments test.jpg  # Debug segments
```

## ExifTool Reference Files

When implementing, check these files in `third-party/exiftool/`:

- `lib/Image/ExifTool/Exif.pm` - Core EXIF handling & composite tags
- `lib/Image/ExifTool/JPEG.pm` - JPEG segment parsing
- `lib/Image/ExifTool/[Manufacturer].pm` - Maker note implementations
- `lib/Image/ExifTool.pm` - ProcessBinaryData pattern
- `exiftool` - Main script (ConvertBinary at lines 3891-3920)
- `t/images/` - Test images with edge cases

Use the sync tools to extract algorithms:
```bash
# Extract specific components
cargo run --bin exiftool_sync extract magic-numbers
cargo run --bin exiftool_sync extract binary-formats
cargo run --bin exiftool_sync extract datetime-patterns
cargo run --bin exiftool_sync extract binary-tags

# Check for updates
cargo run --bin exiftool_sync diff 12.65 12.66
```

## Future Considerations

### Performance Optimizations

- SIMD for endian swapping
- Parallel IFD processing
- String interning for common values
- Memory-mapped files for large RAWs

### Write Support Challenges

- Must preserve unknown tags
- Maker notes often have checksums
- Some cameras require specific tag order
- Use temp file, not in-place updates

## Implementation Checklist

Before implementing ANY new feature:

- [ ] Read `doc/EXIFTOOL-SYNC.md` completely
- [ ] Find the ExifTool implementation in `third-party/exiftool/`
- [ ] Check for existing implementations: `grep -r "EXIFTOOL-SOURCE" src/`
- [ ] If extracting algorithms, use the sync tools:
  ```bash
  cargo run --bin exiftool_sync extract [component]
  ```
- [ ] Add `EXIFTOOL-SOURCE` attribution to your file
- [ ] Write tests that validate against ExifTool output
- [ ] Never "improve" ExifTool's logic - copy it exactly
- [ ] Document any non-obvious quirks with comments
- [ ] If you find yourself guessing, STOP and check ExifTool

## Remember

- The user is new to Rust - write idiomatic code and explain patterns
- What seems like a bug might be a workaround for a 2003 camera model
- **Always read `doc/EXIFTOOL-SYNC.md` before implementing**
- The sync tools exist to help - use them!

**This project stands on giants' shoulders - respect 25 years of accumulated camera quirks.**
