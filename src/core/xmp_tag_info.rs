//! XMP tag information types for generated code
//!
//! Types used by generated XMP namespace lookup tables.

use crate::types::PrintConv;

/// XMP tag definition extracted from ExifTool XMP namespace tables
#[derive(Debug, Clone)]
pub struct XmpTagInfo {
    /// Display name (e.g., "License", "RegionList")
    pub name: &'static str,

    /// Writable type: "string", "lang-alt", "integer", "real", "boolean"
    /// None means not writable or simple string
    pub writable: Option<&'static str>,

    /// RDF container type (Bag, Seq, Alt)
    pub list: Option<XmpListType>,

    /// True if value is a URI resource (not plain string)
    pub resource: bool,

    /// PrintConv conversion lookup (if any)
    pub print_conv: Option<PrintConv>,
}

/// XMP RDF container types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmpListType {
    /// Unordered bag (rdf:Bag)
    Bag,
    /// Ordered sequence (rdf:Seq)
    Seq,
    /// Language alternatives (rdf:Alt)
    Alt,
}

impl XmpTagInfo {
    /// Create a simple XMP tag with just a name
    pub const fn simple(name: &'static str) -> Self {
        Self {
            name,
            writable: None,
            list: None,
            resource: false,
            print_conv: None,
        }
    }

    /// Create a resource (URI) XMP tag
    pub const fn resource(name: &'static str) -> Self {
        Self {
            name,
            writable: None,
            list: None,
            resource: true,
            print_conv: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let tag = XmpTagInfo::simple("AttributionName");
        assert_eq!(tag.name, "AttributionName");
        assert!(!tag.resource);
        assert!(tag.list.is_none());
    }

    #[test]
    fn test_resource_tag() {
        let tag = XmpTagInfo::resource("License");
        assert_eq!(tag.name, "License");
        assert!(tag.resource);
    }

    #[test]
    fn test_list_types() {
        assert_ne!(XmpListType::Bag, XmpListType::Seq);
        assert_ne!(XmpListType::Seq, XmpListType::Alt);
    }
}
