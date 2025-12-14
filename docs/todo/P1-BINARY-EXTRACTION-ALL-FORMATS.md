# TPP: P1 - Binary Extraction for All Image Formats

## Part 1: Define Success

**Problem**: Binary extraction (`-b -TagName`) only works for JPEG files. TIFF, RAW formats (NEF, CR2, ARW, DNG, ORF, RAF, etc.) have embedded previews and thumbnails that cannot be extracted because:
1. Base offset tracking only implemented for JPEG APP1 segments
2. No format-specific processing for TIFF-based formats
3. No support for RAW-specific embedded image tags

**Why it matters**: Photographers with RAW workflows cannot extract embedded previews from their camera files - a core feature for DAM (Digital Asset Management) applications.

**Success test**:
```bash
# TIFF file thumbnail extraction
target/release/exif-oxide -b -ThumbnailImage test-images/canon/Canon_40D.tiff > /tmp/thumb.jpg
file /tmp/thumb.jpg  # Should say "JPEG image data"

# NEF (Nikon RAW) preview extraction
target/release/exif-oxide -b -PreviewImage test-images/nikon/DSC_0001.NEF > /tmp/preview.jpg
diff /tmp/preview.jpg <(exiftool -b -PreviewImage test-images/nikon/DSC_0001.NEF) && echo "SUCCESS"

# CR2 (Canon RAW) embedded JPEG extraction
target/release/exif-oxide -b -JpgFromRaw test-images/canon/Canon_EOS_5D.CR2 > /tmp/jpg.jpg
file /tmp/jpg.jpg  # Should say "JPEG image data"
```

**Key constraint**: Must match ExifTool's offset handling exactly - different formats have different base offset rules.

## Part 2: Share Your Expertise

### A. Current State (What Works)

JPEG IsOffset handling is complete (P0-IFD1-THUMBNAIL-EXTRACTION):

```bash
# Verify JPEG works
cargo run -- test-images/apple/IMG_3755.JPG | grep ThumbnailOffset
# "EXIF:ThumbnailOffset": 3048  (absolute file offset)

# Binary extraction produces valid JPEG
dd if=test-images/apple/IMG_3755.JPG bs=1 skip=3048 count=8106 | file -
# JPEG image data
```

Key implementation files:
- `src/exif/tags.rs:88-156` - `apply_is_offset_adjustment()` applies base offset
- `src/formats/mod.rs:482-487` - JPEG sets base offset before EXIF parsing
- `src/types/tag_info.rs:22-27` - `is_offset: bool` field in TagInfo
- `codegen/src/strategies/tag_kit.rs:802-823` - Extracts IsOffset from ExifTool

### B. The Challenge: Format-Specific Base Offsets

**ExifTool's Base Offset Rules** (from `OFFSET_BASE_MANAGEMENT.md`):

| Format | Base Offset | Notes |
|--------|-------------|-------|
| JPEG | APP1 segment start | Position of TIFF header in file |
| TIFF | 0 | Offsets already absolute |
| NEF/NRW | SubIFD base | Complex - multiple IFD chains |
| CR2 | IFD start | Canon-specific offset handling |
| ARW | 0 or segment | Sony varies by model |
| ORF | MakerNotes base | Olympus uses relative offsets |
| DNG | 0 | Standard TIFF behavior |

**Critical Reference**: ExifTool Exif.pm:7052-7066 shows IsOffset processing:
```perl
if ($$tagInfo{IsOffset} and $$tagInfo{IsOffset} ne '3') {
    my $et = $$dirInfo{Parent};
    if ($et and $$et{TIFF_TYPE} ne 'CR2') {
        # Add base to offset
        $val += $$dirInfo{Base};
    }
}
```

### C. IsOffset Tags Across Formats

```bash
# Find all tags with IsOffset in ExifTool
grep -r "IsOffset\s*=>" third-party/exiftool/lib/Image/ExifTool/*.pm | head -20
```

