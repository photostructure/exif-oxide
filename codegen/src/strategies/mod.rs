//! Strategy pattern system for processing field extractor output
//!
//! This module provides a strategy-based approach to processing JSON symbols
//! extracted from ExifTool modules, replacing the config-driven extraction
//! system with duck-typing pattern recognition.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, trace, warn};

use crate::field_extractor::FieldSymbol;
use crate::ppi::{ExpressionType, PpiFunctionRegistry};

/// Utility function for strategies to write generated files directly to disk
pub fn write_generated_file(output_dir: &str, relative_path: &str, content: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let full_path = Path::new(output_dir).join(relative_path);

    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write the file
    fs::write(&full_path, content)
        .with_context(|| format!("Failed to write generated file: {}", full_path.display()))?;

    debug!("📝 Written: {} ({} bytes)", relative_path, content.len());
    Ok(())
}

/// Core trait that all extraction strategies implement
pub trait ExtractionStrategy: Send + Sync {
    /// Name of the strategy for logging and debugging
    fn name(&self) -> &'static str;

    /// Check if this strategy can handle the given symbol
    /// Uses duck-typing pattern recognition on symbol structure
    fn can_handle(&self, symbol: &FieldSymbol) -> bool;

    /// Extract data from the symbol and generate appropriate code
    fn extract(&mut self, symbol_data: &FieldSymbol, context: &mut ExtractionContext)
        -> Result<()>;

    /// Finalize processing for a specific module
    fn finish_module(&mut self, module_name: &str) -> Result<()>;

    /// Complete extraction and return generated files
    fn finish_extraction(&mut self, context: &mut ExtractionContext) -> Result<Vec<GeneratedFile>>;
}

/// Context passed to strategies during extraction
#[derive(Debug)]
pub struct ExtractionContext {
    /// Output directory for generated code
    pub output_dir: String,

    /// Global symbol registry for cross-references
    pub symbol_registry: HashMap<String, FieldSymbol>,

    /// Strategy selection log for debugging
    pub strategy_log: Vec<StrategySelection>,

    /// PPI function registry for deduplication
    pub ppi_registry: PpiFunctionRegistry,
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
}

