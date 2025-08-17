//! Core PPI function registry implementation
//!
//! This module contains the main PPI function registry that handles
//! AST deduplication and function specification management.

use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

use super::stats::{ConversionStats, RegistryStats};
use crate::ppi::{ExpressionType, PpiNode};

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

    /// Generate hash from AST structure for deduplication
    fn hash_ast_structure(&self, ppi_ast: &PpiNode) -> Result<String> {
        let json_str = serde_json::to_string(ppi_ast)?;
        let mut hasher = DefaultHasher::new();
        json_str.hash(&mut hasher);
        let hash = hasher.finish();
        Ok(format!("{:x}", hash))
    }

    /// Create function spec from hash and expression details
    fn create_function_spec(
        &self,
        ast_hash: &str,
        expression_type: ExpressionType,
        original_expression: &str,
    ) -> FunctionSpec {
        let hash_prefix = ast_hash.chars().take(2).collect::<String>();
        let function_name = match expression_type {
            ExpressionType::ValueConv => format!("ast_value_{}", ast_hash),
            ExpressionType::PrintConv => format!("ast_print_{}", ast_hash),
            ExpressionType::Condition => format!("ast_condition_{}", ast_hash),
        };

        let module_path = format!("crate::generated::functions::hash_{}", hash_prefix);

        FunctionSpec {
            function_name,
            module_path,
            hash_prefix,
            expression_type,
            original_expression: original_expression.to_string(),
        }
    }

    /// Find AST hash for a given function spec (helper for fallback generation)
    pub fn find_ast_hash_for_function(&self, function_spec: &FunctionSpec) -> Result<String> {
        for (ast_hash, spec) in &self.ast_to_function {
            if spec.function_name == function_spec.function_name {
                return Ok(ast_hash.clone());
            }
        }
        anyhow::bail!(
            "No AST hash found for function: {}",
            function_spec.function_name
        )
    }

    /// Record conversion attempt for statistics
    pub fn record_conversion_attempt(&mut self, expression_type: ExpressionType) {
        self.conversion_stats.record_attempt(expression_type);
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total_functions = self.ast_to_function.len();
        let total_registrations: usize = self
            .usage_contexts
            .values()
            .map(|contexts| contexts.len())
            .sum();
        let deduplicated_count = if total_registrations > total_functions {
            total_registrations - total_functions
        } else {
            0
        };

        RegistryStats {
            total_functions,
            total_registrations,
            deduplicated_count,
            conversion_stats: self.conversion_stats.clone(),
        }
    }

    /// Get reference to AST nodes for code generation
    pub fn ast_nodes(&self) -> &HashMap<String, PpiNode> {
        &self.ast_nodes
    }

    /// Get reference to function specs
    pub fn ast_to_function(&self) -> &HashMap<String, FunctionSpec> {
        &self.ast_to_function
    }

    /// Get reference to usage contexts
    pub fn usage_contexts(&self) -> &HashMap<String, BTreeSet<UsageContext>> {
        &self.usage_contexts
    }

    /// Get mutable reference to conversion stats
    pub fn conversion_stats_mut(&mut self) -> &mut ConversionStats {
        &mut self.conversion_stats
    }
}