Key IsOffset tags (all need base adjustment):
- `StripOffsets` (0x111) - Image data location
- `TileOffsets` (0x144) - Tiled image data
- `ThumbnailOffset` (0x201) - IFD1 thumbnail
- `PreviewImageStart` (0x201 in SubIFD) - Preview image
- `JpgFromRawStart` (0x201 variant) - Embedded full JPEG
- `OtherImageStart` (0x201 variant) - Additional images

### D. Landmines to Avoid

1. **CR2 Exception**: Canon CR2 files do NOT apply base offset adjustment for some tags
   - ExifTool: `$$et{TIFF_TYPE} ne 'CR2'` check in Exif.pm:7055
   - Must detect CR2 format and skip adjustment

2. **MakerNotes Relative Offsets**: Some manufacturers use offsets relative to MakerNotes start
   - Already have `maker_notes_original_offset` in ExifReader
   - May need to use this instead of global base for some tags

3. **SubIFD Chain**: NEF files have complex SubIFD chains
   - Each SubIFD may have its own base
   - ExifTool tracks `$firstBase` vs current `$base`

4. **IsOffset Modes** (ExifTool supports multiple):
   - `IsOffset => 1`: Use directory `$base` (standard)
   - `IsOffset => '2'`: Use `$firstBase` (parent directory base)
   - `IsOffset => 3`: Already absolute, no adjustment
   - `IsOffset => '$val > 0'`: Conditional (evaluate expression)

### E. Format Detection

Current format detection (`src/formats/mod.rs`):
```bash
grep -n "file_type\|FileType" src/formats/mod.rs | head -20
```

The `ExifReader.original_file_type` field already tracks detected format (NEF, CR2, etc.).

## Part 3: Tasks

### Task 1: TIFF Base Offset (No Change Needed)
**Status**: ✅ Already Works

TIFF files have base=0, offsets are already absolute:
```bash
# Verify TIFF base is 0
cargo run -- test-images/canon/Canon_40D.tiff | grep -i "offset"
```

**Verification**: TIFF files should work with current implementation since `base` defaults to 0.

### Task 2: Add CR2 Format Exception
**Status**: Not Started

**Problem**: Canon CR2 files should NOT apply IsOffset adjustment for certain tags.

**Success**: CR2 PreviewImage extracts correctly
```bash
target/release/exif-oxide -b -PreviewImage test-images/canon/Canon_EOS_5D.CR2 > /tmp/preview.jpg
diff /tmp/preview.jpg <(exiftool -b -PreviewImage test-images/canon/Canon_EOS_5D.CR2) && echo "SUCCESS"
```

**Implementation**:
1. Check `self.original_file_type` in `apply_is_offset_adjustment()`
2. Skip adjustment for CR2 files (match ExifTool behavior)
3. Add test case for CR2 extraction

**Files to modify**:
- `src/exif/tags.rs:98-100` - Add CR2 check after base==0 check

**ExifTool Reference**: Exif.pm:7055 - `$$et{TIFF_TYPE} ne 'CR2'`

### Task 3: NEF/NRW SubIFD Base Tracking
**Status**: Not Started

**Problem**: Nikon NEF files have SubIFDs with their own base offsets.

**Success**: NEF PreviewImage extracts correctly
```bash
target/release/exif-oxide -b -PreviewImage test-images/nikon/DSC_0001.NEF > /tmp/preview.jpg
diff /tmp/preview.jpg <(exiftool -b -PreviewImage test-images/nikon/DSC_0001.NEF) && echo "SUCCESS"
```

**Implementation**:
1. Track SubIFD base offsets during parsing
2. Use SubIFD-specific base for IsOffset tags within SubIFDs
3. Add Nikon-specific test cases

**Investigation needed**:
```bash
# Check how ExifTool handles NEF SubIFDs
exiftool -v5 test-images/nikon/DSC_0001.NEF | grep -i "base\|subifd"
```

**Files to modify**:
- `src/exif/ifd.rs` - Track SubIFD base during parsing
- `src/exif/processors.rs` - Pass SubIFD base to tag storage

### Task 4: ORF (Olympus) MakerNotes Relative Offsets
**Status**: Not Started

