# TPP: ImageDataHash Support — JPEG & PNG (completed)

**Status**: Shipped 2025-12-13. Re-verified 2026-07-01: `cargo test --features test-helpers,integration-tests --test image_data_hash_test` — all 8 tests pass.

This is the historical record for the JPEG/PNG portion of the ImageDataHash feature. The remaining work (TIFF, TIFF-based RAW, HEIC/AVIF, CR3) continues in `_todo/20251211-image-data-hash.md` and `_todo/20251211-image-data-hash-heic-cr3.md`. Those files carry forward the general hasher API and lifecycle notes needed to build on this work — this file is a record of what shipped, not a dependency for future sessions.

## Summary

**Problem**: Users could not verify image content integrity independent of metadata changes — no way to detect if actual pixel/media data was modified while ignoring metadata edits.

**Why it matters**: Content verification matters for digital forensics, asset management, and detecting unauthorized image manipulation. ExifTool exposes this via `-api requesttags=imagedatahash`; users migrating to exif-oxide need feature parity.

**Solution delivered for this phase**: Compute MD5/SHA256/SHA512 hashes of JPEG scan data and PNG IDAT-family chunk data, matching ExifTool byte-for-byte, opt-in via `--image-hash` / `--image-hash-type`.

**ExifTool's own CLI API** (reference: https://exiftool.org/forum/index.php?topic=14706.msg79218) uses `-api` options rather than a dedicated flag: `exiftool -api requesttags=imagedatahash -api imagehashtype=MD5 image.jpg`. exif-oxide deliberately shipped only the simpler `--image-hash`/`--image-hash-type` flags (Task 6) — the `-api requesttags=imagedatahash` compatibility form was considered and explicitly deferred as unnecessary, since the simpler flag gives equivalent functionality.

**Success test** (both still pass against real ExifTool):

```bash
cargo build --release
./target/release/exif-oxide --image-hash test-images/apple/IMG_3755.JPG
exiftool -api requesttags=imagedatahash test-images/apple/IMG_3755.JPG
# identical Composite:ImageDataHash value

./target/release/exif-oxide --image-hash test-images/png/exif-rgb-thumbnail.png
exiftool -api requesttags=imagedatahash test-images/png/exif-rgb-thumbnail.png
# identical hash

exiftool -Comment="test" -o /tmp/modified.jpg test-images/apple/IMG_3755.JPG
./target/release/exif-oxide --image-hash /tmp/modified.jpg
# hash matches original (metadata-only edit doesn't change it)
```

## What shipped

| Task | Files | Notes |
|------|-------|-------|
| Task 1: Hash dependencies & types | `Cargo.toml`, `src/hash/mod.rs`, `src/lib.rs` | `ImageHashType` enum (Md5/Sha256/Sha512), `ImageDataHasher` struct (`new`, `update`, `hash_from_reader`, `hash_at_offset`, `finalize`, `finalize_unchecked`, `bytes_hashed`) |
| Task 2: FilterOptions | `src/types/metadata.rs`, `src/compat/filtering.rs`, `src/main.rs` | `compute_image_hash: bool`, `image_hash_type: ImageHashType`, `image_hash_only()` constructor |
| Task 2b: Wire ExifReader | `src/exif/mod.rs` | `image_data_hasher: Option<ImageDataHasher>` field + `set_image_data_hasher`/`image_data_hasher_mut`/`take_image_data_hasher`/`is_hashing_enabled` |
| Task 3: JPEG hashing | `src/formats/jpeg.rs` (`hash_jpeg_scan_data`, ~line 855) | 5 unit tests |
| Task 4: PNG hashing | `src/formats/png.rs` (`hash_png_image_data`, ~line 254) | 3 unit tests |
| Task 6: CLI support | `src/main.rs` | `--image-hash`, `--image-hash-type <MD5\|SHA256\|SHA512>` |
| Task 7: Integration tests | `tests/image_data_hash_test.rs` | 8 tests comparing against real `exiftool -api requesttags=imagedatahash` output |

## Format-specific implementation notes (historical — JPEG/PNG only)

**JPEG** (ExifTool: `lib/Image/ExifTool.pm:7217-7406`, marker-inclusion detail at line 7366):

- Hash starts at the SOS (SOF 0xDA) marker, continues through EOI (0xD9), which is *not* hashed.
- The marker bytes themselves are part of the hash, not just the data between them:
  ```rust
  hasher.update(&[0xFF, 0xDA]);  // SOS marker
  hasher.update(&[0xFF, 0xD0]);  // RST0 marker
  hasher.update(&[0xFF, 0x00]);  // stuffed byte (literal 0xFF in entropy-coded data)
  ```
- RST markers (0xD0–0xD7) and stuffed bytes (0xFF 0x00) inside the entropy-coded data are hashed byte-for-byte; all APP segments, COM, DQT, DHT, SOF are excluded. ExifTool's marker-range check (`ExifTool.pm` ProcessJPEG):
  ```perl
  if ($hash and defined $marker and ($marker == 0x00 or $marker == 0xda or
      ($marker >= 0xd0 and $marker <= 0xd7)))
  {
      $hash->add("\xff" . chr($marker));
      $hash->add($buff);
  }
  ```
