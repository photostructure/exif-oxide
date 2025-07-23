# Milestone 23: ImageDataHash

**Duration**: 2-3 weeks  
**Goal**: Implement cryptographic hashing of image/media content for integrity verification

## Overview

ImageDataHash provides cryptographic fingerprinting of the actual image/media content within files, excluding metadata. This enables content integrity verification, duplicate detection, and forensic analysis workflows by creating unique hashes that represent visual/audio content independent of metadata changes.

## Background: ExifTool's ImageDataHash Feature

**Content-Only Hashing**:
- **Includes**: Main image data, video/audio streams
- **Excludes**: JpgFromRaw, OtherImage, ThumbnailImage, PreviewImage, all metadata (EXIF, XMP, IPTC, etc.)

**Supported Algorithms**: MD5 (default), SHA256, SHA512

**Use Cases**:
- Content integrity verification (detect visual changes vs metadata-only changes)
- Duplicate detection across different metadata sets
- Forensic analysis and authenticity verification
- Digital asset management content tracking

## Implementation Strategy

### Phase 1: Core Hashing Infrastructure (Week 1)

**Hash Engine Foundation**:
```rust
use md5::Md5;
use sha2::{Sha256, Sha512};
use digest::{Digest, DynDigest};

pub struct ImageDataHasher {
    algorithm: HashAlgorithm,
    enabled_formats: HashSet<FileType>,
    chunk_size: usize, // Default 64KB for streaming
}

#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
    MD5,
    SHA256, 
    SHA512,
}

#[derive(Debug, Clone)]
pub struct ImageDataHash {
    pub algorithm: HashAlgorithm,
    pub hash_value: String,      // Hex-encoded hash
    pub bytes_hashed: u64,
    pub data_sources: Vec<String>, // e.g., ["MainImage", "JpgFromRaw"]
}

impl ImageDataHasher {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self {
            algorithm,
            enabled_formats: Self::get_supported_formats(),
            chunk_size: 64 * 1024, // 64KB chunks
        }
    }
    
    pub fn hash_file_content(&mut self, reader: &ExifReader) -> Result<Option<ImageDataHash>> {
        let file_type = reader.get_file_type();
        
        if !self.enabled_formats.contains(&file_type) {
            return Ok(None);
        }
        
        let mut hasher = self.create_hasher();
        let mut total_bytes = 0u64;
        let mut data_sources = Vec::new();
        
        // Format-specific content extraction
        match file_type {
            FileType::JPEG => {
                total_bytes += self.hash_jpeg_content(reader, &mut hasher, &mut data_sources)?;
            },
            FileType::PNG => {
                total_bytes += self.hash_png_content(reader, &mut hasher, &mut data_sources)?;
            },
            FileType::TIFF => {
                total_bytes += self.hash_tiff_content(reader, &mut hasher, &mut data_sources)?;
            },
            FileType::MP4 | FileType::QuickTime => {
                total_bytes += self.hash_video_content(reader, &mut hasher, &mut data_sources)?;
            },
            _ => return Ok(None),
        }
        
        if total_bytes == 0 {
            return Ok(None);
        }
        
        let hash_bytes = hasher.finalize();
        let hash_value = hex::encode(hash_bytes);
        
        Ok(Some(ImageDataHash {
            algorithm: self.algorithm,
            hash_value,
            bytes_hashed: total_bytes,
            data_sources,
        }))
    }
    
    fn create_hasher(&self) -> Box<dyn DynDigest> {
        match self.algorithm {
            HashAlgorithm::MD5 => Box::new(Md5::new()),
            HashAlgorithm::SHA256 => Box::new(Sha256::new()),
            HashAlgorithm::SHA512 => Box::new(Sha512::new()),
        }
    }
}
```

### Phase 2: Format-Specific Implementations (Week 1-2)

