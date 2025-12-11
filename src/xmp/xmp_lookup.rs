//! XMP tag lookup using generated namespace tables
//!
//! This module provides lookup functions that map XMP namespace prefixes and
//! property names to their canonical tag information from ExifTool's XMP definitions.

use crate::core::XmpTagInfo;

// Import all generated XMP namespace tables
use crate::generated::XMP_pm::{
    album_tags::XMP_ALBUM_TAGS, aux_tags::XMP_AUX_TAGS, cc_tags::XMP_CC_TAGS,
    crs_tags::XMP_CRS_TAGS, dc_tags::XMP_DC_TAGS, exif_ex_tags::XMP_EXIF_EX_TAGS,
    exif_tags::XMP_EXIF_TAGS, exif_tool_tags::XMP_ET_TAGS, iptc_core_tags::XMP_IPTC4XMP_CORE_TAGS,
    iptc_ext_tags::XMP_IPTC4XMP_EXT_TAGS, lightroom_tags::XMP_LR_TAGS,
    media_pro_tags::XMP_MEDIAPRO_TAGS, pdf_tags::XMP_PDF_TAGS, pdfx_tags::XMP_PDFX_TAGS,
    photoshop_tags::XMP_PHOTOSHOP_TAGS, rdf_tags::XMP_RDF_TAGS, s_area_tags::XMP_ST_AREA_TAGS,
    s_dimensions_tags::XMP_ST_DIM_TAGS, s_font_tags::XMP_ST_FNT_TAGS,
    s_job_ref_tags::XMP_ST_JOB_TAGS, s_manifest_item_tags::XMP_ST_MFS_TAGS,
    s_resource_event_tags::XMP_ST_EVT_TAGS, s_resource_ref_tags::XMP_ST_REF_TAGS,
    s_version_tags::XMP_ST_VER_TAGS, tiff_tags::XMP_TIFF_TAGS, x_tags::XMP_X_TAGS,
    xmp_bj_tags::XMP_XMP_BJ_TAGS, xmp_mm_tags::XMP_XMP_MM_TAGS, xmp_note_tags::XMP_XMP_NOTE_TAGS,
    xmp_rights_tags::XMP_XMP_RIGHTS_TAGS, xmp_tags::XMP_XMP_TAGS, xmp_tpg_tags::XMP_XMP_TPG_TAGS,
};

// Import MWG namespace tables
use crate::generated::MWG_pm::{keywords_tags::XMP_MWG_KW_TAGS, regions_tags::XMP_MWG_RS_TAGS};

/// Look up XMP tag information from generated tables
///
/// Maps namespace prefix + property name to XmpTagInfo.
/// The namespace prefix should match ExifTool's namespace naming (e.g., "dc", "xmp", "tiff").
///
/// # Arguments
/// * `namespace` - The namespace prefix (e.g., "dc", "tiff", "exif")
/// * `property` - The property name as it appears in the XML (e.g., "title", "ImageWidth")
///
/// # Returns
/// * `Some(&XmpTagInfo)` if the tag is found in generated tables
/// * `None` if the namespace or property is not in generated tables
pub fn lookup_xmp_tag(namespace: &str, property: &str) -> Option<&'static XmpTagInfo> {
    match namespace {
        // Core XMP namespaces
        "dc" => XMP_DC_TAGS.get(property),
        "xmp" => XMP_XMP_TAGS.get(property),
        "xmpRights" => XMP_XMP_RIGHTS_TAGS.get(property),
        "xmpMM" => XMP_XMP_MM_TAGS.get(property),
        "xmpBJ" => XMP_XMP_BJ_TAGS.get(property),
        "xmpNote" => XMP_XMP_NOTE_TAGS.get(property),
        "xmpTPg" => XMP_XMP_TPG_TAGS.get(property),

        // TIFF/EXIF in XMP
        "tiff" => XMP_TIFF_TAGS.get(property),
        "exif" => XMP_EXIF_TAGS.get(property),
        "exifEX" => XMP_EXIF_EX_TAGS.get(property),
        "aux" => XMP_AUX_TAGS.get(property),

        // Adobe applications
        "photoshop" => XMP_PHOTOSHOP_TAGS.get(property),
        "crs" => XMP_CRS_TAGS.get(property),
        "lr" => XMP_LR_TAGS.get(property),

        // IPTC
        "Iptc4xmpCore" | "iptc4xmpCore" => XMP_IPTC4XMP_CORE_TAGS.get(property),
        "Iptc4xmpExt" | "iptc4xmpExt" => XMP_IPTC4XMP_EXT_TAGS.get(property),

        // Creative Commons (from XMP2.pl)
        "cc" => XMP_CC_TAGS.get(property),

        // iView MediaPro (from XMP2.pl)
        "mediapro" => XMP_MEDIAPRO_TAGS.get(property),

        // Metadata Working Group (from MWG.pm)
        "mwg-rs" => XMP_MWG_RS_TAGS.get(property),
        "mwg-kw" => XMP_MWG_KW_TAGS.get(property),

        // PDF
        "pdf" => XMP_PDF_TAGS.get(property),
        "pdfx" => XMP_PDFX_TAGS.get(property),

        // Other namespaces
        "album" => XMP_ALBUM_TAGS.get(property),
        "rdf" => XMP_RDF_TAGS.get(property),
        "x" => XMP_X_TAGS.get(property),
        "et" => XMP_ET_TAGS.get(property),

        // Structure type namespaces (stArea, stDim, etc.)
        "stArea" => XMP_ST_AREA_TAGS.get(property),
        "stDim" => XMP_ST_DIM_TAGS.get(property),
        "stFnt" => XMP_ST_FNT_TAGS.get(property),
        "stJob" => XMP_ST_JOB_TAGS.get(property),
        "stMfs" => XMP_ST_MFS_TAGS.get(property),
        "stEvt" => XMP_ST_EVT_TAGS.get(property),
        "stRef" => XMP_ST_REF_TAGS.get(property),
        "stVer" => XMP_ST_VER_TAGS.get(property),

        _ => None,
    }
}

