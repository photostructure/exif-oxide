# Milestone 19: Binary Data Extraction (`-b` support)

**Duration**: 2-3 weeks  
**Goal**: Implement comprehensive binary data extraction equivalent to `exiftool -b`

## Overview

Binary data extraction is a fundamental feature that enables users to extract embedded images, thumbnails, color profiles, and other binary metadata from files. This milestone implements the equivalent of ExifTool's `-b` flag with both CLI and streaming API support.

## Background: ExifTool's `-b` Functionality

From ExifTool manual:

> `-b, --b (-binary, --binary): Output requested metadata in binary format without tag names or descriptions. This option is mainly used for extracting embedded images or other binary data.`

**Common Use Cases**:

- Extract JPEG thumbnails from EXIF data (ThumbnailImage)
- Save preview images from RAW files (PreviewImage, JpgFromRaw)  
- Note: We emulate ExifTool's group resolution logic to find the "best" group delivering the requested payload

## Implementation Strategy

### Prerequisites

Before starting implementation, verify test sample availability:

1. **Check existing test images**: Review `test-images/*` directory for RAW and JPEG samples
2. **Required formats**: Ensure we have samples for:
   - Canon CR2/CR3 with embedded previews
   - Nikon NEF with preview images
   - Sony ARW with embedded JPEGs
   - JPEG files with EXIF thumbnails
3. **Request missing samples**: Ask user to provide any missing format samples needed for comprehensive testing

### Phase 1: Core Binary Extraction Infrastructure (Week 1)

**ExifTool Group Resolution**:

When users request a tag like `exiftool -b -PreviewImage`, ExifTool performs intelligent group resolution to find the best available preview. We must emulate this behavior:

```rust
pub struct GroupResolver {
    priority_groups: HashMap<String, Vec<String>>,
}

impl GroupResolver {
    fn resolve_binary_tag(&self, tag_name: &str, available_tags: &[BinaryTag]) -> Option<&BinaryTag> {
        // Follow ExifTool's priority order for common binary tags
        match tag_name {
            "PreviewImage" => {
                // Priority: JpgFromRaw > PreviewImage > ThumbnailImage
                self.find_best_match(&["JpgFromRaw", "PreviewImage", "ThumbnailImage"], available_tags)
            },
            _ => available_tags.iter().find(|t| t.name == tag_name),
        }
    }
}
```

**Tag Include-List Infrastructure**:

```rust
pub struct BinaryTagFilter {
    allowed_tags: HashSet<String>,
}

impl Default for BinaryTagFilter {
    fn default() -> Self {
        // Start with mainstream binary tags only
        Self {
            allowed_tags: HashSet::from([
                "ThumbnailImage".to_string(),
                "PreviewImage".to_string(), 
                "JpgFromRaw".to_string(),
            ]),
        }
    }
}
```

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

**CRITICAL**: Verify that our streaming API infrastructure from the core library supports binary data extraction without loading entire payloads into memory.

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

        // Basic video thumbnail extraction only
        // (Advanced video/audio extraction deferred to future milestones)
        if let Some(thumbnail) = self.extract_first_video_thumbnail(reader)? {
            tags.push(BinaryTag {
                name: "VideoThumbnail".to_string(),
                size: thumbnail.size,
                mime_type: Some("image/jpeg".to_string()),
                description: "Video thumbnail".to_string(),
                data_location: DataLocation::Embedded {
                    offset: thumbnail.offset,
                    size: thumbnail.size,
                },
            });
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

    /// Specific tag name to extract
    pub tag_name: String,  // Required - we always need a tag name
    
    // Note: Output is always to stdout - no output file option needed
}

// CLI implementation - stdout only, no file output option
pub fn extract_binary_data(args: &BinaryArgs, input_file: &Path) -> Result<()> {
    let reader = ExifReader::from_file(input_file)?;
    
    // Always output to stdout
    let mut stdout = io::stdout();
    
    // Use group resolver to find best matching tag
    let available_tags = reader.list_binary_tags()?;
    let resolver = GroupResolver::default();
    
    if let Some(tag) = resolver.resolve_binary_tag(&args.tag_name, &available_tags) {
        // Stream directly to stdout
        reader.stream_binary_tag(&tag.name, &mut stdout)?;
    } else {
        return Err(ExifError::BinaryTagNotFound(args.tag_name.clone()));
    }

    Ok(())
}
```

**Usage Examples**:

```bash
# Extract EXIF thumbnail
exif-oxide -b ThumbnailImage photo.jpg > thumbnail.jpg

# Extract RAW preview image (uses group resolution to find best preview)
exif-oxide -b PreviewImage camera.nef > preview.jpg

