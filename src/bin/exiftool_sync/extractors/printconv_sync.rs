//! PrintConv synchronization extractor
//!
//! This extractor orchestrates the complete PrintConv synchronization workflow:
//! 1. Extracts PrintConv data from all ExifTool modules using Perl introspection
//! 2. Maps patterns to PrintConvId enum variants using pattern matching tables
//! 3. Generates lookup tables for shared data (lens types, etc.)
//! 4. Tracks unmapped patterns using emit_sync_issue for manual implementation
//! 5. Implements idempotent operation with cache-based change detection
//!
//! The extractor follows the design specified in doc/SYNC-PRINTCONV-DESIGN.md
//! and implements Phase 3 of doc/TODO-SYNC-PRINTCONV.md.

#![doc = "EXIFTOOL-SOURCE: scripts/extract_printconv.pl"]

use super::{emit_sync_issue, Extractor, PerlSource, SyncIssue};
use crate::tag_metadata::TagMetadata;
use exif_oxide::tables::printconv_patterns::{determine_printconv_id, normalize_hash_pattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// PrintConv synchronization extractor
pub struct PrintConvSyncExtractor {
    /// Path to the Perl extraction script
    perl_script: PathBuf,
    /// Output directory for generated lookup tables
    output_dir: PathBuf,
    /// Cache directory for idempotent operation
    cache_dir: PathBuf,
    /// Tag metadata for prioritizing sync issues
    tag_metadata: TagMetadata,
}

/// Data extracted from the Perl script
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedData {
    metadata: ExtractionMetadata,
    statistics: ExtractionStatistics,
    tags: Vec<ExtractedTag>,
    shared_lookups: HashMap<String, SharedLookupData>,
}

/// Metadata about the extraction run
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractionMetadata {
    exiftool_version: String,
    extraction_date: String,
    extraction_mode: String,
    module_count: u32,
    failed_modules: Vec<String>,
}

/// Statistics about the extraction results
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractionStatistics {
    total_tags: u32,
    tags_with_printconv: u32,
    printconv_types: HashMap<String, u32>,
    processed_modules: u32,
    failed_modules: u32,
}

/// A tag with PrintConv data extracted from ExifTool
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedTag {
    tag_id: String,
    tag_name: String,
    module: String,
    printconv_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_func: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    printconv_source: Option<String>,
}

/// Shared lookup table data (lens types, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedLookupData {
    lookup_name: String,
    module: String,
    entries: HashMap<String, serde_json::Value>,
}

/// Cache data for idempotent operation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheData {
    content_hash: String,
    last_extraction: String,
    generated_files: Vec<String>,
}

impl PrintConvSyncExtractor {
    /// Create a new PrintConv sync extractor
    pub fn new() -> Self {
        Self {
            perl_script: PathBuf::from("scripts/extract_printconv.pl"),
            output_dir: PathBuf::from("src/tables/generated"),
            cache_dir: PathBuf::from(".cache/printconv"),
            tag_metadata: TagMetadata::new().unwrap_or_else(|_| TagMetadata::empty()),
        }
    }

    /// Calculate hash of extracted data for change detection
    fn calculate_content_hash(&self, data: &ExtractedData) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the essential data that affects code generation
        data.tags.len().hash(&mut hasher);
        data.shared_lookups.len().hash(&mut hasher);

        // Hash tag data in a deterministic way
        let mut sorted_tags: Vec<_> = data.tags.iter().collect();
        sorted_tags.sort_by(|a, b| a.tag_id.cmp(&b.tag_id));
        for tag in sorted_tags {
            tag.tag_id.hash(&mut hasher);
            tag.tag_name.hash(&mut hasher);
            tag.printconv_type.hash(&mut hasher);
            if let Some(ref data) = tag.printconv_data {
                // Hash the JSON data as string for deterministic hashing
                data.to_string().hash(&mut hasher);
            }
        }

