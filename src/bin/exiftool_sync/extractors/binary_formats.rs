//! Extractor for ProcessBinaryData table definitions from ExifTool

use super::Extractor;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct BinaryFormatsExtractor {
    // Maps Perl module name to extracted tables
    tables: HashMap<String, Vec<BinaryTable>>,
}

#[derive(Debug)]
struct BinaryTable {
    name: String,
    process_proc: String,
    default_format: Option<String>,
    #[allow(dead_code)]
    groups: Option<HashMap<u8, String>>,
    first_entry: Option<i32>,
    entries: Vec<BinaryEntry>,
    #[allow(dead_code)]
    source_file: String,
    line_start: usize,
    line_end: usize,
}

#[derive(Debug)]
struct BinaryEntry {
    offset: f64, // Can be fractional (e.g., 586.1) or negative
    name: String,
    format: Option<String>,
    mask: Option<u32>,
    #[allow(dead_code)]
    shift: Option<u8>,
    condition: Option<String>,
    value_conv: Option<String>,
    print_conv: Option<String>,
    notes: Option<String>,
}

impl BinaryFormatsExtractor {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Parse a Perl module file and extract binary data tables
    fn parse_perl_module(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "Invalid module path".to_string())?;

        println!("  Parsing {}", module_name);

        // Find all table definitions
        let table_regex = Regex::new(r"(?m)^%Image::ExifTool::\w+::(\w+)\s*=\s*\(").unwrap();
        let mut tables = Vec::new();

        for cap in table_regex.captures_iter(&content) {
            let table_name = &cap[1];
            let start_pos = cap.get(0).unwrap().start();

            // Check if this is a ProcessBinaryData table
            if self.is_binary_data_table(&content, start_pos)? {
                if let Some(table) = self.extract_table(&content, table_name, start_pos, path)? {
                    tables.push(table);
                }
            }
        }

        if !tables.is_empty() {
            println!("    Found {} binary data tables", tables.len());
            self.tables.insert(module_name.to_string(), tables);
        }

