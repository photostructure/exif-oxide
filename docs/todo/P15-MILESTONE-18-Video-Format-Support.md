# Milestone 18: Video Format Support

**Duration**: 4-5 weeks  
**Goal**: Add metadata extraction support for smartphone and prosumer video formats

## Overview

Video metadata extraction is essential for digital asset management workflows. This milestone implements support for the most common video formats found in smartphone and prosumer camera workflows, focusing on the formats specified in `docs/MIMETYPES.md`.

## Background Analysis

**ExifTool's Video Processing Complexity**:

- **QuickTime.pm**: 10,638 lines handling MOV/MP4/M4V/3GP family
- **80+ subroutines**: Complex atom parsing, track management, codec handling
- **41 tag tables**: Multiple metadata systems (iTunes, Classic QT, modern MP4)
- **Hierarchical structure**: Recursive atom parsing with state management

**Key Insight**: Unlike simpler formats (AVI, MPEG), QuickTime/MP4 requires substantial upfront infrastructure investment due to its sophisticated container architecture.

## Format Scope: MIMETYPES.md Video Formats

### Primary Formats (Complex)

- **MP4** (`video/mp4`) - MPEG-4 Part 14, includes Insta360 .insv variant
- **QuickTime** (`video/quicktime`) - Apple MOV, QT containers
- **HEIF Video** (`video/heif`) - High efficiency video sequences (.heic/.heif)

### Secondary Formats (Moderate)

- **3GPP/3GPP2** (`video/3gpp`, `video/3gpp2`) - Mobile formats (.3gp, .3g2)
- **M4V** (`video/x-m4v`) - iTunes video variant
- **MPEG-TS** (`video/m2ts`) - Transport stream (.mts, .m2ts, .ts)

### Simpler Formats

- **AVI** (`video/x-msvideo`) - Microsoft RIFF-based format
- **MPEG** (`video/mpeg`) - Basic MPEG streams (.m2v, .mpeg, .mpg)
- **Matroska** (`video/x-matroska`) - MKV containers
- **WebM** (`video/webm`) - Google video format
- **WMV/ASF** (`video/x-ms-wmv`, `video/x-ms-asf`) - Microsoft formats

### Specialized Formats

- **Canon CRM** (`video/x-canon-crm`) - Canon RAW Movie
- **MNG** (`video/mng`) - Multiple-image Network Graphics

## Implementation Strategy

### Phase 1: QuickTime/MP4 Foundation (Week 1-2)

**Critical Decision**: QuickTime/MP4 cannot be implemented incrementally due to architectural requirements.

**Atom Parsing Infrastructure**:

```rust
pub struct VideoProcessor {
    atom_parser: AtomParser,
    track_manager: TrackManager,
    metadata_extractors: HashMap<VideoFormat, Box<dyn MetadataExtractor>>,
}

// Core atom parsing - cannot be simplified
pub struct AtomParser {
    // Handle hierarchical atom structure
    // [4-byte size][4-byte type][variable data with sub-atoms]
}

impl AtomParser {
    pub fn parse_atoms(&mut self, data: &[u8]) -> Result<Vec<Atom>> {
        let mut atoms = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let atom = self.parse_atom(data, offset)?;
            offset += atom.size;

            // Recursive parsing for container atoms
            if atom.has_children() {
                atom.children = self.parse_atoms(&atom.data)?;
            }

            atoms.push(atom);
        }

        Ok(atoms)
    }
}
```

**File Type Detection**:

```rust
// ExifTool's brand detection patterns
pub fn detect_video_format(atoms: &[Atom]) -> Result<VideoFormat> {
    // Find 'ftyp' atom for format identification
    if let Some(ftyp_atom) = atoms.iter().find(|a| a.atom_type == b"ftyp") {
        let brand = &ftyp_atom.data[0..4];

        match brand {
            b"isom" | b"mp41" | b"mp42" => Ok(VideoFormat::MP4),
            b"qt  " => Ok(VideoFormat::QuickTime),
            b"3gp4" | b"3gp5" => Ok(VideoFormat::ThreeGP),
            b"M4V " | b"M4VH" => Ok(VideoFormat::M4V),
            b"heic" | b"mif1" => Ok(VideoFormat::HEIF),
            // 200+ brand variants in ExifTool
            _ => Err(ExifError::UnknownVideoFormat),
        }
    } else {
        Err(ExifError::MissingFtypAtom)
    }
}
```

