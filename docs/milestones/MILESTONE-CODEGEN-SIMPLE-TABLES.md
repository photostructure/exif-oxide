# MILESTONE: Generalized Simple Table Extraction Framework

## Summary

Create a systematic, configuration-driven framework to automatically extract and generate hundreds of simple lookup tables from ExifTool source across all manufacturers. This milestone establishes a scalable pattern for harvesting primitive key-value tables (lens databases, mode settings, model IDs, etc.) while strictly avoiding complex Perl logic.

## Problem Statement

ExifTool contains hundreds of simple lookup tables across 217 modules that provide valuable metadata conversion capabilities:

- **Canon.pm**: `%canonLensTypes` (534 entries), `%canonModelID` (354 entries), `%canonWhiteBalance` (22 entries)
- **Nikon.pm**: `%nikonLensIDs` (618 entries), `%nikonTextEncoding`, lens mode tables
- **Sony.pm**: Lens types, model IDs, camera settings
- **All manufacturers**: Picture styles, quality settings, feature mappings

Currently, these require manual transcription, creating maintenance burden and limiting coverage.

## Trust ExifTool Principle

This milestone exemplifies two core principles:

1. **Trust ExifTool completely** (TRUST-EXIFTOOL.md)

2. **Only perl can parse perl** 

 We extract simple tables verbatim while strictly avoiding Perl parsing:

**Safe to Extract** ✅:

```perl
%canonWhiteBalance = (
    0 => 'Auto',
    1 => 'Daylight',
    2 => 'Cloudy',
    3 => 'Tungsten',
    4 => 'Fluorescent',
);
```

**Not applicable** ❌:

```perl
0xd => [
    {
        Name => 'CanonCameraInfo1D',
        Condition => '$$self{Model} =~ /\b1DS?$/',
        SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraInfo1D' },
    },
];
```

**Source References**: All major manufacturer modules in `third-party/exiftool/lib/Image/ExifTool/`

## Architecture Integration

This framework extends our established codegen architecture to handle manufacturer-specific lookup data:

```
ExifTool Modules → Config-Driven Extractor → JSON → Rust Codegen → Generated Tables
                                                          ↓
                                              Implementation Palette (PrintConv/ValueConv)
```

### Consistency with Existing Patterns

- **HashMap + LazyLock**: Matches current codegen output patterns
- **Manufacturer Modules**: Organized in `src/generated/canon/`, `src/generated/nikon/`, etc.
- **Runtime References**: Integration with PrintConv/ValueConv system
- **JSON Intermediate**: Follows existing extraction pipeline

## Phase 1: Framework Foundation

### Deliverable: Configuration-Driven Extraction System

Create the core framework components for systematic table extraction.

#### Key Requirements

1. **Table Configuration File**: Declarative mapping of tables to extract
2. **Generic Perl Extractor**: Safe extraction with validation
3. **Rust Code Generator**: Type-safe HashMap generation
4. **Integration Points**: PrintConv/ValueConv usage patterns

#### Configuration File Format

Create `codegen/simple_tables.json`:

```json
{
  "$schema": "./simple_tables_schema.json",
  "description": "Configuration for ExifTool simple table extraction",
  "tables": [
    {
      "module": "Canon.pm",
      "hash_name": "%canonLensTypes",
      "output_file": "canon/lenses.rs", 
      "constant_name": "CANON_LENS_TYPES",
      "key_type": "f32",
      "description": "Canon lens type lookup by ID"
    },
    {
      "module": "Canon.pm",
      "hash_name": "%canonModelID",
      "output_file": "canon/models.rs",
      "constant_name": "CANON_MODEL_ID", 
      "key_type": "u32",
      "description": "Canon camera model identification"
    },
    {
      "module": "Canon.pm", 
      "hash_name": "%canonWhiteBalance",
      "output_file": "canon/white_balance.rs",
      "constant_name": "CANON_WHITE_BALANCE",
      "key_type": "u8",
      "description": "White balance mode names"
    },
    {
      "module": "Canon.pm",
      "hash_name": "%pictureStyles", 
      "output_file": "canon/picture_styles.rs",
      "constant_name": "PICTURE_STYLES",
      "key_type": "u8",
      "description": "Picture style mode names"
    },
    {
      "module": "Canon.pm",
      "hash_name": "%canonImageSize",
      "output_file": "canon/image_size.rs", 
      "constant_name": "CANON_IMAGE_SIZE",
      "key_type": "u8", 
      "description": "Image size setting names"
    },
    {
      "module": "Canon.pm",
      "hash_name": "%canonQuality",
      "output_file": "canon/quality.rs",
      "constant_name": "CANON_QUALITY", 
      "key_type": "u8",
      "description": "Image quality setting names"
    },
    {
      "module": "Nikon.pm",
      "hash_name": "%nikonLensIDs", 
      "output_file": "nikon/lenses.rs",
      "constant_name": "NIKON_LENS_IDS",
      "key_type": "String",
      "description": "Nikon lens identification database"
    },
    {
      "module": "Nikon.pm",
      "hash_name": "%nikonTextEncoding",
      "output_file": "nikon/text_encoding.rs",
      "constant_name": "NIKON_TEXT_ENCODING", 
      "key_type": "u8",
      "description": "Text encoding lookup"
    },
    {
      "module": "Sony.pm", 
      "hash_name": "%sonyLensTypes",
      "output_file": "sony/lenses.rs",
      "constant_name": "SONY_LENS_TYPES",
      "key_type": "u16", 
      "description": "Sony lens type identification"
    }
  ]
}
```

And `codegen/simple_tables_schema.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Simple Tables Configuration",
  "description": "Schema for configuring ExifTool simple table extraction",
  "type": "object",
  "properties": {
    "description": {
      "type": "string",
      "description": "Human-readable description of this configuration"
    },
    "tables": {
      "type": "array",
      "description": "List of tables to extract",
      "items": {
        "type": "object",
        "properties": {
          "module": {
            "type": "string", 
            "pattern": "^\\w+\\.pm$",
            "description": "ExifTool module filename (e.g. Canon.pm)"
          },
          "hash_name": {
            "type": "string",
            "pattern": "^%\\w+$", 
            "description": "Perl hash variable name (e.g. %canonLensTypes)"
          },
          "output_file": {
            "type": "string",
            "pattern": "^[\\w/]+\\.rs$",
            "description": "Output Rust file path relative to src/generated/"
          },
          "constant_name": {
            "type": "string",
            "pattern": "^[A-Z_]+$",
            "description": "Rust constant name (SCREAMING_SNAKE_CASE)"
          },
          "key_type": {
            "type": "string", 
            "enum": ["u8", "u16", "u32", "f32", "String"],
            "description": "Rust type for hash keys"
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "description": "Human-readable description of this table"
          }
        },
        "required": ["module", "hash_name", "output_file", "constant_name", "key_type", "description"],
        "additionalProperties": false
      }
    }
  },
  "required": ["tables"],
  "additionalProperties": false
}
```

#### Generic Perl Extractor

Create `codegen/extract_simple_tables.pl`:

