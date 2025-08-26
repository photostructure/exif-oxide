//! PPI Function Registry for Deduplication
//!
//! This registry provides centralized management of PPI-generated functions to eliminate
//! duplicates across modules. Functions are deduplicated based on AST structure hash,
//! ensuring that semantically equivalent expressions share the same implementation.

mod registry;
mod stats;

pub use registry::{FunctionSpec, PpiFunctionRegistry, UsageContext};

use anyhow::Result;
use indoc::formatdoc;
use std::collections::HashMap;
use tracing::debug;

use crate::ppi::rust_generator::generator::RustGenerator;
use crate::ppi::{ExpressionType, PpiNode};
use crate::strategies::GeneratedFile;

// Implement the main functionality for PpiFunctionRegistry
impl PpiFunctionRegistry {
    /// Generate all function files after all modules have been processed
    pub fn generate_function_files(&mut self) -> Result<Vec<GeneratedFile>> {
        self.generate_function_files_with_imports("crate::types::{TagValue, ExifContext}")
    }

    /// Generate all function files with custom imports (for test environment)
    pub fn generate_function_files_with_imports(
        &mut self,
        import_path: &str,
    ) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        // Collect all the data we need first to avoid borrow checker issues
        let mut ast_function_data = Vec::new();
        for (ast_hash, function_spec) in self.ast_to_function() {
            if let Some(ppi_ast) = self.ast_nodes().get(ast_hash) {
                ast_function_data.push((ast_hash.clone(), function_spec.clone(), ppi_ast.clone()));
            }
        }

        // Generate function code with usage contexts now that all registrations are complete
        let mut generated_functions: HashMap<String, String> = HashMap::new();
        for (ast_hash, function_spec, ppi_ast) in ast_function_data {
            let function_code =
                self.generate_function_code_with_stats(&ppi_ast, &function_spec, &ast_hash)?;
            generated_functions.insert(ast_hash, function_code);
        }

        // Group functions by their two-character hash prefix
        let mut functions_by_prefix: HashMap<String, Vec<String>> = HashMap::new();

        for (ast_hash, function_code) in &generated_functions {
            let prefix = ast_hash.chars().take(2).collect::<String>();
            functions_by_prefix
                .entry(prefix)
                .or_default()
                .push(function_code.clone());
        }

        // Generate one file per prefix (sorted for deterministic output)
        let mut prefixes: Vec<_> = functions_by_prefix.keys().collect();
        prefixes.sort();

        for prefix in prefixes {
            let function_codes = functions_by_prefix.get(prefix).unwrap();
            let file_content = self.generate_function_file_content_with_imports(
                prefix,
                function_codes,
                import_path,
            );
            let file_path = format!("functions/hash_{}.rs", prefix);

            files.push(GeneratedFile {
                path: file_path,
                content: file_content,
            });
        }

        // Generate the main mod.rs file
        let mod_file = self.generate_mod_file(&functions_by_prefix)?;
        files.push(mod_file);

