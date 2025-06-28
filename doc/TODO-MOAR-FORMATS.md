# TODO: Enhanced HEIF/HEIC, RAW, and Video Format Support

**Implementation Guide for New Engineers**

---

## Executive Summary

exif-oxide currently supports 26 file formats with excellent performance (sub-10ms parsing), but has significant gaps in HEIF/HEIC, RAW image formats, and video metadata support. This document provides a comprehensive implementation plan to expand support while maintaining ExifTool compatibility and performance standards.

**Critical Success Factors:**
- **ExifTool is Gospel**: Follow `third-party/exiftool/` implementations exactly
- **Zero Performance Regression**: Maintain sub-10ms parsing for existing formats  
- **Table-Driven Architecture**: Use proven sync extractor patterns from existing manufacturers
- **Comprehensive Testing**: 100% compatibility with ExifTool outputs

---

## Current State Analysis

### Processing Flow (Start Here!)

Understanding the complete data flow is essential:

1. **Entry Point**: `src/main.rs:722-732` calls `find_all_metadata_segments()`
2. **Central Dispatch**: `src/core/mod.rs:96-188` routes by file format
3. **Format Parsers**: Each format has dedicated parser (`src/core/heif.rs`, `src/core/tiff.rs`, etc.)
4. **Metadata Extraction**: IFD parsing in `src/core/ifd.rs` converts binary to `ExifValue`
5. **Value Processing**: `src/core/print_conv.rs` applies human-readable conversions

### What's Working Well ✅

**HEIF/HEIC Foundation**:
- Format detection working (`src/detection/mod.rs:95-129`)
- Basic HEIF parser exists (`src/core/heif.rs`) 
- QuickTime atom navigation functional
- TIFF header detection for EXIF data

**RAW Format Coverage**:  
- TIFF-based RAW formats work: CR2, NEF, ARW via `src/core/tiff.rs:247`
- Canon CR2 detection solid (`src/detection/mod.rs:249-254`)
- Universal TIFF parser handles standard EXIF data

**Video Container Support**:
- MP4/MOV structure parsing (`src/core/containers/quicktime.rs`)
- Atom navigation and metadata location
- Basic EXIF extraction from video files

### Critical Gaps 🚨

**HEIF/HEIC Limitations** (High Priority):
```rust
// src/core/mod.rs:249-266 - Basic atom search only
FileType::HEIF | FileType::HEIC | FileType::AVIF => {
    if let Some(segment) = heif::find_exif_atom(reader)? {
        // Only finds basic EXIF - missing comprehensive metadata
    }
}
```

**Missing Features:**
- No HEIF sequence support (HEICS/HEIFS for video-like formats)
- Missing Apple Live Photo detection and metadata
- No `iinf`/`iloc` parsing for multiple metadata items
- AVIF animation metadata not extracted

**RAW Format Gaps** (Critical):
```rust
// src/core/mod.rs:296-321 - 19 formats return Ok(None)
FileType::ARQ | FileType::SRF | FileType::RAW | FileType::RWL 
| FileType::ThreeFR | FileType::FFF | FileType::IIQ | FileType::GPR 
| FileType::ERF | FileType::DCR | FileType::K25 | FileType::KDC 
| FileType::MEF | FileType::MRW | FileType::SRW | FileType::NRW => {
    // No metadata extraction implemented
    Ok(None)
}
```

**Video Metadata Deficiencies**:
- GPMF (GoPro metadata) referenced but not implemented (`src/core/mod.rs:164-166`)
- Basic MP4/MOV parsing missing video-specific tags
- No drone metadata standards support
- Missing duration, codec, resolution extraction

---

## Phase 1: Enhanced HEIF/HEIC Support (2-3 weeks)

### 1.1 Comprehensive HEIF Atom Parsing

**Objective**: Implement full HEIF container parsing following ExifTool's QuickTime.pm algorithm.

**ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm`

**Key Implementation Points**:

```rust
// Enhancement to src/core/heif.rs - Add these functions:

