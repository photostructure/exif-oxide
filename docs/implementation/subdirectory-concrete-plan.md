# SubDirectory Processing: Concrete Implementation Plan

## Overview

Based on our deep analysis, we can implement a **fully DRY subdirectory system** with:
- One generic processor for ALL manufacturers
- A table registry (like conv_registry) for TagTable loading  
- Generated data files with subdirectory definitions
- Minimal special-case code

## Phase 1: Codegen Infrastructure (Day 1)

### 1.1 Extract SubDirectory Definitions

Create `codegen/scripts/extract_subdirectory_defs.pl`:

```perl
#!/usr/bin/env perl
use strict;
use warnings;
use JSON;
use Data::Dumper;

# Extract subdirectory definitions from a module
sub extract_subdirectory_defs {
    my ($module_name) = @_;
    
    # Load the module
    require $module_name;
    my $table_name = $module_name . "::Main";
    my $table = eval "\\%$table_name";
    
    my %subdirectory_defs;
    
    foreach my $tag_id (keys %$table) {
        my $tag_info = $table->{$tag_id};
        
        # Skip if not numeric tag ID
        next unless $tag_id =~ /^\d+$/;
        
        my @variants;
        
        # Handle array of variants
        if (ref $tag_info eq 'ARRAY') {
            foreach my $variant (@$tag_info) {
                if ($variant->{SubDirectory}) {
                    push @variants, extract_variant($variant);
                }
            }
        }
        # Handle single definition
        elsif (ref $tag_info eq 'HASH' && $tag_info->{SubDirectory}) {
            push @variants, extract_variant($tag_info);
        }
        
        if (@variants) {
            $subdirectory_defs{sprintf("0x%04x", $tag_id)} = \@variants;
        }
    }
    
    return \%subdirectory_defs;
}

sub extract_variant {
    my ($tag_info) = @_;
    my $subdir = $tag_info->{SubDirectory};
    
    return {
        name => $tag_info->{Name},
        condition => $tag_info->{Condition},
        tag_table => $subdir->{TagTable} || '',
        validate => $subdir->{Validate},
        process_proc => $subdir->{ProcessProc},
        byte_order => $subdir->{ByteOrder},
        start => $subdir->{Start},
        base => $subdir->{Base},
    };
}

# Process all manufacturer modules
my @modules = qw(
    Image::ExifTool::Canon
    Image::ExifTool::Nikon
    Image::ExifTool::Sony
    Image::ExifTool::Olympus
    Image::ExifTool::Panasonic
    Image::ExifTool::FujiFilm
    Image::ExifTool::Pentax
    Image::ExifTool::Sigma
    Image::ExifTool::Samsung
    Image::ExifTool::Casio
);

foreach my $module (@modules) {
    my $defs = extract_subdirectory_defs($module);
    
    # Save to JSON for Rust codegen to consume
    my $output_file = $module;
    $output_file =~ s/::/_/g;
    $output_file = "extracted_data/${output_file}_subdirectory.json";
    
    open my $fh, '>', $output_file or die $!;
    print $fh encode_json($defs);
    close $fh;
}
```

### 1.2 Generate Rust SubDirectory Definitions

Update `codegen/src/strategies/subdirectory_gen.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct SubDirectoryDef {
    name: String,
    condition: Option<String>,
    tag_table: String,
    validate: Option<String>,
    process_proc: Option<String>,
    byte_order: Option<String>,
    start: Option<String>,
    base: Option<String>,
}

pub fn generate_subdirectory_defs(manufacturer: &str) -> String {
    let json_path = format!("extracted_data/Image_ExifTool_{}_subdirectory.json", manufacturer);
    let json = std::fs::read_to_string(&json_path).unwrap();
    let defs: HashMap<String, Vec<SubDirectoryDef>> = serde_json::from_str(&json).unwrap();
    
    let mut output = String::new();
    output.push_str(&format!(
        r#"//! Generated subdirectory definitions for {}
//!
//! This file is auto-generated. Do not edit manually.

use crate::exif::subdirectory::SubDirectoryDef;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static {}_SUBDIRECTORY_DEFS: LazyLock<HashMap<u16, Vec<SubDirectoryDef>>> = 
    LazyLock::new(|| HashMap::from([
"#,
        manufacturer,
        manufacturer.to_uppercase()
    ));
    
    for (tag_id, variants) in defs {
        output.push_str(&format!("        ({}, vec![\n", tag_id));
        
        for variant in variants {
            output.push_str("            SubDirectoryDef {\n");
            output.push_str(&format!("                name: \"{}\".to_string(),\n", variant.name));
            
            if let Some(cond) = &variant.condition {
                output.push_str(&format!("                condition: Some(\"{}\".to_string()),\n", 
                    cond.replace('"', "\\\"")));
            } else {
                output.push_str("                condition: None,\n");
            }
            
            output.push_str(&format!("                tag_table: \"{}\".to_string(),\n", variant.tag_table));
            // ... other fields
            output.push_str("            },\n");
        }
        
        output.push_str("        ]),\n");
    }
    
    output.push_str("    ]));\n");
    output
}
```

