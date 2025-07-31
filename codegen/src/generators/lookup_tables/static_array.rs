//! Static array generator for direct array access

use anyhow::Result;
use crate::common::escape_string;

/// JSON structure from simple_array.pl extraction
#[derive(Debug, serde::Deserialize)]
pub struct ExtractedArray {
    pub source: SourceInfo,
    pub metadata: MetadataInfo,
    pub elements: Vec<ArrayElement>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SourceInfo {
    pub module: String,
    pub array_expr: String,
    pub extracted_at: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MetadataInfo {
    pub element_count: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ArrayElement {
    pub index: usize,
    pub value: serde_json::Value,
}

/// Generate code for a static array with direct access functions
pub fn generate_static_array(
    extracted_array: &ExtractedArray,
    constant_name: &str,
    element_type: &str,
    expected_size: Option<usize>,
) -> Result<String> {
    let mut code = String::new();
    
    // Validate array size if specified
    if let Some(expected) = expected_size {
        if extracted_array.elements.len() != expected {
            return Err(anyhow::anyhow!(
                "Array size mismatch: expected {}, got {} elements in {}",
                expected,
                extracted_array.elements.len(),
                extracted_array.source.array_expr
            ));
        }
    }
    
    let array_size = extracted_array.elements.len();
    
    // Sort elements by index to ensure correct order
    let mut sorted_elements = extracted_array.elements.clone();
    sorted_elements.sort_by_key(|e| e.index);
    
    // Generate array documentation
    code.push_str(&format!(
        "/// Static array extracted from ExifTool {}\n",
        extracted_array.source.module
    ));
    code.push_str(&format!(
        "/// Source: {} ({})\n",
        extracted_array.source.array_expr,
        extracted_array.source.module
    ));
    code.push_str(&format!(
        "/// Size: {} elements\n",
        array_size
    ));
    
    // Generate the static array declaration
    code.push_str(&format!(
        "pub static {}: [{}; {}] = [\n",
        constant_name,
        element_type,
        array_size
    ));
    
    // Add array elements
    for element in &sorted_elements {
        let formatted_value = format_element_value(&element.value, element_type)?;
        code.push_str(&format!("    {},\n", formatted_value));
    }
    
    code.push_str("];\n");
    
    Ok(code)
}

/// Format a JSON value according to the specified Rust type
fn format_element_value(value: &serde_json::Value, element_type: &str) -> Result<String> {
    match element_type {
        "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => {
            match value {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(format!("{}", i))
                    } else if let Some(u) = n.as_u64() {
                        Ok(format!("{}", u))
                    } else {
                        Err(anyhow::anyhow!("Invalid integer value: {}", value))
                    }
                }
                serde_json::Value::String(s) => {
                    // Try to parse string as number (hex values like "0xff")
                    if s.starts_with("0x") || s.starts_with("0X") {
                        let hex_part = &s[2..];
                        match i64::from_str_radix(hex_part, 16) {
                            Ok(n) => Ok(format!("{}", n)),
                            Err(_) => Err(anyhow::anyhow!("Invalid hex value: {}", s))
                        }
                    } else {
                        match s.parse::<i64>() {
                            Ok(n) => Ok(format!("{}", n)),
                            Err(_) => Err(anyhow::anyhow!("Cannot parse '{}' as {}", s, element_type))
                        }
                    }
                }
                _ => Err(anyhow::anyhow!("Expected number for type {}, got: {}", element_type, value))
            }
        }
        "f32" | "f64" => {
            match value {
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        Ok(format!("{}", f))
                    } else {
                        Err(anyhow::anyhow!("Invalid float value: {}", value))
                    }
                }
                _ => Err(anyhow::anyhow!("Expected number for type {}, got: {}", element_type, value))
            }
        }
        "&'static str" => {
            match value {
                serde_json::Value::String(s) => {
                    Ok(format!("\"{}\"", escape_string(s)))
                }
                _ => Err(anyhow::anyhow!("Expected string for type {}, got: {}", element_type, value))
            }
        }
        _ => Err(anyhow::anyhow!("Unsupported element type: {}", element_type))
    }
}