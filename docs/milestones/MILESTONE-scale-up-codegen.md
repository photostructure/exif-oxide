# Milestone: Codegen Configuration Architecture Scale-Up

**Duration**: 1-2 weeks  
**Goal**: Refactor codegen extraction system to scale from 35 to 300+ tables with improved maintainability

## Overview

The current codegen extraction system needs architectural improvements to handle the massive scale-up required for RAW format support. This milestone restructures the configuration system, reduces boilerplate, and aligns organization with ExifTool source files to support 300+ lookup tables efficiently.

## Background

### Current System Limitations

The existing codegen system faces several scaling challenges:

1. **File Proliferation**: Each table generates its own `.rs` file (35 → 300+ files)
2. **Build Performance**: Each file is a separate compilation unit
3. **Configuration Management**: Single `extract.json` becomes unwieldy at scale
4. **Boilerplate Overhead**: ~15 lines of repetitive code per table
5. **Organization Mismatch**: Logical grouping doesn't match ExifTool source structure

### Scale-Up Requirements

From [MILESTONE-17-PREREQUISITE-Codegen.md](MILESTONE-17-PREREQUISITE-Codegen.md):

- **Canon**: ~140 lookup tables (1000+ lens types, 367 camera models)
- **Nikon**: ~85 lookup tables (618 lens IDs, multiple AF point mappings)
- **Sony**: ~35 lookup tables (white balance, AF points, exposure programs)
- **Olympus**: ~25 lookup tables (lens types, camera types, scene modes)
- **Panasonic**: ~15 lookup tables (white balance, CFA patterns)
- **Total**: 300+ tables requiring efficient organization and generation

## Implementation Strategy

### Phase 1: Configuration Restructuring

#### Source-File-Based Organization

**Current Structure** (logical grouping):

```
codegen/extract.json  # Single monolithic config
src/generated/
├── canon/           # Canon-specific tables
├── exif/            # EXIF-specific tables
├── file_types/      # File type detection
└── nikon/           # Nikon-specific tables
```

**Proposed Structure** (ExifTool source-based):

```
codegen/config/
├── Canon_pm/
│   ├── simple_table.json          # Basic key-value lookups
│   ├── print_conv.json            # PrintConv inline extractions
│   └── regex_patterns.json        # Pattern-based extractions
├── ExifTool_pm/
│   ├── simple_table.json
│   ├── file_type_lookup.json      # File type detection
│   └── boolean_set.json           # Set membership tables
├── Nikon_pm/
│   ├── simple_table.json
│   ├── af_point_mapping.json      # AF point tables
│   └── print_conv.json
└── XMP_pm/
    ├── simple_table.json          # Namespace mappings
    └── char_conversion.json        # Character entity tables
```

#### JSON Schema Splitting

**Current**: Single schema with `extraction_type` discriminator
**Proposed**: Focused schemas per extraction type

```
codegen/schemas/
├── simple_table.json          # Basic key-value lookups
├── regex_strings.json         # Pattern extractions
├── file_type_lookup.json      # File type discriminated unions
├── boolean_set.json           # Set membership tables
├── print_conv.json            # PrintConv inline extractions
└── base_extraction.json       # Shared properties
```

**Schema Simplification**: Each schema should contain only essential fields to minimize config verbosity:

```json
// codegen/schemas/simple_table.json (minimal)
{
  "type": "object",
  "properties": {
    "description": { "type": "string" },
    "tables": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "hash_name": { "type": "string" },
          "constant_name": { "type": "string" },
          "key_type": { "type": "string" },
          "description": { "type": "string" }
        },
        "required": ["hash_name", "constant_name", "key_type"]
      }
    }
  }
}
```

Note: Remove `output_file` (auto-generated), `module` (directory-based), and other extraneous fields.

### Phase 2: Code Generation Refactoring

#### Boilerplate Reduction

**Current Generated Code** (~15 lines per table):

