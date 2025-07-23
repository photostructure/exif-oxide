//! Tag kit splitting logic for managing file sizes
//!
//! Splits tag kits into logical groups to keep generated files under 2000 lines

use std::collections::HashMap;

/// Category for a tag based on its ID and semantic meaning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TagCategory {
    /// Core image properties (dimensions, format, etc) - IDs 254-400
    Core,
    /// Camera and device info - IDs 271-272, 305-306, 315-316, etc
    Camera,
    /// Color and rendering - IDs 318-319, 320, 528-535, etc
    Color,
    /// Document and description - IDs 269-270, 285-300 
    Document,
    /// Date/time related - IDs 306, 36867-36868, etc
    DateTime,
    /// GPS and location - Would be in GPS IFD, not main
    GPS,
    /// Thumbnail and preview - IDs 513-514, etc
    Thumbnail,
    /// EXIF specific tags - IDs 33434-37500
    ExifSpecific,
    /// Interoperability - IDs 1-2, 4096-5120
    Interop,
    /// Windows XP tags - IDs 40091-40095, 18246-18249
    WindowsXP,
    /// Other/Unknown
    Other,
}

impl TagCategory {
    /// Determine category for a tag based on ID and name
    pub fn categorize(tag_id: u32, tag_name: &str) -> Self {
        // First check by name patterns
        if tag_name.starts_with("GPS") {
            return TagCategory::GPS;
        }
        if tag_name.contains("Date") || tag_name.contains("Time") {
            return TagCategory::DateTime;
        }
        if tag_name.starts_with("Thumbnail") || tag_name.contains("Thumbnail") {
            return TagCategory::Thumbnail;
        }
        if tag_name.starts_with("XP") || tag_name.starts_with("XP_") {
            return TagCategory::WindowsXP;
        }
        if tag_name.starts_with("Interop") {
            return TagCategory::Interop;
        }
        
        // Then by ID ranges
        match tag_id {
            // Interop tags
            1..=10 => TagCategory::Interop,
            
            // Core image structure
            254..=267 => TagCategory::Core,
            273..=284 => TagCategory::Core,
            296..=297 => TagCategory::Core,
            
            // Document metadata
            269..=270 => TagCategory::Document,
            285..=295 => TagCategory::Document,
            298..=300 => TagCategory::Document,
            
            // Camera/software info
            271..=272 => TagCategory::Camera,
            305..=306 => TagCategory::Camera,
            315..=317 => TagCategory::Camera,
            
            // Color/rendering
            301 => TagCategory::Color,
            318..=335 => TagCategory::Color,
            528..=535 => TagCategory::Color,
            
            // Date/time (ModifyDate)
            306 => TagCategory::DateTime,
            
            // Thumbnail/preview
            513..=514 => TagCategory::Thumbnail,
            
            // EXIF specific range
            33434..=37500 => TagCategory::ExifSpecific,
            
            // Interop range
            4096..=5120 => TagCategory::Interop,
            
            // Windows XP tags
            18246..=18249 => TagCategory::WindowsXP,
            40091..=40095 => TagCategory::WindowsXP,
            
            // GPS would be in GPS IFD
            0..=31 if tag_name.starts_with("GPS") => TagCategory::GPS,
            
            _ => TagCategory::Other,
        }
    }
    
    /// Get the module name for this category
    pub fn module_name(&self) -> &'static str {
        match self {
            TagCategory::Core => "core",
            TagCategory::Camera => "camera", 
            TagCategory::Color => "color",
            TagCategory::Document => "document",
            TagCategory::DateTime => "datetime",
            TagCategory::GPS => "gps",
            TagCategory::Thumbnail => "thumbnail",
            TagCategory::ExifSpecific => "exif_specific",
            TagCategory::Interop => "interop",
            TagCategory::WindowsXP => "windows_xp",
            TagCategory::Other => "other",
        }
    }
}

/// Split tag kits into categorized groups
pub fn split_tag_kits(tag_kits: &[crate::schemas::tag_kit::TagKit]) -> HashMap<TagCategory, Vec<&crate::schemas::tag_kit::TagKit>> {
    let mut categories: HashMap<TagCategory, Vec<&crate::schemas::tag_kit::TagKit>> = HashMap::new();
    
    for tag_kit in tag_kits {
        let tag_id = tag_kit.tag_id.parse::<u32>().unwrap_or(0);
        let category = TagCategory::categorize(tag_id, &tag_kit.name);
        categories.entry(category).or_insert_with(Vec::new).push(tag_kit);
    }
    
    categories
}

/// Count PrintConv tables that will be generated for a set of tag kits
pub fn count_print_conv_tables(tag_kits: &[&crate::schemas::tag_kit::TagKit]) -> usize {
    tag_kits.iter()
        .filter(|tk| tk.print_conv_type == "Simple")
        .count()
}