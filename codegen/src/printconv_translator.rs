//! PrintConv Expression Translator
//!
//! This module translates ExifTool Perl PrintConv expressions into Rust code.
//! It handles common patterns found in PrintConv expressions across different
//! camera manufacturers, with special focus on Canon CameraSettings.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

/// PrintConv expression translator for converting Perl expressions to Rust code
pub struct PrintConvTranslator {
    /// Cache for compiled regex patterns
    regex_cache: HashMap<String, Regex>,
}

impl PrintConvTranslator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Translate a Perl PrintConv expression to Rust code
    ///
    /// # Arguments
    /// * `perl_expr` - The Perl expression string from ExifTool
    /// * `tag_name` - The name of the tag (for specialized handling)
    /// * `module_name` - The ExifTool module name (e.g., "Canon_pm")
    ///
    /// # Returns
    /// Rust function code that implements the same logic as the Perl expression
    pub fn translate_expression(
        &mut self,
        perl_expr: &str,
        tag_name: &str,
        module_name: &str,
    ) -> Result<String> {
        debug!(
            "Translating PrintConv expression for {}.{}: {}",
            module_name, tag_name, perl_expr
        );

        // Clean up the Perl expression
        let cleaned_expr = self.clean_perl_expression(perl_expr);

        // Handle specific known patterns
        match (module_name, tag_name) {
            ("Canon_pm", "SelfTimer") => self.translate_canon_selftimer(&cleaned_expr),
            ("Canon_pm", "Quality") => self.translate_canon_quality(&cleaned_expr),
            _ => self.translate_generic_expression(&cleaned_expr, tag_name),
        }
    }

    /// Clean up Perl expression by removing common formatting artifacts
    fn clean_perl_expression(&self, expr: &str) -> String {
        expr.trim()
            .replace("\n", " ")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Translate Canon SelfTimer PrintConv expression
    ///
    /// Original Perl:
    /// ```perl
    /// return 'Off' unless $val;
    /// return (($val&0xfff) / 10) . ' s' . ($val & 0x4000 ? ', Custom' : '');
    /// ```
    fn translate_canon_selftimer(&mut self, _perl_expr: &str) -> Result<String> {
        Ok("
/// Canon SelfTimer PrintConv implementation
/// Based on Canon.pm:2182-2184
pub fn canon_selftimer_printconv(val: i16) -> String {
    // return 'Off' unless $val;
    if val == 0 {
        return \"Off\".to_string();
    }
    
    // return (($val&0xfff) / 10) . ' s' . ($val & 0x4000 ? ', Custom' : '');
    let seconds = ((val & 0xfff) as f32) / 10.0;
    let custom = if val & 0x4000 != 0 { \", Custom\" } else { \"\" };
    format!(\"{} s{}\", seconds, custom)
}
".to_string())
    }

    /// Translate Canon Quality PrintConv expression
    ///
    /// Original Perl: `PrintConv => \%canonQuality,`
    /// This is a reference to a hash table that needs to be resolved
    fn translate_canon_quality(&mut self, _perl_expr: &str) -> Result<String> {
        // For now, we'll handle this as a fallback case since it references
        // an external hash that would need to be extracted separately
        Err(anyhow!(
            "Canon Quality PrintConv references external hash %canonQuality - needs simple table extraction"
        ))
    }

    /// Translate generic Perl expressions using pattern matching
    fn translate_generic_expression(&mut self, perl_expr: &str, tag_name: &str) -> Result<String> {
        // Handle Perl substitution patterns (e.g., $val=~s/\s+/, /g; $val)
        if perl_expr.contains("=~s/") || perl_expr.contains("=~ s/") {
            return Err(anyhow!(
                "Perl substitution patterns not yet supported for {}: {}",
                tag_name,
                perl_expr
            ));
        }
        
        // Handle Perl unpack function - too complex to translate
        if perl_expr.contains("unpack(") {
            return Err(anyhow!(
                "Perl unpack() function not yet supported for {}: {}",
                tag_name,
                perl_expr
            ));
        }

        // Handle simple conditional returns
        if let Some(rust_code) = self.try_translate_simple_conditional(perl_expr, tag_name)? {
            return Ok(rust_code);
        }

        // Handle mathematical expressions
        if let Some(rust_code) = self.try_translate_mathematical(perl_expr, tag_name)? {
            return Ok(rust_code);
        }

        // Handle string formatting
        if let Some(rust_code) = self.try_translate_string_formatting(perl_expr, tag_name)? {
            return Ok(rust_code);
        }

        // If no pattern matches, return an error
        Err(anyhow!(
            "Unable to translate complex Perl expression for {}: {}",
            tag_name,
            perl_expr
        ))
    }

    /// Try to translate simple conditional expressions like "return 'value' unless $val;"
    fn try_translate_simple_conditional(&mut self, perl_expr: &str, tag_name: &str) -> Result<Option<String>> {
        let simple_return_regex = self.get_or_compile_regex(r"return\s+'([^']+)'\s+unless\s+\$val;")?;
        
        if let Some(captures) = simple_return_regex.captures(perl_expr) {
            let return_value = &captures[1];
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: i16) -> String {{
    if val == 0 {{
        \"{}\".to_string()
    }} else {{
        format!(\"Unknown ({{}})\", val)
    }}
}}
", tag_name, function_name, return_value)));
        }

        Ok(None)
    }

    /// Try to translate mathematical expressions
    fn try_translate_mathematical(&mut self, perl_expr: &str, tag_name: &str) -> Result<Option<String>> {
        // Handle sprintf formatting expressions
        if let Some(captures) = self.get_or_compile_regex(r#"sprintf\s*\(\s*"([^"]+)"\s*,\s*\$val\s*\)"#)?.captures(perl_expr) {
            let format_str = &captures[1];
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            // Convert Perl sprintf format to Rust format
            let rust_format = match format_str {
                "%.2g" => "{:.2}",
                "%.0f" => "{:.0}",
                "%.2f" => "{:.2}",
                "%.1fx" => "{:.1}x",
                "%.6u" => "{:06}",
                _ => return Ok(None), // Unsupported format
            };
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: f64) -> String {{
    format!(\"{}\", val)
}}
", tag_name, function_name, rust_format)));
        }

        // Handle simple division expressions: "$val / N"
        if let Some(captures) = self.get_or_compile_regex(r#"\$val\s*/\s*(\d+)"#)?.captures(perl_expr) {
            let divisor = &captures[1];
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: f64) -> String {{
    let result = val / {}.0;
    format!(\"{{}}\", result)
}}
", tag_name, function_name, divisor)));
        }

        Ok(None)
    }

    /// Try to translate string formatting expressions
    fn try_translate_string_formatting(&mut self, perl_expr: &str, tag_name: &str) -> Result<Option<String>> {
        // Handle simple string concatenation: "$val mm", "$val C", etc.
        if let Some(captures) = self.get_or_compile_regex(r#"^"?\$val\s*([^"]*)"?$"#)?.captures(perl_expr) {
            let suffix = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: i16) -> String {{
    format!(\"{{}}{}\", val)
}}
", tag_name, function_name, suffix)));
        }

        // Handle plus sign formatting: "$val > 0 ? \"+$val\" : $val"
        if perl_expr.contains(r#"$val > 0 ? "+$val" : $val"#) {
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: i16) -> String {{
    if val > 0 {{
        format!(\"+{{}}\", val)
    }} else {{
        format!(\"{{}}\", val)
    }}
}}
", tag_name, function_name)));
        }

        // Handle conditional with different string outputs
        if let Some(captures) = self.get_or_compile_regex(r#"\$val\s*==\s*(\d+)\s*\?\s*"([^"]+)"\s*:\s*\$val"#)?.captures(perl_expr) {
            let test_value = &captures[1];
            let special_string = &captures[2];
            let function_name = format!("{}_printconv", self.sanitize_function_name(tag_name));
            
            // Determine the appropriate type based on the value
            let test_value_parsed: i64 = test_value.parse().unwrap_or(0);
            let val_type = if test_value_parsed > i16::MAX as i64 || test_value_parsed < i16::MIN as i64 {
                if test_value_parsed > i32::MAX as i64 || test_value_parsed < i32::MIN as i64 {
                    "i64"
                } else if test_value_parsed < 0 {
                    "i32"
                } else if test_value_parsed > u16::MAX as i64 {
                    "u32"
                } else {
                    "u16"
                }
            } else {
                "i16"
            };
            
            return Ok(Some(format!("
/// {} PrintConv implementation  
pub fn {}(val: {}) -> String {{
    if val == {} {{
        \"{}\".to_string()
    }} else {{
        format!(\"{{}}\", val)
    }}
}}
", tag_name, function_name, val_type, test_value, special_string)));
        }

        Ok(None)
    }

    /// Get or compile a regex pattern
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", pattern, e))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
    
    /// Sanitize a tag name to be a valid Rust function name
    fn sanitize_function_name(&self, tag_name: &str) -> String {
        tag_name.to_lowercase()
            .replace('-', "_")
            .replace(' ', "_")
            .replace(':', "_")
            .replace('.', "_")
    }
}

impl Default for PrintConvTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canon_selftimer_translation() {
        let mut translator = PrintConvTranslator::new();
        let perl_expr = r#"
            return 'Off' unless $val;
            return (($val&0xfff) / 10) . ' s' . ($val & 0x4000 ? ', Custom' : '');
        "#;

        let result = translator.translate_expression(perl_expr, "SelfTimer", "Canon_pm");
        assert!(result.is_ok());
        
        let rust_code = result.unwrap();
        assert!(rust_code.contains("canon_selftimer_printconv"));
        assert!(rust_code.contains("if val == 0"));
        assert!(rust_code.contains("return \"Off\""));
        assert!(rust_code.contains("val & 0xfff"));
        assert!(rust_code.contains("val & 0x4000"));
    }

    #[test]
    fn test_simple_conditional_translation() {
        let mut translator = PrintConvTranslator::new();
        let perl_expr = "return 'Disabled' unless $val;";

        let result = translator.try_translate_simple_conditional(perl_expr, "TestTag");
        assert!(result.is_ok());
        
        let rust_code = result.unwrap();
        assert!(rust_code.is_some());
        
        let code = rust_code.unwrap();
        assert!(code.contains("testtag_printconv"));
        assert!(code.contains("\"Disabled\""));
    }

    #[test]
    fn test_string_formatting_translation() {
        let mut translator = PrintConvTranslator::new();
        
        // Test simple string concatenation
        let result = translator.try_translate_string_formatting("\"$val mm\"", "FocalLength");
        assert!(result.is_ok());
        let code = result.unwrap().unwrap();
        assert!(code.contains("focallength_printconv"));
        assert!(code.contains("format!(\"{}mm\", val)"));
        
        // Test plus sign formatting
        let result = translator.try_translate_string_formatting("$val > 0 ? \"+$val\" : $val", "Sharpness");
        assert!(result.is_ok());
        let code = result.unwrap().unwrap();
        assert!(code.contains("sharpness_printconv"));
        assert!(code.contains("if val > 0"));
        assert!(code.contains("format!(\"+{}\", val)"));
    }

    #[test]
    fn test_mathematical_translation() {
        let mut translator = PrintConvTranslator::new();
        
        // Test sprintf formatting
        let result = translator.try_translate_mathematical("sprintf(\"%.2g\",$val)", "MaxAperture");
        assert!(result.is_ok());
        let code = result.unwrap().unwrap();
        assert!(code.contains("maxaperture_printconv"));
        assert!(code.contains("format!(\"{:.2}\", val)"));
        
        // Test division
        let result = translator.try_translate_mathematical("$val / 100", "Distance");
        assert!(result.is_ok());
        let code = result.unwrap().unwrap();
        assert!(code.contains("distance_printconv"));
        assert!(code.contains("val / 100"));
    }
}