# TPP: ImageDataHash Support

## Part 1: Define Success

**Problem**: Users cannot verify image content integrity independent of metadata changes - no way to detect if the actual pixel/media data was modified while ignoring metadata edits.

**Why it matters**: Content verification is critical for digital forensics, asset management systems, and detecting unauthorized image manipulation. ExifTool provides this via API options, and users migrating to exif-oxide need feature parity.

**Solution**: Implement ImageDataHash computation that hashes only the actual image data (scan data for JPEG, IDAT chunks for PNG, strips/tiles for TIFF), excluding all metadata.

**ExifTool CLI API** (reference: https://exiftool.org/forum/index.php?topic=14706.msg79218):

```bash
# ExifTool uses -api options, NOT a direct -ImageDataHash flag:
exiftool -api requesttags=imagedatahash -api imagehashtype=MD5 image.jpg
exiftool -api requesttags=imagedatahash -api imagehashtype=SHA256 image.jpg
```

**Our CLI API** (simplified for usability):

```bash
# We'll support both the ExifTool-compatible API and a simpler flag
./exif-oxide --image-hash image.jpg                    # MD5 (default)
./exif-oxide --image-hash --image-hash-type=sha256 image.jpg
./exif-oxide -api requesttags=imagedatahash image.jpg  # ExifTool compat
```

**Success test**:

```bash
# Build and test against ExifTool
cargo build --release

# JPEG test - compare hash values
./target/release/exif-oxide --image-hash test-images/apple/IMG_3755.JPG
exiftool -api requesttags=imagedatahash test-images/apple/IMG_3755.JPG
# Both should output identical Composite:ImageDataHash value

# PNG test
./target/release/exif-oxide --image-hash test-images/png/exif-rgb-thumbnail.png
exiftool -api requesttags=imagedatahash test-images/png/exif-rgb-thumbnail.png
# Both should output identical hash

# SHA256 test
./target/release/exif-oxide --image-hash --image-hash-type=sha256 test-images/apple/IMG_3755.JPG
exiftool -api requesttags=imagedatahash -api imagehashtype=SHA256 test-images/apple/IMG_3755.JPG
# Both should output identical 64-char hex hash

# Verify metadata-only changes don't affect hash
exiftool -Comment="test" -o /tmp/modified.jpg test-images/apple/IMG_3755.JPG
./target/release/exif-oxide --image-hash /tmp/modified.jpg
# Hash should match original
```

**Key constraint**: Hash values MUST match ExifTool exactly - same bytes hashed in same order for each format.

## Part 2: Share Your Expertise

### A. ExifTool Architecture (Critical Reading)

**Primary Reference**: `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md`

**Key Insight**: ImageDataHash is NOT a composite tag calculated from other tags - it's computed during file parsing by accumulating hashes of image data sections as they're encountered.

**Hash Object Lifecycle** (ExifTool.pm lines 2766-2780, 4378-4386):

```perl
# Creation - when ImageDataHash is requested:
if ($$req{imagedatahash} and not $$self{ImageDataHash}) {
    my $imageHashType = $self->Options('ImageHashType');
    if ($imageHashType =~ /^SHA(256|512)$/i) {
        $$self{ImageDataHash} = Digest::SHA->new($1);
    } else {
        $$self{ImageDataHash} = Digest::MD5->new;
    }
}

# Finalization - in DoneExtract():
if ($$self{ImageDataHash}) {
    my $digest = $$self{ImageDataHash}->hexdigest;
    # Skip empty digests (no image data found)
    $self->FoundTag(ImageDataHash => $digest) unless $digest eq $emptyMD5;
}
```

### B. Format-Specific Hashing Logic

**JPEG** (ExifTool.pm lines 7217-7406):

- Hash SOS (Start of Scan) marker 0xDA and all data until EOI
- Include RST markers 0xD0-0xD7
- Include stuffed bytes 0xFF00
- **Exclude**: All APP segments, COM, DQT, DHT, SOF

```perl
if ($hash and defined $marker and ($marker == 0x00 or $marker == 0xda or
    ($marker >= 0xd0 and $marker <= 0xd7)))
{
    $hash->add("\xff" . chr($marker));
    $hash->add($buff);
}
```

**PNG** (PNG.pm lines 1419-1593):

- Hash IDAT chunks (compressed image data)
- Hash JDAT chunks (JNG JPEG data)
- Hash fdAT chunks (APNG frame data)
- **Exclude**: All text chunks, ancillary chunks, IHDR

```perl
my %isDataChunk = (IDAT=>1, JDAT=>1, fdAT=>1);
if ($isDataChunk{$chunk} and $hash) {
    $et->ImageDataHash($raf, $len);
}
```

**TIFF/RAW** (Exif.pm lines 6200-7094):

- Hash data pointed to by tags with `IsImageData => 1`
- Key tags: StripOffsets (0x111), TileOffsets (0x144), JpgFromRawStart (0x201)
- **Exclude**: ThumbnailImage, PreviewImage, maker note embedded images

### C. Current exif-oxide Architecture Integration Points

**JPEG Processing** (`src/formats/jpeg.rs`):

- `scan_jpeg_segments()` already iterates through segments
- Need to add hash accumulation for SOS and subsequent scan data
- Currently stops at SOS marker - need to continue reading for hash

**PNG Processing** (`src/formats/png.rs`):

- `parse_png_ihdr()` only reads IHDR
- Need new function to iterate all chunks and hash IDAT data

**TIFF Processing** (`src/formats/tiff.rs`):

- `extract_tiff_exif()` reads entire file
- Need to identify StripOffsets/TileOffsets and hash referenced data

**ExifReader State** (`src/exif/mod.rs:36-82`):

- Add `image_data_hash: Option<Box<dyn Digest>>` field
- Add `image_hash_type: ImageHashType` option

**FilterOptions** (`src/types/metadata.rs:46-71`):

- Add `compute_image_hash: bool` flag
- Add `image_hash_type: ImageHashType` enum

### D. Landmines to Avoid

1. **Don't hash after metadata extraction** - The hash must be computed DURING file parsing, not as a post-processing step. ExifTool accumulates the hash as it reads the file.

2. **Chunk size matters** - ExifTool reads in 64KB chunks (`my $n = 65536`). Match this for memory efficiency on large files.

3. **Empty hash suppression** - Must detect and suppress empty hashes:

   - MD5: `d41d8cd98f00b204e9800998ecf8427e`
   - SHA256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
   - SHA512: `cf83e1357...`

4. **JPEG scan data is complex** - The data between SOS and EOI includes restart markers and stuffed bytes. Must hash the markers themselves (0xFF 0xDA, 0xFF 0xD0-D7) as well as the data.

5. **TIFF offset handling** - StripOffsets/TileOffsets values may be arrays for tiled images. Must seek to each offset and hash each strip/tile.

6. **On-demand computation** - Only compute hash when `--image-hash` or `-api requesttags=imagedatahash` is requested, not for every file (performance).

### E. Dependencies Required

Add to `Cargo.toml`:

```toml
md-5 = "0.10"    # MD5 hashing (default algorithm)
sha2 = "0.10"    # SHA256/SHA512 support (optional algorithms)
digest = "0.10"  # Common trait for hash algorithms
```

## Part 3: Tasks

### Task 1: Add Hash Dependencies and Types ✅ COMPLETED

**Status**: DONE (2025-12-11)

**Success**: `cargo build` succeeds with new dependencies

**Implementation**:

1. Add `md-5`, `sha2`, `digest` crates to `Cargo.toml`
2. Create `src/hash/mod.rs` with:

   ```rust
   pub enum ImageHashType {
       Md5,      // Default
       Sha256,
       Sha512,
   }

   pub struct ImageDataHasher {
       hasher: Box<dyn DynDigest + Send>,
       hash_type: ImageHashType,
       bytes_hashed: u64,
   }
   ```

**If architecture changed**: The hash module is standalone. If a different crypto crate is preferred, swap implementations behind the same trait.

**Proof of completion**:

- [x] `cargo build` succeeds
- [x] `rg "ImageHashType" src/hash/` shows enum definition
- [x] `rg "ImageDataHasher" src/hash/` shows struct definition

**What was implemented**:

- Added `md-5 = "0.10"`, `sha2 = "0.10"`, `digest = "0.10"` to `Cargo.toml`
- Created `src/hash/mod.rs` with full implementation including:
  - `ImageHashType` enum with `from_str()`, `empty_hash()`, `Display` trait
  - `ImageDataHasher` struct with:
    - `new(hash_type)` - create hasher with specified algorithm
    - `update(&[u8])` - accumulate data into hash
    - `hash_from_reader()` - hash from reader in 64KB chunks (ExifTool compatible)
    - `hash_at_offset()` - seek and hash for TIFF strip/tile data
    - `finalize()` - returns `None` for empty hashes (ExifTool behavior)
    - `finalize_unchecked()` - always returns hash
    - `bytes_hashed()` - tracking for verbose output
  - 10 passing unit tests
- Re-exported types in `src/lib.rs`: `pub use hash::{ImageDataHasher, ImageHashType};`

### Task 2: Add ImageDataHash Option to FilterOptions ✅ COMPLETED

**Status**: DONE (2025-12-11)

**Success**: `FilterOptions` can be configured to compute image hash

**Implementation**:

1. ~~Add to `ExifReader` struct~~ (deferred to Task 2b)
2. Add to `FilterOptions` (`src/types/metadata.rs`):
   ```rust
   pub compute_image_hash: bool,
   pub image_hash_type: ImageHashType,
   ```
3. ~~Wire up in `ExifReader::new()` based on filter options~~ (deferred to Task 2b)

**If architecture changed**: Look for reader configuration in `src/exif/mod.rs` and option storage in `src/types/`.

**Proof of completion**:

- [ ] `rg "image_data_hasher" src/exif/mod.rs` shows field (Task 2b)
- [x] `rg "compute_image_hash" src/types/` shows option

**What was implemented**:

- Added `compute_image_hash: bool` field to `FilterOptions`
- Added `image_hash_type: ImageHashType` field to `FilterOptions`
- Added `image_hash_only(hash_type)` constructor for hash-only operations
- Updated `Default`, `tags_only()`, `groups_only()` to include new fields
- Updated all FilterOptions usages in:
  - `src/types/metadata.rs` (tests)
  - `src/compat/filtering.rs`
  - `src/main.rs`
- All 416 existing tests pass

### Task 2b: Wire ExifReader to FilterOptions ✅ COMPLETED

**Status**: DONE (2025-12-13)

**Success**: `ExifReader` can be configured to compute image hash

**Implementation**:

1. Add to `ExifReader` struct (`src/exif/mod.rs`):
   ```rust
   pub(crate) image_data_hasher: Option<ImageDataHasher>,
   ```
2. Wire up in `ExifReader::new()` based on filter options
3. Add accessor methods for hasher lifecycle

**Proof of completion**:

- [x] `rg "image_data_hasher" src/exif/mod.rs` shows field

**What was implemented**:

- Added `image_data_hasher: Option<ImageDataHasher>` field to `ExifReader`
- Added `set_image_data_hasher(hasher)` method to set hasher
- Added `image_data_hasher_mut()` to get mutable reference for format handlers
- Added `take_image_data_hasher()` to consume hasher for finalization
- Added `is_hashing_enabled()` convenience method
- Added hasher creation in `extract_metadata()` when `filter_opts.compute_image_hash` is true
- Added hash finalization in `extract_metadata()` that:
  - Finalizes hash and adds `ImageDataHash` tag to File group
  - Suppresses empty hashes (matching ExifTool behavior)
  - Logs bytes hashed for debugging
- All existing tests pass

### Task 3: Implement JPEG Image Data Hashing ✅ COMPLETED

**Status**: DONE (2025-12-13)

**Success**: `./target/release/exif-oxide --image-hash test.jpg` matches ExifTool

**Implementation**:

1. Modify `scan_jpeg_segments()` in `src/formats/jpeg.rs` to accept optional hasher
2. After SOS marker (0xDA), hash:
   - The 0xFF 0xDA marker bytes
   - All segment data
   - RST markers 0xD0-0xD7 with their marker bytes
   - Stuffed bytes sequences
3. Stop at EOI (0xD9)

**ExifTool Reference**: `lib/Image/ExifTool.pm:7217-7406`

**Key Code Pattern** (translate from Perl):

```perl
# ExifTool ProcessJPEG hash logic
if ($hash and defined $marker and ($marker == 0x00 or $marker == 0xda or
    ($marker >= 0xd0 and $marker <= 0xd7)))
{
    $hash->add("\xff" . chr($marker));
    $hash->add($buff);
    $hashsize += $skipped + 2;
}
```

**If architecture changed**: JPEG processing lives in `src/formats/jpeg.rs`. The key is finding where segment iteration happens and adding hash calls.

**Proof of completion**:

- [x] `hash_jpeg_scan_data()` function implemented in `src/formats/jpeg.rs`
- [x] 5 unit tests passing

**What was implemented**:

- Created `hash_jpeg_scan_data()` function in `src/formats/jpeg.rs`:
  - Scans JPEG from start, skipping segments until SOS
  - Hashes SOS marker (0xFF 0xDA) + length + header
  - Processes entropy-coded data byte-by-byte
  - Handles stuffed bytes (0xFF 0x00) - hashes both bytes
  - Handles RST markers (0xFF 0xD0-0xD7) - hashes both bytes
  - Stops at EOI (0xFF 0xD9) - does NOT hash EOI
  - Uses 64KB buffer for efficient chunked reading
- Exported `hash_jpeg_scan_data` in `src/formats/mod.rs`
- Integrated into JPEG processing in `extract_metadata()`
- Added 5 unit tests:
  - `test_hash_jpeg_scan_data_minimal`
  - `test_hash_jpeg_scan_data_with_stuffed_bytes`
  - `test_hash_jpeg_scan_data_with_rst_marker`
  - `test_hash_jpeg_scan_data_no_sos`
  - `test_hash_jpeg_scan_data_with_app_segment`

---

## Part 4: Implementation Progress & Architecture Insights

### Progress Summary (2025-12-13)

| Task | Status | Key Files Modified |
|------|--------|-------------------|
| Task 1: Hash Dependencies | ✅ DONE | `Cargo.toml`, `src/hash/mod.rs`, `src/lib.rs` |
| Task 2: FilterOptions | ✅ DONE | `src/types/metadata.rs`, `src/compat/filtering.rs`, `src/main.rs` |
| Task 2b: Wire ExifReader | ✅ DONE | `src/exif/mod.rs`, `src/formats/mod.rs` |
| Task 3: JPEG Hashing | ✅ DONE | `src/formats/jpeg.rs`, `src/formats/mod.rs` |
| Task 4: PNG Hashing | ⏳ PENDING | `src/formats/png.rs` |
| Task 5: TIFF Hashing | ⏳ PENDING | `src/exif/ifd.rs` |
| Task 6: CLI Support | ⏳ PENDING | `src/main.rs` |
| Task 7: Integration Tests | ⏳ PENDING | `tests/` |

### Key Architecture Pattern: Hash Lifecycle

The hasher follows a **create → accumulate → finalize** lifecycle that matches ExifTool:

```
1. CREATE: extract_metadata() in src/formats/mod.rs
   - Check filter_opts.compute_image_hash
   - Create ImageDataHasher with filter_opts.image_hash_type
   - Store as `mut image_data_hasher: Option<ImageDataHasher>`

2. ACCUMULATE: Format-specific handlers
   - JPEG: hash_jpeg_scan_data() hashes SOS through EOI
   - PNG: (pending) hash_png_image_data() hashes IDAT chunks
   - TIFF: (pending) hash at IsImageData tag offsets
   - Pass `&mut ImageDataHasher` to format handlers

3. FINALIZE: extract_metadata() after all processing
   - Call hasher.finalize()
   - Suppresses empty hashes automatically
   - Add ImageDataHash tag to File group if non-empty
```

### Critical Implementation Detail: Marker Inclusion

For JPEG hashing, the markers themselves ARE included in the hash (not just the data between them):

```rust
// CORRECT: Hash the marker bytes
hasher.update(&[0xFF, 0xDA]);  // SOS marker
hasher.update(&[0xFF, 0xD0]);  // RST0 marker
hasher.update(&[0xFF, 0x00]);  // Stuffed byte (literal 0xFF)

// WRONG: Only hashing data after markers
// This would produce incorrect hashes
```

This matches ExifTool's behavior in `lib/Image/ExifTool.pm:7366`:
```perl
$hash->add("\xff" . chr($marker));  # Includes 0xFF prefix
```

### PNG Implementation Notes (for Task 4)

Based on ExifTool's PNG.pm, the PNG hashing pattern is simpler than JPEG:

```rust
// Pseudo-code for hash_png_image_data()
fn hash_png_image_data(reader, hasher) -> u64 {
    // 1. Skip PNG signature (8 bytes)
    // 2. Iterate chunks: [length:4][type:4][data:length][crc:4]
    // 3. For IDAT, JDAT, fdAT chunks:
    //    - Hash ONLY the data portion (not length/type/CRC)
    //    - For fdAT: skip first 4 bytes (sequence number)
    // 4. Stop at IEND chunk
}
```

Key differences from JPEG:
- Hash chunk DATA only, not the chunk headers or CRC
- PNG signature (89 50 4E 47 0D 0A 1A 0A) is NOT hashed
- fdAT (animated PNG) has 4-byte sequence prefix to skip

### TIFF Implementation Notes (for Task 5)

TIFF hashing is different - it uses **offset/size tag pairs** with `IsImageData` flag:

```rust
// Pseudo-code for hash_tiff_image_data()
// This happens AFTER IFD parsing, not during
fn hash_tiff_image_data(reader, hasher, exif_reader) -> u64 {
    let image_data_tags = [
        (0x111, 0x117),  // StripOffsets + StripByteCounts
        (0x144, 0x145),  // TileOffsets + TileByteCounts
        (0x201, 0x202),  // JpgFromRawStart + JpgFromRawLength
    ];

    for (offset_tag, size_tag) in image_data_tags {
        if let (Some(offsets), Some(sizes)) =
            (get_tag(offset_tag), get_tag(size_tag))
        {
            // May be arrays for tiled/striped images
            for (offset, size) in zip(offsets, sizes) {
                hasher.hash_at_offset(reader, offset, size);
            }
        }
    }
}
```

Important: The offsets may need base adjustment (TIFF base offset for embedded EXIF).

### ExifReader Hasher Methods

Added to `src/exif/mod.rs` for format handler access:

```rust
impl ExifReader {
    // Set hasher before processing
    pub fn set_image_data_hasher(&mut self, hasher: ImageDataHasher);

    // Get mutable reference during processing
    pub fn image_data_hasher_mut(&mut self) -> Option<&mut ImageDataHasher>;

    // Take ownership for finalization
    pub fn take_image_data_hasher(&mut self) -> Option<ImageDataHasher>;

    // Check if hashing is enabled
    pub fn is_hashing_enabled(&self) -> bool;
}
```

### Testing Pattern

Unit tests for format-specific hashing should:
1. Create minimal valid file structure in memory
2. Hash with known algorithm (MD5 for determinism)
3. Verify bytes_hashed > 0 (sanity check)
4. Verify hash length matches algorithm (MD5=32, SHA256=64, SHA512=128)

Integration tests (Task 7) should compare against actual ExifTool output.

---

### Task 4: Implement PNG Image Data Hashing

**Success**: `./target/release/exif-oxide --image-hash test.png` matches ExifTool

**Implementation**:

1. Create `scan_png_chunks()` function that iterates all chunks
2. For IDAT, JDAT, fdAT chunks, hash the chunk data (not header or CRC)
3. Pass hasher through chunk iteration

**ExifTool Reference**: `lib/Image/ExifTool/PNG.pm:1419-1593`

**Key Pattern**:

```perl
my %isDataChunk = (IDAT=>1, JDAT=>1, fdAT=>1);
if ($isDataChunk{$chunk}) {
    if ($hash) {
        $et->ImageDataHash($raf, $len);
    }
}
```

**If architecture changed**: PNG processing in `src/formats/png.rs`. May need to extend beyond IHDR parsing.

**Proof of completion**:

- [ ] Test: PNG hash matches ExifTool
- [ ] Test: Handles APNG (fdAT chunks) correctly

### Task 5: Implement TIFF Image Data Hashing

**Success**: `./target/release/exif-oxide --image-hash test.tif` matches ExifTool

**Implementation**:

1. During IFD parsing, identify tags with IsImageData:
   - StripOffsets (0x111) + StripByteCounts (0x117)
   - TileOffsets (0x144) + TileByteCounts (0x145)
   - JpgFromRawStart (0x201) + JpgFromRawLength (0x202)
2. After IFD parsing, seek to each offset and hash the data
3. Handle multi-value offsets (arrays for tiled images)

**ExifTool Reference**: `lib/Image/ExifTool/Exif.pm:6200-7094`, `WriteExif.pl:425-462`

**Key Pattern**:

```perl
# AddImageDataHash function
foreach $tagID (sort keys %$offsetInfo) {
    next unless $$tagInfo{IsImageData};
    my @offsets = split ' ', $$offsetInfo{$tagID}[1];
    my @sizes = split ' ', $$offsetInfo{$sizeID}[1];
    foreach $offset (@offsets) {
        my $size = shift @sizes;
        $raf->Seek($offset, 0);
        $total += $et->ImageDataHash($raf, $size);
    }
}
```

**If architecture changed**: TIFF/IFD processing in `src/exif/ifd.rs`. Need to track IsImageData tags.

**Proof of completion**:

- [ ] Test: TIFF hash matches ExifTool
- [ ] Test: Handles tiled TIFFs correctly

### Task 6: Add CLI Support

**Success**: Both `--image-hash` flag and ExifTool-compatible `-api` options work

**ExifTool API Reference**:
ExifTool uses `-api` options for ImageDataHash (not a direct flag):

```bash
exiftool -api requesttags=imagedatahash -api imagehashtype=MD5 image.jpg
```

Reference: https://exiftool.org/forum/index.php?topic=14706.msg79218

**Implementation**:

1. In `src/main.rs` `parse_exiftool_args()`, add support for:

   - `--image-hash` / `--image-data-hash` - Simple flag to enable hashing
   - `--image-hash-type=md5|sha256|sha512` - Algorithm selection
   - `-api requesttags=imagedatahash` - ExifTool compatibility
   - `-api imagehashtype=MD5|SHA256|SHA512` - ExifTool algorithm option

2. Set `filter_options.compute_image_hash = true` when enabled
3. Set `filter_options.image_hash_type` based on algorithm option
4. Output as `Composite:ImageDataHash` in JSON (matching ExifTool group)

**API Option Parsing** (ExifTool style):

```rust
// In parse_exiftool_args(), handle -api KEY=VALUE pairs
if arg == "-api" {
    if let Some(next_arg) = args.next() {
        if let Some((key, value)) = next_arg.split_once('=') {
            match key.to_lowercase().as_str() {
                "requesttags" if value.eq_ignore_ascii_case("imagedatahash") => {
                    filter_options.compute_image_hash = true;
                }
                "imagehashtype" => {
                    filter_options.image_hash_type = match value.to_uppercase().as_str() {
                        "SHA256" => ImageHashType::Sha256,
                        "SHA512" => ImageHashType::Sha512,
                        _ => ImageHashType::Md5,
                    };
                }
                _ => {}
            }
        }
    }
}
```

**If architecture changed**: CLI parsing in `src/main.rs`. Look for existing filter flag handling patterns.

**Proof of completion**:

- [ ] `./target/release/exif-oxide --image-hash image.jpg` outputs MD5 hash
- [ ] `--image-hash-type=sha256` produces 64-char hex hash
- [ ] `--image-hash-type=sha512` produces 128-char hex hash
- [ ] `-api requesttags=imagedatahash` works (ExifTool compat)
- [ ] `-api imagehashtype=SHA256` works (ExifTool compat)

### Task 7: Integration Tests

**Success**: All format tests pass against ExifTool reference

**Implementation**:
Create `tests/image_data_hash_test.rs`:

```rust
#[test]
fn test_jpeg_image_data_hash_matches_exiftool() {
    // Compare our hash against ExifTool's output
}

#[test]
fn test_png_image_data_hash_matches_exiftool() { ... }

#[test]
fn test_tiff_image_data_hash_matches_exiftool() { ... }

#[test]
fn test_hash_unchanged_after_metadata_edit() {
    // Modify metadata with ExifTool, verify hash unchanged
}

#[test]
fn test_empty_hash_suppression() {
    // File with no image data should not output ImageDataHash
}
```

**Proof of completion**:

- [ ] `cargo t image_data_hash` - all tests pass
- [ ] Tests cover JPEG, PNG, TIFF formats
- [ ] Tests verify ExifTool compatibility

### Task 8: TIFF-Based RAW Formats (CR2, ARW, NEF, DNG)

**Success**: `./target/release/exif-oxide --image-hash test.cr2` matches ExifTool for all TIFF-based RAW formats

**Research Summary** (completed 2025-12-13):

All TIFF-based RAW formats use the same mechanism: **IsImageData flag on offset/size tag pairs** in Exif.pm. The hash includes actual sensor data (StripOffsets/TileOffsets) and full-size embedded JPEGs (JpgFromRaw), but excludes thumbnails and previews.

| Format    | Extension | Key IsImageData Tags                                                        |
| --------- | --------- | --------------------------------------------------------------------------- |
| Canon CR2 | .cr2      | StripOffsets (0x111), TileOffsets (0x144), JpgFromRaw (0x201)               |
| Sony ARW  | .arw      | StripOffsets (0x111), TileOffsets (0x144), JpgFromRaw (0x201)               |
| Nikon NEF | .nef      | StripOffsets (0x111), TileOffsets (0x144), JpgFromRaw (0x201)               |
| Adobe DNG | .dng      | StripOffsets (0x111), TileOffsets (0x144), OtherImageStart, PreviewJXLStart |

**ExifTool Source References**:

- `Exif.pm:582-655` - Tag definitions with `IsImageData => 1`
- `WriteExif.pl:425-462` - `AddImageDataHash()` function
- Key tags: 0x111 (StripOffsets) + 0x117 (StripByteCounts), 0x144 (TileOffsets) + 0x145 (TileByteCounts)

**Implementation** (leverages Task 5 TIFF work):

1. Ensure IFD parsing tracks `IsImageData` tags already defined in codegen
2. For each IsImageData offset tag, get corresponding size tag via `OffsetPair`
3. Handle multi-value offsets (arrays for tiled/striped images)
4. Hash each strip/tile at absolute file offset
5. Process all SubIFDs (main image often in SubIFD, not IFD0)

**DNG-specific handling**:

- Conditional tags based on Compression value (34892 = lossy JPEG, 52546 = JPEG XL)
- Multiple SubIFDs may each have IsImageData tags
- DNGPrivateData blob is NOT hashed (contains preserved maker notes)

**What's hashed**:

- RAW sensor data (StripOffsets/TileOffsets)
- Full embedded JPEG (JpgFromRaw in SubIFD)
- DNG OtherImage (compression 34892)
- DNG PreviewJXL (compression 52546)

**What's excluded**:

- ThumbnailImage (IFD1) - no IsImageData flag
- PreviewImage - no IsImageData flag
- Encrypted maker notes (Sony 0x94xx, Nikon encrypted sections)
- DNGPrivateData blob

**If architecture changed**: TIFF processing in `src/formats/tiff.rs`, IFD parsing in `src/exif/ifd.rs`. Key is IsImageData flag already in codegen.

**Proof of completion**:

- [ ] Test: CR2 hash matches ExifTool
- [ ] Test: ARW hash matches ExifTool
- [ ] Test: NEF hash matches ExifTool
- [ ] Test: DNG hash matches ExifTool
- [ ] Test: Handles multi-strip/tile images correctly

### Task 9: HEIC/AVIF Support (QuickTime-Based)

**Success**: `./target/release/exif-oxide --image-hash test.heic` matches ExifTool

**Research Summary** (completed 2025-12-13):

HEIC/AVIF use QuickTime container format with **codec-type detection** via `%isImageData` hash. Image data is located via `iloc` box item extents.

**Supported codec types** (QuickTime.pm line 537):

```perl
our %isImageData = ( av01 => 1, avc1 => 1, hvc1 => 1, lhv1 => 1, hvt1 => 1 );
```

| Codec | Description  | Format        |
| ----- | ------------ | ------------- |
| av01  | AV1 codec    | AVIF          |
| avc1  | H.264/AVC    | Some HEIC     |
| hvc1  | HEVC/H.265   | Main HEIC     |
| lhv1  | Layered HEVC | HEIC variants |
| hvt1  | HEVC tiles   | HEIC tiles    |

**ExifTool Source References**:

- `QuickTime.pm:537` - `%isImageData` codec hash
- `QuickTime.pm:9367-9376` - Hash processing for image items

**Implementation**:

1. Parse HEIF/AVIF container (already partially supported in `src/formats/avif.rs`)
2. Locate `iloc` box for item extent information
3. For each image item with codec type in `isImageData`:
   - Get item extents (offset, length pairs)
   - Seek to each extent (offset + base)
   - Hash extent data
4. Handle fragmented data (multiple extents per item)

**Key differences from TIFF**:

- Uses codec type detection, not tag IDs
- Data may be fragmented across multiple extents
- Extent offsets are relative to base offset
- No `OffsetPair` mechanism - extent includes offset AND size

**What's hashed**:

- Image items with codec types: av01, avc1, hvc1, lhv1, hvt1
- All extents for each item

**What's excluded**:

- Metadata tracks
- Thumbnail images
- Text tracks
- Atoms outside image item extents

**If architecture changed**: AVIF processing in `src/formats/avif.rs`. Need to add HEIC support and iloc box parsing.

**Proof of completion**:

- [ ] Test: HEIC (hvc1 codec) hash matches ExifTool
- [ ] Test: AVIF (av01 codec) hash matches ExifTool
- [ ] Test: Handles multi-extent images correctly

### Task 10: Canon CR3 Support (QuickTime-Based RAW)

**Success**: `./target/release/exif-oxide --image-hash test.cr3` matches ExifTool

**Research Summary** (completed 2025-12-13):

Canon CR3 uses QuickTime container (like HEIC), NOT TIFF structure like CR2. Uses same `%isImageData` codec detection as HEIC/AVIF.

**Implementation**: Same as Task 9 (HEIC/AVIF) - CR3 uses QuickTime.pm's `%isImageData` mechanism.

**ExifTool Source**: `QuickTime.pm:537, 9367-9376`

**Proof of completion**:

- [ ] Test: CR3 hash matches ExifTool

## Format Support Priority (Updated)

| Priority | Format        | Type                | Complexity | ExifTool Reference           |
| -------- | ------------- | ------------------- | ---------- | ---------------------------- |
| P0       | JPEG          | Scan data           | Medium     | ExifTool.pm:7217-7406        |
| P0       | PNG           | IDAT chunks         | Low        | PNG.pm:1419-1593             |
| P1       | TIFF          | IsImageData tags    | Medium     | Exif.pm:6200-7094            |
| P1       | CR2, ARW, NEF | TIFF-based RAW      | Medium     | Exif.pm (IsImageData)        |
| P1       | DNG           | TIFF + conditionals | Medium     | Exif.pm:582-655              |
| P2       | HEIC/AVIF     | QuickTime codec     | High       | QuickTime.pm:537, 9367-9376  |
| P2       | CR3           | QuickTime codec     | High       | QuickTime.pm:537, 9367-9376  |
| P3       | MOV/MP4       | Video tracks        | High       | QuickTimeStream.pl:1284-1570 |

**Implementation order**:

1. P0: JPEG, PNG (foundation)
2. P1: TIFF → CR2/ARW/NEF/DNG (reuse TIFF IsImageData)
3. P2: HEIC/AVIF/CR3 (QuickTime container)

Focus on P0 formats first for initial implementation.

## Quality Checklist

- [x] Problem and success criteria fit in one paragraph
- [x] Included actual commands that find relevant code
- [x] Documented "learned the hard way" lessons (chunk size, empty hash suppression, marker inclusion)
- [x] Each task has a verifiable success command
- [x] Explained how to adapt if architecture changed
- [x] ExifTool source references provided for each format
- [x] Code follows Trust ExifTool principle - match exact hashing logic
- [x] Added Part 4 with architecture insights for future engineers
- [x] Documented hash lifecycle pattern (create → accumulate → finalize)
- [x] Included pseudo-code for pending tasks (PNG, TIFF)
- [x] File reference section updated with implementation status

## Files Referenced

**ExifTool Sources**:

- `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md` - Complete implementation documentation
- `third-party/exiftool/lib/Image/ExifTool.pm:2766-2780` - Hash object creation
- `third-party/exiftool/lib/Image/ExifTool.pm:4378-4386` - Hash finalization
- `third-party/exiftool/lib/Image/ExifTool.pm:7217-7406` - JPEG hash logic
- `third-party/exiftool/lib/Image/ExifTool/PNG.pm:1419-1593` - PNG hash logic
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:582-655` - IsImageData tag definitions (TIFF/RAW)
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:6200-7094` - TIFF hash logic
- `third-party/exiftool/lib/Image/ExifTool/WriteExif.pl:425-462` - AddImageDataHash() function
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:537` - HEIC/AVIF/CR3 isImageData codec hash
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:9367-9376` - HEIC/AVIF hash processing

**exif-oxide Sources**:

- `src/hash/mod.rs` - ✅ NEW: ImageHashType enum, ImageDataHasher struct (10 tests)
- `src/types/metadata.rs:72-89` - ✅ UPDATED: FilterOptions with compute_image_hash, image_hash_type
- `src/compat/filtering.rs` - ✅ UPDATED: FilterOptions usages with new fields
- `src/main.rs` - ✅ UPDATED: FilterOptions usages with new fields (pending CLI flag parsing for Task 6)
- `src/lib.rs:49` - ✅ UPDATED: Re-exports ImageDataHasher, ImageHashType
- `Cargo.toml:74-77` - ✅ UPDATED: md-5, sha2, digest dependencies
- `src/hash/mod.rs` - ✅ CREATED: ImageDataHasher, ImageHashType, streaming hash support
- `src/exif/mod.rs` - ✅ UPDATED: image_data_hasher field + accessor methods
- `src/formats/mod.rs` - ✅ UPDATED: Hasher creation/finalization in extract_metadata()
- `src/formats/jpeg.rs` - ✅ UPDATED: hash_jpeg_scan_data() function + 5 unit tests
- `src/formats/png.rs` - PENDING: PNG chunk hashing (Task 4)
- `src/formats/tiff.rs` - PENDING: TIFF processing (Task 5 - may need src/exif/ifd.rs instead)
- `tests/image_data_hash_test.rs` - PENDING: Integration tests (Task 7)
