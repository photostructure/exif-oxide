//! Extractor for maker note detection patterns from ExifTool

#![allow(dead_code)]

use super::Extractor;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct MakerDetectionExtractor {
    // Maps manufacturer name to detection patterns
    patterns: HashMap<String, Vec<DetectionPattern>>,
}

#[derive(Debug, Clone)]
struct DetectionPattern {
    signature: Vec<u8>,        // Binary signature to match
    version: Option<u8>,       // Version number if applicable
    offset: usize,             // Start offset for IFD parsing
    description: String,       // Human-readable description
    condition: Option<String>, // Additional conditions from Perl
    source_line: usize,        // Line number in source file
}

impl MakerDetectionExtractor {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Parse a manufacturer's Perl module for detection patterns
    fn parse_manufacturer_module(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let manufacturer = self.extract_manufacturer_name(path)?;

        // Look for maker note processing functions and signature patterns
        let patterns = self.extract_detection_patterns(&content, &manufacturer)?;

        if !patterns.is_empty() {
            self.patterns.insert(manufacturer, patterns);
        }

        Ok(())
    }

    /// Extract manufacturer name from file path
    fn extract_manufacturer_name(&self, path: &Path) -> Result<String, String> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;

        // Convert ExifTool module names to our naming convention
        let manufacturer = match filename {
            "Canon" => "canon",
            "Nikon" => "nikon",
            "Sony" => "sony",
            "Olympus" => "olympus",
            "Pentax" => "pentax",
            "FujiFilm" => "fujifilm",
            "Panasonic" => "panasonic",
            "Samsung" => "samsung",
            "Sigma" => "sigma",
            "Apple" => "apple",
            "Hasselblad" => "hasselblad",
            "Leica" => "leica",
            "Ricoh" => "ricoh",
            _ => return Err(format!("Unknown manufacturer: {}", filename)),
        };

