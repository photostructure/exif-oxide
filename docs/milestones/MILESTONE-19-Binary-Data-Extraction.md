# Milestone 19: Binary Data Extraction (`-b` support)

**Duration**: 2-3 weeks  
**Goal**: Implement comprehensive binary data extraction equivalent to `exiftool -b`

## Overview

Binary data extraction is a fundamental feature that enables users to extract embedded images, thumbnails, color profiles, and other binary metadata from files. This milestone implements the equivalent of ExifTool's `-b` flag with both CLI and streaming API support.

## Background: ExifTool's `-b` Functionality

From ExifTool manual:

> `-b, --b (-binary, --binary): Output requested metadata in binary format without tag names or descriptions. This option is mainly used for extracting embedded images or other binary data.`

**Common Use Cases**:

- Extract JPEG thumbnails from EXIF data
- Save preview images from RAW files
- Extract ICC color profiles
- Save embedded audio/video from image files
- Extract lens correction data and manufacturer-specific binary metadata

## Implementation Strategy

### Phase 1: Core Binary Extraction Infrastructure (Week 1)

**Binary Tag Detection**:

```rust
pub struct BinaryExtractor {
    format_handlers: HashMap<FileType, Box<dyn BinaryHandler>>,
    size_limits: BinarySizeLimits,
}

pub trait BinaryHandler {
    fn extract_binary_tags(&self, reader: &ExifReader) -> Result<Vec<BinaryTag>>;
    fn stream_binary_tag(&self, reader: &ExifReader, tag_name: &str) -> Result<Box<dyn Read>>;
}

#[derive(Debug, Clone)]
pub struct BinaryTag {
    pub name: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub description: String,
    pub data_location: DataLocation,
}

#[derive(Debug, Clone)]
pub enum DataLocation {
    Embedded { offset: u64, size: u64 },
    Referenced { path: PathBuf },
    Computed { generator: String },
}
```

**Streaming API**:

```rust
impl ExifReader {
    /// Extract binary tag data as a stream (memory-efficient for large data)
    pub fn stream_binary_tag<W: Write>(
        &self,
        tag_name: &str,
        writer: &mut W
    ) -> Result<u64> {
        let binary_tag = self.find_binary_tag(tag_name)?;

        match binary_tag.data_location {
            DataLocation::Embedded { offset, size } => {
                self.stream_embedded_data(offset, size, writer)
            },
            DataLocation::Referenced { path } => {
                self.stream_referenced_file(&path, writer)
            },
            DataLocation::Computed { generator } => {
                self.generate_binary_data(&generator, writer)
            },
        }
    }

    /// List all available binary tags in the file
    pub fn list_binary_tags(&self) -> Result<Vec<BinaryTag>> {
        let file_type = self.get_file_type();
        let handler = self.binary_extractor.get_handler(file_type)?;
        handler.extract_binary_tags(self)
    }
}
```

### Phase 2: Format-Specific Binary Handlers (Week 1-2)

**JPEG Binary Handler**:

```rust
pub struct JPEGBinaryHandler;
impl BinaryHandler for JPEGBinaryHandler {
    fn extract_binary_tags(&self, reader: &ExifReader) -> Result<Vec<BinaryTag>> {
        let mut tags = Vec::new();

        // EXIF thumbnail extraction
        if let Some(thumbnail_offset) = reader.get_tag_value("ThumbnailOffset") {
            if let Some(thumbnail_length) = reader.get_tag_value("ThumbnailLength") {
                tags.push(BinaryTag {
                    name: "ThumbnailImage".to_string(),
                    size: thumbnail_length.as_u64().unwrap_or(0),
                    mime_type: Some("image/jpeg".to_string()),
                    description: "EXIF embedded thumbnail".to_string(),
                    data_location: DataLocation::Embedded {
                        offset: thumbnail_offset.as_u64().unwrap_or(0),
                        size: thumbnail_length.as_u64().unwrap_or(0),
                    },
                });
            }
        }

        // ICC Profile extraction
        if let Some(icc_profile) = reader.get_binary_tag("ICC_Profile") {
            tags.push(BinaryTag {
                name: "ICC_Profile".to_string(),
                size: icc_profile.len() as u64,
                mime_type: Some("application/vnd.iccprofile".to_string()),
                description: "Embedded ICC color profile".to_string(),
                data_location: DataLocation::Embedded {
                    offset: icc_profile.offset,
                    size: icc_profile.size,
                },
            });
        }

        Ok(tags)
    }
}
```

