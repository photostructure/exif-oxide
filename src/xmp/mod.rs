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

pub mod processor;

pub use processor::XmpProcessor;
