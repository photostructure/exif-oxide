//! XMP (Extensible Metadata Platform) support
//!
//! XMP is Adobe's XML-based metadata format that complements EXIF with richer,
//! hierarchical metadata including keywords, ratings, and creator information.

use thiserror::Error;

pub mod parser;
pub mod reader;
pub mod types;

pub use parser::{extract_simple_properties, parse_xmp};
pub use reader::{extract_xmp_properties, read_xmp_from_jpeg};
pub use types::{ExtendedXmp, XmpArray, XmpMetadata, XmpPacket, XmpValue};

/// XMP-specific errors
#[derive(Debug, Error)]
pub enum XmpError {
    #[error("Invalid XMP signature")]
    InvalidSignature,

    #[error("XML parsing error: {0}")]
    XmlError(String),

    #[error("Extended XMP error: {0}")]
    ExtendedXmpError(String),

    #[error("Invalid UTF-8 in XMP data")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// XMP signature in JPEG APP1 segments
pub const XMP_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

/// Extended XMP signature in JPEG APP1 segments  
pub const XMP_EXTENSION_SIGNATURE: &[u8] = b"http://ns.adobe.com/xmp/extension/\0";

/// Common XMP namespaces
pub mod namespaces {
    /// XMP basic namespace
    pub const XMP: &str = "http://ns.adobe.com/xap/1.0/";

    /// Dublin Core namespace
    pub const DC: &str = "http://purl.org/dc/elements/1.1/";

    /// EXIF namespace
    pub const EXIF: &str = "http://ns.adobe.com/exif/1.0/";

    /// TIFF namespace
    pub const TIFF: &str = "http://ns.adobe.com/tiff/1.0/";

    /// Photoshop namespace
    pub const PHOTOSHOP: &str = "http://ns.adobe.com/photoshop/1.0/";

    /// XMP Rights namespace
    pub const XMP_RIGHTS: &str = "http://ns.adobe.com/xap/1.0/rights/";

    /// IPTC Core namespace
    pub const IPTC: &str = "http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/";
}

/// Check if data starts with XMP signature
pub fn is_xmp_data(data: &[u8]) -> bool {
    data.starts_with(XMP_SIGNATURE)
}

/// Check if data starts with Extended XMP signature
pub fn is_extended_xmp_data(data: &[u8]) -> bool {
    data.starts_with(XMP_EXTENSION_SIGNATURE)
}

/// Extract XMP packet from APP1 segment data
pub fn extract_xmp_packet(data: &[u8]) -> Result<&[u8], XmpError> {
    if !is_xmp_data(data) {
        return Err(XmpError::InvalidSignature);
    }

    // Skip the signature
    Ok(&data[XMP_SIGNATURE.len()..])
}

/// Extract Extended XMP data from APP1 segment
pub fn extract_extended_xmp_data(data: &[u8]) -> Result<(&[u8], &[u8]), XmpError> {
    if !is_extended_xmp_data(data) {
        return Err(XmpError::InvalidSignature);
    }

    // Skip the signature
    let data = &data[XMP_EXTENSION_SIGNATURE.len()..];

    // Extended XMP format:
    // - GUID: 32 bytes (ASCII hex)
    // - Total length: 4 bytes
    // - Offset: 4 bytes
    // - Data: remaining bytes

    if data.len() < 40 {
        return Err(XmpError::ExtendedXmpError(
            "Invalid extended XMP header".to_string(),
        ));
    }

    let header = &data[..40];
    let chunk_data = &data[40..];

    Ok((header, chunk_data))
}
