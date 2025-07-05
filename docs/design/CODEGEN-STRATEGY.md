# exif-oxide Code Generation Strategy

This document describes the code generation approach for exif-oxide, including what is generated automatically from ExifTool source analysis versus what is manually implemented.

## Overview

The code generation pipeline extracts metadata definitions from ExifTool's Perl modules and generates Rust code for:

- Tag table definitions
- Simple conversion functions
- Processor dispatch tables
- Reference lists for required implementations

Complex logic is intentionally NOT generated - it's manually implemented with ExifTool source references.

## DO NOT WRITE A PERL CODE PARSER

**The `perl` interpreter is the only competent perl parser!**

Use `use` or `require` in `perl`. DO NOT USE REGEX to try to extract something from a perl script!

There are too many gotchas and surprising perl-isms--any code that tries to extract maps, hashes, or other bits from perl in rust or regex is a bad idea, be brittle, lead us to ruin, and haunt us in the future. Use perl to read perl.

## Build Pipeline

1. **ExifTool Source** (Perl modules)
2. **Perl Extractor** (`codegen/extract_tables.pl`)
3. **JSON** (`codegen/generated/tag_tables.json`)
4. **Rust Codegen** (`codegen/src/main.rs`)
5. **Generated Code** (`src/generated/`)
6. **Implementation Palette** (`src/implementations/`)
7. **exif-oxide Library**

## What Codegen Handles

### 1. Tag Table Generation

```rust
// Generated from ExifTool tables
pub static EXIF_TAGS: TagTable = TagTable {
    name: "EXIF::Main",
    process_proc: ProcessProc::Exif,
    tags: &[
        Tag {
            id: 0x010e,
            name: "ImageDescription",
            format: Format::String,
            // Simple PrintConv only
            print_conv: Some(PrintConv::PassThrough),
        },
        // ... more tags
    ],
};
```

### 2. Simple Conversions

```rust
// Generated for simple hash lookups
fn print_conv_orientation(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "Horizontal (normal)".to_string(),
        Some(2) => "Mirror horizontal".to_string(),
        // ...
        _ => format!("Unknown ({})", val),
    }
}
```

### 3. Dispatch Tables

```rust
// Generated processor dispatch
fn select_processor(table: &TagTable) -> ProcessFunc {
    match table.process_proc {
        ProcessProc::Exif => process::exif::process_exif,
        ProcessProc::BinaryData => process::binary_data::process_binary,
        ProcessProc::Custom(name) => {
            registry::get_processor(name)
                .unwrap_or(process::exif::process_exif)
        }
    }
}
```

### 4. Conversion Reference Lists

```rust
// Generated from same source as tag definitions
// Ensures single source of truth for conversion requirements
pub static REQUIRED_PRINT_CONV: &[&str] = &[
    "orientation_print_conv",
    "flash_print_conv",
    "colorspace_print_conv",
    // ... automatically extracted from all tag tables
];

pub static REQUIRED_VALUE_CONV: &[&str] = &[
    "gps_coordinate_value_conv",
    "apex_shutter_value_conv",
    // ... extracted from ExifTool value conversion references
];
```

## What Codegen Does NOT Handle

1. **Complex Perl Logic**: Multi-line conditions, evals, complex math
2. **Dynamic Patterns**: Anything beyond `simple[$val{N}]`
3. **Encryption**: All crypto is manually implemented
4. **Hook Logic**: ProcessBinaryData Hooks are manual
5. **Error Recovery**: Manufacturer quirks are manual

## Mainstream Tag Filtering

To maintain a manageable scope, exif-oxide only implements tags that meet one of these criteria:

1. **Frequency > 80%**: Tags appearing in more than 80% of images
2. **Mainstream flag**: Tags marked as `mainstream: true` in TagMetadata.json
3. **Critical dependencies**: Tags required by other mainstream tags (DataMember)

This reduces scope from ~15,000 tags to ~500-1000, focusing on tags that matter for media management applications.

### Filtering During Codegen

```rust
// In codegen/src/filter.rs
fn should_generate_tag(tag: &Tag, metadata: &TagMetadata) -> bool {
    if let Some(meta) = metadata.get(&tag.name) {
        meta.mainstream || meta.frequency > 0.8
    } else {
        false // Unknown tags excluded by default
    }
}
```

## TODO Tracking System

Instead of generating thousands of stub functions that would panic with `todo!()`, we use a runtime fallback system:

### For PrintConv/ValueConv - Runtime References