- `hash_jpeg_scan_data()` reads in a 64KB buffer (matches ExifTool's `my $n = 65536` chunking for large files).

**PNG** (ExifTool: `lib/Image/ExifTool/PNG.pm:1419-1593`, chunk table at line ~92):

- Only chunk *data* is hashed (not the 4-byte length, 4-byte type, or CRC).
- Data chunks: `IDAT`, `JDAT`, `JDAA` (matches ExifTool's `%isDatChunk` table — note the shipped code covers `JDAA` rather than the `fdAT` APNG variant mentioned in early design notes; APNG frame data was not part of this phase's test coverage). ExifTool's chunk check:
  ```perl
  my %isDataChunk = (IDAT=>1, JDAT=>1, fdAT=>1);
  if ($isDataChunk{$chunk} and $hash) {
      $et->ImageDataHash($raf, $len);
  }
  ```
- The 8-byte PNG signature is never hashed; iteration stops at `IEND`.

## Shared lifecycle these formats established (for context only — see `_todo/` files for the authoritative reference)

Hash object is created once per `extract_metadata()` call when `filter_opts.compute_image_hash` is set, accumulated during format-specific parsing, and finalized after all processing — mirroring ExifTool's own create-during-parse / finalize-in-`DoneExtract()` model:

```perl
# Creation - when ImageDataHash is requested (ExifTool.pm:2766-2780):
if ($$req{imagedatahash} and not $$self{ImageDataHash}) {
    my $imageHashType = $self->Options('ImageHashType');
    if ($imageHashType =~ /^SHA(256|512)$/i) {
        $$self{ImageDataHash} = Digest::SHA->new($1);
    } else {
        $$self{ImageDataHash} = Digest::MD5->new;
    }
}

# Finalization - in DoneExtract() (ExifTool.pm:4378-4386):
if ($$self{ImageDataHash}) {
    my $digest = $$self{ImageDataHash}->hexdigest;
    # Skip empty digests (no image data found)
    $self->FoundTag(ImageDataHash => $digest) unless $digest eq $emptyMD5;
}
```

Empty-hash suppression (no image data found → no `ImageDataHash` tag) is handled centrally at finalization (`ImageDataHasher::finalize()`, `src/hash/mod.rs:201-216`), not per-format. The empty-digest constants it checks against are hardcoded (`src/hash/mod.rs:72-77`, `empty_hash()`), matching the well-known empty-input digests for each algorithm:

- MD5: `d41d8cd98f00b204e9800998ecf8427e`
- SHA256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- SHA512: `cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e`

## Verification

```
cargo test --features test-helpers,integration-tests --test image_data_hash_test
# test_jpeg_image_hash_md5_matches_exiftool ... ok
# test_jpeg_image_hash_sha256_matches_exiftool ... ok
# test_jpeg_image_hash_sha512_matches_exiftool ... ok
# test_png_image_hash_md5_matches_exiftool ... ok
# test_png_image_hash_sha256_matches_exiftool ... ok
# test_hash_not_computed_when_not_requested ... ok
# test_sha512_hash_length ... ok
# test_hash_consistency ... ok
# 8 passed
```

Unit tests (format-internal): `test_hash_jpeg_scan_data_minimal`, `test_hash_jpeg_scan_data_with_stuffed_bytes`, `test_hash_jpeg_scan_data_with_rst_marker`, `test_hash_jpeg_scan_data_no_sos`, `test_hash_jpeg_scan_data_with_app_segment` (jpeg.rs); `test_hash_png_image_data_minimal`, `test_hash_png_image_data_multiple_idat`, `test_hash_png_image_data_invalid_signature` (png.rs).

## ExifTool references used (JPEG/PNG only)

- `third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md` — overview doc (still relevant to future format work)
- `third-party/exiftool/lib/Image/ExifTool.pm:2766-2780` — hash object creation
- `third-party/exiftool/lib/Image/ExifTool.pm:4378-4386` — hash finalization
- `third-party/exiftool/lib/Image/ExifTool.pm:7217-7406` — JPEG hash logic
- `third-party/exiftool/lib/Image/ExifTool.pm:7366` — marker-byte inclusion detail
- `third-party/exiftool/lib/Image/ExifTool/PNG.pm:1419-1593` — PNG hash logic

## Files touched

- `Cargo.toml` — added `md-5 = "0.10"`, `sha2 = "0.10"`, `digest = "0.10"`
- `src/hash/mod.rs` — new module: `ImageHashType`, `ImageDataHasher`, 10 unit tests
- `src/lib.rs` — re-exports `ImageDataHasher`, `ImageHashType`
- `src/types/metadata.rs` — `FilterOptions.compute_image_hash`, `.image_hash_type`
- `src/compat/filtering.rs`, `src/main.rs` — wired new `FilterOptions` fields through CLI and compat filtering
- `src/exif/mod.rs` — `image_data_hasher` field + accessor methods
- `src/formats/mod.rs` — hasher creation/finalization in `extract_metadata()`
- `src/formats/jpeg.rs` — `hash_jpeg_scan_data()`
- `src/formats/png.rs` — `hash_png_image_data()`
- `tests/image_data_hash_test.rs` — 8 integration tests
