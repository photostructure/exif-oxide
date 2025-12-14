//! ImageDataHash support for computing cryptographic hashes of image data
//!
//! This module implements ExifTool's ImageDataHash feature, which computes a hash
//! of only the actual image/media data, excluding all metadata. This allows detecting
//! changes to image content while ignoring metadata modifications.
//!
//! ## ExifTool API Reference
//!
//! ExifTool uses API options to enable ImageDataHash:
//! ```bash
//! exiftool -api requesttags=imagedatahash -api imagehashtype=MD5 image.jpg
//! exiftool -api requesttags=imagedatahash -api imagehashtype=SHA256 image.jpg
//! ```
//!
//! Reference: https://exiftool.org/forum/index.php?topic=14706.msg79218
//!
//! ## Supported Hash Algorithms
//!
//! - **MD5** (default): 32-character hex string, matches ExifTool default
//! - **SHA256**: 64-character hex string
//! - **SHA512**: 128-character hex string
//!
//! ## What Gets Hashed
//!
//! The hash includes only actual image data, not metadata:
//!
//! - **JPEG**: SOS marker through EOI (scan data + RST markers + stuffed bytes)
//! - **PNG**: IDAT, JDAT, fdAT chunk data (not headers or CRC)
//! - **TIFF**: Data at StripOffsets/TileOffsets/JpgFromRawStart (tags with IsImageData)
//!
//! ## ExifTool Source References
//!
//! - Hash object creation: `lib/Image/ExifTool.pm:2766-2780`
//! - Hash finalization: `lib/Image/ExifTool.pm:4378-4386`
//! - JPEG hashing: `lib/Image/ExifTool.pm:7217-7406`
//! - PNG hashing: `lib/Image/ExifTool/PNG.pm:1419-1593`
//! - TIFF hashing: `lib/Image/ExifTool/Exif.pm:6200-7094`

use digest::{Digest, DynDigest};
use md5::Md5;
use sha2::{Sha256, Sha512};
use std::fmt;
use std::io::{Read, Seek, SeekFrom};

/// Hash algorithm selection for ImageDataHash
///
/// Matches ExifTool's `ImageHashType` API option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageHashType {
    /// MD5 hash (32 hex chars) - ExifTool default
    #[default]
    Md5,
    /// SHA-256 hash (64 hex chars)
    Sha256,
    /// SHA-512 hash (128 hex chars)
    Sha512,
}

impl ImageHashType {
    /// Parse from string (case-insensitive), matching ExifTool API
    /// Note: Using parse_str instead of from_str to avoid FromStr trait expectation
    pub fn parse_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SHA256" => Self::Sha256,
            "SHA512" => Self::Sha512,
            _ => Self::Md5, // Default to MD5
        }
    }

    /// Get the empty hash for this algorithm (hash of zero bytes)
    /// Used for suppressing output when no image data was found
    pub fn empty_hash(&self) -> &'static str {
        match self {
            Self::Md5 => "d41d8cd98f00b204e9800998ecf8427e",
            Self::Sha256 => "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            Self::Sha512 => "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        }
    }
}

impl fmt::Display for ImageHashType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Md5 => write!(f, "MD5"),
            Self::Sha256 => write!(f, "SHA256"),
            Self::Sha512 => write!(f, "SHA512"),
        }
    }
}

/// Streaming hasher for image data
///
/// Accumulates hash of image data as it's read during file processing.
/// Follows ExifTool's approach of maintaining a hash object throughout
/// file parsing and finalizing at the end.
pub struct ImageDataHasher {
    /// The underlying hasher (boxed for type erasure)
    hasher: Box<dyn DynDigest + Send>,
    /// Algorithm type for empty hash detection
    hash_type: ImageHashType,
    /// Total bytes hashed (for verbose output)
    bytes_hashed: u64,
}

impl ImageDataHasher {
    /// Create a new hasher with the specified algorithm
    pub fn new(hash_type: ImageHashType) -> Self {
        let hasher: Box<dyn DynDigest + Send> = match hash_type {
            ImageHashType::Md5 => Box::new(Md5::new()),
            ImageHashType::Sha256 => Box::new(Sha256::new()),
            ImageHashType::Sha512 => Box::new(Sha512::new()),
        };

        Self {
            hasher,
            hash_type,
            bytes_hashed: 0,
        }
    }

    /// Add data to the hash
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
        self.bytes_hashed += data.len() as u64;
    }

    /// Hash data from a reader with optional size limit
    ///
    /// Follows ExifTool's ImageDataHash function (Writer.pl:7086-7109):
    /// - Reads in 64KB chunks for memory efficiency
    /// - If size is None, reads until EOF
    /// - Returns bytes read
    ///
    /// # Arguments
    /// * `reader` - The reader positioned at the start of data to hash
    /// * `size` - Optional size limit; if None, reads until EOF
    pub fn hash_from_reader<R: Read>(
        &mut self,
        reader: &mut R,
        size: Option<u64>,
    ) -> std::io::Result<u64> {
        const CHUNK_SIZE: usize = 65536; // 64KB, matches ExifTool
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut bytes_read = 0u64;
        let mut remaining = size;

        loop {
            // Determine how much to read this iteration
            let to_read = match remaining {
                Some(0) => break, // Size limit reached
                Some(r) if r < CHUNK_SIZE as u64 => r as usize,
                Some(_) => CHUNK_SIZE,
                None => CHUNK_SIZE, // No limit, read full chunk
            };

            let n = reader.read(&mut buffer[..to_read])?;
            if n == 0 {
                break; // EOF
            }

            self.update(&buffer[..n]);
            bytes_read += n as u64;

            if let Some(ref mut r) = remaining {
                *r -= n as u64;
            }
        }

        Ok(bytes_read)
    }

    /// Hash data from a seekable reader at a specific offset
    ///
    /// Seeks to the offset, hashes the specified number of bytes, and returns.
    /// Used for TIFF strip/tile hashing where offsets are known.
    pub fn hash_at_offset<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        offset: u64,
        size: u64,
    ) -> std::io::Result<u64> {
        reader.seek(SeekFrom::Start(offset))?;
        self.hash_from_reader(reader, Some(size))
    }

    /// Get the total bytes hashed so far
    pub fn bytes_hashed(&self) -> u64 {
        self.bytes_hashed
    }

    /// Get the hash type
    pub fn hash_type(&self) -> ImageHashType {
        self.hash_type
    }

    /// Finalize and return the hash as a hex string
    ///
    /// Returns None if the hash equals the empty hash (no data was hashed),
    /// following ExifTool's behavior of suppressing empty hashes.
    pub fn finalize(self) -> Option<String> {
        let result = self.hasher.finalize();
        let hex = hex_encode(&result);

        // Suppress empty hashes (ExifTool behavior)
        if hex == self.hash_type.empty_hash() {
            None
        } else {
            Some(hex)
        }
    }

    /// Finalize and return the hash, even if empty
    ///
    /// Use this when you need the hash regardless of whether data was hashed.
    pub fn finalize_unchecked(self) -> String {
        let result = self.hasher.finalize();
        hex_encode(&result)
    }
}

