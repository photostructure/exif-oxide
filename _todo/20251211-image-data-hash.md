# TPP: ImageDataHash Support — TIFF & TIFF-based RAW (CR2/ARW/NEF/DNG)

**Status (2026-07-01)**: JPEG and PNG hashing shipped — see `_done/20251211-image-data-hash-jpeg-png.md`. This file covers the remaining TIFF work (Task 5) and TIFF-based RAW formats (Task 8). HEIC/AVIF and CR3 (QuickTime-based, unrelated architecture) are tracked separately in `_todo/20251211-image-data-hash-heic-cr3.md`.

**Verified 2026-07-01**: `rg "IsImageData|hash_at_offset|hash_tiff_image_data" src/formats/tiff.rs src/exif/ifd.rs` returns nothing — this work has not been started despite being next in priority order.

## Current phase

- [x] Research & Planning
- [ ] Write breaking tests (JPEG/PNG were covered by `tests/image_data_hash_test.rs`; TIFF/RAW tests do not exist yet)
- [x] Design alternatives
- [x] Task breakdown
- [ ] Implementation (Task 5: TIFF, Task 8: CR2/ARW/NEF/DNG)
- [ ] Review & Refinement
- [ ] Final Integration

## Required reading

- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- [TDD.md](../docs/TDD.md)
- `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md` — overview of ExifTool's ImageDataHash design
- `_done/20251211-image-data-hash-jpeg-png.md` — shipped JPEG/PNG work; establishes the hasher lifecycle these tasks reuse
- `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md` — prior art for adding an offset-related tag attribute to codegen (see Tribal Knowledge below)

## Summary

**Problem**: TIFF, CR2, ARW, NEF, and DNG files have no `ImageDataHash` support. exif-oxide already hashes JPEG scan data and PNG IDAT chunks; the same content-integrity feature is missing for TIFF-structured formats.

**Why it matters**: Most RAW formats (Canon CR2, Sony ARW, Nikon NEF, Adobe DNG) are TIFF-structured. Without this, the feature has no coverage for RAW workflows, which is where content-hash verification (detecting sensor-data tampering vs. metadata edits) matters most.

**Solution**: Hash the actual pixel/strip/tile data referenced by TIFF offset tags (`StripOffsets`, `TileOffsets`, `JpgFromRawStart`, and DNG-specific variants), excluding thumbnails, previews, and maker notes — using the `ImageDataHasher` already built for JPEG/PNG.

**Success test**:

```bash
cargo build --release
./target/release/exif-oxide --image-hash test.tif
exiftool -api requesttags=imagedatahash test.tif
# identical hash

./target/release/exif-oxide --image-hash test.cr2
exiftool -api requesttags=imagedatahash test.cr2
# identical hash (repeat for .arw, .nef, .dng)
```

**Key constraint**: Hash values MUST match ExifTool exactly — same bytes, in the same tag-iteration order, per format.

## Tribal knowledge

### The hasher API and lifecycle already exist — don't rebuild them

`src/hash/mod.rs` already provides everything a format handler needs:

```rust
impl ImageDataHasher {
    pub fn new(hash_type: ImageHashType) -> Self;
    pub fn update(&mut self, data: &[u8]);
    pub fn hash_from_reader<R: Read>(&mut self, reader: &mut R, len: u64) -> Result<u64>;
    pub fn hash_at_offset<R: Read + Seek>(&mut self, reader: &mut R, offset: u64, len: u64) -> Result<u64>;
    pub fn bytes_hashed(&self) -> u64;
    pub fn finalize(self) -> Option<String>;       // None if no data was hashed (matches ExifTool's empty-digest suppression)
    pub fn finalize_unchecked(self) -> String;
}
```

`hash_at_offset()` (line 177) is exactly what TIFF strip/tile hashing needs — seek to an absolute file offset, hash `len` bytes.

The create → accumulate → finalize lifecycle is already wired in `src/formats/mod.rs`: hasher created at line ~78 when `filter_opts.compute_image_hash` is set, finalized (and the `ImageDataHash` tag added, empty hashes suppressed) at line ~1335. `src/exif/mod.rs` exposes the hasher to format handlers via `image_data_hasher_mut()` (line ~127) and `take_image_data_hasher()` (line ~133). **Your job for Task 5/8 is to call into this existing hasher after IFD parsing, not to build a new lifecycle.**

### CORRECTION to earlier drafts of this TPP: `IsImageData` does NOT exist in codegen