impl ExtractionContext {
    /// Create new extraction context
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            symbol_registry: HashMap::new(),
            strategy_log: Vec::new(),
            ppi_registry: PpiFunctionRegistry::new(),
        }
    }

    /// Log strategy selection decision
    pub fn log_strategy_selection(
        &mut self,
        symbol: &FieldSymbol,
        strategy_name: &str,
        reasoning: &str,
    ) {
        let selection = StrategySelection {
            symbol_name: symbol.name.clone(),
            module_name: symbol.module.clone(),
            strategy_name: strategy_name.to_string(),
            reasoning: reasoning.to_string(),
        };

        debug!(
            "Strategy selection: {} -> {} ({})",
            symbol.name, strategy_name, reasoning
        );
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

impl Default for StrategyDispatcher {
    fn default() -> Self {
        Self::new()
    }
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
        output_dir: &str,
    ) -> Result<Vec<GeneratedFile>> {
        use std::collections::HashSet;
        use std::time::Instant;

        let mut context = ExtractionContext::new(output_dir.to_string());
        let process_start = Instant::now();

        info!(
            "🔄 Processing {} symbols through strategy system",
            symbols.len()
        );
        trace!("🚀 Strategy processing started");

        // Track which modules are being processed
        let mut processed_modules = HashSet::new();

        // Register all symbols first for cross-references
        let register_start = Instant::now();
        trace!(
            "📋 Registering {} symbols for cross-references",
            symbols.len()
        );
        for symbol in &symbols {
            context.register_symbol(symbol.clone());
            processed_modules.insert(symbol.module.clone());
        }
        let register_time = register_start.elapsed();
        trace!(
            "⏱️  Symbol registration completed in {:.2}ms",
            register_time.as_millis()
        );

        info!(
            "📦 Found {} unique modules to process: {:?}",
            processed_modules.len(),
            processed_modules.iter().collect::<Vec<_>>()
        );

        // Process each symbol through strategies
        let symbol_processing_start = Instant::now();
        trace!("🚀 Starting individual symbol processing");
        let mut processed_count = 0;
        for symbol in symbols {
            self.process_single_symbol(symbol, &mut context)?;
            processed_count += 1;
            if processed_count % 100 == 0 {
                trace!("📊 Processed {} symbols so far", processed_count);
            }
        }
        let symbol_processing_time = symbol_processing_start.elapsed();
        trace!(
            "⏱️  Symbol processing completed in {:.2}ms",
            symbol_processing_time.as_millis()
        );

        // Call finish_module() for each processed module
        let finalize_start = Instant::now();
        trace!("🔄 Finalizing {} modules", processed_modules.len());
        for module_name in &processed_modules {
            debug!("🔧 Finalizing module: {}", module_name);
            let module_finalize_start = Instant::now();
            for strategy in &mut self.strategies {
                strategy.finish_module(module_name)?;
            }
            trace!(
                "⏱️  Module {} finalized in {:.2}ms",
                module_name,
                module_finalize_start.elapsed().as_millis()
            );
        }
        let finalize_time = finalize_start.elapsed();
        trace!(
            "⏱️  All modules finalized in {:.2}ms",
            finalize_time.as_millis()
        );

        // Finalize all strategies and collect generated files
        let extraction_finalize_start = Instant::now();
        trace!("🏁 Finalizing {} strategies", self.strategies.len());
        let mut generated_files = Vec::new();
        for strategy in &mut self.strategies {
            let strategy_finalize_start = Instant::now();
            let files = strategy.finish_extraction(&mut context)?;
            let strategy_finalize_time = strategy_finalize_start.elapsed();
            trace!(
                "⏱️  Strategy '{}' finalized in {:.2}ms, generated {} files",
                strategy.name(),
                strategy_finalize_time.as_millis(),
                files.len()
            );

            // Write files immediately as they're generated
            for file in &files {
                write_generated_file(output_dir, &file.path, &file.content)?;
            }

            generated_files.extend(files);
        }

        // Generate AST function files after all strategies complete
        let ast_generation_start = Instant::now();
        trace!("🔧 Generating AST function files");
        let ast_files = context.ppi_registry.generate_function_files()?;
        let ast_generation_time = ast_generation_start.elapsed();
        trace!(
            "⏱️  AST function generation completed in {:.2}ms, generated {} files",
            ast_generation_time.as_millis(),
            ast_files.len()
        );

        // Write AST function files
        for file in &ast_files {
            write_generated_file(output_dir, &file.path, &file.content)?;
        }
        generated_files.extend(ast_files);

        let extraction_finalize_time = extraction_finalize_start.elapsed();
        trace!(
            "⏱️  All strategies finalized in {:.2}ms",
            extraction_finalize_time.as_millis()
        );

        // Note: mod.rs generation moved to main.rs after file writing

        // Write strategy selection log for debugging
        let log_start = Instant::now();
        trace!("📋 Writing strategy selection log");
        self.write_strategy_log(&context, output_dir)?;
        let log_time = log_start.elapsed();
        trace!("⏱️  Strategy log written in {:.2}ms", log_time.as_millis());

        let total_process_time = process_start.elapsed();
        info!(
            "✅ Strategy processing complete: {} files generated in {:.2}ms",
            generated_files.len(),
            total_process_time.as_millis()
        );

        // Detailed timing breakdown
        trace!("📊 Strategy processing time breakdown:");
        trace!(
            "  • Symbol registration: {:.1}ms ({:.1}%)",
            register_time.as_millis(),
            (register_time.as_millis() as f64 / total_process_time.as_millis() as f64) * 100.0
        );
        trace!(
            "  • Symbol processing: {:.1}ms ({:.1}%)",
            symbol_processing_time.as_millis(),
            (symbol_processing_time.as_millis() as f64 / total_process_time.as_millis() as f64)
                * 100.0
        );
        trace!(
            "  • Module finalization: {:.1}ms ({:.1}%)",
            finalize_time.as_millis(),
            (finalize_time.as_millis() as f64 / total_process_time.as_millis() as f64) * 100.0
        );
        trace!(
            "  • Strategy finalization: {:.1}ms ({:.1}%)",
            extraction_finalize_time.as_millis(),
            (extraction_finalize_time.as_millis() as f64 / total_process_time.as_millis() as f64)
                * 100.0
        );
        // Mod file update timing moved to main.rs
        trace!(
            "  • Log writing: {:.1}ms ({:.1}%)",
            log_time.as_millis(),
            (log_time.as_millis() as f64 / total_process_time.as_millis() as f64) * 100.0
        );

        Ok(generated_files)
    }

    /// Process a single symbol through the first matching strategy
    fn process_single_symbol(
        &mut self,
        symbol: FieldSymbol,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        let single_symbol_start = std::time::Instant::now();
        trace!(
            "🔍 Processing symbol: {} ({})",
            symbol.name,
            symbol.symbol_type
        );

        // Find the first matching strategy
        let mut matched_strategy_index = None;
        let mut reasoning = String::new();
        let strategy_match_start = std::time::Instant::now();

        for (index, strategy) in self.strategies.iter().enumerate() {
            if strategy.can_handle(&symbol) {
                reasoning = format!("Pattern matched: {}", self.describe_pattern(&symbol.data));
                matched_strategy_index = Some(index);
                trace!(
                    "✓ Strategy '{}' matched for symbol '{}'",
                    strategy.name(),
                    symbol.name
                );
                break;
            }
        }
        let strategy_match_time = strategy_match_start.elapsed();

        // Process with the matched strategy
        if let Some(index) = matched_strategy_index {
            let strategy = &mut self.strategies[index];
            context.log_strategy_selection(&symbol, strategy.name(), &reasoning);

            let extract_start = std::time::Instant::now();
            let result = strategy.extract(&symbol, context);
            let extract_time = extract_start.elapsed();

            let total_time = single_symbol_start.elapsed();
            trace!(
                "⏱️  Symbol '{}' processed by '{}' in {:.2}ms (match: {:.2}ms, extract: {:.2}ms)",
                symbol.name,
                strategy.name(),
                total_time.as_millis(),
                strategy_match_time.as_millis(),
                extract_time.as_millis()
            );

            result
        } else {
            // No strategy could handle this symbol
            let total_time = single_symbol_start.elapsed();
            info!(
                "⚠️  No strategy found for symbol: {} (type: {}, module: {}) after {:.2}ms",
                symbol.name,
                symbol.symbol_type,
                symbol.module,
                total_time.as_millis()
            );

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

        // Add PPI processing statistics at the end
        log_content.push_str("\n# PPI Processing Statistics\n");
        log_content.push_str(
            "# Shows how many conversions were successfully processed by PPI vs other methods\n\n",
        );

        let registry_stats = context.ppi_registry.stats();
        let conversion_stats = &registry_stats.conversion_stats;

        // PrintConv statistics
        let print_conv_rate = conversion_stats.success_rate(ExpressionType::PrintConv);
        log_content.push_str(&format!(
            "PrintConv: {:.1}% ({}/{}) processed by PPI\n",
            print_conv_rate,
            conversion_stats.print_conv_ppi_successes,
            conversion_stats.print_conv_attempts
        ));

        // ValueConv statistics
        let value_conv_rate = conversion_stats.success_rate(ExpressionType::ValueConv);
        log_content.push_str(&format!(
            "ValueConv: {:.1}% ({}/{}) processed by PPI\n",
            value_conv_rate,
            conversion_stats.value_conv_ppi_successes,
            conversion_stats.value_conv_attempts
        ));

        // Condition statistics
        let condition_rate = conversion_stats.success_rate(ExpressionType::Condition);
        log_content.push_str(&format!(
            "Condition: {:.1}% ({}/{}) processed by PPI\n",
            condition_rate,
            conversion_stats.condition_ppi_successes,
            conversion_stats.condition_attempts
        ));

        // Overall summary
        let total_attempts = conversion_stats.print_conv_attempts
            + conversion_stats.value_conv_attempts
            + conversion_stats.condition_attempts;
        let total_successes = conversion_stats.print_conv_ppi_successes
            + conversion_stats.value_conv_ppi_successes
            + conversion_stats.condition_ppi_successes;

        if total_attempts > 0 {
            let overall_rate = (total_successes as f64 / total_attempts as f64) * 100.0;
            log_content.push_str(&format!(
                "\nOverall: {:.1}% ({}/{}) conversions processed by PPI\n",
                overall_rate, total_successes, total_attempts
            ));
        }

        fs::write(log_path, log_content)?;
        debug!("📋 Strategy selection log written to strategy_selection.log");

        Ok(())
    }

    /// Update the main src/generated/mod.rs file to include all processed modules
    #[allow(dead_code)]
    fn update_main_mod_file(&self, output_dir: &str) -> Result<()> {
        use std::collections::BTreeSet;
        use std::fs;

        // Scan filesystem directly to find all module directories and their .rs files
        let mut modules_with_files = std::collections::HashMap::new();

        // Read all entries in the output directory
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process directories (skip files like composite_tags.rs)
            if path.is_dir() {
                let module_name = path.file_name().unwrap().to_string_lossy().to_string();

                // Skip non-module directories
                if module_name == "file_types" && !path.join("mod.rs").exists() {
                    continue;
                }

                // Find all .rs files in this module directory
                let mut file_set = BTreeSet::new();
                for module_file in fs::read_dir(&path)? {
                    let module_file = module_file?;
                    let file_path = module_file.path();

                    // Only include .rs files, excluding mod.rs itself
                    if file_path.extension().is_some_and(|ext| ext == "rs") {
                        if let Some(filename) = file_path.file_stem() {
                            let filename_str = filename.to_string_lossy().to_string();
                            if filename_str != "mod" {
                                file_set.insert(filename_str);
                            }
                        }
                    }
                }

                if !file_set.is_empty() {
                    modules_with_files.insert(module_name, file_set);
                }
            }
        }

        info!(
            "📝 Creating mod.rs files for {} modules (filesystem scan)",
            modules_with_files.len()
        );

        // Create mod.rs files for each module directory
        for (module_dir, file_set) in &modules_with_files {
            let module_dir_path = Path::new(output_dir).join(module_dir);
            let module_mod_path = module_dir_path.join("mod.rs");

            // Ensure the module directory exists
            if let Err(e) = fs::create_dir_all(&module_dir_path) {
                warn!(
                    "Failed to create directory {}: {}",
                    module_dir_path.display(),
                    e
                );
                continue;
            }

            let mut content = String::new();
            content.push_str(&format!(
                "//! Generated module for {}\n",
                module_dir.strip_suffix("_pm").unwrap_or(module_dir)
            ));
            content.push_str("//!\n");
            content.push_str("//! This file is auto-generated by codegen/src/strategies/mod.rs. Do not edit manually.\n\n");

            // Files are already deduplicated and sorted from BTreeSet
            // Generate pub mod declarations
            for filename in file_set {
                content.push_str(&format!("pub mod {filename};\n"));
            }

            content.push('\n');

            // Generate re-exports for commonly used items
            content.push_str("// Re-export commonly used items\n");

            // Always re-export main_tags if it exists
            if file_set.contains("main_tags") {
                // Read the actual constant name from the generated main_tags.rs file
                let main_tags_path = module_dir_path.join("main_tags.rs");
                if let Ok(main_tags_content) = fs::read_to_string(&main_tags_path) {
                    // Look for the pattern "pub static CONSTANT_NAME: LazyLock"
                    if let Some(start) = main_tags_content.find("pub static ") {
                        // Find the colon after the constant name (more robust pattern matching)
                        if let Some(colon_pos) = main_tags_content[start..].find(':') {
                            let constant_def = &main_tags_content[start..start + colon_pos];
                            if let Some(const_name) = constant_def.strip_prefix("pub static ") {
                                // Trim any whitespace from the constant name
                                let const_name = const_name.trim();
                                debug!(
                                    "Found MAIN_TAGS constant: '{}' in module {}",
                                    const_name, module_dir
                                );
                                content.push_str(&format!("pub use main_tags::{const_name};\n"));
                            } else {
                                debug!(
                                    "Failed to parse constant name from: '{}' in module {}",
                                    constant_def, module_dir
                                );
                                // Fallback: Use module-prefixed name based on module directory
                                let module_upper = module_dir.to_uppercase().replace('-', "_");
                                let expected_const = format!("{module_upper}_MAIN_TAGS");
                                debug!("Using fallback constant name: {}", expected_const);
                                content
                                    .push_str(&format!("pub use main_tags::{expected_const};\n"));
                            }
                        } else {
                            debug!("No colon found after 'pub static' in module {}", module_dir);
                            // Fallback: Use module-prefixed name
                            let module_upper = module_dir.to_uppercase().replace('-', "_");
                            let expected_const = format!("{module_upper}_MAIN_TAGS");
                            content.push_str(&format!("pub use main_tags::{expected_const};\n"));
                        }
                    } else {
                        debug!(
                            "No 'pub static' found in main_tags.rs for module {}",
                            module_dir
                        );
                        // Fallback: Use module-prefixed name
                        let module_upper = module_dir.to_uppercase().replace('-', "_");
                        let expected_const = format!("{module_upper}_MAIN_TAGS");
                        content.push_str(&format!("pub use main_tags::{expected_const};\n"));
                    }
                } else {
                    debug!("Failed to read main_tags.rs for module {}", module_dir);
                    // Fallback: Use module-prefixed name
                    let module_upper = module_dir.to_uppercase().replace('-', "_");
                    let expected_const = format!("{module_upper}_MAIN_TAGS");
                    content.push_str(&format!("pub use main_tags::{expected_const};\n"));
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
            debug!(
                "📄 Created mod.rs for {} with {} files",
                module_dir,
                file_set.len()
            );
        }

        // Completely regenerate main src/generated/mod.rs based purely on generated modules
        let main_mod_path = Path::new(output_dir).join("mod.rs");
        let mut main_content = String::new();

        // Generate header from scratch - no manual edits allowed
        main_content.push_str("//! Generated code module\n");
        main_content.push_str("//!\n");
        main_content.push_str("//! This file is auto-generated by codegen/src/strategies/mod.rs. Do not edit manually.\n");
        main_content.push_str("//!\n");
        main_content.push_str("//! This module re-exports all generated code for easy access.\n\n");

        // Collect all modules in a BTreeSet for deterministic, sorted output
        let mut all_modules = BTreeSet::new();

        // Add all generated modules
        for module_dir in modules_with_files.keys() {
            all_modules.insert(module_dir.clone());
        }

        // Add standalone files if they exist
        if Path::new(output_dir)
            .join("file_types")
            .join("mod.rs")
            .exists()
        {
            all_modules.insert("file_types".to_string());
        }
        if Path::new(output_dir).join("composite_tags.rs").exists() {
            all_modules.insert("composite_tags".to_string());
        }
        if Path::new(output_dir).join("supported_tags.rs").exists() {
            all_modules.insert("supported_tags".to_string());
        }

        // Generate module declarations in sorted order
        for module_dir in &all_modules {
            // Add special attribute for non-snake-case modules
            main_content.push_str(&format!("pub mod {module_dir};\n"));
        }

        main_content.push('\n');

        // Add standard re-exports (only if modules exist)
        main_content.push_str("// Re-export commonly used types and functions\n");
        if Path::new(output_dir).join("supported_tags.rs").exists() {
            main_content.push_str("pub use supported_tags::{\n");
            main_content.push_str("    SUPPORTED_TAG_COUNT, SUPPORTED_COMPOSITE_TAG_COUNT, TOTAL_SUPPORTED_TAG_COUNT,\n");
            main_content.push_str("    SUPPORTED_TAG_NAMES, SUPPORTED_COMPOSITE_TAG_NAMES,\n");
            main_content.push_str("    tag_counts_by_group, supported_tag_summary\n");
            main_content.push_str("};\n");
        }
        if Path::new(output_dir).join("composite_tags.rs").exists() {
            main_content.push_str("pub use composite_tags::{CompositeTagDef, COMPOSITE_TAGS, lookup_composite_tag, all_composite_tag_names, composite_tag_count};\n");
        }
        main_content.push('\n');

        main_content.push_str("/// Initialize all lazy static data structures\n");
        main_content.push_str(
            "/// This can be called during startup to avoid lazy initialization costs later\n",
        );
        main_content.push_str("pub fn initialize_all() {\n");
        main_content.push_str("}\n");

        let modules_added = all_modules.len();

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

// StrategyDispatcher no longer implements Default since it requires output_dir

// Strategy implementations
mod binary_data;
mod boolean_set;
mod composite_tag;
mod file_type_lookup;
mod magic_numbers;
mod mime_type;
mod scalar_array;
mod simple_table;
mod tag_kit;

// Output location utilities
pub mod output_locations;

// Re-export strategy implementations
pub use binary_data::BinaryDataStrategy;
pub use boolean_set::BooleanSetStrategy;
pub use composite_tag::CompositeTagStrategy;
pub use file_type_lookup::FileTypeLookupStrategy;
pub use magic_numbers::MagicNumberStrategy;
pub use mime_type::MimeTypeStrategy;
pub use scalar_array::ScalarArrayStrategy;
pub use simple_table::SimpleTableStrategy;
pub use tag_kit::TagKitStrategy;

/// Registry of all available strategies
/// Ordered by precedence - first-match-wins
pub fn all_strategies() -> Vec<Box<dyn ExtractionStrategy>> {
    vec![
        // Order matters: first-match wins
        Box::new(CompositeTagStrategy::new()), // Composite tag definitions (MUST be first)
        // File type detection strategies (MUST be before TagKitStrategy)
        Box::new(FileTypeLookupStrategy::new()), // ExifTool %fileTypeLookup discriminated union
        Box::new(MagicNumberStrategy::new()),    // ExifTool %magicNumber regex patterns
        Box::new(MimeTypeStrategy::new()),       // ExifTool %mimeType simple mappings
        // Simple lookup tables (MUST be before TagKitStrategy to claim mixed-key tables like canonLensTypes)
        Box::new(SimpleTableStrategy::new()), // Simple key-value lookups with mixed keys
        // Scalar arrays (MUST be before TagKitStrategy to handle arrays of primitives)
        Box::new(ScalarArrayStrategy::new()), // Arrays of scalars (u8[], i32[], &str[])
        Box::new(TagKitStrategy::new()), // Complex tag definitions (Main tables) - after specific patterns
        Box::new(BinaryDataStrategy::new()), // ProcessBinaryData tables (CameraInfo*, etc.)
        Box::new(BooleanSetStrategy::new()), // Membership sets (isDat*, isTxt*, etc.)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_strategy_dispatcher_creation() {
        let dispatcher = StrategyDispatcher::new();
        assert_eq!(dispatcher.strategies.len(), 9); // All 9 strategies registered
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
                is_composite_table: 0,
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
        assert_eq!(
            dispatcher.describe_pattern(&json!({"a": "1", "b": "2"})),
            "string map (2 keys)"
        );
        assert_eq!(
            dispatcher.describe_pattern(&json!({"PrintConv": "test"})),
            "tag definition with conversions"
        );
        assert_eq!(
            dispatcher.describe_pattern(&json!([1, 2, 3])),
            "array (3 elements)"
        );
        assert_eq!(dispatcher.describe_pattern(&json!("test")), "string scalar");
    }
}
