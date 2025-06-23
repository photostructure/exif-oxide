//! Main datetime intelligence coordination engine
//!
//! This module coordinates all datetime intelligence heuristics to produce
//! the best possible datetime with timezone information.

use crate::datetime::gps_timezone::GpsTimezoneInference;
use crate::datetime::parser::DateTimeParser;
use crate::datetime::quirks::ManufacturerQuirks;
use crate::datetime::types::*;
use crate::datetime::utc_delta::UtcDeltaCalculator;
use crate::error::Result;
use chrono::FixedOffset;

/// Main datetime intelligence engine
///
/// Coordinates all heuristics to resolve the best datetime with timezone information.
/// Implements the priority order from exiftool-vendored:
/// 1. Explicit timezone tags (highest priority)  
/// 2. GPS coordinate inference
/// 3. UTC timestamp delta calculation
/// 4. Manufacturer-specific quirks (lowest priority)
pub struct DateTimeIntelligence {
    #[allow(dead_code)]
    gps_inference: GpsTimezoneInference,
    #[allow(dead_code)]
    utc_calculator: UtcDeltaCalculator,
    #[allow(dead_code)]
    quirk_handler: ManufacturerQuirks,
}

impl DateTimeIntelligence {
    /// Create a new datetime intelligence engine
    pub fn new() -> Self {
        Self {
            gps_inference: GpsTimezoneInference,
            utc_calculator: UtcDeltaCalculator,
            quirk_handler: ManufacturerQuirks,
        }
    }

    /// Resolve the best capture datetime with timezone intelligence
    pub fn resolve_capture_datetime(
        &self,
        collection: &DateTimeCollection,
        camera_info: &CameraInfo,
    ) -> Result<ResolvedDateTime> {
        // Start with the highest priority datetime available
        let mut primary_datetime = collection
            .primary_datetime()
            .ok_or_else(|| {
                crate::error::Error::InvalidDateTime("No datetime available".to_string())
            })?
            .clone();

        let mut result = ResolvedDateTime::new(primary_datetime.clone());
        result.add_inference_step("Started with primary datetime".to_string());

        // Apply timezone inference in priority order
        let timezone_inference = self.infer_timezone(collection, camera_info, &mut result);

        if let Some(inference) = timezone_inference {
            primary_datetime = self.apply_timezone_inference(&primary_datetime, &inference)?;
            result.datetime = primary_datetime.clone();
            result.add_inference_step(format!(
                "Applied timezone inference: {}",
                inference.description()
            ));
        }

        // Apply manufacturer quirks
        let quirks = ManufacturerQuirks::apply_quirks(&mut result.datetime, camera_info);
        for quirk in &quirks {
            if quirk.correction_applied {
                result.add_warning(DateTimeWarning::QuirkApplied {
                    make: quirk.make.clone(),
                    quirk: quirk.description.clone(),
                });
            }
            result.add_inference_step(format!(
                "Applied {} quirk: {}",
                quirk.make, quirk.description
            ));
        }

        // Validate and generate warnings
        let validation_warnings = DateTimeParser::validate_datetime_ranges(&result.datetime);
        for warning in validation_warnings {
            result.add_warning(warning);
        }

        // Cross-validate with other datetime sources
        self.cross_validate_datetimes(&mut result, collection);

        // Calculate final confidence score
        result.confidence = ConfidenceScorer::calculate_confidence(
            &result.datetime.inference_source,
            !result.alternatives.is_empty(),
            &result.warnings,
        );

        Ok(result)
    }

    /// Apply all timezone inference heuristics in priority order
    fn infer_timezone(
        &self,
        collection: &DateTimeCollection,
        camera_info: &CameraInfo,
        result: &mut ResolvedDateTime,
    ) -> Option<InferenceSource> {
        // 1. Check explicit timezone tags (highest priority)
        if let Some(explicit) = self.check_explicit_timezone_tags(collection) {
            result.add_inference_step("Found explicit timezone tags".to_string());
            return Some(explicit);
        }

        // 2. GPS coordinate inference (high priority)
        if let Some(gps_tz) = self.infer_from_gps_coordinates(collection) {
            result.add_inference_step("Inferred timezone from GPS coordinates".to_string());
            return Some(gps_tz);
        }

        // 3. UTC delta calculation (medium priority)
        if let Some(delta_tz) = self.calculate_utc_delta(collection) {
            result.add_inference_step("Calculated timezone from UTC delta".to_string());
            return Some(delta_tz);
        }

        // 4. Manufacturer quirk handling (low priority)
        if let Some(quirk_tz) = self.apply_manufacturer_timezone_quirks(collection, camera_info) {
            result.add_inference_step("Applied manufacturer timezone quirk".to_string());
            return Some(quirk_tz);
        }

        result.add_inference_step("No timezone information available".to_string());
        None
    }