**JPEG Content Hashing**:
```rust
impl ImageDataHasher {
    fn hash_jpeg_content(
        &self,
        reader: &ExifReader,
        hasher: &mut Box<dyn DynDigest>,
        data_sources: &mut Vec<String>,
    ) -> Result<u64> {
        
        let mut total_bytes = 0u64;
        
        // Hash main JPEG data (SOS segments)
        if let Some(main_image_data) = reader.get_main_image_data()? {
            total_bytes += self.hash_data_stream(hasher, main_image_data)?;
            data_sources.push("MainImage".to_string());
        }
        
        // Hash JpgFromRaw (for RAW files with embedded JPEG)
        if let Some(jpg_from_raw) = reader.get_binary_tag("JpgFromRaw")? {
            total_bytes += self.hash_data_stream(hasher, jpg_from_raw)?;
            data_sources.push("JpgFromRaw".to_string());
        }
        
        // Hash OtherImage segments (but skip thumbnails/previews)
        for other_image in reader.get_other_images()? {
            if !other_image.is_thumbnail && !other_image.is_preview {
                total_bytes += self.hash_data_stream(hasher, other_image.data)?;
                data_sources.push(format!("OtherImage{}", other_image.index));
            }
        }
        
        Ok(total_bytes)
    }
    
    fn hash_data_stream(
        &self,
        hasher: &mut Box<dyn DynDigest>,
        mut data_reader: Box<dyn Read>,
    ) -> Result<u64> {
        
        let mut buffer = vec![0u8; self.chunk_size];
        let mut total_bytes = 0u64;
        
        loop {
            let bytes_read = data_reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
            total_bytes += bytes_read as u64;
        }
        
        Ok(total_bytes)
    }
}
```

**PNG Content Hashing**:
```rust
impl ImageDataHasher {
    fn hash_png_content(
        &self,
        reader: &ExifReader,
        hasher: &mut Box<dyn DynDigest>,
        data_sources: &mut Vec<String>,
    ) -> Result<u64> {
        
        let mut total_bytes = 0u64;
        
        // PNG stores image data in IDAT chunks
        // Must hash all IDAT chunks while skipping metadata chunks
        let png_chunks = reader.get_png_chunks()?;
        
        for chunk in png_chunks {
            match chunk.chunk_type.as_str() {
                "IDAT" => {
                    // This is image data - include in hash
                    hasher.update(&chunk.data);
                    total_bytes += chunk.data.len() as u64;
                },
                // Skip metadata chunks
                "tEXt" | "zTXt" | "iTXt" | "eXIf" | "iCCP" => {
                    // Skip these - they're metadata
                },
                // Include other critical chunks that affect image rendering
                "PLTE" | "tRNS" | "gAMA" | "cHRM" | "sRGB" | "sBIT" => {
                    hasher.update(&chunk.data);
                    total_bytes += chunk.data.len() as u64;
                },
                _ => {
                    // Unknown chunk - be conservative and include it
                    // unless it's clearly metadata (lowercase first letter = ancillary)
                    if chunk.chunk_type.chars().next().unwrap().is_uppercase() {
                        hasher.update(&chunk.data);
                        total_bytes += chunk.data.len() as u64;
                    }
                }
            }
        }
        
        if total_bytes > 0 {
            data_sources.push("MainImage".to_string());
        }
        
        Ok(total_bytes)
    }
}
```

**Video Content Hashing (QuickTime/MP4)**:
```rust
impl ImageDataHasher {
    fn hash_video_content(
        &self,
        reader: &ExifReader,
        hasher: &mut Box<dyn DynDigest>,
        data_sources: &mut Vec<String>,
    ) -> Result<u64> {
        
        let mut total_bytes = 0u64;
        
        // For video files, hash the media data (mdat) atoms
        // These contain the actual audio/video streams
        let atoms = reader.get_quicktime_atoms()?;
        
        for atom in atoms {
            match atom.atom_type.as_str() {
                "mdat" => {
                    // Media data - this is what we want to hash
                    let mut media_reader = atom.get_data_reader()?;
                    total_bytes += self.hash_data_stream(hasher, media_reader)?;
                    data_sources.push("MediaData".to_string());
                },
                "ftyp" | "moov" | "meta" | "udta" => {
                    // Skip metadata atoms
                },
                _ => {
                    // For unknown atoms, check if they might contain media data
                    // This is conservative - include unknown atoms that might be media
                    if atom.size > 1024 { // Likely to be media if large
                        let mut atom_reader = atom.get_data_reader()?;
                        total_bytes += self.hash_data_stream(hasher, atom_reader)?;
                        data_sources.push(format!("Atom_{}", atom.atom_type));
                    }
                }
            }
        }
        
        Ok(total_bytes)
    }
}
```