```perl
#!/usr/bin/env perl
use strict;
use warnings;
use JSON qw(encode_json decode_json);
use FindBin qw($Bin);

# Read configuration file
sub load_table_config {
    my $config_file = "$Bin/simple_tables.json";
    
    open(my $fh, '<', $config_file) or die "Cannot open $config_file: $!";
    my $json_text = do { local $/; <$fh> };
    close($fh);
    
    my $config = decode_json($json_text);
    
    return @{$config->{tables}};
}

# Extract a single table from ExifTool module
sub extract_table {
    my ($module_file, $hash_name) = @_;

    open(my $fh, '<', $module_file) or die "Cannot open $module_file: $!";
    my @lines = <$fh>;
    close($fh);

    my @entries;
    my $in_target_hash = 0;
    my $brace_count = 0;

    for my $line_num (0..$#lines) {
        my $line = $lines[$line_num];

        # Detect target hash start
        if ($line =~ /^\Q$hash_name\E\s*=\s*\(/) {
            $in_target_hash = 1;
            $brace_count = 1;
            next;
        }

        next unless $in_target_hash;

        # Track brace nesting for complex hashes
        $brace_count += ($line =~ tr/\(/\(/);
        $brace_count -= ($line =~ tr/\)/\)/);

        # Extract simple key-value pairs
        if ($line =~ /^\s*([+-]?\d*\.?\d+|'[^']*'|"[^"]*")\s*=>\s*'([^']*)'/) {
            my ($key, $value) = ($1, $2);

            # Validate primitiveness - no variables or expressions
            next if $key =~ /\$/;     # No Perl variables
            next if $value =~ /\$/;   # No variable interpolation
            next if $line =~ /\\\w/;  # No escape sequences beyond basic ones

            # Clean up key (remove quotes if string)
            $key =~ s/^'([^']*)'$/$1/;
            $key =~ s/^"([^"]*)"$/$1/;

            push @entries, {
                key => $key,
                value => $value,
                source_line => $line_num + 1,
                raw_line => $line,
            };
        }

        # Check for hash end
        if ($brace_count <= 0) {
            last;
        }
    }

    return @entries;
}

# Validate table is truly primitive (no Perl logic)
sub validate_primitive_table {
    my @entries = @_;

    for my $entry (@entries) {
        # Check for Perl expressions in keys or values
        return 0 if $entry->{key} =~ /[{}|\[\]\\]/;     # Complex structures
        return 0 if $entry->{value} =~ /\$|\@|%/;       # Variables
        return 0 if $entry->{raw_line} =~ /=>/&&$entry->{raw_line} =~ /[{}]/; # Nested structures
    }

    return 1;
}

# Main extraction logic
sub main {
    my @table_configs = load_table_config();
    my %extracted_data;

    for my $config (@table_configs) {
        my $module_file = "$Bin/../third-party/exiftool/lib/Image/ExifTool/$config->{module}";

        print STDERR "Extracting $config->{hash_name} from $config->{module}...\n";

        my @entries = extract_table($module_file, $config->{hash_name});

        if (!validate_primitive_table(@entries)) {
            warn "WARNING: $config->{hash_name} contains non-primitive data, skipping\n";
            next;
        }

        if (@entries == 0) {
            warn "WARNING: No entries found for $config->{hash_name}\n";
            next;
        }

        $extracted_data{$config->{hash_name}} = {
            config => $config,
            entries => \@entries,
            entry_count => scalar @entries,
        };

        print STDERR "  Extracted " . scalar(@entries) . " entries\n";
    }

    # Output unified JSON
    print encode_json({
        extracted_at => scalar(gmtime()),
        extraction_config => "simple_tables.json",
        total_tables => scalar(keys %extracted_data),
        tables => \%extracted_data,
    });
}

main();
```

#### Enhanced Rust Code Generator

Extend `codegen/src/main.rs`:

