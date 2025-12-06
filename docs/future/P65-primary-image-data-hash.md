# P65: PrimaryImageDataHash Implementation

**Duration**: 2-3 weeks  
**Goal**: Implement content-based hashing for image deduplication that excludes embedded images and metadata  
**Problem**: Need to identify duplicate images based on visual content alone, not embedded previews or metadata  

## Project Overview

### High-Level Goal
Create a new `PrimaryImageDataHash` feature that generates cryptographic hashes of ONLY the primary image/video content, explicitly excluding embedded images, thumbnails, and metadata for accurate deduplication.

### Problem Statement
- ExifTool's ImageDataHash includes embedded images (JpgFromRaw, OtherImage), making it unsuitable for deduplication
- Users need to identify duplicate photos even when they have different embedded previews or metadata
- Current solutions require extracting and comparing full images, which is inefficient

### Critical Constraints
- ⚡ Must stream data without loading entire files into memory
- 🎯 Hash ONLY primary visual content, no embedded images
- 🔧 Opt-in feature (not computed by default due to I/O cost)
- 📐 Clear API/CLI interface for integration into DAM workflows

## Background & Context

### Why This Work is Needed
- **Deduplication**: DAM systems need content-based matching, not file-based
- **Storage Efficiency**: Identify duplicate RAWs with different embedded JPEGs
- **User Request**: ExifTool's approach doesn't match deduplication use case
- **Clear Semantics**: "Primary" image data has unambiguous meaning

### Related Documentation
- [docs/todo/P64-MILESTONE-23-ImageDataHash.md](P64-MILESTONE-23-ImageDataHash.md) - Original ExifTool-compatible approach
- [docs/reference/SUPPORTED-FORMATS.md](../reference/SUPPORTED-FORMATS.md) - All formats we need to support

## Technical Foundation

### Key Differences from ExifTool
| Aspect | ExifTool ImageDataHash | PrimaryImageDataHash |
|--------|------------------------|---------------------|
| Embedded Images | Includes | **Excludes** |
| Thumbnails | Includes some | **Excludes all** |
| Purpose | Integrity verification | **Deduplication** |

### Core Components
- `src/hash/mod.rs` - Hash computation infrastructure
- `src/formats/` - Format-specific data identification
- `src/main.rs` - CLI integration
- `src/types/mod.rs` - API types

## Work Completed

### Research Findings

**1. ExifTool DOES Include Embedded Images** ✅
- Confirmed via exiftool-researcher agent investigation
- `IsImageData => 1` tags like JpgFromRaw are included
- This is intentional for integrity verification
- Not suitable for our deduplication use case

**2. Format-Specific Primary Data Patterns** ✅
- JPEG: SOS segments only (0xFFDA)
- PNG: IDAT chunks only
- TIFF/RAW: Main IFD strips/tiles (exclude SubIFDs)
- Video: First video track mdat chunks

## Remaining Tasks

### High Confidence Implementation Tasks

#### 1. Core Hash Infrastructure

**Acceptance Criteria**: Streaming hash computation with algorithm selection

**✅ Correct Output:**
```rust
let options = PrimaryImageHashOptions::default();
let hash = compute_primary_image_hash_from_path("image.jpg", options)?;
// Returns: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

**❌ Common Mistake:**
```rust
// Loading entire file into memory - WRONG
let data = std::fs::read("image.jpg")?;
let hash = sha256(&data);
```

**Implementation**: Create streaming infrastructure in `src/hash/mod.rs` with 64KB chunking

#### 2. API Design and Types

**Acceptance Criteria**: Clean API for library and CLI usage

**✅ Correct Output:**
```rust
pub struct PrimaryImageHashOptions {
    pub algorithm: HashAlgorithm,
    pub include_alpha: bool,
    pub video_stream_index: Option<usize>,
}
```

**Implementation**: Add to `src/types/mod.rs`, integrate with ExifReader

#### 3. CLI Integration

**Acceptance Criteria**: Enable via `-api` flag matching ExifTool pattern

**✅ Correct Output:**
```bash
# Enable PrimaryImageHash computation (following ExifTool's -api pattern)
exif-oxide -api PrimaryImageHash image.jpg
{
  "EXIF:Make": "Canon",
  "Composite:PrimaryImageDataHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  ...
}