        Ok(files)
    }

    /// Generate Rust function code from AST with usage context and statistics tracking
    fn generate_function_code_with_stats(
        &mut self,
        ppi_ast: &PpiNode,
        function_spec: &FunctionSpec,
        ast_hash: &str,
    ) -> Result<String> {
        let generator = RustGenerator::new(
            function_spec.expression_type,
            function_spec.function_name.clone(),
            function_spec.original_expression.clone(),
        );

        // Try PPI generation first
        let mut function_code = match generator.generate_function(ppi_ast) {
            Ok(code) => {
                // PPI generation succeeded - record the success
                self.conversion_stats_mut()
                    .record_success(function_spec.expression_type);
                code
            }
            Err(e) => {
                // PPI generation failed - try impl_registry fallback
                debug!(
                    "PPI generation failed for '{}': {}, trying impl_registry fallback",
                    function_spec.original_expression, e
                );

                self.generate_fallback_function(function_spec, ast_hash)?
            }
        };

        // Add usage context to the documentation if available
        if let Some(usage_set) = self.usage_contexts().get(ast_hash) {
            if !usage_set.is_empty() {
                // Find the end of the perl expression doc comment block
                // Look for the closing ``` followed by the function declaration
                if let Some(end_pos) = function_code.find("/// ```\npub fn") {
                    // Insert right after the closing ```
                    let insert_pos = end_pos + 8; // After "/// ```\n"
                    let mut usage_docs = String::from("/// Used by:\n");
                    for context in usage_set {
                        usage_docs.push_str(&format!("/// - {}\n", context));
                    }
                    function_code.insert_str(insert_pos, &usage_docs);
                }
            }
        }

        Ok(function_code)
    }

    /// Generate content for a single function file with custom imports
    fn generate_function_file_content_with_imports(
        &self,
        prefix: &str,
        function_codes: &[String],
        import_path: &str,
    ) -> String {
        let mut content = formatdoc! {r#"
            //! Generated AST functions for hash prefix {prefix_upper}
            //!
            //! This file is auto-generated by codegen/src/ppi/fn_registry/mod.rs.
            //! DO NOT EDIT MANUALLY.

            #![allow(dead_code, unused_variables, unreachable_code)]

            use {import_path};

        "#, prefix_upper = prefix.to_uppercase(), import_path = import_path};

        // Sort function codes for deterministic output
        let mut sorted_function_codes = function_codes.to_vec();
        sorted_function_codes.sort();

        // Add all functions for this prefix
        for function_code in sorted_function_codes {
            content.push_str(&function_code);
            content.push_str("\n\n");
        }

        content
    }

    /// Generate the main mod.rs file that declares all submodules
    fn generate_mod_file(
        &self,
        functions_by_prefix: &HashMap<String, Vec<String>>,
    ) -> Result<GeneratedFile> {
        let mut content = formatdoc! {r#"
            //! AST-generated function modules
            //!
            //! This file is auto-generated by codegen/src/ppi/fn_registry/mod.rs. Do not edit manually.
            //! Functions are organized by the first two characters of their AST hash.

        "#};

        // Declare all submodules (sorted for deterministic output)
        let mut prefixes: Vec<_> = functions_by_prefix.keys().collect();
        prefixes.sort();

        for prefix in prefixes {
            content.push_str(&format!("pub mod hash_{};\n", prefix));
        }

        Ok(GeneratedFile {
            path: "functions/mod.rs".to_string(),
            content,
        })
    }

    /// Generate fallback function using impl_registry when PPI generation fails
    fn generate_fallback_function(
        &self,
        function_spec: &FunctionSpec,
        ast_hash: &str,
    ) -> Result<String> {
        // For now, just generate a placeholder function
        // TODO: Add impl_registry integration later
        Ok(self.generate_placeholder_function(function_spec, ast_hash))
    }

    /// Generate a placeholder function for expressions with no implementation
    fn generate_placeholder_function(
        &self,
        function_spec: &FunctionSpec,
        ast_hash: &str,
    ) -> String {
        let (signature, default_return) = match function_spec.expression_type {
            ExpressionType::Condition => (
                format!(
                    "pub fn {}(val: &TagValue, ctx: &ExifContext) -> bool",
                    function_spec.function_name
                ),
                "false",
            ),
            ExpressionType::ValueConv => (
                format!(
                    "pub fn {}(val: &TagValue) -> Result<TagValue, codegen_runtime::types::ExifError>",
                    function_spec.function_name
                ),
                "Ok(val.clone())",
            ),
            ExpressionType::PrintConv => (
                format!(
                    "pub fn {}(val: &TagValue) -> TagValue",
                    function_spec.function_name
                ),
                "val.clone()",
            ),
        };

        // Add usage context if available
        let usage_docs = if let Some(usage_set) = self.usage_contexts().get(ast_hash) {
            if !usage_set.is_empty() {
                let mut docs = String::from("/// Used by:\n");
                for context in usage_set {
                    docs.push_str(&format!("/// - {}\n", context));
                }
                docs
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Generate the Perl expression documentation
        let perl_doc =
            RustGenerator::format_perl_expression_doc(&function_spec.original_expression);

        formatdoc! {r#"
            /// PLACEHOLDER: Unsupported expression (missing implementation)
            {}{}/// TODO: Add support for this expression pattern
            {}
            {{
                tracing::warn!("Missing implementation for expression in {{}}", file!());
                {}
            }}
        "#,
            perl_doc,
            usage_docs,
            signature,
            default_return
        }
    }
}
