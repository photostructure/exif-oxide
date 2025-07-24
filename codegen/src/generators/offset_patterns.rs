//! Sony offset pattern generator
//!
//! This generator creates Rust code from extracted offset calculation patterns,
//! providing model-specific offset calculations for Sony RAW files.

use crate::common::escape_string;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct OffsetPatternExtraction {
    pub manufacturer: String,
    pub source: SourceInfo,
    pub offset_patterns: OffsetPatternsData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub module: String,
    pub function_pattern: String,
    pub extracted_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OffsetPatternsData {
    pub model_conditions: Vec<ModelCondition>,
    pub offset_calculations: Vec<OffsetCalculation>,
    pub base_types: Vec<BaseType>,
    pub idc_patterns: Vec<IDCPattern>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub operator: String,
    pub pattern: String,
    pub raw_pattern: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OffsetCalculation {
    pub raw_expression: String,
    pub operation: String,
    pub base_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_variable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_variable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_variable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseType {
    pub name: String,
    pub pattern: String,
    pub usage_count: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IDCPattern {
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub description: String,
    pub pattern: String,
    pub recovery: String,
}

/// Generate Rust code for offset patterns
#[allow(dead_code)]
pub fn generate_offset_patterns(data: &OffsetPatternExtraction) -> Result<String> {
    let mut code = String::new();
    
    // Add header comment
    code.push_str(&format!(
        "//! {} offset calculation patterns\n",
        data.manufacturer
    ));
    code.push_str(&format!(
        "//! ExifTool: {} extracted with pattern: {}\n\n",
        data.source.module, data.source.function_pattern
    ));
    
    // Add imports
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    code.push_str("use crate::exif::ExifReader;\n");
    code.push_str("use crate::error::Result;\n");
    code.push_str("use crate::expressions::ExpressionEvaluator;\n\n");
    
    // Generate model conditions if present
    if !data.offset_patterns.model_conditions.is_empty() {
        code.push_str(&generate_model_conditions(&data.offset_patterns.model_conditions)?);
        code.push('\n');
    }
    
    // Generate offset calculation functions
    if !data.offset_patterns.offset_calculations.is_empty() {
        code.push_str(&generate_offset_calculators(&data.offset_patterns.offset_calculations)?);
        code.push('\n');
    }
    
    // Generate IDC recovery patterns if present
    if !data.offset_patterns.idc_patterns.is_empty() {
        code.push_str(&generate_idc_recovery(&data.offset_patterns.idc_patterns)?);
    }
    
    Ok(code)
}

#[allow(dead_code)]
fn generate_model_conditions(conditions: &[ModelCondition]) -> Result<String> {
    let mut code = String::new();
    
    code.push_str("/// Model condition patterns from Sony.pm\n");
    code.push_str("pub static SONY_MODEL_CONDITIONS: LazyLock<Vec<(String, String, String)>> = LazyLock::new(|| {\n");
    code.push_str("    vec![\n");
    
    for condition in conditions {
        if condition.condition_type == "regex" {
            code.push_str(&format!(
                "        (\"{}\".to_string(), \"{}\".to_string(), \"{}\".to_string()),\n",
                escape_string(&condition.operator),
                escape_string(&condition.pattern),
                escape_string(&condition.condition_type)
            ));
        }
    }
    
    code.push_str("    ]\n");
    code.push_str("});\n\n");
    
    code.push_str("/// Check if a model matches any Sony-specific conditions\n");
    code.push_str("pub fn matches_sony_model(model: &str, evaluator: &ExpressionEvaluator) -> bool {\n");
    code.push_str("    for (operator, pattern, _) in SONY_MODEL_CONDITIONS.iter() {\n");
    code.push_str("        if evaluator.matches_pattern(model, pattern, operator) {\n");
    code.push_str("            return true;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    false\n");
    code.push_str("}\n");
    
    Ok(code)
}

#[allow(dead_code)]
fn generate_offset_calculators(calculations: &[OffsetCalculation]) -> Result<String> {
    let mut code = String::new();
    
    code.push_str("/// Offset calculation patterns extracted from Sony.pm\n");
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str("pub enum SonyOffsetCalculation {\n");
    
    // Generate enum variants for each operation type
    let mut operation_types = HashMap::new();
    for calc in calculations {
        if calc.operation != "complex_expression" {
            operation_types.insert(calc.operation.clone(), calc);
        }
    }
    
    for (op_type, example) in operation_types {
        match op_type.as_str() {
            "get32u_valpt" | "get16u_valpt" | "get8u_valpt" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str(&format!("    {}(u32), // offset\n", to_camel_case(&op_type)));
            }
            "get32u_entry" | "get16u_entry" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str(&format!("    {}(u32), // entry offset\n", to_camel_case(&op_type)));
            }
            "get32u_variable" | "get16u_variable" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str(&format!("    {}(String), // variable name\n", to_camel_case(&op_type)));
            }
            "variable_plus_constant" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str("    VariablePlusConstant { variable: String, constant: u32 },\n");
            }
            "array_offset" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str("    ArrayOffset { base: String, base_offset: u32, index: String, element_size: u32 },\n");
            }
            "entry_hash_offset" => {
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str("    EntryHashOffset { tag_id: u16, offset: u32 },\n");
            }
            _ => {
                // Handle other types generically
                code.push_str(&format!("    /// {} - e.g., {}\n", op_type, example.raw_expression));
                code.push_str(&format!("    {}(String), // expression\n", to_camel_case(&op_type)));
            }
        }
    }
    
    code.push_str("}\n\n");
    
    // Generate calculation function
    code.push_str("/// Calculate offset based on Sony-specific patterns\n");
    code.push_str("pub fn calculate_sony_offset(\n");
    code.push_str("    reader: &ExifReader,\n");
    code.push_str("    calculation: &SonyOffsetCalculation,\n");
    code.push_str("    context: &HashMap<String, u64>,\n");
    code.push_str(") -> Result<u64> {\n");
    code.push_str("    use SonyOffsetCalculation::*;\n");
    code.push_str("    \n");
    code.push_str("    match calculation {\n");
    code.push_str("        Get32uValpt(offset) => {\n");
    code.push_str("            // Read 32-bit value from value pointer + offset\n");
    code.push_str("            // ExifTool: Get32u($valPt, offset)\n");
    code.push_str("            reader.get_u32_at(*offset as u64)\n");
    code.push_str("        }\n");
    code.push_str("        Get16uValpt(offset) => {\n");
    code.push_str("            // Read 16-bit value from value pointer + offset\n");
    code.push_str("            reader.get_u16_at(*offset as u64)\n");
    code.push_str("        }\n");
    code.push_str("        Get16uEntry(entry_offset) => {\n");
    code.push_str("            // Read 16-bit value from entry + offset\n");
    code.push_str("            // ExifTool: Get16u($dataPt, $entry + offset)\n");
    code.push_str("            if let Some(entry_base) = context.get(\"entry\") {\n");
    code.push_str("                reader.get_u16_at(entry_base + *entry_offset as u64)\n");
    code.push_str("            } else {\n");
    code.push_str("                Err(\"Missing entry base in context\".into())\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("        VariablePlusConstant { variable, constant } => {\n");
    code.push_str("            // Variable + constant offset\n");
    code.push_str("            if let Some(base) = context.get(variable) {\n");
    code.push_str("                Ok(base + *constant as u64)\n");
    code.push_str("            } else {\n");
    code.push_str("                Err(format!(\"Missing variable {} in context\", variable).into())\n");
    code.push_str("            }\n");
    code.push_str("        }\n");
    code.push_str("        _ => {\n");
    code.push_str("            // TODO: Implement other calculation types\n");
    code.push_str("            Err(\"Unsupported offset calculation type\".into())\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(code)
}

#[allow(dead_code)]
fn generate_idc_recovery(patterns: &[IDCPattern]) -> Result<String> {
    let mut code = String::new();
    
    code.push_str("/// Sony IDC corruption recovery patterns\n");
    code.push_str("pub fn recover_idc_offset(tag: u16, offset: u64, model: &str) -> u64 {\n");
    code.push_str("    // ExifTool: Sony.pm SetARW() IDC handling\n");
    code.push_str("    match tag {\n");
    
    for pattern in patterns {
        if pattern.pattern_type == "A100_IDC" {
            code.push_str("        0x014a if model.contains(\"A100\") => {\n");
            code.push_str("            // A100 IDC corruption fix\n");
            code.push_str("            if offset < 0x10000 {\n");
            code.push_str("                0x2000  // Reset to known good value\n");
            code.push_str("            } else {\n");
            code.push_str("                offset\n");
            code.push_str("            }\n");
            code.push_str("        }\n");
        }
    }
    
    code.push_str("        0x7200 => offset.saturating_sub(0x10),  // Encryption key offset\n");
    code.push_str("        0x7201 => offset + 0x2000,  // Lens info offset\n");
    code.push_str("        _ => offset,  // No adjustment needed\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    
    Ok(code)
}

#[allow(dead_code)]
fn to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}