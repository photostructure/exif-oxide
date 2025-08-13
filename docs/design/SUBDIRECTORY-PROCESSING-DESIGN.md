# SubDirectory Processing Design - Trust ExifTool

## Overview

This document defines the correct subdirectory processing architecture for exif-oxide, following ExifTool's implementation exactly as documented in `third-party/exiftool/doc/concepts/SUBDIRECTORY_SYSTEM.md`.

## Core Principle: Trust ExifTool

We must implement subdirectory processing **exactly** as ExifTool does it. No improvements, no optimizations, no simplifications. Every quirk exists for a reason discovered over 25 years of real-world camera data.

## ExifTool's SubDirectory Architecture

### 1. Tag Definition Structure

ExifTool supports multiple tag variants with the same ID, selected via Conditions:

```perl
# Canon.pm:1553+
0xf => [
    {
        Name => 'CustomFunctions1D',
        Condition => '$$self{Model} =~ /EOS-1D/',
        SubDirectory => {
            TagTable => 'Image::ExifTool::CanonCustom::Functions1D',
            Validate => 'Image::ExifTool::Canon::Validate($dirData,$subdirStart,$size)',
        },
    },
    {
        Name => 'CustomFunctions5D',
        Condition => '$$self{Model} =~ /EOS 5D/',
        SubDirectory => {
            TagTable => 'Image::ExifTool::CanonCustom::Functions5D',
            # ...
        },
    },
    # ... more variants
]
```

### 2. Processing Flow (ExifTool.pm:10701+, Exif.pm:6807+)

```
1. GetTagInfo evaluates all Conditions for tag variants
2. First variant with passing Condition (or no Condition) is selected
3. SubDirectory parameters are extracted
4. ProcessDirectory is called recursively with new TagTable
5. Tags are extracted into parent context
6. PrintConv happens later during value display (NOT during extraction)
```

### 3. Condition Evaluation Context

Conditions have access to:
- `$$self{Model}` - Camera model string
- `$$self{Make}` - Manufacturer string  
- `$$valPt` - Pointer to tag value data
- `$format` - Data format
- `$count` - Element count
- Any other ExifTool object state

## Required Rust Implementation

### 1. Tag Variant Structure

```rust
// In generated code for each manufacturer
pub struct TagVariant {
    pub name: String,
    pub condition: Option<String>,  // ExifTool Condition expression
    pub subdirectory: Option<SubDirectoryDef>,
    // ... other fields
}

pub struct SubDirectoryDef {
    pub tag_table: String,           // Target tag table module
    pub validate: Option<String>,    // Validation expression
    pub process_proc: Option<String>, // Custom processor
    pub byte_order: Option<String>,  // Endianness override
    pub start: Option<String>,       // Offset calculation
    pub base: Option<String>,        // Base offset adjustment
}

// Tag definitions become Vec<TagVariant> instead of single entries
pub type TagDefinitions = HashMap<u16, Vec<TagVariant>>;
```

### 2. Condition Evaluation

```rust
impl ExpressionEvaluator {
    /// Evaluate subdirectory condition with full context
    pub fn evaluate_subdirectory_condition(
        &mut self,
        condition: &str,
        model: Option<&str>,
        make: Option<&str>,
        value_data: Option<&[u8]>,
        format: Option<&str>,
    ) -> Result<bool> {
        // Parse condition expression
        let expr = parse_expression(condition)?;
        
        // Build evaluation context matching ExifTool's $self
        let context = SubDirectoryContext {
            model,
            make,
            value_data,
            format,
            // ... other fields
        };
        
        // Evaluate using our expression system
        self.evaluate_with_context(&expr, &context)
    }
}
```

### 3. Manufacturer-Specific Modules

Each manufacturer module must implement:

```rust
// src/implementations/canon/subdirectory.rs
pub struct CanonSubDirectoryProcessor;

impl CanonSubDirectoryProcessor {
    /// Get all subdirectory definitions for a tag
    pub fn get_subdirectory_definitions(tag_id: u16) -> Option<Vec<TagVariant>> {
        // Return from generated Canon tags
        CANON_SUBDIRECTORY_TAGS.get(&tag_id).cloned()
    }
    
    /// Select the correct variant based on conditions
    pub fn select_variant(
        variants: Vec<TagVariant>,
        evaluator: &mut ExpressionEvaluator,
        model: Option<&str>,
        make: Option<&str>,
        value_data: &[u8],
    ) -> Option<TagVariant> {
        for variant in variants {
            if let Some(condition) = &variant.condition {
                match evaluator.evaluate_subdirectory_condition(
                    condition,
                    model,
                    make,
                    Some(value_data),
                    None,
                ) {
                    Ok(true) => return Some(variant),
                    Ok(false) => continue,
                    Err(e) => {
                        debug!("Condition evaluation failed: {}", e);
                        continue;
                    }
                }
            } else {
                // No condition means always match
                return Some(variant);
            }
        }
        None
    }
    
    /// Process the subdirectory with selected variant
    pub fn process_subdirectory(
        variant: &TagVariant,
        data: &[u8],
        byte_order: ByteOrder,
        base_offset: u32,
    ) -> Result<HashMap<String, TagValue>> {
        let subdir = variant.subdirectory.as_ref()
            .ok_or_else(|| ExifError::ParseError("No subdirectory def".into()))?;
        
        // Load the target tag table
        let tag_table = match &subdir.tag_table[..] {
            "Image::ExifTool::CanonCustom::Functions1D" => {
                load_canon_custom_functions_1d_table()
            }
            "Image::ExifTool::CanonCustom::Functions5D" => {
                load_canon_custom_functions_5d_table()
            }
            // ... more tables
            _ => return Err(ExifError::ParseError(
                format!("Unknown tag table: {}", subdir.tag_table)
            )),
        };
        
        // Process based on table type
        if tag_table.is_binary_data {
            process_binary_data(&tag_table, data, byte_order)
        } else {
            process_ifd_directory(&tag_table, data, byte_order, base_offset)
        }
    }
}
```

