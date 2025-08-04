//! Strategy pattern system for processing field extractor output
//!
//! This module provides a strategy-based approach to processing JSON symbols
//! extracted from ExifTool modules, replacing the config-driven extraction
//! system with duck-typing pattern recognition.

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::field_extractor::FieldSymbol;

/// Core trait that all extraction strategies implement
pub trait ExtractionStrategy: Send + Sync {
    /// Name of the strategy for logging and debugging
    fn name(&self) -> &'static str;
    
    /// Check if this strategy can handle the given symbol data
    /// Uses duck-typing pattern recognition on JSON structure
    fn can_handle(&self, symbol_data: &JsonValue) -> bool;
    
    /// Extract data from the symbol and generate appropriate code
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext) -> Result<()>;
    
    /// Finalize processing for a specific module
    fn finish_module(&mut self, module_name: &str) -> Result<()>;
    
    /// Complete extraction and return generated files
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>>;
}

/// Context passed to strategies during extraction
#[derive(Debug)]
pub struct ExtractionContext {
    /// Output directory for generated code
    pub output_dir: String,
    
    /// Current module being processed
    pub current_module: Option<String>,
    
    /// Global symbol registry for cross-references
    pub symbol_registry: HashMap<String, FieldSymbol>,
    
    /// Strategy selection log for debugging
    pub strategy_log: Vec<StrategySelection>,
}

/// Record of strategy selection decisions for debugging
#[derive(Debug, Clone)]
pub struct StrategySelection {
    pub symbol_name: String,
    pub module_name: String,
    pub strategy_name: String,
    pub reasoning: String,
}

/// Generated file from strategy processing
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Relative path from output directory
    pub path: String,
    
    /// Generated Rust code content
    pub content: String,
    
    /// Strategy that generated this file
    pub strategy: String,
}

impl ExtractionContext {
    /// Create new extraction context
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            current_module: None,
            symbol_registry: HashMap::new(),
            strategy_log: Vec::new(),
        }
    }
    
    /// Log strategy selection decision
    pub fn log_strategy_selection(&mut self, symbol: &FieldSymbol, strategy_name: &str, reasoning: &str) {
        let selection = StrategySelection {
            symbol_name: symbol.name.clone(),
            module_name: symbol.module.clone(),
            strategy_name: strategy_name.to_string(),
            reasoning: reasoning.to_string(),
        };
        
        debug!("Strategy selection: {} -> {} ({})", symbol.name, strategy_name, reasoning);
        self.strategy_log.push(selection);
    }
    
    /// Register symbol for cross-references
    pub fn register_symbol(&mut self, symbol: FieldSymbol) {
        let key = format!("{}::{}", symbol.module, symbol.name);
        self.symbol_registry.insert(key, symbol);
    }
}

/// Strategy dispatcher that processes symbols through available strategies
pub struct StrategyDispatcher {
    strategies: Vec<Box<dyn ExtractionStrategy>>,
}

impl StrategyDispatcher {
    /// Create new dispatcher with all available strategies
    pub fn new() -> Self {
        Self {
            strategies: all_strategies(),
        }
    }
    
    /// Process a collection of symbols through the strategy system
    pub fn process_symbols(
        &mut self, 
        symbols: Vec<FieldSymbol>, 
        output_dir: &str
    ) -> Result<Vec<GeneratedFile>> {
        let mut context = ExtractionContext::new(output_dir.to_string());
        let mut generated_files = Vec::new();
        
        info!("🔄 Processing {} symbols through strategy system", symbols.len());
        
        // Register all symbols first for cross-references
        for symbol in &symbols {
            context.register_symbol(symbol.clone());
        }
        
        // Process each symbol through strategies
        for symbol in symbols {
            self.process_single_symbol(symbol, &mut context)?;
        }
        
        // Finalize all strategies and collect generated files
        for strategy in &mut self.strategies {
            let mut files = strategy.finish_extraction()?;
            generated_files.append(&mut files);
        }
        
        // Write strategy selection log for debugging
        self.write_strategy_log(&context, output_dir)?;
        
        info!("✅ Strategy processing complete: {} files generated", generated_files.len());
        Ok(generated_files)
    }
    
    /// Process a single symbol through the first matching strategy
    fn process_single_symbol(&mut self, symbol: FieldSymbol, context: &mut ExtractionContext) -> Result<()> {
        // Find the first matching strategy
        let mut matched_strategy_index = None;
        let mut reasoning = String::new();
        
        for (index, strategy) in self.strategies.iter().enumerate() {
            if strategy.can_handle(&symbol.data) {
                reasoning = format!("Pattern matched: {}", self.describe_pattern(&symbol.data));
                matched_strategy_index = Some(index);
                break;
            }
        }
        
        // Process with the matched strategy
        if let Some(index) = matched_strategy_index {
            let strategy = &mut self.strategies[index];
            context.log_strategy_selection(&symbol, strategy.name(), &reasoning);
            strategy.extract(&symbol, context)
        } else {
            // No strategy could handle this symbol
            warn!("⚠️  No strategy found for symbol: {} (type: {}, module: {})", 
                  symbol.name, symbol.symbol_type, symbol.module);
            
            context.log_strategy_selection(&symbol, "none", "No matching pattern");
            Ok(())
        }
    }
    