```rust
// In generated code - NO STUBS GENERATED
Tag {
    id: 0x0112,
    name: "Orientation",
    print_conv: Some("exif_orientation_lookup"), // Just a reference
    value_conv: None,
}

// At runtime - graceful fallback
fn apply_print_conv(tag: &Tag, value: &TagValue) -> String {
    if let Some(conv_name) = &tag.print_conv {
        match registry.get_print_conv(conv_name) {
            Some(conv_fn) => conv_fn(value),
            None => {
                // Track missing implementation
                metrics::log_missing_impl(conv_name, &tag.name);
                // Return raw value formatted - never panic!
                format!("{:?}", value)
            }
        }
    } else {
        format!("{:?}", value)
    }
}
```

The auto-generated `REQUIRED_PRINT_CONV`/`REQUIRED_VALUE_CONV` arrays provide development visibility into the complete scope of conversion implementations needed, while maintaining DRY principles.

### For PROCESS_PROC - Required Implementations

Since there are only ~50 custom processors, we can enumerate them:

```rust
// Generated enum of all known processors
pub enum ProcessorType {
    Exif,
    BinaryData,
    Canon(CanonProcessor),
    Nikon(NikonProcessor),
    // ... ~50 variants total
}

// Runtime dispatch with clear errors
fn dispatch_processor(proc_type: ProcessorType, data: &[u8]) -> Result<()> {
    match proc_type {
        ProcessorType::Canon(CanonProcessor::SerialData) => {
            registry.get_processor("Canon::SerialData")
                .ok_or(ExifError::missing_processor("Canon::SerialData"))?
                (data)
        }
        // ...
    }
}
```

### Missing Implementation Tracking

```rust
// Generated metadata about what implementations are needed
pub static TAG_IMPL_REQUIREMENTS: &[(TagDef, ImplRequirement)] = &[
    (
        TagDef { table: "EXIF::Main", id: 0x0112, name: "Orientation" },
        ImplRequirement {
            print_conv: Some("orientation_lookup"),
            value_conv: None,
            priority: Priority::High,
            test_images: &["t/images/Canon.jpg"],
        }
    ),
    // ... all requirements
];

// Runtime tracking
lazy_static! {
    static ref MISSING_IMPLS: Mutex<HashMap<String, MissingImpl>> =
        Mutex::new(HashMap::new());
}
```

## Developer Tools

```bash
# Show what implementations are actually needed
cargo run -p exif-oxide -- --show-missing photo.jpg

Output:
Missing Implementations for photo.jpg
=====================================
HIGH PRIORITY (blocks common tags):
- orientation_lookup (PrintConv)
  Used by: EXIF:Orientation

MEDIUM PRIORITY:
- canon_custom_functions (PrintConv)
  Used by: Canon:CustomFunctions

# Generate stubs only for what's needed
cargo run -p exif-oxide -- --generate-stubs photo.jpg
# Creates: implementations/stubs/photo_jpg_stubs.rs with 2 functions
```

## Graceful Degradation

```rust
// In generated tag extraction
impl Tag {
    fn extract(&self, data: &[u8], ctx: &Context) -> Result<ExtractedTag> {
        let raw_value = self.parse_raw(data)?;

        // ValueConv with fallback
        let converted_value = self.value_conv
            .and_then(|ref_name| ctx.apply_value_conv(ref_name, &raw_value).ok())
            .unwrap_or(raw_value.clone());

        // PrintConv with fallback
        let display_value = self.print_conv
            .and_then(|ref_name| ctx.apply_print_conv(ref_name, &converted_value).ok())
            .unwrap_or_else(|| format!("{:?}", converted_value));

        Ok(ExtractedTag {
            name: self.name.clone(),
            raw: raw_value,
            converted: converted_value,
            display: display_value,
        })
    }
}
```

## Development Workflow

### 1. Extract Phase

```bash
perl codegen/extract_tables.pl > codegen/tag_tables.json
```

### 2. Generate Phase

```bash
cargo run -p codegen
```

Output:

```
Generated: 823 mainstream tag definitions (from 15,234 total)
Simple conversions implemented: 156
Complex conversions referenced: 234 (no stubs generated)
Custom processors identified: 47

Code is ready to compile and run!
Use --show-missing on actual images to see what's needed.
```

### 3. Discover Missing Implementations

```bash
# Run on actual images to find what's needed
cargo run -p exif-oxide -- t/images/Canon/EOS-5D.jpg --show-missing

Missing implementations for this file:
- orientation_lookup (EXIF:Orientation)
- canon_ev_format (Canon:ExposureCompensation)
- canon_wb_lookup (Canon:WhiteBalance)

# Generate just these stubs
cargo run -p exif-oxide -- t/images/Canon/EOS-5D.jpg --generate-stubs
# Creates: implementations/stubs/eos_5d_stubs.rs
```

### 4. Implement What's Needed

- Developer implements the specific functions
- References ExifTool source
- Registers in implementation palette
- No need to implement unused conversions!

### 5. Validate Phase

```bash
cargo test
# Runs against ExifTool test images
# Compares output with exiftool -j
# Shows coverage metrics
```