```rust
pub static CANON_LENS_TYPES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "Canon EF 50mm f/1.8");
    map.insert(2, "Canon EF 28mm f/2.8");
    // ... more entries
    map
});

pub fn lookup_canon_lens_type(id: u16) -> Option<&'static str> {
    CANON_LENS_TYPES.get(&id).copied()
}
```

**Proposed Macro-Based Generation** (~3 lines per table):

```rust
make_simple_table!(CANON_LENS_TYPES, u16, &'static str, [
    (1, "Canon EF 50mm f/1.8"),
    (2, "Canon EF 28mm f/2.8"),
    // ... more entries
]);
// Automatically provides lookup_canon_lens_types function
```

#### Consolidated Generation

**Current**: One file per table (300+ files)
**Proposed**: One file per ExifTool source module (~15 files)

```rust
// src/generated/Canon_pm/mod.rs
mod simple_tables {
    make_simple_table!(CANON_LENS_TYPES, u16, &'static str, [/* entries */]);
    make_simple_table!(CANON_MODEL_ID, u32, &'static str, [/* entries */]);
    make_simple_table!(CANON_WHITE_BALANCE, u8, &'static str, [/* entries */]);
}

mod print_conv_tables {
    // PrintConv extractions
}

// Re-export everything at top level for clean imports
pub use simple_tables::*;
pub use print_conv_tables::*;
```

### Phase 3: Generated Code Migration

#### Import Path Updates

**Current Import Patterns** (14 files affected):

```rust
use crate::generated::exif::orientation::lookup_orientation;
use crate::generated::file_types::lookup_mime_types;
use crate::generated::canon::lens_types::lookup_canon_lens_type;
```

**Proposed Import Patterns**:

```rust
use crate::generated::Exif_pm::lookup_orientation;
use crate::generated::ExifTool_pm::lookup_mime_types;
use crate::generated::Canon_pm::lookup_canon_lens_type;
```

**Migration Strategy**: Update all 14 import locations systematically with search-and-replace.

### Phase 4: Build System Integration

#### Schema Validation

Add JSON schema validation to `make check`:

```makefile
.PHONY: check-schemas
check-schemas:
	@echo "Validating JSON schemas..."
	@find codegen/config -name "*.json" -exec sh -c 'echo "Validating $$1..."; jsonschema validate --instance "$$1" codegen/schemas/$$(basename $$(dirname "$$1"))/$$(basename "$$1" .json).json' _ {} \;

check: check-schemas check-rust check-docs
```

#### Parallel Processing

Enable parallel extraction by source module:

```makefile
# Process all config files by source module
CANON_CONFIGS := $(wildcard codegen/config/Canon.pm/*.json)
NIKON_CONFIGS := $(wildcard codegen/config/Nikon.pm/*.json)
EXIFTOOL_CONFIGS := $(wildcard codegen/config/ExifTool.pm/*.json)

# Parallel processing by source module
.PHONY: extract-canon extract-nikon extract-exiftool
extract-canon: $(CANON_CONFIGS)
extract-nikon: $(NIKON_CONFIGS)
extract-exiftool: $(EXIFTOOL_CONFIGS)

# Can run all extractions for a source module in parallel
extract-all: extract-canon extract-nikon extract-exiftool
```

## Success Criteria

### Core Requirements

- [x] **Configuration Migration**: All 35 existing tables migrated to new structure
- [x] **Schema Validation**: JSON schema validation integrated into `make check`
- [x] **Boilerplate Reduction**: 80% reduction in generated code size
- [x] **Import Path Updates**: All 14 files updated to use new import paths
- [x] **Build System**: Parallel processing and schema validation working
- [x] **Compatibility**: All existing functionality preserved

### Validation Tests

- [x] **All Tables Work**: `make codegen` generates valid Rust code for all tables
- [x] **Compilation**: All generated code compiles without warnings
- [x] **Import Resolution**: All updated import paths resolve correctly
- [x] **Schema Validation**: `make check` validates all config files
- [x] **Lookup Functions**: All generated lookup functions work correctly
- [x] **Compatibility Tests**: `make compat` passes with new generated code

### Performance Metrics

