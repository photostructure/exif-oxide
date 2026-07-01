# TPP: ImageDataHash Support — HEIC/AVIF & CR3 (QuickTime-based)

**Status (2026-07-01)**: Not started. Split out from `_todo/20251211-image-data-hash.md` because these formats use a completely different container mechanism (QuickTime/ISO-BMFF boxes) than TIFF-based formats. JPEG/PNG shipped — see `_done/20251211-image-data-hash-jpeg-png.md`. TIFF/CR2/ARW/NEF/DNG are tracked in `_todo/20251211-image-data-hash.md` — this file has no dependency on that one landing first; the two can be worked in either order.

**Verified 2026-07-01**: `src/formats/avif.rs` has ISO-BMFF box parsing for `ispe`/`pitm`/`iinf`/`ipma` boxes (dimensions/primary-item metadata) but no `iloc` box parsing — the box that locates actual item data extents. `rg -n "iloc" src/formats/avif.rs` returns nothing. No CR3-specific source file exists (`find src -iname '*cr3*'` returns nothing); CR3 is currently only referenced in RAW format detection/dispatch, not content parsing.

## Current phase

- [x] Research & Planning
- [ ] Write breaking tests
- [x] Design alternatives (see Tribal Knowledge — no live design debate needed, ExifTool's mechanism is unambiguous)
- [x] Task breakdown
- [ ] Implementation (Task 9: HEIC/AVIF, Task 10: CR3)
- [ ] Review & Refinement
- [ ] Final Integration

## Required reading

- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- [TDD.md](../docs/TDD.md)
- `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md`
- `_done/20251211-image-data-hash-jpeg-png.md` — establishes the `ImageDataHasher` API and lifecycle these tasks reuse

## Summary

**Problem**: HEIC, AVIF, and Canon CR3 files have no `ImageDataHash` support. Unlike TIFF-based formats, these use an ISO-BMFF (QuickTime-style) container where image data is located via `iloc` (item location) box extents, not tag-based offset/size pairs.

**Why it matters**: HEIC is Apple's default photo format since iOS 11; CR3 is Canon's current mirrorless RAW format. Both are common enough that lacking hash support here is a visible feature-parity gap with ExifTool.

**Solution**: Parse the `iloc` box to find each image item's data extents (offset, length pairs), filter items by codec type (`ExifTool`'s `%isImageData` table), and hash the extent bytes using the existing `ImageDataHasher`.

**Success test**:

```bash
cargo build --release
./target/release/exif-oxide --image-hash test.heic
exiftool -api requesttags=imagedatahash test.heic
# identical hash

./target/release/exif-oxide --image-hash test.avif
exiftool -api requesttags=imagedatahash test.avif
# identical hash

./target/release/exif-oxide --image-hash test.cr3
exiftool -api requesttags=imagedatahash test.cr3
# identical hash
```

**Key constraint**: Hash values MUST match ExifTool exactly — same extents, same order, same codec-type filter.

## Tribal knowledge

### Reuse the existing hasher — don't rebuild it

`src/hash/mod.rs`'s `ImageDataHasher` (`new`, `update`, `hash_at_offset`, `finalize`) and the create/accumulate/finalize lifecycle already wired in `src/formats/mod.rs` (hasher created ~line 78, finalized ~line 1335) and `src/exif/mod.rs` (`image_data_hasher_mut()` ~line 127) are format-agnostic. Task 9/10 only need to call `hash_at_offset()` for each qualifying extent — see `_done/20251211-image-data-hash-jpeg-png.md` and the sibling TIFF TPP for the exact method signatures.

### Codec-type detection, not tag flags

ExifTool identifies hashable image items by **codec type**, not by a per-tag flag like TIFF's `IsImageData` (`third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:537`):

```perl
our %isImageData = ( av01 => 1, avc1 => 1, hvc1 => 1, lhv1 => 1, hvt1 => 1 );
```