### 4. Integration with ExifReader

```rust
// In ExifReader or processor
pub fn process_maker_notes(&mut self) -> Result<()> {
    // Get camera model from extracted tags
    let model = self.get_tag_value("Model")
        .and_then(|v| v.as_string());
    
    let make = self.get_tag_value("Make")
        .and_then(|v| v.as_string());
    
    // Process each manufacturer's tags
    for (tag_id, tag_value) in &self.extracted_tags {
        // Check if tag has subdirectory processing
        if let Some(variants) = CanonSubDirectoryProcessor::get_subdirectory_definitions(*tag_id) {
            // Select correct variant based on conditions
            let mut evaluator = ExpressionEvaluator::new();
            if let Some(variant) = CanonSubDirectoryProcessor::select_variant(
                variants,
                &mut evaluator,
                model,
                make,
                tag_value.as_bytes()?,
            ) {
                // Process the subdirectory
                let extracted = CanonSubDirectoryProcessor::process_subdirectory(
                    &variant,
                    tag_value.as_bytes()?,
                    self.byte_order,
                    self.base_offset,
                )?;
                
                // Store extracted tags (NO PrintConv here!)
                for (name, value) in extracted {
                    self.store_tag_with_synthetic_id(&name, value);
                }
                
                // Remove original binary array
                self.remove_tag(*tag_id);
            }
        }
    }
    
    Ok(())
}
```

## Critical Implementation Rules

### 1. NO PrintConv During SubDirectory Processing

PrintConv is applied **after** tag extraction, during value display. SubDirectory processing only extracts raw values.

### 2. Exact Condition Evaluation

Conditions must be evaluated exactly as ExifTool does:
- Model patterns must match ExifTool's regex behavior
- All context variables must be available
- Evaluation order must be preserved

### 3. Tag Table Switching

When a SubDirectory specifies a different TagTable, we must:
1. Load the specified table (from generated code)
2. Use that table's tag definitions
3. Respect the table's FORMAT and processing rules

### 4. Validation Functions

Manufacturer-specific validation (like `Canon::Validate`) must be implemented to match ExifTool's validation logic.

## Migration Path

### Phase 1: Refactor Current Implementation
1. Remove PrintConv from subdirectory_processing.rs
2. Split BinaryData from SubDirectory processing
3. Add Condition evaluation support

### Phase 2: Implement Manufacturer Modules
1. Canon subdirectory processor
2. Nikon subdirectory processor  
3. Sony subdirectory processor
4. Olympus subdirectory processor

### Phase 3: Codegen Enhancement
1. Generate tag variant structures
2. Extract Condition fields
3. Generate table loading functions

### Phase 4: Validation
1. Compare output with ExifTool for each manufacturer
2. Test condition evaluation with various camera models
3. Verify all subdirectory types work correctly

## Testing Strategy

### Test Cases Required

1. **Canon EOS-1D** - Should use CustomFunctions1D table
2. **Canon EOS 5D** - Should use CustomFunctions5D table
3. **Canon EOS 30D** - Should use CustomFunctions30D table
4. **Mixed manufacturers** - Ensure no cross-contamination

### Validation Method

```bash
# For each test image
exiftool -j -struct -G test.jpg > expected.json
cargo run -- test.jpg > actual.json
# Compare CustomFunctions values
```

## References

- `third-party/exiftool/doc/concepts/SUBDIRECTORY_SYSTEM.md` - Complete subdirectory documentation
- `third-party/exiftool/lib/Image/ExifTool.pm:10701+` - GetTagInfo with Condition evaluation
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:6807+` - ProcessExif subdirectory handling
- `third-party/exiftool/lib/Image/ExifTool/Canon.pm:1553+` - Canon tag 0xf variants
- `docs/TRUST-EXIFTOOL.md` - Prime directive