## Phase 2: Table Registry (Day 2)

### 2.1 Create Table Registry Module

Create `codegen/src/table_registry/mod.rs`:

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TagTableMapping {
    pub exiftool_name: String,       // "Image::ExifTool::Canon::CameraSettings"
    pub rust_module: String,         // "Canon_pm::camera_settings_tags"
    pub table_const: String,         // "CAMERA_SETTINGS_TAGS"
    pub is_binary_data: bool,
    pub format: Option<String>,
    pub first_entry: Option<i32>,
}

pub struct TagTableRegistry {
    mappings: HashMap<String, TagTableMapping>,
}

impl TagTableRegistry {
    pub fn build() -> Self {
        let mut registry = Self { mappings: HashMap::new() };
        
        // Add all known mappings
        registry.add_canon_tables();
        registry.add_nikon_tables();
        registry.add_sony_tables();
        // ... etc
        
        registry
    }
    
    fn add_canon_tables(&mut self) {
        // Main Canon tables
        self.add("Image::ExifTool::Canon::CameraSettings", TagTableMapping {
            exiftool_name: "Image::ExifTool::Canon::CameraSettings".to_string(),
            rust_module: "Canon_pm::camera_settings_tags".to_string(),
            table_const: "CAMERA_SETTINGS_TAGS".to_string(),
            is_binary_data: true,
            format: Some("int16s".to_string()),
            first_entry: Some(1),
        });
        
        // CanonCustom functions
        self.add("Image::ExifTool::CanonCustom::Functions1D", TagTableMapping {
            exiftool_name: "Image::ExifTool::CanonCustom::Functions1D".to_string(),
            rust_module: "CanonCustom_pm::functions1d_tags".to_string(),
            table_const: "FUNCTIONS1D_TAGS".to_string(),
            is_binary_data: true,
            format: Some("int16s".to_string()),
            first_entry: Some(1),
        });
        
        // ... add all other tables
    }
    
    pub fn generate_loader_code(&self) -> String {
        let mut code = String::from(
            r#"//! Generated table loader
//!
//! This file is auto-generated. Do not edit manually.

use crate::exif::subdirectory::LoadedTagTable;
use crate::types::{Result, ExifError};

pub fn load_tag_table(exiftool_name: &str) -> Result<LoadedTagTable> {
    match exiftool_name {
"#
        );
        
        for (name, mapping) in &self.mappings {
            code.push_str(&format!(
                r#"        "{}" => Ok(LoadedTagTable {{
            tags: crate::generated::{}::{}.clone(),
            is_binary_data: {},
            format: {:?},
            first_entry: {:?},
        }}),
"#,
                name,
                mapping.rust_module,
                mapping.table_const,
                mapping.is_binary_data,
                mapping.format,
                mapping.first_entry,
            ));
        }
        
