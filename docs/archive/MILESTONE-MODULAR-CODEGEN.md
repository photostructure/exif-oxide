# Milestone: Codegen Architecture Refactor

**Status**: 🏗️ **PLANNING**  
**Estimated Duration**: **2-3 weeks**  
**Priority**: **CRITICAL** - Foundation for all future milestones

## Summary

Refactor the monolithic codegen architecture into a modular, extensible system that can scale with upcoming milestones. The current approach of massive Perl scripts generating huge JSON payloads will become unmanageable as we add XMP/XML, MIME detection, RAW formats, and video format support.

## 🎯 **Goals**

### Primary Goal

Transform codegen from monolithic scripts to modular, debuggable architecture that scales with project growth.

### Secondary Goals

- **Improve Debugging**: Individual JSON files instead of 117K+ token payloads
- **Enable Parallelization**: Independent extractors can run concurrently
- **Increase Type Safety**: Clean schemas instead of complex discriminated unions
- **Future-Proof**: Easy to add new extraction types for upcoming milestones

## 🚨 **Current Pain Points**

### Monolithic Complexity

- `extract_tables.pl` (534 lines): EXIF tags + GPS tags + composite tags
- `extract_simple_tables.pl` (365 lines): Simple tables + regex patterns + file type lookup
- `main.rs` (1374 lines): All JSON schemas + all Rust generation

### Debugging Nightmare

- `simple_tables.json`: 117K+ tokens making failures impossible to debug
- Monolithic failure: One bad table breaks entire codegen
- Schema complexity: Discriminated unions mixed with simple tables

### Extensibility Problems

- Adding new extraction types requires modifying complex existing scripts
- No clear separation between different extraction patterns
- Rust codegen uses complex conditional logic for different JSON schemas

### Future Growth Risk

Upcoming milestones will add many more extraction types:

- **Milestone 15**: XMP/XML namespace extraction
- **Milestone 16**: MIME type pattern matching
- **Milestone 17**: RAW format magic numbers
- **Milestone 18**: Video container parsing patterns

## 🏗️ **Proposed Architecture**

### Core Principle: One Extractor Per Concern

```
ExifTool Source → Specialized Perl Extractors → Individual JSON Files → Targeted Rust Generators → Generated Code
```

### New Directory Structure

```
codegen/
├── lib/
│   └── ExifToolExtract.pm         # DRY common utilities
├── extractors/
│   ├── simple_tables.pl           # Simple lookup tables only
│   ├── tag_tables.pl              # EXIF/GPS tag definitions only
│   ├── composite_tags.pl          # Composite tag definitions only
│   ├── regex_patterns.pl          # Magic number patterns only
│   ├── file_type_lookup.pl        # File type discriminated unions only
│   └── xmp_namespaces.pl          # Future: XMP namespace extraction
├── generated/
│   ├── simple_tables/             # Individual JSON per table
│   │   ├── canon_white_balance.json
│   │   ├── nikon_lenses.json
│   │   ├── sony_models.json
│   │   └── ...
│   ├── tag_tables.json
│   ├── composite_tags.json
│   ├── regex_patterns.json
│   ├── file_type_lookup.json
│   └── xmp_namespaces.json        # Future milestone
└── src/
    ├── main.rs                    # Orchestrator only
    ├── generators/
    │   ├── simple_tables.rs       # Generate lookup tables
    │   ├── tag_tables.rs          # Generate tag definitions
    │   ├── composite_tags.rs      # Generate composite tags
    │   ├── regex_patterns.rs      # Generate regex tables
    │   ├── file_type_lookup.rs    # Generate file type infrastructure
    │   └── xmp_namespaces.rs      # Future: XMP namespace generation
    ├── schemas/
    │   ├── simple_table.rs        # Type-safe simple table schema
    │   ├── tag_table.rs           # Type-safe tag schema
    │   ├── composite_tag.rs       # Type-safe composite schema
    │   ├── regex_pattern.rs       # Type-safe regex schema
    │   └── file_type_lookup.rs    # Type-safe file type schema
    └── common/
        └── utils.rs               # Shared codegen utilities
```

## 📋 **Implementation Plan**

### Phase 1: Extract Common Library (3-4 days)

**Goal**: Create DRY foundation for all extractors

**Deliverables**:

- `lib/ExifToolExtract.pm` with shared utilities:
  - `load_module_from_file()` - Safe Perl module loading
  - `get_package_hash()` - Access to package variables
  - `validate_primitive_table()` - Primitive data validation
  - `format_json_output()` - Consistent JSON formatting
  - `extract_source_line_info()` - Traceability utilities

**Success Criteria**:

- All existing extractors can use common library functions
- No code duplication between extractors
- Maintains backward compatibility with current output

### Phase 2: Split Perl Extractors (4-5 days)

**Goal**: Replace monoliths with focused extractors

**Deliverables**:

- `extractors/simple_tables.pl` - Generate individual JSON per table
- `extractors/tag_tables.pl` - EXIF/GPS tags only
- `extractors/composite_tags.pl` - Composite tags only
- `extractors/regex_patterns.pl` - Magic number patterns only
- `extractors/file_type_lookup.pl` - File type lookup only

**Success Criteria**:

- Each extractor <200 lines and single responsibility
- Generated JSON files are human-readable and debuggable
- Individual tables can be regenerated independently
- Total output matches current functionality