An earlier version of this TPP's Task 8 said "Ensure IFD parsing tracks IsImageData tags already defined in codegen." **This is false — verified 2026-07-01.** `rg -ni "isimagedata" codegen/ src/` (excluding a single doc-comment in `src/hash/mod.rs`) returns nothing. ExifTool's `Exif.pm` has 13 tags flagged `IsImageData => 1`, but exif-oxide's codegen has never extracted this attribute. `src/types/tag_info.rs:9-27` defines `TagInfo` with only `is_offset: bool` — there is no `is_image_data` field to check.

What *does* already exist: `StripOffsets`, `StripByteCounts`, `TileOffsets`, `JpgFromRawStart`, `JpgFromRawLength` are present as ordinary tags in `src/generated/Exif_pm/main_tags.rs` (e.g. lines 178, 206, 409, 3008, 3018), and the offset tags already carry `is_offset: true`, so their values are absolute file offsets by the time IFD parsing finishes (see `is_offset` adjustment logic referenced in `src/main.rs:459-461`). What's missing is purely the classification "this tag's bytes are hashable image data" — see Solutions below for two ways to get it.

**Precedent for adding a new codegen attribute**: `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md` documents exactly this kind of change for `IsOffset` — added to `codegen/src/strategies/tag_kit.rs` (`extract_is_offset()` at line 805, used at lines 465-468) and to `src/types/tag_info.rs`. If Option B (below) is chosen, follow this same pattern with an `extract_is_image_data()` function.

### An existing (but not directly reusable) offset/length pattern

`src/main.rs:462-575` (`extract_binary_data()` / `find_tag_pair()`) already resolves named offset/length tag pairs (`ThumbnailOffset`/`ThumbnailLength`, `PreviewImageStart`/`PreviewImageLength`, `OtherImageStart`/`OtherImageLength`) by scanning `ExifData.tags` by name. This is useful as a *reference pattern* for how tag-pair lookups are named and matched — but it operates on the fully-built `ExifData` after `extract_metadata()` returns, which is too late for hashing (the hasher must accumulate and finalize inside `extract_metadata()`, per the landmine below). Don't call it directly; adapt the tag-name-matching idea to whatever internal, mid-parse tag store the IFD parser exposes at the point Task 5 hooks in.

### Landmines

1. **Hash during parsing, not after**: ExifTool computes the hash as it reads the file, inside the same pass as metadata extraction — it is not a post-processing step over already-extracted tags. For TIFF this means: after IFD0 (and any SubIFDs) are parsed and the offset/size tag values are known, hash immediately — still within the same `extract_metadata()` call, before the existing finalize step at `src/formats/mod.rs:~1335`.
2. **Offsets may be arrays**: `StripOffsets`/`StripByteCounts` and `TileOffsets`/`TileByteCounts` are per-strip/per-tile arrays for tiled or multi-strip images. Zip offsets with sizes and hash each range in order.
3. **Base offset adjustment**: TIFF offsets are relative to a base that can shift for embedded EXIF (e.g. inside JPEG) vs. standalone TIFF/RAW. The `is_offset: true` tags are already adjusted to absolute file positions by existing IFD parsing (confirmed in `src/generated/Exif_pm/main_tags.rs`) — verify this holds for `StripOffsets`/`TileOffsets`/`JpgFromRawStart` specifically before assuming it, since the thumbnail-offset precedent TPP only confirms it for `ThumbnailOffset`/`PreviewImageStart`.
4. **Multiple SubIFDs**: RAW formats (CR2, NEF, DNG) frequently store the actual sensor image in a SubIFD, not IFD0 — IFD0 often holds only a preview/thumbnail. All SubIFDs must be walked, and only the ones with genuine image-data tags should be hashed.
5. **DNG conditional tags**: DNG's `Compression` value determines which tags carry the real image data (34892 = lossy JPEG via `OtherImageStart`/`OtherImageLength`, 52546 = JPEG XL via `PreviewJXLStart`/`PreviewJXLLength`). `DNGPrivateData` (preserved maker notes) must NOT be hashed.
6. **Exclusions**: `ThumbnailImage` (IFD1), `PreviewImage`, and encrypted maker-note blobs (Sony 0x94xx, Nikon encrypted sections) have no `IsImageData` equivalent in ExifTool and must not be hashed.

### Testing pattern

