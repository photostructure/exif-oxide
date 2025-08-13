//! PPI Function Registry for Deduplication
//!
//! This registry provides centralized management of PPI-generated functions to eliminate
//! duplicates across modules. Functions are deduplicated based on AST structure hash,
//! ensuring that semantically equivalent expressions share the same implementation.

use anyhow::Result;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::ppi::{ExpressionType, PpiNode, RustGenerator};
use crate::strategies::GeneratedFile;

/// Specification for a deduplicated function
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    /// Function name (e.g., "ast_value_a1b2c3d4")
    pub function_name: String,
    /// Import path for use in tag modules (e.g., "crate::generated::fn::a1")
    pub module_path: String,
    /// Two-character hash prefix for file organization
    pub hash_prefix: String,
    /// Type of expression (ValueConv, PrintConv, Condition)
    pub expression_type: ExpressionType,
    /// Original Perl expression for documentation
    pub original_expression: String,
}

/// PPI function registry for AST-based deduplication
#[derive(Debug)]
pub struct PpiFunctionRegistry {
    /// Map from AST hash to function specification
    ast_to_function: HashMap<String, FunctionSpec>,
    /// Map from AST hash to generated Rust function code
    generated_functions: HashMap<String, String>,
}

impl Default for PpiFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PpiFunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            ast_to_function: HashMap::new(),
            generated_functions: HashMap::new(),
        }
    }

    /// Register an AST and get back a function specification
    ///
    /// If the AST has been seen before, returns the existing function spec.
    /// Otherwise, generates a new function and returns its spec.
    pub fn register_ast(
        &mut self,
        ppi_ast: &PpiNode,
        expression_type: ExpressionType,
        original_expression: &str,
    ) -> Result<FunctionSpec> {
        // Generate hash from AST structure (not expression text)
        let ast_hash = self.hash_ast_structure(ppi_ast)?;

        // Check if we already have this AST registered
        if let Some(existing_spec) = self.ast_to_function.get(&ast_hash) {
            return Ok(existing_spec.clone());
        }

        // Generate new function for this AST
        let function_spec =
            self.create_function_spec(&ast_hash, expression_type, original_expression);
        let function_code = self.generate_function_code(ppi_ast, &function_spec)?;

        // Store both the spec and the generated code
        self.ast_to_function
            .insert(ast_hash.clone(), function_spec.clone());
        self.generated_functions.insert(ast_hash, function_code);

        Ok(function_spec)
    }

    /// Generate all function files after all modules have been processed
    pub fn generate_function_files(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        // Group functions by their two-character hash prefix
        let mut functions_by_prefix: HashMap<String, Vec<String>> = HashMap::new();

        for (ast_hash, function_code) in &self.generated_functions {
            let prefix = ast_hash.chars().take(2).collect::<String>();
            functions_by_prefix
                .entry(prefix)
                .or_default()
                .push(function_code.clone());
        }

        // Generate one file per prefix
        for (prefix, function_codes) in &functions_by_prefix {
            let file_content = self.generate_function_file_content(prefix, function_codes);
            let file_path = format!("functions/hash_{}.rs", prefix);

            files.push(GeneratedFile {
                path: file_path,
                content: file_content,
                strategy: "PpiFunctionRegistry".to_string(),
            });
        }

        // Generate the main mod.rs file
        let mod_file = self.generate_mod_file(&functions_by_prefix)?;
        files.push(mod_file);

        Ok(files)
    }

    /// Hash the AST structure for deduplication
    fn hash_ast_structure(&self, ppi_ast: &PpiNode) -> Result<String> {
        // Serialize AST to JSON and hash the structure
        let ast_json = serde_json::to_string(ppi_ast)
            .map_err(|e| anyhow::anyhow!("Failed to serialize AST: {}", e))?;

        let mut hasher = DefaultHasher::new();
        ast_json.hash(&mut hasher);

        Ok(format!("{:016x}", hasher.finish()))
    }

    /// Create function specification for a new AST
    fn create_function_spec(
        &self,
        ast_hash: &str,
        expression_type: ExpressionType,
        original_expression: &str,
    ) -> FunctionSpec {
        let hash_prefix = ast_hash.chars().take(2).collect::<String>();
        let function_name = format!(
            "ast_{}_{}",
            match expression_type {
                ExpressionType::ValueConv => "value",
                ExpressionType::PrintConv => "print",
                ExpressionType::Condition => "condition",
            },
            ast_hash.chars().take(8).collect::<String>()
        );
        let module_path = format!("crate::generated::functions::hash_{}", hash_prefix);

        FunctionSpec {
            function_name,
            module_path,
            hash_prefix,
            expression_type,
            original_expression: original_expression.to_string(),
        }
    }

    /// Generate Rust function code from AST
    fn generate_function_code(
        &self,
        ppi_ast: &PpiNode,
        function_spec: &FunctionSpec,
    ) -> Result<String> {
        let generator = RustGenerator::new(
            function_spec.expression_type,
            function_spec.function_name.clone(),
            function_spec.original_expression.clone(),
        );

        generator
            .generate_function(ppi_ast)
            .map_err(|e| anyhow::anyhow!("Code generation failed: {}", e))
    }

    /// Generate content for a single function file
    fn generate_function_file_content(&self, prefix: &str, function_codes: &[String]) -> String {
        let mut content = format!(
            "//! Generated AST functions for hash prefix {}\n",
            prefix.to_uppercase()
        );
        content.push_str("//!\n");
        content.push_str(
            "//! This file is auto-generated by PpiFunctionRegistry. Do not edit manually.\n",
        );
        content.push('\n');

        // Standard imports
        content.push_str("use crate::types::{TagValue, ExifError};\n");
        content.push('\n');

        // Add all functions for this prefix
        for function_code in function_codes {
            content.push_str(function_code);
            content.push_str("\n\n");
        }

        content
    }

    /// Generate the main mod.rs file that declares all submodules
    fn generate_mod_file(
        &self,
        functions_by_prefix: &HashMap<String, Vec<String>>,
    ) -> Result<GeneratedFile> {
        let mut content = String::new();
        content.push_str("//! AST-generated function modules\n");
        content.push_str("//!\n");
        content.push_str(
            "//! This file is auto-generated by PpiFunctionRegistry. Do not edit manually.\n",
        );
        content.push_str(
            "//! Functions are organized by the first two characters of their AST hash.\n",
        );
        content.push('\n');

        // Declare all submodules (sorted for deterministic output)
        let mut prefixes: Vec<_> = functions_by_prefix.keys().collect();
        prefixes.sort();

        for prefix in prefixes {
            content.push_str(&format!("pub mod hash_{};\n", prefix));
        }

        Ok(GeneratedFile {
            path: "functions/mod.rs".to_string(),
            content,
            strategy: "PpiFunctionRegistry".to_string(),
        })
    }

    /// Get registry statistics for debugging
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_functions: self.generated_functions.len(),
            unique_asts: self.ast_to_function.len(),
        }
    }
}

