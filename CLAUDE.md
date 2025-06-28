# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with exif-oxide.

## Project Overview

exif-oxide is a high-performance Rust implementation of Phil Harvey's [ExifTool](https://exiftool.org/):

- **Performance**: 10-50x faster than Perl
- **Memory safety**: Safe for untrusted files
- **Key features**: Binary extraction (thumbnails/previews)

## Critical Development Principles

### 1. ExifTool is Gospel

- 25 years of camera-specific quirks and edge cases
- **How ExifTool handles it is the correct way** - no exceptions
- Maintain exact tag name and structure compatibility
- Do not invent any parsing heuristics. **ALWAYS** defer to ExifTool's algorithms, as verbatim as possible -- Chesterton's Fence applies here in a big way.
- Please always include a comment pointing back to the exiftool code for Engineers of Tomorrow to know where magic values came from.

**⚠️ CRITICAL**: Never attempt to "improve" or "simplify" ExifTool's logic:

- If ExifTool checks for `0x41` before `0x42`, do it in that order
- If ExifTool has a weird offset calculation, copy it exactly
- If ExifTool special-cases "NIKON CORPORATION" vs "NIKON", there's a reason
- No Camera Follows The Spec. Trust The ExifTool Code.

### 2. NEVER parse perl code in rust

Perl requires perl to understand perl. Any sort of regex "parser" is going to be brittle and fail.

If we need to import or sync code and tabular data from `third-party/exiftool`,
**only use perl** to do that extraction and translate it into something that's
easy for our rust sync code to ingest (like JSON)

## ⚠️ MANDATORY READING

Before implementing ANY feature or parsing logic:

1. **READ `doc/SYNC-DESIGN.md`** - This document explains:

   - How to track ExifTool source code references
   - The synchronization workflow with ExifTool updates
   - Algorithm extraction tools and processes
   - Required attribution patterns
   - New tasks may require additional sync features to be invented and implemented!

2. **CHECK ExifTool Implementation** - Never guess or invent:

   - Look in `third-party/exiftool/lib/Image/ExifTool/` for the canonical implementation
   - Use the exact same logic, quirks, and edge cases
   - If ExifTool does something weird, there's a reason (usually a camera quirk!)

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
├── print_conv.rs   # 🆕 TABLE-DRIVEN PrintConv system
├── types.rs        # EXIF format definitions
└── endian.rs       # Byte order handling

tables/
├── exif_tags.rs    # 🆕 COMPREHENSIVE EXIF tags with PrintConvId (643 tags)
├── pentax_tags.rs  # 🆕 Tag definitions with PrintConvId
├── canon_tags.rs   # 🆕 Canon maker note tags with PrintConvId
├── sony_tags.rs    # 🆕 Sony maker note tags with PrintConvId
├── olympus_tags.rs # 🆕 Olympus maker note tags with PrintConvId
├── dji_tags.rs     # 🆕 DJI drone tags with PrintConvId
└── [generated]/    # Generated from ExifTool (530+ tags)

maker/              # Manufacturer-specific parsing
├── pentax.rs       # 🆕 Table-driven parser (200 lines vs 6K Perl)
├── dji.rs          # 🆕 DJI drone parser with float conversions
└── [others]/       # Canon, Nikon, Sony, Fujifilm, etc.