```rust
// Add to existing structures
#[derive(Debug, Deserialize)]
struct SimpleTablesData {
    extracted_at: String,
    extraction_config: String,
    total_tables: usize,
    tables: HashMap<String, ExtractedTable>,
}

#[derive(Debug, Deserialize)]
struct ExtractedTable {
    config: TableConfig,
    entries: Vec<TableEntry>,
    entry_count: usize,
}

#[derive(Debug, Deserialize)]
struct TableConfig {
    module: String,
    hash_name: String,
    output_file: String,
    constant_name: String,
    key_type: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct TableEntry {
    key: String,
    value: String,
    source_line: u32,
    raw_line: String,
}

// Add to main() function
fn main() -> Result<()> {
    // ... existing code ...

    // Generate simple tables if data exists
    let simple_tables_path = "generated/simple_tables.json";
    if Path::new(simple_tables_path).exists() {
        generate_simple_tables(simple_tables_path, output_dir)?;
    }

    // ... rest of main ...
}

fn generate_simple_tables(input_path: &str, output_dir: &str) -> Result<()> {
    let json_content = fs::read_to_string(input_path)?;
    let tables_data: SimpleTablesData = serde_json::from_str(&json_content)?;

    for (hash_name, table_data) in &tables_data.tables {
        let config = &table_data.config;

        // Create manufacturer directory
        let output_path = Path::new(output_dir).join(&config.output_file);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate Rust code for this table
        let code = generate_table_code(hash_name, table_data)?;
        fs::write(&output_path, code)?;

        println!("Generated: {} ({} entries)", config.output_file, table_data.entry_count);
    }

    println!("Generated {} simple tables total", tables_data.total_tables);
    Ok(())
}

fn generate_table_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let config = &table_data.config;
    let mut code = String::new();

    // File header
    code.push_str(&format!(
        "//! Generated {} lookup table\n//!\n//! This file is automatically generated.\n//! DO NOT EDIT MANUALLY - changes will be overwritten.\n//!\n//! Source: ExifTool {} {}\n//! Description: {}\n\n",
        config.constant_name.to_lowercase(),
        config.module,
        hash_name,
        config.description
    ));

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use std::sync::LazyLock;\n\n");

    // Determine HashMap type based on key_type
    let key_rust_type = match config.key_type.as_str() {
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "f32" => "f32",
        "String" => "&'static str",
        _ => "&'static str", // Default fallback
    };

    // Generate static HashMap
    code.push_str(&format!(
        "/// {} lookup table\n/// Source: ExifTool {} {} ({} entries)\npub static {}: LazyLock<HashMap<{}, &'static str>> = LazyLock::new(|| {{\n",
        config.description,
        config.module,
        hash_name,
        table_data.entry_count,
        config.constant_name,
        key_rust_type
    ));

    code.push_str("    let mut map = HashMap::new();\n");

    // Add entries
    for entry in &table_data.entries {
        let key_value = if config.key_type == "String" {
            format!("\"{}\"", entry.key)
        } else {
            entry.key.clone()
        };

        code.push_str(&format!(
            "    map.insert({}, \"{}\"); // ExifTool {}:{}\n",
            key_value,
            escape_string(&entry.value),
            config.module,
            entry.source_line
        ));
    }

    code.push_str("    map\n});\n\n");

    // Generate lookup function
    let fn_name = config.constant_name.to_lowercase();
    code.push_str(&format!(
        "/// Look up {} value by key\npub fn lookup_{}(key: {}) -> Option<&'static str> {{\n",
        config.description.to_lowercase(),
        fn_name,
        key_rust_type
    ));
    code.push_str(&format!("    {}.get(&key).copied()\n", config.constant_name));
    code.push_str("}\n");

    Ok(code)
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
```

## Phase 2: Nikon Lens Database Implementation

### Deliverable: First Working Example

Implement the Nikon lens database extraction as the proving ground for the framework.

#### Implementation Tasks

1. **Configure Nikon Lens Table**: Add to `simple_tables.conf`
2. **Test Extraction**: Verify 618 entries extracted correctly
3. **Generate Rust Code**: Create `src/generated/nikon/lenses.rs`
4. **Integration**: Update `src/implementations/nikon/lens_database.rs`
5. **Validation**: Comprehensive test coverage

#### Example Generated Code

Target: `src/generated/nikon/lenses.rs`

```rust
//! Generated NIKON_LENS_IDS lookup table
//!
//! This file is automatically generated.
//! DO NOT EDIT MANUALLY - changes will be overwritten.
//!
//! Source: ExifTool Nikon.pm %nikonLensIDs
//! Description: Nikon lens identification database

use std::collections::HashMap;
use std::sync::LazyLock;

/// Nikon lens identification database lookup table
/// Source: ExifTool Nikon.pm %nikonLensIDs (618 entries)
pub static NIKON_LENS_IDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("01 58 50 50 14 14 02 00", "AF Nikkor 50mm f/1.8"); // ExifTool Nikon.pm:96
    map.insert("01 58 50 50 14 14 05 00", "AF Nikkor 50mm f/1.8"); // ExifTool Nikon.pm:97
    map.insert("02 42 44 5C 2A 34 02 00", "AF Zoom-Nikkor 35-70mm f/3.3-4.5"); // ExifTool Nikon.pm:98
    // ... 618 total entries
    map
});

/// Look up nikon lens identification database value by key
pub fn lookup_nikon_lens_ids(key: &str) -> Option<&'static str> {
    NIKON_LENS_IDS.get(key).copied()
}
```

