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

    /// Statically resolved FoundTag priority used for duplicate-name
    /// resolution: the per-tag `Priority`, else the table-level `PRIORITY`
    /// (e.g. 0 for the tiff/exif/exifEX namespaces, XMP.pm:1900/1992/2462),
    /// else 0 when the tag is marked `Avoid` (directly, or via a table-level
    /// `AVOID` such as the PRISM namespaces). `None` means ExifTool's runtime
    /// default of 1 applies.
    /// ExifTool: lib/Image/ExifTool.pm:9469-9473 (priority chain),
    /// 9250-9251 (table AVOID propagated to tag Avoid), 9562 (default 1).
    pub priority: Option<i8>,

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
            priority: None,
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
            priority: None,
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
