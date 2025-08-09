//! Tag management and source tracking
//!
//! This module handles tag storage, conflict resolution, and source tracking
//! for proper namespace handling and priority-based tag resolution.
//!
//! ExifTool Reference: Tag storage and conflict resolution logic

use crate::types::{TagSourceInfo, TagValue};
use tracing::debug;

use super::ExifReader;

impl ExifReader {
    /// Store tag with namespace-aware storage and conflict resolution
    /// ExifTool behavior: Tags with same ID but different contexts are stored separately
    pub fn store_tag_with_precedence(
        &mut self,
        tag_id: u16,
        value: TagValue,
        source_info: TagSourceInfo,
    ) {
        let key = (tag_id, source_info.namespace.clone());

        // Check if tag already exists in this namespace
        if let Some(existing_source) = self.tag_sources.get(&key) {
            // Compare priorities - higher priority wins within same namespace
            if source_info.priority > existing_source.priority {
                debug!(
                    "Tag 0x{:04x} ({}): Replacing lower priority with higher priority",
                    tag_id, source_info.namespace
                );
                self.extracted_tags.insert(key.clone(), value);
                self.tag_sources.insert(key, source_info);
            } else if source_info.priority == existing_source.priority {
                // Same priority - keep first encountered (ExifTool behavior)
                debug!(
                    "Tag 0x{:04x} ({}): Keeping first encountered",
                    tag_id, source_info.namespace
                );
                // Do not overwrite - keep existing
            } else {
                // Lower priority - ignore
                debug!(
                    "Tag 0x{:04x} ({}): Ignoring lower priority",
                    tag_id, source_info.namespace
                );
            }
        } else {
            // New tag - store it with namespace-aware key
            debug!(
                "Tag 0x{:04x} ({}): Storing new tag",
                tag_id, source_info.namespace
            );
            self.extracted_tags.insert(key.clone(), value);
            self.tag_sources.insert(key, source_info);
        }
    }

    /// Find tag value across namespaces with priority order
    /// Used for accessing tags that might exist in multiple contexts (like Make tag)
    pub(crate) fn get_tag_across_namespaces(&self, tag_id: u16) -> Option<&TagValue> {
        // Priority order: EXIF -> GPS -> MakerNotes
        // EXIF namespace has the highest priority for most tags
        let namespaces = ["EXIF", "GPS", "MakerNotes"];

        for namespace in namespaces {
            let key = (tag_id, namespace.to_string());
            if let Some(value) = self.extracted_tags.get(&key) {
                return Some(value);
            }
        }
        None
    }

    /// DEPRECATED: Legacy access method for backwards compatibility
    /// Other modules should migrate to get_tag_across_namespaces or store_tag_with_precedence
    pub fn legacy_get_tag(&self, tag_id: u16) -> Option<&TagValue> {
        self.get_tag_across_namespaces(tag_id)
    }

    /// DEPRECATED: Legacy storage method for backwards compatibility  
    /// Other modules should migrate to store_tag_with_precedence with proper namespace
    pub fn legacy_insert_tag(&mut self, tag_id: u16, value: TagValue, namespace: &str) {
        let source_info = self.create_tag_source_info(namespace);
        self.store_tag_with_precedence(tag_id, value, source_info);
    }

