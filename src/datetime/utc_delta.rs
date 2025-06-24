//! UTC delta calculation for timezone inference
//!
//! This module implements timezone inference by comparing local timestamps
//! with UTC references like GPS datetime.

use crate::datetime::types::*;

/// Calculator for timezone offsets based on UTC timestamp deltas
pub struct UtcDeltaCalculator;

impl UtcDeltaCalculator {
    /// Calculate timezone offset from GPS datetime delta
    ///
    /// Compares a local datetime (like DateTimeOriginal) with a UTC reference
    /// (like GPSDateTime) to infer the timezone offset.
    pub fn calculate_offset_from_gps_delta(
        local_time: &ExifDateTime,
        gps_datetime: &ExifDateTime,
    ) -> Option<InferenceSource> {
        // Calculate the difference in seconds
        let local_timestamp = local_time.datetime.timestamp();
        let utc_timestamp = gps_datetime.datetime.timestamp();
        let delta_seconds = local_timestamp - utc_timestamp;

        // Convert to minutes
        let delta_minutes = delta_seconds / 60;

        // Validate delta is within reasonable range (±14 hours = ±840 minutes)
        if delta_minutes.abs() > 14 * 60 {
            return None;
        }

        // Round to nearest 15-minute boundary (most timezones align to this)
        let rounded_delta = Self::round_to_timezone_boundary(delta_minutes as i32);

        Some(InferenceSource::UtcDelta {
            reference_tag: "GPSDateTime".to_string(),
            delta_minutes: rounded_delta,
        })
    }

    /// Find UTC reference timestamps in datetime collection
    pub fn find_utc_references(collection: &DateTimeCollection) -> Vec<UtcReference> {
        let mut references = Vec::new();

        // GPS datetime is always UTC
        if let Some(gps_dt) = &collection.gps_datetime {
            references.push(UtcReference {
                datetime: gps_dt.clone(),
                tag_name: "GPSDateTime".to_string(),
                confidence: 0.95, // High confidence - GPS is always UTC
            });
        }

        // TODO: Add other UTC reference sources like:
        // - DateTimeUTC (explicit UTC timestamp)
        // - TimeStamp (Unix timestamp, inherently UTC)

        references
    }

    /// Validate that a timezone delta is reasonable
    pub fn validate_timezone_delta(delta_minutes: i32) -> bool {
        // Check range (±14 hours per RFC 3339)
        if delta_minutes.abs() > 14 * 60 {
            return false;
        }

        // Check if delta aligns with known timezone boundaries
        // Most timezones are multiples of 15 or 30 minutes
        Self::is_valid_timezone_offset(delta_minutes)
    }

    /// Round delta to nearest valid timezone boundary
    fn round_to_timezone_boundary(delta_minutes: i32) -> i32 {
        // Most timezones are on 15-minute boundaries
        let remainder = delta_minutes % 15;

        if remainder.abs() <= 7 {
            // Round down to nearest 15-minute boundary
            delta_minutes - remainder
        } else {
            // Round up to nearest 15-minute boundary
            delta_minutes + (15 - remainder.abs()) * remainder.signum()
        }
    }

    /// Check if offset aligns with known timezone boundaries
    fn is_valid_timezone_offset(offset_minutes: i32) -> bool {
        // List of known timezone offsets (in minutes)
        // Based on valid timezone offsets from exiftool-vendored
        let valid_offsets = [
            -11 * 60,
            -10 * 60,
            -9 * 60 - 30,
            -9 * 60,
            -8 * 60 - 30,
            -8 * 60,
            -7 * 60,
            -6 * 60,
            -5 * 60,
            -4 * 60 - 30,
            -4 * 60,
            -3 * 60 - 30,
            -3 * 60,
            -2 * 60 - 30,
            -2 * 60,
            -60,
            0,
            60,
            2 * 60,
            3 * 60,
            3 * 60 + 30,
            4 * 60,
            4 * 60 + 30,
            5 * 60,
            5 * 60 + 30,
            5 * 60 + 45,
            6 * 60,
            6 * 60 + 30,
            7 * 60,
            7 * 60 + 30,
            8 * 60,
            8 * 60 + 30,
            8 * 60 + 45,
            9 * 60,
            9 * 60 + 30,
            9 * 60 + 45,
            10 * 60,
            10 * 60 + 30,
            11 * 60,
            12 * 60,
            12 * 60 + 45,
            13 * 60,
            13 * 60 + 45,
            14 * 60,
        ];

        // Check exact match first
        if valid_offsets.contains(&offset_minutes) {
            return true;
        }

        // Allow some tolerance (±5 minutes) for measurement errors
        valid_offsets
            .iter()
            .any(|&valid| (offset_minutes - valid).abs() <= 5)
    }

