//! XMP namespace registry and common namespace definitions

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/XMP.pm"]

use std::collections::HashMap;

/// Registry of XMP namespaces with common namespaces pre-registered
pub struct NamespaceRegistry {
    /// Known namespaces (prefix -> URI)
    known: HashMap<&'static str, &'static str>,
    /// Custom namespaces discovered during parsing
    custom: HashMap<String, String>,
}

impl NamespaceRegistry {
    /// Create a new registry with common namespaces
    pub fn new() -> Self {
        let mut known = HashMap::new();

        // Common XMP namespaces
        known.insert("x", "adobe:ns:meta/");
        known.insert("xmp", "http://ns.adobe.com/xap/1.0/");
        known.insert("xmpRights", "http://ns.adobe.com/xap/1.0/rights/");
        known.insert("xmpMM", "http://ns.adobe.com/xap/1.0/mm/");
        known.insert("xmpBJ", "http://ns.adobe.com/xap/1.0/bj/");
        known.insert("xmpTPg", "http://ns.adobe.com/xap/1.0/t/pg/");
        known.insert("xmpDM", "http://ns.adobe.com/xap/1.0/DynamicMedia/");

        // Dublin Core
        known.insert("dc", "http://purl.org/dc/elements/1.1/");

        // EXIF and TIFF
        known.insert("exif", "http://ns.adobe.com/exif/1.0/");
        known.insert("tiff", "http://ns.adobe.com/tiff/1.0/");
        known.insert("exifEX", "http://cipa.jp/exif/1.0/");

        // Adobe applications
        known.insert("photoshop", "http://ns.adobe.com/photoshop/1.0/");
        known.insert("pdf", "http://ns.adobe.com/pdf/1.3/");
        known.insert("pdfx", "http://ns.adobe.com/pdfx/1.3/");

        // Camera and lens data
        known.insert("crs", "http://ns.adobe.com/camera-raw-settings/1.0/");
        known.insert("aux", "http://ns.adobe.com/exif/1.0/aux/");

        // RDF
        known.insert("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");

        // IPTC
        known.insert(
            "Iptc4xmpCore",
            "http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/",
        );
        known.insert("Iptc4xmpExt", "http://iptc.org/std/Iptc4xmpExt/2008-02-29/");

        // Other common namespaces
        known.insert("stJob", "http://ns.adobe.com/xap/1.0/sType/Job#");
        known.insert("stEvt", "http://ns.adobe.com/xap/1.0/sType/ResourceEvent#");
        known.insert("stRef", "http://ns.adobe.com/xap/1.0/sType/ResourceRef#");
        known.insert("stVer", "http://ns.adobe.com/xap/1.0/sType/Version#");
        known.insert("stDim", "http://ns.adobe.com/xap/1.0/sType/Dimensions#");

        Self {
            known,
            custom: HashMap::new(),
        }
    }

    /// Register a custom namespace
    pub fn register(&mut self, prefix: String, uri: String) {
        self.custom.insert(prefix, uri);
    }

    /// Get URI for a prefix
    pub fn get_uri(&self, prefix: &str) -> Option<&str> {
        self.known
            .get(prefix)
            .copied()
            .or_else(|| self.custom.get(prefix).map(|s| s.as_str()))
    }

    /// Check if a prefix is registered
    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.known.contains_key(prefix) || self.custom.contains_key(prefix)
    }

    /// Get all registered prefixes
    pub fn prefixes(&self) -> Vec<&str> {
        let mut prefixes: Vec<_> = self.known.keys().copied().collect();
        prefixes.extend(self.custom.keys().map(|s| s.as_str()));
        prefixes
    }
}

impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_namespaces() {
        let registry = NamespaceRegistry::new();

        // Test some common namespaces
        assert_eq!(
            registry.get_uri("dc"),
            Some("http://purl.org/dc/elements/1.1/")
        );
        assert_eq!(
            registry.get_uri("xmp"),
            Some("http://ns.adobe.com/xap/1.0/")
        );
        assert_eq!(
            registry.get_uri("rdf"),
            Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#")
        );
        assert_eq!(
            registry.get_uri("photoshop"),
            Some("http://ns.adobe.com/photoshop/1.0/")
        );
    }

    #[test]
    fn test_custom_namespace() {
        let mut registry = NamespaceRegistry::new();

        // Register custom namespace
        registry.register(
            "custom".to_string(),
            "http://example.com/custom/1.0/".to_string(),
        );

        assert_eq!(
            registry.get_uri("custom"),
            Some("http://example.com/custom/1.0/")
        );
        assert!(registry.has_prefix("custom"));
    }
}
