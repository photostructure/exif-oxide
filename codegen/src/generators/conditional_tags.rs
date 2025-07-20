//! Conditional tag definition generator for complex ExifTool logic
//!
//! This generator creates Rust code from complex ExifTool conditional tag definitions,
//! handling count-based conditions, binary pattern matching, and cross-tag dependencies.

use crate::common::escape_string;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionalTagsExtraction {
    pub manufacturer: String,
    pub source: SourceInfo,
    pub conditional_data: ConditionalData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub table: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionalData {
    pub table_name: String,
    pub conditional_arrays: Vec<ConditionalArray>,
    pub count_conditions: Vec<ConditionalEntry>,
    pub binary_patterns: Vec<ConditionalEntry>,
    pub format_conditions: Vec<ConditionalEntry>,
    pub cross_tag_dependencies: Vec<ConditionalEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionalArray {
    pub tag_id: String,
    pub conditions: Vec<ConditionalEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConditionalEntry {
    pub tag_id: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_member: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_conv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_conv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_conv: Option<serde_json::Value>,
}

/// Generate Rust code for conditional tag definitions
pub fn generate_conditional_tags(data: &ConditionalTagsExtraction) -> Result<String> {
    let mut code = String::new();

    // Add header comment
    code.push_str(&format!(
        "//! {} conditional tag definitions from {} table\n",
        data.manufacturer, data.conditional_data.table_name
    ));
    code.push_str(&format!(
        "//! ExifTool: {} %{}::{}\n",
        data.source.module, data.manufacturer, data.conditional_data.table_name
    ));

    // Add imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use crate::expressions::{ExpressionEvaluator, parse_expression};\n");
    code.push_str("use crate::processor_registry::ProcessorContext;\n");
    code.push_str("use crate::types::TagValue;\n\n");

    // Generate context structure for condition evaluation
    code.push_str(&generate_evaluation_context()?);
    code.push_str("\n");

    // Generate condition resolvers
    if !data.conditional_data.conditional_arrays.is_empty() {
        code.push_str(&generate_conditional_array_resolver(data)?);
        code.push_str("\n");
    }

    if !data.conditional_data.count_conditions.is_empty() {
        code.push_str(&generate_count_condition_resolver(data)?);
        code.push_str("\n");
    }

    if !data.conditional_data.binary_patterns.is_empty() {
        code.push_str(&generate_binary_pattern_resolver(data)?);
        code.push_str("\n");
    }

    // Generate the main conditional tag processor
    let struct_name = format!("{}ConditionalTags", data.manufacturer);

    code.push_str(&format!(
        "/// {} conditional tag resolution engine\n",
        data.manufacturer
    ));
    code.push_str(&format!(
        "/// Arrays: {}, Count: {}, Binary: {}, Format: {}, Dependencies: {}\n",
        data.conditional_data.conditional_arrays.len(),
        data.conditional_data.count_conditions.len(),
        data.conditional_data.binary_patterns.len(),
        data.conditional_data.format_conditions.len(),
        data.conditional_data.cross_tag_dependencies.len()
    ));
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {} {{}}\n\n", struct_name));

    // Generate implementation
    code.push_str(&format!("impl {} {{\n", struct_name));

    // Constructor
    code.push_str("    /// Create new conditional tag processor\n");
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        Self {}\n");
    code.push_str("    }\n\n");

    // Main resolution method
    code.push_str("    /// Resolve conditional tag based on context\n");
    code.push_str("    pub fn resolve_tag(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag> {\n");
    code.push_str("        // Try conditional arrays first\n");
    code.push_str(
        "        if let Some(resolved) = self.resolve_conditional_array(tag_id, context) {\n",
    );
    code.push_str("            return Some(resolved);\n");
    code.push_str("        }\n\n");
    code.push_str("        // Try count-based conditions\n");
    code.push_str(
        "        if let Some(resolved) = self.resolve_count_condition(tag_id, context) {\n",
    );
    code.push_str("            return Some(resolved);\n");
    code.push_str("        }\n\n");
    code.push_str("        // Try binary pattern matching\n");
    code.push_str(
        "        if let Some(resolved) = self.resolve_binary_pattern(tag_id, context) {\n",
    );
    code.push_str("            return Some(resolved);\n");
    code.push_str("        }\n\n");
    code.push_str("        None\n");
    code.push_str("    }\n\n");

    // Individual resolution methods
    if !data.conditional_data.conditional_arrays.is_empty() {
        code.push_str("    /// Resolve using conditional arrays\n");
        code.push_str("    fn resolve_conditional_array(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag> {\n");
        code.push_str("        CONDITIONAL_ARRAYS.get(tag_id)?\n");
        code.push_str("            .iter()\n");
        code.push_str(
            "            .find(|entry| self.evaluate_condition(&entry.condition, context))\n",
        );
        code.push_str("            .map(|entry| ResolvedTag {\n");
        code.push_str("                name: entry.name.to_string(),\n");
        code.push_str("                subdirectory: entry.subdirectory,\n");
        code.push_str("                writable: entry.writable,\n");
        code.push_str("                format: entry.format.map(|s| s.to_string()),\n");
        code.push_str("            })\n");
        code.push_str("    }\n\n");
    }

    if !data.conditional_data.count_conditions.is_empty() {
        code.push_str("    /// Resolve using count conditions\n");
        code.push_str("    fn resolve_count_condition(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag> {\n");
        code.push_str("        COUNT_CONDITIONS.get(tag_id)?\n");
        code.push_str("            .iter()\n");
        code.push_str("            .find(|entry| self.evaluate_count_condition(&entry.condition, context.count))\n");
        code.push_str("            .map(|entry| ResolvedTag {\n");
        code.push_str("                name: entry.name.to_string(),\n");
        code.push_str("                subdirectory: entry.subdirectory,\n");
        code.push_str("                writable: entry.writable,\n");
        code.push_str("                format: entry.format.map(|s| s.to_string()),\n");
        code.push_str("            })\n");
        code.push_str("    }\n\n");
    }

    if !data.conditional_data.binary_patterns.is_empty() {
        code.push_str("    /// Resolve using binary pattern matching\n");
        code.push_str("    fn resolve_binary_pattern(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag> {\n");
        code.push_str("        if let Some(binary_data) = &context.binary_data {\n");
        code.push_str("            BINARY_PATTERNS.get(tag_id)?\n");
        code.push_str("                .iter()\n");
        code.push_str("                .find(|entry| self.evaluate_binary_pattern(&entry.condition, binary_data))\n");
        code.push_str("                .map(|entry| ResolvedTag {\n");
        code.push_str("                    name: entry.name.to_string(),\n");
        code.push_str("                    subdirectory: entry.subdirectory,\n");
        code.push_str("                    writable: entry.writable,\n");
        code.push_str("                    format: entry.format.map(|s| s.to_string()),\n");
        code.push_str("                })\n");
        code.push_str("        } else {\n");
        code.push_str("            None\n");
        code.push_str("        }\n");
        code.push_str("    }\n\n");
    }

    // Unified expression evaluation using the shared system
    code.push_str("    /// Evaluate a condition using the unified expression system\n");
    code.push_str("    fn evaluate_condition(&self, condition: &str, context: &ConditionalContext) -> bool {\n");
    code.push_str("        let mut evaluator = ExpressionEvaluator::new();\n");
    code.push_str("        \n");
    code.push_str("        // Build ProcessorContext from ConditionalContext\n");
    code.push_str("        let mut processor_context = ProcessorContext::default();\n");
    code.push_str("        if let Some(model) = &context.model {\n");
    code.push_str("            processor_context = processor_context.with_model(model.clone());\n");
    code.push_str("        }\n");
    code.push_str("        if let Some(make) = &context.make {\n");
    code.push_str("            processor_context = processor_context.with_manufacturer(make.clone());\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        // Add conditional context values to processor context\n");
    code.push_str("        if let Some(count) = context.count {\n");
    code.push_str("            processor_context.parent_tags.insert(\"count\".to_string(), TagValue::U32(count));\n");
    code.push_str("        }\n");
    code.push_str("        if let Some(format) = &context.format {\n");
    code.push_str("            processor_context.parent_tags.insert(\"format\".to_string(), TagValue::String(format.clone()));\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        // Try context-based evaluation first\n");
    code.push_str("        if let Ok(result) = evaluator.evaluate_context_condition(&processor_context, condition) {\n");
    code.push_str("            return result;\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        false\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// Evaluate count-based conditions using unified system\n");
    code.push_str("    fn evaluate_count_condition(&self, condition: &str, count: Option<u32>) -> bool {\n");
    code.push_str("        let mut evaluator = ExpressionEvaluator::new();\n");
    code.push_str("        let mut processor_context = ProcessorContext::default();\n");
    code.push_str("        \n");
    code.push_str("        if let Some(count_val) = count {\n");
    code.push_str("            processor_context.parent_tags.insert(\"count\".to_string(), TagValue::U32(count_val));\n");
    code.push_str("        }\n");
    code.push_str("        \n");
    code.push_str("        evaluator.evaluate_context_condition(&processor_context, condition).unwrap_or(false)\n");
    code.push_str("    }\n\n");
    
    code.push_str("    /// Evaluate binary pattern conditions using unified system\n");
    code.push_str("    fn evaluate_binary_pattern(&self, condition: &str, binary_data: &[u8]) -> bool {\n");
    code.push_str("        let mut evaluator = ExpressionEvaluator::new();\n");
    code.push_str("        evaluator.evaluate_data_condition(binary_data, condition).unwrap_or(false)\n");
    code.push_str("    }\n");

    code.push_str("}\n\n");

    Ok(code)
}

/// Generate evaluation context structure
fn generate_evaluation_context() -> Result<String> {
    let mut code = String::new();

    code.push_str("/// Context for evaluating conditional tag conditions\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct ConditionalContext {\n");
    code.push_str("    pub model: Option<String>,\n");
    code.push_str("    pub make: Option<String>,\n");
    code.push_str("    pub count: Option<u32>,\n");
    code.push_str("    pub format: Option<String>,\n");
    code.push_str("    pub binary_data: Option<Vec<u8>>,\n");
    code.push_str("}\n\n");

    code.push_str("/// Resolved tag information\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct ResolvedTag {\n");
    code.push_str("    pub name: String,\n");
    code.push_str("    pub subdirectory: bool,\n");
    code.push_str("    pub writable: bool,\n");
    code.push_str("    pub format: Option<String>,\n");
    code.push_str("}\n\n");

    code.push_str("/// Conditional entry for resolution\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub struct ConditionalEntry {\n");
    code.push_str("    pub condition: &'static str,\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub subdirectory: bool,\n");
    code.push_str("    pub writable: bool,\n");
    code.push_str("    pub format: Option<&'static str>,\n");
    code.push_str("}\n");

    Ok(code)
}

/// Generate conditional array resolver
fn generate_conditional_array_resolver(data: &ConditionalTagsExtraction) -> Result<String> {
    let mut code = String::new();

    code.push_str("/// Conditional array resolution mapping\n");
    code.push_str("static CONDITIONAL_ARRAYS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");

    for array in &data.conditional_data.conditional_arrays {
        code.push_str(&format!("    map.insert(\"{}\", vec![\n", array.tag_id));

        for condition in &array.conditions {
            code.push_str("        ConditionalEntry {\n");
            code.push_str(&format!(
                "            condition: \"{}\",\n",
                escape_string(&condition.condition)
            ));
            code.push_str(&format!(
                "            name: \"{}\",\n",
                escape_string(&condition.name)
            ));
            code.push_str(&format!(
                "            subdirectory: {},\n",
                condition.subdirectory
            ));
            code.push_str(&format!("            writable: {},\n", condition.writable));
            code.push_str(&format!(
                "            format: {},\n",
                if let Some(format) = &condition.format {
                    format!("Some(\"{}\")", escape_string(format))
                } else {
                    "None".to_string()
                }
            ));
            code.push_str("        },\n");
        }

        code.push_str("    ]);\n");
    }

    code.push_str("    map\n");
    code.push_str("});\n");

    Ok(code)
}

/// Generate count condition resolver
fn generate_count_condition_resolver(data: &ConditionalTagsExtraction) -> Result<String> {
    let mut code = String::new();

    code.push_str("/// Count-based condition resolution mapping\n");
    code.push_str("static COUNT_CONDITIONS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");

    // Group count conditions by tag_id
    let mut grouped_conditions: HashMap<String, Vec<&ConditionalEntry>> = HashMap::new();
    for condition in &data.conditional_data.count_conditions {
        grouped_conditions
            .entry(condition.tag_id.clone())
            .or_default()
            .push(condition);
    }

    for (tag_id, conditions) in grouped_conditions {
        code.push_str(&format!("    map.insert(\"{}\", vec![\n", tag_id));

        for condition in conditions {
            code.push_str("        ConditionalEntry {\n");
            code.push_str(&format!(
                "            condition: \"{}\",\n",
                escape_string(&condition.condition)
            ));
            code.push_str(&format!(
                "            name: \"{}\",\n",
                escape_string(&condition.name)
            ));
            code.push_str(&format!(
                "            subdirectory: {},\n",
                condition.subdirectory
            ));
            code.push_str(&format!("            writable: {},\n", condition.writable));
            code.push_str(&format!(
                "            format: {},\n",
                if let Some(format) = &condition.format {
                    format!("Some(\"{}\")", escape_string(format))
                } else {
                    "None".to_string()
                }
            ));
            code.push_str("        },\n");
        }

        code.push_str("    ]);\n");
    }

    code.push_str("    map\n");
    code.push_str("});\n");

    Ok(code)
}

/// Generate binary pattern resolver
fn generate_binary_pattern_resolver(data: &ConditionalTagsExtraction) -> Result<String> {
    let mut code = String::new();

    code.push_str("/// Binary pattern condition resolution mapping\n");
    code.push_str("static BINARY_PATTERNS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(|| {\n");
    code.push_str("    let mut map = HashMap::new();\n");

    // Group binary patterns by tag_id
    let mut grouped_patterns: HashMap<String, Vec<&ConditionalEntry>> = HashMap::new();
    for pattern in &data.conditional_data.binary_patterns {
        grouped_patterns
            .entry(pattern.tag_id.clone())
            .or_default()
            .push(pattern);
    }

    for (tag_id, patterns) in grouped_patterns {
        code.push_str(&format!("    map.insert(\"{}\", vec![\n", tag_id));

        for pattern in patterns {
            code.push_str("        ConditionalEntry {\n");
            code.push_str(&format!(
                "            condition: \"{}\",\n",
                escape_string(&pattern.condition)
            ));
            code.push_str(&format!(
                "            name: \"{}\",\n",
                escape_string(&pattern.name)
            ));
            code.push_str(&format!(
                "            subdirectory: {},\n",
                pattern.subdirectory
            ));
            code.push_str(&format!("            writable: {},\n", pattern.writable));
            code.push_str(&format!(
                "            format: {},\n",
                if let Some(format) = &pattern.format {
                    format!("Some(\"{}\")", escape_string(format))
                } else {
                    "None".to_string()
                }
            ));
            code.push_str("        },\n");
        }

        code.push_str("    ]);\n");
    }

    code.push_str("    map\n");
    code.push_str("});\n");

    Ok(code)
}