Unit tests for format-specific hashing (following the JPEG/PNG precedent in `_done/20251211-image-data-hash-jpeg-png.md`) should: build a minimal valid file structure in memory, hash with a known algorithm (MD5 for determinism), verify `bytes_hashed() > 0` as a sanity check, and verify hash length matches the algorithm (MD5=32 hex chars, SHA256=64, SHA512=128). Integration tests (in `tests/image_data_hash_test.rs`) should compare against real `exiftool -api requesttags=imagedatahash` output and skip gracefully if the required test fixture isn't present, matching the existing JPEG/PNG tests.

### Format priority context

From the overall ImageDataHash effort: P0 (JPEG/PNG) shipped — see `_done/20251211-image-data-hash-jpeg-png.md`. This file is P1:

| Priority | Format | Complexity | ExifTool Reference |
| --- | --- | --- | --- |
| P1 | TIFF | Medium | `Exif.pm:6200-7094` |
| P1 | CR2, ARW, NEF | Medium (TIFF-based RAW) | `Exif.pm` (`IsImageData`) |
| P1 | DNG | Medium (TIFF + conditionals) | `Exif.pm:582-655` |

P2 (HEIC/AVIF/CR3) is tracked in `_todo/20251211-image-data-hash-heic-cr3.md` and has no dependency on this file landing first.

## Solutions

### Option A (preferred): Hardcoded known tag-name pairs, no codegen change

Maintain a small static list of `(offset_tag_name, size_tag_name)` pairs in the Rust hashing code — `("StripOffsets", "StripByteCounts")`, `("TileOffsets", "TileByteCounts")`, `("JpgFromRawStart", "JpgFromRawLength")`, plus DNG's `("OtherImageStart", "OtherImageLength")` and `("PreviewJXLStart", "PreviewJXLLength")` gated on `Compression`. This mirrors the existing `find_tag_pair` pattern in `src/main.rs` and needs zero codegen changes.

- Pro: Small, testable, matches SIMPLE-DESIGN's "fewest elements" — ExifTool's own `IsImageData` set is only 13 tags and rarely changes.
- Con: If ExifTool adds a new `IsImageData` tag upstream, someone has to notice and update the hardcoded list by hand.

### Option B: Extend codegen to extract `IsImageData`, like `IsOffset`

Add `is_image_data: bool` to `TagInfo` (`src/types/tag_info.rs`), add `extract_is_image_data()` to `codegen/src/strategies/tag_kit.rs` alongside `extract_is_offset()` (line 805), and drive hashing off the generated flag instead of a hardcoded name list.

- Pro: Automatically stays in sync with ExifTool's tag tables; "Trust ExifTool" via generation rather than manual transcription.
- Con: Touches codegen and regenerates ~hundreds of generated files for a 13-tag attribute; larger blast radius for a set of tags that's been stable for years.

**Recommendation**: Start with Option A for Task 5/8. Revisit Option B only if HEIC/AVIF or a future format needs the same flag on a much larger tag set (unlikely — QuickTime uses codec-type detection, not per-tag flags; see the sibling TPP).

## Tasks

### Task 5: Implement TIFF Image Data Hashing

**Success**: `./target/release/exif-oxide --image-hash test.tif` matches ExifTool.

**Implementation**:

1. During/after IFD0 and SubIFD parsing (`src/exif/ifd.rs`), resolve the tag pairs from Option A (or the generated flag from Option B) to get offset(s) and size(s).
2. For each `(offset, size)` pair, call `hasher.hash_at_offset(reader, offset, size)` (`src/hash/mod.rs:177`) — do this from wherever `extract_metadata()` has both the reader and the hasher in scope (see `src/formats/mod.rs`), likely requiring a hook after TIFF/IFD parsing completes but before the existing finalize call.
3. Handle multi-value offsets (arrays) for tiled/multi-strip images — zip offsets with sizes in order.

**ExifTool Reference**: `lib/Image/ExifTool/Exif.pm:6200-7094`, `lib/Image/ExifTool/WriteExif.pl:425-462` (`AddImageDataHash()`):