**Problem**: Olympus ORF files use offsets relative to MakerNotes start.

**Success**: ORF embedded images extract correctly
```bash
target/release/exif-oxide -b -ThumbnailImage test-images/olympus/*.ORF > /tmp/thumb.jpg
file /tmp/thumb.jpg  # Should say "JPEG image data"
```

**Implementation**:
1. Use `maker_notes_original_offset` for MakerNotes-relative offsets
2. Detect when tag is within MakerNotes context
3. Apply MakerNotes base instead of global base

**Files to modify**:
- `src/exif/tags.rs` - Check if source is MakerNotes context
- Use existing `self.maker_notes_original_offset`

### Task 5: Sony ARW Handling
**Status**: Not Started

**Problem**: Sony ARW files have variable offset handling by model.

**Investigation needed**:
```bash
# Check ExifTool Sony handling
grep -A20 "IsOffset" third-party/exiftool/lib/Image/ExifTool/Sony.pm | head -30
```

**Implementation**: Requires investigation of Sony-specific patterns.

### Task 6: Binary Extraction CLI Integration
**Status**: Partially Done

Current `-b` flag exists but may not work for all formats.

**Success**: `-b -ThumbnailImage` works for all formats with embedded thumbnails
```bash
for img in test-images/**/*.{jpg,tiff,nef,cr2,arw,orf}; do
    echo "Testing: $img"
    target/release/exif-oxide -b -ThumbnailImage "$img" > /tmp/thumb.jpg 2>/dev/null
    file /tmp/thumb.jpg
done
```

**Files to verify**:
- `src/main.rs` - Binary extraction handling
- Check `extract_binary_data()` function

### Task 7: Add Integration Tests
**Status**: Not Started

**Success**: All format tests pass
```bash
cargo t --test binary_extraction_test
```

**Implementation**:
1. Create `tests/binary_extraction_test.rs`
2. Add test cases for each format: JPEG, TIFF, NEF, CR2, ARW, ORF, DNG
3. Compare extracted data with ExifTool output

**Test structure**:
```rust
#[test]
fn test_jpeg_thumbnail_extraction() { ... }

#[test]
fn test_tiff_thumbnail_extraction() { ... }

#[test]
fn test_nef_preview_extraction() { ... }

#[test]
fn test_cr2_preview_extraction() { ... }
```

## Handoff Context

### Current State Summary

| Format | IsOffset Works | Binary Extraction | Notes |
|--------|---------------|-------------------|-------|
| JPEG | ✅ Yes | ✅ Yes | P0 complete |
| TIFF | ✅ Yes (base=0) | ❓ Untested | Should work |
| NEF | ❌ No | ❌ No | Needs SubIFD base |
| CR2 | ❌ No | ❌ No | Needs exception |
| ARW | ❌ Unknown | ❌ Unknown | Investigation needed |
| ORF | ❌ No | ❌ No | Needs MakerNotes base |
| DNG | ✅ Yes (base=0) | ❓ Untested | Should work |

### Architecture Adaptation

If the IsOffset implementation changes:
1. Core need unchanged: Raw offset → absolute file position
2. Search for offset handling: `rg "is_offset|base.*offset" src/`
3. Goal remains: Extract embedded binary data at correct file position

## Quality Checklist

- [x] Problem and success criteria fit in one paragraph
- [x] Included actual commands that find relevant code
- [x] Documented format-specific "landmines" (CR2 exception, SubIFD bases)
- [x] Each task has a verifiable success command
- [x] Explained how to adapt if architecture changed
- [x] ExifTool source references for each format
- [ ] Bug fix starts with failing test (Task 7 creates tests)

## Priority Order

1. **Task 7** (Tests) - Write failing tests first per TDD.md
2. **Task 2** (CR2) - Common RAW format, simple fix
3. **Task 3** (NEF) - Complex but well-documented
4. **Task 4** (ORF) - Uses existing `maker_notes_original_offset`
5. **Task 5** (ARW) - Requires investigation
6. **Task 6** (CLI) - Polish after formats work
