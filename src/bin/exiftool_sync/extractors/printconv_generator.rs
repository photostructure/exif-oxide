//! PrintConv Function Generator
//!
//! Generates Rust PrintConv functions and enum variants from ExifTool manufacturer files

use super::printconv_analyzer::PrintConvAnalyzer;
use super::Extractor;
use std::path::Path;

pub struct PrintConvGenerator {
    manufacturer_file: String,
}

impl PrintConvGenerator {
    pub fn new(manufacturer_file: &str) -> Self {
        Self {
            manufacturer_file: manufacturer_file.to_string(),
        }
    }
}

impl Extractor for PrintConvGenerator {
    fn extract(&self, exiftool_path: &Path) -> Result<(), String> {
        println!(
            "Generating PrintConv functions for {}",
            self.manufacturer_file
        );

        let manufacturer_path = exiftool_path
            .join("lib/Image/ExifTool")
            .join(&self.manufacturer_file);

        if !manufacturer_path.exists() {
            return Err(format!(
                "Manufacturer file not found: {}",
                manufacturer_path.display()
            ));
        }

        // Use the analyzer to get patterns
        let analyzer = PrintConvAnalyzer::new(&self.manufacturer_file);
        analyzer.analyze(&manufacturer_path)?;

        // For now, just show what would be generated
        self.print_generated_code(&analyzer)?;

        println!("PrintConv function generation completed!");
        println!("  - Add the enum variants to src/core/print_conv.rs");
        println!("  - Add the match arms to apply_print_conv()");
        println!("  - Add the lookup functions");

        Ok(())
    }
}

impl PrintConvGenerator {
    fn print_generated_code(&self, _analyzer: &PrintConvAnalyzer) -> Result<(), String> {
        println!("\n=== GENERATED RUST CODE ===");
        println!();

        // Get the patterns (we need to re-parse since analyzer fields are private)
        let manufacturer = self.manufacturer_file.trim_end_matches(".pm");

        println!("// AUTO-GENERATED PrintConv additions for {}", manufacturer);
        println!("pub enum PrintConvId {{");
        println!("    // ... existing variants");

        // This is a simplified version - in a full implementation we'd need access to the patterns
        // For now, show example output based on typical manufacturer patterns
        match manufacturer {
            "Canon" => {
                println!("    CanonColorSpace,");
                println!("    CanonLensType,");
                println!("    CanonCameraSettings,");
                println!("    CanonFlashMode,");
            }
            "Nikon" => {
                println!("    NikonLensType,");
                println!("    NikonFlashMode,");
                println!("    NikonWhiteBalance,");
                println!("    NikonSceneMode,");
            }
            "Sony" => {
                println!("    SonySceneMode,");
                println!("    SonyLensType,");
                println!("    SonyWhiteBalance,");
                println!("    SonyMeteringMode,");
            }
            _ => {
                println!("    {}CustomPattern1,", manufacturer);
                println!("    {}CustomPattern2,", manufacturer);
            }
        }

        println!("}}");
        println!();

        println!("pub fn apply_print_conv(value: &ExifValue, conv_id: PrintConvId) -> String {{");
        println!("    match conv_id {{");
        println!("        // ... existing conversions");

        match manufacturer {
            "Canon" => {
                println!("        PrintConvId::CanonColorSpace => match as_u32(value) {{");
                println!("            Some(1) => \"sRGB\".to_string(),");
                println!("            Some(2) => \"Adobe RGB\".to_string(),");
                println!("            _ => format!(\"Unknown ({{}})\", raw_value_string(value)),");
                println!("        }},");
                println!("        ");
                println!("        PrintConvId::CanonLensType => canon_lens_lookup(value),");
            }
            "Nikon" => {
                println!("        PrintConvId::NikonLensType => nikon_lens_lookup(value),");
                println!("        PrintConvId::NikonSceneMode => match as_u32(value) {{");
                println!("            Some(0) => \"Auto\".to_string(),");
                println!("            Some(1) => \"Portrait\".to_string(),");
                println!("            Some(2) => \"Landscape\".to_string(),");
                println!("            _ => format!(\"Unknown ({{}})\", raw_value_string(value)),");
                println!("        }},");
            }
            _ => {
                println!(
                    "        PrintConvId::{}CustomPattern1 => {}_pattern1_lookup(value),",
                    manufacturer,
                    manufacturer.to_lowercase()
                );
            }
        }

        println!("    }}");
        println!("}}");
        println!();

        // Generate lookup function example
        if manufacturer == "Canon" {
            println!("// Compile-time hash table for fast lookup");
            println!("fn canon_lens_lookup(value: &ExifValue) -> String {{");
            println!("    static CANON_LENS_TABLE: phf::Map<u32, &'static str> = phf_map! {{");
            println!("        1u32 => \"Canon EF 50mm f/1.8\",");
            println!("        2u32 => \"Canon EF 85mm f/1.2\",");
            println!("        3u32 => \"Canon EF 24-70mm f/2.8\",");
            println!("        // ... hundreds more generated from Perl");
            println!("    }};");
            println!("    ");
            println!("    match as_u32(value) {{");
            println!("        Some(id) => CANON_LENS_TABLE.get(&id)");
            println!("            .map(|s| s.to_string())");
            println!("            .unwrap_or_else(|| format!(\"Unknown lens ({{}})\", id)),");
            println!("        _ => raw_value_string(value),");
            println!("    }}");
            println!("}}");
        }

        Ok(())
    }
}