# Multiple files
exif-oxide -api PrimaryImageHash *.jpg
[
  {
    "SourceFile": "image1.jpg",
    "Composite:PrimaryImageDataHash": "e3b0c44...",
    ...
  },
  {
    "SourceFile": "image2.jpg", 
    "Composite:PrimaryImageDataHash": "a1b2c3d...",
    ...
  }
]
```

**Implementation**: 
- Add `-api` argument parsing to `src/main.rs`
- Pass API options through to metadata extraction
- Only compute hash when `PrimaryImageHash` is in API options

#### 4. Format Implementations

**Acceptance Criteria**: Each format correctly identifies primary data

**✅ Correct Examples:**

JPEG:
```rust
// Hash ONLY SOS segments
for segment in segments {
    if segment.marker == 0xFFDA { // SOS
        hash_segment_data(segment);
    }
}
```

PNG:
```rust
// Hash ONLY IDAT chunks
for chunk in chunks {
    if chunk.type == b"IDAT" {
        hash_chunk_data(chunk);
    }
}
```

**❌ Common Mistake:**
```rust
// Including APP1 (EXIF) data - WRONG
if segment.marker >= 0xFFE0 && segment.marker <= 0xFFEF {
    hash_segment_data(segment);
}
```

### Tasks Requiring Research

#### 1. HEIC/HEIF Primary Item Detection 🔍

**Research Needed**:
- How to identify 'pitm' (primary item) box
- Distinguish from thumbnail/auxiliary items
- Handle multi-image HEIC files

**Success Criteria**: Can extract and hash ONLY the primary image from multi-image HEIC

#### 2. RAW Format Sensor Data Location 🔍

**Research Needed**:
- Canon CR2/CR3: Where is raw sensor data vs embedded JPEG?
- Nikon NEF: How to skip JpgFromRaw in SubIFD?
- Sony ARW: Strip locations in primary IFD

**Success Criteria**: Hash produces different results for same RAW with different embedded JPEGs

#### 3. Video Stream Selection 🔍

**Research Needed**:
- How to identify primary video track in MP4/MOV
- Skip audio/subtitle tracks
- Handle fragmented MP4

**Success Criteria**: Can hash video content without audio track

## Prerequisites

- ✅ Binary data extraction infrastructure (from previous milestones)
- ✅ Format detection and parsing
- ⚠️ HEIC/HEIF support (not yet implemented - needed for full coverage)

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_jpeg_metadata_change_same_hash() {
    // Same image, different EXIF = same hash
}

#[test]
fn test_raw_embedded_jpeg_different_hash() {
    // CR2 with different embedded JPEG = same hash
}
```

### Integration Tests

**Hash Validation Strategy**:

1. **Golden Reference Approach**:
```rust
// tests/primary_image_hash_golden.rs
const GOLDEN_HASHES: &[(&str, &str)] = &[
    ("test-images/jpeg/canon_eos_r5.jpg", "a1b2c3d4e5f6..."),
    ("test-images/png/transparent_logo.png", "f6e5d4c3b2a1..."),
    ("test-images/raw/nikon_d850.nef", "9876543210ab..."),
];

#[test]
fn test_golden_hashes() {
    for (path, expected) in GOLDEN_HASHES {
        let hash = compute_primary_image_hash(path, Default::default())?;
        assert_eq!(hash, *expected, "Hash mismatch for {}", path);
    }
}
```

