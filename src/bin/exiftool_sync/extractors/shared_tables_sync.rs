//! Shared lookup tables synchronization extractor
//!
//! This extractor implements Phase 1 of the new two-phase PrintConv architecture.
//! It extracts module-level shared lookup tables like %canonLensTypes that are
//! defined once in ExifTool and referenced by multiple tags.
//!
//! This preserves ExifTool's DRY architecture by generating single lookup table
//! files that can be referenced by multiple PrintConvId variants.

#![doc = "EXIFTOOL-SOURCE: scripts/extract_shared_tables.pl"]

use super::{emit_sync_issue, Extractor, PerlSource, Priority, SyncIssue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Shared lookup tables synchronization extractor
pub struct SharedTablesSyncExtractor {
    /// Path to the Perl extraction script
    perl_script: PathBuf,
    /// Output directory for generated lookup tables
    output_dir: PathBuf,
    /// Cache directory for idempotent operation
    cache_dir: PathBuf,
}

/// Data extracted from the shared tables Perl script
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedTablesData {
    metadata: SharedTablesMetadata,
    statistics: SharedTablesStatistics,
    shared_tables: HashMap<String, SharedTable>,
}

/// Metadata about the shared tables extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedTablesMetadata {
    extraction_mode: String,
    extraction_date: String,
    exiftool_version: String,
    processed_modules: u32,
}

/// Statistics about shared tables extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedTablesStatistics {
    total_shared_tables: u32,
    total_entries: u32,
}

/// A shared lookup table from ExifTool
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedTable {
    module: String,
    table_name: String,
    entry_count: u32,
    entries: HashMap<String, String>,
}

/// Cache data for idempotent operation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedTablesCacheData {
    content_hash: String,
    last_extraction: String,
    generated_files: Vec<String>,
}

impl SharedTablesSyncExtractor {
    /// Create a new shared tables sync extractor
    pub fn new() -> Self {
        Self {
            perl_script: PathBuf::from("scripts/extract_shared_tables.pl"),
            output_dir: PathBuf::from("src/tables/generated"),
            cache_dir: PathBuf::from(".cache/shared_tables"),
        }
    }

    /// Calculate hash of shared tables data for change detection
    fn calculate_content_hash(&self, data: &SharedTablesData) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the essential data that affects code generation
        data.shared_tables.len().hash(&mut hasher);

        // Hash table data in a deterministic way
        let mut sorted_tables: Vec<_> = data.shared_tables.iter().collect();
        sorted_tables.sort_by(|a, b| a.0.cmp(b.0));

