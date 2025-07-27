# MILESTONE-23: ImageDataHash Implementation CANCELLED

**⚠️ UPDATE**: After research, we've decided to implement our own `PrimaryImageDataHash` instead. See [P65-primary-image-data-hash.md](P65-primary-image-data-hash.md) for the new approach that focuses on deduplication rather than integrity verification.

**Duration**: 2-3 weeks  
**Goal**: Implement cryptographic hashing of image/media content for integrity verification

## Project Overview

### High-Level Goal
Implement ExifTool-compatible ImageDataHash functionality that generates cryptographic fingerprints of media content (excluding metadata) for integrity verification, duplicate detection, and forensic analysis.

**⚠️ IMPORTANT DISCOVERY**: ExifTool's ImageDataHash includes embedded images (JpgFromRaw, OtherImage, etc.) which makes it unsuitable for deduplication use cases. For true content-based deduplication, see our custom [PrimaryImageDataHash](P65-primary-image-data-hash.md) implementation.

### Problem Statement
- Need to verify media content integrity independent of metadata changes
- Require duplicate detection across files with different metadata
- Support forensic workflows that track visual/audio content modifications

## Background & Context

### Why This Work is Needed
- **Content Authentication**: Verify image/video hasn't been visually altered
- **Duplicate Detection**: Find identical media with different metadata
- **Forensic Analysis**: Track content through processing pipelines
- **Digital Asset Management**: Content-based cataloging

### ExifTool Implementation References
- **Core hash function**: `lib/Image/ExifTool/Writer.pl:7085` - ImageDataHash()
- **TIFF/EXIF handler**: `lib/Image/ExifTool/WriteExif.pl` - AddImageDataHash()
- **Video processing**: `lib/Image/ExifTool/QuickTimeStream.pl` - ProcessSamples()
- **Hash initialization**: `lib/Image/ExifTool.pm:4327-4340`

## Technical Foundation

### Key ExifTool Patterns

**Hash Initialization** (`ExifTool.pm:4327`):
```perl
if ($$req{imagedatahash} and not $$self{ImageDataHash}) {
    my $imageHashType = $self->Options('ImageHashType');
    if ($imageHashType =~ /^SHA(256|512)$/i) {
        $$self{ImageDataHash} = Digest::SHA->new($1);
    } elsif (require Digest::MD5) {
        $$self{ImageDataHash} = Digest::MD5->new;
    }
}
```

**Format-Specific Data Identification**:
- `%isImageData` hashes identify content chunks per format
- `IsImageData => 1` tag property marks TIFF/EXIF image offsets
- Format-specific handlers for each file type

**Empty Hash Constants** (ignored by ExifTool):
- MD5: `d41d8cd98f00b204e9800998ecf8427e`
- SHA256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- SHA512: `cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e`

### Codegen Opportunities

**1. Format-Specific isImageData Tables**:
```perl
# From Jpeg2000.pm
my %isImageData = ( jp2c=>1, jbrd=>1, jxlp=>1, jxlc=>1 );

# From QuickTime.pm  
my %isImageData = ( av01=>1, avc1=>1, hvc1=>1, lhv1=>1, hvt1=>1 );

# From RIFF.pm
my %isImageData = (
    LIST_movi => 1,  # AVI video
    data => 1,       # WAV audio
    'VP8 '=>1, VP8L=>1, ANIM=>1, ANMF=>1, ALPH=>1, # WebP
);
```

**2. EXIF Tags with IsImageData Property**:
- Extract from `Exif.pm` tags with `IsImageData => 1`
- Generate offset/size pair mappings
- Examples: StripOffsets, TileOffsets, JpgFromRawStart

**3. Empty Hash Constant Table**:
- Generate lookup for the three empty hash values
- Map by algorithm type for quick detection

## Work Completed

### Research Findings

**Format-Specific Patterns Discovered**:

- **JPEG**: 
  - Hash SOS (Start of Scan) segments  
  - Include JpgFromRaw if present
  - Include OtherImage (non-thumbnail/preview)
  - JP2 format uses SOD marker

- **PNG**:
  - Hash all IDAT chunks (image data)
  - Include critical chunks: PLTE, tRNS, gAMA, cHRM, sRGB, sBIT
  - Skip metadata chunks: tEXt, zTXt, iTXt, eXIf, iCCP

- **TIFF/EXIF**:
  - Use AddImageDataHash() pattern with offset/size pairs
  - Handle StripOffsets/StripByteCounts
  - Handle TileOffsets/TileByteCounts  
  - Special cases: JpgFromRaw, OtherImage

- **Video (MOV/MP4)**:
  - Hash 'mdat' atoms containing media data
  - Process based on handler type ('vide', 'soun')
  - Use stco/co64 chunk offset tables

- **RIFF-based**:
  - AVI: LIST_movi chunks
  - WAV: data chunks
  - WebP: VP8/VP8L/ANIM/ANMF/ALPH chunks

### Implementation Patterns Identified

1. **64KB Chunked Processing**: Prevents memory issues with large files
2. **Lazy Hash Object Creation**: Only when specifically requested
3. **Format Dispatch**: Each format has specific data identification
4. **Offset/Size Pairing**: Critical for TIFF formats
5. **Handler-Based Processing**: Video uses handler type for dispatch

## Remaining Tasks

### High Confidence Implementation Tasks