    /// Describe the pattern structure for logging
    fn describe_pattern(&self, value: &JsonValue) -> String {
        match value {
            JsonValue::Object(map) => {
                if map.is_empty() {
                    "empty object".to_string()
                } else if map.contains_key("PrintConv") || map.contains_key("ValueConv") {
                    "tag definition with conversions".to_string()
                } else if map.values().all(|v| v.is_string()) {
                    format!("string map ({} keys)", map.len())
                } else {
                    format!("complex object ({} keys)", map.len())
                }
            }
            JsonValue::Array(arr) => {
                format!("array ({} elements)", arr.len())
            }
            JsonValue::String(_) => "string scalar".to_string(),
            JsonValue::Number(_) => "number scalar".to_string(),
            JsonValue::Bool(_) => "boolean scalar".to_string(),
            JsonValue::Null => "null".to_string(),
        }
    }
    
    /// Write strategy selection log for debugging
    fn write_strategy_log(&self, context: &ExtractionContext, output_dir: &str) -> Result<()> {
        use std::fs;
        
        let log_path = Path::new(output_dir).join("strategy_selection.log");
        let mut log_content = String::new();
        
        log_content.push_str("# Strategy Selection Log\n");
        log_content.push_str("# Format: Symbol Module Strategy Reasoning\n\n");
        
        for selection in &context.strategy_log {
            log_content.push_str(&format!(
                "{} {} {} {}\n",
                selection.symbol_name,
                selection.module_name, 
                selection.strategy_name,
                selection.reasoning
            ));
        }
        
        fs::write(log_path, log_content)?;
        debug!("📋 Strategy selection log written to strategy_selection.log");
        
        Ok(())
    }
}

impl Default for StrategyDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// Strategy implementations
mod simple_table;
mod tag_kit;
mod binary_data;
mod boolean_set;
mod composite_tag;

// Output location utilities
pub mod output_locations;

// Re-export strategy implementations
pub use simple_table::SimpleTableStrategy;
pub use tag_kit::TagKitStrategy;
pub use binary_data::BinaryDataStrategy;
pub use boolean_set::BooleanSetStrategy;
pub use composite_tag::CompositeTagStrategy;

/// Registry of all available strategies
/// Ordered by precedence - first-match-wins
pub fn all_strategies() -> Vec<Box<dyn ExtractionStrategy>> {
    vec![
        // Order matters: first-match wins
        Box::new(TagKitStrategy::new()),      // Complex tag definitions (Main, Composite tables)
        Box::new(BinaryDataStrategy::new()),  // ProcessBinaryData tables (CameraInfo*, etc.)
        Box::new(CompositeTagStrategy::new()), // Composite tag definitions
        Box::new(BooleanSetStrategy::new()),  // Membership sets (isDat*, isTxt*, etc.)
        Box::new(SimpleTableStrategy::new()), // Simple key-value lookups (fallback)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_strategy_dispatcher_creation() {
        let dispatcher = StrategyDispatcher::new();
        assert_eq!(dispatcher.strategies.len(), 1); // Only SimpleTableStrategy for now
    }
    
    #[test]
    fn test_extraction_context() {
        let mut context = ExtractionContext::new("output".to_string());
        
        let symbol = FieldSymbol {
            symbol_type: "hash".to_string(),
            name: "test_symbol".to_string(),
            data: json!({"key": "value"}),
            module: "TestModule".to_string(),
            metadata: crate::field_extractor::FieldMetadata {
                size: 1,
                complexity: "simple".to_string(),
                has_non_serializable: false,
            },
        };
        
        context.log_strategy_selection(&symbol, "TestStrategy", "test reasoning");
        assert_eq!(context.strategy_log.len(), 1);
        
        context.register_symbol(symbol);
        assert_eq!(context.symbol_registry.len(), 1);
    }
    
    #[test]
    fn test_pattern_description() {
        let dispatcher = StrategyDispatcher::new();
        
        // Test different pattern types
        assert_eq!(dispatcher.describe_pattern(&json!({})), "empty object");
        assert_eq!(dispatcher.describe_pattern(&json!({"a": "1", "b": "2"})), "string map (2 keys)");
        assert_eq!(dispatcher.describe_pattern(&json!({"PrintConv": "test"})), "tag definition with conversions");
        assert_eq!(dispatcher.describe_pattern(&json!([1, 2, 3])), "array (3 elements)");
        assert_eq!(dispatcher.describe_pattern(&json!("test")), "string scalar");
    }
}