**RAW Binary Handler**:

```rust
pub struct RAWBinaryHandler;
impl BinaryHandler for RAWBinaryHandler {
    fn extract_binary_tags(&self, reader: &ExifReader) -> Result<Vec<BinaryTag>> {
        let mut tags = Vec::new();
        let make = reader.get_tag_value("Make").unwrap_or_default();

        match make.as_str() {
            "Canon" => self.extract_canon_binary_tags(reader, &mut tags)?,
            "NIKON CORPORATION" => self.extract_nikon_binary_tags(reader, &mut tags)?,
            "SONY" => self.extract_sony_binary_tags(reader, &mut tags)?,
            _ => self.extract_generic_raw_binary_tags(reader, &mut tags)?,
        }

        Ok(tags)
    }
}

impl RAWBinaryHandler {
    fn extract_canon_binary_tags(&self, reader: &ExifReader, tags: &mut Vec<BinaryTag>) -> Result<()> {
        // Canon CR2/CR3 preview images
        // Multiple preview sizes: thumbnail, medium preview, large preview
        if let Some(preview_image_start) = reader.get_tag_value("PreviewImageStart") {
            if let Some(preview_image_length) = reader.get_tag_value("PreviewImageLength") {
                tags.push(BinaryTag {
                    name: "PreviewImage".to_string(),
                    size: preview_image_length.as_u64().unwrap_or(0),
                    mime_type: Some("image/jpeg".to_string()),
                    description: "Canon RAW preview image".to_string(),
                    data_location: DataLocation::Embedded {
                        offset: preview_image_start.as_u64().unwrap_or(0),
                        size: preview_image_length.as_u64().unwrap_or(0),
                    },
                });
            }
        }

        // Canon lens correction data
        if let Some(lens_correction) = reader.get_binary_tag("LensInfo") {
            tags.push(BinaryTag {
                name: "LensCorrection".to_string(),
                size: lens_correction.len() as u64,
                mime_type: None,
                description: "Canon lens correction data".to_string(),
                data_location: DataLocation::Embedded {
                    offset: lens_correction.offset,
                    size: lens_correction.size,
                },
            });
        }

        Ok(())
    }
}
```

**Video Binary Handler**:

```rust
pub struct VideoBinaryHandler;
impl BinaryHandler for VideoBinaryHandler {
    fn extract_binary_tags(&self, reader: &ExifReader) -> Result<Vec<BinaryTag>> {
        let mut tags = Vec::new();

        // Video thumbnail extraction from QuickTime/MP4
        if let Some(video_thumbnails) = self.extract_video_thumbnails(reader)? {
            for (index, thumbnail) in video_thumbnails.iter().enumerate() {
                tags.push(BinaryTag {
                    name: format!("VideoThumbnail{}", index + 1),
                    size: thumbnail.size,
                    mime_type: Some("image/jpeg".to_string()),
                    description: format!("Video thumbnail {}", index + 1),
                    data_location: DataLocation::Embedded {
                        offset: thumbnail.offset,
                        size: thumbnail.size,
                    },
                });
            }
        }

        // Audio track extraction
        if let Some(audio_tracks) = self.extract_audio_tracks(reader)? {
            for (index, track) in audio_tracks.iter().enumerate() {
                tags.push(BinaryTag {
                    name: format!("AudioTrack{}", index + 1),
                    size: track.size,
                    mime_type: Some(track.codec.mime_type()),
                    description: format!("Audio track {} ({})", index + 1, track.codec),
                    data_location: DataLocation::Embedded {
                        offset: track.offset,
                        size: track.size,
                    },
                });
            }
        }

        Ok(tags)
    }
}
```