/// Parse iinf (Item Info) atom to find all metadata items
fn parse_item_info_atom<R: Read + Seek>(
    reader: &mut R, 
    iinf_offset: u64, 
    iinf_size: u64
) -> Result<Vec<ItemInfo>> {
    // ExifTool algorithm: QuickTime.pm lines 4500-4600
    // Parse item_count, then iterate through item_info_entry boxes
    // Look for item_type == "Exif" or "mime" with content_type "application/rdf+xml"
}

/// Find Primary Item (pitm) to identify main image
fn find_primary_item<R: Read + Seek>(
    reader: &mut R,
    meta_offset: u64,
    meta_size: u64  
) -> Result<Option<u16>> {
    // ExifTool: QuickTime.pm lines 4400-4450
    // Parse pitm atom to get primary item ID
}

/// Parse Item Properties (iprp) for image characteristics  
fn parse_item_properties<R: Read + Seek>(
    reader: &mut R,
    iprp_offset: u64,
    iprp_size: u64
) -> Result<Vec<ImageProperty>> {
    // ExifTool: QuickTime.pm lines 4650-4750
    // Extract rotation, color profile, dimensions
}
```

**Testing Strategy**:
```bash
# Validate against ExifTool for HEIF files
./exiftool -struct -json test.heic > exiftool_output.json
cargo run -- test.heic > exif_oxide_output.json  
# Compare outputs - must match exactly
```

### 1.2 Apple Live Photo Support

**Objective**: Detect and extract Live Photo metadata from HEIF containers.

**New Module**: `src/core/live_photo.rs`

```rust
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/QuickTime.pm"]

/// Apple Live Photo detection and metadata extraction
pub struct LivePhoto {
    pub still_image_item: u16,
    pub motion_item: u16,
    pub duration: Option<f64>,
    pub auto_loop: bool,
}

/// Detect Live Photo HEIF files by checking for motion items
pub fn detect_live_photo<R: Read + Seek>(
    reader: &mut R,
    item_infos: &[ItemInfo]
) -> Result<Option<LivePhoto>> {
    // ExifTool algorithm: Look for "acci" item type indicating Live Photo
    // Extract motion photo relationships from iref atom
}
```

**Integration Point**: `src/core/heif.rs:113` - Add Live Photo detection after basic parsing.

### 1.3 HEIF Sequence Support

**Objective**: Support HEICS/HEIFS video-like sequence formats.

**Key Changes**:
```rust
// src/detection/mod.rs:115-116 - Already detects sequence formats
b"hevc" => FileType::HEICS, // HEIC sequence  
b"msf1" => FileType::HEIFS, // HEIF sequence

// src/core/heif.rs - Add sequence-specific parsing
/// Parse HEIF sequences with timeline metadata
pub fn parse_heif_sequence<R: Read + Seek>(
    reader: &mut R
) -> Result<Option<SequenceMetadata>> {
    // Look for "meco" (multiple entity container) atoms
    // Extract frame timing and sequence properties
}
```

**Tribal Knowledge**: HEIF sequences are essentially video containers using HEIF structure. Follow QuickTime.pm sequence parsing patterns.

---

## Phase 2: Comprehensive RAW Format Support (3-4 weeks) 

### 2.1 Manufacturer-Specific RAW Parsers

**Critical Understanding**: Most RAW formats use manufacturer-specific binary structures that require dedicated parsers following ExifTool's ProcessBinaryData pattern.

**Architecture Pattern**: Follow existing maker note structure:

```rust
// src/maker/sony_raw.rs - New module
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Sony.pm"]

use crate::core::types::ExifValue;
use crate::error::Result;
use std::collections::HashMap;

pub struct SonyRawParser;

