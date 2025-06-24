//! Multi-source datetime extraction from EXIF and XMP data
//!
//! This module handles extraction of datetime fields from various sources
//! and prioritizes them by reliability.

use crate::datetime::parser::DateTimeParser;
use crate::datetime::types::*;
use crate::error::Result;
use std::collections::HashMap;

/// Extractor for datetime fields from EXIF and XMP metadata
pub struct DateTimeExtractor;

impl DateTimeExtractor {
    /// Extract all datetime fields from EXIF and XMP data
    pub fn extract_all_datetimes(
        exif_data: &HashMap<u16, String>,
        xmp_data: Option<&crate::xmp::types::XmpMetadata>,
    ) -> Result<DateTimeCollection> {
        let mut collection = DateTimeCollection::default();

        // Extract primary datetime fields from EXIF
        if let Some(dt_orig) = exif_data.get(&0x9003) {
            // DateTimeOriginal
            if let Ok(parsed) = DateTimeParser::parse_exif_datetime(dt_orig) {
                collection.datetime_original = Some(parsed);
            }
        }

        if let Some(dt_digit) = exif_data.get(&0x9004) {
            // DateTimeDigitized
            if let Ok(parsed) = DateTimeParser::parse_exif_datetime(dt_digit) {
                collection.datetime_digitized = Some(parsed);
            }
        }

        if let Some(modify_date) = exif_data.get(&0x0132) {
            // ModifyDate
            if let Ok(parsed) = DateTimeParser::parse_exif_datetime(modify_date) {
                collection.modify_date = Some(parsed);
            }
        }

        // Extract timezone-related fields
        collection.offset_time_original = exif_data.get(&0x9010).cloned(); // OffsetTimeOriginal
        collection.offset_time_digitized = exif_data.get(&0x9011).cloned(); // OffsetTimeDigitized

        // Extract GPS coordinates for timezone inference
        if let (Some(lat_str), Some(lng_str)) = (exif_data.get(&0x0002), exif_data.get(&0x0004)) {
            // TODO: Parse GPS coordinates properly from degrees/minutes/seconds
            // For now, simplified parsing assuming decimal degrees
            if let (Ok(lat), Ok(lng)) = (lat_str.parse::<f64>(), lng_str.parse::<f64>()) {
                collection.gps_latitude = Some(lat);
                collection.gps_longitude = Some(lng);
            }
        }

        // Extract subsecond precision fields
        collection.subsec_time_original = exif_data.get(&0x9291).cloned(); // SubSecTimeOriginal
        collection.subsec_time_digitized = exif_data.get(&0x9292).cloned(); // SubSecTimeDigitized

        // TODO: Extract from XMP data
        if let Some(_xmp) = xmp_data {
            // Extract XMP datetime fields when available
        }

        Ok(collection)
    }

    /// Prioritize datetime sources by reliability
    pub fn prioritize_datetime_sources(
        collection: &DateTimeCollection,
    ) -> Vec<PrioritizedDateTime> {
        let mut prioritized = Vec::new();

        // Priority order based on exiftool-vendored patterns:
        // 1. DateTimeOriginal (camera capture time) - highest priority
        if let Some(dt) = &collection.datetime_original {
            prioritized.push(PrioritizedDateTime {
                datetime: dt.clone(),
                priority: 1,
                source_description: "Camera capture time (DateTimeOriginal)".to_string(),
            });
        }

        // 2. DateTimeDigitized (scan time for film photos)
        if let Some(dt) = &collection.datetime_digitized {
            prioritized.push(PrioritizedDateTime {
                datetime: dt.clone(),
                priority: 2,
                source_description: "Digitization time (DateTimeDigitized)".to_string(),
            });
        }

        // 3. CreateDate (file creation)
        if let Some(dt) = &collection.create_date {
            prioritized.push(PrioritizedDateTime {
                datetime: dt.clone(),
                priority: 3,
                source_description: "File creation date (CreateDate)".to_string(),
            });
        }

        // 4. ModifyDate (last edit) - lower priority
        if let Some(dt) = &collection.modify_date {
            prioritized.push(PrioritizedDateTime {
                datetime: dt.clone(),
                priority: 4,
                source_description: "File modification date (ModifyDate)".to_string(),
            });
        }

        prioritized
    }
}

/// Datetime with priority information
#[derive(Debug, Clone)]
pub struct PrioritizedDateTime {
    pub datetime: ExifDateTime,
    pub priority: u8, // 1 = highest priority
    pub source_description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extract_basic_datetimes() {
        let mut exif_data = HashMap::new();
        exif_data.insert(0x9003, "2024:03:15 14:30:00".to_string()); // DateTimeOriginal
        exif_data.insert(0x0132, "2024:03:16 10:00:00".to_string()); // ModifyDate

        let collection = DateTimeExtractor::extract_all_datetimes(&exif_data, None).unwrap();

        assert!(collection.datetime_original.is_some());
        assert!(collection.modify_date.is_some());
        assert!(collection.datetime_digitized.is_none());
    }

    #[test]
    fn test_prioritize_datetime_sources() {
        let collection = DateTimeCollection {
            modify_date: Some(ExifDateTime::new(
                chrono::Utc::now(),
                None,
                "2024:03:16 10:00:00".to_string(),
                InferenceSource::None,
                0.3,
            )),
            datetime_original: Some(ExifDateTime::new(
                chrono::Utc::now(),
                None,
                "2024:03:15 14:30:00".to_string(),
                InferenceSource::None,
                0.8,
            )),
            ..Default::default()
        };

        let prioritized = DateTimeExtractor::prioritize_datetime_sources(&collection);

        assert_eq!(prioritized.len(), 2);
        assert_eq!(prioritized[0].priority, 1); // DateTimeOriginal should be first
        assert_eq!(prioritized[1].priority, 4); // ModifyDate should be second
    }
}