### Phase 2: Metadata Extraction Systems (Week 2-3)

**Multiple Metadata Systems** (ExifTool handles 4 different systems):

```rust
pub trait MetadataExtractor {
    fn extract_metadata(&self, atoms: &[Atom], reader: &mut ExifReader) -> Result<()>;
}

// System 1: Classic QuickTime User Data
pub struct QuickTimeUserDataExtractor;
impl MetadataExtractor for QuickTimeUserDataExtractor {
    fn extract_metadata(&self, atoms: &[Atom], reader: &mut ExifReader) -> Result<()> {
        // Process 'udta' atoms
        // Extract copyright, creation date, camera info
        for atom in atoms.iter().filter(|a| a.atom_type == b"udta") {
            self.process_user_data_atom(atom, reader)?;
        }
        Ok(())
    }
}

// System 2: iTunes Metadata (ItemList)
pub struct ITunesMetadataExtractor;
impl MetadataExtractor for ITunesMetadataExtractor {
    fn extract_metadata(&self, atoms: &[Atom], reader: &mut ExifReader) -> Result<()> {
        // Process 'ilst' atoms
        // Extract title, artist, album, genre, rating
        for atom in atoms.iter().filter(|a| a.atom_type == b"ilst") {
            self.process_item_list_atom(atom, reader)?;
        }
        Ok(())
    }
}

// System 3: Modern MP4 Meta Atoms
pub struct MP4MetadataExtractor;

// System 4: Codec-Specific Configuration
pub struct CodecConfigExtractor;
```

**Track Management** (Essential for video metadata):

```rust
pub struct TrackManager {
    tracks: HashMap<u32, TrackInfo>,
    current_track: Option<u32>,
}

pub struct TrackInfo {
    track_id: u32,
    track_type: TrackType, // Video, Audio, Metadata, Timecode
    handler: String,        // Track handler type
    language: String,       // ISO 639 language code
    duration: Duration,
    dimensions: Option<(u32, u32)>,
}

impl TrackManager {
    pub fn process_track_header(&mut self, tkhd_atom: &Atom) -> Result<()> {
        // Extract track dimensions, duration, creation time
        // Handle track enable/disable flags
        // Manage track hierarchies for complex videos
    }
}
```

### Phase 3: Simpler Format Support (Week 3-4)

**AVI Format** (RIFF-based, much simpler):

```rust
pub struct AVIProcessor;
impl MetadataExtractor for AVIProcessor {
    fn extract_metadata(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // RIFF chunk structure - much simpler than atoms
        // Find 'LIST' chunks containing metadata
        // Extract AVI header, stream headers, info chunks
        self.process_riff_chunks(data, reader)
    }
}
```

**MPEG Transport Stream**:

```rust
pub struct MPEGTSProcessor;
impl MetadataExtractor for MPEGTSProcessor {
    fn extract_metadata(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Stream-based processing
        // Extract PMT/PAT tables for stream info
        // Limited metadata compared to container formats
        self.process_transport_stream(data, reader)
    }
}
```

### Phase 4: Integration and Specialized Formats (Week 4-5)

**Format Dispatcher**:

```rust
impl VideoProcessor {
    pub fn process_video_file(&mut self, file_type: FileType, reader: &mut ExifReader) -> Result<()> {
        match file_type {
            // Complex formats requiring full atom infrastructure
            FileType::MP4 | FileType::QuickTime | FileType::M4V | FileType::ThreeGP => {
                let atoms = self.atom_parser.parse_file(reader)?;
                let format = detect_video_format(&atoms)?;
                self.extract_quicktime_metadata(format, &atoms, reader)
            },

            // Simpler formats with different processing
            FileType::AVI => self.process_avi_metadata(reader),
            FileType::MPEG => self.process_mpeg_metadata(reader),
            FileType::MKV => self.process_matroska_metadata(reader),

            // Specialized formats
            FileType::CanonCRM => self.process_canon_crm(reader),

            _ => Err(ExifError::UnsupportedVideoFormat),
        }
    }
}
```