impl SonyRawParser {
    /// Parse Sony ARW files using ProcessBinaryData equivalent
    pub fn parse_arw<R: Read + Seek>(
        reader: &mut R
    ) -> Result<HashMap<u16, ExifValue>> {
        // ExifTool: Sony.pm ARW processing
        // 1. Find Sony-specific IFD structure
        // 2. Parse embedded JPEG previews
        // 3. Extract lens correction data
        // 4. Process Sony-specific tags
    }
}
```

**Priority Order** (based on format popularity):
1. **Sony RAW** (ARW, SR2, SRF) - `Sony.pm` in ExifTool
2. **Olympus RAW** (ORF) - `Olympus.pm`  
3. **Fujifilm RAW** (RAF) - `FujiFilm.pm`
4. **Panasonic RAW** (RW2) - `Panasonic.pm`
5. **Samsung RAW** (SRW) - `Samsung.pm`

**Integration Pattern**:
```rust
// src/core/mod.rs:296-321 - Replace Ok(None) with:
FileType::ARW | FileType::SR2 => {
    sony_raw::SonyRawParser::parse_arw(reader)
},
FileType::ORF => {
    olympus_raw::OlympusRawParser::parse_orf(reader)  
},
// ... continue for each format
```

### 2.2 Sync Tool Enhancement for RAW Formats

**Objective**: Extend `src/bin/exiftool_sync/` to extract RAW format tables.

**New Extractor**: `src/bin/exiftool_sync/extractors/raw_formats.rs`

```rust
/// Extract ProcessBinaryData tables from RAW format modules
pub fn extract_raw_formats() -> Result<()> {
    // Parse Sony.pm, Olympus.pm, etc. for:
    // 1. ProcessBinaryData table definitions
    // 2. Tag ID to name mappings  
    // 3. Format conversion specifications
    // 4. Binary data structure layouts
    
    // Generate src/tables/sony_raw_tags.rs, etc.
}
```

**Command Integration**:
```bash
# Add to existing sync workflow
cargo run --bin exiftool_sync extract raw-formats
# Or include in extract-all
```

### 2.3 Advanced RAW Features

**Binary Preview Extraction**: 
```rust
// src/binary.rs - Add RAW preview support
/// Extract large embedded JPEG from RAW files
pub fn extract_raw_preview(
    format: FileType,
    data: &[u8]
) -> Result<Option<Vec<u8>>> {
    match format {
        FileType::ARW => sony_raw::extract_preview(data),
        FileType::ORF => olympus_raw::extract_preview(data),
        // Follow ExifTool preview extraction algorithms
    }
}
```

**Tribal Knowledge**: RAW files often contain multiple preview sizes. ExifTool extracts the largest by default, but supports size selection. Maintain this compatibility.

---

## Phase 3: Enhanced Video Metadata Support (2-3 weeks)

### 3.1 GPMF Implementation  

**Critical Context**: GPMF (GoPro Metadata Format) is referenced in current code but not implemented. It's essential for action camera and drone video metadata.

**New Module**: `src/gpmf/mod.rs`

```rust
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/GoPro.pm"]

/// GPMF stream parser following ExifTool's ProcessGoPro
pub struct GpmfParser {
    // GPMF uses nested KLV (Key-Length-Value) structure
}

impl GpmfParser {
    /// Parse GPMF stream from MP4 mdat atom
    pub fn parse_stream(&self, data: &[u8]) -> Result<Vec<GpmfSample>> {
        // ExifTool algorithm: GoPro.pm lines 200-500
        // 1. Find "GP\x06\0" markers in mdat atom  
        // 2. Parse KLV structure
        // 3. Extract GPS, accelerometer, gyroscope data
        // 4. Build timeline of telemetry samples
    }
}

