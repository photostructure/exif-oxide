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
    /// Store tag with conflict resolution and proper namespace handling
    /// ExifTool behavior: Main EXIF tags take precedence over MakerNote tags with same ID
    pub fn store_tag_with_precedence(
        &mut self,
        tag_id: u16,
        value: TagValue,
        source_info: TagSourceInfo,
    ) {
        // Check if tag already exists
        if let Some(existing_source) = self.tag_sources.get(&tag_id) {
            // Compare priorities - higher priority wins
            if source_info.priority > existing_source.priority {
                debug!(
                    "Tag 0x{:04x}: Replacing lower priority {} with higher priority {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                self.extracted_tags.insert(tag_id, value);
                self.tag_sources.insert(tag_id, source_info);
            } else if source_info.priority == existing_source.priority {
                // Same priority - keep first encountered (ExifTool behavior)
                debug!(
                    "Tag 0x{:04x}: Keeping first encountered {} over {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                // Do not overwrite - keep existing
            } else {
                // Lower priority - ignore
                debug!(
                    "Tag 0x{:04x}: Ignoring lower priority {} (existing: {})",
                    tag_id, source_info.namespace, existing_source.namespace
                );
            }
        } else {
            // New tag - store it
            debug!(
                "Tag 0x{:04x}: Storing new {} tag",
                tag_id, source_info.namespace
            );
            self.extracted_tags.insert(tag_id, value);
            self.tag_sources.insert(tag_id, source_info);
        }
    }

    /// Create TagSourceInfo from IFD name with proper namespace mapping
    /// Maps legacy IFD names to proper ExifTool group names
    pub(crate) fn create_tag_source_info(&self, ifd_name: &str) -> TagSourceInfo {
        // Map IFD names to ExifTool group names
        // ExifTool: lib/Image/ExifTool/Exif.pm group mappings
        let namespace = match ifd_name {
            "Root" | "IFD0" | "IFD1" => "EXIF",
            "GPS" => "EXIF",     // GPS tags belong to EXIF group in ExifTool
            "ExifIFD" => "EXIF", // ExifIFD tags belong to EXIF group (Group0) in ExifTool
            "InteropIFD" => "EXIF",
            "MakerNotes" => "MakerNotes",
            // Manufacturer-specific MakerNotes IFDs should use MakerNotes namespace
            // ExifTool: Canon.pm, Nikon.pm, Sony.pm, etc. all use MakerNotes group
            name if name.starts_with("Canon") => "MakerNotes",
            name if name.starts_with("Nikon") => "MakerNotes",
            name if name.starts_with("Sony") => "MakerNotes",
            name if name.starts_with("Olympus") => "MakerNotes",
            name if name.starts_with("Panasonic") => "MakerNotes",
            name if name.starts_with("Fujifilm") => "MakerNotes",
            // RAW format-specific IFDs (maintain existing behavior)
            "KyoceraRaw" => "EXIF", // Kyocera RAW uses EXIF group
            _ => "EXIF",            // Default to EXIF for unknown IFDs
        };

        let processor_name = if namespace == "MakerNotes" {
            // For MakerNotes, use a generic MakerNotes processor name
            "MakerNotes".to_string()
        } else {
            "Exif".to_string()
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
    ) -> (TagValue, TagValue) {
        use crate::expressions::ExpressionEvaluator;
        use crate::generated::Exif_pm::tag_kit;
        use crate::generated::GPS_pm::tag_kit as gps_tag_kit;

        let mut value = raw_value.clone();

        // Process based on IFD context
        if ifd_name == "GPS" {
            // For GPS IFD, check GPS tag kit
            if let Some(tag_def) = gps_tag_kit::GPS_PM_TAG_KITS.get(&(tag_id as u32)) {
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
                        use crate::registry;
                        registry::apply_print_conv("gpslatitude_print_conv", &value)
                    }
                    "GPSLongitude" => {
                        use crate::registry;
                        registry::apply_print_conv("gpslongitude_print_conv", &value)
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
                        // Expression: "$val =~ /^(inf|undef)$/ ? $val : \"$val m\""
                        match value.as_f64() {
                            Some(v) if v.is_infinite() => TagValue::String("inf".to_string()),
                            Some(v) if v.is_nan() => TagValue::String("undef".to_string()),
                            Some(v) => TagValue::String(format!("{} m", v)),
                            None => {
                                if let Some(s) = value.as_string() {
                                    if s == "inf" || s == "undef" {
                                        TagValue::String(s.to_string())
                                    } else {
                                        TagValue::String(format!("{} m", s))
                                    }
                                } else {
                                    TagValue::String(format!("{} m", value))
                                }
                            }
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
        } else {
            // For other IFDs, check EXIF tag kit
            if let Some(tag_def) = tag_kit::EXIF_PM_TAG_KITS.get(&(tag_id as u32)) {
                // Apply ValueConv first (if present) using generated function
                if tag_def.value_conv.is_some() {
                    let mut value_conv_errors = Vec::new();
                    match tag_kit::apply_value_conv(tag_id as u32, &value, &mut value_conv_errors) {
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

                let print = tag_kit::apply_print_conv(
                    tag_id as u32,
                    &value,
                    &mut evaluator,
                    &mut errors,
                    &mut warnings,
                );

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