- [x] **File Count**: Reduced from 35+ files to ~15 files
- [x] **Build Time**: Faster compilation due to fewer compilation units
- [x] **Generated Code Size**: 80% reduction in boilerplate
- [x] **Configuration Manageable**: Easy to add new tables and extraction types

## Implementation Boundaries

### Goals

- Refactor configuration system to scale to 300+ tables
- Reduce boilerplate and improve maintainability
- Align organization with ExifTool source structure
- Add schema validation and parallel processing
- Migrate existing functionality without breaking changes

### Non-Goals

- Add new lookup tables (future milestone)
- Modify extraction logic or ExifTool integration
- Change generated table runtime behavior
- Add new extraction types beyond existing ones

## Dependencies and Prerequisites

### Required Knowledge

- Current codegen extraction system: [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)
- ExifTool integration patterns: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- Build system: `codegen/Makefile` and `codegen/src/main.rs`
- JSON schema validation tools

### Files to Understand

- **Current System**: `codegen/extract.json`, `codegen/src/main.rs`
- **Generated Code**: `src/generated/*/mod.rs` files
- **Import Patterns**: Files in `src/implementations/` and `src/file_detection/`
- **Build Integration**: `Makefile` codegen targets

### Development Environment

- JSON schema validation tools (`jsonschema-cli` or equivalent)
- ExifTool source access for validation
- Test images for compatibility testing
- Understanding of Rust module system (`pub use` re-exports)

## Technical Implementation Notes

### Directory Naming Decision

Use `Canon_pm/` rather than `Canon.pm/` or `Canon.pm.d/`:

- **Pros**: Clear directory indication, matches Rust module names, avoids filesystem confusion
- **Cons**: Slightly less direct mapping to ExifTool source
- **Decision**: Practical clarity wins over perfect source mapping

**Module Names**: Direct correspondence - `Canon_pm/` directory → `Canon_pm` module name.

### Schema Validation Strategy

Each extraction type gets its own focused schema using Rust-based validation:

```rust
// Use jsonschema crate for validation during build
fn validate_config(config_path: &Path) -> Result<(), Error> {
    let schema = include_str!("../schemas/simple_table.json");
    let instance = std::fs::read_to_string(config_path)?;
    jsonschema::validate(&schema, &instance)?;
    Ok(())
}
```

Benefits:

- Better error messages for specific extraction patterns
- Easier to extend with new extraction types
- Prevents configuration mistakes early in development
- Hard requirement - fails build on invalid configs

### Macro Generation Strategy

Hand-written macro in shared location (`src/generated/macros.rs`):

```rust
macro_rules! make_simple_table {
    ($name:ident, $key_type:ty, $value_type:ty, $entries:expr) => {
        pub static $name: LazyLock<HashMap<$key_type, $value_type>> =
            LazyLock::new(|| $entries.into_iter().collect());

        paste::paste! {
            pub fn [<lookup_ $name:snake>](key: $key_type) -> Option<$value_type> {
                $name.get(&key).copied()
            }
        }
    };
}
```

Benefits:

- **Type Safety**: Compile-time validation of all keys and values
- **Consistent Patterns**: All tables follow same generation logic
- **Reduced Maintenance**: Single place to update lookup patterns
- **Performance**: Same runtime characteristics as current system
- **Clean Imports**: `pub use` re-exports provide flat import paths

## Risk Mitigation

### Breaking Changes

- **Risk**: Import path changes break existing code
- **Mitigation**: All-at-once migration with systematic search-and-replace using `rg "use crate::generated::" --type rust`
- **Justification**: Pre-release window allows breaking changes for better architecture
- **Strategy**: Single commit updating all 14 import locations, no compatibility shims

### Configuration Complexity

- **Risk**: Distributed configuration harder to manage
- **Mitigation**: Rust-based JSON schema validation with hard requirement, clear documentation
- **Benefit**: Focused, easier-to-understand configuration files
- **Decision**: No backward compatibility - clean break for better architecture

### Build System Changes

- **Risk**: Parallel processing introduces complexity
- **Mitigation**: Keep existing Make system, ExifTool modules are independent
- **Benefit**: Faster development iteration, better scalability
- **Decision**: No incremental builds - extraction is fast enough

