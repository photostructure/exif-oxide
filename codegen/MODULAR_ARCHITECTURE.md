# Modularized Codegen Architecture

## Overview

The refactored codegen architecture splits the monolithic 1374-line `main.rs` into a clean, modular structure that's easier to maintain and extend.

## Directory Structure

```
codegen/src/
├── main.rs                    # Orchestrator only (~150 lines)
├── common/
│   ├── mod.rs                # Module exports
│   └── utils.rs              # Shared utilities (escape_string, parse_hex_id, etc.)
├── schemas/
│   ├── mod.rs                # Module exports
│   ├── input.rs              # Input JSON schemas (ExtractedData, TableEntry, etc.)
│   └── output.rs             # Generated Rust types (TagDef, CompositeTagDef, etc.)
├── generators/
│   ├── mod.rs                # Module exports
│   ├── tags.rs               # EXIF tag table generator
│   ├── composite_tags.rs     # Composite tag generator
│   ├── conversion_refs.rs    # PrintConv/ValueConv reference generator
│   ├── supported_tags.rs     # Milestone-based supported tags generator
│   ├── simple_tables/
│   │   ├── mod.rs           # Simple table orchestrator
│   │   ├── standard.rs      # Standard lookup table generator
│   │   ├── regex.rs         # Regex pattern table generator
│   │   └── file_type.rs     # File type lookup generator
│   └── module.rs             # Module file generator
└── config/
    └── mod.rs                # Configuration constants (MILESTONE_COMPLETIONS)
```

## Key Benefits

### 1. **Clear Separation of Concerns**

- Each generator focuses on one type of code generation
- Schemas are centralized and reusable
- Common utilities are shared across all generators

### 2. **Type Safety**

- Input schemas validate JSON from Perl extractors
- Output schemas define the Rust structures we generate
- No more discriminated unions in the main logic

### 3. **Extensibility**

- Adding new extraction types is straightforward
- New generators can be added as separate modules
- Future milestones (XMP, MIME, RAW, Video) just add new generators

### 4. **Maintainability**

- Each file is focused and <300 lines
- Easy to find and modify specific generation logic
- Clear dependencies between modules

### 5. **Testing**

- Each generator can be unit tested independently
- Mock schemas for testing
- Integration tests can verify the full pipeline

## Example: Adding a New Generator

To add support for XMP namespaces (Milestone 15):

1. **Add Perl Extractor**: `extractors/xmp_namespaces.pl`
2. **Add Input Schema**: Update `schemas/input.rs` with `XmpNamespace` struct
3. **Add Generator**: Create `generators/xmp_namespaces.rs`
4. **Update Orchestrator**: Add call in `main.rs`

## Migration Status

### Phase 1: Common Library ✅

- Created `lib/ExifToolExtract.pm` with shared Perl utilities
- All extractors now use common functions
- No code duplication

### Phase 2: Split Perl Extractors ✅

- `simple_tables.pl` - Generates individual JSON files per table
- `tag_tables.pl` - EXIF/GPS tag definitions only
- `composite_tags.pl` - Composite tag definitions only
- `regex_patterns.pl` - Magic number patterns only
- `file_type_lookup.pl` - File type discriminated unions only

### Phase 3: Modularize Rust Codegen (In Progress)

- Created directory structure
- Extracted common utilities
- Created type-safe schemas
- Started generator modules

### Phase 4: Update Build System (Pending)

- Update Makefile for parallel execution
- Support incremental regeneration
- Individual extractor targets

## Usage

```bash
# Full codegen pipeline
make codegen

# Or run individual extractors
perl extractors/simple_tables.pl
perl extractors/tag_tables.pl > generated/tag_tables.json

# Run Rust codegen
cd codegen && cargo run
```

## Next Steps

1. Complete extraction of all generators
2. Update Makefile for parallel execution
3. Add integration tests
4. Document adding new extraction types
5. Optimize for incremental regeneration

This modular architecture provides a solid foundation for all future codegen needs while immediately improving development experience through better organization and faster builds.
