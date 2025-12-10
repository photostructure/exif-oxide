//! Tag value types and conversion utilities
//!
//! This module defines the core `TagValue` enum that represents all possible
//! EXIF tag values after parsing, along with its conversion methods and
//! display formatting.

mod conversion;
mod display;
mod ops;
mod serialization;

#[cfg(test)]
mod tests;

use serde::Deserialize;
use std::collections::HashMap;

/// Represents a tag value that can be of various types
///
/// ExifTool handles many different data types. This enum represents
/// the possible values a tag can contain after parsing.
///
/// # Creating String TagValues
///
/// There are several convenient ways to create string TagValues:
///
/// ```
/// use exif_oxide::core::TagValue;
///
/// // Most ergonomic - using From trait
/// let tag1: TagValue = "Hello".into();
/// let tag2 = TagValue::from("World");
///
/// // Using the convenience method
/// let tag3 = TagValue::string("Foo");
///
/// // Traditional way (still works)
/// let tag4 = TagValue::String("Bar".to_string());
///
/// // All create the same type
/// assert_eq!(tag1, TagValue::String("Hello".to_string()));
/// assert_eq!(tag2, TagValue::String("World".to_string()));
/// assert_eq!(tag3, TagValue::String("Foo".to_string()));
/// assert_eq!(tag4, TagValue::String("Bar".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TagValue {
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer  
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Signed 16-bit integer
    I16(i16),
    /// Signed 32-bit integer
    I32(i32),
    /// Floating point number
    F64(f64),
    /// Text string
    String(String),
    /// Boolean value
    Bool(bool),
    /// Array of unsigned 8-bit integers (for binary data)
    U8Array(Vec<u8>),
    /// Array of unsigned 16-bit integers
    U16Array(Vec<u16>),
    /// Array of unsigned 32-bit integers  
    U32Array(Vec<u32>),
    /// Array of floating point numbers (for rational arrays)
    F64Array(Vec<f64>),
    /// Rational number as numerator/denominator pair (RATIONAL format)
    /// ExifTool: Format type 5 (rational64u) - 2x uint32
    Rational(u32, u32),
    /// Signed rational number as numerator/denominator pair (SRATIONAL format)  
    /// ExifTool: Format type 10 (rational64s) - 2x int32
    SRational(i32, i32),
    /// Array of rational numbers for multi-value tags like GPS coordinates
    /// ExifTool: GPSLatitude/GPSLongitude arrays [degrees/1, minutes/1, seconds/100]
    RationalArray(Vec<(u32, u32)>),
    /// Array of signed rational numbers
    SRationalArray(Vec<(i32, i32)>),
    /// Raw binary data when type is unknown
    Binary(Vec<u8>),
    /// Nested object for structured data (e.g., XMP structures)
    /// Used for hierarchical metadata like ContactInfo, LocationCreated
    Object(HashMap<String, TagValue>),
    /// Array of heterogeneous values (e.g., XMP RDF containers)
    /// Used for RDF Bag/Seq containers and mixed-type arrays
    Array(Vec<TagValue>),
    /// Empty/undefined value for missing dependencies or composite tag failures
    /// ExifTool equivalent: undef
    Empty,
}
