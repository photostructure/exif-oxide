# Milestone 16: Codegen Architecture Clarity

## Summary

Refactor the confusing "simple_tables" naming and architecture into logical, purpose-driven modules that clearly communicate their functionality. This milestone addresses developer experience issues where the current "simple_tables" system has evolved far beyond its original scope, creating cognitive overhead for new engineers.

## Problem Statement

The current `simple_tables` system suffers from several architectural clarity issues:

### 1. **Misleading Naming**
- Called "simple_tables" but handles 4 different extraction types
- Generates complex enum structures, regex patterns, and boolean sets
- Over 600 lines of complex code generation logic

### 2. **Cognitive Overhead**
- New engineers must understand multiple patterns: `simple_table`, `regex_strings`, `file_type_lookup`, `boolean_set`
- Each pattern has different Rust output structures and helper functions
- Complex type system with varying key/value types

### 3. **Scattered Concepts**
- File type detection mixed with manufacturer lookup tables
- Regex pattern extraction bundled with simple key-value lookups
- Boolean set logic different from table logic

## Solution: Logical Module Organization

### **New Architecture**

```
codegen/src/generators/
├── lookup_tables/     # Pure key-value lookups
│   ├── mod.rs        # Canon white balance, Nikon lens IDs, etc.
│   └── standard.rs   # Simple HashMap generation
├── file_detection/   # File type detection system
│   ├── mod.rs        # Orchestrator
│   ├── patterns.rs   # Magic number patterns (regex)
│   ├── types.rs      # File type discriminated unions
│   ├── mime.rs       # MIME type mappings
│   └── extensions.rs # Extension mappings
└── data_sets/        # Boolean membership testing
    ├── mod.rs        # HashSet-based lookups
    └── boolean.rs    # Simple is_X() function generation
```

### **Benefits**
- **Clear Boundaries**: Each module has a single, well-defined responsibility
- **Intuitive Naming**: Names directly match actual functionality
- **Easier Maintenance**: Related functionality grouped together
- **Better Documentation**: Each module can have focused documentation
- **Simpler Onboarding**: New engineers understand purpose immediately
- **Future-Proofing**: Clean architecture for upcoming milestones (XMP, RAW, Video)

## Implementation Plan

### **Phase 1: Architecture Setup (3 days)**

#### Tasks:
1. **Create New Module Structure**
   - Create `codegen/src/generators/lookup_tables/` directory
   - Create `codegen/src/generators/file_detection/` directory  
   - Create `codegen/src/generators/data_sets/` directory
   - Add proper mod.rs files with documentation

2. **Split Current simple_tables.rs**
   - Extract pure HashMap generation logic → `lookup_tables/standard.rs`
   - Extract file type logic → `file_detection/types.rs`
   - Extract regex patterns → `file_detection/patterns.rs`
   - Extract boolean sets → `data_sets/boolean.rs`

3. **Update Configuration**
   - Modify `simple_tables.json` to use new extraction type categories
   - Update configuration validation to match new structure

#### Deliverables:
- New directory structure with focused modules
- Extracted generation logic in appropriate modules
- Updated configuration files

### **Phase 2: Generator Implementation (4 days)**

#### Tasks:
1. **Implement lookup_tables/ Module**
   ```rust
   // lookup_tables/standard.rs
   pub fn generate_lookup_table(config: &TableConfig, entries: &[TableEntry]) -> Result<String> {
       // Clean, focused HashMap generation
       // No regex patterns, no file types, no boolean sets
   }
   ```

2. **Implement file_detection/ Module**
   ```rust
   // file_detection/mod.rs
   pub fn generate_file_detection_code(config: &FileDetectionConfig) -> Result<()> {
       // Orchestrate all file detection code generation
   }
   
   // file_detection/types.rs
   pub fn generate_file_type_lookup(lookups: &[FileTypeLookup]) -> Result<String> {
       // Discriminated unions with aliases
   }
   
   // file_detection/patterns.rs  
   pub fn generate_magic_patterns(patterns: &[RegexPattern]) -> Result<String> {
       // Regex patterns for magic number detection
   }
   ```

3. **Implement data_sets/ Module**
   ```rust
   // data_sets/boolean.rs
   pub fn generate_boolean_set(config: &BooleanSetConfig, keys: &[String]) -> Result<String> {
       // HashSet generation with is_X() functions
   }
   ```

4. **Update Main Generator**
   - Modify `codegen/src/main.rs` to use new module structure
   - Remove old `simple_tables` references
   - Add calls to new focused generators

#### Deliverables:
- Working generator modules with clean separation
- Updated main.rs orchestration
- All existing functionality preserved

### **Phase 3: Migration and Testing (2 days)**

