//! Model detection pattern generator for manufacturer-specific logic
//!
//! This generator creates Rust code from ExifTool model detection patterns,
//! providing camera model matching and conditional logic for tag definitions.

use crate::common::escape_string;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelDetectionExtraction {
    pub manufacturer: String,
    pub source: SourceInfo,
    pub patterns_data: PatternsData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatternsData {
    pub table_name: String,
    pub patterns: Vec<ModelPattern>,
    pub conditional_tags: Vec<ConditionalTag>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelPattern {
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub operator: String,
    pub pattern: String,
    pub condition_context: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionalTag {
    pub tag_id: String,
    pub conditions: Vec<ConditionData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionData {
    pub condition: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub subdirectory: bool,
}


/// Generate Rust code for model detection patterns
pub fn generate_model_detection(data: &ModelDetectionExtraction) -> Result<String> {
    let mut code = String::new();
    
    // Add header comment
    code.push_str(&format!(
        "//! {} model detection patterns from {} table\n",
        data.manufacturer, data.patterns_data.table_name
    ));
    code.push_str(&format!(
        "//! ExifTool: {} %{}::{}\n",
        data.source.module, data.manufacturer, data.patterns_data.table_name
    ));
    code.push_str(&format!(
        "//! Generated at: {}\n\n",
        data.source.extracted_at
    ));
    
    // Add imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use regex::Regex;\n\n");
    
    // Generate model pattern matcher if we have patterns
    if !data.patterns_data.patterns.is_empty() {
        code.push_str(&generate_model_matcher(data)?);
        code.push_str("\n");
    }
    
    // Generate conditional tag dispatcher if we have conditional tags
    if !data.patterns_data.conditional_tags.is_empty() {
        code.push_str(&generate_conditional_tag_dispatcher(data)?);
        code.push_str("\n");
    }
    
    // Generate the main model detection structure
    let struct_name = format!("{}ModelDetection", data.manufacturer);
    
    code.push_str(&format!(
        "/// {} model detection and conditional tag resolution\n",
        data.manufacturer
    ));
    code.push_str(&format!(
        "/// Patterns: {}, Conditional tags: {}\n",
        data.patterns_data.patterns.len(),
        data.patterns_data.conditional_tags.len()
    ));
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {} {{\n", struct_name));
    code.push_str("    /// Model string for pattern matching\n");
    code.push_str("    pub model: String,\n");
    code.push_str("}\n\n");
    
    // Generate implementation
    code.push_str(&format!("impl {} {{\n", struct_name));
    
    // Constructor
    code.push_str("    /// Create new model detection instance\n");
    code.push_str("    pub fn new(model: String) -> Self {\n");
    code.push_str("        Self { model }\n");
    code.push_str("    }\n\n");
    
    // Model matching methods
    if !data.patterns_data.patterns.is_empty() {
        code.push_str("    /// Check if model matches any known patterns\n");
        code.push_str("    pub fn matches_pattern(&self, pattern_type: &str) -> bool {\n");
        code.push_str("        match pattern_type {\n");
        
        // Group patterns by type for efficient matching
        let mut pattern_groups: HashMap<String, Vec<&ModelPattern>> = HashMap::new();
        for pattern in &data.patterns_data.patterns {
            pattern_groups.entry(pattern.pattern_type.clone())
                .or_default()
                .push(pattern);
        }
        
        for (pattern_type, patterns) in pattern_groups {
            code.push_str(&format!("            \"{}\" => {{\n", pattern_type));
            for pattern in patterns {
                match pattern.pattern_type.as_str() {
                    "regex" => {
                        code.push_str(&format!(
                            "                if MODEL_PATTERN_{}.is_match(&self.model) {{ return true; }}\n",
                            sanitize_pattern_name(&pattern.pattern)
                        ));
                    }
                    "string" => {
                        if pattern.operator.contains('=') {
                            code.push_str(&format!(
                                "                if self.model == \"{}\" {{ return true; }}\n",
                                escape_string(&pattern.pattern)
                            ));
                        } else if pattern.operator.contains("!") {
                            code.push_str(&format!(
                                "                if self.model != \"{}\" {{ return true; }}\n",
                                escape_string(&pattern.pattern)
                            ));
                        }
                    }
                    _ => {
                        // Skip complex expressions for now
                        code.push_str(&format!(
                            "                // TODO: Implement {} pattern: {}\n",
                            pattern.pattern_type, escape_string(&pattern.pattern)
                        ));
                    }
                }
            }
            code.push_str("                false\n");
            code.push_str("            }\n");
        }
        
        code.push_str("            _ => false,\n");
        code.push_str("        }\n");
        code.push_str("    }\n\n");
    }
    
    // Conditional tag resolution
    if !data.patterns_data.conditional_tags.is_empty() {
        code.push_str("    /// Resolve conditional tag based on model and other conditions\n");
        code.push_str("    pub fn resolve_conditional_tag(&self, tag_id: &str, context: &ConditionalContext) -> Option<&'static str> {\n");
        code.push_str("        CONDITIONAL_TAG_RESOLVER.get(tag_id)?\n");
        code.push_str("            .iter()\n");
        code.push_str("            .find(|condition| self.evaluate_condition(&condition.condition, context))\n");
        code.push_str("            .map(|condition| condition.name)\n");
        code.push_str("    }\n\n");
        
        code.push_str("    /// Evaluate a single condition against the current context\n");
        code.push_str("    fn evaluate_condition(&self, condition: &str, context: &ConditionalContext) -> bool {\n");
        code.push_str("        // Simplified condition evaluation - can be enhanced\n");
        code.push_str("        if condition.contains(\"$$self{Model}\") {\n");
        code.push_str("            return self.evaluate_model_condition(condition);\n");
        code.push_str("        }\n");
        code.push_str("        if condition.contains(\"$$self{Make}\") {\n");
        code.push_str("            if let Some(make) = &context.make {\n");
        code.push_str("                return self.evaluate_make_condition(condition, make);\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        false\n");
        code.push_str("    }\n\n");
        
        code.push_str("    /// Evaluate model-specific conditions\n");
        code.push_str("    fn evaluate_model_condition(&self, condition: &str) -> bool {\n");
        code.push_str("        // Simple string matching for now - can be enhanced with regex\n");
        code.push_str("        if condition.contains(\" eq \") {\n");
        code.push_str("            if let Some(quoted) = extract_quoted_string(condition) {\n");
        code.push_str("                return self.model == quoted;\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        if condition.contains(\" ne \") {\n");
        code.push_str("            if let Some(quoted) = extract_quoted_string(condition) {\n");
        code.push_str("                return self.model != quoted;\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        // TODO: Implement regex matching for =~ patterns\n");
        code.push_str("        false\n");
        code.push_str("    }\n\n");
        
        code.push_str("    /// Evaluate make-specific conditions\n");
        code.push_str("    fn evaluate_make_condition(&self, condition: &str, make: &str) -> bool {\n");
        code.push_str("        if condition.contains(\" =~ \") {\n");
        code.push_str("            // Simple substring matching for now\n");
        code.push_str("            if condition.contains(\"/^GENERAL IMAGING/\") {\n");
        code.push_str("                return make.starts_with(\"GENERAL IMAGING\");\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("        false\n");
        code.push_str("    }\n");
    }
    
    code.push_str("}\n\n");
    
    // Generate context structure
    code.push_str("/// Context for evaluating conditional tag conditions\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct ConditionalContext {\n");
    code.push_str("    pub make: Option<String>,\n");
    code.push_str("    pub count: Option<u32>,\n");
    code.push_str("    pub format: Option<String>,\n");
    code.push_str("}\n\n");
    
    // Helper functions
    code.push_str("/// Extract quoted string from Perl condition\n");
    code.push_str("fn extract_quoted_string(condition: &str) -> Option<String> {\n");
    code.push_str("    if let Some(start) = condition.find('\"') {\n");
    code.push_str("        if let Some(end) = condition[start + 1..].find('\"') {\n");
    code.push_str("            return Some(condition[start + 1..start + 1 + end].to_string());\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    None\n");
    code.push_str("}\n");
    
    Ok(code)
}

/// Generate regex patterns for model matching
fn generate_model_matcher(data: &ModelDetectionExtraction) -> Result<String> {
    let mut code = String::new();
    
    code.push_str("/// Compiled regex patterns for model detection\n");
    
    for pattern in &data.patterns_data.patterns {
        if pattern.pattern_type == "regex" {
            let pattern_name = sanitize_pattern_name(&pattern.pattern);
            code.push_str(&format!(
                "static MODEL_PATTERN_{}: LazyLock<Regex> = LazyLock::new(|| {{\n",
                pattern_name
            ));
            code.push_str(&format!(
                "    Regex::new(r\"{}\").expect(\"Valid regex pattern for {}\")\n",
                escape_regex_pattern(&pattern.pattern),
                pattern.pattern
            ));
            code.push_str("});\n\n");
        }
    }
    
    Ok(code)
}

/// Generate conditional tag dispatcher
fn generate_conditional_tag_dispatcher(data: &ModelDetectionExtraction) -> Result<String> {
    let mut code = String::new();
    
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct ConditionalTagEntry {\n");
    code.push_str("    pub condition: &'static str,\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("}\n\n");
    
    code.push_str("/// Conditional tag resolution mapping\n");
    code.push_str("static CONDITIONAL_TAG_RESOLVER: LazyLock<HashMap<&'static str, Vec<ConditionalTagEntry>>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");
    
    for conditional_tag in &data.patterns_data.conditional_tags {
        code.push_str(&format!("    map.insert(\"{}\", vec![\n", conditional_tag.tag_id));
        
        for condition in &conditional_tag.conditions {
            code.push_str("        ConditionalTagEntry {\n");
            code.push_str(&format!(
                "            condition: \"{}\",\n",
                escape_string(&condition.condition)
            ));
            code.push_str(&format!(
                "            name: \"{}\",\n",
                escape_string(&condition.name)
            ));
            code.push_str("        },\n");
        }
        
        code.push_str("    ]);\n");
    }
    
    code.push_str("    map\n");
    code.push_str("});\n");
    
    Ok(code)
}

/// Sanitize pattern name for use as Rust identifier
fn sanitize_pattern_name(pattern: &str) -> String {
    pattern
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_uppercase()
}

/// Escape regex pattern for Rust raw string literal
fn escape_regex_pattern(pattern: &str) -> String {
    // For now, return the pattern as-is for raw string literals
    // More sophisticated escaping may be needed for complex patterns
    pattern.to_string()
}