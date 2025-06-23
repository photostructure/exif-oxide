//! EXIF datetime string parsing with timezone support
//!
//! This module handles parsing of various datetime formats found in EXIF metadata,
//! including subsecond precision and timezone offset parsing.

use crate::datetime::types::*;
use crate::error::{Error, Result};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use regex::Regex;

/// Parser for EXIF datetime strings and related fields
pub struct DateTimeParser;

impl DateTimeParser {
    /// Parse EXIF datetime string in format: "YYYY:MM:DD HH:MM:SS[.SSS][±HH:MM]"
    ///
    /// Handles various formats:
    /// - "2024:03:15 14:30:00"           (standard EXIF format)
    /// - "2024:03:15 14:30:00.123"      (with milliseconds)  
    /// - "2024:03:15 14:30:00+05:00"    (with timezone)
    /// - "2024:03:15 14:30:00.123-08:00" (full format)
    /// - "2024-03-15T14:30:00Z"         (ISO 8601 format)
    pub fn parse_exif_datetime(input: &str) -> Result<ExifDateTime> {
        let input = input.trim();
        if input.is_empty() {
            return Err(Error::InvalidDateTime("Empty datetime string".to_string()));
        }

        // Try different parsing strategies in order of preference
        if let Ok(dt) = Self::parse_exif_standard(input) {
            return Ok(dt);
        }

        if let Ok(dt) = Self::parse_iso_format(input) {
            return Ok(dt);
        }

        if let Ok(dt) = Self::parse_loose_format(input) {
            return Ok(dt);
        }

        Err(Error::InvalidDateTime(format!(
            "Could not parse datetime: '{}'",
            input
        )))
    }

    /// Parse standard EXIF format: "YYYY:MM:DD HH:MM:SS[.SSS][±HH:MM]"
    fn parse_exif_standard(input: &str) -> Result<ExifDateTime> {
        lazy_static::lazy_static! {
            static ref EXIF_REGEX: Regex = Regex::new(
                r"^(?P<year>\d{4}):(?P<month>\d{1,2}):(?P<day>\d{1,2})\s+(?P<hour>\d{1,2}):(?P<minute>\d{1,2}):(?P<second>\d{1,2})(?:\.(?P<subsec>\d{1,6}))?(?P<tz>[+-]\d{2}:?\d{2})?$"
            ).unwrap();
        }

        let caps = EXIF_REGEX.captures(input).ok_or_else(|| {
            Error::InvalidDateTime(format!("Invalid EXIF datetime format: '{}'", input))
        })?;

        // Parse date/time components
        let year = caps.name("year").unwrap().as_str().parse::<i32>()?;
        let month = caps.name("month").unwrap().as_str().parse::<u32>()?;
        let day = caps.name("day").unwrap().as_str().parse::<u32>()?;
        let hour = caps.name("hour").unwrap().as_str().parse::<u32>()?;
        let minute = caps.name("minute").unwrap().as_str().parse::<u32>()?;
        let second = caps.name("second").unwrap().as_str().parse::<u32>()?;

        // Validate ranges
        Self::validate_datetime_components(year, month, day, hour, minute, second)?;

        // Parse subsecond precision if present
        let subsecond = caps
            .name("subsec")
            .map(|m| Self::parse_subseconds(m.as_str()))
            .transpose()?;

        // Parse timezone offset if present
        let timezone_offset = caps
            .name("tz")
            .map(|m| Self::parse_timezone_offset(m.as_str()))
            .transpose()?;

        // Create naive datetime first
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| Error::InvalidDateTime("Invalid date components".to_string()))?;
        let naive_dt = naive_date
            .and_hms_opt(hour, minute, second)
            .ok_or_else(|| Error::InvalidDateTime("Invalid time components".to_string()))?;

        // Convert to UTC based on timezone information
        let (utc_datetime, local_offset, inference_source) = match timezone_offset {
            Some(offset) => {
                // Explicit timezone provided
                let fixed_offset = FixedOffset::east_opt(offset * 60)
                    .ok_or_else(|| Error::InvalidDateTime("Invalid timezone offset".to_string()))?;
                let local_dt = fixed_offset
                    .from_local_datetime(&naive_dt)
                    .single()
                    .ok_or_else(|| {
                        Error::InvalidDateTime("Ambiguous local datetime".to_string())
                    })?;
                (
                    local_dt.with_timezone(&Utc),
                    Some(fixed_offset),
                    InferenceSource::ExplicitTag {
                        tag_name: "embedded".to_string(),
                    },
                )
            }
            None => {
                // No timezone, assume UTC for now (will be inferred later)
                (
                    Utc.from_utc_datetime(&naive_dt),
                    None,
                    InferenceSource::None,
                )
            }
        };