### Phase 3: Modularize Rust Codegen (3-4 days)

**Goal**: Clean type-safe generators

**Deliverables**:

- Split `main.rs` into specialized generators
- Type-safe schemas for each extraction type in `schemas/`
- Clean generator modules in `generators/`
- Shared utilities in `common/utils.rs`

**Success Criteria**:

- No more discriminated union complexity
- Each generator <300 lines with clear responsibility
- Type-safe deserialization with proper error handling
- Generated Rust code identical to current output

### Phase 4: Update Build System (1-2 days)

**Goal**: Parallel execution and incremental regeneration

**Deliverables**:

- Updated Makefile targets for individual extractors
- Parallel execution support where possible
- Incremental regeneration (only changed tables)
- Integration with existing `make codegen` workflow

**Success Criteria**:

- `make codegen` still works as primary interface
- Individual extractors can be run independently
- Faster builds through parallelization
- Clear error attribution when something fails

## 🎯 **Key Benefits**

### 1. Granular Debugging

- Each table gets own JSON file (`canon_white_balance.json`, `nikon_lenses.json`)
- Easy to identify exactly which table failed and why
- Individual files are human-readable and debuggable

### 2. Failure Isolation

- One bad table doesn't break entire codegen process
- Can regenerate just the tables that changed
- Clear error attribution to specific extractor

### 3. Type Safety & Clean Code

- Each extraction type gets dedicated Rust schemas
- No more discriminated union complexity in single generator
- Pattern matching becomes clean and type-safe

### 4. Parallelization

- Can run multiple extractors concurrently
- Independent JSON files can be generated in parallel
- Rust generators can run independently

### 5. Extensibility

- New extraction types are just new extractors + generators
- Zero impact on existing code
- Clear template to follow for new patterns

### 6. Future Milestone Readiness

- **Milestone 15 (XMP)**: Add `extractors/xmp_namespaces.pl` + `generators/xmp_namespaces.rs`
- **Milestone 16 (MIME)**: Add `extractors/mime_patterns.pl` + `generators/mime_patterns.rs`
- **Milestone 17 (RAW)**: Add `extractors/raw_formats.pl` + `generators/raw_formats.rs`
- **Milestone 18 (Video)**: Add `extractors/video_containers.pl` + `generators/video_containers.rs`

## ✅ **Success Criteria**

### Functional Requirements

- [ ] All current codegen output remains identical
- [ ] Individual extractors can run independently
- [ ] Failed table doesn't break entire build
- [ ] Human-readable JSON files for debugging
- [ ] All tests continue to pass

### Performance Requirements

- [ ] Build time same or faster through parallelization
- [ ] Memory usage reduced (no massive JSON payloads)
- [ ] Incremental regeneration works correctly

### Maintainability Requirements

- [ ] Each extractor <200 lines with single responsibility
- [ ] Each generator <300 lines with clear purpose
- [ ] No code duplication (DRY library usage)
- [ ] Clear template for adding new extraction types

### Integration Requirements

- [ ] `make codegen` workflow unchanged for users
- [ ] All existing generated code compiles cleanly
- [ ] Compatible with current Makefile targets
- [ ] Ready for upcoming milestone requirements

## 🧪 **Testing Strategy**

### 1. Regression Testing

- Compare all generated code before/after refactor
- Ensure identical output from modular vs monolithic approach
- Validate all existing tests continue to pass

### 2. Integration Testing

- Test individual extractors independently
- Test parallel execution scenarios
- Test failure isolation (one bad table doesn't break others)

### 3. Performance Testing

- Compare build times before/after refactor
- Measure memory usage with individual JSON files
- Validate incremental regeneration performance

### 4. Extensibility Testing

- Add a mock new extraction type to validate template
- Ensure future milestone patterns work correctly

## 🔗 **Related Documentation**

- **[CODEGEN.md](../design/CODEGEN.md)**: Core codegen philosophy and patterns
- **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)**: Fundamental principle driving this work
- **[MILESTONE-CODEGEN-SIMPLE-TABLES.md](../archive/MILESTONE-CODEGEN-SIMPLE-TABLES.md)**: Previous codegen milestone for context

## 🚧 **Risk Mitigation**

### Risk: Breaking Existing Functionality

**Mitigation**: Comprehensive regression testing, identical output validation, backward compatibility

### Risk: Build System Complexity

**Mitigation**: Maintain simple `make codegen` interface, clear error messages, documented recovery procedures

### Risk: Over-Engineering

**Mitigation**: Implement incrementally, validate each phase, focus on clear immediate benefits

### Risk: Performance Regression

**Mitigation**: Benchmark before/after, optimize hot paths, leverage parallelization benefits

## 🎉 **Expected Impact**

### Immediate Benefits

- **10x faster debugging** through individual JSON files
- **Clear error attribution** to specific extractors
- **Type-safe code generation** with clean schemas
- **Parallel build execution** for faster development cycles

### Long-term Benefits

- **Easy addition of new extraction types** for upcoming milestones
- **Zero impact** when adding XMP, MIME, RAW, video support
- **Maintenance-free scalability** as project grows
- **Clear template** for any future codegen needs

This refactor establishes the foundation for all upcoming codegen-heavy milestones while immediately improving the development experience through better debugging and faster builds.