2. **Invariant Testing**:
```rust
#[test]
fn test_metadata_changes_dont_affect_hash() {
    let original = "test.jpg";
    let modified = "test_modified.jpg";
    
    // Copy and modify metadata
    fs::copy(original, modified)?;
    modify_exif_metadata(modified)?;
    
    let hash1 = compute_primary_image_hash(original)?;
    let hash2 = compute_primary_image_hash(modified)?;
    assert_eq!(hash1, hash2);
}
```

3. **Cross-Format Validation**:
```rust
#[test]
fn test_same_image_different_formats() {
    // Same image saved as JPEG and PNG should have different hashes
    // (due to compression differences) but this tests our format handling
    let jpeg_hash = compute_primary_image_hash("image.jpg")?;
    let png_hash = compute_primary_image_hash("image.png")?;
    assert_ne!(jpeg_hash, png_hash);
}
```

### Manual Testing
```bash
# Create test scenario
cp image.jpg image2.jpg
exiftool -Artist="Different" image2.jpg

# Should produce same hash
exif-oxide -api PrimaryImageHash image.jpg image2.jpg | jq '.[].["Composite:PrimaryImageDataHash"]'
```

### Hash Snapshot Management

**Approach**: Use snapshot testing with review process
```toml
# Cargo.toml
[dev-dependencies]
insta = "1.34"
```

```rust
#[test]
fn test_hash_snapshots() {
    let hash = compute_primary_image_hash("test.jpg")?;
    insta::assert_snapshot!(hash);
}
```

When hashes change:
1. Review with `cargo insta review`
2. Document why hash changed in commit
3. Update golden references if needed

## Success Criteria & Quality Gates

### Core Requirements
- ✅ Produces consistent hashes for same visual content
- ✅ Excludes ALL embedded images/thumbnails
- ✅ Streams data (no full file loads)
- ✅ Supports all formats in SUPPORTED-FORMATS.md

### Performance Targets
- Process 100MB file with <200MB memory usage
- Hash computation speed >50MB/s
- Linear time complexity O(n) with file size

### Quality Gates
- All unit tests pass
- Integration tests cover all supported formats
- No memory leaks in streaming
- Documentation complete

### Post-Completion Tasks
- Update README.md with PrimaryImageDataHash feature documentation
- Update CLI help text to explain `-api PrimaryImageHash` option
- Add to API documentation with use case examples
- Create example scripts for deduplication workflows
- Document difference between our PrimaryImageDataHash and ExifTool's ImageDataHash
- Add performance benchmarks to documentation
- Consider contributing the concept back to ExifTool community

## Gotchas & Tribal Knowledge

### Design Decisions

**Why NOT Follow ExifTool**: ExifTool's ImageDataHash serves integrity verification, not deduplication. Including embedded images makes sense for their use case but breaks ours.

**Why Streaming is Critical**: RAW files can be 100MB+. Video files can be gigabytes. Loading into memory is not acceptable.

**Why Exclude Alpha by Default**: Most deduplication workflows care about visual content, not transparency. Make it opt-in.

### Implementation Warnings

**Format Detection First**: Must detect format before attempting to hash - different formats store primary data differently.

**Seek Support Required**: Can't stream linearly - must seek to data chunks and skip metadata.

**Empty Files**: Return standard empty hash for each algorithm (don't error).

### Edge Cases

**Multi-Image Files**: HEIC can contain multiple images. Only hash the one marked as primary.

**Fragmented Video**: MP4 can have mdat boxes scattered throughout. Must process all.

**Missing Data**: Some files may have metadata but no image data. Return empty hash.

## Implementation Notes

### Phase 1: Foundation (Week 1)
1. Core types and API
2. CLI integration
3. JPEG and PNG support
4. Basic test suite

### Phase 2: Expand Formats (Week 2)
1. TIFF/RAW formats
2. Video support (MP4/MOV)
3. WebP support
4. Performance optimization

### Phase 3: Advanced Formats (Week 3)
1. HEIC/HEIF (if prerequisites met)
2. AVIF support
3. Comprehensive testing
4. Documentation

This is a new feature not in ExifTool, so we have flexibility in the implementation while maintaining our high performance standards.