        let mut dt = ExifDateTime::new(
            utc_datetime,
            local_offset,
            input.to_string(),
            inference_source,
            if timezone_offset.is_some() { 0.95 } else { 0.3 },
        );

        dt.subsecond = subsecond;
        Ok(dt)
    }

    /// Parse ISO 8601 format datetime
    fn parse_iso_format(input: &str) -> Result<ExifDateTime> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
            let utc_dt = dt.with_timezone(&Utc);
            let offset = *dt.offset();

            return Ok(ExifDateTime::new(
                utc_dt,
                Some(offset),
                input.to_string(),
                InferenceSource::ExplicitTag {
                    tag_name: "ISO8601".to_string(),
                },
                0.90,
            ));
        }

        if let Ok(dt) = DateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%SZ") {
            return Ok(ExifDateTime::new(
                dt.with_timezone(&Utc),
                None,
                input.to_string(),
                InferenceSource::ExplicitTag {
                    tag_name: "ISO8601_UTC".to_string(),
                },
                0.85,
            ));
        }

        Err(Error::InvalidDateTime("Not a valid ISO format".to_string()))
    }

    /// Parse loose/alternative datetime formats found in the wild
    fn parse_loose_format(input: &str) -> Result<ExifDateTime> {
        // Try various loose formats based on exiftool-vendored patterns
        let loose_formats = [
            "%b %d %Y %H:%M:%S",  // "Mar 15 2024 14:30:00"
            "%b %d %Y, %H:%M:%S", // "Mar 15 2024, 14:30:00"
            "%Y-%m-%d %H:%M:%S",  // "2024-03-15 14:30:00"
            "%m/%d/%Y %H:%M:%S",  // "03/15/2024 14:30:00"
        ];

        // First try standard formats
        for format in &loose_formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(input, format) {
                return Ok(ExifDateTime::new(
                    Utc.from_utc_datetime(&naive_dt),
                    None,
                    input.to_string(),
                    InferenceSource::None,
                    0.60, // Lower confidence for loose parsing
                ));
            }
        }

        // Handle weekday-prefixed formats by stripping the weekday
        // Example: "Thu Mar 15 14:30:00 2024" -> "Mar 15 14:30:00 2024"
        if let Some(stripped) = Self::strip_weekday_prefix(input) {
            // Try the format without weekday: "Mar 15 14:30:00 2024"
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&stripped, "%b %d %H:%M:%S %Y") {
                return Ok(ExifDateTime::new(
                    Utc.from_utc_datetime(&naive_dt),
                    None,
                    input.to_string(),
                    InferenceSource::None,
                    0.55, // Slightly lower confidence for weekday-stripped parsing
                ));
            }
        }

        Err(Error::InvalidDateTime(
            "No matching format found".to_string(),
        ))
    }

    /// Strip weekday prefix from datetime strings
    ///
    /// Converts "Thu Mar 15 14:30:00 2024" to "Mar 15 14:30:00 2024"
    fn strip_weekday_prefix(input: &str) -> Option<String> {
        // Common weekday abbreviations
        let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        let trimmed = input.trim();
        for weekday in &weekdays {
            if trimmed.starts_with(weekday) && trimmed.len() > weekday.len() {
                let remainder = &trimmed[weekday.len()..];
                if let Some(stripped) = remainder.strip_prefix(' ') {
                    return Some(stripped.to_string());
                }
            }
        }

        None
    }

    /// Parse subsecond string into milliseconds
    ///
    /// Handles various formats:
    /// - "123" → 123.0 ms (3 digits = milliseconds)
    /// - "123456" → 123.456 ms (6 digits = microseconds)
    /// - "1" → 100.0 ms (1 digit = tenths of second)
    pub fn parse_subseconds(subsec_str: &str) -> Result<f32> {
        if subsec_str.is_empty() {
            return Err(Error::InvalidDateTime("Empty subsecond string".to_string()));
        }

        let digits: String = subsec_str.chars().take(6).collect(); // Max 6 digits (microseconds)
        let num = digits
            .parse::<u32>()
            .map_err(|_| Error::InvalidDateTime("Invalid subsecond digits".to_string()))?;

        // Convert to milliseconds based on number of digits
        let ms = match digits.len() {
            1 => num as f32 * 100.0,  // tenths → ms
            2 => num as f32 * 10.0,   // hundredths → ms
            3 => num as f32,          // milliseconds
            4 => num as f32 / 10.0,   // ten-thousandths → ms
            5 => num as f32 / 100.0,  // hundred-thousandths → ms
            6 => num as f32 / 1000.0, // microseconds → ms
            _ => {
                return Err(Error::InvalidDateTime(
                    "Too many subsecond digits".to_string(),
                ))
            }
        };

        Ok(ms.clamp(0.0, 999.999))
    }

    /// Parse timezone offset string into minutes
    ///
    /// Handles formats: "+05:00", "-08:30", "+0530", "-8", etc.
    pub fn parse_timezone_offset(offset_str: &str) -> Result<i32> {
        lazy_static::lazy_static! {
            static ref TZ_REGEX: Regex = Regex::new(
                r"^(?P<sign>[+-])(?P<hours>\d{1,2})(?::?(?P<minutes>\d{2}))?$"
            ).unwrap();
        }

        let caps = TZ_REGEX.captures(offset_str).ok_or_else(|| {
            Error::InvalidDateTime(format!("Invalid timezone format: '{}'", offset_str))
        })?;

        let sign = if caps.name("sign").unwrap().as_str() == "+" {
            1
        } else {
            -1
        };
        let hours = caps.name("hours").unwrap().as_str().parse::<i32>()?;
        let minutes = caps
            .name("minutes")
            .map(|m| m.as_str().parse::<i32>())
            .transpose()?
            .unwrap_or(0);

        // Validate ranges
        if hours > 14 || minutes >= 60 {
            return Err(Error::InvalidDateTime(
                "Timezone offset out of range".to_string(),
            ));
        }

        let total_minutes = sign * (hours * 60 + minutes);

        // Validate total offset (±14 hours max per RFC 3339)
        if total_minutes.abs() > 14 * 60 {
            return Err(Error::InvalidDateTime(
                "Timezone offset exceeds ±14 hours".to_string(),
            ));
        }

        Ok(total_minutes)
    }

    /// Validate datetime component ranges
    fn validate_datetime_components(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<()> {
        if !(1000..=9999).contains(&year) {
            return Err(Error::InvalidDateTime(format!("Invalid year: {}", year)));
        }
        if !(1..=12).contains(&month) {
            return Err(Error::InvalidDateTime(format!("Invalid month: {}", month)));
        }
        if !(1..=31).contains(&day) {
            return Err(Error::InvalidDateTime(format!("Invalid day: {}", day)));
        }
        if hour >= 24 {
            return Err(Error::InvalidDateTime(format!("Invalid hour: {}", hour)));
        }
        if minute >= 60 {
            return Err(Error::InvalidDateTime(format!(
                "Invalid minute: {}",
                minute
            )));
        }
        if second >= 60 {
            return Err(Error::InvalidDateTime(format!(
                "Invalid second: {}",
                second
            )));
        }

        Ok(())
    }

    /// Generate datetime validation warnings
    pub fn validate_datetime_ranges(dt: &ExifDateTime) -> Vec<DateTimeWarning> {
        let mut warnings = Vec::new();

        // Check for future dates
        let now = Utc::now();
        if dt.datetime > now + chrono::Duration::days(1) {
            warnings.push(DateTimeWarning::FutureDate {
                datetime: dt.raw_value.clone(),
            });
        }

        // Check for very old dates (before 1970)
        let year = dt.datetime.year();
        if year < 1970 {
            warnings.push(DateTimeWarning::VeryOldDate {
                datetime: dt.raw_value.clone(),
                year,
            });
        }

        // Check for suspicious timezone offsets
        if let Some(offset_minutes) = dt.timezone_offset_minutes() {
            if offset_minutes.abs() > 12 * 60 {
                warnings.push(DateTimeWarning::SuspiciousTimezone {
                    offset_minutes,
                    reason: "Offset exceeds ±12 hours".to_string(),
                });
            }

            // Check if offset aligns with 15-minute boundaries (most timezones do)
            if offset_minutes % 15 != 0 && offset_minutes % 30 != 0 {
                warnings.push(DateTimeWarning::SuspiciousTimezone {
                    offset_minutes,
                    reason: "Offset not aligned to 15/30 minute boundary".to_string(),
                });
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike, Utc};

    #[test]
    fn test_parse_standard_exif_format() {
        let result = DateTimeParser::parse_exif_datetime("2024:03:15 14:30:00").unwrap();
        assert_eq!(result.raw_value, "2024:03:15 14:30:00");
        assert_eq!(result.datetime.year(), 2024);
        assert_eq!(result.datetime.month(), 3);
        assert_eq!(result.datetime.day(), 15);
        assert_eq!(result.datetime.hour(), 14);
        assert_eq!(result.datetime.minute(), 30);
        assert_eq!(result.datetime.second(), 0);
        assert!(result.local_offset.is_none());
    }

    #[test]
    fn test_parse_with_subseconds() {
        let result = DateTimeParser::parse_exif_datetime("2024:03:15 14:30:00.123").unwrap();
        assert_eq!(result.subsecond, Some(123.0));

        // Test microsecond precision
        let result = DateTimeParser::parse_exif_datetime("2024:03:15 14:30:00.123456").unwrap();
        assert_eq!(result.subsecond, Some(123.456));
    }

    #[test]
    fn test_parse_with_timezone() {
        let result = DateTimeParser::parse_exif_datetime("2024:03:15 14:30:00+05:00").unwrap();
        assert_eq!(result.timezone_offset_minutes(), Some(300)); // 5 hours = 300 minutes
        assert!(result.has_timezone());

        let result = DateTimeParser::parse_exif_datetime("2024:03:15 14:30:00-08:30").unwrap();
        assert_eq!(result.timezone_offset_minutes(), Some(-510)); // -8.5 hours = -510 minutes
    }

    #[test]
    fn test_parse_iso_format() {
        let result = DateTimeParser::parse_exif_datetime("2024-03-15T14:30:00Z").unwrap();
        assert_eq!(result.datetime.year(), 2024);
        assert!(matches!(
            result.inference_source,
            InferenceSource::ExplicitTag { .. }
        ));

        let result = DateTimeParser::parse_exif_datetime("2024-03-15T14:30:00+05:00").unwrap();
        assert_eq!(result.timezone_offset_minutes(), Some(300));
    }

    #[test]
    fn test_parse_subseconds() {
        assert_eq!(DateTimeParser::parse_subseconds("123").unwrap(), 123.0);
        assert_eq!(DateTimeParser::parse_subseconds("1").unwrap(), 100.0);
        assert_eq!(DateTimeParser::parse_subseconds("12").unwrap(), 120.0);
        assert_eq!(DateTimeParser::parse_subseconds("1234").unwrap(), 123.4);
        assert_eq!(DateTimeParser::parse_subseconds("123456").unwrap(), 123.456);
    }

    #[test]
    fn test_parse_timezone_offset() {
        assert_eq!(
            DateTimeParser::parse_timezone_offset("+05:00").unwrap(),
            300
        );
        assert_eq!(
            DateTimeParser::parse_timezone_offset("-08:30").unwrap(),
            -510
        );
        assert_eq!(DateTimeParser::parse_timezone_offset("+0530").unwrap(), 330);
        assert_eq!(DateTimeParser::parse_timezone_offset("-8").unwrap(), -480);

        // Test invalid formats
        assert!(DateTimeParser::parse_timezone_offset("+25:00").is_err());
        assert!(DateTimeParser::parse_timezone_offset("+05:70").is_err());
    }

    #[test]
    fn test_datetime_validation() {
        let future_time = Utc::now() + chrono::Duration::days(30);
        let dt = ExifDateTime::new(
            future_time,
            None,
            "2025:12:31 23:59:59".to_string(),
            InferenceSource::None,
            0.5,
        );

        let warnings = DateTimeParser::validate_datetime_ranges(&dt);
        assert!(warnings
            .iter()
            .any(|w| matches!(w, DateTimeWarning::FutureDate { .. })));

        let old_time = Utc.with_ymd_and_hms(1950, 1, 1, 0, 0, 0).unwrap();
        let dt = ExifDateTime::new(
            old_time,
            None,
            "1950:01:01 00:00:00".to_string(),
            InferenceSource::None,
            0.5,
        );

        let warnings = DateTimeParser::validate_datetime_ranges(&dt);
        assert!(warnings
            .iter()
            .any(|w| matches!(w, DateTimeWarning::VeryOldDate { .. })));
    }

    #[test]
    fn test_invalid_datetime_strings() {
        assert!(DateTimeParser::parse_exif_datetime("").is_err());
        assert!(DateTimeParser::parse_exif_datetime("not a date").is_err());
        assert!(DateTimeParser::parse_exif_datetime("2024:13:15 14:30:00").is_err()); // Invalid month
        assert!(DateTimeParser::parse_exif_datetime("2024:03:32 14:30:00").is_err()); // Invalid day
        assert!(DateTimeParser::parse_exif_datetime("2024:03:15 25:30:00").is_err());
        // Invalid hour
    }

    #[test]
    fn test_loose_format_parsing() {
        let result = DateTimeParser::parse_exif_datetime("Mar 15 2024 14:30:00").unwrap();
        assert_eq!(result.datetime.year(), 2024);
        assert_eq!(result.datetime.month(), 3);
        assert_eq!(result.datetime.day(), 15);
        assert!(result.confidence < 0.8); // Lower confidence for loose parsing

        let result = DateTimeParser::parse_exif_datetime("Thu Mar 15 14:30:00 2024").unwrap();
        assert_eq!(result.datetime.year(), 2024);
    }
}