**HEIF Video Support**:

```rust
pub struct HEIFVideoProcessor;
impl MetadataExtractor for HEIFVideoProcessor {
    fn extract_metadata(&self, atoms: &[Atom], reader: &mut ExifReader) -> Result<()> {
        // HEIF containers can contain both images and video sequences
        // Same file extensions (.heic/.heif) used for both
        // Requires content inspection to determine media type

        self.detect_heif_content_type(atoms, reader)?;
        self.extract_heif_sequence_metadata(atoms, reader)
    }
}
```

## Success Criteria

### Core Requirements

- [ ] **Atom Parsing**: Robust QuickTime/MP4 atom parsing with recursion support
- [ ] **Format Detection**: Accurate detection of video formats from MIMETYPES.md
- [ ] **Track Management**: Extract track information (video/audio/metadata tracks)
- [ ] **Multiple Metadata Systems**: Support for iTunes, QuickTime, MP4 metadata
- [ ] **Basic Video Info**: Duration, dimensions, codec information, creation date

### Validation Tests

- Process smartphone videos (iPhone MOV, Android MP4)
- Handle action camera footage (GoPro MP4)
- Extract metadata from social media exports
- Process prosumer camera videos (Canon, Sony, Panasonic)

## Implementation Boundaries

### Goals (Milestone 18)

- Basic video metadata extraction for smartphone/prosumer workflows
- Duration, dimensions, creation date, camera info
- Track information and codec details
- HEIF/HEIC video sequence support

### Non-Goals (Future Milestones)

- **Advanced video analysis**: Frame rate analysis, quality metrics
- **Video preview extraction**: Thumbnail generation from video frames
- **Complex codec metadata**: Detailed codec configuration parameters
- **Professional metadata**: Timecode, chapter markers, subtitle tracks
- **Video writing**: Metadata modification in video files

## Dependencies and Prerequisites

### Milestone Prerequisites

- **Milestone 16**: File type detection for video format identification

### Technical Dependencies

- **Large file handling**: Videos can be multi-gigabyte files
- **Streaming support**: Process metadata without loading entire video
- **Memory management**: Efficient atom parsing for large containers

## Risk Mitigation

### Complexity Risk: QuickTime/MP4

- **Risk**: Atom parsing infrastructure represents 60-70% of implementation complexity
- **Mitigation**: Cannot be avoided - QuickTime/MP4 requires complete implementation
- **Evidence**: ExifTool's 10,638 lines show this is inherently complex

### Performance Risk: Large Video Files

- **Risk**: Multi-gigabyte video files could cause memory issues
- **Mitigation**: Stream-based atom parsing, extract only metadata atoms
- **Pattern**: Follow ExifTool's LargeFileSupport approach

### Format Fragmentation Risk

- **Risk**: Many video format variants and codec combinations
- **Mitigation**: Focus on mainstream formats first, add variants incrementally

## Comparison with Other Formats

| Format Family | Lines of Code | Complexity | Container Type     |
| ------------- | ------------- | ---------- | ------------------ |
| QuickTime/MP4 | 10,638        | Very High  | Hierarchical atoms |
| AVI (RIFF)    | 2,269         | Moderate   | Simple chunks      |
| MPEG          | 735           | Low        | Stream-based       |

## Related Documentation

### Required Reading

- **QuickTime.pm**: 10,638 lines of atom parsing and metadata extraction
- **MIMETYPES.md**: Complete list of video formats to support
- **FILE_TYPES.md**: Video format detection patterns

### Missing Documentation to Create

- **VIDEO_ATOM_PARSING.md**: Deep dive on QuickTime atom structure
- **VIDEO_METADATA_SYSTEMS.md**: Comparison of iTunes vs QuickTime vs MP4 metadata
- **HEIF_VIDEO_DETECTION.md**: Distinguishing HEIF images from video sequences

This milestone establishes video format support as a major capability while acknowledging the inherent complexity of modern video container formats. The infrastructure investment in QuickTime/MP4 parsing enables comprehensive metadata extraction across the entire smartphone and prosumer video ecosystem.
