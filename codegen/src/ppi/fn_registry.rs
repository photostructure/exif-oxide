//! PPI Function Registry for Deduplication
//!
//! This registry provides centralized management of PPI-generated functions to eliminate
//! duplicates across modules. Functions are deduplicated based on AST structure hash,
//! ensuring that semantically equivalent expressions share the same implementation.

use anyhow::Result;
use indoc::formatdoc;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use tracing::debug;

use crate::impl_registry::{
    classify_valueconv_expression, lookup_printconv, lookup_tag_specific_printconv, ValueConvType,
};
use crate::ppi::{ExpressionType, PpiNode, RustGenerator};
use crate::strategies::GeneratedFile;

/// Statistics for tracking conversion processing success rates
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    /// PrintConv attempts and successes
    pub print_conv_attempts: usize,
    pub print_conv_successes: usize,
    /// ValueConv attempts and successes  
    pub value_conv_attempts: usize,
    pub value_conv_successes: usize,
    /// Condition attempts and successes
    pub condition_attempts: usize,
    pub condition_successes: usize,
}

impl ConversionStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an attempt to process a conversion
    pub fn record_attempt(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_attempts += 1,
            ExpressionType::ValueConv => self.value_conv_attempts += 1,
            ExpressionType::Condition => self.condition_attempts += 1,
        }
    }

    /// Record a successful conversion processing
    pub fn record_success(&mut self, expression_type: ExpressionType) {
        match expression_type {
            ExpressionType::PrintConv => self.print_conv_successes += 1,
            ExpressionType::ValueConv => self.value_conv_successes += 1,
            ExpressionType::Condition => self.condition_successes += 1,
        }
    }

    /// Calculate success rate for a given expression type
    pub fn success_rate(&self, expression_type: ExpressionType) -> f64 {
        let (attempts, successes) = match expression_type {
            ExpressionType::PrintConv => (self.print_conv_attempts, self.print_conv_successes),
            ExpressionType::ValueConv => (self.value_conv_attempts, self.value_conv_successes),
            ExpressionType::Condition => (self.condition_attempts, self.condition_successes),
        };

        if attempts == 0 {
            0.0
        } else {
            (successes as f64 / attempts as f64) * 100.0
        }
    }
}

/// Context for where a function is used
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UsageContext {
    /// Module name (e.g., "FujiFilm_pm")
    pub module: String,
    /// Table name (e.g., "MAIN_TAGS")
    pub table: String,
    /// Tag name (e.g., "DigitalZoom")
    pub tag: String,
}

impl std::fmt::Display for UsageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}.{}", self.module, self.table, self.tag)
    }
}