#[derive(Debug)]
pub struct GpmfSample {
    pub timestamp: f64,
    pub gps_coords: Option<(f64, f64, f64)>, // lat, lon, alt
    pub accelerometer: Option<(f32, f32, f32)>,
    pub gyroscope: Option<(f32, f32, f32)>,
}
```

**Integration Point**: `src/core/mod.rs:149-168` - MP4/MOV processing

```rust
FileType::MP4 | FileType::MOV => {
    // ... existing code ...
    
    // Add GPMF extraction
    if let Some(gpmf_data) = gpmf::find_gpmf_stream(reader)? {
        for gpmf_segment in gpmf_data {
            collection.gpmf.push(MetadataSegment {
                data: gpmf_segment.data,
                offset: gpmf_segment.offset, 
                source_format: format,
                metadata_type: MetadataType::Gpmf,
            });
        }
    }
}
```

### 3.2 Enhanced MP4/MOV Metadata

**Objective**: Extract comprehensive video metadata beyond basic EXIF.

**Enhancement**: `src/core/containers/quicktime.rs`

```rust
/// Video-specific metadata structure
#[derive(Debug)]
pub struct VideoMetadata {
    pub duration: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>, 
    pub resolution: Option<(u32, u32)>,
    pub frame_rate: Option<f64>,
    pub bitrate: Option<u32>,
}

/// Extract video track information from moov atom
fn parse_video_tracks<R: Read + Seek>(
    reader: &mut R,
    moov_offset: u64,
    moov_size: u64
) -> Result<Vec<VideoMetadata>> {
    // ExifTool: QuickTime.pm track parsing
    // 1. Find trak atoms within moov
    // 2. Parse mdia->minf->stbl for codec info
    // 3. Extract duration from mvhd atom
    // 4. Calculate resolution from tkhd atom
}
```

### 3.3 Drone Metadata Standards

**New Modules**:
- `src/maker/dji_video.rs` - DJI drone video metadata
- `src/standards/xmp_drone.rs` - Drone XMP schema support

```rust
// DJI video metadata extraction
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/DJI.pm"]

/// Extract DJI drone telemetry from video files
pub fn extract_dji_telemetry<R: Read + Seek>(
    reader: &mut R
) -> Result<Option<DroneMetadata>> {
    // ExifTool: DJI.pm video processing
    // Look for DJI-specific atoms in MP4 structure
    // Extract flight path, gimbal data, camera settings
}

#[derive(Debug)]
pub struct DroneMetadata {
    pub flight_path: Vec<GpsPoint>,
    pub gimbal_pitch: Option<f32>,
    pub gimbal_yaw: Option<f32>,
    pub camera_iso: Option<u32>,
    pub camera_shutter: Option<f32>,
}
```

---

## Technical Implementation Guidelines

### Essential Patterns

**1. ExifTool Source Attribution** (Mandatory):
```rust
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/ModuleName.pm"]
```

**2. Error Handling Pattern**:
```rust
// Continue parsing on non-fatal errors (ExifTool behavior)
match parse_tag(&data) {
    Ok(tag) => tags.insert(tag.id, tag.value),
    Err(e) => {
        // Log warning but continue
        eprintln!("Warning: Failed to parse tag: {}", e);
        continue;
    }
}
```

**3. Binary Data Safety**:
```rust
// ALWAYS bounds check before reading
if offset + size > data.len() {
    return Err(ExifError::InvalidOffset(offset));
}
let value = &data[offset..offset + size];
```

**4. Table-Driven Architecture**:
```rust
// Follow existing pattern in src/tables/
pub static RAW_TAGS: &[(u16, TagInfo)] = &[
    (0x0001, TagInfo {
        name: "SonyModelID", 
        format: ExifFormat::U16,
        print_conv: PrintConvId::SonyModel,
    }),
    // ... generated from ExifTool sync
];
```

### Performance Considerations

**Memory Management**:
- Use streaming parsers for video files (don't load entire file)
- Implement lazy evaluation for complex metadata
- Pre-allocate collections with reasonable capacity

**Processing Efficiency**:  
- Early termination when metadata sections end
- Cache atom structures for repeated access
- Minimize string allocations in hot paths

**Target Metrics**:
- **Images**: Maintain sub-10ms parsing
- **Video**: Target sub-50ms for metadata (excluding full GPMF streams)
- **Memory**: Peak usage under 200MB for largest files

### Testing Strategy

**Test File Requirements**:
- One sample file per supported RAW format  
- HEIF files with Live Photos and sequences
- MP4/MOV files with GPMF and comprehensive metadata
- Video files from major drone manufacturers

**Validation Process**:
```bash
# For each new format, validate against ExifTool
./exiftool -struct -json test_file.ext > reference.json
cargo run -- test_file.ext > output.json