        Ok(manufacturer.to_string())
    }

    /// Extract detection patterns from Perl module content
    fn extract_detection_patterns(
        &self,
        content: &str,
        manufacturer: &str,
    ) -> Result<Vec<DetectionPattern>, String> {
        let mut patterns = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Look for signature matching patterns (various formats)
        let signature_regexes = [
            // Standard format: $$dataPt =~ /^pattern/
            Regex::new(r#"\$\$dataPt\s*=~\s*/\^([^/]+)/"#).unwrap(),
            // Alternate format: $val =~ /^pattern/
            Regex::new(r#"\$val\s*=~\s*/\^([^/]+)/"#).unwrap(),
            // String comparison: substr($$dataPt, 0, N) eq 'pattern'
            Regex::new(r#"substr\(\$\$dataPt,\s*0,\s*\d+\)\s*eq\s*'([^']+)'"#).unwrap(),
            // Direct string contains: $$dataPt =~ /pattern/
            Regex::new(r#"\$\$dataPt\s*=~\s*/([^/]+)/"#).unwrap(),
        ];

        // Look for version detection patterns
        let version_regex = Regex::new(r#"\$version\s*=\s*(\d+)"#)
            .map_err(|e| format!("Failed to compile version regex: {}", e))?;

        // Look for offset/start calculations
        let offset_regex = Regex::new(r#"\$start\s*=\s*(\d+)"#)
            .map_err(|e| format!("Failed to compile offset regex: {}", e))?;

        // Look for literal string patterns in module definitions
        let literal_patterns = self.extract_literal_patterns(content, manufacturer)?;

        // Add any literal patterns found
        patterns.extend(literal_patterns);

        for (line_num, line) in lines.iter().enumerate() {
            // Skip comments and empty lines
            if line.trim().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            // Try each signature regex
            for signature_regex in &signature_regexes {
                if let Some(caps) = signature_regex.captures(line) {
                    if let Some(sig_match) = caps.get(1) {
                        let signature_str = sig_match.as_str();

                        // Parse the signature into bytes
                        if let Ok(signature) = self.parse_signature(signature_str, manufacturer) {
                            let mut pattern = DetectionPattern {
                                signature,
                                version: None,
                                offset: 0,
                                description: format!("{} maker note signature", manufacturer),
                                condition: None,
                                source_line: line_num + 1,
                            };

                            // Look for version assignment in surrounding lines
                            for line in lines
                                .iter()
                                .take((line_num + 2).min(lines.len() - 1) + 1)
                                .skip(line_num.saturating_sub(2))
                            {
                                if let Some(version_caps) = version_regex.captures(line) {
                                    if let Some(version_str) = version_caps.get(1) {
                                        if let Ok(version) = version_str.as_str().parse::<u8>() {
                                            pattern.version = Some(version);
                                            pattern.description =
                                                format!("{} maker note v{}", manufacturer, version);
                                        }
                                    }
                                }

                                // Look for offset assignment
                                if let Some(offset_caps) = offset_regex.captures(line) {
                                    if let Some(offset_str) = offset_caps.get(1) {
                                        if let Ok(offset) = offset_str.as_str().parse::<usize>() {
                                            pattern.offset = offset;
                                        }
                                    }
                                }
                            }

                            patterns.push(pattern);
                            break; // Found a match, don't try other regexes
                        }
                    }
                }
            }
        }

        Ok(patterns)
    }

    /// Extract literal patterns based on manufacturer name from module content
    fn extract_literal_patterns(
        &self,
        _content: &str,
        manufacturer: &str,
    ) -> Result<Vec<DetectionPattern>, String> {
        let mut patterns = Vec::new();

        // Look for known manufacturer signatures based on common patterns
        let manufacturer_signatures = match manufacturer {
            "nikon" => vec![
                (
                    b"Nikon\x00\x01".to_vec(),
                    Some(1),
                    8,
                    "Nikon Type 1 maker note",
                ),
                (
                    b"Nikon\x00\x02".to_vec(),
                    Some(2),
                    10,
                    "Nikon Type 2 maker note",
                ),
                (b"Nikon".to_vec(), None, 0, "Generic Nikon maker note"),
            ],
            "canon" => vec![(b"Canon".to_vec(), None, 0, "Canon maker note")],
            "sony" => vec![
                (b"SONY DSC".to_vec(), None, 0, "Sony DSC maker note"),
                (b"SONY CAM".to_vec(), None, 0, "Sony CAM maker note"),
                (b"SONY".to_vec(), None, 0, "Generic Sony maker note"),
            ],
            "olympus" => vec![
                (
                    b"OLYMPUS\x00\x01\x00".to_vec(),
                    Some(1),
                    12,
                    "Olympus Type 1 maker note",
                ),
                (
                    b"OLYMPUS\x00\x02\x00".to_vec(),
                    Some(2),
                    12,
                    "Olympus Type 2 maker note",
                ),
                (
                    b"OLYMP\x00\x01\x00".to_vec(),
                    Some(1),
                    10,
                    "Olympus short Type 1 maker note",
                ),
                (
                    b"OLYMP\x00\x02\x00".to_vec(),
                    Some(2),
                    10,
                    "Olympus short Type 2 maker note",
                ),
            ],
            "pentax" => vec![
                (b"AOC\x00".to_vec(), None, 4, "Pentax AOC maker note"),
                (
                    b"PENTAX \x00\x01".to_vec(),
                    Some(1),
                    10,
                    "Pentax Type 1 maker note",
                ),
                (b"PENTAX".to_vec(), None, 0, "Generic Pentax maker note"),
            ],
            "panasonic" => vec![(
                b"Panasonic\x00\x00\x00".to_vec(),
                None,
                12,
                "Panasonic maker note",
            )],
            "samsung" => vec![(b"Samsung".to_vec(), None, 0, "Samsung maker note")],
            "sigma" => vec![
                (b"SIGMA\x00\x00\x00".to_vec(), None, 10, "Sigma maker note"),
                (
                    b"FOVEON\x00\x00".to_vec(),
                    None,
                    10,
                    "Sigma Foveon maker note",
                ),
            ],
            "leica" => vec![(b"LEICA\x00\x00\x00".to_vec(), None, 8, "Leica maker note")],
            "hasselblad" => vec![(b"Hasselblad".to_vec(), None, 0, "Hasselblad maker note")],
            "ricoh" => vec![
                (
                    b"RICOH\x00II".to_vec(),
                    None,
                    8,
                    "Ricoh maker note (little-endian)",
                ),
                (
                    b"RICOH\x00MM".to_vec(),
                    None,
                    8,
                    "Ricoh maker note (big-endian)",
                ),
            ],
            _ => vec![],
        };

        for (signature, version, offset, description) in manufacturer_signatures {
            patterns.push(DetectionPattern {
                signature,
                version,
                offset,
                description: description.to_string(),
                condition: None,
                source_line: 0, // Literal pattern, not from specific line
            });
        }

        Ok(patterns)
    }

    /// Parse a Perl signature string into bytes
    fn parse_signature(&self, sig_str: &str, manufacturer: &str) -> Result<Vec<u8>, String> {
        let mut signature = Vec::new();

        // Handle manufacturer-specific patterns
        match manufacturer {
            "nikon" => {
                if sig_str.contains("Nikon") {
                    signature.extend_from_slice(b"Nikon");

                    // Parse hex escape sequences like \x00\x01
                    if sig_str.contains("\\x00\\x01") {
                        signature.extend_from_slice(&[0x00, 0x01]);
                    } else if sig_str.contains("\\x00\\x02") {
                        signature.extend_from_slice(&[0x00, 0x02]);
                    }
                }
            }
            "canon" => {
                if sig_str.contains("Canon") {
                    signature.extend_from_slice(b"Canon");
                }
            }
            "olympus" => {
                if sig_str.contains("OLYMPUS") {
                    signature.extend_from_slice(b"OLYMPUS");
                } else if sig_str.contains("OLYMP") {
                    signature.extend_from_slice(b"OLYMP");
                }
            }
            "sony" => {
                if sig_str.contains("SONY") {
                    signature.extend_from_slice(b"SONY");
                }
            }
            "pentax" => {
                if sig_str.contains("PENTAX") {
                    signature.extend_from_slice(b"PENTAX");
                } else if sig_str.contains("AOC") {
                    signature.extend_from_slice(b"AOC");
                }
            }
            "ricoh" => {
                if sig_str.contains("RICOH") {
                    signature.extend_from_slice(b"RICOH");
                }
            }
            "fujifilm" => {
                if sig_str.contains("FUJIFILM") {
                    signature.extend_from_slice(b"FUJIFILM");
                }
            }
            "panasonic" => {
                if sig_str.contains("Panasonic") {
                    signature.extend_from_slice(b"Panasonic");
                }
            }
            "samsung" => {
                if sig_str.contains("Samsung") {
                    signature.extend_from_slice(b"Samsung");
                }
            }
            "sigma" => {
                if sig_str.contains("SIGMA") {
                    signature.extend_from_slice(b"SIGMA");
                } else if sig_str.contains("FOVEON") {
                    signature.extend_from_slice(b"FOVEON");
                }
            }
            _ => {
                // Generic pattern parsing - try to extract literal strings
                if let Some(start) = sig_str.find(char::is_alphabetic) {
                    let end = sig_str[start..]
                        .find(|c: char| !c.is_alphanumeric())
                        .map(|i| start + i)
                        .unwrap_or(sig_str.len());
                    signature.extend_from_slice(sig_str[start..end].as_bytes());
                }
            }
        }

        if signature.is_empty() {
            return Err(format!("Could not parse signature: {}", sig_str));
        }

        Ok(signature)
    }

    /// Generate Rust detection code for a manufacturer
    fn generate_detection_code(&self, manufacturer: &str, patterns: &[DetectionPattern]) -> String {
        let mut code = String::new();

        // File header
        code.push_str(&format!(
            "// AUTO-GENERATED by exiftool_sync extract maker-detection\n\
             // Source: third-party/exiftool/lib/Image/ExifTool/{}.pm\n\
             // Generated: {}\n\
             // DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract maker-detection`\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/{}.pm\"]\n\n\
             /// Detection patterns for {} maker notes\n\
             #[derive(Debug, Clone, PartialEq)]\n\
             pub struct {}DetectionResult {{\n\
             \x20\x20\x20\x20pub version: Option<u8>,\n\
             \x20\x20\x20\x20pub ifd_offset: usize,\n\
             \x20\x20\x20\x20pub description: String,\n\
             }}\n\n",
            manufacturer.to_uppercase(),
"2025-06-24",
            manufacturer.to_uppercase(),
            manufacturer,
            manufacturer.to_uppercase()
        ));

        // Detection function
        code.push_str(&format!(
            "/// Detect {} maker note format and extract version information\n\
             /// \n\
             /// Returns Some(DetectionResult) if this appears to be a {} maker note,\n\
             /// None otherwise.\n\
             pub fn detect_{}_maker_note(data: &[u8]) -> Option<{}DetectionResult> {{\n",
            manufacturer,
            manufacturer,
            manufacturer,
            manufacturer.to_uppercase()
        ));

        // Generate pattern matching logic
        for pattern in patterns {
            let signature_bytes = pattern
                .signature
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(", ");

            code.push_str(&format!(
                "\x20\x20\x20\x20// Pattern from source line {}: {}\n\
                 \x20\x20\x20\x20if data.len() >= {} && data.starts_with(&[{}]) {{\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20return Some({}DetectionResult {{\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20version: {},\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20ifd_offset: {},\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20description: \"{}\".to_string(),\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20}});\n\
                 \x20\x20\x20\x20}}\n\n",
                pattern.source_line,
                pattern.description,
                pattern.signature.len(),
                signature_bytes,
                manufacturer.to_uppercase(),
                pattern
                    .version
                    .map(|v| format!("Some({})", v))
                    .unwrap_or_else(|| "None".to_string()),
                pattern.offset,
                pattern.description
            ));
        }

        code.push_str("\x20\x20\x20\x20None\n}\n\n");

        // Add tests
        code.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");

        for (i, pattern) in patterns.iter().enumerate() {
            let test_name = format!("test_{}_detection_pattern_{}", manufacturer, i);
            let signature_bytes = pattern
                .signature
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(", ");

            code.push_str(&format!(
                "    #[test]\n\
                 \x20\x20\x20\x20fn {}() {{\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20let test_data = &[{}];\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20let result = detect_{}_maker_note(test_data);\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20assert!(result.is_some());\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20let detection = result.unwrap();\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20assert_eq!(detection.version, {});\n\
                 \x20\x20\x20\x20\x20\x20\x20\x20assert_eq!(detection.ifd_offset, {});\n\
                 \x20\x20\x20\x20}}\n\n",
                test_name,
                signature_bytes,
                manufacturer,
                pattern
                    .version
                    .map(|v| format!("Some({})", v))
                    .unwrap_or_else(|| "None".to_string()),
                pattern.offset
            ));
        }

        code.push_str("}\n");
        code
    }

    /// Write detection code to file system
    fn write_detection_file(&self, manufacturer: &str, code: &str) -> Result<(), String> {
        // Create manufacturer directory if it doesn't exist
        let maker_dir = format!("src/maker/{}", manufacturer);
        fs::create_dir_all(&maker_dir)
            .map_err(|e| format!("Failed to create directory {}: {}", maker_dir, e))?;

        // Write detection.rs file
        let file_path = format!("{}/detection.rs", maker_dir);
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create {}: {}", file_path, e))?;

        file.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

        println!("Generated {}", file_path);
        Ok(())
    }

    /// Generate stub detection file for manufacturers without patterns
    fn generate_stub_detection_code(&self, manufacturer: &str) -> String {
        format!(
            "// AUTO-GENERATED stub by exiftool_sync extract maker-detection\n\
             // Source: third-party/exiftool/lib/Image/ExifTool/{}.pm\n\
             // Generated: 2025-06-24\n\
             // DO NOT EDIT - Regenerate with `cargo run --bin exiftool_sync extract maker-detection`\n\n\
             #![doc = \"EXIFTOOL-SOURCE: lib/Image/ExifTool/{}.pm\"]\n\n\
             /// Detection patterns for {} maker notes\n\
             #[derive(Debug, Clone, PartialEq)]\n\
             pub struct {}DetectionResult {{\n\
             \x20\x20\x20\x20pub version: Option<u8>,\n\
             \x20\x20\x20\x20pub ifd_offset: usize,\n\
             \x20\x20\x20\x20pub description: String,\n\
             }}\n\n\
             /// Detect {} maker note format and extract version information\n\
             /// \n\
             /// Returns None - no detection patterns found for {}.\n\
             pub fn detect_{}_maker_note(_data: &[u8]) -> Option<{}DetectionResult> {{\n\
             \x20\x20\x20\x20// No detection patterns found in ExifTool source\n\
             \x20\x20\x20\x20None\n\
             }}\n\n\
             #[cfg(test)]\n\
             mod tests {{\n\
             \x20\x20\x20\x20use super::*;\n\n\
             \x20\x20\x20\x20#[test]\n\
             \x20\x20\x20\x20fn test_{}_no_detection() {{\n\
             \x20\x20\x20\x20\x20\x20\x20\x20let test_data = b\"test\";\n\
             \x20\x20\x20\x20\x20\x20\x20\x20let result = detect_{}_maker_note(test_data);\n\
             \x20\x20\x20\x20\x20\x20\x20\x20assert!(result.is_none());\n\
             \x20\x20\x20\x20}}\n\
             }}\n",
            manufacturer.to_uppercase(),
            manufacturer.to_uppercase(),
            manufacturer,
            manufacturer.to_uppercase(),
            manufacturer,
            manufacturer,
            manufacturer,
            manufacturer.to_uppercase(),
            manufacturer,
            manufacturer,
        )
    }
}

impl Extractor for MakerDetectionExtractor {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        let mut extractor = Self::new();

        // List of manufacturer modules to process
        let manufacturers = [
            "Canon.pm",
            "Nikon.pm",
            "Sony.pm",
            "Olympus.pm",
            "Pentax.pm",
            "FujiFilm.pm",
            "Panasonic.pm",
            "Samsung.pm",
            "Sigma.pm",
            "Apple.pm",
        ];

        // Process each manufacturer module
        for module_name in &manufacturers {
            let module_path = exiftool_path.join("lib/Image/ExifTool").join(module_name);

            if module_path.exists() {
                match extractor.parse_manufacturer_module(&module_path) {
                    Ok(()) => {
                        println!("Processed {}", module_name);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to process {}: {}", module_name, e);
                        continue;
                    }
                }
            } else {
                eprintln!("Warning: Module not found: {}", module_path.display());
            }
        }

        // List of all supported manufacturers
        let all_manufacturers = [
            "canon",
            "nikon",
            "sony",
            "olympus",
            "pentax",
            "fujifilm",
            "panasonic",
            "samsung",
            "sigma",
            "apple",
            "ricoh",
        ];

        // Generate detection code for each manufacturer (with patterns or stub)
        let mut generated_count = 0;
        for manufacturer in &all_manufacturers {
            if let Some(patterns) = extractor.patterns.get(*manufacturer) {
                // Generate real detection code
                let code = extractor.generate_detection_code(manufacturer, patterns);
                extractor.write_detection_file(manufacturer, &code)?;
                generated_count += 1;
            } else {
                // Generate stub detection code
                let code = extractor.generate_stub_detection_code(manufacturer);
                extractor.write_detection_file(manufacturer, &code)?;
                generated_count += 1;
            }
        }

        println!(
            "Successfully generated detection logic for {} manufacturers",
            generated_count
        );
        if !extractor.patterns.is_empty() {
            println!("  - {} with detection patterns", extractor.patterns.len());
            println!(
                "  - {} with stub implementations",
                all_manufacturers.len() - extractor.patterns.len()
            );
        }

        Ok(())
    }
}