#### Integration Example

Update `src/implementations/nikon/lens_database.rs`:

```rust
//! Nikon lens database and identification system
//!
//! **Trust ExifTool**: This code uses generated ExifTool data verbatim.

use crate::generated::nikon::lenses::{NIKON_LENS_IDS, lookup_nikon_lens_ids};
use tracing::{debug, trace, warn};

/// Look up Nikon lens by 8-byte lens data
/// ExifTool: Nikon.pm LensIDConv function
pub fn lookup_nikon_lens(lens_data: &[u8]) -> Option<String> {
    if lens_data.len() < 8 {
        warn!("Insufficient lens data for Nikon lookup: {} bytes", lens_data.len());
        return None;
    }

    // Format 8-byte lens data as hex string pattern
    let id_pattern = format!(
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
        lens_data[0], lens_data[1], lens_data[2], lens_data[3],
        lens_data[4], lens_data[5], lens_data[6], lens_data[7]
    );

    trace!("Looking up Nikon lens with pattern: {}", id_pattern);

    // Use generated lookup table
    if let Some(description) = lookup_nikon_lens_ids(&id_pattern) {
        debug!("Found Nikon lens match: {} -> {}", id_pattern, description);
        return Some(description.to_string());
    }

    debug!("No Nikon lens match found for pattern: {}", id_pattern);
    None
}

// Keep existing API functions for backward compatibility
// ... rest of existing implementation
```

## Phase 3: Canon Tables Implementation

### Deliverable: Multi-Table Manufacturer Support

Expand to Canon's multiple lookup tables, proving the framework scales.

#### Target Tables

Based on your research:

- `%canonLensTypes` (534 entries) - High value lens identification
- `%canonModelID` (354 entries) - Camera model identification
- `%canonWhiteBalance` (22 entries) - White balance mode names
- `%pictureStyles` (24 entries) - Picture style names
- `%canonImageSize` (19 entries) - Image size settings
- `%canonQuality` (9 entries) - Quality settings

#### Implementation Tasks

1. **Add Canon Config**: Configure all 6 Canon tables in `simple_tables.conf`
2. **Batch Extraction**: Test framework handles multiple tables from same module
3. **Generated Modules**: Create organized `src/generated/canon/` structure
4. **PrintConv Integration**: Use generated tables in Canon PrintConv implementations
5. **Validation**: Ensure all 962 total entries extracted correctly

#### Generated Module Structure

```
src/generated/canon/
├── lenses.rs           # CANON_LENS_TYPES (534 entries)
├── models.rs           # CANON_MODEL_ID (354 entries)
├── white_balance.rs    # CANON_WHITE_BALANCE (22 entries)
├── picture_styles.rs   # PICTURE_STYLES (24 entries)
├── image_size.rs       # CANON_IMAGE_SIZE (19 entries)
├── quality.rs          # CANON_QUALITY (9 entries)
└── mod.rs              # Module exports
```

#### PrintConv Integration Example

```rust
// In src/implementations/print_conv.rs
use crate::generated::canon::{
    white_balance::lookup_canon_white_balance,
    picture_styles::lookup_picture_styles,
    quality::lookup_canon_quality,
};

pub fn canon_white_balance_print_conv(value: &TagValue) -> Result<String> {
    if let Some(wb_value) = value.as_u8() {
        if let Some(description) = lookup_canon_white_balance(wb_value) {
            return Ok(description.to_string());
        }
    }
    Ok(format!("Unknown ({})", value))
}

pub fn canon_picture_style_print_conv(value: &TagValue) -> Result<String> {
    if let Some(style_value) = value.as_u8() {
        if let Some(description) = lookup_picture_styles(style_value) {
            return Ok(description.to_string());
        }
    }
    Ok(format!("Unknown ({})", value))
}
```

## Phase 4: Multi-Manufacturer Expansion

### Deliverable: Systematic Coverage

Expand framework to all major manufacturers with simple tables.

#### Target Expansion

Survey and implement tables from:

- **Sony.pm**: Lens types, model IDs, camera modes
- **Panasonic.pm**: Lens databases, quality settings
- **Olympus.pm**: Lens identification, camera modes
- **Pentax.pm**: Lens types, model mappings
- **Samsung.pm**: Camera settings, mode tables

#### Estimated Scale

- **Total potential tables**: 50-100 simple lookup tables
- **Total entries**: 3,000-5,000 lookup entries
- **Manufacturer coverage**: All major camera brands
- **PrintConv coverage**: Hundreds of conversion functions automated

#### Implementation Tasks

1. **Manufacturer Survey**: Identify all simple tables across manufacturer modules
2. **Config Expansion**: Add discovered tables to `simple_tables.conf`
3. **Bulk Generation**: Prove framework handles dozens of tables efficiently
4. **Integration Testing**: Ensure generated tables work across all PrintConv implementations
5. **Performance Validation**: Maintain fast lookup times with larger datasets

## Phase 5: Build Integration and Validation

### Deliverable: Production-Ready Pipeline

Complete build system integration and comprehensive validation.

#### Enhanced Build Process

Update Makefile:

```makefile
# Enhanced codegen targets
.PHONY: codegen-simple-tables codegen-full test-simple-tables

# Extract simple tables from ExifTool
codegen-simple-tables:
	@echo "Extracting simple tables from ExifTool..."
	cd codegen && perl extract_simple_tables.pl > generated/simple_tables.json
	@echo "Generated: codegen/generated/simple_tables.json"

# Enhanced codegen target
codegen-full: codegen-simple-tables
	@echo "Generating Rust code from ExifTool extractions..."
	cd codegen && perl extract_tables.pl > generated/tag_tables.json
	cd codegen && cargo run
	@echo "Code generation complete"

# Test simple table functionality
test-simple-tables:
	@echo "Testing simple table lookups..."
	cargo test simple_tables
	cargo test --test table_integration
	@echo "Simple table tests passed"
```

#### Comprehensive Test Suite

```rust
// tests/simple_tables_integration.rs
#[cfg(test)]
mod simple_table_tests {
    use exif_oxide::generated::canon::*;
    use exif_oxide::generated::nikon::*;

    #[test]
    fn test_nikon_lens_database_completeness() {
        use nikon::lenses::NIKON_LENS_IDS;

        // Should have exactly 618 entries from ExifTool
        assert_eq!(NIKON_LENS_IDS.len(), 618);

        // Test known entries
        assert_eq!(
            NIKON_LENS_IDS.get("01 58 50 50 14 14 02 00"),
            Some(&"AF Nikkor 50mm f/1.8")
        );
    }

    #[test]
    fn test_canon_tables_coverage() {
        use canon::{lenses::CANON_LENS_TYPES, models::CANON_MODEL_ID};

        // Should have expected entry counts
        assert_eq!(CANON_LENS_TYPES.len(), 534);
        assert_eq!(CANON_MODEL_ID.len(), 354);

        // Test sample entries
        assert!(CANON_LENS_TYPES.get(&1.0).is_some());
        assert!(CANON_MODEL_ID.get(&1).is_some());
    }

    #[test]
    fn test_lookup_functions() {
        use canon::white_balance::lookup_canon_white_balance;
        use nikon::lenses::lookup_nikon_lens_ids;

        // Test Canon white balance lookup
        assert_eq!(lookup_canon_white_balance(0), Some("Auto"));
        assert_eq!(lookup_canon_white_balance(1), Some("Daylight"));
        assert_eq!(lookup_canon_white_balance(255), None);

        // Test Nikon lens lookup
        assert_eq!(
            lookup_nikon_lens_ids("01 58 50 50 14 14 02 00"),
            Some("AF Nikkor 50mm f/1.8")
        );
        assert_eq!(lookup_nikon_lens_ids("FF FF FF FF FF FF FF FF"), None);
    }

    #[test]
    fn test_performance_benchmarks() {
        use std::time::Instant;
        use canon::lenses::lookup_canon_lens_types;

        let start = Instant::now();

        // Perform 10,000 lookups
        for i in 0..10000 {
            lookup_canon_lens_types(i as f32);
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "10K lookups should complete in <100ms");
    }

    #[test]
    fn test_all_generated_modules_compile() {
        // This test ensures all generated modules compile and export correctly
        use canon::*;
        use nikon::*;
        // Add other manufacturers as implemented

        // If this compiles, all generated modules are syntactically correct
        assert!(true);
    }
}
```