        code.push_str(r#"        _ => Err(ExifError::ParseError(format!("Unknown tag table: {}", exiftool_name)))
    }
}
"#);
        
        code
    }
}
```

### 2.2 Generate Registry During Build

Update `codegen/src/main.rs`:

```rust
// Add to codegen process
let registry = table_registry::TagTableRegistry::build();
let loader_code = registry.generate_loader_code();
std::fs::write("src/generated/table_loader.rs", loader_code)?;
```

## Phase 3: Generic SubDirectory Processor (Day 3)

### 3.1 Core SubDirectory Types

Create `src/exif/subdirectory/types.rs`:

```rust
use crate::types::{TagValue, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SubDirectoryDef {
    pub name: String,
    pub condition: Option<String>,
    pub tag_table: String,
    pub validate: Option<String>,
    pub process_proc: Option<String>,
    pub byte_order: Option<String>,
    pub start: Option<String>,
    pub base: Option<String>,
}

#[derive(Debug)]
pub struct LoadedTagTable {
    pub tags: HashMap<u16, TagInfo>,
    pub is_binary_data: bool,
    pub format: Option<String>,
    pub first_entry: Option<i32>,
}

#[derive(Debug)]
pub struct ProcessingContext {
    pub model: Option<String>,
    pub make: Option<String>,
    pub firmware: Option<String>,
    pub value_data: Option<Vec<u8>>,
    pub byte_order: ByteOrder,
    pub base_offset: u32,
}
```

### 3.2 Generic Processor Implementation

Create `src/exif/subdirectory/processor.rs`:

```rust
use super::types::*;
use crate::expressions::ExpressionEvaluator;
use crate::generated::table_loader;
use crate::types::{Result, TagValue, ExifError};
use std::collections::HashMap;
use tracing::{debug, trace};

pub struct SubDirectoryProcessor {
    evaluator: ExpressionEvaluator,
}

impl SubDirectoryProcessor {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
        }
    }
    
    /// Process any subdirectory from any manufacturer
    pub fn process(
        &mut self,
        tag_id: u16,
        tag_value: &TagValue,
        subdirectory_defs: &[SubDirectoryDef],
        context: &ProcessingContext,
    ) -> Result<HashMap<String, TagValue>> {
        debug!("Processing subdirectory for tag 0x{:04x} with {} variants", 
               tag_id, subdirectory_defs.len());
        
        // 1. Select the correct variant based on conditions
        let variant = self.select_variant(subdirectory_defs, context)?;
        debug!("Selected variant: {} (table: {})", variant.name, variant.tag_table);
        
        // 2. Validate if needed
        if let Some(validate_expr) = &variant.validate {
            if !self.validate_data(validate_expr, tag_value, context)? {
                return Err(ExifError::ParseError(
                    format!("Validation failed for {}", variant.name)
                ));
            }
        }
        
        // 3. Load the tag table
        let table = table_loader::load_tag_table(&variant.tag_table)?;
        debug!("Loaded table: {} (binary_data: {})", 
               variant.tag_table, table.is_binary_data);
        
        // 4. Process based on table type
        let data = tag_value.as_bytes()
            .ok_or_else(|| ExifError::ParseError("Expected binary data".into()))?;
        
        if table.is_binary_data {
            self.process_binary_data(&table, data, context)
        } else {
            self.process_ifd(&table, data, context)
        }
    }
    
    fn select_variant(
        &mut self,
        defs: &[SubDirectoryDef],
        context: &ProcessingContext,
    ) -> Result<&SubDirectoryDef> {
        for def in defs {
            if let Some(condition) = &def.condition {
                trace!("Evaluating condition: {}", condition);
                
                // Evaluate using our expression system
                let matches = self.evaluator.evaluate_subdirectory_condition(
                    condition,
                    context.model.as_deref(),
                    context.make.as_deref(),
                    context.value_data.as_deref(),
                    None,
                )?;
                
                if matches {
                    debug!("Condition matched: {}", condition);
                    return Ok(def);
                }
            } else {
                // No condition means always match
                debug!("No condition, using default variant: {}", def.name);
                return Ok(def);
            }
        }
        
        Err(ExifError::ParseError("No matching subdirectory variant".into()))
    }
    
    fn process_binary_data(
        &self,
        table: &LoadedTagTable,
        data: &[u8],
        context: &ProcessingContext,
    ) -> Result<HashMap<String, TagValue>> {
        use crate::binary_data::process_binary_data_table;
        
        // Use existing binary data processing
        process_binary_data_table(
            data,
            &table.tags,
            table.format.as_deref().unwrap_or("int8u"),
            table.first_entry.unwrap_or(0),
            context.byte_order,
        )
    }
    
    fn process_ifd(
        &self,
        table: &LoadedTagTable,
        data: &[u8],
        context: &ProcessingContext,
    ) -> Result<HashMap<String, TagValue>> {
        // Process as IFD structure
        // This would use existing IFD processing logic
        todo!("IFD processing not yet implemented")
    }
    
    fn validate_data(
        &self,
        validate_expr: &str,
        tag_value: &TagValue,
        context: &ProcessingContext,
    ) -> Result<bool> {
        // Simple validation for now
        // Full implementation would evaluate the validation expression
        debug!("Validation expression: {}", validate_expr);
        Ok(true) // Placeholder
    }
}
```

## Phase 4: Integration (Day 4)

### 4.1 Update ExifReader

Modify `src/exif/mod.rs`:

```rust
impl ExifReader {
    pub fn process_subdirectories(&mut self) -> Result<()> {
        // Get context from extracted tags
        let context = ProcessingContext {
            model: self.get_tag_string("Model"),
            make: self.get_tag_string("Make"),
            firmware: self.get_tag_string("FirmwareVersion"),
            byte_order: self.byte_order,
            base_offset: self.base_offset,
            value_data: None,
        };
        
        let mut processor = SubDirectoryProcessor::new();
        
        // Process each manufacturer
        self.process_manufacturer_subdirectories(
            &mut processor, 
            &context,
            "Canon",
            &CANON_SUBDIRECTORY_DEFS,
        )?;
        
        self.process_manufacturer_subdirectories(
            &mut processor,
            &context,
            "Nikon",
            &NIKON_SUBDIRECTORY_DEFS,
        )?;
        
        // ... other manufacturers
        
        Ok(())
    }
    