        Ok(())
    }

    /// Check if a table is a ProcessBinaryData table
    fn is_binary_data_table(&self, content: &str, start_pos: usize) -> Result<bool, String> {
        // Look for PROCESS_PROC => \&ProcessBinaryData or similar
        let check_region = &content[start_pos..std::cmp::min(start_pos + 1000, content.len())];
        Ok(check_region.contains("PROCESS_PROC")
            && (check_region.contains("ProcessBinaryData")
                || check_region.contains("ProcessNikonEncrypted")
                || check_region.contains("ProcessCanon")
                || check_region.contains("ProcessSony")))
    }

    /// Extract a single binary data table
    fn extract_table(
        &self,
        content: &str,
        name: &str,
        start_pos: usize,
        path: &Path,
    ) -> Result<Option<BinaryTable>, String> {
        // Find the end of the table (closing parenthesis)
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escape = false;
        let mut end_pos = start_pos;

        for (i, ch) in content[start_pos..].char_indices() {
            if escape {
                escape = false;
                continue;
            }

            match ch {
                '\\' => escape = true,
                '\'' | '"' if !in_string => in_string = true,
                '\'' | '"' if in_string => in_string = false,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        end_pos = start_pos + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end_pos == start_pos {
            return Err(format!("Could not find end of table {}", name));
        }

        let table_content = &content[start_pos..end_pos];
        let line_start = content[..start_pos].matches('\n').count() + 1;
        let line_end = content[..end_pos].matches('\n').count() + 1;

        // Extract table-level attributes
        let process_proc = self
            .extract_process_proc(table_content)
            .unwrap_or_else(|_| "ProcessBinaryData".to_string());
        let default_format = self.extract_table_attribute(table_content, "FORMAT");
        let first_entry = self
            .extract_table_attribute(table_content, "FIRST_ENTRY")
            .and_then(|s| s.parse::<i32>().ok());

        // Parse table entries
        let entries = self.parse_table_entries(table_content)?;

        // Skip tables with no entries
        if entries.is_empty() {
            return Ok(None);
        }

        println!("      Found table {} with {} entries", name, entries.len());

        Ok(Some(BinaryTable {
            name: name.to_string(),
            process_proc,
            default_format,
            groups: None, // TODO: Parse GROUPS
            first_entry,
            entries,
            source_file: path.to_string_lossy().to_string(),
            line_start,
            line_end,
        }))
    }

    /// Extract PROCESS_PROC value
    fn extract_process_proc(&self, table_content: &str) -> Result<String, String> {
        let proc_regex = Regex::new(r"PROCESS_PROC\s*=>\s*\\&(?:Image::ExifTool::)?(\w+)").unwrap();
        proc_regex
            .captures(table_content)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "No PROCESS_PROC found".to_string())
    }

    /// Extract a table-level attribute
    fn extract_table_attribute(&self, table_content: &str, attr: &str) -> Option<String> {
        let regex_str = format!(r"{}\s*=>\s*'([^']+)'", attr);
        let attr_regex = Regex::new(&regex_str).ok()?;
        attr_regex
            .captures(table_content)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Parse individual entries from a table
    fn parse_table_entries(&self, table_content: &str) -> Result<Vec<BinaryEntry>, String> {
        let mut entries = Vec::new();

        // Match both simple and complex entry patterns
        // Simple: 1 => 'TagName',
        // Complex: 1 => { Name => 'TagName', ... },
        let entry_regex =
            Regex::new(r"(?m)^\s*(-?\d+(?:\.\d+)?|0x[0-9a-fA-F]+)\s*=>\s*(.+?)(?:,\s*(?:\n|$))")
                .unwrap();

        for cap in entry_regex.captures_iter(table_content) {
            let offset_str = &cap[1];
            let value_part = cap[2].trim();

            // Parse offset (decimal, hex, or fractional)
            let offset = if let Some(stripped) = offset_str.strip_prefix("0x") {
                i32::from_str_radix(stripped, 16)
                    .map(|v| v as f64)
                    .map_err(|_| format!("Invalid hex offset: {}", offset_str))?
            } else {
                offset_str
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid offset: {}", offset_str))?
            };

            // Skip meta entries like PROCESS_PROC, FORMAT, etc.
            if offset_str.chars().all(|c| c.is_alphabetic() || c == '_') {
                continue;
            }

            let mut entry = BinaryEntry {
                offset,
                name: String::new(),
                format: None,
                mask: None,
                shift: None,
                condition: None,
                value_conv: None,
                print_conv: None,
                notes: None,
            };

            if value_part.starts_with('{') {
                // Complex format: parse hash
                // Find the matching closing brace
                let mut brace_count = 0;
                let mut end_pos = 0;
                for (i, ch) in value_part.char_indices() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_pos = i;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                let entry_content = &value_part[1..end_pos];

                // Extract fields from hash
                entry.name = self
                    .extract_entry_field(entry_content, "Name")
                    .ok_or_else(|| format!("No Name found for offset {}", offset))?;
                entry.format = self.extract_entry_field(entry_content, "Format");
                entry.condition = self.extract_entry_field(entry_content, "Condition");
                entry.value_conv = self.extract_entry_field(entry_content, "ValueConv");
                entry.print_conv = self.extract_entry_field(entry_content, "PrintConv");
                entry.notes = self.extract_entry_field(entry_content, "Notes");
                entry.mask = self.extract_hex_field(entry_content, "Mask");
            } else if value_part.starts_with('\'') {
                // Simple format: the value is the tag name
                let name_regex = Regex::new(r"^'([^']+)'").unwrap();
                if let Some(cap) = name_regex.captures(value_part) {
                    entry.name = cap[1].to_string();
                } else {
                    continue; // Skip entries we can't parse
                }
            } else {
                // Skip other types of values (like references to other tables)
                continue;
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Extract a field value from entry content
    fn extract_entry_field(&self, entry_content: &str, field: &str) -> Option<String> {
        let regex_str = format!(r"{}\s*=>\s*'([^']+)'", field);
        let field_regex = Regex::new(&regex_str).ok()?;
        field_regex
            .captures(entry_content)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract a hex field value
    fn extract_hex_field(&self, entry_content: &str, field: &str) -> Option<u32> {
        let regex_str = format!(r"{}\s*=>\s*(0x[0-9a-fA-F]+)", field);
        let field_regex = Regex::new(&regex_str).ok()?;
        field_regex
            .captures(entry_content)
            .and_then(|cap| cap.get(1))
            .and_then(|m| u32::from_str_radix(&m.as_str()[2..], 16).ok())
    }

    /// Generate Rust code for extracted tables
    fn generate_rust_code(&self) -> Result<(), String> {
        // Create output directory
        let output_dir = Path::new("src/binary/formats");
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate a file for each module
        for (module, tables) in &self.tables {
            let filename = format!("{}.rs", module.to_lowercase());
            let filepath = output_dir.join(&filename);

            println!("  Generating {}", filepath.display());

            let mut file = fs::File::create(&filepath)
                .map_err(|e| format!("Failed to create {}: {}", filepath.display(), e))?;

            // Write file header
            writeln!(
                file,
                "// AUTO-GENERATED by exiftool_sync extract binary-formats"
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(
                file,
                "// Source: third-party/exiftool/lib/Image/ExifTool/{}.pm",
                module
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "// Generated by exiftool_sync extract binary-formats")
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "// DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract binary-formats`")
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file).map_err(|e| format!("Write error: {}", e))?;
            writeln!(
                file,
                "#![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/{}.pm\"]",
                module
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file).map_err(|e| format!("Write error: {}", e))?;
            writeln!(
                file,
                "use crate::core::binary_data::{{BinaryDataTable, BinaryDataTableBuilder}};"
            )
            .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file, "use crate::core::types::ExifFormat;")
                .map_err(|e| format!("Write error: {}", e))?;
            writeln!(file).map_err(|e| format!("Write error: {}", e))?;

            // Generate a function for each table
            for table in tables {
                writeln!(file, "/// Binary data table: {}", table.name)
                    .map_err(|e| format!("Write error: {}", e))?;
                writeln!(
                    file,
                    "/// Source lines: {}-{}",
                    table.line_start, table.line_end
                )
                .map_err(|e| format!("Write error: {}", e))?;
                writeln!(file, "/// Process function: {}", table.process_proc)
                    .map_err(|e| format!("Write error: {}", e))?;
                writeln!(
                    file,
                    "pub fn create_{}_table() -> BinaryDataTable {{",
                    table.name.to_lowercase()
                )
                .map_err(|e| format!("Write error: {}", e))?;

                // Determine default format
                let default_format = table.default_format.as_deref().unwrap_or("int8u");
                writeln!(
                    file,
                    "    BinaryDataTableBuilder::new(\"{}\", ExifFormat::{})",
                    table.name,
                    self.format_to_rust(default_format)
                )
                .map_err(|e| format!("Write error: {}", e))?;

                // Add encryption flag if needed
                if table.process_proc.contains("Encrypted") {
                    writeln!(file, "        .encrypted(true)")
                        .map_err(|e| format!("Write error: {}", e))?;
                }

                // Add first entry if specified
                if let Some(first_entry) = table.first_entry {
                    writeln!(file, "        .first_entry({})", first_entry)
                        .map_err(|e| format!("Write error: {}", e))?;
                }

                // Add entries
                for entry in &table.entries {
                    // Determine format (use entry format or default)
                    let format = entry.format.as_deref().unwrap_or(default_format);

                    // Handle fractional offsets (bit fields)
                    if entry.offset.fract() != 0.0 {
                        let base_offset = entry.offset.floor() as i32;
                        let bit_pos = ((entry.offset.fract() * 10.0).round() as u8) - 1;

                        if let Some(mask) = entry.mask {
                            writeln!(
                                file,
                                "        .add_bit_field({}, \"{}\", 0x{:x}, {})",
                                base_offset, entry.name, mask, bit_pos
                            )
                            .map_err(|e| format!("Write error: {}", e))?;
                        }
                    } else {
                        // Regular field
                        let offset = entry.offset as i32;
                        let (fmt, count) = self.parse_format_spec(format);

                        writeln!(
                            file,
                            "        .add_field({}, \"{}\", ExifFormat::{}, {})",
                            offset, entry.name, fmt, count
                        )
                        .map_err(|e| format!("Write error: {}", e))?;
                    }

                    // Add condition if present
                    if let Some(condition) = &entry.condition {
                        writeln!(file, "        // Condition: {}", condition)
                            .map_err(|e| format!("Write error: {}", e))?;
                    }
                }

                writeln!(file, "        .build()").map_err(|e| format!("Write error: {}", e))?;
                writeln!(file, "}}").map_err(|e| format!("Write error: {}", e))?;
                writeln!(file).map_err(|e| format!("Write error: {}", e))?;
            }

            file.flush().map_err(|e| format!("Flush error: {}", e))?;
        }

        Ok(())
    }

    /// Parse format specification like "string[4]" or "int16u"
    fn parse_format_spec(&self, format: &str) -> (&'static str, u32) {
        // Check for array notation like "string[4]" or "undef[12]"
        if let Some(bracket_pos) = format.find('[') {
            let base_format = &format[..bracket_pos];
            let count_str = &format[bracket_pos + 1..format.len() - 1];
            let count = count_str.parse::<u32>().unwrap_or(1);
            (self.format_to_rust(base_format), count)
        } else {
            (self.format_to_rust(format), 1)
        }
    }

    /// Convert Perl format to Rust ExifFormat
    fn format_to_rust(&self, perl_format: &str) -> &'static str {
        match perl_format {
            "int8u" | "int8" => "U8",
            "int16u" => "U16",
            "int16s" => "I16",
            "int32u" => "U32",
            "int32s" => "I32",
            "int64u" => "U64",
            "int64s" => "I64",
            "string" => "AsciiString",
            "undef" => "Undefined",
            "rational64u" => "URational",
            "rational64s" => "IRational",
            "float" => "Float",
            "double" => "Double",
            _ => "U8", // Default
        }
    }
}

impl Extractor for BinaryFormatsExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        let mut extractor = BinaryFormatsExtractor::new();

        println!("Scanning for ProcessBinaryData tables...");

        // Scan all Perl modules
        let lib_path = exiftool_path.join("lib/Image/ExifTool");
        if !lib_path.exists() {
            return Err(format!(
                "ExifTool lib directory not found: {}",
                lib_path.display()
            ));
        }

        // Process main manufacturer modules
        let manufacturers = [
            "Canon",
            "Nikon",
            "Sony",
            "Fujifilm",
            "Olympus",
            "Pentax",
            "Panasonic",
            "Samsung",
            "Sigma",
            "Apple",
        ];

        for manufacturer in &manufacturers {
            let module_path = lib_path.join(format!("{}.pm", manufacturer));
            if module_path.exists() {
                extractor.parse_perl_module(&module_path)?;
            }
        }

        // Generate Rust code
        println!("\nGenerating Rust code...");
        extractor.generate_rust_code()?;

        println!(
            "\nExtracted {} modules with binary data tables",
            extractor.tables.len()
        );

        Ok(())
    }
}