/// Specification for a deduplicated function
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    /// Function name (e.g., "ast_value_a1b2c3d4")
    pub function_name: String,
    /// Import path for use in tag modules (e.g., "crate::generated::fn::a1")
    pub module_path: String,
    /// Two-character hash prefix for file organization
    #[allow(dead_code)]
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
    /// Map from AST hash to PPI AST (stored for later code generation)
    ast_nodes: HashMap<String, PpiNode>,
    /// Map from AST hash to all places where this function is used (sorted for determinism)
    usage_contexts: HashMap<String, BTreeSet<UsageContext>>,
    /// Statistics tracking for conversion processing
    conversion_stats: ConversionStats,
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
            ast_nodes: HashMap::new(),
            usage_contexts: HashMap::new(),
            conversion_stats: ConversionStats::new(),
        }
    }

    /// Register an AST and get back a function specification
    ///
    /// If the AST has been seen before, returns the existing function spec and adds the new usage.
    /// Otherwise, generates a new function and returns its spec.
    pub fn register_ast(
        &mut self,
        ppi_ast: &PpiNode,
        expression_type: ExpressionType,
        original_expression: &str,
        usage_context: Option<UsageContext>,
    ) -> Result<FunctionSpec> {
        // Generate hash from AST structure (not expression text)
        let ast_hash = self.hash_ast_structure(ppi_ast)?;

        // Add usage context if provided
        if let Some(context) = usage_context {
            self.usage_contexts
                .entry(ast_hash.clone())
                .or_insert_with(BTreeSet::new)
                .insert(context);
        }

        // Check if we already have this AST registered
        if let Some(existing_spec) = self.ast_to_function.get(&ast_hash) {
            return Ok(existing_spec.clone());
        }

        // Generate new function for this AST
        let function_spec =
            self.create_function_spec(&ast_hash, expression_type, original_expression);

        // Store the spec and the AST for later code generation
        self.ast_to_function
            .insert(ast_hash.clone(), function_spec.clone());
        self.ast_nodes.insert(ast_hash.clone(), ppi_ast.clone());

        // Note: Success tracking moved to generate_function_code where actual PPI generation happens

        Ok(function_spec)
    }

    /// Generate all function files after all modules have been processed
    pub fn generate_function_files(&mut self) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        // Collect all the data we need first to avoid borrow checker issues
        let mut ast_function_data = Vec::new();
        for (ast_hash, function_spec) in &self.ast_to_function {
            if let Some(ppi_ast) = self.ast_nodes.get(ast_hash) {
                ast_function_data.push((ast_hash.clone(), function_spec.clone(), ppi_ast.clone()));
            }
        }

        // Generate function code with usage contexts now that all registrations are complete
        let mut generated_functions = HashMap::new();
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
            let file_content = self.generate_function_file_content(prefix, function_codes);
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
                self.conversion_stats
                    .record_success(function_spec.expression_type);
                code
            }
            Err(e) => {
                // PPI generation failed - try impl_registry fallback
                debug!(
                    "PPI generation failed for '{}': {}, trying impl_registry fallback",
                    function_spec.original_expression, e
                );

                self.generate_fallback_function(function_spec)?
            }
        };

        // Add usage context to the documentation if available
        if let Some(usage_set) = self.usage_contexts.get(ast_hash) {
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

    /// Generate Rust function code from AST with usage context (legacy method)
    fn generate_function_code(
        &self,
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
            Ok(code) => code,
            Err(e) => {
                // PPI generation failed - try impl_registry fallback
                debug!(
                    "PPI generation failed for '{}': {}, trying impl_registry fallback",
                    function_spec.original_expression, e
                );

                self.generate_fallback_function(function_spec)?
            }
        };

        // Add usage context to the documentation if available
        if let Some(usage_set) = self.usage_contexts.get(ast_hash) {
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

    /// Find the AST hash for a given function spec (reverse lookup)
    fn find_ast_hash_for_function(&self, function_spec: &FunctionSpec) -> Result<String> {
        for (ast_hash, spec) in &self.ast_to_function {
            if spec.function_name == function_spec.function_name {
                return Ok(ast_hash.clone());
            }
        }
        Err(anyhow::anyhow!(
            "Could not find AST hash for function: {}",
            function_spec.function_name
        ))
    }

    /// Generate fallback function using impl_registry when PPI generation fails
    fn generate_fallback_function(&self, function_spec: &FunctionSpec) -> Result<String> {
        match function_spec.expression_type {
            ExpressionType::PrintConv => {
                // Try tag-specific lookup first if we have usage context
                let ast_hash = self.find_ast_hash_for_function(function_spec)?;
                if let Some(usage_set) = self.usage_contexts.get(&ast_hash) {
                    if let Some(context) = usage_set.first() {
                        if let Some((module_path, func_name)) =
                            lookup_tag_specific_printconv(&context.module, &context.tag)
                        {
                            return Ok(self.generate_registry_wrapper_function(
                                function_spec,
                                module_path,
                                func_name,
                                "Tag-specific PrintConv registry match",
                            ));
                        }
                    }
                }

                // Fall back to general PrintConv lookup
                if let Some((module_path, func_name)) =
                    lookup_printconv(&function_spec.original_expression, "unknown")
                {
                    return Ok(self.generate_registry_wrapper_function(
                        function_spec,
                        module_path,
                        func_name,
                        "General PrintConv registry match",
                    ));
                }
            }
            ExpressionType::ValueConv => {
                // Try impl_registry lookup for ValueConv
                match classify_valueconv_expression(&function_spec.original_expression, "unknown") {
                    ValueConvType::CustomFunction(module_path, func_name) => {
                        return Ok(self.generate_registry_wrapper_function(
                            function_spec,
                            module_path,
                            func_name,
                            "ValueConv registry match",
                        ));
                    }
                    _ => {
                        // Other ValueConv types like PpiGenerated* are not applicable here
                        // since we're in the fallback path because PPI failed
                    }
                }
            }
            ExpressionType::Condition => {
                // For now, conditions don't have a registry lookup
                // Fall through to placeholder generation
            }
        }

        // No registry match found - generate placeholder function
        Ok(self.generate_placeholder_function(function_spec))
    }

    /// Generate a wrapper function that calls an impl_registry function
    fn generate_registry_wrapper_function(
        &self,
        function_spec: &FunctionSpec,
        module_path: &str,
        func_name: &str,
        source_description: &str,
    ) -> String {
        let (signature, _return_type) = match function_spec.expression_type {
            ExpressionType::Condition => (
                format!(
                    "pub fn {}(val: &TagValue, ctx: &ExifContext) -> bool",
                    function_spec.function_name
                ),
                "bool",
            ),
            ExpressionType::ValueConv => (
                format!(
                    "pub fn {}(val: &TagValue) -> Result<TagValue, crate::types::ExifError>",
                    function_spec.function_name
                ),
                "Result<TagValue, crate::types::ExifError>",
            ),
            ExpressionType::PrintConv => (
                format!(
                    "pub fn {}(val: &TagValue) -> TagValue",
                    function_spec.function_name
                ),
                "TagValue",
            ),
        };

        formatdoc! {r#"
            /// Registry fallback for: {}
            /// Source: {}
            /// Original expression:
            /// ```perl
            /// {}
            /// ```
            {} {{
                {}::{}(val)
            }}
        "#,
            function_spec.function_name,
            source_description,
            function_spec.original_expression,
            signature,
            module_path,
            func_name
        }
    }

    /// Generate a placeholder function for expressions with no impl_registry match
    fn generate_placeholder_function(&self, function_spec: &FunctionSpec) -> String {
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
                    "pub fn {}(val: &TagValue) -> Result<TagValue, crate::types::ExifError>",
                    function_spec.function_name
                ),
                "Ok(val.clone())",
            ),
            ExpressionType::PrintConv => (
                format!(
                    "pub fn {}(val: &TagValue) -> TagValue",
                    function_spec.function_name
                ),
                "TagValue::String(format!(\"[MISSING: {}]\", expr))",
            ),
        };

        // Escape the original expression for both doc comments and string literals
        let escaped_for_doc = function_spec
            .original_expression
            .replace('\\', "\\\\") // Escape backslashes
            .replace('"', "\\\"") // Escape quotes
            .replace('\'', "\\'") // Escape single quotes for Rust
            .replace('\n', "\\n") // Escape newlines
            .replace('\r', "\\r") // Escape carriage returns
            .replace('\t', "\\t"); // Escape tabs

        let escaped_for_string = function_spec
            .original_expression
            .replace('\\', "\\\\") // Escape backslashes first
            .replace('"', "\\\"") // Escape double quotes
            .replace('\'', "\\'"); // Escape single quotes

        formatdoc! {r#"
            /// PLACEHOLDER: Unsupported expression (missing implementation)
            /// Original expression:
            /// ```perl
            /// {}
            /// ```
            /// TODO: Add support for this expression pattern
            {} {{
                let expr = "{}";
                tracing::warn!("Missing implementation for expression: {{}}", expr);
                {}
            }}
        "#,
            escaped_for_doc,
            signature,
            escaped_for_string,
            default_return
        }
    }

    /// Generate content for a single function file
    fn generate_function_file_content(&self, prefix: &str, function_codes: &[String]) -> String {
        let mut content = formatdoc! {r#"
            //! Generated AST functions for hash prefix {prefix_upper}
            //!
            //! This file is auto-generated by codegen/src/ppi/fn_registry.rs.
            //! DO NOT EDIT MANUALLY.

            use crate::types::{{TagValue, ExifError}};

        "#, prefix_upper = prefix.to_uppercase()};

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
            //! This file is auto-generated by codegen/src/ppi/fn_registry.rs. Do not edit manually.
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

    /// Record an attempt to process a conversion expression
    pub fn record_conversion_attempt(&mut self, expression_type: ExpressionType) {
        self.conversion_stats.record_attempt(expression_type);
    }

    /// Get registry statistics for debugging
    #[allow(dead_code)]
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_functions: self.ast_to_function.len(),
            unique_asts: self.ast_to_function.len(),
            conversion_stats: self.conversion_stats.clone(),
        }
    }
}

