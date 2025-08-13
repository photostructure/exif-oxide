# SubDirectory Mechanics: Deep Analysis

## 1. TagTable Registry Requirement

### The Problem

ExifTool references tag tables as strings:
```perl
TagTable => 'Image::ExifTool::Canon::CameraSettings'
TagTable => 'Image::ExifTool::CanonCustom::Functions1D'
TagTable => 'Image::ExifTool::Nikon::ColorBalanceA'
```

We need to map these to our generated Rust modules:
```
'Image::ExifTool::Canon::CameraSettings' -> src/generated/Canon_pm/camera_settings_tags.rs
'Image::ExifTool::CanonCustom::Functions1D' -> src/generated/CanonCustom_pm/functions1d_tags.rs
```

### Existing Infrastructure

**Good news**: We already have the generated modules! 
- `/src/generated/Canon_pm/` has 100+ modules
- `/src/generated/CanonCustom_pm/` has Functions1D, Functions5D, etc.
- `/src/generated/Nikon_pm/`, `/src/generated/Sony_pm/`, etc.

### Solution: TagTable Registry

Similar to our `conv_registry`, we need a **compile-time tag table registry**:

```rust
// codegen/src/table_registry/mod.rs
pub struct TagTableRegistry {
    // Maps ExifTool module names to Rust module paths
    mappings: HashMap<String, TagTableMapping>,
}

pub struct TagTableMapping {
    pub exiftool_name: String,        // "Image::ExifTool::Canon::CameraSettings"
    pub rust_module: String,          // "crate::generated::Canon_pm::camera_settings_tags"
    pub table_const: String,          // "CAMERA_SETTINGS_TAGS"
    pub is_binary_data: bool,
    pub format: Option<String>,       // "int16s" for binary data tables
    pub first_entry: Option<i32>,     // 1 for 1-based indexing
}

impl TagTableRegistry {
    pub fn get_table_loader(&self, exiftool_name: &str) -> Option<String> {
        // Returns Rust code to load the table:
        // "crate::generated::Canon_pm::camera_settings_tags::CAMERA_SETTINGS_TAGS"
    }
}
```

**During Codegen**: Build the registry by scanning generated modules
**During Runtime**: Use registry to load correct table

## 2. DRY Pattern Across Manufacturers

### Common Pattern Analysis

After analyzing Canon, Nikon, Sony, Olympus, etc., the subdirectory pattern is:

1. **Tag has SubDirectory definition** (with optional Condition)
2. **Select variant** based on Conditions (model, data pattern, etc.)
3. **Load TagTable** specified in SubDirectory
4. **Process based on table type**:
   - Binary data tables → ProcessBinaryData
   - IFD tables → ProcessExif
   - Custom processors → ProcessProc function

### The DRY Solution: Generic SubDirectory Processor

Instead of per-manufacturer implementations, we can have ONE generic processor with manufacturer-specific **data**:

```rust
// src/exif/subdirectory_processor.rs

pub struct SubDirectoryProcessor {
    table_registry: Arc<TagTableRegistry>,
    expression_evaluator: ExpressionEvaluator,
}

impl SubDirectoryProcessor {
    /// Generic subdirectory processing for ANY manufacturer
    pub fn process_subdirectory(
        &mut self,
        tag_id: u16,
        tag_value: &TagValue,
        subdirectory_defs: &[SubDirectoryDef],  // From generated code
        context: &ProcessingContext,
    ) -> Result<HashMap<String, TagValue>> {
        // 1. Select variant based on conditions
        let variant = self.select_variant(subdirectory_defs, context)?;
        
        // 2. Load the tag table using registry
        let table = self.load_tag_table(&variant.tag_table)?;
        
        // 3. Process based on table type
        match table.table_type {
            TableType::BinaryData => {
                self.process_binary_data(&table, tag_value, context)
            }
            TableType::IFD => {
                self.process_ifd(&table, tag_value, context)
            }
            TableType::Custom(processor_name) => {
                self.process_custom(processor_name, tag_value, context)
            }
        }
    }
    
    fn select_variant(
        &mut self,
        defs: &[SubDirectoryDef],
        context: &ProcessingContext,
    ) -> Result<&SubDirectoryDef> {
        for def in defs {
            if let Some(condition) = &def.condition {
                if self.expression_evaluator.evaluate_subdirectory_condition(
                    condition,
                    context.model.as_deref(),
                    context.make.as_deref(),
                    context.value_data,
                    None,
                )? {
                    return Ok(def);
                }
            } else {
                // No condition = always match
                return Ok(def);
            }
        }
        Err(ExifError::ParseError("No matching subdirectory variant".into()))
    }
    
    fn load_tag_table(&self, table_name: &str) -> Result<LoadedTagTable> {
        // Use registry to get table loader
        let loader = self.table_registry.get_table_loader(table_name)
            .ok_or_else(|| ExifError::ParseError(format!("Unknown table: {}", table_name)))?;
        
        // This would be generated code that knows how to load each table
        load_table_by_name(&loader)
    }
}
```