#### Performance Requirements

- **Lookup Time**: < 100ns per table lookup (HashMap efficiency)
- **Memory Usage**: < 1MB total for all simple tables
- **Startup Time**: < 10ms for all LazyLock initializations
- **Build Time**: < 30s additional codegen time for all tables

## Success Criteria

### Functional Requirements

- [ ] **Framework Completeness**: Config-driven extraction for any simple table
- [ ] **Nikon Implementation**: 618-entry lens database fully generated and integrated
- [ ] **Canon Implementation**: All 6 tables (962 entries total) generated and usable
- [ ] **Multi-Manufacturer**: Framework proven across 3+ camera manufacturers
- [ ] **PrintConv Integration**: Generated tables usable in conversion functions
- [ ] **API Consistency**: All tables follow same HashMap + lookup function pattern

### Technical Requirements

- [ ] **Type Safety**: Proper Rust types for all key-value combinations
- [ ] **Generated Code Quality**: Clean, documented, readable output
- [ ] **Build Integration**: Automated extraction in codegen pipeline
- [ ] **Test Coverage**: >95% code coverage with integration tests
- [ ] **Performance**: Sub-microsecond lookup times across all tables
- [ ] **Documentation**: Complete ExifTool source references for every entry

### Quality Requirements

- [ ] **Trust ExifTool**: Verbatim extraction with zero interpretation
- [ ] **Maintainable**: Easy addition of new tables via config
- [ ] **Traceable**: Every generated entry references ExifTool source line
- [ ] **Robust**: Safe validation prevents extraction of complex tables
- [ ] **Scalable**: Framework handles hundreds of tables efficiently

## Benefits and Impact

### Coverage Expansion

- **From**: Manual transcription of individual high-value tables
- **To**: Systematic harvesting of hundreds of manufacturer tables
- **Impact**: 10x+ increase in metadata conversion coverage

### Maintenance Efficiency

- **From**: Manual updates with each ExifTool release
- **To**: Automated regeneration of all simple tables
- **Impact**: Near-zero maintenance overhead for simple lookups

### Implementation Velocity

- **From**: Days to implement each manufacturer's tables
- **To**: Minutes to configure and generate new tables
- **Impact**: Enables rapid expansion to all camera manufacturers

### Quality Consistency

- **From**: Error-prone manual transcription
- **To**: Verified extraction with ExifTool source references
- **Impact**: Perfect fidelity with automatic updates

## Related Documentation

### Architecture References

- **[CODEGEN-STRATEGY.md](../design/CODEGEN-STRATEGY.md)**: Core codegen philosophy and established patterns
- **[ARCHITECTURE.md](../ARCHITECTURE.md)**: Overall system architecture and design principles
- **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)**: Fundamental principle for ExifTool data extraction
- **[IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md)**: Manual implementation integration patterns

### Technical Guides

- **[DEVELOPMENT-WORKFLOW.md](../guides/DEVELOPMENT-WORKFLOW.md)**: Day-to-day development processes
- **[EXIFTOOL-UPDATE-WORKFLOW.md](../guides/EXIFTOOL-UPDATE-WORKFLOW.md)**: Updating with new ExifTool versions
- **[READING-EXIFTOOL-SOURCE.md](../guides/READING-EXIFTOOL-SOURCE.md)**: Understanding ExifTool Perl code structure

### Current Implementation

- **[src/implementations/nikon/lens_database.rs](../../src/implementations/nikon/lens_database.rs)**: Current manual lens database (target for replacement)
- **[codegen/extract_tables.pl](../../codegen/extract_tables.pl)**: Existing tag extraction patterns
- **[codegen/src/main.rs](../../codegen/src/main.rs)**: Current Rust code generation framework

