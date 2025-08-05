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
    
    /// Check if this strategy can handle the given symbol
    /// Uses duck-typing pattern recognition on symbol structure
    fn can_handle(&self, symbol: &FieldSymbol) -> bool;
    
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
        use std::collections::HashSet;
        
        let mut context = ExtractionContext::new(output_dir.to_string());
        let mut generated_files = Vec::new();
        
        info!("🔄 Processing {} symbols through strategy system", symbols.len());
        
        // Track which modules are being processed
        let mut processed_modules = HashSet::new();
        
        // Register all symbols first for cross-references
        for symbol in &symbols {
            context.register_symbol(symbol.clone());
            processed_modules.insert(symbol.module.clone());
        }
        
        info!("📦 Found {} unique modules to process: {:?}", 
              processed_modules.len(), 
              processed_modules.iter().collect::<Vec<_>>());
        
        // Process each symbol through strategies
        for symbol in symbols {
            self.process_single_symbol(symbol, &mut context)?;
        }
        
        // Call finish_module() for each processed module
        for module_name in &processed_modules {
            debug!("🔧 Finalizing module: {}", module_name);
            for strategy in &mut self.strategies {
                strategy.finish_module(module_name)?;
            }
        }
        
        // Finalize all strategies and collect generated files
        for strategy in &mut self.strategies {
            let mut files = strategy.finish_extraction()?;
            generated_files.append(&mut files);
        }
        
        // Generate main mod.rs file to include all processed modules
        self.update_main_mod_file(output_dir, &processed_modules, &generated_files)?;
        
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
            if strategy.can_handle(&symbol) {
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
    
    /// Update the main src/generated/mod.rs file to include all processed modules
    fn update_main_mod_file(
        &self,
        output_dir: &str,
        _processed_modules: &std::collections::HashSet<String>,
        generated_files: &[GeneratedFile],
    ) -> Result<()> {
        use std::fs;
        use std::collections::HashMap;
        
        // Group generated files by module to identify which modules have files
        let mut modules_with_files = HashMap::new();
        for file in generated_files {
            // Extract module name from file path (e.g., "canon_pm/file.rs" -> "canon_pm")
            if let Some(module_dir) = file.path.split('/').next() {
                if module_dir.ends_with("_pm") {
                    let files_list = modules_with_files.entry(module_dir.to_string()).or_insert_with(Vec::new);
                    // Extract filename without .rs extension
                    if let Some(filename) = file.path.split('/').last() {
                        if let Some(name) = filename.strip_suffix(".rs") {
                            files_list.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        info!("📝 Creating mod.rs files for {} modules", modules_with_files.len());
        
        // Create mod.rs files for each module directory
        for (module_dir, file_list) in &modules_with_files {
            let module_dir_path = Path::new(output_dir).join(module_dir);
            let module_mod_path = module_dir_path.join("mod.rs");
            
            // Ensure the module directory exists
            if let Err(e) = fs::create_dir_all(&module_dir_path) {
                warn!("Failed to create directory {}: {}", module_dir_path.display(), e);
                continue;
            }
            
            let mut content = String::new();
            content.push_str(&format!(
                "//! Generated module for {}\n",
                module_dir.strip_suffix("_pm").unwrap_or(module_dir)
            ));
            content.push_str("//!\n");
            content.push_str("//! This file is auto-generated. Do not edit manually.\n\n");
            
            // Sort files for consistent output
            let mut sorted_files = file_list.clone();
            sorted_files.sort();
            
            // Generate pub mod declarations
            for filename in &sorted_files {
                content.push_str(&format!("pub mod {};\n", filename));
            }
            
            content.push('\n');
            
            // Generate re-exports for commonly used items
            content.push_str("// Re-export commonly used items\n");
            for filename in &sorted_files {
                // Only re-export lookup functions and constants
                if filename.contains("white_balance") || filename.contains("lens") || 
                   filename.contains("quality") || filename.contains("model") {
                    content.push_str(&format!("pub use {}::*;\n", filename));
                }
            }
            
            if let Err(e) = fs::write(&module_mod_path, content) {
                return Err(anyhow::anyhow!(
                    "Failed to write mod.rs file for module '{}' at path '{}': {}",
                    module_dir,
                    module_mod_path.display(),
                    e
                ));
            }
            debug!("📄 Created mod.rs for {} with {} files", module_dir, file_list.len());
        }
        
        // Update main src/generated/mod.rs to include all snake_case modules
        let main_mod_path = Path::new(output_dir).join("mod.rs");
        let mut main_content = String::new();
        
        // Read existing content or create new
        if main_mod_path.exists() {
            main_content = fs::read_to_string(&main_mod_path)?;
        } else {
            main_content.push_str("//! Generated code module\n");
            main_content.push_str("//!\n");
            main_content.push_str("//! This file is automatically generated by codegen.\n");
            main_content.push_str("//! DO NOT EDIT MANUALLY - changes will be overwritten.\n");
            main_content.push_str("//!\n");
            main_content.push_str("//! This module re-exports all generated code for easy access.\n\n");
        }
        
        // Add module declarations for snake_case modules that have mod.rs files
        let mut modules_added = 0;
        for module_dir in modules_with_files.keys() {
            let module_declaration = format!("pub mod {};\n", module_dir);
            if !main_content.contains(&module_declaration) {
                // Insert before existing re-exports
                if let Some(reexport_pos) = main_content.find("// Re-export commonly used types") {
                    main_content.insert_str(reexport_pos, &module_declaration);
                } else {
                    main_content.push_str(&module_declaration);
                }
                modules_added += 1;
            }
        }
        
        if let Err(e) = fs::write(&main_mod_path, main_content) {
            return Err(anyhow::anyhow!(
                "Failed to write main mod.rs file at path '{}': {}",
                main_mod_path.display(),
                e
            ));
        }
        info!("📋 Updated main mod.rs with {} new modules", modules_added);
        
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
        Box::new(CompositeTagStrategy::new()), // Composite tag definitions (MUST be first)
        Box::new(TagKitStrategy::new()),      // Complex tag definitions (Main tables)
        Box::new(BinaryDataStrategy::new()),  // ProcessBinaryData tables (CameraInfo*, etc.)
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