    /// Calculate confidence based on delta characteristics
    pub fn calculate_delta_confidence(
        delta_minutes: i32,
        local_confidence: f32,
        utc_confidence: f32,
    ) -> f32 {
        let mut confidence = (local_confidence + utc_confidence) / 2.0;

        // Boost confidence for exact timezone boundary matches
        if Self::is_valid_timezone_offset(delta_minutes) {
            confidence += 0.1;
        }

        // Reduce confidence for unusual offsets
        if delta_minutes % 15 != 0 && delta_minutes % 30 != 0 {
            confidence -= 0.2;
        }

        // Reduce confidence for extreme offsets (>12 hours)
        if delta_minutes.abs() > 12 * 60 {
            confidence -= 0.1;
        }

        confidence.clamp(0.0, 1.0)
    }
}

/// UTC reference timestamp for delta calculation
#[derive(Debug, Clone)]
pub struct UtcReference {
    pub datetime: ExifDateTime,
    pub tag_name: String,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_calculate_gps_delta() {
        // Create local time (DateTimeOriginal) - let's say 2:30 PM local
        let local_time = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        );

        // Create GPS time - 10:30 PM UTC (8 hours ahead, so local is UTC-8)
        let gps_time = ExifDateTime::new(
            Utc.with_ymd_and_hms(2024, 3, 15, 22, 30, 0).unwrap(),
            None,
            "2024:03:15 22:30:00".to_string(),
            InferenceSource::None,
            0.95,
        );

        let result = UtcDeltaCalculator::calculate_offset_from_gps_delta(&local_time, &gps_time);
        assert!(result.is_some());

        if let Some(InferenceSource::UtcDelta { delta_minutes, .. }) = result {
            assert_eq!(delta_minutes, -8 * 60); // -8 hours = -480 minutes
        }
    }

    #[test]
    fn test_validate_timezone_delta() {
        // Valid timezone offsets
        assert!(UtcDeltaCalculator::validate_timezone_delta(-8 * 60)); // PST
        assert!(UtcDeltaCalculator::validate_timezone_delta(5 * 60 + 30)); // IST
        assert!(UtcDeltaCalculator::validate_timezone_delta(0)); // UTC

        // Invalid offsets (too large)
        assert!(!UtcDeltaCalculator::validate_timezone_delta(15 * 60)); // >14 hours
        assert!(!UtcDeltaCalculator::validate_timezone_delta(-15 * 60)); // <-14 hours
    }

    #[test]
    fn test_round_to_timezone_boundary() {
        // Should round to nearest 15-minute boundary
        assert_eq!(UtcDeltaCalculator::round_to_timezone_boundary(-482), -480); // -8:02 → -8:00
        assert_eq!(UtcDeltaCalculator::round_to_timezone_boundary(-473), -480); // -7:53 → -8:00
        assert_eq!(UtcDeltaCalculator::round_to_timezone_boundary(337), 330); // 5:37 → 5:30
    }

    #[test]
    fn test_is_valid_timezone_offset() {
        // Common timezone offsets should be valid
        assert!(UtcDeltaCalculator::is_valid_timezone_offset(-8 * 60)); // PST
        assert!(UtcDeltaCalculator::is_valid_timezone_offset(5 * 60 + 30)); // IST
        assert!(UtcDeltaCalculator::is_valid_timezone_offset(9 * 60 + 30)); // ACST

        // Unusual offsets should be invalid
        assert!(!UtcDeltaCalculator::is_valid_timezone_offset(-7 * 60 - 23)); // Weird offset
        assert!(!UtcDeltaCalculator::is_valid_timezone_offset(4 * 60 + 17)); // Non-standard
    }

    #[test]
    fn test_find_utc_references() {
        let collection = DateTimeCollection {
            gps_datetime: Some(ExifDateTime::new(
                Utc.with_ymd_and_hms(2024, 3, 15, 22, 30, 0).unwrap(),
                None,
                "2024:03:15 22:30:00".to_string(),
                InferenceSource::None,
                0.95,
            )),
            ..Default::default()
        };

        let references = UtcDeltaCalculator::find_utc_references(&collection);
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].tag_name, "GPSDateTime");
        assert_eq!(references[0].confidence, 0.95);
    }

    #[test]
    fn test_calculate_delta_confidence() {
        // High confidence for exact timezone match
        let confidence = UtcDeltaCalculator::calculate_delta_confidence(
            -8 * 60, // Exact PST offset
            0.8,     // Local confidence
            0.95,    // UTC confidence
        );
        assert!(confidence > 0.8);

        // Lower confidence for unusual offset
        let confidence = UtcDeltaCalculator::calculate_delta_confidence(
            -7 * 60 - 23, // Weird offset
            0.8,          // Local confidence
            0.95,         // UTC confidence
        );
        assert!(confidence < 0.7);
    }
}