### Phase 3: CLI Integration (Week 2)

**Command Line Interface**:

```rust
// CLI argument parsing
#[derive(Parser)]
pub struct BinaryArgs {
    /// Extract binary data without tag names (equivalent to exiftool -b)
    #[arg(short = 'b', long = "binary")]
    pub binary: bool,

    /// Extract all binary tags
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Specific tag name to extract
    pub tag_name: Option<String>,

    /// Output file (default: stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

// CLI implementation
pub fn extract_binary_data(args: &BinaryArgs, input_file: &Path) -> Result<()> {
    let reader = ExifReader::from_file(input_file)?;

    if args.all {
        // Extract all binary tags
        let binary_tags = reader.list_binary_tags()?;
        for tag in binary_tags {
            let output_path = format!("{}_{}",
                input_file.file_stem().unwrap().to_str().unwrap(),
                tag.name);
            let mut output_file = File::create(output_path)?;
            reader.stream_binary_tag(&tag.name, &mut output_file)?;
            println!("Extracted: {} ({} bytes)", tag.name, tag.size);
        }
    } else if let Some(tag_name) = &args.tag_name {
        // Extract specific tag
        let mut output: Box<dyn Write> = if let Some(output_path) = &args.output {
            Box::new(File::create(output_path)?)
        } else {
            Box::new(io::stdout())
        };

        let bytes_written = reader.stream_binary_tag(tag_name, &mut output)?;
        if args.output.is_some() {
            eprintln!("Extracted {} bytes to {}", bytes_written, args.output.as_ref().unwrap().display());
        }
    } else {
        return Err(ExifError::MissingBinaryTag);
    }

    Ok(())
}
```

**Usage Examples**:

```bash
# Extract EXIF thumbnail
exif-oxide -b ThumbnailImage photo.jpg > thumbnail.jpg

# Extract RAW preview image
exif-oxide -b PreviewImage camera.nef -o preview.jpg

# Extract ICC color profile
exif-oxide -b ICC_Profile image.tiff > profile.icc

# Extract all binary data from file
exif-oxide -b -a video.mp4

# List available binary tags
exif-oxide --list-binary photo.jpg
```

### Phase 4: Advanced Features and Testing (Week 3)

**Size Limits and Safety**:

```rust
#[derive(Debug, Clone)]
pub struct BinarySizeLimits {
    pub max_thumbnail_size: u64,      // 10MB default
    pub max_preview_size: u64,        // 50MB default
    pub max_profile_size: u64,        // 1MB default
    pub max_total_extraction: u64,    // 500MB default
    pub warn_large_extraction: u64,   // 100MB default
}

impl BinaryExtractor {
    fn validate_extraction_size(&self, tag: &BinaryTag) -> Result<()> {
        let limit = match tag.name.as_str() {
            name if name.contains("Thumbnail") => self.size_limits.max_thumbnail_size,
            name if name.contains("Preview") => self.size_limits.max_preview_size,
            name if name.contains("ICC") => self.size_limits.max_profile_size,
            _ => self.size_limits.max_preview_size, // Default to preview limit
        };

        if tag.size > limit {
            return Err(ExifError::BinaryDataTooLarge {
                tag_name: tag.name.clone(),
                size: tag.size,
                limit,
            });
        }

        if tag.size > self.size_limits.warn_large_extraction {
            warn!("Large binary extraction: {} ({} bytes)", tag.name, tag.size);
        }

        Ok(())
    }
}
```

**Progress Reporting for Large Extractions**:

```rust
pub struct ProgressReporter {
    callback: Box<dyn Fn(u64, u64)>, // (bytes_processed, total_bytes)
}

impl BinaryExtractor {
    pub fn stream_with_progress<W: Write>(
        &self,
        reader: &ExifReader,
        tag_name: &str,
        writer: &mut W,
        progress: Option<ProgressReporter>,
    ) -> Result<u64> {
        let tag = reader.find_binary_tag(tag_name)?;
        self.validate_extraction_size(&tag)?;

        let mut total_bytes = 0;
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks

        let mut data_reader = reader.get_binary_reader(&tag)?;

        loop {
            let bytes_read = data_reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read])?;
            total_bytes += bytes_read as u64;

            if let Some(ref progress) = progress {
                (progress.callback)(total_bytes, tag.size);
            }
        }

        Ok(total_bytes)
    }
}
```

## Success Criteria

### Core Requirements

- [ ] **CLI Compatibility**: `exif-oxide -b TagName file.jpg` works equivalent to ExifTool
- [ ] **Streaming API**: Extract large binary data without loading into memory
- [ ] **Format Coverage**: Support JPEG, TIFF, RAW, and video binary extraction
- [ ] **Safety Limits**: Configurable size limits prevent excessive memory usage
- [ ] **Error Handling**: Graceful handling of missing or corrupted binary data

### Validation Tests

- Extract thumbnails from JPEG files and verify they are valid images
- Extract preview images from Canon CR2, Nikon NEF, Sony ARW files
- Extract ICC profiles and validate they can be used by color management systems
- Test with large video files to ensure streaming works correctly
- Verify size limits prevent extraction of maliciously large data

## Implementation Boundaries

### Goals (Milestone 19)

- Complete binary data extraction for mainstream use cases
- CLI equivalence with `exiftool -b` functionality
- Streaming API for memory-efficient large data handling
- Safety limits and progress reporting for large extractions

### Non-Goals (Future Milestones)

- **Binary data writing**: Only extraction, not modification
- **Format conversion**: Extract data as-is, no format conversion
- **Advanced video processing**: Basic preview/thumbnail extraction only
- **Lens correction application**: Extract data only, not apply corrections

## Dependencies and Prerequisites

### Milestone Prerequisites

- **Milestone 17**: RAW format support for RAW binary extraction
- **Milestone 18**: Video format support for video binary extraction
- **Core EXIF/TIFF**: Basic metadata extraction infrastructure

### Technical Dependencies

- **Streaming I/O**: Efficient large file handling
- **Format detection**: Know where binary data is located in each format
- **Memory management**: Avoid loading large binary data into memory

## Risk Mitigation

### Memory Usage Risk

- **Risk**: Large binary extractions could cause memory issues
- **Mitigation**: Streaming API with configurable size limits
- **Implementation**: Process data in chunks, never load entire binary data

### Security Risk: Binary Data Size

- **Risk**: Maliciously crafted files with enormous "binary data" could cause DoS
- **Mitigation**: Conservative default size limits with user override capability
- **Validation**: Validate reported sizes against file size and reasonable limits

### Format-Specific Extraction Complexity

- **Risk**: Each format stores binary data differently
- **Mitigation**: Modular handler approach allows format-specific implementation
- **Strategy**: Start with common formats, add specialized handlers incrementally

## Related Documentation

### Required Reading

- **ExifTool Manual**: `-b` flag documentation and usage patterns
- **Format Documentation**: Understanding where each format stores binary data
- **MIMETYPES.md**: Binary data types supported across different formats

### Implementation References

- **Existing RAW Processors**: Leverage preview extraction from Milestone 17
- **Video Processors**: Use video format infrastructure from Milestone 18
- **EXIF/TIFF Infrastructure**: Binary data location patterns

This milestone completes the core metadata extraction capabilities by adding binary data support, enabling users to fully extract all data types that ExifTool can provide while maintaining memory efficiency and security through streaming and size limits.
