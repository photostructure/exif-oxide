//! Reporting and difference analysis for ExifTool compatibility
//!
//! This module provides structured analysis and reporting of differences between
//! ExifTool and exif-oxide output.

use crate::compat::{comparison::*, load_supported_tags};
// use crate::generated::composite_tags::lookup_composite_tag;
use serde_json::Value;
use std::collections::HashSet;

/// Analysis result for a specific tag difference
#[derive(Debug, Clone)]
pub struct TagDifference {
    pub tag: String,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
    pub difference_type: DifferenceType,
    pub sample_file: String,
}

impl TagDifference {
    /// Format a value for display, truncating large arrays to avoid massive output
    pub fn format_value_truncated(value: &Option<Value>) -> String {
        match value {
            Some(Value::Array(arr)) if arr.len() > 10 => {
                format!(
                    "Array[{}] with {} elements (showing first 3: {:?}...)",
                    arr.len(),
                    arr.len(),
                    arr.iter().take(3).collect::<Vec<_>>()
                )
            }
            Some(Value::String(s)) if s.len() > 100 => {
                format!("String[{}]: \"{}...\"", s.len(), &s[..97])
            }
            Some(val) => format!("Some({:?})", val),
            None => "None".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DifferenceType {
    Working,             // Values match exactly
    ValueFormatMismatch, // Same data, different format (e.g., "1.5" vs 1.5)
    Missing,             // Tag missing from our output
    DependencyFailure,   // Composite tag missing due to unmet dependencies
    TypeMismatch,        // Completely different data types
    OnlyInOurs,          // Tag only exists in our output
}

/// Structured compatibility analysis result
#[derive(Debug)]
pub struct CompatibilityReport {
    pub working_tags: Vec<String>,
    pub value_format_mismatches: Vec<TagDifference>,
    pub missing_tags: Vec<TagDifference>,
    pub dependency_failures: Vec<TagDifference>,
    pub type_mismatches: Vec<TagDifference>,
    pub only_in_ours: Vec<TagDifference>,
    pub total_tags_tested: usize,
    pub total_files_tested: usize,
}

impl CompatibilityReport {
    pub fn new() -> Self {
        Self {
            working_tags: Vec::new(),
            value_format_mismatches: Vec::new(),
            missing_tags: Vec::new(),
            dependency_failures: Vec::new(),
            type_mismatches: Vec::new(),
            only_in_ours: Vec::new(),
            total_tags_tested: 0,
            total_files_tested: 0,
        }
    }

    pub fn print_summary(&self) {
        let total_failing = self.value_format_mismatches.len()
            + self.missing_tags.len()
            + self.dependency_failures.len()
            + self.type_mismatches.len();
        let success_rate = if self.total_tags_tested > 0 {
            (self.working_tags.len() * 100) / self.total_tags_tested
        } else {
            0
        };

        let sample_size = 30;

        println!("\n🎯 ExifTool Compatibility Report");
        println!("=================================");
        println!("Files tested: {}", self.total_files_tested);
        println!("Unique tags tested: {}", self.total_tags_tested);
        println!(
            "Success rate: {}% ({}/{})",
            success_rate,
            self.working_tags.len(),
            self.total_tags_tested
        );
        println!();

        if !self.working_tags.is_empty() {
            println!("✅ WORKING ({} tags):", self.working_tags.len());
            let mut sorted_working = self.working_tags.clone();
            sorted_working.sort();
            for tag in sorted_working.iter().take(10) {
                println!("  {}", tag);
            }
            if self.working_tags.len() > 10 {
                println!("  ... and {} more", self.working_tags.len() - 10);
            }
            println!();
        }

        if !self.value_format_mismatches.is_empty() {
            println!(
                "⚠️  VALUE FORMAT MISMATCHES ({} tags):",
                self.value_format_mismatches.len()
            );
            for diff in self.value_format_mismatches.iter().take(sample_size) {
                println!(
                    "  {}: Expected: {}, Got: {} ({})",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    TagDifference::format_value_truncated(&diff.actual),
                    diff.sample_file
                );
            }
            if self.value_format_mismatches.len() > sample_size {
                println!(
                    "  ... and {} more",
                    self.value_format_mismatches.len() - sample_size
                );
            }
            println!();
        }

        if !self.missing_tags.is_empty() {
            println!("❌ MISSING TAGS ({} tags):", self.missing_tags.len());
            for diff in self.missing_tags.iter().take(sample_size) {
                println!(
                    "  {}: Expected: {} ({})",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    diff.sample_file
                );
            }
            if self.missing_tags.len() > sample_size {
                println!("  ... and {} more", self.missing_tags.len() - sample_size);
            }
            println!();
        }

        if !self.dependency_failures.is_empty() {
            println!(
                "🔗 MISSING COMPOSITE DEPENDENCIES ({} tags):",
                self.dependency_failures.len()
            );
            for diff in self.dependency_failures.iter().take(sample_size) {
                println!(
                    "  {}: Expected: {} ({})",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    diff.sample_file
                );
            }
            if self.dependency_failures.len() > sample_size {
                println!(
                    "  ... and {} more",
                    self.dependency_failures.len() - sample_size
                );
            }
            println!();
        }

        if !self.type_mismatches.is_empty() {
            println!("🔥 TYPE MISMATCHES ({} tags):", self.type_mismatches.len());
            for diff in self.type_mismatches.iter().take(sample_size) {
                println!(
                    "  {}: Expected: {}, Got: {} ({})",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    TagDifference::format_value_truncated(&diff.actual),
                    diff.sample_file
                );
            }
            if self.type_mismatches.len() > sample_size {
                println!(
                    "  ... and {} more",
                    self.type_mismatches.len() - sample_size
                );
            }
            println!();
        }

        if !self.only_in_ours.is_empty() {
            println!("ℹ️  ONLY IN EXIF-OXIDE ({} tags):", self.only_in_ours.len());
            for diff in self.only_in_ours.iter().take(sample_size) {
                println!(
                    "  {}: {} ({})",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.actual),
                    diff.sample_file
                );
            }
            if self.only_in_ours.len() > sample_size {
                println!("  ... and {} more", self.only_in_ours.len() - sample_size);
            }
            println!();
        }

        println!(
            "Summary: {}% working, {} needing attention",
            success_rate, total_failing
        );
    }

    /// Print a simplified summary focused on the most important differences
    pub fn print_simple_differences(&self, max_items: usize) {
        println!("📊 Comparison Summary");
        println!("==================");

        if !self.missing_tags.is_empty() {
            println!("\n❌ Missing in exif-oxide ({}):", self.missing_tags.len());
            for diff in self.missing_tags.iter().take(max_items) {
                println!(
                    "  {}: {}",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected)
                );
            }
            if self.missing_tags.len() > max_items {
                println!("  ... and {} more", self.missing_tags.len() - max_items);
            }
        }

        if !self.only_in_ours.is_empty() {
            println!("\n➕ Only in exif-oxide ({}):", self.only_in_ours.len());
            for diff in self.only_in_ours.iter().take(max_items) {
                println!(
                    "  {}: {}",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.actual)
                );
            }
            if self.only_in_ours.len() > max_items {
                println!("  ... and {} more", self.only_in_ours.len() - max_items);
            }
        }

        if !self.type_mismatches.is_empty() {
            println!("\n🔄 Different values ({}):", self.type_mismatches.len());
            for diff in self.type_mismatches.iter().take(max_items) {
                println!(
                    "  {}: ExifTool={}, exif-oxide={}",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    TagDifference::format_value_truncated(&diff.actual)
                );
            }
            if self.type_mismatches.len() > max_items {
                println!("  ... and {} more", self.type_mismatches.len() - max_items);
            }
        }

        if !self.value_format_mismatches.is_empty() {
            println!(
                "\n⚠️  Format differences ({}):",
                self.value_format_mismatches.len()
            );
            for diff in self.value_format_mismatches.iter().take(max_items) {
                println!(
                    "  {}: ExifTool={}, exif-oxide={}",
                    diff.tag,
                    TagDifference::format_value_truncated(&diff.expected),
                    TagDifference::format_value_truncated(&diff.actual)
                );
            }
            if self.value_format_mismatches.len() > max_items {
                println!(
                    "  ... and {} more",
                    self.value_format_mismatches.len() - max_items
                );
            }
        }

        let total_differences = self.missing_tags.len()
            + self.only_in_ours.len()
            + self.type_mismatches.len()
            + self.value_format_mismatches.len();

        if total_differences == 0 {
            println!(
                "\n✅ Perfect match! All {} tags are identical.",
                self.working_tags.len()
            );
        } else {
            println!(
                "\n📈 {} working, {} differences total",
                self.working_tags.len(),
                total_differences
            );
        }
    }
}

