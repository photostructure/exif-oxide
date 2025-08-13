//! # Composite Tag Strategy - Runtime Metadata Generation
//!
//! Processes ExifTool composite tag definitions to generate `CompositeTagDef` registry
//! structures for runtime dynamic evaluation.
//!
//! ## Architecture Role
//!
//! - **Input**: FieldSymbol data from ExifTool Composite tables with Require/Desire dependencies
//! - **Filters**: Accepts composite table symbols, rejects regular tables (handled by TagKitStrategy)
//! - **Processing**: Extracts dependencies, inhibit lists, ValueConv/PrintConv expressions
//! - **Output**: Generated `CompositeTagDef` constants and lookup registry
//! - **Runtime**: Composite tags use dynamic evaluation with dependency resolution
//!
//! ## Why Runtime Generation vs Compile-time?
//!
//! Composite tags have dynamic dependencies that can't be resolved until runtime:
//! - **Require/Desire arrays**: Tags that may or may not exist in any given file
//! - **`$val[n]` expressions**: Array access patterns that depend on resolved dependencies
//! - **Conditional logic**: Expressions like `$val[1] =~ /^S/i ? -$val[0] : $val[0]`
//! - **Context queries**: Access to `$$self{Field}` processor state during evaluation
//!
//! ## Generated Registry Structure
//!
//! 1. **Individual Constants**: `COMPOSITE_GPS_GPSLATITUDE` with all metadata
//! 2. **Global Registry**: `COMPOSITE_TAGS` HashMap for runtime lookup
//! 3. **Raw Expressions**: ValueConv/PrintConv stored as Perl strings for evaluation
//! 4. **Dependency Arrays**: Require/Desire lists for runtime resolution
//!
//! This strategy is part of the hybrid compile-time/runtime architecture documented in
//! [`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) under "Expression Evaluation Architecture".

use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::{ExtractionContext, ExtractionStrategy, GeneratedFile};
use crate::common::utils::format_rust_string;
use crate::field_extractor::FieldSymbol;

/// Strategy for processing composite tag definitions  
pub struct CompositeTagStrategy {
    /// Composite symbols from field extractor
    composite_symbols: Vec<FieldSymbol>,
}

/// Parsed composite tag definition from field extractor
#[derive(Debug, Clone)]
struct CompositeDefinition {
    name: String,
    module: String,
    require: Vec<String>,
    desire: Vec<String>,
    inhibit: Vec<String>,
    value_conv: Option<String>,
    print_conv: Option<String>,
    description: Option<String>,
    groups: HashMap<String, String>,
}

impl Default for CompositeTagStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeTagStrategy {
    pub fn new() -> Self {
        Self {
            composite_symbols: Vec::new(),
        }
    }

    /// Check if symbol contains composite tag characteristics
    fn is_composite_symbol(symbol: &FieldSymbol) -> bool {
        // Strategy Routing Decision Tree:
        // 1. Is it marked as composite table? → Accept for CompositeTagStrategy
        // 2. Otherwise → Reject (TagKitStrategy or other strategies handle it)
        //
        // The is_composite_table flag is set by our ExifTool patcher when it detects
        // AddCompositeTags calls, ensuring clean separation between strategy types.

        // Simply trust the metadata flag set by our patcher when AddCompositeTags was called
        symbol.metadata.is_composite_table == 1
    }

    /// Extract composite tags from field extractor symbol data
    fn extract_composite_definitions(
        &self,
        symbol: &FieldSymbol,
    ) -> Result<Vec<CompositeDefinition>> {
        debug!(
            "Processing composite symbol: {}::{}",
            symbol.module, symbol.name
        );
        debug!(
            "Symbol data keys: {:?}",
            symbol
                .data
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>())
        );

        let mut definitions = Vec::new();

        // Get the composite data from the field extractor symbol
        // The composite definitions are directly in symbol.data (not in symbol.data["data"])
        // This matches the structure from field extractor JSON output
        let data = symbol
            .data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Symbol data is not an object"))?;

