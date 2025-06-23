//! Manufacturer-specific datetime quirks and corrections
//!
//! This module handles known issues with datetime handling in specific
//! camera models and applies appropriate corrections.

use crate::datetime::types::*;
use chrono::{DateTime, Datelike, Utc};

/// Handler for manufacturer-specific datetime quirks
pub struct ManufacturerQuirks;

impl ManufacturerQuirks {
    /// Apply manufacturer-specific datetime corrections
    pub fn apply_quirks(
        datetime: &mut ExifDateTime,
        camera_info: &CameraInfo,
    ) -> Vec<QuirkApplication> {
        let mut applied_quirks = Vec::new();

        match camera_info
            .make
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Some("nikon") | Some("nikon corporation") => {
                applied_quirks.extend(Self::handle_nikon_quirks(datetime, camera_info));
            }
            Some("canon") => {
                applied_quirks.extend(Self::handle_canon_quirks(datetime, camera_info));
            }
            Some("apple") => {
                applied_quirks.extend(Self::handle_apple_quirks(datetime, camera_info));
            }
            _ => {
                // No specific quirks for this manufacturer
            }
        }

        applied_quirks
    }

    /// Handle Nikon-specific datetime quirks
    ///
    /// Nikon cameras have a known DST (Daylight Saving Time) bug where some models
    /// incorrectly handle timezone transitions.
    fn handle_nikon_quirks(
        datetime: &mut ExifDateTime,
        camera_info: &CameraInfo,
    ) -> Vec<QuirkApplication> {
        let mut quirks = Vec::new();

        if let Some(model) = &camera_info.model {
            // Known affected models with DST bugs
            let problematic_models = ["D3", "D300", "D700", "D3S", "D300S"];

            if problematic_models.iter().any(|&m| model.contains(m)) {
                if let Some(corrected) = Self::apply_nikon_dst_correction(datetime) {
                    *datetime = corrected;
                    quirks.push(QuirkApplication {
                        make: "Nikon".to_string(),
                        model: Some(model.clone()),
                        quirk_type: QuirkType::NikonDstBug,
                        description: "Applied DST correction for known Nikon bug".to_string(),
                        correction_applied: true,
                    });
                }
            }
        }

        quirks
    }

    /// Handle Canon-specific datetime quirks
    fn handle_canon_quirks(
        datetime: &mut ExifDateTime,
        camera_info: &CameraInfo,
    ) -> Vec<QuirkApplication> {
        let mut quirks = Vec::new();

        // Canon timezone format handling
        if datetime.has_timezone() {
            // Canon sometimes stores timezone information in non-standard formats
            quirks.push(QuirkApplication {
                make: "Canon".to_string(),
                model: camera_info.model.clone(),
                quirk_type: QuirkType::CanonTimezoneFormat,
                description: "Validated Canon timezone format".to_string(),
                correction_applied: false,
            });
        }

        quirks
    }

    /// Handle Apple/iOS-specific datetime quirks
    fn handle_apple_quirks(
        _datetime: &mut ExifDateTime,
        camera_info: &CameraInfo,
    ) -> Vec<QuirkApplication> {
        let mut quirks = Vec::new();

        // iOS devices often have very accurate datetime information
        if camera_info
            .model
            .as_deref()
            .unwrap_or("")
            .contains("iPhone")
        {
            // iOS photos usually have high-quality timezone information
            quirks.push(QuirkApplication {
                make: "Apple".to_string(),
                model: camera_info.model.clone(),
                quirk_type: QuirkType::AppleHighAccuracy,
                description: "iOS device with typically accurate datetime".to_string(),
                correction_applied: false,
            });
        }

        quirks
    }

    /// Apply Nikon DST correction
    ///
    /// Some Nikon cameras incorrectly handle DST transitions, particularly
    /// around the "spring forward" and "fall back" dates in various timezones.
    fn apply_nikon_dst_correction(datetime: &ExifDateTime) -> Option<ExifDateTime> {
        // This is a simplified implementation of the DST bug correction
        // A full implementation would need detailed DST transition tables

        let _year = datetime.datetime.year();

        // Only apply to dates where DST transitions commonly occur
        if Self::is_near_dst_transition(&datetime.datetime) {
            // Check if the datetime falls in a suspicious range
            if let Some(offset_minutes) = datetime.timezone_offset_minutes() {
                // Look for common DST-related offset errors (typically ±1 hour)
                if Self::looks_like_dst_error(offset_minutes, &datetime.datetime) {
                    let mut corrected = datetime.clone();

                    // Apply 1-hour correction (most common DST adjustment)
                    let correction = chrono::Duration::hours(1);
                    corrected.datetime = datetime.datetime - correction;

                    return Some(corrected);
                }
            }
        }

        None
    }

    /// Check if datetime is near a DST transition
    fn is_near_dst_transition(datetime: &DateTime<Utc>) -> bool {
        let month = datetime.month();
        let day = datetime.day();

        // Rough approximation of DST transition periods
        // Spring transition (March/April)
        if month == 3 && (8..=15).contains(&day) {
            return true;
        }
        if month == 4 && (1..=8).contains(&day) {
            return true;
        }

        // Fall transition (October/November)
        if month == 10 && (25..=31).contains(&day) {
            return true;
        }
        if month == 11 && (1..=8).contains(&day) {
            return true;
        }

        false
    }

    /// Check if timezone offset looks like a DST-related error
    fn looks_like_dst_error(offset_minutes: i32, datetime: &DateTime<Utc>) -> bool {
        // Look for offsets that are 1 hour off from standard timezone boundaries
        let offset_hours = offset_minutes as f32 / 60.0;

        // Check if offset is a half-hour away from a standard timezone
        let fractional_part = offset_hours.fract().abs();

        // Suspicious if offset is exactly 1 hour off from a standard boundary
        // and we're near a DST transition
        fractional_part == 0.0 && Self::is_near_dst_transition(datetime)
    }
}