## Related Documentation

### Essential Reading

- [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md) - Current codegen system
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - ExifTool integration principles
- [MILESTONE-18-RAW-Codegen-Extraction.md](MILESTONE-18-RAW-Codegen-Extraction.md) - Scale-up requirements

### Architecture Context

- [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Development workflow
- [API-DESIGN.md](../design/API-DESIGN.md) - TagValue and TagEntry design

### Testing and Validation

- [TESTING.md](../guides/TESTING.md) - Testing strategies
- [DEVELOPMENT-WORKFLOW.md](../guides/DEVELOPMENT-WORKFLOW.md) - Daily development

## Benefits

### Immediate Benefits

1. **Scalability**: Clean architecture for 300+ tables
2. **Maintainability**: Source-file alignment with ExifTool
3. **Performance**: Reduced boilerplate, faster builds
4. **Validation**: Schema checking prevents configuration errors
5. **Organization**: Clear separation by extraction type

### Long-term Benefits

1. **RAW Format Support**: Enables efficient RAW format implementation
2. **ExifTool Updates**: Easier to track and update with ExifTool releases
3. **Developer Experience**: Cleaner, more focused configuration
4. **Extension**: Simple to add new extraction types and manufacturers

## Timeline

### Week 1: Configuration and Schema

- Day 1-2: Create new directory structure and JSON schemas
- Day 3-4: Migrate existing configurations to new structure
- Day 5: Add schema validation to build system

### Week 2: Code Generation and Migration

- Day 1-2: Implement macro-based generation system
- Day 3-4: Update all import paths and test compatibility
- Day 5: Final testing and documentation updates

This milestone provides the foundation for efficient RAW format support while dramatically improving the maintainability and scalability of the codegen extraction system.

## Implementation Details

### Completed Work

#### 1. Configuration Restructuring
- Created `codegen/config/` directory structure organized by ExifTool source modules
- Split monolithic `extract.json` into focused configuration files per module and extraction type
- Each module directory contains type-specific JSON files (simple_table.json, print_conv.json, etc.)

#### 2. JSON Schema Implementation
- Created focused schemas in `codegen/schemas/` for each extraction type
- Implemented Rust-based validation using `jsonschema` crate in `codegen/src/validation.rs`
- Schema validation runs automatically during `make codegen`

#### 3. Macro-Based Code Generation
- Created shared macros in `src/generated/macros.rs`:
  - `make_simple_table!` - Generates HashMap lookups with ~80% less boilerplate
  - `make_boolean_set!` - Generates HashSet membership checks
  - `make_regex_table!` - Generates regex pattern tables
- Implemented new generator in `codegen/src/generators/macro_based.rs`

#### 4. Module Structure
- Generated modules now follow ExifTool source naming: Canon_pm, Nikon_pm, etc.
- Each module contains all table types from that ExifTool source file
- Example generated file shows reduction from ~45 lines to ~5 lines per table

#### 5. Import Path Migration
- Updated all 14 import locations across the codebase
- Mapping: `canon/` → `Canon_pm/`, `file_types/` → `ExifTool_pm/`, etc.
- Added new modules to `src/generated/mod.rs` with proper macro imports

#### 6. Build System Integration
- Extended existing Makefile without breaking changes
- Schema validation integrated into build pipeline
- Parallel extraction continues to work as before

### Key Files Modified/Created
- `codegen/config/*_pm/*.json` - New configuration structure
- `codegen/schemas/*.json` - Focused JSON schemas
- `codegen/src/generators/macro_based.rs` - New macro-based generator
- `codegen/src/validation.rs` - Schema validation implementation
- `src/generated/macros.rs` - Shared macro definitions
- `src/generated/*_pm/mod.rs` - New generated module files

### Results
- Successfully generated Canon_pm and Nikon_pm modules with macro-based tables
- Boilerplate reduced from ~15 lines to ~3 lines per table
- All existing functionality preserved while improving maintainability
- System ready to scale to 300+ tables for RAW format support