### Phase 3: Integration and Storage (Week 2)

**ExifReader Integration**:
```rust
impl ExifReader {
    /// Calculate hash of image/media content (excluding metadata)
    pub fn calculate_image_data_hash(&self, algorithm: HashAlgorithm) -> Result<Option<ImageDataHash>> {
        let mut hasher = ImageDataHasher::new(algorithm);
        hasher.hash_file_content(self)
    }
    
    /// Get image data hash if already calculated or calculate it
    pub fn get_or_calculate_image_hash(&mut self, algorithm: HashAlgorithm) -> Result<Option<ImageDataHash>> {
        // Check if hash was already calculated and stored
        if let Some(existing_hash) = self.get_stored_image_hash(algorithm)? {
            return Ok(Some(existing_hash));
        }
        
        // Calculate new hash
        let hash = self.calculate_image_data_hash(algorithm)?;
        
        // Store for future reference (optional)
        if let Some(ref hash_value) = hash {
            self.store_image_hash(hash_value)?;
        }
        
        Ok(hash)
    }
    
    fn store_image_hash(&mut self, hash: &ImageDataHash) -> Result<()> {
        // Store in XMP if XMP support is available
        let algorithm_name = match hash.algorithm {
            HashAlgorithm::MD5 => "MD5",
            HashAlgorithm::SHA256 => "SHA256", 
            HashAlgorithm::SHA512 => "SHA512",
        };
        
        // Store as ExifTool-compatible XMP tags
        self.set_tag_value("XMP-et:OriginalImageHash", TagValue::String(hash.hash_value.clone()))?;
        self.set_tag_value("XMP-et:OriginalImageHashType", TagValue::String(algorithm_name.to_string()))?;
        
        Ok(())
    }
}
```

**CLI Integration**:
```rust
// CLI support for image data hashing
#[derive(Parser)]
pub struct HashArgs {
    /// Calculate image data hash
    #[arg(long = "image-hash")]
    pub image_hash: bool,
    
    /// Hash algorithm (MD5, SHA256, SHA512)
    #[arg(long = "hash-type", default_value = "MD5")]
    pub hash_algorithm: String,
    
    /// Store hash in XMP metadata
    #[arg(long = "store-hash")]
    pub store_hash: bool,
}

pub fn handle_image_hash(args: &HashArgs, file_path: &Path) -> Result<()> {
    let mut reader = ExifReader::from_file(file_path)?;
    
    if args.image_hash {
        let algorithm = match args.hash_algorithm.to_uppercase().as_str() {
            "MD5" => HashAlgorithm::MD5,
            "SHA256" => HashAlgorithm::SHA256,
            "SHA512" => HashAlgorithm::SHA512,
            _ => return Err(ExifError::UnsupportedHashAlgorithm(args.hash_algorithm.clone())),
        };
        
        if let Some(hash) = reader.calculate_image_data_hash(algorithm)? {
            println!("ImageDataHash: {}", hash.hash_value);
            println!("Algorithm: {:?}", hash.algorithm);
            println!("Bytes Hashed: {}", hash.bytes_hashed);
            println!("Data Sources: {}", hash.data_sources.join(", "));
            
            if args.store_hash {
                reader.store_image_hash(&hash)?;
                println!("Hash stored in XMP metadata");
            }
        } else {
            println!("No image data found for hashing");
        }
    }
    
    Ok(())
}
```

### Phase 4: Testing and Validation (Week 3)