    fn process_manufacturer_subdirectories(
        &mut self,
        processor: &mut SubDirectoryProcessor,
        context: &ProcessingContext,
        manufacturer: &str,
        defs: &HashMap<u16, Vec<SubDirectoryDef>>,
    ) -> Result<()> {
        // Find tags from this manufacturer
        let tags_to_process: Vec<_> = self.extracted_tags
            .iter()
            .filter(|((id, ns), _)| ns == manufacturer)
            .filter(|(id, _)| defs.contains_key(&id.0))
            .collect();
        
        for ((tag_id, _), tag_value) in tags_to_process {
            if let Some(subdirectory_defs) = defs.get(&tag_id) {
                match processor.process(
                    tag_id,
                    tag_value,
                    subdirectory_defs,
                    context,
                ) {
                    Ok(extracted) => {
                        // Store extracted tags
                        for (name, value) in extracted {
                            self.store_subdirectory_tag(&name, value);
                        }
                        // Remove original binary array
                        self.extracted_tags.remove(&(tag_id, manufacturer.to_string()));
                    }
                    Err(e) => {
                        debug!("Failed to process subdirectory: {}", e);
                        // Keep original data on failure
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## Phase 5: Special Processors (Day 5)

### 5.1 Registry of Special Processors

Create `src/exif/subdirectory/special_processors.rs`:

```rust
pub enum ProcessProc {
    Standard,
    NikonEncrypted,
    CanonSerialData,
    SonyDeciphered,
}

pub fn get_process_proc(name: Option<&str>) -> ProcessProc {
    match name {
        Some("\\&Image::ExifTool::Nikon::ProcessNikonEncrypted") => {
            ProcessProc::NikonEncrypted
        }
        Some("\\&ProcessSerialData") => {
            ProcessProc::CanonSerialData
        }
        _ => ProcessProc::Standard,
    }
}

pub fn process_nikon_encrypted(
    data: &[u8],
    context: &ProcessingContext,
) -> Result<HashMap<String, TagValue>> {
    // Implement Nikon decryption
    todo!("Nikon encryption not yet implemented")
}
```

## Deliverables Summary

### New Files
1. `codegen/scripts/extract_subdirectory_defs.pl` - Extract definitions from ExifTool
2. `codegen/src/table_registry/` - Table registry system
3. `src/exif/subdirectory/processor.rs` - Generic processor
4. `src/exif/subdirectory/types.rs` - Common types
5. `src/generated/table_loader.rs` - Generated table loader
6. `src/generated/*/subdirectory_defs.rs` - Generated definitions per manufacturer

### Modified Files
1. `codegen/src/main.rs` - Add subdirectory generation
2. `src/exif/mod.rs` - Integration point
3. `src/expressions/mod.rs` - Enhanced condition evaluation

### Key Benefits
- **ONE processor for ALL manufacturers** - Ultimate DRY
- **Data-driven, not code-driven** - Easy to update from ExifTool
- **Registry pattern** - Similar to successful conv_registry
- **Generated code** - Automatic updates from ExifTool
- **Minimal maintenance** - Changes only needed for new special cases

## Testing Strategy

1. **Unit tests** for condition evaluation
2. **Integration tests** with real camera files
3. **Comparison with ExifTool** output
4. **Performance benchmarks**

## Success Metrics

- Canon EOS 5D selects Functions5D table ✓
- Canon EOS-1D selects Functions1D table ✓
- All subdirectory tags match ExifTool output ✓
- No performance regression ✓
- Code is DRY and maintainable ✓