/// Analyze differences between ExifTool reference and our output for a single file
pub fn analyze_tag_differences(
    file_path: &str,
    exiftool_data: &Value,
    our_data: &Value,
) -> Vec<TagDifference> {
    let mut differences = Vec::new();
    let supported_tags: HashSet<String> = load_supported_tags().into_iter().collect();

    let empty_map = serde_json::Map::new();
    let exiftool_obj = exiftool_data.as_object().unwrap_or(&empty_map);
    let our_obj = our_data.as_object().unwrap_or(&empty_map);

    // Check all supported tags
    for tag in &supported_tags {
        if tag == "SourceFile" {
            continue; // Skip SourceFile as it's always different
        }

        let expected = exiftool_obj.get(tag);
        let actual = our_obj.get(tag);

        let difference_type = match (expected, actual) {
            (Some(exp), Some(act)) => {
                if values_match_semantically(exp, act) || values_match_with_tolerance(tag, exp, act)
                {
                    DifferenceType::Working // Semantic matches or GPS coordinates within tolerance should be Working
                } else if same_data_different_format_with_tag(tag, exp, act) {
                    DifferenceType::ValueFormatMismatch
                } else {
                    DifferenceType::TypeMismatch
                }
            }
            (Some(_), None) => {
                // Check if this is a composite tag with unmet dependencies
                if is_composite_dependency_failure(tag, our_obj) {
                    DifferenceType::DependencyFailure
                } else {
                    DifferenceType::Missing
                }
            }
            (None, Some(_)) => DifferenceType::OnlyInOurs,
            (None, None) => continue, // Tag not present in either - skip, don't count
        };

        differences.push(TagDifference {
            tag: tag.clone(),
            expected: expected.cloned(),
            actual: actual.cloned(),
            difference_type,
            sample_file: file_path.to_string(),
        });
    }

    differences
}

