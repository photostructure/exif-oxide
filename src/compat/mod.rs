//! ExifTool compatibility module
//!
//! This module provides utilities for comparing exif-oxide output with ExifTool
//! using the same normalization logic as our compatibility tests.
//!
//! Key features:
//! - Value normalization to handle format differences (strings vs numbers, units, etc.)
//! - Tolerance-based comparison for GPS coordinates and numeric values
//! - Structured difference reporting
//! - Support for group-based filtering (File:, EXIF:, etc.)

pub mod comparison;
pub mod filtering;
pub mod normalization;
pub mod reporting;

pub use comparison::*;
pub use filtering::*;
pub use normalization::*;
pub use reporting::*;

use serde_json::Value;
use std::collections::HashMap;

/// Load supported tags from the configuration file
pub fn load_supported_tags() -> Vec<String> {
    const CONFIG_JSON: &str = include_str!("../../config/supported_tags.json");
    serde_json::from_str(CONFIG_JSON).expect("Failed to parse supported_tags.json")
}

/// Filter JSON object to only include supported tags
/// Handles group-prefixed tag names from supported_tags.json
pub fn filter_to_supported_tags(data: &Value) -> Value {
    if let Some(obj) = data.as_object() {
        let supported_tags = load_supported_tags();
        let supported_tag_refs: Vec<&str> = supported_tags.iter().map(|s| s.as_str()).collect();

        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| {
                // Always include SourceFile
                if key.as_str() == "SourceFile" {
                    return true;
                }

                // Check if the full group:tag key is in the supported list
                supported_tag_refs.contains(&key.as_str())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
    }
}

/// Filter JSON object to only include tags from specific groups
/// Groups should be specified with colon (e.g., "File:", "EXIF:", "MakerNotes:")
pub fn filter_to_groups(data: &Value, groups: &[&str]) -> Value {
    if let Some(obj) = data.as_object() {
        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| {
                // Always include SourceFile regardless of group filtering
                if key.as_str() == "SourceFile" {
                    return true;
                }

                // Check if key starts with any of the specified groups
                groups.iter().any(|group| key.starts_with(group))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
    }
}

/// Filter JSON object to only include specific tags by name
/// Tags should be specified as full tag names (e.g., "Composite:Lens", "EXIF:Make")
pub fn filter_to_custom_tags(data: &Value, tags: &[&str]) -> Value {
    if let Some(obj) = data.as_object() {
        let filtered: HashMap<String, Value> = obj
            .iter()
            .filter(|(key, _)| {
                // Always include SourceFile regardless of tag filtering
                if key.as_str() == "SourceFile" {
                    return true;
                }

                // Check if key matches any of the specified tags exactly
                tags.contains(&key.as_str())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::to_value(filtered).unwrap()
    } else {
        data.clone()
    }
}

/// Run exif-oxide and return parsed JSON for a file
pub fn run_exif_oxide(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(crate::extract_metadata_json(file_path)?)
}

/// Run ExifTool with the standard flags and return parsed JSON
pub fn run_exiftool(file_path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("exiftool")
        .args(["-j", "-struct", "-G", file_path])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ExifTool failed with exit code: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    // ExifTool returns an array with one object for single files
    if let Some(array) = json.as_array() {
        if let Some(first) = array.first() {
            Ok(first.clone())
        } else {
            Err("ExifTool returned empty array".into())
        }
    } else {
        Ok(json)
    }
}