/// Get the canonical tag name for an XMP property
///
/// Returns the ExifTool-style tag name from generated tables,
/// or falls back to capitalizing the first letter of the property name.
///
/// # Arguments
/// * `namespace` - The namespace prefix (e.g., "dc", "tiff")
/// * `property` - The property name as it appears in XML
///
/// # Returns
/// The canonical tag name (e.g., "title" -> "Title", "ImageLength" -> "ImageHeight")
pub fn get_xmp_tag_name(namespace: &str, property: &str) -> String {
    if let Some(tag_info) = lookup_xmp_tag(namespace, property) {
        tag_info.name.to_string()
    } else {
        // Fallback: capitalize first letter
        let mut chars = property.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => property.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_dc_tags() {
        // dc namespace uses lowercase keys
        let title = lookup_xmp_tag("dc", "title");
        assert!(title.is_some());
        assert_eq!(title.unwrap().name, "Title");

        let creator = lookup_xmp_tag("dc", "creator");
        assert!(creator.is_some());
        assert_eq!(creator.unwrap().name, "Creator");
    }

    #[test]
    fn test_lookup_tiff_tags() {
        // tiff namespace uses PascalCase keys
        let orientation = lookup_xmp_tag("tiff", "Orientation");
        assert!(orientation.is_some());
        assert_eq!(orientation.unwrap().name, "Orientation");
        // Should have PrintConv
        assert!(orientation.unwrap().print_conv.is_some());

        // ImageLength is renamed to ImageHeight
        let image_length = lookup_xmp_tag("tiff", "ImageLength");
        assert!(image_length.is_some());
        assert_eq!(image_length.unwrap().name, "ImageHeight");
    }

    #[test]
    fn test_lookup_xmp_tags() {
        let rating = lookup_xmp_tag("xmp", "Rating");
        assert!(rating.is_some());
        assert_eq!(rating.unwrap().name, "Rating");

        let create_date = lookup_xmp_tag("xmp", "CreateDate");
        assert!(create_date.is_some());
        assert_eq!(create_date.unwrap().name, "CreateDate");
    }

    #[test]
    fn test_lookup_unknown_returns_none() {
        assert!(lookup_xmp_tag("unknown_ns", "property").is_none());
        assert!(lookup_xmp_tag("dc", "nonexistent_property").is_none());
    }

    #[test]
    fn test_get_tag_name_from_generated() {
        assert_eq!(get_xmp_tag_name("dc", "title"), "Title");
        assert_eq!(get_xmp_tag_name("tiff", "ImageLength"), "ImageHeight");
    }

    #[test]
    fn test_get_tag_name_fallback() {
        // Unknown property - should capitalize first letter
        assert_eq!(get_xmp_tag_name("dc", "unknownProperty"), "UnknownProperty");
        assert_eq!(get_xmp_tag_name("unknown", "property"), "Property");
    }

    #[test]
    fn test_iptc_case_insensitive() {
        // Both cases should work for IPTC
        assert!(lookup_xmp_tag("Iptc4xmpCore", "Location").is_some());
        assert!(lookup_xmp_tag("iptc4xmpCore", "Location").is_some());
    }

    #[test]
    fn test_lookup_cc_tags() {
        // Creative Commons namespace (from XMP2.pl)
        let license = lookup_xmp_tag("cc", "license");
        assert!(license.is_some());
        assert_eq!(license.unwrap().name, "License");

        let attribution = lookup_xmp_tag("cc", "attributionName");
        assert!(attribution.is_some());
        assert_eq!(attribution.unwrap().name, "AttributionName");

        // Permits has PrintConv
        let permits = lookup_xmp_tag("cc", "permits");
        assert!(permits.is_some());
        assert_eq!(permits.unwrap().name, "Permits");
        assert!(permits.unwrap().print_conv.is_some());
    }

    #[test]
    fn test_lookup_mediapro_tags() {
        // iView MediaPro namespace (from XMP2.pl)
        let people = lookup_xmp_tag("mediapro", "People");
        assert!(people.is_some());
        assert_eq!(people.unwrap().name, "People");
    }

    #[test]
    fn test_lookup_iptc_ext_tags() {
        // IPTC Extensions namespace (from XMP2.pl)
        let person = lookup_xmp_tag("Iptc4xmpExt", "PersonInImage");
        assert!(person.is_some());
        assert_eq!(person.unwrap().name, "PersonInImage");
    }

    #[test]
    fn test_lookup_mwg_tags() {
        // MWG Regions namespace (from MWG.pm)
        let regions = lookup_xmp_tag("mwg-rs", "RegionsRegionList");
        assert!(regions.is_some());
        assert_eq!(regions.unwrap().name, "RegionList");

        // MWG Keywords namespace (from MWG.pm)
        let keywords = lookup_xmp_tag("mwg-kw", "Keywords");
        assert!(keywords.is_some());
        assert_eq!(keywords.unwrap().name, "KeywordInfo");
    }
}