        for (name, table) in sorted_tables {
            name.hash(&mut hasher);
            table.table_name.hash(&mut hasher);
            table.module.hash(&mut hasher);
            table.entry_count.hash(&mut hasher);

            // Hash entries in deterministic order
            let mut sorted_entries: Vec<_> = table.entries.iter().collect();
            sorted_entries.sort_by(|a, b| a.0.cmp(b.0));
            sorted_entries.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Check if cached data indicates no changes needed
    fn has_changes(&self, data: &SharedTablesData) -> Result<bool, String> {
        let cache_file = self.cache_dir.join("last_sync.json");

        if !cache_file.exists() {
            return Ok(true); // No cache means we need to run
        }

        let cached_data: SharedTablesCacheData = match fs::read_to_string(&cache_file) {
            Ok(content) => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse cache data: {}", e))?,
            Err(_) => return Ok(true), // Cannot read cache, assume changes
        };

        let current_hash = self.calculate_content_hash(data);
        Ok(current_hash != cached_data.content_hash)
    }

    /// Update cache with current data
    fn update_cache(
        &self,
        data: &SharedTablesData,
        generated_files: &[String],
    ) -> Result<(), String> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        let cache_data = SharedTablesCacheData {
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

    /// Extract shared tables data from all ExifTool modules
    fn extract_shared_tables_data(&self) -> Result<SharedTablesData, String> {
        println!("  Running shared tables extraction script...");

        let output = Command::new("perl")
            .arg(&self.perl_script)
            .arg("--all-modules")
            .output()
            .map_err(|e| format!("Failed to run Perl script: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Perl script failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let shared_tables_data: SharedTablesData = serde_json::from_str(&output_str)
            .map_err(|e| {
                fs::create_dir_all(".cache").ok(); // Ignore errors
                if fs::write(".cache/debug_shared_tables_output.json", output_str.as_bytes()).is_err() {
                    // Ignore write errors
                }
                format!("Failed to parse JSON output: {}. Output saved to .cache/debug_shared_tables_output.json", e)
            })?;

        println!(
            "  ✅ Extracted {} shared tables with {} total entries",
            shared_tables_data.statistics.total_shared_tables,
            shared_tables_data.statistics.total_entries
        );

        Ok(shared_tables_data)
    }

    /// Generate lookup table files for shared data
    fn generate_shared_lookup_tables(
        &self,
        data: &SharedTablesData,
    ) -> Result<Vec<String>, String> {
        let mut generated_files = Vec::new();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate a lookup table file for each shared table
        for (table_name, table_data) in &data.shared_tables {
            let filename = self.generate_shared_table_file(table_name, table_data)?;
            generated_files.push(filename);
        }

        println!(
            "  ✅ Generated {} shared lookup table files",
            generated_files.len()
        );
        Ok(generated_files)
    }

    /// Generate a single shared lookup table file
    fn generate_shared_table_file(
        &self,
        table_name: &str,
        table_data: &SharedTable,
    ) -> Result<String, String> {
        // Convert table name to valid Rust module name
        let rust_module_name = table_name
            .to_lowercase()
            .replace("types", "_types")
            .replace("ids", "_ids");

        let filename = format!("{}.rs", rust_module_name);
        let output_path = self.output_dir.join(&filename);

        let mut content = String::new();

        // File header with attribution
        content.push_str(&format!(
            "//! Shared lookup table for {}\n\
             //!\n\
             //! AUTO-GENERATED by exiftool_sync extract shared-tables\n\
             //! Source: {} from {}\n\
             //! DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract shared-tables`\n\n",
            table_name, table_data.table_name, table_data.module
        ));

        content.push_str("#![doc = \"EXIFTOOL-SOURCE: ");
        content.push_str(
            &table_data
                .module
                .replace("Image::ExifTool::", "lib/Image/ExifTool/"),
        );
        content.push_str(".pm\"]\n\n");

        // Imports
        content.push_str("use phf::phf_map;\n\n");

        // Determine entry type based on keys
        let entry_type = if table_data
            .entries
            .keys()
            .all(|k| k.parse::<u16>().is_ok() || k.parse::<i16>().is_ok())
        {
            "i32" // Use i32 to handle negative values like -1
        } else {
            "&str"
        };

        // Generate the constant name
        let const_name = rust_module_name.to_uppercase();

        // Generate PHF map
        content.push_str(&format!(
            "/// {} shared lookup table\n\
             ///\n\
             /// This table maps {} identifiers to human-readable strings.\n\
             /// Contains {} entries extracted from ExifTool {}.\n\
             ///\n\
             /// Used by multiple tags via PrintConvId::{} reference.\n\
             pub static {}: phf::Map<{}, &'static str> = phf_map! {{\n",
            table_name,
            table_name
                .replace("Types", "")
                .replace("IDs", "")
                .replace("ID", ""),
            table_data.entry_count,
            table_data.module.replace("Image::ExifTool::", ""),
            self.table_name_to_printconv_id(table_name),
            const_name,
            entry_type
        ));

        // Add entries, sorted for deterministic output
        let mut sorted_entries: Vec<_> = table_data.entries.iter().collect();
        sorted_entries.sort_by(|a, b| {
            // Try to sort numerically if possible, otherwise lexically
            match (a.0.parse::<f64>(), b.0.parse::<f64>()) {
                (Ok(a_num), Ok(b_num)) => a_num
                    .partial_cmp(&b_num)
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => a.0.cmp(b.0),
            }
        });

        for (key, value) in sorted_entries {
            let formatted_key = if entry_type == "i32" {
                // Parse as number, handling negative values and decimals
                if let Ok(int_val) = key.parse::<i32>() {
                    format!("{}i32", int_val)
                } else if let Ok(float_val) = key.parse::<f64>() {
                    format!("{}i32", float_val as i32)
                } else {
                    format!("0i32 /* Invalid key: {} */", key)
                }
            } else {
                format!("\"{}\"", key.replace("\"", "\\\""))
            };

            content.push_str(&format!(
                "    {} => \"{}\",\n",
                formatted_key,
                value.replace("\"", "\\\"")
            ));
        }

        content.push_str("};\n");

        // Write file
        fs::write(&output_path, content)
            .map_err(|e| format!("Failed to write shared lookup table {}: {}", filename, e))?;

        Ok(filename)
    }

    /// Convert table name to corresponding PrintConvId variant name
    fn table_name_to_printconv_id(&self, table_name: &str) -> String {
        match table_name {
            "canonLensTypes" => "CanonLensTypes".to_string(),
            "canonModelID" => "CanonModelID".to_string(),
            "nikonLensIDs" => "NikonLensIDs".to_string(),
            "pentaxLensTypes" => "PentaxLensTypes".to_string(),
            "sonyLensTypes" => "SonyLensTypes".to_string(),
            "sonyLensTypes2" => "SonyLensTypes2".to_string(),
            "sigmaLensTypes" => "SigmaLensTypes".to_string(),
            _ => {
                // Fallback: convert camelCase to PascalCase
                table_name
                    .chars()
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .to_string()
                    + &table_name[1..]
            }
        }
    }

    /// Emit sync issues for any problems discovered
    fn emit_sync_issues(&self, data: &SharedTablesData) -> Result<(), String> {
        // Check for potential issues with shared tables
        for (table_name, table_data) in &data.shared_tables {
            // Warn about very large tables that might need optimization
            if table_data.entry_count > 1000 {
                emit_sync_issue(SyncIssue {
                    priority: Priority::Low,
                    command: "shared-tables".to_string(),
                    perl_source: PerlSource {
                        file: format!(
                            "{}.pm",
                            table_data
                                .module
                                .replace("Image::ExifTool::", "lib/Image/ExifTool/")
                        ),
                        lines: None,
                    },
                    rust_target: Some(format!(
                        "src/tables/generated/{}.rs",
                        table_name.to_lowercase()
                    )),
                    description: format!(
                        "Large shared table {} has {} entries - consider optimization",
                        table_name, table_data.entry_count
                    ),
                    suggested_implementation: "Review if table can be split or compressed"
                        .to_string(),
                })?;
            }

            // Warn about tables with non-standard key formats
            let has_decimal_keys = table_data.entries.keys().any(|k| k.contains('.'));
            if has_decimal_keys {
                emit_sync_issue(SyncIssue {
                    priority: Priority::Medium,
                    command: "shared-tables".to_string(),
                    perl_source: PerlSource {
                        file: format!(
                            "{}.pm",
                            table_data
                                .module
                                .replace("Image::ExifTool::", "lib/Image/ExifTool/")
                        ),
                        lines: None,
                    },
                    rust_target: Some(format!(
                        "src/tables/generated/{}.rs",
                        table_name.to_lowercase()
                    )),
                    description: format!(
                        "Shared table {} contains decimal keys - verify integer conversion",
                        table_name
                    ),
                    suggested_implementation: "Review decimal key handling in lookup logic"
                        .to_string(),
                })?;
            }
        }

        Ok(())
    }
}

impl Extractor for SharedTablesSyncExtractor {
    fn extract(&self, _exiftool_path: &Path) -> Result<(), String> {
        println!("🔄 Extracting shared lookup tables from ExifTool modules...");

        // 1. Extract shared tables data using Perl script
        let data = self.extract_shared_tables_data()?;

        // 2. Check if anything changed (idempotent operation)
        if !self.has_changes(&data)? {
            println!("  ℹ️  Shared tables data unchanged, skipping regeneration");
            return Ok(());
        }

        // 3. Generate shared lookup table files
        let generated_files = self.generate_shared_lookup_tables(&data)?;

        // 4. Emit sync issues for any discovered problems
        self.emit_sync_issues(&data)?;

        // 5. Update cache for next run
        self.update_cache(&data, &generated_files)?;

        // 6. Report results
        println!("  📊 Shared tables sync results:");
        println!(
            "     - Total shared tables: {}",
            data.statistics.total_shared_tables
        );
        println!(
            "     - Total lookup entries: {}",
            data.statistics.total_entries
        );
        println!("     - Generated files: {}", generated_files.len());

        println!("  ✅ Shared lookup tables synchronization complete");
        Ok(())
    }
}