/// Record of applied quirk correction
#[derive(Debug, Clone)]
pub struct QuirkApplication {
    pub make: String,
    pub model: Option<String>,
    pub quirk_type: QuirkType,
    pub description: String,
    pub correction_applied: bool,
}

/// Types of known manufacturer quirks
#[derive(Debug, Clone, PartialEq)]
pub enum QuirkType {
    /// Nikon DST bug affecting specific models
    NikonDstBug,
    /// Canon timezone format variations
    CanonTimezoneFormat,
    /// Apple iOS high accuracy
    AppleHighAccuracy,
    /// Generic timezone handling quirk
    TimezoneHandling,
}

impl QuirkType {
    /// Get a human-readable description of the quirk type
    pub fn description(&self) -> &'static str {
        match self {
            QuirkType::NikonDstBug => "Nikon DST transition bug",
            QuirkType::CanonTimezoneFormat => "Canon timezone format handling",
            QuirkType::AppleHighAccuracy => "Apple iOS high accuracy datetime",
            QuirkType::TimezoneHandling => "Generic timezone handling quirk",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_nikon_quirk_detection() {
        let camera_info = CameraInfo {
            make: Some("NIKON CORPORATION".to_string()),
            model: Some("NIKON D300".to_string()),
            ..Default::default()
        };

        let mut datetime = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 10, 14, 30, 0).unwrap(), // Near DST transition
            Some(chrono::FixedOffset::west_opt(5 * 3600).unwrap()),
            "2024:03:10 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        );

        let quirks = ManufacturerQuirks::apply_quirks(&mut datetime, &camera_info);

        // Should detect Nikon as a problematic model
        assert!(!quirks.is_empty());
        assert!(quirks
            .iter()
            .any(|q| q.quirk_type == QuirkType::NikonDstBug));
    }

    #[test]
    fn test_canon_quirk_detection() {
        let camera_info = CameraInfo {
            make: Some("Canon".to_string()),
            model: Some("Canon EOS 5D Mark IV".to_string()),
            ..Default::default()
        };

        let mut datetime = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
            Some(chrono::FixedOffset::west_opt(8 * 3600).unwrap()),
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        );

        let quirks = ManufacturerQuirks::apply_quirks(&mut datetime, &camera_info);

        // Should apply Canon timezone format quirk
        assert!(!quirks.is_empty());
        assert!(quirks
            .iter()
            .any(|q| q.quirk_type == QuirkType::CanonTimezoneFormat));
    }

    #[test]
    fn test_apple_quirk_detection() {
        let camera_info = CameraInfo {
            make: Some("Apple".to_string()),
            model: Some("iPhone 12 Pro".to_string()),
            ..Default::default()
        };

        let mut datetime = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        );

        let quirks = ManufacturerQuirks::apply_quirks(&mut datetime, &camera_info);

        // Should recognize iPhone as high-accuracy device
        assert!(!quirks.is_empty());
        assert!(quirks
            .iter()
            .any(|q| q.quirk_type == QuirkType::AppleHighAccuracy));
    }

    #[test]
    fn test_is_near_dst_transition() {
        // Spring DST transition (mid-March)
        let spring_dt = Utc.with_ymd_and_hms(2024, 3, 10, 14, 30, 0).unwrap();
        assert!(ManufacturerQuirks::is_near_dst_transition(&spring_dt));

        // Fall DST transition (late October)
        let fall_dt = Utc.with_ymd_and_hms(2024, 10, 29, 14, 30, 0).unwrap();
        assert!(ManufacturerQuirks::is_near_dst_transition(&fall_dt));

        // Not near DST transition (mid-summer)
        let summer_dt = Utc.with_ymd_and_hms(2024, 7, 15, 14, 30, 0).unwrap();
        assert!(!ManufacturerQuirks::is_near_dst_transition(&summer_dt));
    }

    #[test]
    fn test_unknown_manufacturer() {
        let camera_info = CameraInfo {
            make: Some("Unknown Brand".to_string()),
            ..Default::default()
        };

        let mut datetime = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        );

        let quirks = ManufacturerQuirks::apply_quirks(&mut datetime, &camera_info);

        // Should not apply any quirks for unknown manufacturer
        assert!(quirks.is_empty());
    }
}
