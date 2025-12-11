//! XMP (eXtensible Metadata Platform) processing module
//!
//! This module implements XMP metadata extraction using structured output
//! (equivalent to `exiftool -j -struct`). XMP is Adobe's RDF/XML-based
//! metadata standard used in JPEG, TIFF, and other formats.
//!
//! Key features:
//! - RDF/XML parsing with namespace awareness
//! - Structured output preserving hierarchical data
//! - RDF container mapping (Bag/Seq → Array, Alt → Object)
//! - Language alternative support
//! - Generated tag tables for 719 XMP tags across 40 namespaces

pub mod processor;
pub mod xmp_lookup;

pub use processor::XmpProcessor;
pub use xmp_lookup::{get_xmp_tag_name, lookup_xmp_tag};