        // Process each composite tag definition
        for (tag_name, tag_def) in data.iter() {
            // Skip non-tag entries (like GROUPS)
            if tag_name == "GROUPS" {
                continue;
            }

            if let Some(tag_obj) = tag_def.as_object() {
                // Check if this is a composite tag (has IsComposite = 1)
                if tag_obj.get("IsComposite").and_then(|v| v.as_i64()) == Some(1) {
                    definitions.push(self.parse_composite_definition(tag_name, tag_obj)?);
                }
            }
        }

        Ok(definitions)
    }

    /// Parse a composite tag definition from ExifTool field extractor output
    fn parse_composite_definition(
        &self,
        tag_name: &str,
        tag_obj: &serde_json::Map<String, JsonValue>,
    ) -> Result<CompositeDefinition> {
        let mut definition = CompositeDefinition {
            name: tag_name.to_string(),
            module: String::new(), // Will be set later
            require: Vec::new(),
            desire: Vec::new(),
            inhibit: Vec::new(),
            value_conv: None,
            print_conv: None,
            description: None,
            groups: HashMap::new(),
        };

        // Parse Require dependencies
        if let Some(require_obj) = tag_obj.get("Require").and_then(|r| r.as_object()) {
            for (_, tag_name) in require_obj.iter() {
                if let Some(tag_str) = tag_name.as_str() {
                    definition.require.push(tag_str.to_string());
                }
            }
        }

        // Parse Desire dependencies
        if let Some(desire_obj) = tag_obj.get("Desire").and_then(|d| d.as_object()) {
            for (_, tag_name) in desire_obj.iter() {
                if let Some(tag_str) = tag_name.as_str() {
                    definition.desire.push(tag_str.to_string());
                }
            }
        }

        // Parse Inhibit dependencies
        if let Some(inhibit_obj) = tag_obj.get("Inhibit").and_then(|i| i.as_object()) {
            for (_, tag_name) in inhibit_obj.iter() {
                if let Some(tag_str) = tag_name.as_str() {
                    definition.inhibit.push(tag_str.to_string());
                }
            }
        }

        // Extract ValueConv expression
        if let Some(value_conv) = tag_obj.get("ValueConv").and_then(|v| v.as_str()) {
            definition.value_conv = Some(value_conv.to_string());
        }

        // Extract PrintConv expression
        if let Some(print_conv) = tag_obj.get("PrintConv").and_then(|p| p.as_str()) {
            definition.print_conv = Some(print_conv.to_string());
        }

        // Extract description
        if let Some(desc) = tag_obj.get("Description").and_then(|d| d.as_str()) {
            definition.description = Some(desc.to_string());
        }

        // Parse Groups
        if let Some(groups_obj) = tag_obj.get("Groups").and_then(|g| g.as_object()) {
            for (group_num, group_name) in groups_obj.iter() {
                if let Some(name_str) = group_name.as_str() {
                    definition
                        .groups
                        .insert(group_num.to_string(), name_str.to_string());
                }
            }
        }

        Ok(definition)
    }

    /// Generate the main composite_tags.rs module with CompositeTagDef structures
    fn generate_composite_tags_module(
        &self,
        definitions: &[CompositeDefinition],
    ) -> Result<String> {
        let mut code = String::new();

        // File header
        code.push_str("//! Generated composite tag definitions and registry\n");
        code.push_str("//!\n");
        code.push_str("//! This file is auto-generated by codegen/src/strategies/composite_tag.rs. Do not edit manually.\n");
        code.push_str("//!\n");
        code.push_str("//! This module provides:\n");
        code.push_str("//! - CompositeTagDef: Structure defining composite tag dependencies and calculation logic\n");
        code.push_str("//! - COMPOSITE_TAGS: Global registry of all composite tag definitions\n");
        code.push('\n');

        // Imports
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::LazyLock;\n");
        code.push('\n');

        // CompositeTagDef structure
        code.push_str(
            "/// Definition of a composite tag with dependencies and calculation logic\n",
        );
        code.push_str("///\n");
        code.push_str(
            "/// Mirrors ExifTool's composite tag structure from AddCompositeTags() function.\n",
        );
        code.push_str("/// See: third-party/exiftool/lib/Image/ExifTool.pm:5662-5720\n");
        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct CompositeTagDef {\n");
        code.push_str("    /// Tag name (e.g., \"GPSLatitude\", \"ImageSize\")\n");
        code.push_str("    pub name: &'static str,\n");
        code.push_str("    \n");
        code.push_str("    /// Source module (e.g., \"GPS\", \"Exif\", \"Canon\")\n");
        code.push_str("    pub module: &'static str,\n");
        code.push_str("    \n");
        code.push_str("    /// Required tag dependencies that must exist\n");
        code.push_str("    pub require: &'static [&'static str],\n");
        code.push_str("    \n");
        code.push_str("    /// Optional tag dependencies that enhance calculation\n");
        code.push_str("    pub desire: &'static [&'static str],\n");
        code.push_str("    \n");
        code.push_str("    /// Tags that inhibit this composite if present\n");
        code.push_str("    pub inhibit: &'static [&'static str],\n");
        code.push_str("    \n");
        code.push_str("    /// ExifTool ValueConv expression for calculation\n");
        code.push_str("    pub value_conv: Option<&'static str>,\n");
        code.push_str("    \n");
        code.push_str("    /// ExifTool PrintConv expression for formatting\n");
        code.push_str("    pub print_conv: Option<&'static str>,\n");
        code.push_str("    \n");
        code.push_str("    /// Human-readable description\n");
        code.push_str("    pub description: Option<&'static str>,\n");
        code.push_str("    \n");
        code.push_str("    /// Group assignments by family number\n");
        code.push_str("    pub groups: &'static [(u8, &'static str)],\n");
        code.push_str("}\n");
        code.push('\n');

        // Generate individual composite tag definitions
        for def in definitions {
            // Include module name to prevent conflicts from same tag name across different modules
            // e.g., LensID from Exif -> COMPOSITE_EXIF_LENSID, LensID from Nikon -> COMPOSITE_NIKON_LENSID
            let safe_module = def.module.to_uppercase();
            let safe_tag = def.name.replace([':', '-'], "_").to_uppercase();
            let safe_name = format!("{safe_module}_{safe_tag}");

            code.push_str(&format!(
                "/// {} composite tag definition from {} module\n",
                def.name, def.module
            ));
            if let Some(desc) = &def.description {
                code.push_str(&format!("/// {desc}\n"));
            }
            code.push_str(&format!(
                "pub static COMPOSITE_{safe_name}: CompositeTagDef = CompositeTagDef {{\n"
            ));
            code.push_str(&format!("    name: \"{}\",\n", def.name));
            code.push_str(&format!("    module: \"{}\",\n", def.module));

            // Require dependencies
            if def.require.is_empty() {
                code.push_str("    require: &[],\n");
            } else {
                code.push_str("    require: &[\n");
                for req in &def.require {
                    code.push_str(&format!("        \"{req}\",\n"));
                }
                code.push_str("    ],\n");
            }

            // Desire dependencies
            if def.desire.is_empty() {
                code.push_str("    desire: &[],\n");
            } else {
                code.push_str("    desire: &[\n");
                for des in &def.desire {
                    code.push_str(&format!("        \"{des}\",\n"));
                }
                code.push_str("    ],\n");
            }

            // Inhibit dependencies
            if def.inhibit.is_empty() {
                code.push_str("    inhibit: &[],\n");
            } else {
                code.push_str("    inhibit: &[\n");
                for inh in &def.inhibit {
                    code.push_str(&format!("        \"{inh}\",\n"));
                }
                code.push_str("    ],\n");
            }

            // ValueConv
            match &def.value_conv {
                Some(conv) => {
                    // Escape the Perl expression properly
                    let escaped = format_rust_string(conv);
                    code.push_str(&format!("    value_conv: Some({escaped}),\n"));
                }
                None => code.push_str("    value_conv: None,\n"),
            }

            // PrintConv
            match &def.print_conv {
                Some(conv) => {
                    let escaped = format_rust_string(conv);
                    code.push_str(&format!("    print_conv: Some({escaped}),\n"));
                }
                None => code.push_str("    print_conv: None,\n"),
            }

            // Description
            match &def.description {
                Some(desc) => {
                    let escaped = desc.replace('"', "\\\"");
                    code.push_str(&format!("    description: Some(\"{escaped}\"),\n"));
                }
                None => code.push_str("    description: None,\n"),
            }

            // Groups
            if def.groups.is_empty() {
                code.push_str("    groups: &[],\n");
            } else {
                code.push_str("    groups: &[\n");
                let mut sorted_groups: Vec<_> = def.groups.iter().collect();
                sorted_groups.sort_by_key(|&(k, _)| k.parse::<u8>().unwrap_or(0));
                for (family, name) in sorted_groups {
                    if let Ok(family_num) = family.parse::<u8>() {
                        code.push_str(&format!("        ({family_num}, \"{name}\"),\n"));
                    }
                }
                code.push_str("    ],\n");
            }

            code.push_str("};\n");
            code.push('\n');
        }

        // Generate COMPOSITE_TAGS registry
        code.push_str("/// Global registry of all composite tag definitions\n");
        code.push_str("///\n");
        code.push_str("/// Maps composite tag names to their definitions for runtime lookup.\n");
        code.push_str("/// Populated from all ExifTool modules with composite tables.\n");
        code.push_str("pub static COMPOSITE_TAGS: LazyLock<HashMap<&'static str, &'static CompositeTagDef>> = LazyLock::new(|| {\n");

        // Collect and sort entries for deterministic output
        let mut entries = Vec::new();
        for def in definitions {
            // Use same naming scheme as static variable generation
            let safe_module = def.module.to_uppercase();
            let safe_tag = def.name.replace([':', '-'], "_").to_uppercase();
            let safe_name = format!("{safe_module}_{safe_tag}");
            entries.push((def.name.clone(), safe_name));
        }

        // Sort by tag name for deterministic output
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        if entries.is_empty() {
            code.push_str("    HashMap::new()\n");
        } else {
            code.push_str("    HashMap::from([\n");
            for (tag_name, safe_name) in entries {
                code.push_str(&format!(
                    "        (\"{tag_name}\", &COMPOSITE_{safe_name}),\n"
                ));
            }
            code.push_str("    ])\n");
        }

        code.push_str("});\n");
        code.push('\n');

        // Helper functions
        code.push_str("/// Look up a composite tag definition by name\n");
        code.push_str(
            "pub fn lookup_composite_tag(name: &str) -> Option<&'static CompositeTagDef> {\n",
        );
        code.push_str("    COMPOSITE_TAGS.get(name).copied()\n");
        code.push_str("}\n");
        code.push('\n');

        code.push_str("/// Get all composite tag names\n");
        code.push_str("pub fn all_composite_tag_names() -> Vec<&'static str> {\n");
        code.push_str("    COMPOSITE_TAGS.keys().copied().collect()\n");
        code.push_str("}\n");
        code.push('\n');

        code.push_str(&format!(
            "/// Total number of composite tags: {}\n",
            definitions.len()
        ));
        code.push_str("pub fn composite_tag_count() -> usize {\n");
        code.push_str("    COMPOSITE_TAGS.len()\n");
        code.push_str("}\n");

        Ok(code)
    }
}