# Extract full resolution JPEG from RAW
exif-oxide -b JpgFromRaw photo.cr2 > full_res.jpg
```

### Phase 4: Testing Infrastructure (Week 3)

**Test Setup**:

```rust
// tests/binary_extraction_tests.rs
use common::*;
use std::process::Command;
use sha2::{Sha256, Digest};

#[test]
fn test_binary_extraction_compatibility() {
    // Check if we have required test images
    let test_images = vec![
        "test-images/Canon/canon_cr2.cr2",
        "test-images/Nikon/nikon_nef.nef", 
        "test-images/Sony/sony_arw.arw",
        "test-images/jpeg_with_thumbnail.jpg",
    ];
    
    // Request missing samples if needed
    for image_path in &test_images {
        if !Path::new(image_path).exists() {
            eprintln!("Missing test image: {} - please add to test-images/", image_path);
        }
    }
    
    // Test each supported binary tag
    let binary_tags = vec!["ThumbnailImage", "PreviewImage", "JpgFromRaw"];
    
    for test_image in test_images {
        for tag in &binary_tags {
            // Extract with ExifTool
            let exiftool_output = Command::new("exiftool")
                .args(&["-b", &format!("-{}", tag), test_image])
                .output()
                .expect("Failed to run exiftool");
                
            if !exiftool_output.stdout.is_empty() {
                // Extract with exif-oxide
                let oxide_output = Command::new("./target/release/exif-oxide")
                    .args(&["-b", tag, test_image])
                    .output()
                    .expect("Failed to run exif-oxide");
                    
                // Compare SHA256 hashes
                let exiftool_sha = Sha256::digest(&exiftool_output.stdout);
                let oxide_sha = Sha256::digest(&oxide_output.stdout);
                
                assert_eq!(exiftool_sha, oxide_sha, 
                    "Binary extraction mismatch for {} in {}", tag, test_image);
                    
                // Optionally save for debugging
                if std::env::var("SAVE_BINARY_DEBUG").is_ok() {
                    let debug_path = format!("tmp/{}/{}.jpg", 
                        test_image.replace('/', "_"), tag);
                    std::fs::create_dir_all(Path::new(&debug_path).parent().unwrap()).ok();
                    std::fs::write(&debug_path, &oxide_output.stdout).ok();
                }
            }
        }
    }
}
```

### Phase 5: Safety and Size Limits (Week 3)

**Size Limits and Safety**:

```rust
#[derive(Debug, Clone)]
pub struct BinarySizeLimits {
    pub max_thumbnail_size: u64,      // 1MB default (conservative)
    pub max_preview_size: u64,        // 10MB default (conservative)
    pub max_profile_size: u64,        // 1MB default
    pub max_total_extraction: u64,    // 50MB default
    pub warn_large_extraction: u64,   // 5MB default
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
- [ ] **Group Resolution**: Emulate ExifTool's logic for finding best binary tag match
- [ ] **Streaming API**: Verify existing API supports binary extraction without loading into memory
- [ ] **Format Coverage**: Support JPEG, TIFF, RAW binary extraction (ThumbnailImage, PreviewImage, JpgFromRaw)
- [ ] **Tag Filtering**: Include-list based approach for supported binary tags
- [ ] **Safety Limits**: Conservative size limits prevent excessive memory usage
- [ ] **Error Handling**: Graceful handling of missing or corrupted binary data

### Validation Tests

- [ ] **SHA Comparison**: Extract binary data with both tools and compare SHA256 hashes
- [ ] **Test Coverage**: Use existing test-images/* samples for each manufacturer
- [ ] **Missing Samples**: Check and request any missing RAW format samples
- [ ] **Tag Support**: Test all tags in include-list (ThumbnailImage, PreviewImage, JpgFromRaw)
- [ ] **Size Limits**: Verify conservative limits prevent malicious extraction
- [ ] **Debug Output**: Optional saving to tmp/ directories for debugging

## Implementation Boundaries

### Goals (Milestone 19)

- Complete binary data extraction for mainstream use cases
- CLI equivalence with `exiftool -b` functionality
- Streaming API for memory-efficient large data handling
- Safety limits and progress reporting for large extractions

### Non-Goals (Future Milestones)

- **Binary data writing**: Only extraction, not modification
- **Format conversion**: Extract data as-is, no format conversion  
- **Advanced features**: No -a (all) flag, no --list-binary option
- **Output options**: Stdout only, no -o output file option
- **Complex video/audio**: Future milestone, only basic video thumbnail for now
- **Lens correction**: Future milestone

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
- **Mitigation**: Streaming API with conservative default size limits (1MB thumbnail, 10MB preview)
- **Implementation**: Process data in chunks, never load entire binary data
- **Validation**: Compare extracted size against file size before extraction

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