| Codec | Description | Typical format |
| --- | --- | --- |
| `av01` | AV1 | AVIF |
| `avc1` | H.264/AVC | some HEIC |
| `hvc1` | HEVC/H.265 | main HEIC |
| `lhv1` | Layered HEVC | HEIC variants |
| `hvt1` | HEVC tiles | tiled HEIC |

Hash processing itself: `QuickTime.pm:9367-9376`.

### Where the data actually is: `iloc` box extents

Each image item's bytes are located via the `iloc` (item location) box: for each item, one or more `(extent_offset, extent_length)` pairs, with offsets relative to a construction-method-dependent base (usually file-absolute for `construction_method = 0`). This is structurally different from TIFF's offset+size tag pairs:

- No `OffsetPair`-style mechanism — the extent record itself carries both offset and length.
- An item's data may be fragmented across multiple extents; all must be hashed in order.
- Must cross-reference `iinf`/`infe` (item type/codec — already parsed, see below) with `iloc` (item location — not yet parsed) to know which items qualify.

### What already exists in `src/formats/avif.rs`

Confirmed present: `parse_box_header()`, `find_box_by_type()`, `parse_ispe_box()` (dimensions), `parse_pitm_box()` (primary item ID), `parse_iinf_box()`/`parse_infe_box()` (item info — this is where codec type per item would be read), `parse_ipma_box()` (item property associations), `extract_heic_dimensions_primary_item()`, `extract_avif_dimensions()`. None of these parse `iloc`. This is genuinely new parsing work, not a matter of wiring up something that already exists (contrast with Task 8 in the sibling TPP, which mostly falls out of Task 5).

### CR3 shares this exact mechanism

Confirmed via `src/file_detection/tiff_raw.rs:31-33` and `src/formats/mod.rs:939-987`: CR3 is detected as an `ftyp`-based MOV/QuickTime container (not TIFF), explicitly distinct from CR2/CRW handling. ExifTool uses the same `QuickTime.pm:537,9367-9376` `%isImageData` mechanism for CR3 as for HEIC/AVIF — Task 10 is Task 9 applied to a Canon-specific brand/item layout, not a separate design.

### Landmines