#### Tasks:
1. **Migrate Existing Generated Code**
   - Regenerate all lookup tables using new generators
   - Verify output matches existing patterns
   - Update import statements in main codebase

2. **Update Build System**
   - Modify Makefile targets to use new structure
   - Update extraction scripts if needed
   - Test parallel generation

3. **Documentation and Examples**
   - Update EXIFTOOL-INTEGRATION.md with new architecture
   - Add examples for each generator type
   - Document migration path for future tables

4. **Comprehensive Testing**
   - Run full test suite to ensure no regressions
   - Test `make precommit` passes
   - Verify all file detection still works
   - Test lookup table functionality

#### Deliverables:
- Migrated codebase with new architecture
- Updated documentation
- Passing test suite
- Working build system

## Success Criteria

### **Functional Requirements**
- [ ] All existing generated code functionality preserved
- [ ] `make precommit` passes with no regressions
- [ ] File type detection continues to work correctly
- [ ] All lookup tables generate identical output
- [ ] Boolean sets function as before

### **Architecture Requirements**
- [ ] New engineers can understand module purpose from names
- [ ] Each module has single, clear responsibility
- [ ] No code duplication between modules
- [ ] Clean separation of concerns
- [ ] Documentation clearly explains each module's role

### **Developer Experience**
- [ ] Adding new lookup tables is straightforward
- [ ] File detection extensions are intuitive
- [ ] Boolean set additions are simple
- [ ] Build system remains fast and parallel
- [ ] Error messages are clear and helpful

## Example Code Comparisons

### **Before: Confusing Mixed Responsibilities**
```rust
// simple_tables.rs - 600+ lines mixing everything
fn generate_table_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    let extraction_type = table_data.extraction_type.as_deref().unwrap_or("simple_table");
    
    match extraction_type {
        "regex_strings" => generate_regex_table_code(hash_name, table_data),      // File detection
        "file_type_lookup" => generate_file_type_lookup_code(hash_name, table_data), // File detection  
        "boolean_set" => generate_boolean_set_code(hash_name, table_data),        // Data sets
        _ => generate_simple_table_code(hash_name, table_data),                   // Lookup tables
    }
}
```

### **After: Clear Focused Modules**
```rust
// lookup_tables/standard.rs - ~150 lines, one responsibility
pub fn generate_lookup_table(config: &TableConfig, entries: &[TableEntry]) -> Result<String> {
    // Only HashMap generation, nothing else
}

// file_detection/types.rs - ~200 lines, focused on file types
pub fn generate_file_type_lookup(lookups: &[FileTypeLookup]) -> Result<String> {
    // Only file type discriminated unions
}

// data_sets/boolean.rs - ~100 lines, focused on sets
pub fn generate_boolean_set(config: &BooleanSetConfig, keys: &[String]) -> Result<String> {
    // Only HashSet generation
}
```

## Risk Mitigation

### **Regression Prevention**
- Comprehensive test suite before changes
- Output comparison testing during migration
- Incremental rollout of new generators

### **Build System Stability**
- Parallel testing of old and new generators
- Fallback mechanisms during transition
- Makefile targets for both systems during migration

### **Documentation Continuity**
- Update all references to old structure
- Maintain compatibility examples
- Clear migration guide for future developers

## Related Documentation

- [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md) - Integration patterns
- [MODULAR_ARCHITECTURE.md](../../codegen/MODULAR_ARCHITECTURE.md) - Current codegen structure
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core principles for implementation
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Developer onboarding guide

## Future Impact

This refactoring will significantly benefit upcoming milestones:

### **Milestone 15: XMP/XML Support**
- Clean separation makes adding XMP namespace tables straightforward
- File detection improvements help with XMP sidecar files

### **Milestone 17: RAW Format Support**
- File detection module can easily handle RAW magic numbers
- Manufacturer lookup tables fit cleanly in lookup_tables/

### **Milestone 18: Video Format Support**
- File detection patterns extend naturally to video formats
- Clean architecture makes video-specific lookups easy to add

## Completion Timeline

- **Total Duration**: 2 weeks
- **Phase 1**: 3 days (Architecture Setup)
- **Phase 2**: 4 days (Generator Implementation)  
- **Phase 3**: 2 days (Migration and Testing)
- **Buffer**: 1 day for unexpected issues

## Context for Next Engineer

This milestone focuses on **developer experience** and **maintainability** rather than new functionality. The current system works but has grown organically beyond its original scope. This refactoring will:

1. **Reduce cognitive load** for new engineers
2. **Improve code organization** for future development
3. **Make the system more maintainable** as it grows
4. **Provide clear extension points** for upcoming milestones

The work is primarily about **reorganization** and **clarification** - all existing functionality must be preserved while making the architecture more intuitive and maintainable.