# Automated comparison script  
python scripts/compare_outputs.py reference.json output.json

# Performance validation
cargo bench -- format_name
```

**Regression Testing**:
- Run full test suite after each phase
- Validate no performance degradation on existing formats
- Ensure memory usage stays within bounds

---

## ExifTool Synchronization Workflow

### Using Sync Tools

**Current Tools** (Already Available):
```bash
# Extract all components
cargo run --bin exiftool_sync extract-all

# Check what ExifTool files need parsing
cargo run --bin exiftool_sync scan

# Compare ExifTool versions
cargo run --bin exiftool_sync diff 12.65 12.66
```

**New Extractors to Implement**:

**1. RAW Format Tables**:
```bash
cargo run --bin exiftool_sync extract raw-formats
# Generates: src/tables/sony_raw_tags.rs, olympus_raw_tags.rs, etc.
```

**2. Video Metadata Tables**:
```bash  
cargo run --bin exiftool_sync extract video-formats
# Generates: src/tables/quicktime_tags.rs, gpmf_tags.rs
```

**3. HEIF/Container Tables**:
```bash
cargo run --bin exiftool_sync extract container-formats  
# Generates: src/tables/heif_tags.rs, mp4_tags.rs
```

### Sync Tool Architecture

**Extractor Pattern** (Follow `src/bin/exiftool_sync/extractors/exif_tags.rs`):

```rust
// src/bin/exiftool_sync/extractors/sony_raw.rs
pub struct SonyRawExtractor;

