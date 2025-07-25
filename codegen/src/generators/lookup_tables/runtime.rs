//! Runtime table generation for ProcessBinaryData tables
//!
//! This module generates runtime HashMap creation functions for ProcessBinaryData tables
//! that require model-conditional logic, variable format specifications, and complex PrintConv expressions.

use anyhow::Result;
use serde_json::Value;
use crate::schemas::input::{RuntimeTablesData, ExtractedRuntimeTable, RuntimeTagDefinition, PrintConvSpec, ConditionSpec, FormatSpec};

/// Generate runtime table creation function
pub fn generate_runtime_table(
    runtime_data: &RuntimeTablesData,
    table_config: &Value,
) -> Result<String> {
    let table_name = table_config["table_name"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing table_name in config"))?;
    
    let function_name = table_config["function_name"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing function_name in config"))?;
    
    // Get the first (and typically only) runtime table from the data
    let runtime_table = runtime_data.tables.values().next()
        .ok_or_else(|| anyhow::anyhow!("No runtime table data found"))?;
    
    let mut code = String::new();
    
    // Generate file header
    generate_file_header(&mut code, &runtime_data.source, table_name)?;
    
    // Generate imports
    generate_imports(&mut code, runtime_table)?;
    
    // Generate helper functions for complex conversions
    generate_helper_functions(&mut code, runtime_table)?;
    
    // Generate main runtime table creation function
    generate_table_creation_function(&mut code, function_name, runtime_table, table_config)?;
    
    // Generate supporting types and enums if needed
    generate_supporting_types(&mut code, runtime_table)?;
    
    Ok(code)
}

/// Generate file header with documentation and source reference
/// NOTE: Do NOT add extraction timestamps - they create spurious git diffs
/// that make it impossible to track real changes to generated code
fn generate_file_header(code: &mut String, source: &crate::schemas::input::TableSource, table_name: &str) -> Result<()> {
    code.push_str(&format!(r#"//! Runtime table generation for {}
//! 
//! This file is auto-generated from ExifTool source: {}
//! 
//! DO NOT EDIT: This file is automatically generated by the codegen system.
//! Any changes will be overwritten on the next codegen run.

"#, table_name, source.module));
    
    Ok(())
}

/// Generate necessary imports
fn generate_imports(code: &mut String, runtime_table: &ExtractedRuntimeTable) -> Result<()> {
    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n");
    
    // Check if we need model detection
    if runtime_table.metadata.has_model_conditions {
        code.push_str("use crate::core::model_detection::ModelInfo;\n");
    }
    
    // Check if we need binary data types
    code.push_str("use crate::core::binary_data::{BinaryDataTag, BinaryDataFormat};\n");
    
    // Check if we need value conversion types  
    code.push_str("use crate::core::values::TagValue;\n");
    
    code.push('\n');
    Ok(())
}

/// Generate helper functions for complex conversions
fn generate_helper_functions(code: &mut String, runtime_table: &ExtractedRuntimeTable) -> Result<()> {
    // Generate PrintConv helper functions
    for tag_def in runtime_table.tag_definitions.values() {
        if let Some(print_conv) = &tag_def.print_conv {
            generate_print_conv_function(code, tag_def, print_conv)?;
        }
    }
    
    Ok(())
}

/// Generate PrintConv function for a specific tag
fn generate_print_conv_function(
    code: &mut String, 
    tag_def: &RuntimeTagDefinition,
    print_conv: &PrintConvSpec
) -> Result<()> {
    let function_name = format!("print_conv_{}", 
        tag_def.name.to_lowercase().replace(' ', "_"));
    
    match print_conv.conversion_type {
        crate::schemas::input::PrintConvType::SimpleHash => {
            // Generate simple hash lookup function
            if let Value::Object(hash_map) = &print_conv.data {
                code.push_str(&format!(r#"
/// PrintConv for {} (simple hash lookup)
fn {}(value: &TagValue) -> String {{
    match value {{
"#, tag_def.name, function_name));
                
                for (key, val) in hash_map {
                    if let Value::String(val_str) = val {
                        if let Ok(key_num) = key.parse::<u32>() {
                            code.push_str(&format!(r#"        TagValue::U32({key_num}) => "{val_str}".to_string(),
"#));
                        } else {
                            code.push_str(&format!(r#"        TagValue::String(s) if s == "{key}" => "{val_str}".to_string(),
"#));
                        }
                    }
                }
                
                code.push_str(r#"        _ => format!("Unknown ({})", value),
    }
}
"#);
            }
        },
        crate::schemas::input::PrintConvType::PerlExpression => {
            // Generate stub for Perl expressions (to be expanded later)
            code.push_str(&format!(r#"
/// PrintConv for {} (Perl expression - simplified)
fn {}(value: &TagValue) -> String {{
    // TODO: Implement Perl expression evaluation
    // Original expression: {}
    format!("Unknown ({{}})", value)
}}
"#, tag_def.name, function_name, 
    print_conv.data.as_str().unwrap_or("UNKNOWN")));
        },
        _ => {
            // Generate generic fallback
            code.push_str(&format!(r#"
/// PrintConv for {} (complex - fallback)
fn {}(value: &TagValue) -> String {{
    format!("Unknown ({{}})", value)
}}
"#, tag_def.name, function_name));
        }
    }
    
    Ok(())
}

/// Generate the main table creation function
fn generate_table_creation_function(
    code: &mut String,
    function_name: &str,
    runtime_table: &ExtractedRuntimeTable,
    table_config: &Value,
) -> Result<()> {
    let description = table_config["description"].as_str()
        .unwrap_or("Runtime-generated ProcessBinaryData table");
    
    code.push_str(&format!(r#"
/// {description}
/// 
/// Creates a runtime HashMap for ProcessBinaryData processing.
/// This table is generated based on ExifTool's ProcessBinaryData structure
/// and includes model-conditional logic and format specifications.
pub fn {function_name}() -> HashMap<u32, BinaryDataTag> {{
    let mut table = HashMap::new();
    
"#));
    
    // Generate table entries
    for (offset, tag_def) in &runtime_table.tag_definitions {
        generate_table_entry(code, offset, tag_def, runtime_table)?;
    }
    
    code.push_str(r#"    table
}
"#);
    
    Ok(())
}

/// Generate a single table entry
fn generate_table_entry(
    code: &mut String,
    offset: &str,
    tag_def: &RuntimeTagDefinition,
    _runtime_table: &ExtractedRuntimeTable,
) -> Result<()> {
    // Parse offset (could be decimal, hex, or fractional)
    let offset_value = if let Some(stripped) = offset.strip_prefix("0x") {
        u32::from_str_radix(stripped, 16)
            .unwrap_or(0)
    } else if let Ok(val) = offset.parse::<f64>() {
        val as u32
    } else {
        0
    };
    
    code.push_str(&format!(r#"    // Tag: {} at offset {}
"#, tag_def.name, offset));
    
    // Add conditional logic if present
    if let Some(condition) = &tag_def.condition {
        generate_condition_check(code, condition)?;
        code.push_str("    {\n        ");
    } else {
        code.push_str("    ");
    }
    
    // Generate the table entry
    code.push_str(&format!(r#"table.insert({}, BinaryDataTag {{
        name: "{}".to_string(),
        format: {},
        print_conv: {},
        notes: {},
    }});
"#, 
        offset_value,
        tag_def.name,
        generate_format_spec(&tag_def.format)?,
        generate_print_conv_ref(tag_def)?,
        generate_notes_field(&tag_def.notes)?
    ));
    
    // Close conditional block if present
    if tag_def.condition.is_some() {
        code.push_str("    }\n");
    }
    
    code.push('\n');
    Ok(())
}

/// Generate condition check (simplified for initial implementation)
fn generate_condition_check(code: &mut String, condition: &ConditionSpec) -> Result<()> {
    match condition.condition_type {
        crate::schemas::input::ConditionType::ModelRegex => {
            // For now, generate a comment - full model checking requires more infrastructure
            code.push_str(&format!(r#"    // Condition: {}
    if true // TODO: Implement model regex checking
"#, condition.expression));
        },
        _ => {
            code.push_str(&format!(r#"    // Condition: {}
    if true // TODO: Implement condition checking
"#, condition.expression));
        }
    }
    Ok(())
}

/// Generate format specification
fn generate_format_spec(format: &Option<FormatSpec>) -> Result<String> {
    if let Some(format_spec) = format {
        let base_format = match format_spec.base_type.as_str() {
            "int8u" => "BinaryDataFormat::UInt8",
            "int8s" => "BinaryDataFormat::Int8", 
            "int16u" => "BinaryDataFormat::UInt16",
            "int16s" => "BinaryDataFormat::Int16",
            "int32u" => "BinaryDataFormat::UInt32",
            "int32s" => "BinaryDataFormat::Int32",
            "string" => "BinaryDataFormat::String",
            _ => "BinaryDataFormat::UInt8", // Default fallback
        };
        
        if format_spec.is_variable {
            Ok(format!("{}  // Variable: {}", base_format, 
                format_spec.array_size.as_ref().unwrap_or(&"unknown".to_string())))
        } else {
            Ok(base_format.to_string())
        }
    } else {
        Ok("BinaryDataFormat::UInt8".to_string()) // Default format
    }
}

/// Generate PrintConv reference
fn generate_print_conv_ref(tag_def: &RuntimeTagDefinition) -> Result<String> {
    if tag_def.print_conv.is_some() {
        let function_name = format!("print_conv_{}", 
            tag_def.name.to_lowercase().replace(' ', "_"));
        Ok(format!("Some({function_name})"))
    } else {
        Ok("None".to_string())
    }
}

/// Generate notes field
fn generate_notes_field(notes: &Option<String>) -> Result<String> {
    if let Some(notes_text) = notes {
        Ok(format!(r#"Some("{}".to_string())"#, notes_text.replace('"', r#"\""#)))
    } else {
        Ok("None".to_string())
    }
}

/// Generate supporting types and enums if needed
#[allow(clippy::ptr_arg)]
fn generate_supporting_types(_code: &mut String, _runtime_table: &ExtractedRuntimeTable) -> Result<()> {
    // For now, we use existing types from the core module
    // Future enhancement: Generate custom enums for specific tag values
    Ok(())
}