/// Statistics about the registry state
#[derive(Debug)]
pub struct RegistryStats {
    pub total_functions: usize,
    pub unique_asts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppi::parse_ppi_json;
    use serde_json::json;

    #[test]
    fn test_ast_hashing_consistency() {
        let registry = PpiFunctionRegistry::new();

        // Same AST structure should produce same hash
        let ast_json = json!({
            "class": "PPI::Document",
            "children": [{
                "class": "PPI::Statement",
                "children": [{
                    "class": "PPI::Token::Symbol",
                    "content": "$val"
                }, {
                    "class": "PPI::Token::Operator",
                    "content": "/"
                }, {
                    "class": "PPI::Token::Number",
                    "content": "100",
                    "numeric_value": 100
                }]
            }]
        });

        let ppi_ast = parse_ppi_json(&ast_json).unwrap();
        let hash1 = registry.hash_ast_structure(&ppi_ast).unwrap();
        let hash2 = registry.hash_ast_structure(&ppi_ast).unwrap();

        assert_eq!(hash1, hash2, "Same AST should produce same hash");
    }

    #[test]
    fn test_function_spec_generation() {
        let registry = PpiFunctionRegistry::new();
        let spec = registry.create_function_spec(
            "a1b2c3d4e5f67890",
            ExpressionType::ValueConv,
            "$val / 100",
        );

        assert_eq!(spec.function_name, "ast_value_a1b2c3d4");
        assert_eq!(spec.module_path, "crate::generated::functions::hash_a1");
        assert_eq!(spec.hash_prefix, "a1");
        assert_eq!(spec.original_expression, "$val / 100");
    }

    #[test]
    fn test_deduplication() {
        let mut registry = PpiFunctionRegistry::new();

        let ast_json = json!({
            "class": "PPI::Token::Symbol",
            "content": "$val"
        });
        let ppi_ast = parse_ppi_json(&ast_json).unwrap();

        // Register the same AST twice
        let spec1 = registry
            .register_ast(&ppi_ast, ExpressionType::ValueConv, "$val")
            .unwrap();
        let spec2 = registry
            .register_ast(&ppi_ast, ExpressionType::ValueConv, "$val")
            .unwrap();

        assert_eq!(
            spec1.function_name, spec2.function_name,
            "Same AST should return same function"
        );
        assert_eq!(
            registry.generated_functions.len(),
            1,
            "Should only generate one function"
        );
    }
}