### Manufacturer-Specific Data (Generated)

Each manufacturer only needs **data definitions**, not code:

```rust
// src/generated/Canon_pm/subdirectory_defs.rs
pub static CANON_SUBDIRECTORY_DEFS: LazyLock<HashMap<u16, Vec<SubDirectoryDef>>> = 
    LazyLock::new(|| HashMap::from([
        (0x0001, vec![SubDirectoryDef {
            name: "CanonCameraSettings",
            condition: None,
            tag_table: "Image::ExifTool::Canon::CameraSettings",
            validate: Some("Image::ExifTool::Canon::Validate"),
            byte_order: None,
        }]),
        (0x000f, vec![
            SubDirectoryDef {
                name: "CustomFunctions1D",
                condition: Some("$$self{Model} =~ /EOS-1D/"),
                tag_table: "Image::ExifTool::CanonCustom::Functions1D",
                validate: Some("Image::ExifTool::Canon::Validate"),
                byte_order: None,
            },
            SubDirectoryDef {
                name: "CustomFunctions5D",
                condition: Some("$$self{Model} =~ /EOS 5D/"),
                tag_table: "Image::ExifTool::CanonCustom::Functions5D",
                validate: Some("Image::ExifTool::Canon::Validate"),
                byte_order: None,
            },
            // ... 50+ more variants
        ]),
    ]));
```

## 3. Implementation Mechanics

### Step 1: Enhanced Codegen

Extract subdirectory definitions into data structures:

```perl
# codegen/scripts/extract_subdirectory_defs.pl
my %subdirectory_defs;

for my $tagID (keys %tagTable) {
    my $tagInfo = $tagTable{$tagID};
    
    # Handle single definition
    if (ref $tagInfo eq 'HASH' && $tagInfo->{SubDirectory}) {
        push @{$subdirectory_defs{$tagID}}, {
            name => $tagInfo->{Name},
            condition => $tagInfo->{Condition},
            subdirectory => $tagInfo->{SubDirectory},
        };
    }
    # Handle multiple variants
    elsif (ref $tagInfo eq 'ARRAY') {
        for my $variant (@$tagInfo) {
            if ($variant->{SubDirectory}) {
                push @{$subdirectory_defs{$tagID}}, {
                    name => $variant->{Name},
                    condition => $variant->{Condition},
                    subdirectory => $variant->{SubDirectory},
                };
            }
        }
    }
}
```

### Step 2: Table Registry Generation

Scan all generated modules to build registry:

```rust
// codegen/src/table_registry/builder.rs
pub fn build_table_registry() -> TagTableRegistry {
    let mut registry = TagTableRegistry::new();
    
    // Scan src/generated/*/mod.rs files
    for module_dir in glob("src/generated/*_pm/").unwrap() {
        let module_name = extract_module_name(&module_dir);
        
        // Map each table in the module
        for table_file in glob(&format!("{}/*_tags.rs", module_dir)).unwrap() {
            let table_info = extract_table_info(&table_file);
            registry.add_mapping(
                table_info.exiftool_name,
                table_info.rust_path,
                table_info.metadata,
            );
        }
    }
    
    registry
}
```

### Step 3: Runtime Table Loading

Generate a table loader that can load any table by name:

```rust
// src/generated/table_loader.rs (generated)
pub fn load_table_by_name(table_path: &str) -> Result<LoadedTagTable> {
    match table_path {
        "crate::generated::Canon_pm::camera_settings_tags::CAMERA_SETTINGS_TAGS" => {
            Ok(LoadedTagTable {
                tags: &Canon_pm::camera_settings_tags::CAMERA_SETTINGS_TAGS,
                is_binary_data: true,
                format: Some("int16s"),
                first_entry: Some(1),
            })
        }
        "crate::generated::CanonCustom_pm::functions1d_tags::FUNCTIONS1D_TAGS" => {
            Ok(LoadedTagTable {
                tags: &CanonCustom_pm::functions1d_tags::FUNCTIONS1D_TAGS,
                is_binary_data: true,
                format: Some("int16s"),
                first_entry: Some(1),
            })
        }
        // ... hundreds more cases (generated)
        _ => Err(ExifError::ParseError(format!("Unknown table: {}", table_path)))
    }
}
```