    /// Check for explicit timezone tags in EXIF data
    fn check_explicit_timezone_tags(
        &self,
        collection: &DateTimeCollection,
    ) -> Option<InferenceSource> {
        // Check OffsetTimeOriginal tag
        if let Some(offset_str) = &collection.offset_time_original {
            if crate::datetime::parser::DateTimeParser::parse_timezone_offset(offset_str).is_ok() {
                return Some(InferenceSource::ExplicitTag {
                    tag_name: "OffsetTimeOriginal".to_string(),
                });
            }
        }

        // Check OffsetTimeDigitized tag
        if let Some(offset_str) = &collection.offset_time_digitized {
            if crate::datetime::parser::DateTimeParser::parse_timezone_offset(offset_str).is_ok() {
                return Some(InferenceSource::ExplicitTag {
                    tag_name: "OffsetTimeDigitized".to_string(),
                });
            }
        }

        // Check TimeZoneOffset tag (in hours)
        if let Some(tz_offset) = collection.timezone_offset {
            if (-14..=14).contains(&tz_offset) {
                return Some(InferenceSource::ExplicitTag {
                    tag_name: "TimeZoneOffset".to_string(),
                });
            }
        }

        None
    }

    /// Infer timezone from GPS coordinates
    fn infer_from_gps_coordinates(
        &self,
        collection: &DateTimeCollection,
    ) -> Option<InferenceSource> {
        if !collection.has_valid_gps() {
            return None;
        }

        let lat = collection.gps_latitude?;
        let lng = collection.gps_longitude?;

        GpsTimezoneInference::infer_timezone(lat, lng)
    }

    /// Calculate timezone from UTC delta
    fn calculate_utc_delta(&self, collection: &DateTimeCollection) -> Option<InferenceSource> {
        let primary = collection.primary_datetime()?;

        // Look for UTC references
        let utc_refs = UtcDeltaCalculator::find_utc_references(collection);
        if utc_refs.is_empty() {
            return None;
        }

        // Use the highest confidence UTC reference
        let best_utc_ref = utc_refs
            .into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())?;

        UtcDeltaCalculator::calculate_offset_from_gps_delta(primary, &best_utc_ref.datetime)
    }

    /// Apply manufacturer-specific timezone quirks
    fn apply_manufacturer_timezone_quirks(
        &self,
        _collection: &DateTimeCollection,
        camera_info: &CameraInfo,
    ) -> Option<InferenceSource> {
        // This is a placeholder for manufacturer-specific timezone inference
        // that doesn't involve corrections but rather timezone detection patterns

        match camera_info.make.as_deref() {
            Some("Apple") => {
                // iOS devices often have accurate timezone information
                Some(InferenceSource::ManufacturerQuirk {
                    make: "Apple".to_string(),
                    model: camera_info.model.clone(),
                    quirk_description: "iOS typically has accurate timezone".to_string(),
                })
            }
            _ => None,
        }
    }

    /// Apply timezone inference to a datetime
    fn apply_timezone_inference(
        &self,
        datetime: &ExifDateTime,
        inference: &InferenceSource,
    ) -> Result<ExifDateTime> {
        let mut result = datetime.clone();

        match inference {
            InferenceSource::ExplicitTag { tag_name: _ } => {
                // Timezone was already parsed in the original datetime
                result.inference_source = inference.clone();
                result.confidence = 0.95;
            }

            InferenceSource::GpsCoordinates {
                lat,
                lng,
                timezone: _,
            } => {
                // Calculate timezone offset from GPS coordinates
                if let Some(offset_minutes) =
                    GpsTimezoneInference::get_timezone_offset(*lat, *lng, datetime.datetime)
                {
                    if let Some(fixed_offset) = FixedOffset::east_opt(offset_minutes * 60) {
                        result.local_offset = Some(fixed_offset);
                        result.inference_source = inference.clone();
                        result.confidence = 0.80;
                    }
                }
            }

            InferenceSource::UtcDelta { delta_minutes, .. } => {
                if let Some(fixed_offset) = FixedOffset::east_opt(delta_minutes * 60) {
                    result.local_offset = Some(fixed_offset);
                    result.inference_source = inference.clone();
                    result.confidence = 0.70;
                }
            }

            InferenceSource::ManufacturerQuirk { .. } => {
                // Apply manufacturer-specific timezone handling
                result.inference_source = inference.clone();
                result.confidence = 0.60;
            }

            InferenceSource::None => {
                // No change needed
            }
        }

        Ok(result)
    }

    /// Cross-validate datetime consistency across sources
    fn cross_validate_datetimes(
        &self,
        result: &mut ResolvedDateTime,
        collection: &DateTimeCollection,
    ) {
        let primary_timestamp = result.datetime.datetime.timestamp();
        let primary_raw = result.datetime.raw_value.clone();

        // Check consistency with other datetime sources
        let candidates = [
            ("DateTimeDigitized", &collection.datetime_digitized),
            ("CreateDate", &collection.create_date),
            ("ModifyDate", &collection.modify_date),
        ];

        for (_tag_name, candidate_opt) in &candidates {
            if let Some(candidate) = candidate_opt {
                let delta_seconds = (primary_timestamp - candidate.datetime.timestamp()).abs();
                let delta_hours = delta_seconds as f32 / 3600.0;

                // Warn about large discrepancies (>24 hours)
                if delta_hours > 24.0 {
                    result.add_warning(DateTimeWarning::InconsistentDatetimes {
                        primary: primary_raw.clone(),
                        secondary: candidate.raw_value.clone(),
                        delta_hours,
                    });
                } else if delta_hours < 1.0 {
                    // Close match - add as alternative
                    result.add_alternative(candidate.clone());
                }
            }
        }
    }
}