        // Hash shared lookup data
        let mut sorted_lookups: Vec<_> = data.shared_lookups.iter().collect();
        sorted_lookups.sort_by(|a, b| a.0.cmp(b.0));
        for (name, lookup) in sorted_lookups {
            name.hash(&mut hasher);
            lookup.lookup_name.hash(&mut hasher);
            // Sort entries for deterministic hashing
            let mut sorted_entries: Vec<_> = lookup.entries.iter().collect();
            sorted_entries.sort_by(|a, b| a.0.cmp(b.0));
            sorted_entries.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Check if cached data indicates no changes needed
    fn has_changes(&self, data: &ExtractedData) -> Result<bool, String> {
        let cache_file = self.cache_dir.join("last_sync.json");

        if !cache_file.exists() {
            return Ok(true); // No cache means we need to run
        }

        let cached_data: CacheData = match fs::read_to_string(&cache_file) {
            Ok(content) => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse cache data: {}", e))?,
            Err(_) => return Ok(true), // Cannot read cache, assume changes
        };

        let current_hash = self.calculate_content_hash(data);
        Ok(current_hash != cached_data.content_hash)
    }

    /// Update cache with current data
    fn update_cache(&self, data: &ExtractedData, generated_files: &[String]) -> Result<(), String> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        let cache_data = CacheData {
            content_hash: self.calculate_content_hash(data),
            last_extraction: chrono::Utc::now().to_rfc3339(),
            generated_files: generated_files.to_vec(),
        };

        let cache_file = self.cache_dir.join("last_sync.json");
        let cache_content = serde_json::to_string_pretty(&cache_data)
            .map_err(|e| format!("Failed to serialize cache data: {}", e))?;

        fs::write(&cache_file, cache_content)
            .map_err(|e| format!("Failed to write cache: {}", e))?;

        Ok(())
    }

    /// Extract PrintConv data from all ExifTool modules
    fn extract_printconv_data(&self) -> Result<ExtractedData, String> {
        println!("  Running Perl extraction script with --all-modules...");

        let output = Command::new("perl")
            .arg(&self.perl_script)
            .arg("--all-modules")
            .output()
            .map_err(|e| format!("Failed to run Perl script: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Perl script failed: {}", stderr));
        }

        // First try to clean up any invalid UTF-8 from the output
        let output_str = String::from_utf8_lossy(&output.stdout);

        let extracted_data: ExtractedData = serde_json::from_str(&output_str)
            .map_err(|e| {
                // Write the problematic output to a debug file for investigation
                fs::create_dir_all(".cache").ok(); // Ignore errors
                if fs::write(".cache/debug_output.json", output_str.as_bytes()).is_err() {
                    // Ignore write errors
                }
                format!("Failed to parse JSON output: {}. Output saved to .cache/debug_output.json for investigation", e)
            })?;

        println!(
            "  ✅ Extracted {} tags from {} modules",
            extracted_data.statistics.tags_with_printconv,
            extracted_data.statistics.processed_modules
        );