## ExifTool Source References

### Primary Data Sources

All manufacturer modules in `third-party/exiftool/lib/Image/ExifTool/`:

- **Canon.pm**: 534 lens types, 354 model IDs, plus mode/setting tables
- **Nikon.pm**: 618 lens IDs, text encoding, camera mode tables
- **Sony.pm**: Lens types, model identification, camera settings
- **Panasonic.pm**: Lens databases, quality settings, mode tables
- **Olympus.pm**: Lens identification, camera mode definitions
- **Pentax.pm**: Lens types, model mappings, setting tables
- **Samsung.pm**: Camera settings, mode configuration tables

### Table Pattern Examples

**Simple Lookup (Safe for Extraction)**:

```perl
%canonWhiteBalance = (
    0 => 'Auto',
    1 => 'Daylight',
    2 => 'Cloudy',
    3 => 'Tungsten',
    4 => 'Fluorescent',
    5 => 'Flash',
    6 => 'Custom',
    15 => 'Shade',
    16 => 'Kelvin',
    17 => 'PC-1',
    18 => 'PC-2',
    19 => 'PC-3',
    20 => 'Daylight Fluorescent',
    21 => 'Custom 1',
    22 => 'Custom 2',
);
```

**Complex Structure (Never Extract)**:

```perl
my %Image::ExifTool::Canon::CameraInfo1DmkIII = (
    %binaryDataAttrs,
    FORMAT => 'int8u',
    FIRST_ENTRY => 0,
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    NOTES => 'CameraInfo tags for the EOS 1D Mark III.',
    0x03 => { %ciFNumber },
    0x04 => { %ciExposureTime },
    # ... complex binary format definitions
);
```

## Potential Challenges

### Technical Challenges

1. **Table Discovery**: Finding all simple tables across 217 modules
2. **Type Inference**: Determining correct Rust types for keys automatically
3. **Validation Complexity**: Ensuring extraction truly captures only primitive tables
4. **Performance at Scale**: Maintaining fast lookups with hundreds of tables

### Mitigation Strategies

1. **Conservative Validation**: Strict primitive-only validation with manual verification
2. **Incremental Rollout**: Start with proven high-value tables, expand systematically
3. **Type Annotations**: Explicit type configuration rather than inference
4. **Performance Monitoring**: Benchmark at each expansion phase

### Integration Risks

1. **API Compatibility**: Ensuring generated tables integrate cleanly with existing code
2. **Build Dependencies**: Config file maintenance across team and deployments
3. **Memory Usage**: Hundreds of LazyLock tables consuming significant memory

## Future Extensions

### Advanced Table Types

Once the primitive framework is proven:

- **Nested Simple Tables**: Tables with simple nested structures
- **Conditional Tables**: Tables with basic conditional logic
- **Cross-Reference Tables**: Tables that reference other simple tables

### Enhanced Features

- **Table Validation**: Cross-verification with ExifTool test output
- **Documentation Generation**: Auto-generated docs from table metadata
- **Usage Analytics**: Track which tables are actually used in practice
- **Custom Overrides**: User-defined table extensions and corrections

### Ecosystem Integration

- **Third-Party Extensions**: Community contribution of manufacturer-specific tables
- **Tool Integration**: ExifTool compatibility validation tools
- **Performance Optimization**: Compile-time optimizations for frequently-used tables

## Conclusion

This milestone establishes a scalable, systematic approach to harvesting ExifTool's wealth of simple lookup data. By focusing strictly on primitive key-value tables, we avoid the complexity of Perl parsing while capturing significant value.

The framework's configuration-driven approach enables rapid expansion across all camera manufacturers, transforming manual table transcription into an automated pipeline. This directly supports our "Trust ExifTool" principle by ensuring perfect fidelity with automatic updates.

**Key Innovation**: Rather than one-off extraction scripts, this creates a reusable framework that can systematically harvest hundreds of lookup tables while maintaining strict safety boundaries.

**Total Potential Impact**: From ~100 manually maintained lens entries to 3,000+ automatically generated lookup entries across all major camera manufacturers, with near-zero maintenance overhead.