impl Default for DateTimeIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_explicit_timezone_priority() {
        let mut collection = DateTimeCollection::default();

        // Add datetime with explicit timezone offset
        collection.offset_time_original = Some("+05:00".to_string());
        collection.datetime_original = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // Also add GPS coordinates (lower priority)
        collection.gps_latitude = Some(40.7128);
        collection.gps_longitude = Some(-74.0060);

        let camera_info = CameraInfo::default();
        let intelligence = DateTimeIntelligence::new();
        let mut result =
            ResolvedDateTime::new(collection.datetime_original.as_ref().unwrap().clone());

        let inference = intelligence.infer_timezone(&collection, &camera_info, &mut result);

        // Should prioritize explicit tag over GPS
        assert!(inference.is_some());
        if let Some(InferenceSource::ExplicitTag { tag_name }) = inference {
            assert_eq!(tag_name, "OffsetTimeOriginal");
        } else {
            panic!("Expected explicit tag inference");
        }
    }

    #[test]
    fn test_gps_timezone_inference() {
        let mut collection = DateTimeCollection::default();

        collection.datetime_original = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // Add valid GPS coordinates (New York)
        collection.gps_latitude = Some(40.7128);
        collection.gps_longitude = Some(-74.0060);

        let camera_info = CameraInfo::default();
        let intelligence = DateTimeIntelligence::new();
        let mut result =
            ResolvedDateTime::new(collection.datetime_original.as_ref().unwrap().clone());

        let inference = intelligence.infer_timezone(&collection, &camera_info, &mut result);

        assert!(inference.is_some());
        if let Some(InferenceSource::GpsCoordinates { lat, lng, timezone }) = inference {
            assert_eq!(lat, 40.7128);
            assert_eq!(lng, -74.0060);
            assert_eq!(timezone, "America/New_York");
        } else {
            panic!("Expected GPS coordinate inference");
        }
    }

    #[test]
    fn test_utc_delta_calculation() {
        let mut collection = DateTimeCollection::default();

        // Local datetime (DateTimeOriginal)
        collection.datetime_original = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // GPS datetime (UTC) - 8 hours ahead of local time
        collection.gps_datetime = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 22, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 22:30:00".to_string(),
            InferenceSource::None,
            0.95,
        ));

        let camera_info = CameraInfo::default();
        let intelligence = DateTimeIntelligence::new();
        let mut result =
            ResolvedDateTime::new(collection.datetime_original.as_ref().unwrap().clone());

        let inference = intelligence.infer_timezone(&collection, &camera_info, &mut result);

        assert!(inference.is_some());
        if let Some(InferenceSource::UtcDelta { delta_minutes, .. }) = inference {
            assert_eq!(delta_minutes, -8 * 60); // -8 hours = -480 minutes
        } else {
            panic!("Expected UTC delta inference");
        }
    }

    #[test]
    fn test_no_timezone_inference() {
        let mut collection = DateTimeCollection::default();

        collection.datetime_original = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // No timezone information available

        let camera_info = CameraInfo::default();
        let intelligence = DateTimeIntelligence::new();
        let mut result =
            ResolvedDateTime::new(collection.datetime_original.as_ref().unwrap().clone());

        let inference = intelligence.infer_timezone(&collection, &camera_info, &mut result);

        assert!(inference.is_none());
    }

    #[test]
    fn test_cross_validation_warnings() {
        let mut collection = DateTimeCollection::default();

        // Primary datetime
        collection.datetime_original = Some(ExifDateTime::new(
            chrono::Utc
                .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
                .unwrap(),
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // Modify date that's very different (should generate warning)
        collection.modify_date = Some(ExifDateTime::new(
            chrono::Utc.with_ymd_and_hms(2024, 3, 17, 10, 0, 0).unwrap(),
            None,
            "2024:03:17 10:00:00".to_string(),
            InferenceSource::None,
            0.3,
        ));

        let camera_info = CameraInfo::default();
        let intelligence = DateTimeIntelligence::new();

        let result = intelligence
            .resolve_capture_datetime(&collection, &camera_info)
            .unwrap();

        // Should generate inconsistency warning
        assert!(!result.warnings.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|w| matches!(w, DateTimeWarning::InconsistentDatetimes { .. })));
    }
}
