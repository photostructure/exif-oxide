//! GPS coordinate-based timezone inference
//!
//! This module provides timezone inference from GPS coordinates using
//! the tzf-rs timezone boundary database.

use crate::datetime::types::*;
use lazy_static::lazy_static;
use tzf_rs::DefaultFinder;

// Initialize timezone finder once for performance
lazy_static! {
    static ref FINDER: DefaultFinder = DefaultFinder::new();
}

/// GPS-based timezone inference engine
pub struct GpsTimezoneInference;

impl GpsTimezoneInference {
    /// Infer timezone from GPS coordinates
    ///
    /// Uses the tzf-rs timezone boundary database for accurate timezone lookup.
    /// This provides comprehensive global coverage with timezone boundary accuracy.
    pub fn infer_timezone(lat: f64, lng: f64) -> Option<InferenceSource> {
        // Validate GPS coordinates (reject 0,0 as invalid per exiftool-vendored)
        if lat.abs() < 0.0001 && lng.abs() < 0.0001 {
            return None;
        }

        // Validate coordinate ranges
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            return None;
        }

        // Use tzf-rs for accurate timezone lookup
        let timezone = FINDER.get_tz_name(lng, lat); // tzf-rs uses (lng, lat) order

        if timezone.is_empty() {
            return None; // No timezone found for these coordinates
        }

        Some(InferenceSource::GpsCoordinates {
            lat,
            lng,
            timezone: timezone.to_string(),
        })
    }

    /// Get timezone offset for specific coordinates and timestamp
    ///
    /// Uses tzf-rs to get the timezone name, then chrono to calculate the actual
    /// offset at the given timestamp (including DST transitions).
    pub fn get_timezone_offset(
        lat: f64,
        lng: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Option<i32> {
        // Get timezone name from coordinates
        let timezone_name = FINDER.get_tz_name(lng, lat);

        if timezone_name.is_empty() {
            // Fallback to rough longitude-based calculation
            let rough_offset_hours = (lng / 15.0).round() as i32;
            let offset_hours = rough_offset_hours.clamp(-12, 14);
            return Some(offset_hours * 60);
        }

        // Parse timezone using chrono
        use chrono_tz::Tz;
        if let Ok(tz) = timezone_name.parse::<Tz>() {
            // Get the offset at the specific timestamp (handles DST)
            let local_time = timestamp.with_timezone(&tz);
            // Get the offset by converting timezone offset to seconds
            // Note: chrono-tz uses different offset types, so we need to format and parse
            let offset_seconds = local_time
                .format("%z")
                .to_string()
                .parse::<i32>()
                .map(|h| h / 100 * 3600 + (h % 100) * 60) // Convert HHMM to seconds
                .unwrap_or_else(|_| {
                    // Fallback to rough calculation
                    ((lng / 15.0).round() as i32).clamp(-12, 14) * 3600
                });
            Some(offset_seconds / 60) // Convert to minutes
        } else {
            // Fallback to rough longitude-based calculation if timezone parsing fails
            let rough_offset_hours = (lng / 15.0).round() as i32;
            let offset_hours = rough_offset_hours.clamp(-12, 14);
            Some(offset_hours * 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_invalid_gps_coordinates() {
        // Should reject (0,0) as invalid
        assert!(GpsTimezoneInference::infer_timezone(0.0, 0.0).is_none());

        // Should reject out-of-range coordinates
        assert!(GpsTimezoneInference::infer_timezone(95.0, 0.0).is_none()); // Lat > 90
        assert!(GpsTimezoneInference::infer_timezone(0.0, 185.0).is_none()); // Lng > 180
    }

    #[test]
    fn test_simple_timezone_inference() {
        // Test New York coordinates
        let ny_result = GpsTimezoneInference::infer_timezone(40.7128, -74.0060);
        assert!(ny_result.is_some());
        if let Some(InferenceSource::GpsCoordinates { timezone, .. }) = ny_result {
            assert_eq!(timezone, "America/New_York");
        }

        // Test London coordinates
        let london_result = GpsTimezoneInference::infer_timezone(51.5074, -0.1278);
        assert!(london_result.is_some());
        if let Some(InferenceSource::GpsCoordinates { timezone, .. }) = london_result {
            assert_eq!(timezone, "Europe/London");
        }

        // Test Tokyo coordinates
        let tokyo_result = GpsTimezoneInference::infer_timezone(35.6762, 139.6503);
        assert!(tokyo_result.is_some());
        if let Some(InferenceSource::GpsCoordinates { timezone, .. }) = tokyo_result {
            assert_eq!(timezone, "Asia/Tokyo");
        }
    }

    #[test]
    fn test_timezone_offset_calculation() {
        // Test rough offset calculation
        let offset = GpsTimezoneInference::get_timezone_offset(
            40.7128,
            -74.0060, // New York
            chrono::Utc::now(),
        );
        assert!(offset.is_some());

        // Should be roughly -5 hours (-300 minutes) for New York longitude
        // Note: This is a very rough approximation
        let offset_hours = offset.unwrap() / 60;
        assert!((-7..=-4).contains(&offset_hours)); // Allow some variation
    }
}