/// Statistics about the registry state
#[derive(Debug)]
#[allow(dead_code)]
pub struct RegistryStats {
    pub total_functions: usize,
    pub unique_asts: usize,
    pub conversion_stats: ConversionStats,
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
            .register_ast(&ppi_ast, ExpressionType::ValueConv, "$val", None)
            .unwrap();
        let spec2 = registry
            .register_ast(&ppi_ast, ExpressionType::ValueConv, "$val", None)
            .unwrap();

        assert_eq!(
            spec1.function_name, spec2.function_name,
            "Same AST should return same function"
        );
        assert_eq!(
            registry.ast_nodes.len(),
            1,
            "Should only store one AST node"
        );
    }

    #[test]
    fn test_conversion_statistics() {
        let mut registry = PpiFunctionRegistry::new();

        // Test recording attempts
        registry.record_conversion_attempt(ExpressionType::PrintConv);
        registry.record_conversion_attempt(ExpressionType::PrintConv);
        registry.record_conversion_attempt(ExpressionType::ValueConv);

        let stats = registry.stats();
        assert_eq!(stats.conversion_stats.print_conv_attempts, 2);
        assert_eq!(stats.conversion_stats.value_conv_attempts, 1);
        assert_eq!(stats.conversion_stats.condition_attempts, 0);

        // Test success recording through actual function generation
        let ast_json = json!({
            "class": "PPI::Token::Symbol",
            "content": "$val"
        });
        let ppi_ast = parse_ppi_json(&ast_json).unwrap();

        // Register the AST (doesn't record success yet)
        let function_spec = registry
            .register_ast(&ppi_ast, ExpressionType::PrintConv, "$val", None)
            .unwrap();

        // Success is recorded during function generation
        let _function_code = registry
            .generate_function_code_with_stats(&ppi_ast, &function_spec, "test_hash")
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.conversion_stats.print_conv_successes, 1);
        assert_eq!(stats.conversion_stats.value_conv_successes, 0);

        // Test success rate calculation
        assert_eq!(
            stats
                .conversion_stats
                .success_rate(ExpressionType::PrintConv),
            50.0
        ); // 1/2
        assert_eq!(
            stats
                .conversion_stats
                .success_rate(ExpressionType::ValueConv),
            0.0
        ); // 0/1
        assert_eq!(
            stats
                .conversion_stats
                .success_rate(ExpressionType::Condition),
            0.0
        ); // 0/0
    }
}
