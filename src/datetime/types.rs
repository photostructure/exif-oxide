//! Core datetime data structures and types
//!
//! This module defines the fundamental data structures used throughout the
//! datetime intelligence system, closely modeled after exiftool-vendored's
//! ExifDateTime and timezone handling patterns.

use chrono::{DateTime, FixedOffset, Utc};
use std::fmt;

/// High-level datetime with timezone intelligence
///
/// This structure represents a datetime extracted from EXIF metadata with
/// associated timezone information inferred through various heuristics.
/// It combines chrono's proven datetime handling with EXIF-specific metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ExifDateTime {
    /// The datetime in UTC, parsed from EXIF string
    pub datetime: DateTime<Utc>,

    /// Local timezone offset if known/inferred (stored as seconds from UTC)
    pub local_offset: Option<FixedOffset>,

    /// Original raw string from EXIF data (e.g., "2024:03:15 14:30:00")
    pub raw_value: String,

    /// How the timezone information was determined
    pub inference_source: InferenceSource,

    /// Confidence level in the timezone inference (0.0 = no confidence, 1.0 = certain)
    pub confidence: f32,

    /// Subsecond precision if available (0.0-999.999 milliseconds)
    pub subsecond: Option<f32>,
}

impl ExifDateTime {
    /// Create a new ExifDateTime from components
    pub fn new(
        datetime: DateTime<Utc>,
        local_offset: Option<FixedOffset>,
        raw_value: String,
        inference_source: InferenceSource,
        confidence: f32,
    ) -> Self {
        Self {
            datetime,
            local_offset,
            raw_value,
            inference_source,
            confidence: confidence.clamp(0.0, 1.0),
            subsecond: None,
        }
    }

    /// Get the datetime in the local timezone if available, otherwise UTC
    pub fn to_local_datetime(&self) -> DateTime<FixedOffset> {
        match self.local_offset {
            Some(offset) => self.datetime.with_timezone(&offset),
            None => self
                .datetime
                .with_timezone(&FixedOffset::east_opt(0).unwrap()),
        }
    }

    /// Get timezone offset in minutes (positive = east of UTC, negative = west)
    pub fn timezone_offset_minutes(&self) -> Option<i32> {
        self.local_offset
            .map(|offset| offset.local_minus_utc() / 60)
    }

    /// Check if timezone information is available
    pub fn has_timezone(&self) -> bool {
        self.local_offset.is_some()
    }

    /// Format as ISO 8601 string with timezone if available
    pub fn to_iso_string(&self) -> String {
        match self.local_offset {
            Some(_) => self.to_local_datetime().to_rfc3339(),
            None => format!("{}Z", self.datetime.format("%Y-%m-%dT%H:%M:%S%.3f")),
        }
    }

    /// Format as EXIF datetime string (YYYY:MM:DD HH:MM:SS)
    pub fn to_exif_string(&self) -> String {
        let local = self.to_local_datetime();
        let base = local.format("%Y:%m:%d %H:%M:%S").to_string();

        // Add subsecond precision if available
        if let Some(subsec) = self.subsecond {
            format!("{}.{:03}", base, (subsec * 1000.0) as u32)
        } else {
            base
        }
    }
}

impl fmt::Display for ExifDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso_string())
    }
}

/// How timezone information was determined
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceSource {
    /// Found in explicit EXIF timezone tags
    ExplicitTag { tag_name: String },

    /// Inferred from GPS coordinates
    GpsCoordinates {
        lat: f64,
        lng: f64,
        timezone: String,
    },

    /// Calculated from UTC timestamp delta
    UtcDelta {
        reference_tag: String,
        delta_minutes: i32,
    },

    /// Applied manufacturer-specific quirk
    ManufacturerQuirk {
        make: String,
        model: Option<String>,
        quirk_description: String,
    },

    /// No timezone information available
    None,
}

impl InferenceSource {
    /// Get a human-readable description of the inference method
    pub fn description(&self) -> String {
        match self {
            InferenceSource::ExplicitTag { tag_name } => {
                format!("Explicit timezone from EXIF tag {}", tag_name)
            }
            InferenceSource::GpsCoordinates { lat, lng, timezone } => {
                format!(
                    "Inferred {} from GPS coordinates ({:.4}, {:.4})",
                    timezone, lat, lng
                )
            }
            InferenceSource::UtcDelta {
                reference_tag,
                delta_minutes,
            } => {
                format!(
                    "Calculated UTC{:+} offset from {} timestamp",
                    *delta_minutes as f32 / 60.0,
                    reference_tag
                )
            }
            InferenceSource::ManufacturerQuirk {
                make,
                model,
                quirk_description,
            } => match model {
                Some(m) => format!("Applied {} {} quirk: {}", make, m, quirk_description),
                None => format!("Applied {} quirk: {}", make, quirk_description),
            },
            InferenceSource::None => "No timezone information available".to_string(),
        }
    }
}