```perl
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

**If architecture changed**: TIFF/IFD processing lives in `src/exif/ifd.rs` (IFD walking) and `src/formats/tiff.rs` (top-level TIFF file handling). If neither exists by the name you expect, search: `rg "fn extract_tiff_exif|fn parse_ifd" src/`.

**Proof of completion**:

- [ ] Test: TIFF hash matches ExifTool (`cargo t` new test in `tests/image_data_hash_test.rs`)
- [ ] Test: handles tiled TIFFs (multi-offset arrays) correctly
- [ ] `rg "hash_at_offset" src/exif/ifd.rs src/formats/tiff.rs` shows usage

### Task 8: TIFF-based RAW Formats (CR2, ARW, NEF, DNG)

**Success**: `./target/release/exif-oxide --image-hash test.cr2` (and `.arw`, `.nef`, `.dng`) matches ExifTool.

All of these are TIFF-structured and route through the same IFD parsing as Task 5 (confirmed: only `CR3`/`CRW` are special-cased as non-TIFF in `src/formats/mod.rs:939-987` and `src/file_detection/tiff_raw.rs:31-33` — CR2/ARW/NEF/DNG are not). **This task should mostly fall out of Task 5 once it lands**; the remaining work here is per-format verification and the DNG-specific conditional tags.

| Format | Extension | Image-data tag pairs |
| --- | --- | --- |
| Canon CR2 | `.cr2` | StripOffsets/StripByteCounts, TileOffsets/TileByteCounts, JpgFromRawStart/JpgFromRawLength |
| Sony ARW | `.arw` | StripOffsets/StripByteCounts, TileOffsets/TileByteCounts, JpgFromRawStart/JpgFromRawLength |
| Nikon NEF | `.nef` | StripOffsets/StripByteCounts, TileOffsets/TileByteCounts, JpgFromRawStart/JpgFromRawLength |
| Adobe DNG | `.dng` | StripOffsets/StripByteCounts, TileOffsets/TileByteCounts, OtherImageStart/OtherImageLength (Compression=34892), PreviewJXLStart/PreviewJXLLength (Compression=52546) |

**Implementation**:

1. After Task 5 lands, confirm CR2/ARW/NEF hash correctly with no additional code (they use the same tag pairs).
2. For DNG, branch on the `Compression` tag value to select the correct extra tag pair (see Landmine 5 above).
3. Process all SubIFDs — the real sensor image is frequently in a SubIFD, not IFD0 (Landmine 4).

**ExifTool Reference**: `lib/Image/ExifTool/Exif.pm:582-655` (`IsImageData` tag definitions), `WriteExif.pl:425-462`.

**Proof of completion**:

- [ ] Test: CR2 hash matches ExifTool
- [ ] Test: ARW hash matches ExifTool
- [ ] Test: NEF hash matches ExifTool
- [ ] Test: DNG hash matches ExifTool (including a lossy-JPEG-compression sample)
- [ ] Test: multi-strip/tile RAW handled correctly

## Files referenced

**ExifTool sources**:

- `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md`
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:582-655` — `IsImageData` tag definitions
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:6200-7094` — TIFF hash logic
- `third-party/exiftool/lib/Image/ExifTool/WriteExif.pl:425-462` — `AddImageDataHash()`

**exif-oxide sources**:

- `src/hash/mod.rs` — `ImageDataHasher`/`ImageHashType` (done; reuse, don't rebuild)
- `src/exif/mod.rs:86,121-139` — hasher field + accessor methods on `ExifReader` (done)
- `src/formats/mod.rs:~78,~1335` — hasher create/finalize lifecycle in `extract_metadata()` (done)
- `src/types/tag_info.rs:9-27` — `TagInfo` struct (`is_offset` exists; `is_image_data` does not — see Solutions)
- `codegen/src/strategies/tag_kit.rs:465-468,805` — where `is_offset` is extracted; template for `is_image_data` if Option B is chosen
- `src/generated/Exif_pm/main_tags.rs` — confirms `StripOffsets`/`TileOffsets`/`JpgFromRawStart`/`JpgFromRawLength` already exist as tags (lines 178, 206, 409, 3008, 3018)
- `src/main.rs:462-575` — `extract_binary_data()`/`find_tag_pair()`, reference pattern for offset/length tag-pair resolution (not directly reusable, see Tribal Knowledge)
- `src/exif/ifd.rs`, `src/formats/tiff.rs` — where Task 5's hook needs to be added (PENDING)
- `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md` — precedent for adding a codegen tag attribute

## Quality checklist

- [x] Problem and success criteria fit in one paragraph
- [x] Included actual commands that find relevant code
- [x] Documented a "learned the hard way" lesson (the `IsImageData` codegen correction)
- [ ] Each task has a verifiable success command that currently passes
- [x] Explained how to adapt if architecture changed
- [x] ExifTool source references provided for each format
- [x] Two design options presented with a recommendation