1. **Fragmented items**: an item with multiple extents must have all of them hashed, in extent order, as one continuous logical stream (matches ExifTool's per-item accumulation, not one hash per extent).
2. **Base offset**: extents may be relative to a `construction_method`-specific base (e.g. an `idat` box for `construction_method = 1`), not always the file start — verify against `iloc` version/flags before assuming absolute offsets.
3. **Exclusions**: metadata tracks, thumbnail items, and text tracks must not be hashed — filter strictly by the `%isImageData` codec-type table, not by "every item in `iloc`."
4. **No existing test fixtures confirmed** — check `test-images/` for `.heic`/`.avif`/`.cr3` samples before writing integration tests; the JPEG/PNG integration tests skip gracefully when fixtures are absent (`tests/image_data_hash_test.rs` pattern), follow the same approach here.

### Testing pattern

Same pattern established for JPEG/PNG (see `_done/20251211-image-data-hash-jpeg-png.md`): unit tests build a minimal valid file structure in memory, hash with a known algorithm (MD5 for determinism), verify `bytes_hashed() > 0`, and verify hash length matches the algorithm (MD5=32 hex chars, SHA256=64, SHA512=128). Integration tests in `tests/image_data_hash_test.rs` compare against real `exiftool -api requesttags=imagedatahash` output and skip gracefully when the fixture is missing.

## Tasks

### Task 9: HEIC/AVIF Support (QuickTime-based)

**Success**: `./target/release/exif-oxide --image-hash test.heic` and `--image-hash test.avif` match ExifTool.

**Implementation**:

1. Add `iloc` box parsing to `src/formats/avif.rs` — extract per-item extent lists `(offset, length)`.
2. Cross-reference with `iinf`/`infe` (already parsed) to get each item's codec type (4CC).
3. Filter items to those with codec type in `{av01, avc1, hvc1, lhv1, hvt1}`.
4. For each qualifying item, hash all its extents in order via `hasher.hash_at_offset()`.
5. Handle multi-extent items (fragmented data) as one continuous hash accumulation per item.

**ExifTool Reference**: `lib/Image/ExifTool/QuickTime.pm:537` (codec table), `QuickTime.pm:9367-9376` (hash processing).

**Proof of completion**:

- [ ] Test: HEIC (`hvc1` codec) hash matches ExifTool
- [ ] Test: AVIF (`av01` codec) hash matches ExifTool
- [ ] Test: multi-extent item handled correctly
- [ ] `rg "iloc" src/formats/avif.rs` shows new parsing code

### Task 10: Canon CR3 Support

**Success**: `./target/release/exif-oxide --image-hash test.cr3` matches ExifTool.

**Implementation**: Apply Task 9's `iloc`/codec-type mechanism to CR3's container. No separate design — same QuickTime `%isImageData` detection (`QuickTime.pm:537,9367-9376`). The main unknown is where CR3-specific box parsing should live; no `src/formats/cr3.rs` or `src/raw/formats/canon_cr3.rs` exists yet — check whether CR3 dispatch should reuse `src/formats/avif.rs`'s box-parsing primitives (`parse_box_header`, `find_box_by_type`) directly, or needs its own thin wrapper for Canon-specific brand/item quirks (`rg -n "cr3\|CR3" src/raw/formats/canon.rs` to check current CR3 handling before starting).

**Proof of completion**:

- [ ] Test: CR3 hash matches ExifTool
- [ ] `rg "iloc" src/raw/formats/canon.rs src/formats/avif.rs` shows CR3 wired to the same extent-hashing path as Task 9

## Format priority context

From the overall ImageDataHash effort (P0 = JPEG/PNG, shipped; P1 = TIFF/RAW, sibling TPP):

| Priority | Format | Complexity | ExifTool Reference |
| --- | --- | --- | --- |
| P2 | HEIC/AVIF | High (new `iloc` parsing) | `QuickTime.pm:537, 9367-9376` |
| P2 | CR3 | High (shares HEIC/AVIF mechanism) | `QuickTime.pm:537, 9367-9376` |
| P3 (not in scope here) | MOV/MP4 | High | `QuickTimeStream.pl:1284-1570` |

MOV/MP4 video-track hashing is explicitly out of scope for this TPP — no task exists for it; if picked up later, start a new TPP rather than folding it in here.

## Files referenced

**ExifTool sources**:

- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:537` — `%isImageData` codec table
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:9367-9376` — hash processing for image items
- `third-party/exiftool/lib/Image/ExifTool/QuickTimeStream.pl:1284-1570` — MOV/MP4 video tracks (out of scope, P3)

**exif-oxide sources**:

- `src/hash/mod.rs` — `ImageDataHasher`/`ImageHashType` (done; reuse)
- `src/exif/mod.rs`, `src/formats/mod.rs` — hasher lifecycle (done; reuse)
- `src/formats/avif.rs` — existing ISO-BMFF box parsing (`parse_box_header`, `find_box_by_type`, `parse_ispe_box`, `parse_pitm_box`, `parse_iinf_box`/`parse_infe_box`, `parse_ipma_box`); `iloc` parsing PENDING
- `src/raw/formats/canon.rs`, `src/raw/detector.rs`, `src/file_detection/tiff_raw.rs:31-33`, `src/formats/mod.rs:939-987` — confirm CR3 is detected/dispatched as QuickTime-based, not TIFF-based; no CR3 content-parsing file exists yet

## Quality checklist

- [x] Problem and success criteria fit in one paragraph
- [x] Included actual commands that find relevant code
- [x] Documented a "learned the hard way" lesson (iloc parsing doesn't exist yet — this is new work, not wiring)
- [ ] Each task has a verifiable success command that currently passes
- [x] Explained how to adapt if architecture changed
- [x] ExifTool source references provided for each format