## 4. Special Cases & Complexity

### ProcessProc Functions

Some subdirectories use custom processors:

```perl
SubDirectory => {
    TagTable => 'Image::ExifTool::Nikon::ShotInfoD500',
    ProcessProc => \&Image::ExifTool::Nikon::ProcessNikonEncrypted,
}
```

Solution: Registry of custom processors:

```rust
pub enum ProcessProc {
    Standard,  // Use standard ProcessBinaryData or ProcessExif
    NikonEncrypted,
    CanonSerialData,
    SonyDeciphered,
    // ... other special processors
}

fn get_process_proc(name: &str) -> ProcessProc {
    match name {
        "Image::ExifTool::Nikon::ProcessNikonEncrypted" => ProcessProc::NikonEncrypted,
        "Image::ExifTool::Canon::ProcessSerialData" => ProcessProc::CanonSerialData,
        _ => ProcessProc::Standard,
    }
}
```

### Validation Functions

```perl
Validate => 'Image::ExifTool::Canon::Validate($dirData,$subdirStart,$size)'
```

Solution: Manufacturer-specific validators:

```rust
pub trait SubDirectoryValidator {
    fn validate(&self, data: &[u8], start: usize, size: usize) -> bool;
}

pub struct CanonValidator;
impl SubDirectoryValidator for CanonValidator {
    fn validate(&self, data: &[u8], start: usize, size: usize) -> bool {
        // Port Canon::Validate logic
        // Check magic numbers, size constraints, etc.
    }
}

fn get_validator(validate_expr: &str) -> Option<Box<dyn SubDirectoryValidator>> {
    if validate_expr.contains("Canon::Validate") {
        Some(Box::new(CanonValidator))
    } else if validate_expr.contains("Nikon::Validate") {
        Some(Box::new(NikonValidator))
    } else {
        None
    }
}
```

## 5. The Big Win: Total DRY

With this approach:

1. **ONE subdirectory processor** handles ALL manufacturers
2. **Generated data files** contain the manufacturer-specific definitions
3. **Table registry** maps names to modules (like conv_registry)
4. **Custom processors** registered by name
5. **Validators** implemented once per manufacturer

Instead of 10-15 manufacturer implementations, we have:
- 1 generic processor
- 1 table registry (generated)
- 10-15 data definition files (generated)
- A few special processor implementations

## 6. Integration with Existing Code

### Current State
- We have `subdirectory_processing.rs` but it's incorrectly architected
- We have generated tag tables in `src/generated/*_pm/`
- We have `ExpressionEvaluator` for conditions

### Migration Path
1. Keep existing generated tables
2. Add subdirectory definitions extraction
3. Generate table registry
4. Refactor `subdirectory_processing.rs` to use generic approach
5. Remove manufacturer-specific subdirectory code

## 7. Example: Canon Tag 0xf Processing

With the new system:

```rust
// At runtime, when we encounter Canon tag 0xf:
let subdirectory_defs = CANON_SUBDIRECTORY_DEFS.get(&0x000f).unwrap();
// Returns 50+ variants with different Conditions

let context = ProcessingContext {
    model: Some("Canon EOS 5D Mark III"),
    make: Some("Canon"),
    // ...
};

let extracted_tags = processor.process_subdirectory(
    0x000f,
    &tag_value,
    subdirectory_defs,
    &context,
)?;
// Automatically:
// 1. Evaluates conditions, finds "$$self{Model} =~ /EOS 5D/" matches
// 2. Loads Image::ExifTool::CanonCustom::Functions5D via registry
// 3. Processes as binary data with int16s format
// 4. Returns extracted tags

```

## Summary

The key insight: **Subdirectory processing is data-driven, not code-driven**

We don't need different CODE for each manufacturer. We need:
1. A registry to find tables (like conv_registry)
2. Generic processing logic that works for all
3. Manufacturer-specific DATA (conditions, table names, validators)