**Hash Validation Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jpeg_hash_consistency() {
        // Test that identical JPEG content produces identical hashes
        // regardless of metadata differences
        
        let original_file = "tests/fixtures/test_image.jpg";
        let modified_metadata_file = "tests/fixtures/test_image_modified_metadata.jpg";
        
        let hash1 = ExifReader::from_file(original_file)?
            .calculate_image_data_hash(HashAlgorithm::MD5)?
            .unwrap();
            
        let hash2 = ExifReader::from_file(modified_metadata_file)?
            .calculate_image_data_hash(HashAlgorithm::MD5)?
            .unwrap();
        
        assert_eq!(hash1.hash_value, hash2.hash_value);
    }
    
    #[test] 
    fn test_hash_algorithm_differences() {
        // Test that different algorithms produce different hashes
        let file = "tests/fixtures/test_image.jpg";
        let reader = ExifReader::from_file(file)?;
        
        let md5_hash = reader.calculate_image_data_hash(HashAlgorithm::MD5)?.unwrap();
        let sha256_hash = reader.calculate_image_data_hash(HashAlgorithm::SHA256)?.unwrap();
        
        assert_ne!(md5_hash.hash_value, sha256_hash.hash_value);
        assert_eq!(md5_hash.bytes_hashed, sha256_hash.bytes_hashed);
    }
    
    #[test]
    fn test_video_hash_calculation() {
        // Test video content hashing
        let video_file = "tests/fixtures/test_video.mp4";
        let hash = ExifReader::from_file(video_file)?
            .calculate_image_data_hash(HashAlgorithm::SHA256)?
            .unwrap();
        
        assert!(hash.bytes_hashed > 0);
        assert!(hash.data_sources.contains(&"MediaData".to_string()));
    }
}
```

## Success Criteria

### Core Requirements
- [ ] **Multi-Format Support**: Hash calculation for JPEG, PNG, TIFF, MP4/MOV
- [ ] **Multiple Algorithms**: MD5, SHA256, SHA512 support
- [ ] **Content-Only Hashing**: Exclude metadata, include only image/media data
- [ ] **Streaming Processing**: Handle large files efficiently with chunked processing
- [ ] **XMP Storage**: Store calculated hashes in XMP metadata tags

### Validation Tests
- Verify identical content produces identical hashes across different metadata
- Test large file processing without memory issues
- Validate hash values match ExifTool output for same files
- Test with various image formats and video files

## Implementation Boundaries

### Goals (Milestone 23)
- Content integrity verification through cryptographic hashing
- Support for mainstream image and video formats
- Integration with existing metadata processing pipeline
- CLI and API support for hash calculation

### Non-Goals (Future Enhancements)
- **Perceptual hashing**: Only cryptographic hashes, not visual similarity
- **All format support**: Focus on common formats, add others based on demand
- **Performance optimization**: Basic streaming approach, optimize later if needed
- **Hash comparison tools**: Only calculation, not duplicate detection workflows

## Dependencies and Prerequisites

### Milestone Prerequisites
- **Core format support**: JPEG, PNG, TIFF, video format parsing infrastructure
- **Binary data extraction**: Ability to access raw image/media data

### Technical Dependencies
- **Rust crypto crates**: `md5`, `sha2`, `hex` for hash calculation
- **Streaming I/O**: Efficient processing of large media files
- **Format parsers**: Understanding of format-specific data structures

## Risk Mitigation

### Algorithm Performance Risk
- **Risk**: Hash calculation on large video files could be slow
- **Mitigation**: Chunked processing with configurable chunk sizes
- **Monitoring**: Provide progress feedback for large operations

### Format-Specific Complexity Risk
- **Risk**: Each format requires custom implementation for data extraction
- **Mitigation**: Start with common formats, add others incrementally
- **Strategy**: Reuse existing format parsing infrastructure

### Memory Usage Risk
- **Risk**: Large files could cause memory issues during processing
- **Mitigation**: Streaming approach processes data in small chunks
- **Validation**: Test with multi-gigabyte video files

## Related Documentation

### Required Reading
- **ExifTool ImageHashType**: Feature documentation and use cases
- **Format specifications**: Understanding data vs metadata sections in each format
- **Cryptographic hash standards**: MD5, SHA256, SHA512 specifications

### Implementation References
- **Format parsers**: Existing JPEG, PNG, TIFF, video parsing infrastructure
- **Binary data handling**: Streaming data access patterns
- **XMP integration**: Metadata storage and retrieval patterns

This milestone adds specialized forensic and integrity verification capabilities to exif-oxide, enabling professional workflows that require content authentication and duplicate detection independent of metadata variations.