/// Check if a composite tag has unmet dependencies
/// Returns true if the tag is a composite tag and its required dependencies are not available
fn is_composite_dependency_failure(tag: &str, our_obj: &serde_json::Map<String, Value>) -> bool {
    // Only check composite tags
    if !tag.starts_with("Composite:") {
        return false;
    }

    // Strip the "Composite:" prefix to get the tag name
    let tag_name = &tag[10..];

    // Look up the composite tag definition
    // TODO: Re-enable when composite_tags is generated
    /*
    if let Some(composite_def) = lookup_composite_tag(tag_name) {
        // Check if all required dependencies are available in our output
        for required_tag in &composite_def.require {
            // Check various possible tag name formats
            let possible_names = [
                required_tag.to_string(),
                format!(
                    "{}:{}",
                    required_tag.split(':').next().unwrap_or(required_tag),
                    required_tag.split(':').next_back().unwrap_or(required_tag)
                ),
                required_tag
                    .split(':')
                    .next_back()
                    .unwrap_or(required_tag)
                    .to_string(),
            ];

            let mut found = false;
            for name in &possible_names {
                if our_obj.contains_key(name) {
                    found = true;
                    break;
                }
            }

            if !found {
                // Missing a required dependency
                return true;
            }
        }
    }
    */

    false
}

/// Analyze differences between two JSON objects without filtering to supported tags
/// Useful for ad-hoc comparison of any two JSON outputs
pub fn analyze_all_tag_differences(
    file_path: &str,
    exiftool_data: &Value,
    our_data: &Value,
) -> Vec<TagDifference> {
    let mut differences = Vec::new();

    let empty_map = serde_json::Map::new();
    let exiftool_obj = exiftool_data.as_object().unwrap_or(&empty_map);
    let our_obj = our_data.as_object().unwrap_or(&empty_map);

    // Collect all unique tags from both outputs
    let mut all_tags: HashSet<String> = HashSet::new();
    all_tags.extend(exiftool_obj.keys().cloned());
    all_tags.extend(our_obj.keys().cloned());

    // Analyze each tag
    for tag in all_tags {
        if tag == "SourceFile" {
            continue; // Skip SourceFile as it's always different
        }

        let expected = exiftool_obj.get(&tag);
        let actual = our_obj.get(&tag);

        let difference_type = match (expected, actual) {
            (Some(exp), Some(act)) => {
                if values_match_semantically(exp, act)
                    || values_match_with_tolerance(&tag, exp, act)
                {
                    DifferenceType::Working
                } else if same_data_different_format_with_tag(&tag, exp, act) {
                    DifferenceType::ValueFormatMismatch
                } else {
                    DifferenceType::TypeMismatch
                }
            }
            (Some(_), None) => DifferenceType::Missing,
            (None, Some(_)) => DifferenceType::OnlyInOurs,
            (None, None) => continue, // Tag not present in either
        };

        differences.push(TagDifference {
            tag: tag.clone(),
            expected: expected.cloned(),
            actual: actual.cloned(),
            difference_type,
            sample_file: file_path.to_string(),
        });
    }

    differences
}