binary.rs           # Direct tag-based extraction
xmp/                # Full hierarchical XML parsing
detection/          # 43 formats, sub-1ms performance
```

### Key Design Decisions

1. **Direct binary parsing** - No nom, transparent and efficient
2. **HashMap storage** - O(1) tag lookup by ID
3. **Table-driven PrintConv** - Revolutionary approach to value conversion
4. **Auto-generated sync** - Generated from ExifTool Perl
5. **Zero-copy binary** - Memory efficient extraction

## 🚀 PrintConv Architecture

**Problem**: ExifTool has ~50,000 lines of PrintConv code across all manufacturers. Manual porting would be a maintenance nightmare.

**Solution**: Table-driven PrintConv system with ~50 reusable conversion functions achieving **96% code reduction**.

**📖 Complete PrintConv Documentation**: 
See **[`doc/PRINTCONV-SYNC-20250625.md`](doc/PRINTCONV-SYNC-20250625.md)** for the definitive guide including:
- Table-driven architecture with 96% code reduction
- Smart synchronization system that prevents work loss
- Phase 0-3 completion status and achievements
- Critical sync clobbering fix (blocks current development)
- Implementation roadmap and universal pattern framework

## Development Workflow

1. **FIRST: Read `doc/SYNC-DESIGN.md`** - Understand the synchronization process and attribution requirements
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
# Extract ALL components in one command (recommended)
cargo run --bin exiftool_sync extract-all
# Or use the Makefile shorthand
make sync

# Extract specific components (all work with single commands)
cargo run --bin exiftool_sync extract binary-formats    # ProcessBinaryData tables
cargo run --bin exiftool_sync extract maker-detection   # Maker note signatures  
cargo run --bin exiftool_sync extract binary-tags       # Composite tag definitions
cargo run --bin exiftool_sync extract magic-numbers     # File type detection
cargo run --bin exiftool_sync extract datetime-patterns # Date parsing patterns

# Check synchronization status
cargo run --bin exiftool_sync status
cargo run --bin exiftool_sync scan                      # List all source dependencies

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

- [ ] Read `doc/SYNC-DESIGN.md` completely
- [ ] Find the ExifTool implementation in `third-party/exiftool/`
- [ ] Check for existing implementations: `grep -r "EXIFTOOL-SOURCE" src/`
- [ ] Use automated extraction tools when possible:
  ```bash
  # For ProcessBinaryData tables
  cargo run --bin exiftool_sync extract binary-formats
  
  # For maker note detection patterns  
  cargo run --bin exiftool_sync extract maker-detection
  
  # For composite tags (thumbnails, previews)
  cargo run --bin exiftool_sync extract binary-tags
  ```
- [ ] Add `EXIFTOOL-SOURCE` attribution to your file (or use auto-generated files)
- [ ] Write tests that validate against ExifTool output
- [ ] Never "improve" ExifTool's logic - copy it exactly  
- [ ] Document any non-obvious quirks with comments
- [ ] If you find yourself guessing, STOP and check ExifTool

## Major Synchronization Milestones Complete ✅

### Phase 0 Complete: Synchronization Infrastructure ✅

**As of June 2025, Phase 0 synchronization infrastructure is complete**:

- ✅ **Auto-generated maker note detection** for all 10 manufacturers
- ✅ **ProcessBinaryData table extraction** with 530+ tags
- ✅ **Composite tag definitions** for binary extraction
- ✅ **Smooth regeneration system** - no manual steps required
- ✅ **Build system integration** - always compiles, even with missing files
- ✅ **Test synchronization** - automated ExifTool output comparison
- ✅ **Extract-all command** - single command regenerates everything
- ✅ **Makefile integration** - `make sync` for convenience

### EXIF Migration Complete: Revolutionary Improvement ✅

**As of June 2025, EXIF migration is complete - 87% coverage gap eliminated**:

- ✅ **28x improvement**: 643 EXIF tags extracted vs previous ~23
- ✅ **Comprehensive EXIF coverage**: All standard photography tags now available
- ✅ **Table-driven architecture**: Following proven sync extractor pattern
- ✅ **Zero regressions**: All 123 tests passing with full backward compatibility
- ✅ **ExifTool synchronization**: Following `third-party/exiftool/lib/Image/ExifTool/Exif.pm` exactly
- ✅ **PrintConv integration**: EXIF-specific conversions (ExposureTime, FNumber, etc.)
- ✅ **Automatic string conversion**: Undefined EXIF data properly converted to strings

**Key Benefit**: Standard EXIF tags like Make, Model, ExposureTime, FNumber, ISO are now comprehensively extracted with ExifTool-compatible formatting.

**Usage**: Run `make sync` or `cargo run --bin exiftool_sync extract-all` to regenerate everything. EXIF tags are automatically included.

**Implementation**: See `src/bin/exiftool_sync/extractors/exif_tags.rs` for the EXIF sync extractor and `src/tables/exif_tags.rs` for the generated tag table.

**New Extractor Pattern**: See `src/bin/exiftool_sync/extractors/EXTRACTOR_PATTERN.md` for the required pattern that ensures all extractors work smoothly without manual intervention.