        Ok(extracted_data)
    }

    /// Generate lookup tables for shared data
    fn generate_lookup_tables(&self, data: &ExtractedData) -> Result<Vec<String>, String> {
        let mut generated_files = Vec::new();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate lookup tables from all hash-based PrintConv data
        let mut hash_tables: HashMap<String, SharedLookupData> = HashMap::new();

        // Collect all hash PrintConv data and group by similar patterns
        for tag in &data.tags {
            if tag.printconv_type == "hash" {
                if let Some(serde_json::Value::Object(map)) = &tag.printconv_data {
                    let table_name = self.generate_table_name(&tag.module, &tag.tag_name);
                    let lookup_data = SharedLookupData {
                        lookup_name: tag.tag_name.clone(),
                        module: tag.module.clone(),
                        entries: map.clone().into_iter().collect(),
                    };
                    hash_tables.insert(table_name, lookup_data);
                }
            }
        }

        // Generate lookup tables for all collected hash data
        for (name, lookup_data) in &hash_tables {
            if self.should_generate_table(name, lookup_data) {
                let output_file = self.generate_lookup_table(name, lookup_data)?;
                generated_files.push(output_file);
            }
        }

        // Also handle any shared_lookups from the JSON (if present)
        for (name, lookup_data) in &data.shared_lookups {
            if self.should_generate_table(name, lookup_data) {
                let output_file = self.generate_lookup_table(name, lookup_data)?;
                generated_files.push(output_file);
            }
        }

        println!("  ✅ Generated {} lookup tables", generated_files.len());
        Ok(generated_files)
    }

    /// Generate a table name from module and tag name
    fn generate_table_name(&self, module: &str, tag_name: &str) -> String {
        let module_clean = module
            .replace("Image::ExifTool::", "")
            .replace("::", "_")
            .to_lowercase();
        let tag_clean = tag_name.to_lowercase().replace(" ", "_");
        format!("{}_{}", module_clean, tag_clean)
    }

    /// Check if we should generate a lookup table for this data
    fn should_generate_table(&self, _name: &str, lookup_data: &SharedLookupData) -> bool {
        // Generate tables for hash lookups with reasonable number of entries
        let entry_count = lookup_data.entries.len();

        // Skip very small tables (< 3 entries) and very large ones (> 1000 entries)
        // Small tables are not worth the overhead, large ones might be test data
        (3..=1000).contains(&entry_count)
    }

    /// Generate a lookup table file for shared data
    fn generate_lookup_table(
        &self,
        name: &str,
        lookup_data: &SharedLookupData,
    ) -> Result<String, String> {
        let filename = format!("{}.rs", name.to_lowercase());
        let output_path = self.output_dir.join(&filename);

        let mut content = String::new();

        // File header with attribution
        content.push_str(&format!(
            "//! Generated lookup table for {}\n\
             //!\n\
             //! AUTO-GENERATED by exiftool_sync extract printconv-sync\n\
             //! Source: {} from {}\n\
             //! DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract printconv-sync`\n\n",
            name, lookup_data.lookup_name, lookup_data.module));

        content.push_str("#![doc = \"EXIFTOOL-SOURCE: ");
        content.push_str(
            &lookup_data
                .module
                .replace("Image::ExifTool::", "lib/Image/ExifTool/"),
        );
        content.push_str(".pm\"]\n\n");

        // Imports
        content.push_str("use phf::phf_map;\n\n");

        // Generate the constant name
        let const_name = name.to_uppercase();
        let entry_type = if lookup_data.entries.keys().all(|k| k.parse::<u16>().is_ok()) {
            "u16"
        } else {
            "&str"
        };

        // Generate PHF map
        content.push_str(&format!(
            "/// {} lookup table\n\
             ///\n\
             /// This table maps {} values to human-readable strings.\n\
             /// Contains {} entries extracted from ExifTool.\n\
             pub static {}: phf::Map<{}, &'static str> = phf_map! {{\n",
            name,
            name.replace("lenstypes", " lens types")
                .replace("modes", " modes"),
            lookup_data.entries.len(),
            const_name,
            entry_type
        ));

        // Add entries, sorted for deterministic output
        let mut sorted_entries: Vec<_> = lookup_data.entries.iter().collect();
        sorted_entries.sort_by(|a, b| {
            // Try to sort numerically if possible, otherwise lexically
            match (a.0.parse::<u32>(), b.0.parse::<u32>()) {
                (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                _ => a.0.cmp(b.0),
            }
        });

        for (key, value) in sorted_entries {
            let formatted_key = if entry_type == "u16" {
                format!("{}u16", key)
            } else {
                format!("\"{}\"", key.replace("\"", "\\\""))
            };

            // Convert serde_json::Value to string, handling different types
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(), // Fallback for other types
            };

            content.push_str(&format!(
                "    {} => \"{}\",\n",
                formatted_key,
                value_str.replace("\"", "\\\"")
            ));
        }

        content.push_str("};\n");

        // Write file
        fs::write(&output_path, content)
            .map_err(|e| format!("Failed to write lookup table {}: {}", filename, e))?;

        Ok(filename)
    }

    /// Analyze patterns and emit sync issues for unmapped ones
    fn analyze_patterns(&self, data: &ExtractedData) -> Result<(usize, usize), String> {
        let mut mapped_count = 0;
        let mut unmapped_count = 0;
        let mut unmapped_patterns: HashMap<String, Vec<String>> = HashMap::new();

        for tag in &data.tags {
            if tag.printconv_type == "none" {
                continue; // Skip tags without PrintConv
            }

            let can_map = self.can_map_printconv(tag);

            if can_map {
                mapped_count += 1;
            } else {
                unmapped_count += 1;

                // Group unmapped patterns for better reporting
                let pattern_key = self.get_pattern_key(tag);
                unmapped_patterns
                    .entry(pattern_key)
                    .or_default()
                    .push(format!(
                        "{}::{}",
                        tag.module.replace("Image::ExifTool::", ""),
                        tag.tag_name
                    ));

                // Emit sync issue for this unmapped pattern
                self.emit_unmapped_pattern_issue(tag)?;
            }
        }

        // Report unmapped pattern summary
        if !unmapped_patterns.is_empty() {
            println!("  ⚠️  Top unmapped patterns:");
            let mut pattern_counts: Vec<_> = unmapped_patterns
                .iter()
                .map(|(pattern, tags)| (tags.len(), pattern, tags))
                .collect();
            pattern_counts.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by frequency, descending

            for (count, pattern, tags) in pattern_counts.iter().take(5) {
                println!(
                    "    - {} (used by {} tags): {}",
                    pattern,
                    count,
                    tags.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                );
                if tags.len() > 3 {
                    println!("      ... and {} more", tags.len() - 3);
                }
            }
        }

        Ok((mapped_count, unmapped_count))
    }

    /// Check if a PrintConv pattern can be mapped to existing PrintConvId
    fn can_map_printconv(&self, tag: &ExtractedTag) -> bool {
        // Convert serde_json::Value to HashMap for determine_printconv_id
        let printconv_data_map = tag.printconv_data.as_ref().and_then(|data| {
            if let serde_json::Value::Object(map) = data {
                // Convert serde_json::Map to HashMap
                let hashmap: HashMap<String, serde_json::Value> =
                    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                Some(hashmap)
            } else if let serde_json::Value::Array(arr) = data {
                // If it's an array, use the first object if it exists
                arr.first().and_then(|first| {
                    if let serde_json::Value::Object(map) = first {
                        let hashmap: HashMap<String, serde_json::Value> =
                            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        Some(hashmap)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        });

        let printconv_id = determine_printconv_id(
            &tag.printconv_type,
            printconv_data_map.as_ref(),
            tag.printconv_ref.as_deref(),
            tag.printconv_source.as_deref(),
            tag.printconv_func.as_deref(),
        );

        // Check if we got a meaningful mapping (not just None)
        !matches!(
            printconv_id,
            exif_oxide::core::print_conv::PrintConvId::None
        )
    }

    /// Get a pattern key for grouping unmapped patterns
    fn get_pattern_key(&self, tag: &ExtractedTag) -> String {
        match tag.printconv_type.as_str() {
            "string" => tag
                .printconv_source
                .as_deref()
                .unwrap_or("unknown_string")
                .to_string(),
            "hash" => {
                if let Some(ref data) = tag.printconv_data {
                    // Handle both object and array formats
                    if let serde_json::Value::Object(map) = data {
                        let hashmap: HashMap<String, serde_json::Value> =
                            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        normalize_hash_pattern(&hashmap)
                    } else if let serde_json::Value::Array(arr) = data {
                        if let Some(serde_json::Value::Object(map)) = arr.first() {
                            let hashmap: HashMap<String, serde_json::Value> =
                                map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                            normalize_hash_pattern(&hashmap)
                        } else {
                            "unknown_array_hash".to_string()
                        }
                    } else {
                        "unknown_hash_type".to_string()
                    }
                } else {
                    "unknown_hash".to_string()
                }
            }
            "hash_ref" => {
                format!("\\%{}", tag.printconv_ref.as_deref().unwrap_or("unknown"))
            }
            "code_ref" => {
                format!("\\&{}", tag.printconv_func.as_deref().unwrap_or("unknown"))
            }
            _ => tag.printconv_type.clone(),
        }
    }

    /// Emit a sync issue for an unmapped PrintConv pattern
    fn emit_unmapped_pattern_issue(&self, tag: &ExtractedTag) -> Result<(), String> {
        let priority = self.tag_metadata.get_priority(&tag.tag_name);

        let description = match tag.printconv_type.as_str() {
            "string" => format!(
                "Unmapped string PrintConv pattern for tag {} ({}): {}",
                tag.tag_name,
                tag.tag_id,
                tag.printconv_source.as_deref().unwrap_or("unknown")
            ),
            "hash" => {
                let pattern = if let Some(ref data) = tag.printconv_data {
                    // Handle both object and array formats
                    if let serde_json::Value::Object(map) = data {
                        let hashmap: HashMap<String, serde_json::Value> =
                            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        normalize_hash_pattern(&hashmap)
                    } else if let serde_json::Value::Array(arr) = data {
                        if let Some(serde_json::Value::Object(map)) = arr.first() {
                            let hashmap: HashMap<String, serde_json::Value> =
                                map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                            normalize_hash_pattern(&hashmap)
                        } else {
                            "unknown_array_hash".to_string()
                        }
                    } else {
                        "unknown_hash_type".to_string()
                    }
                } else {
                    "unknown".to_string()
                };
                format!(
                    "Unmapped hash PrintConv pattern for tag {} ({}): {}",
                    tag.tag_name, tag.tag_id, pattern
                )
            }
            "hash_ref" => format!(
                "Unmapped shared lookup PrintConv for tag {} ({}): \\%{}",
                tag.tag_name,
                tag.tag_id,
                tag.printconv_ref.as_deref().unwrap_or("unknown")
            ),
            "code_ref" => format!(
                "Unmapped function PrintConv for tag {} ({}): \\&{}",
                tag.tag_name,
                tag.tag_id,
                tag.printconv_func.as_deref().unwrap_or("unknown")
            ),
            _ => format!(
                "Unmapped PrintConv pattern for tag {} ({}): type={}",
                tag.tag_name, tag.tag_id, tag.printconv_type
            ),
        };

        let suggested_implementation = match tag.printconv_type.as_str() {
            "string" => {
                "Add string pattern to PERL_STRING_PATTERNS in src/tables/printconv_patterns.rs"
            }
            "hash" => "Add hash pattern to HASH_PATTERNS in src/tables/printconv_patterns.rs",
            "hash_ref" => {
                "Add shared lookup case to HASH_REF_PATTERNS or implement custom table generation"
            }
            "code_ref" => {
                "Add function name to FUNCTION_PATTERNS in src/tables/printconv_patterns.rs"
            }
            _ => "Add new PrintConvId variant and pattern matching logic",
        };

        emit_sync_issue(SyncIssue {
            priority,
            command: "printconv-sync".to_string(),
            perl_source: PerlSource {
                file: format!(
                    "{}.pm",
                    tag.module
                        .replace("Image::ExifTool::", "lib/Image/ExifTool/")
                ),
                lines: None,
            },
            rust_target: Some("src/core/print_conv.rs".to_string()),
            description,
            suggested_implementation: suggested_implementation.to_string(),
        })?;

        Ok(())
    }
}

impl Extractor for PrintConvSyncExtractor {
    fn extract(&self, _exiftool_path: &Path) -> Result<(), String> {
        println!("🔄 Extracting PrintConv data from all ExifTool modules...");

        // 1. Extract PrintConv data using Perl script
        let data = self.extract_printconv_data()?;

        // 2. Check if anything changed (idempotent operation)
        if !self.has_changes(&data)? {
            println!("  ℹ️  PrintConv data unchanged, skipping regeneration");
            return Ok(());
        }

        // 3. Generate lookup tables for shared data
        let generated_files = self.generate_lookup_tables(&data)?;

        // 4. Analyze patterns and emit sync issues for unmapped ones
        let (mapped_count, unmapped_count) = self.analyze_patterns(&data)?;

        // 5. Update cache for next run
        self.update_cache(&data, &generated_files)?;

        // 6. Report results
        println!("  📊 PrintConv sync results:");
        println!(
            "     - Total tags with PrintConv: {}",
            data.statistics.tags_with_printconv
        );
        println!(
            "     - Successfully mapped: {} ({:.1}%)",
            mapped_count,
            (mapped_count as f64 / data.statistics.tags_with_printconv as f64) * 100.0
        );
        println!("     - Generated lookup tables: {}", generated_files.len());

        if unmapped_count > 0 {
            println!(
                "     - Unmapped patterns: {} (see sync-todos.jsonl)",
                unmapped_count
            );
        }

        if !data.metadata.failed_modules.is_empty() {
            println!(
                "     - Failed modules: {} (expected - virtual modules)",
                data.metadata.failed_modules.len()
            );
        }

        println!("  ✅ PrintConv synchronization complete");
        Ok(())
    }
}
