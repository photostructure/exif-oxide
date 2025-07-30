# P16: Complete Binary Data Extraction (`-b` flag support)

## Project Overview

- **Goal**: Implement complete ExifTool-compatible binary data extraction for all mainstream binary tags, enabling users to extract embedded images with byte-identical output using `exif-oxide -b TagName file.ext > output.jpg`
- **Problem**: Binary extraction is partially implemented but has critical gaps - wrong data extraction, missing tag types, and format-specific naming mismatches cause user-facing failures
- **Constraints**: Must maintain byte-identical compatibility with ExifTool `-b` flag across all supported formats and maintain memory efficiency for large preview images (500KB+)

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Current State Analysis (2025-07-30)

### ✅ What's Working
- **CLI `-b` flag**: Fully implemented in `src/main.rs:200-503` with streaming I/O
- **Binary indicators**: Show correctly in regular output ("Binary data X bytes, use -b option to extract")
- **Sony ARW PreviewImage**: Perfect extraction - 508,756 bytes, SHA256-verified byte-identical to ExifTool
- **Basic infrastructure**: Binary extraction pipeline, composite tag generation, memory-efficient streaming

### ❌ Critical Failures
1. **Wrong data extraction**: `-b -ThumbnailImage` on Sony ARW returns PreviewImage data (508,756 bytes instead of 10,857)
2. **Unsupported binary tags**: `-b -JpgFromRaw` fails completely ("Binary extraction not supported for tag: JpgFromRaw")
3. **Format-specific naming mismatches**: Canon CR2 uses `JpgFromRawStart/Length` but binary extraction looks for `PreviewImageStart/Length`
4. **JPEG thumbnail extraction**: Regular JPEG files fail ("Required offset/length tags not found for: ThumbnailImage")

### 🔍 Root Cause: Incomplete Binary Tag Mapping

**Current implementation** (`src/main.rs:402-428`) only handles 3 binary tag types:
- `thumbnailimage` → tries `ThumbnailOffset/Length` OR `OtherImageStart/Length`
- `previewimage` → tries `PreviewImageStart/Length` OR `OtherImageStart/Length`  
- `otherimage` → uses `OtherImageStart/Length`

**ExifTool reality**: 13+ offset/length patterns, format-specific naming, composite tag construction, and special JPEG handling.

## Technical Foundation

### ExifTool Binary Extraction Architecture

**Core Pattern** (`lib/Image/ExifTool.pm:9716-9749`):
1. **Dual-mode behavior**: Returns placeholder text OR actual binary data based on `-b` flag
2. **Offset pair mapping**: 13+ predefined offset/length relationships in `HtmlDump.pm`
3. **Format-specific naming**: Same binary data uses different tag names per manufacturer
4. **Composite construction**: Binary tags generated on-demand, not stored directly

**Key Mappings** (from `HtmlDump.pm`):
```perl
my %offsetPair = (
    ThumbnailOffset   => 'ThumbnailLength',
    PreviewImageStart => 'PreviewImageLength', 
    JpgFromRawStart   => 'JpgFromRawLength',
    OtherImageStart   => 'OtherImageLength',
    PreviewJXLStart   => 'PreviewJXLLength',
    IDCPreviewStart   => 'IDCPreviewLength',
    # ... 8 more patterns
);
```

### Binary Tags Requiring Implementation

Based on user requirements and ExifTool analysis:

**High Priority** (Required for mainstream workflows):
- `Composite:PreviewImage` ✅ **WORKING (Sony ARW only)**
- `EXIF:JpgFromRaw` ❌ **MISSING** 
- `EXIF:PreviewImage` ❌ **BROKEN (Canon CR2)**
- `EXIF:ThumbnailImage` ❌ **WRONG DATA**
- `EXIF:ThumbnailTIFF` ❌ **MISSING**
- `JFIF:ThumbnailImage` ❌ **MISSING (special JFIF handling)**
- `MakerNotes:PreviewImage` ❌ **MISSING**
- `MakerNotes:ThumbnailImage` ❌ **MISSING**

**Medium Priority** (Format-specific):
- `File:PreviewImage` ❌ **MISSING**
- `FlashPix:PreviewImage` ❌ **MISSING** 
- `MPF:PreviewImage` ❌ **MISSING (Multi-Picture Format)**
- `PanasonicRaw:JpgFromRaw2` ❌ **MISSING**
- `Preview:JpgFromRaw2` ❌ **MISSING**

**Advanced** (Future consideration):
- `PreviewJXL`, `IDCPreview`, `SamsungRawPointers` - New binary types not in original user list

## Implementation Plan

### Phase 1: Core Binary Tag Mapping (P16a)

**Goal**: Fix wrong data extraction and implement complete offset/length pattern mapping

**Tasks**:
1. **Implement comprehensive offset pair mapping** - Port ExifTool's 13+ offset/length patterns
2. **Fix Sony ARW thumbnail/preview discrimination** - Multiple binary images need proper differentiation  
3. **Add Canon format-specific mapping** - `JpgFromRawStart/Length` → `JpgFromRaw` binary tag
4. **Implement JPEG/JFIF thumbnail extraction** - Special case using `MakeTiffHeader()` equivalent
5. **Add missing binary tag types** - Extend beyond current 3 to support all mainstream tags

