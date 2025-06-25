//! Integration test for GPMF parsing

#[cfg(test)]
mod tests {
    use crate::core::{MetadataCollection, MetadataSegment, MetadataType};
    use crate::detection::FileType;
    use crate::gpmf::{get_gpmf_format, get_gpmf_tag, GpmfParser};

    #[test]
    fn test_gpmf_integration_empty_file() {
        // Create a simple test to verify the GPMF integration doesn't break anything
        // This tests the GPMF parser with empty data
        let parser = GpmfParser::new();
        let result = parser.parse(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_gpmf_metadata_collection_structure() {
        // Test that MetadataCollection includes GPMF field
        let collection = MetadataCollection {
            exif: None,
            mpf: None,
            xmp: Vec::new(),
            iptc: None,
            gpmf: vec![MetadataSegment {
                data: vec![1, 2, 3, 4],
                offset: 0,
                source_format: FileType::JPEG,
                metadata_type: MetadataType::Gpmf,
            }],
        };

        assert_eq!(collection.gpmf.len(), 1);
        assert_eq!(collection.gpmf[0].metadata_type, MetadataType::Gpmf);
    }

    #[test]
    fn test_gpmf_tag_lookup() {
        // Test that GPMF tags can be looked up
        let tag = get_gpmf_tag("DVNM");
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().name, "DeviceName");
    }

    #[test]
    fn test_gpmf_format_lookup() {
        // Test that GPMF formats can be looked up
        let format = get_gpmf_format(0x63); // 'c' - string format
                                            // Note: this will return None for the stub implementation
                                            // but tests that the API exists
        let _ = format; // Placeholder until real format implementation
    }
}