## Update Workflow for ExifTool Releases

When a new ExifTool version is released:

1. **Update ExifTool Submodule**

   ```bash
   cd third-party/exiftool
   git fetch origin
   git checkout v12.77  # new version
   cd ../..
   ```

2. **Regenerate and Build**

   ```bash
   # Extract updated tag definitions
   perl codegen/extract_tables.pl > codegen/tag_tables.json

   # Run codegen - will show new missing implementations
   cargo run -p codegen
   ```

3. **Review Changes**

   ```
   New in ExifTool 12.77:
   - 3 new mainstream tags requiring implementation
   - 1 new Canon processor variant
   - 47 non-mainstream tags (ignored)

   Missing implementations (priority order):
   1. canon_new_lens_type (PrintConv) - 15 test images
   2. nikon_z9_af_mode (PrintConv) - 8 test images
   3. ProcessCanonCR3 (Processor) - 5 test images
   ```

4. **Implement Missing Pieces**

   - Add implementations to palette
   - Reference ExifTool source
   - Test against provided images

5. **Ship Updated Version**
   ```bash
   cargo test
   # All passing - ready to release!
   ```

For minor ExifTool updates that only add tags within existing processors, the process is often just regenerate and ship. New processors or complex conversions require manual implementation.

## Simple Table Extraction Framework

### What Are Simple Tables?

ExifTool contains hundreds of primitive lookup tables across manufacturer modules that provide valuable metadata conversion capabilities. These are safe for automated extraction:

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

**Never Extract** ❌:

```perl
0xd => [
    {
        Name => 'CanonCameraInfo1D',
        Condition => '$$self{Model} =~ /\b1DS?$/',
        SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraInfo1D' },
    },
];
```

### How to Add New Simple Tables

1. **Identify Candidate Tables**: Look for simple `%hash = (key => 'value')` patterns in ExifTool modules
2. **Validate Primitive-ness**: Ensure no Perl variables, expressions, or complex structures
3. **Add to Configuration**: Update `codegen/simple_tables.json`
4. **Run Extraction**: `make codegen-simple-tables`
5. **Integrate**: Use generated lookup functions in PrintConv implementations

### Adding a New Table (Example)

Found a new simple table in `Canon.pm`:

```perl
%canonFlashMode = (
    0 => 'Off',
    1 => 'Auto',
    2 => 'On',
    3 => 'Red-eye Reduction',
    4 => 'Slow Sync',
    5 => 'Auto + Red-eye Reduction',
);
```

**Step 1**: Add entry to `codegen/simple_tables.json`:

```json
{
  "module": "Canon.pm",
  "hash_name": "%canonFlashMode",
  "output_file": "canon/flash_mode.rs",
  "constant_name": "CANON_FLASH_MODE",
  "key_type": "u8",
  "description": "Canon flash mode setting names"
}
```

**Step 2**: Run extraction:

```bash
make codegen-simple-tables
```

**Step 3**: Use in PrintConv:

```rust
use crate::generated::canon::flash_mode::lookup_canon_flash_mode;

pub fn canon_flash_mode_print_conv(value: &TagValue) -> Result<String> {
    if let Some(mode_value) = value.as_u8() {
        if let Some(description) = lookup_canon_flash_mode(mode_value) {
            return Ok(description.to_string());
        }
    }
    Ok(format!("Unknown ({})", value))
}
```

### Guidelines for Table Selection

**Include**:

- Simple hash tables with primitive keys (numbers, strings)
- Values are plain strings (no variables or expressions)
- High-value data (lens databases, mode settings, model IDs)
- Tables with >10 entries (worth the automation)

**Exclude**:

- Any Perl expressions in keys or values
- Nested structures or references
- Conditional logic or complex formatting
- Tables with <5 entries (manual implementation easier)

### Validation Process

The extraction framework automatically validates tables:

- ✅ Keys must be simple primitives (numbers, quoted strings)
- ✅ Values must be simple quoted strings
- ✅ No variables, expressions, or function calls
- ❌ Skips tables that don't meet criteria

### Benefits

- **Systematic Coverage**: Extract hundreds of tables consistently
- **Automatic Updates**: Regenerate with ExifTool releases
- **Type Safety**: Generated Rust code with proper types
- **Performance**: Fast HashMap lookups with LazyLock initialization
- **Traceability**: Every entry references ExifTool source line

See [MILESTONE-CODEGEN-SIMPLE-TABLES.md](../milestones/MILESTONE-CODEGEN-SIMPLE-TABLES.md) for complete implementation details.

## Related Documentation

- [API-DESIGN.md](API-DESIGN.md) - How the generated API is structured
- [IMPLEMENTATION-PALETTE.md](IMPLEMENTATION-PALETTE.md) - How manual implementations are registered
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Practical implementation guide