impl ExtractionStrategy for CompositeTagStrategy {
    fn name(&self) -> &'static str {
        "CompositeTagStrategy"
    }

    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        Self::is_composite_symbol(symbol)
    }

    fn extract(
        &mut self,
        symbol_data: &FieldSymbol,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        context.log_strategy_selection(
            symbol_data,
            self.name(),
            "Detected Composite table or symbols with Require/Desire dependencies",
        );

        debug!(
            "Storing composite symbol: {}::{}",
            symbol_data.module, symbol_data.name
        );
        self.composite_symbols.push(symbol_data.clone());

        Ok(())
    }

    fn finish_module(&mut self, _module_name: &str) -> Result<()> {
        // Nothing to do per-module
        Ok(())
    }

    fn finish_extraction(
        &mut self,
        _context: &mut ExtractionContext,
    ) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        info!(
            "Processing {} composite symbols",
            self.composite_symbols.len()
        );

        // Debug: Log first composite symbol to see its structure
        if let Some(first_symbol) = self.composite_symbols.first() {
            debug!("First composite symbol structure:");
            debug!(
                "  Module: {}, Name: {}",
                first_symbol.module, first_symbol.name
            );
            debug!(
                "  Data keys: {:?}",
                first_symbol
                    .data
                    .as_object()
                    .map(|o| o.keys().collect::<Vec<_>>())
            );
            debug!(
                "  Full data: {}",
                serde_json::to_string_pretty(&first_symbol.data)
                    .unwrap_or_else(|_| "Failed to serialize".to_string())
            );
        }

        // Collect all composite definitions from all modules
        let mut all_definitions = Vec::new();

        for symbol in &self.composite_symbols {
            // TEMP: Debug symbol structure before attempting extraction
            debug!(
                "Processing stored composite symbol: {}::{}",
                symbol.module, symbol.name
            );
            debug!(
                "Symbol data keys: {:?}",
                symbol
                    .data
                    .as_object()
                    .map(|o| o.keys().collect::<Vec<_>>())
            );

            // Check if it has the expected "data" field
            if let Some(data_field) = symbol.data.get("data") {
                debug!(
                    "Found 'data' field in symbol, type: {}",
                    if data_field.is_object() {
                        "object"
                    } else if data_field.is_array() {
                        "array"
                    } else if data_field.is_string() {
                        "string"
                    } else {
                        "other"
                    }
                );

                if let Some(data_obj) = data_field.as_object() {
                    debug!(
                        "Data field has {} keys: {:?}",
                        data_obj.len(),
                        data_obj.keys().collect::<Vec<_>>()
                    );

                    // Check for composite indicators
                    let composite_count = data_obj
                        .values()
                        .filter(|v| {
                            v.as_object().is_some_and(|obj| {
                                obj.get("IsComposite").and_then(|ic| ic.as_i64()) == Some(1)
                            })
                        })
                        .count();
                    debug!("Found {} composite tags in this symbol", composite_count);
                }
            } else {
                debug!("No 'data' field found in symbol");
                debug!(
                    "Available top-level keys: {:?}",
                    symbol
                        .data
                        .as_object()
                        .map(|o| o.keys().collect::<Vec<_>>())
                );
            }

            // Now try the actual extraction to see the error
            match self.extract_composite_definitions(symbol) {
                Ok(mut defs) => {
                    // Set module name for each definition
                    for def in &mut defs {
                        def.module = symbol.module.clone();
                    }
                    let count = defs.len();
                    all_definitions.extend(defs);
                    debug!(
                        "Extracted {} composite definitions from {}::{}",
                        count, symbol.module, symbol.name
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to extract composite definitions from {}::{}: {}",
                        symbol.module, symbol.name, e
                    );
                }
            }
        }

        if !all_definitions.is_empty() {
            // Generate the main composite_tags.rs file
            let composite_code = self.generate_composite_tags_module(&all_definitions)?;

            files.push(GeneratedFile {
                path: "composite_tags.rs".to_string(),
                content: composite_code,
                strategy: self.name().to_string(),
            });

            info!(
                "Generated composite_tags.rs with {} composite definitions",
                all_definitions.len()
            );
        }

        info!("CompositeTagStrategy generated {} files", files.len());
        Ok(files)
    }
}