    /// Create TagSourceInfo from IFD name with proper namespace mapping
    /// Maps legacy IFD names to proper ExifTool group names
    pub(crate) fn create_tag_source_info(&self, ifd_name: &str) -> TagSourceInfo {
        // Map IFD names to ExifTool group names
        // ExifTool: lib/Image/ExifTool/Exif.pm group mappings
        let namespace = match ifd_name {
            "Root" | "IFD0" | "IFD1" => "EXIF",
            "GPS" => "GPS", // GPS tags need distinct namespace to avoid tag ID collisions
            "ExifIFD" => "EXIF", // ExifIFD tags belong to EXIF group (Group0) in ExifTool
            "InteropIFD" => "EXIF",
            "MakerNotes" => "MakerNotes",
            // Manufacturer-specific MakerNotes IFDs should use manufacturer namespace for Group1 assignment
            // ExifTool: Canon.pm tags get group1="Canon", Nikon.pm tags get group1="Nikon", etc.
            name if name.starts_with("Canon") => "Canon",
            name if name.starts_with("Nikon") => "Nikon",
            name if name.starts_with("Sony") => "Sony",
            name if name.starts_with("Olympus") => "Olympus",
            name if name.starts_with("Panasonic") => "Panasonic",
            name if name.starts_with("Fujifilm") => "Fujifilm",
            // RAW format-specific IFDs (maintain existing behavior)
            "KyoceraRaw" => "EXIF", // Kyocera RAW uses EXIF group
            _ => "EXIF",            // Default to EXIF for unknown IFDs
        };

        let processor_name = match namespace {
            "MakerNotes" => "MakerNotes".to_string(),
            // Manufacturer-specific MakerNotes processors
            "Canon" => "Canon".to_string(),
            "Nikon" => "Nikon".to_string(),
            "Sony" => "Sony".to_string(),
            "Olympus" => "Olympus".to_string(),
            "Panasonic" => "Panasonic".to_string(),
            "Fujifilm" => "Fujifilm".to_string(),
            _ => "Exif".to_string(),
        };

        TagSourceInfo::new(namespace.to_string(), ifd_name.to_string(), processor_name)
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    /// Returns tuple of (value, print) where:
    /// - value: The result after ValueConv (or raw if no ValueConv)
    /// - print: The result after PrintConv (or value if no PrintConv)
    pub(crate) fn apply_conversions(
        &self,
        raw_value: &TagValue,
        tag_id: u16,
        ifd_name: &str,
        source_info: Option<&TagSourceInfo>,
    ) -> (TagValue, TagValue) {
        use crate::expressions::ExpressionEvaluator;
        use crate::generated::Exif_pm::main_tags;
        use crate::generated::GPS_pm::main_tags as gps_tag_kit;
        use crate::generated::Sony_pm::main_tags as sony_tag_kit;

        let mut value = raw_value.clone();

        // Apply RawConv first for special tags that need character encoding or data processing
        // ExifTool: RawConv is applied before ValueConv/PrintConv
        if tag_id == 0x9286 {
            // UserComment tag needs RawConv for character encoding
            use crate::registry;
            value = registry::apply_raw_conv("convert_exif_text", &value);
        }

        // Process based on IFD context
        if ifd_name == "GPS" {
            // For GPS IFD, check GPS tag kit
            if let Some(tag_def) = gps_tag_kit::GPS_MAIN_TAGS.get(&tag_id) {
                // Apply ValueConv first (if present) using generated function
                if tag_def.value_conv.is_some() {
                    let mut value_conv_errors = Vec::new();
                    match gps_tag_kit::apply_value_conv(
                        tag_id as u32,
                        &value,
                        &mut value_conv_errors,
                    ) {
                        Ok(converted) => {
                            debug!(
                                "Applied ValueConv to GPS tag 0x{:04x}: {:?} -> {:?}",
                                tag_id, value, converted
                            );
                            value = converted;
                        }
                        Err(e) => {
                            debug!(
                                "Failed to apply ValueConv to GPS tag 0x{:04x}: {}",
                                tag_id, e
                            );
                        }
                    }
                }

                // Apply PrintConv second (if present) to get display value
                // For GPS coordinates, we want to use our manual registry functions
                let print = match tag_def.name {
                    "GPSLatitude" => {
                        // Note: Keep raw EXIF GPS coordinates unsigned - sign handling happens in composite tags
                        // ExifTool: Raw GPS coordinates in EXIF are always positive rationals
                        // Convert rational to decimal if needed (unsigned, for composite tag consumption)
                        if let Some(rational) = value.as_rational() {
                            TagValue::F64(rational.0 as f64 / rational.1 as f64)
                        } else {
                            value.clone()
                        }
                    }
                    "GPSLongitude" => {
                        // Note: Keep raw EXIF GPS coordinates unsigned - sign handling happens in composite tags
                        // ExifTool: Raw GPS coordinates in EXIF are always positive rationals
                        // Convert rational to decimal if needed (unsigned, for composite tag consumption)
                        if let Some(rational) = value.as_rational() {
                            TagValue::F64(rational.0 as f64 / rational.1 as f64)
                        } else {
                            value.clone()
                        }
                    }
                    "GPSDestLatitude" => {
                        use crate::registry;
                        registry::apply_print_conv("gpsdestlatitude_print_conv", &value)
                    }
                    "GPSDestLongitude" => {
                        use crate::registry;
                        registry::apply_print_conv("gpsdestlongitude_print_conv", &value)
                    }
                    "GPSAltitude" => {
                        // Note: Skip PrintConv for GPSAltitude because our test snapshots use
                        // ExifTool's -GPSAltitude# flag which outputs converted decimal values
                        // without PrintConv units to match PhotoStructure DAM requirements

                        // Convert rational to decimal if needed (matching ExifTool -GPSAltitude# behavior)
                        if let Some(rational) = value.as_rational() {
                            TagValue::F64(rational.0 as f64 / rational.1 as f64)
                        } else {
                            value.clone()
                        }
                    }
                    "GPSHPositioningError" => {
                        // Expression: "\"$val m\""
                        TagValue::String(format!("{} m", value))
                    }
                    _ => {
                        // Use the generic tag kit apply_print_conv for other GPS tags
                        let mut evaluator = ExpressionEvaluator::new();
                        let mut errors = Vec::new();
                        let mut warnings = Vec::new();

                        let result = gps_tag_kit::apply_print_conv(
                            tag_id as u32,
                            &value,
                            &mut evaluator,
                            &mut errors,
                            &mut warnings,
                        );

                        // Log any warnings
                        for warning in warnings {
                            debug!("PrintConv warning for tag 0x{:04x}: {}", tag_id, warning);
                        }

                        result
                    }
                };

                (value, print)
            } else {
                // No tag definition found, return raw value for both
                (value.clone(), value)
            }
        } else if ifd_name == "Sony"
            || (source_info.is_some() && source_info.unwrap().namespace == "Sony")
        {
            // Debug logging for Sony context detection
            debug!(
                "Sony context check - ifd_name: '{}', source_info: {:?}",
                ifd_name,
                source_info.map(|si| format!(
                    "namespace: '{}', ifd_name: '{}'",
                    si.namespace, si.ifd_name
                ))
            );
            // For Sony IFD, check Sony tag kit
            if let Some(tag_def) = sony_tag_kit::SONY_MAIN_TAGS.get(&tag_id) {
                debug!(
                    "Found Sony tag definition for tag 0x{:04x}: {}",
                    tag_id, tag_def.name
                );
                // Apply ValueConv first (if present) using generated function
                if tag_def.value_conv.is_some() {
                    let mut value_conv_errors = Vec::new();
                    match sony_tag_kit::apply_value_conv(
                        tag_id as u32,
                        &value,
                        &mut value_conv_errors,
                    ) {
                        Ok(converted) => {
                            debug!(
                                "Applied ValueConv to Sony tag 0x{:04x}: {:?} -> {:?}",
                                tag_id, value, converted
                            );
                            value = converted;
                        }
                        Err(e) => {
                            debug!(
                                "Failed to apply ValueConv to Sony tag 0x{:04x}: {}",
                                tag_id, e
                            );
                        }
                    }
                }

                // Apply PrintConv second (if present) to get display value
                let mut evaluator = ExpressionEvaluator::new();
                let mut errors = Vec::new();
                let mut warnings = Vec::new();

                let print = sony_tag_kit::apply_print_conv(
                    tag_id as u32,
                    &value,
                    &mut evaluator,
                    &mut errors,
                    &mut warnings,
                );

                // Log any warnings (but suppress if we successfully applied fallback)
                if print == value {
                    for warning in warnings {
                        debug!(
                            "PrintConv warning for Sony tag 0x{:04x}: {}",
                            tag_id, warning
                        );
                    }
                }

                (value, print)
            } else {
                // No Sony tag definition found, return raw value for both
                debug!("Sony tag 0x{:04x} not found in SONY_PM_TAG_KITS", tag_id);
                (value.clone(), value)
            }
        } else {
            // For other IFDs, check EXIF tag kit
            if let Some(tag_def) = main_tags::EXIF_MAIN_TAGS.get(&tag_id) {
                debug!(
                    "Found tag definition for tag 0x{:04x}: {}",
                    tag_id, tag_def.name
                );
                // Apply ValueConv first (if present) using generated function
                if tag_def.value_conv.is_some() {
                    let mut value_conv_errors = Vec::new();
                    match main_tags::apply_value_conv(tag_id as u32, &value, &mut value_conv_errors)
                    {
                        Ok(converted) => {
                            debug!(
                                "Applied ValueConv to EXIF tag 0x{:04x}: {:?} -> {:?}",
                                tag_id, value, converted
                            );
                            value = converted;
                        }
                        Err(e) => {
                            debug!(
                                "Failed to apply ValueConv to EXIF tag 0x{:04x}: {}",
                                tag_id, e
                            );
                        }
                    }
                }

                // Apply PrintConv second (if present) to get display value
                let mut evaluator = ExpressionEvaluator::new();
                let mut errors = Vec::new();
                let mut warnings = Vec::new();

                let print = match tag_def.name {
                    "SubSecTime" | "SubSecTimeOriginal" | "SubSecTimeDigitized" => {
                        // Note: These are stored as strings in TIFF but ExifTool outputs them as
                        // numbers in JSON when they contain only digits (matching ExifTool's JSON behavior)
                        if let Some(s) = value.as_string() {
                            // Apply ValueConv: trim trailing whitespace like ExifTool
                            let trimmed = s.trim_end();
                            TagValue::string_with_numeric_detection(trimmed)
                        } else {
                            value.clone()
                        }
                    }
                    _ => {
                        // Use EXIF tag kit PrintConv - call the specific function from generated EXIF module
                        main_tags::apply_print_conv(
                            tag_id as u32,
                            &value,
                            &mut evaluator,
                            &mut errors,
                            &mut warnings,
                        )
                    }
                };

                // Log any warnings (but suppress if we successfully applied fallback)
                if print == value {
                    for warning in warnings {
                        debug!("PrintConv warning for tag 0x{:04x}: {}", tag_id, warning);
                    }
                }

                (value, print)
            } else {
                // No tag definition found, return raw value for both
                (value.clone(), value)
            }
        }
    }
}