**1. Core Hash Infrastructure** ✅
```rust
// Implement chunked hash computation matching ExifTool
impl ImageDataHasher {
    fn hash_data_stream(&self, reader: impl Read, hasher: &mut dyn Digest) -> Result<u64> {
        let mut buffer = vec![0u8; 65536]; // 64KB chunks like ExifTool
        let mut total = 0u64;
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
            total += n as u64;
        }
        Ok(total)
    }
}
```

**2. Format-Specific Handlers** ✅
- Implement discovered patterns for each format
- Use generated isImageData lookups
- Follow ExifTool's exact data selection

**3. Empty Hash Detection** ✅
```rust
// Use generated constant table
fn is_empty_hash(hash: &str, algorithm: HashAlgorithm) -> bool {
    EMPTY_HASHES.get(&algorithm).map_or(false, |empty| empty == hash)
}
```

**4. TIFF/EXIF Implementation** ✅
- Port AddImageDataHash logic from WriteExif.pl
- Handle OffsetPair relationships
- Process multiple strips/tiles

### Tasks Requiring Additional Research

**1. HEIC/AVIF Support** 🔍
- Uses `isImageData{$type}` check in QuickTime.pm
- Need to understand 'av01' box structure
- Research extent-based data assembly

**2. CR3 Format Handling** 🔍
- References A100DataOffset special case
- Need to understand Canon's CR3 structure
- May require processor_registry integration

**3. Video Fragment Processing** 🔍
- Parse stco/co64/stsc/stts tables
- Handle fragmented MP4 (fMP4)
- Understand sample-to-chunk mapping

**4. Panasonic RAW Handling** 🔍
- Research NotRealPair hack (size=999999999)
- Understand EOF data storage pattern

## Prerequisites

### Codegen Infrastructure Updates

**1. Create isImageData Extractor**:
```json
// codegen/config/Jpeg2000_pm/isImageData.json
{
  "description": "JP2/JXL image data box types",
  "hash_name": "%isImageData",
  "key_type": "string",
  "value_type": "bool"
}
```

**2. Create IsImageData Tag Extractor**:
- New extractor to find tags with `IsImageData => 1`
- Extract OffsetPair relationships
- Generate tag ID to property mappings

**3. Empty Hash Constants Generator**:
- Extract the three hash values from ExifTool.pm
- Generate by algorithm type

### Existing Dependencies
- ✅ Binary data extraction (from previous milestones)
- ✅ Format parsing infrastructure
- ✅ Streaming I/O patterns

## Testing Strategy

### Unit Tests
- Hash consistency across metadata changes
- Algorithm selection (MD5/SHA256/SHA512)
- Empty hash detection
- Chunked processing with various sizes

### Integration Tests
- Compare with `exiftool -ImageDataHash` output
- Test each supported format
- Large file handling (>4GB videos)
- Corrupted file handling

### Format-Specific Test Cases
- JPEG with embedded thumbnails
- PNG with ancillary chunks
- TIFF with multiple strips
- Video with multiple streams
- HEIC with multiple images

## Success Criteria & Quality Gates

### Core Requirements
- ✅ Hash values match ExifTool exactly
- ✅ Support MD5, SHA256, SHA512
- ✅ Handle all mainstream formats
- ✅ 64KB streaming prevents memory issues

### Performance Targets
- Within 2x of ExifTool speed
- Memory usage <100MB for any file size
- Linear time complexity with file size

### Compatibility Requirements
- Identical hash values to ExifTool
- Handle same edge cases
- Support same format variants

## Gotchas & Tribal Knowledge

### ExifTool Implementation Quirks

**1. Camera-Specific Workarounds**:
- A200 stores StripOffsets in wrong byte order (requires swap)
- JpgFromRaw location varies: SubIFD (NEF/NRW), IFD2 (PEF)
- Some cameras use NotRealPair (data to EOF)

**2. Format-Specific Edge Cases**:
- JPEG in TIFF: Check for SOD marker in JP2
- PNG: Uppercase first letter = critical chunk
- Video: Must check handler type before processing

**3. Processing Order Critical**:
- Must process offset/size pairs atomically
- Video requires chunk offset table parsing first
- TIFF strips must be processed in order

### Implementation Warnings

**1. Never Hardcode Data Identification**:
- MUST use codegen for isImageData tables
- Manual maintenance will drift from ExifTool
- New formats added monthly to ExifTool

**2. Hash Object Lifecycle**:
- Create only when requested
- Check ImageDataHash option first
- Destroy after file processing

**3. Error Handling Patterns**:
- Seek errors are warnings, not fatal
- Continue processing other data on errors
- Report total bytes hashed even with errors

### Performance Considerations
- 64KB chunk size is optimal (ExifTool tested)
- Avoid reading entire file into memory
- Skip hash if not requested (lazy init)

## Implementation Notes

### Key Files to Study
- `Writer.pl:7085-7108` - Core ImageDataHash function
- `WriteExif.pl:AddImageDataHash` - TIFF/EXIF pattern
- `QuickTimeStream.pl:ProcessSamples` - Video handling
- Format modules for `%isImageData` definitions

### Testing Resources
- ExifTool test suite has hash test cases
- Use `exiftool -v3 -ImageDataHash` to see processing
- Compare with `-j` output for exact values

This implementation adds forensic-grade content verification to exif-oxide, matching ExifTool's trusted behavior while leveraging Rust's performance and safety.