impl Extractor for SonyRawExtractor {
    fn extract(&self) -> Result<()> {
        // 1. Parse third-party/exiftool/lib/Image/ExifTool/Sony.pm
        // 2. Extract ProcessBinaryData tables  
        // 3. Generate src/tables/sony_raw_tags.rs
        // 4. Follow exact ExifTool tag definitions
    }
}
```

**Build Integration** (Update `build.rs`):
```rust
// Ensure sync extractors run before compilation
if std::env::var("CARGO_FEATURE_SYNC").is_ok() {
    run_sync_extractors()?;
}
```

---

## Success Metrics and Validation

### Phase 1 Success Criteria (HEIF/HEIC)

**Functionality**:
- [ ] Extract all metadata items from multi-item HEIF files
- [ ] Detect and parse Apple Live Photos correctly
- [ ] Support HEICS/HEIFS sequence metadata
- [ ] 95% tag compatibility with ExifTool HEIF output

**Performance**:
- [ ] Sub-15ms parsing for typical HEIF files
- [ ] No regression in existing JPEG/PNG performance
- [ ] Memory usage under 50MB for largest HEIF files

**Testing**:
- [ ] 20+ HEIF test files with comprehensive metadata
- [ ] Automated comparison with ExifTool outputs
- [ ] Live Photo samples from multiple iOS versions

### Phase 2 Success Criteria (RAW Formats)

**Functionality**:
- [ ] Support 15+ additional RAW formats
- [ ] Extract manufacturer-specific metadata
- [ ] Binary preview extraction for all formats
- [ ] 100% pass rate against ExifTool for implemented formats

**Performance**:  
- [ ] Sub-20ms parsing for typical RAW files
- [ ] Dual-mode parsing (metadata vs full file) working
- [ ] Stream processing for large RAW files

**Testing**:
- [ ] Sample files for each supported RAW format
- [ ] Comprehensive preview extraction tests
- [ ] Memory usage validation for 100MB+ files

### Phase 3 Success Criteria (Video)

**Functionality**:
- [ ] GPMF parsing and telemetry extraction  
- [ ] Comprehensive MP4/MOV video metadata
- [ ] Drone metadata standards support
- [ ] Video timeline and chapter extraction

**Performance**:
- [ ] Sub-50ms for video metadata (excluding GPMF streams)
- [ ] Streaming parser for multi-GB video files
- [ ] GPMF processing under 5 seconds for 1-hour videos

**Testing**:
- [ ] Video samples from GoPro, DJI, phones
- [ ] GPMF timeline accuracy validation
- [ ] Stress testing with large video files

### Overall Project Success

**Code Quality**:
- [ ] 100% ExifTool source attribution
- [ ] Zero unsafe code in new modules
- [ ] Comprehensive error handling and graceful degradation
- [ ] Full integration with existing sync tools

**Documentation**:
- [ ] Updated DESIGN.md with new architecture
- [ ] API documentation for all new modules
- [ ] Performance benchmarks and optimization notes
- [ ] Troubleshooting guide for new formats

**Integration**:
- [ ] All formats work through central dispatcher
- [ ] Binary extraction supports new formats
- [ ] CLI tool handles all new metadata types
- [ ] No breaking changes to existing APIs

---

## Troubleshooting and Common Pitfalls

### HEIF/HEIC Issues

**Problem**: HEIF files not extracting all metadata
**Solution**: Check `iinf` atom parsing - ensure all item types are processed

**Problem**: Live Photo detection failing  
**Solution**: Verify `acci` item type detection and `iref` relationship parsing

**Problem**: Sequence metadata missing
**Solution**: Look for `meco` atoms and timeline tracks in HEIF sequences

### RAW Format Issues

**Problem**: Manufacturer tags not extracted
**Solution**: Verify ProcessBinaryData table extraction from sync tools

**Problem**: Preview extraction failing
**Solution**: Check for multiple preview formats and size selection logic

**Problem**: Performance degradation with large RAW files
**Solution**: Implement streaming mode and lazy evaluation for complex structures

### Video Metadata Issues

**Problem**: GPMF data not found
**Solution**: Ensure proper `mdat` atom parsing and "GP\x06\0" marker detection

**Problem**: Video track information missing
**Solution**: Verify `moov`/`trak`/`mdia` atom traversal and codec identification

**Problem**: Memory usage too high for video files
**Solution**: Implement streaming parser instead of loading entire file

### General Implementation Issues

**Problem**: ExifTool compatibility failures
**Solution**: Always validate against ExifTool output and check source attribution

**Problem**: Performance regression
**Solution**: Profile with `cargo bench` and optimize hot paths

**Problem**: Build errors with new sync extractors
**Solution**: Ensure extractor follows established pattern and integrates with build.rs

---

## Getting Started Checklist

### Prerequisites
- [ ] Read `doc/DESIGN.md` completely (focus on Critical Implementation Insights)
- [ ] Read `CLAUDE.md` development principles
- [ ] Understand ExifTool source attribution requirements
- [ ] Review existing sync extractor pattern in `src/bin/exiftool_sync/extractors/`

### Development Environment
- [ ] Build current exif-oxide: `cargo build && cargo test`
- [ ] Verify ExifTool installation: `./third-party/exiftool/exiftool -ver`
- [ ] Run sync tools: `cargo run --bin exiftool_sync extract-all`
- [ ] Benchmark baseline: `cargo bench`

### First Implementation Steps
1. **Choose Phase 1, 2, or 3** based on your interests/experience
2. **Start with sync extractor** for your chosen area (follow existing patterns)
3. **Implement basic parser** with minimal functionality
4. **Add comprehensive tests** before expanding features
5. **Validate against ExifTool** at each step
6. **Performance test** regularly to catch regressions early

### Support Resources
- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/`
- **Architecture Reference**: `doc/DESIGN.md`
- **Development Patterns**: `CLAUDE.md`
- **Existing Implementations**: `src/maker/canon.rs`, `src/tables/exif_tags.rs`
- **Test Framework**: `tests/` directory with real-world examples

---

**Remember**: ExifTool compatibility is non-negotiable. When in doubt, check ExifTool's implementation and copy it exactly. The goal is 10-50x performance improvement while maintaining 100% functional compatibility.