/// Collection of datetime fields extracted from EXIF/XMP data
#[derive(Debug, Clone, Default)]
pub struct DateTimeCollection {
    /// When the photo was actually taken (most reliable)
    pub datetime_original: Option<ExifDateTime>,

    /// When film was digitized (for scanned photos)
    pub datetime_digitized: Option<ExifDateTime>,

    /// Generic creation date
    pub create_date: Option<ExifDateTime>,

    /// Last modification date
    pub modify_date: Option<ExifDateTime>,

    /// GPS timestamp (always UTC)
    pub gps_datetime: Option<ExifDateTime>,

    /// Raw subsecond values for precision
    pub subsec_time_original: Option<String>,
    pub subsec_time_digitized: Option<String>,

    /// Timezone offset tags
    pub offset_time_original: Option<String>,
    pub offset_time_digitized: Option<String>,
    pub timezone_offset: Option<i16>, // in hours

    /// GPS coordinates for timezone inference
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,

    /// File system timestamps (lowest priority)
    pub file_modify_date: Option<DateTime<Utc>>,
}

impl DateTimeCollection {
    /// Check if any datetime information is available
    pub fn is_empty(&self) -> bool {
        self.datetime_original.is_none()
            && self.datetime_digitized.is_none()
            && self.create_date.is_none()
            && self.modify_date.is_none()
            && self.gps_datetime.is_none()
    }

    /// Get the highest priority datetime available
    pub fn primary_datetime(&self) -> Option<&ExifDateTime> {
        self.datetime_original
            .as_ref()
            .or(self.datetime_digitized.as_ref())
            .or(self.create_date.as_ref())
            .or(self.modify_date.as_ref())
    }

    /// Check if GPS coordinates are available and valid
    pub fn has_valid_gps(&self) -> bool {
        match (self.gps_latitude, self.gps_longitude) {
            (Some(lat), Some(lng)) => {
                // GPS coordinates (0,0) are considered invalid in exiftool-vendored
                !(lat.abs() < 0.0001 && lng.abs() < 0.0001)
            }
            _ => false,
        }
    }
}

/// Final resolved datetime result with all inference metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedDateTime {
    /// The best datetime with timezone information
    pub datetime: ExifDateTime,

    /// Overall confidence in the result (0.0-1.0)
    pub confidence: f32,

    /// Any warnings discovered during processing
    pub warnings: Vec<DateTimeWarning>,

    /// Alternative datetime interpretations
    pub alternatives: Vec<ExifDateTime>,

    /// Detailed audit trail of inference steps
    pub inference_chain: Vec<String>,
}

impl ResolvedDateTime {
    /// Create a new resolved datetime result
    pub fn new(datetime: ExifDateTime) -> Self {
        let confidence = datetime.confidence;
        Self {
            datetime,
            confidence,
            warnings: Vec::new(),
            alternatives: Vec::new(),
            inference_chain: Vec::new(),
        }
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: DateTimeWarning) {
        self.warnings.push(warning);
    }

    /// Add an alternative datetime interpretation
    pub fn add_alternative(&mut self, alternative: ExifDateTime) {
        self.alternatives.push(alternative);
    }

    /// Add a step to the inference audit trail
    pub fn add_inference_step(&mut self, step: String) {
        self.inference_chain.push(step);
    }
}

/// Warnings that can be generated during datetime processing
#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeWarning {
    /// Datetime is in the future
    FutureDate { datetime: String },

    /// Datetime is suspiciously old
    VeryOldDate { datetime: String, year: i32 },

    /// Large discrepancy between datetime sources
    InconsistentDatetimes {
        primary: String,
        secondary: String,
        delta_hours: f32,
    },

    /// Timezone inference resulted in suspicious offset
    SuspiciousTimezone { offset_minutes: i32, reason: String },

    /// Subsecond precision was truncated
    SubsecondTruncated { original: String, truncated: f32 },

    /// Manufacturer quirk was applied
    QuirkApplied { make: String, quirk: String },
}

impl DateTimeWarning {
    /// Get a human-readable description of the warning
    pub fn description(&self) -> String {
        match self {
            DateTimeWarning::FutureDate { datetime } => {
                format!("DateTime '{}' is in the future", datetime)
            }
            DateTimeWarning::VeryOldDate { datetime, year } => {
                format!("DateTime '{}' is very old ({})", datetime, year)
            }
            DateTimeWarning::InconsistentDatetimes {
                primary,
                secondary,
                delta_hours,
            } => {
                format!(
                    "Large discrepancy: '{}' vs '{}' ({:.1}h difference)",
                    primary, secondary, delta_hours
                )
            }
            DateTimeWarning::SuspiciousTimezone {
                offset_minutes,
                reason,
            } => {
                format!(
                    "Suspicious timezone offset UTC{:+} ({})",
                    *offset_minutes as f32 / 60.0,
                    reason
                )
            }
            DateTimeWarning::SubsecondTruncated {
                original,
                truncated,
            } => {
                format!(
                    "Subsecond precision truncated: '{}' → {:.3}ms",
                    original, truncated
                )
            }
            DateTimeWarning::QuirkApplied { make, quirk } => {
                format!("Applied {} quirk: {}", make, quirk)
            }
        }
    }
}