impl fmt::Debug for ImageDataHasher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageDataHasher")
            .field("hash_type", &self.hash_type)
            .field("bytes_hashed", &self.bytes_hashed)
            .finish()
    }
}

/// Convert bytes to lowercase hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_md5_empty_hash() {
        let hasher = ImageDataHasher::new(ImageHashType::Md5);
        let result = hasher.finalize();
        // Empty hash should be suppressed
        assert!(result.is_none());
    }

    #[test]
    fn test_md5_hello_world() {
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);
        hasher.update(b"Hello, World!");
        let result = hasher.finalize().unwrap();
        // MD5 of "Hello, World!" is 65a8e27d8879283831b664bd8b7f0ad4
        assert_eq!(result, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_sha256_hello_world() {
        let mut hasher = ImageDataHasher::new(ImageHashType::Sha256);
        hasher.update(b"Hello, World!");
        let result = hasher.finalize().unwrap();
        // SHA256 of "Hello, World!"
        assert_eq!(
            result,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_sha512_empty_hash_suppression() {
        let hasher = ImageDataHasher::new(ImageHashType::Sha512);
        let result = hasher.finalize();
        assert!(result.is_none());
    }

    #[test]
    fn test_hash_from_reader() {
        let data = b"Test data for hashing";
        let mut cursor = Cursor::new(data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        let bytes_read = hasher.hash_from_reader(&mut cursor, None).unwrap();
        assert_eq!(bytes_read, data.len() as u64);
        assert_eq!(hasher.bytes_hashed(), data.len() as u64);

        let result = hasher.finalize().unwrap();
        // MD5 of "Test data for hashing"
        assert_eq!(result, "29beaab220adf762cba5208784ed02b0");
    }

    #[test]
    fn test_hash_from_reader_with_size_limit() {
        let data = b"Test data for hashing";
        let mut cursor = Cursor::new(data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        // Only hash first 4 bytes ("Test")
        let bytes_read = hasher.hash_from_reader(&mut cursor, Some(4)).unwrap();
        assert_eq!(bytes_read, 4);
        assert_eq!(hasher.bytes_hashed(), 4);

        let result = hasher.finalize().unwrap();
        // MD5 of "Test"
        assert_eq!(result, "0cbc6611f5540bd0809a388dc95a615b");
    }

    #[test]
    fn test_hash_at_offset() {
        let data = b"PREFIXTest data for hashingSUFFIX";
        let mut cursor = Cursor::new(data);
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);

        // Hash "Test data for hashing" starting at offset 6
        let bytes_read = hasher.hash_at_offset(&mut cursor, 6, 21).unwrap();
        assert_eq!(bytes_read, 21);

        let result = hasher.finalize().unwrap();
        // MD5 of "Test data for hashing"
        assert_eq!(result, "29beaab220adf762cba5208784ed02b0");
    }

    #[test]
    fn test_hash_type_from_str() {
        assert_eq!(ImageHashType::parse_str("MD5"), ImageHashType::Md5);
        assert_eq!(ImageHashType::parse_str("md5"), ImageHashType::Md5);
        assert_eq!(ImageHashType::parse_str("SHA256"), ImageHashType::Sha256);
        assert_eq!(ImageHashType::parse_str("sha256"), ImageHashType::Sha256);
        assert_eq!(ImageHashType::parse_str("SHA512"), ImageHashType::Sha512);
        assert_eq!(ImageHashType::parse_str("unknown"), ImageHashType::Md5); // Default
    }

    #[test]
    fn test_incremental_hashing() {
        // Hash in one go
        let mut hasher1 = ImageDataHasher::new(ImageHashType::Md5);
        hasher1.update(b"Hello, World!");
        let result1 = hasher1.finalize_unchecked();

        // Hash incrementally
        let mut hasher2 = ImageDataHasher::new(ImageHashType::Md5);
        hasher2.update(b"Hello, ");
        hasher2.update(b"World!");
        let result2 = hasher2.finalize_unchecked();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_bytes_hashed_tracking() {
        let mut hasher = ImageDataHasher::new(ImageHashType::Md5);
        assert_eq!(hasher.bytes_hashed(), 0);

        hasher.update(b"Hello");
        assert_eq!(hasher.bytes_hashed(), 5);

        hasher.update(b", World!");
        assert_eq!(hasher.bytes_hashed(), 13);
    }
}