**Key Files**:
- `src/main.rs:extract_binary_data()` - Binary extraction engine
- `src/composite_tags/implementations.rs` - Binary indicator generation 
- New: `src/binary_extraction/` - Format-specific binary handlers

### Phase 2: Format-Specific Binary Handlers (P16b)

**Goal**: Implement manufacturer-specific binary extraction patterns

**Tasks**:
1. **Canon binary extraction** - CR2/CR3 preview and thumbnail handling
2. **Nikon binary extraction** - NEF format-specific patterns
3. **Olympus binary extraction** - ORF multiple preview sizes
4. **Panasonic binary extraction** - RW2 and embedded JPEG processing
5. **Multi-Picture Format support** - MPF binary image extraction

**Prerequisites**: P16a complete (core mapping infrastructure)

### Phase 3: Advanced Binary Features (P16c)

**Goal**: Handle edge cases and optimize for real-world usage

**Tasks**:
1. **Error handling and validation** - Corrupted data, missing offsets, format validation
2. **Memory optimization** - Large preview streaming (tested with 500KB+ files)
3. **Video format support** - QuickTime/MP4 preview frame extraction
4. **Comprehensive testing** - All binary tags across all supported formats

**Prerequisites**: P16b complete (format handlers working)

## Success Criteria

**Definition of Done**:
- [ ] All 15 binary tags from user requirements support `-b` extraction
- [ ] Byte-identical output compared to ExifTool (SHA256 verified)
- [ ] Binary indicators appear correctly in regular metadata output
- [ ] Memory usage remains constant regardless of binary data size (streaming)
- [ ] `make binary-compat-test` passes with >95% success rate across test corpus

**Quality Gates**:
- Sony ARW: Both ThumbnailImage (10,857 bytes) AND PreviewImage (508,756 bytes) extract correctly
- Canon CR2: JpgFromRaw extraction works using `JpgFromRawStart/Length` tags
- JPEG files: ThumbnailImage extraction works using JFIF embedded thumbnail construction
- All manufacturers: Format-specific binary tags extract byte-identical to ExifTool

## Dependencies

- **P10a EXIF Foundation**: Binary tags often in EXIF IFDs ✅ **COMPLETE**
- **P13 MakerNotes**: Manufacturer-specific binary tags ⚠️ **IN PROGRESS**
- **Tag Kit System**: Binary tag definitions and composite generation ✅ **COMPLETE**

## Testing Strategy

**Existing Infrastructure**:
- `tests/binary_extraction_comprehensive.rs` - Multi-format binary extraction validation ✅ **EXISTS**
- `make binary-compat-test` - Comprehensive test suite ✅ **EXISTS** 
- Test image corpus: 379 files across all supported formats ✅ **EXISTS**

**Test Plan**:
1. **Unit tests**: Offset pair mapping logic, format-specific handlers
2. **Integration tests**: Full pipeline with real image files (extend existing comprehensive test)
3. **Compatibility validation**: SHA256 comparison with ExifTool output for each binary tag type
4. **Performance testing**: Memory usage profiling with large preview images

## Risk Assessment

**High Risk**:
- **Format-specific complexity**: Each manufacturer has subtle differences in binary storage
- **Memory management**: Large binary data (4MB+ JpgFromRaw) requires careful streaming
- **Tag naming conflicts**: Same binary data, different names across formats

**Mitigation**:
- **Trust ExifTool principle**: Translate exact logic, don't "improve" or optimize
- **Incremental testing**: Validate each format individually before moving to next
- **Comprehensive test coverage**: Use existing 379-file test corpus for validation

## Context and Gotchas

### Format-Specific Binary Storage Patterns

**Sony ARW**: Uses `OtherImageStart/Length` for BOTH thumbnail AND preview data - differentiation requires additional logic (thumbnail typically <50KB, preview >500KB)

**Canon CR2**: Uses `JpgFromRawStart/Length` for preview data, but our current mapping looks for `PreviewImageStart/Length`

**JPEG/JFIF**: Thumbnails are embedded in JFIF APP segments, not offset/length pairs - requires special construction using TIFF header generation

### Multiple Binary Images Per File

Many RAW files contain:
- Small thumbnail (typically 160x120, <50KB) for quick display
- Large preview (typically 1600x1080, >500KB) for editing preview
- Full-resolution JPEG (4MB+) for immediate processing

Each needs separate extraction logic despite similar underlying storage.

### ExifTool Compatibility Requirements

ExifTool's `-b` flag has specific behaviors:
- Output format: Raw bytes to stdout (no base64 encoding)
- Error handling: Silent failure for missing tags (no error message)
- Memory efficiency: Streaming for large data (never loads full binary into memory)
- Validation: JPEG magic number checking, format validation

## Status Updates

**2025-07-30**: Initial analysis complete. Binary extraction partially working but needs comprehensive rewrite for complete ExifTool compatibility. Core infrastructure exists but binary tag mapping system needs complete overhaul.