/// Camera information for manufacturer-specific quirks
#[derive(Debug, Clone, Default)]
pub struct CameraInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
}

impl CameraInfo {
    /// Extract camera info from EXIF data
    pub fn from_exif(exif_data: &std::collections::HashMap<u16, String>) -> Self {
        Self {
            make: exif_data.get(&0x010F).cloned(),     // Make
            model: exif_data.get(&0x0110).cloned(),    // Model
            software: exif_data.get(&0x0131).cloned(), // Software
        }
    }

    /// Check if this is a known problematic camera model
    pub fn is_known_problematic(&self) -> bool {
        match (self.make.as_deref(), self.model.as_deref()) {
            (Some("NIKON CORPORATION"), Some(model)) => {
                // Known Nikon models with DST bugs
                model.contains("D3") || model.contains("D300") || model.contains("D700")
            }
            _ => false,
        }
    }
}

/// Confidence scoring for datetime inference quality
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Calculate confidence score based on inference source and validation
    pub fn calculate_confidence(
        source: &InferenceSource,
        has_validation: bool,
        warnings: &[DateTimeWarning],
    ) -> f32 {
        let base_confidence = match source {
            InferenceSource::ExplicitTag { .. } => 0.95,
            InferenceSource::GpsCoordinates { .. } => 0.80,
            InferenceSource::UtcDelta { .. } => 0.70,
            InferenceSource::ManufacturerQuirk { .. } => 0.60,
            InferenceSource::None => 0.10,
        };

        // Boost confidence if we have cross-validation
        let validation_boost = if has_validation { 0.05 } else { 0.0 };

        // Reduce confidence for each warning
        let warning_penalty = warnings.len() as f32 * 0.05;

        (base_confidence + validation_boost - warning_penalty).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_exif_datetime_creation() {
        let utc_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();
        let offset = FixedOffset::west_opt(8 * 3600).unwrap(); // UTC-8

        let dt = ExifDateTime::new(
            utc_time,
            Some(offset),
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::ExplicitTag {
                tag_name: "OffsetTimeOriginal".to_string(),
            },
            0.95,
        );

        assert_eq!(dt.timezone_offset_minutes(), Some(-480)); // -8 hours in minutes
        assert!(dt.has_timezone());
        assert_eq!(dt.confidence, 0.95);
    }

    #[test]
    fn test_datetime_collection_priority() {
        let mut collection = DateTimeCollection::default();

        let utc_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();
        let modify_time = Utc.with_ymd_and_hms(2024, 3, 16, 10, 0, 0).unwrap();

        collection.modify_date = Some(ExifDateTime::new(
            modify_time,
            None,
            "2024:03:16 10:00:00".to_string(),
            InferenceSource::None,
            0.3,
        ));

        collection.datetime_original = Some(ExifDateTime::new(
            utc_time,
            None,
            "2024:03:15 14:30:00".to_string(),
            InferenceSource::None,
            0.8,
        ));

        // Should prioritize DateTimeOriginal over ModifyDate
        let primary = collection.primary_datetime().unwrap();
        assert_eq!(primary.raw_value, "2024:03:15 14:30:00");
    }

    #[test]
    fn test_gps_coordinate_validation() {
        // Invalid GPS coordinates (0,0)
        let mut collection = DateTimeCollection {
            gps_latitude: Some(0.0),
            gps_longitude: Some(0.0),
            ..Default::default()
        };
        assert!(!collection.has_valid_gps());

        // Valid GPS coordinates
        collection.gps_latitude = Some(37.7749);
        collection.gps_longitude = Some(-122.4194);
        assert!(collection.has_valid_gps());
    }

    #[test]
    fn test_confidence_scoring() {
        let explicit_source = InferenceSource::ExplicitTag {
            tag_name: "OffsetTimeOriginal".to_string(),
        };
        let gps_source = InferenceSource::GpsCoordinates {
            lat: 37.7749,
            lng: -122.4194,
            timezone: "America/Los_Angeles".to_string(),
        };

        // Explicit tags should have higher confidence
        let explicit_confidence =
            ConfidenceScorer::calculate_confidence(&explicit_source, false, &[]);
        let gps_confidence = ConfidenceScorer::calculate_confidence(&gps_source, false, &[]);

        assert!(explicit_confidence > gps_confidence);
        assert!(explicit_confidence >= 0.9);
        assert!(gps_confidence >= 0.7);
    }
}
