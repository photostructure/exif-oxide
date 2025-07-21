# Technical Project Plan: Enhance Tag Structure Generator for Subdirectories

**Created**: 2025-07-21  
**Status**: Ready for implementation  
**Priority**: High (blocking Olympus RAW support)

## Project Overview

**Goal**: Enhance the codegen `tag_table_structure` generator to automatically generate tag name lookup functions for subdirectory tables (Equipment, CameraSettings, etc.), eliminating manual maintenance of lookup tables.

**Problem**: Currently, subdirectory tables like Olympus Equipment require manual tag name resolution functions (`equipment_tags.rs`), violating CODEGEN.md principles and creating maintenance burden across all manufacturers.

## Background & Context

### Why This Work is Needed

- **Violates CODEGEN.md**: Manual `src/implementations/olympus/equipment_tags.rs` should be auto-generated
- **Affects All Manufacturers**: Canon CameraSettings, Nikon AFInfo, Sony subdirectories have same issue  
- **Maintenance Burden**: Manual lookup tables drift from ExifTool source with monthly releases
- **Blocks Olympus RAW**: Equipment tag name resolution needed for Milestone 17c completion

### Related Documentation

- [MILESTONE-17c-Olympus-RAW.md](MILESTONE-17c-Olympus-RAW.md) - Current blocking issue #2
- [CODEGEN.md](../CODEGEN.md) - Code generation principles and architecture
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Must follow ExifTool exactly

## Technical Foundation

### Key Systems

- **Codegen System**: `codegen/src/generators/tag_structure.rs` - Main generator to enhance
- **Extraction**: `codegen/extractors/tag_table_structure.pl` - Already extracts subdirectory data
- **Configuration**: `codegen/config/Olympus_pm/equipment_tag_table_structure.json` - Already configured

### Current Architecture

```
ExifTool Olympus.pm Equipment table
    ↓ (tag_table_structure.pl)
olympus_equipment_tag_structure.json
    ↓ (tag_structure.rs generator) 
OlympusDataType enum variant only ❌
    
MISSING: Equipment tag lookup function ❌
```

### Target Architecture

```
ExifTool Olympus.pm Equipment table
    ↓ (tag_table_structure.pl) 
olympus_equipment_tag_structure.json
    ↓ (enhanced tag_structure.rs generator)
OlympusDataType enum + Equipment lookup function ✅
```

## Work Completed

### Investigation Phase ✅

- **Root cause identified**: `tag_structure` generator only handles main tables
- **Data availability confirmed**: `olympus_equipment_tag_structure.json` contains all needed data
- **Current workaround analyzed**: Manual `equipment_tags.rs` has 25 tag mappings
- **Extraction verified**: Extraction works correctly, only generation needs enhancement

### Configuration Validated ✅

- `equipment_tag_table_structure.json` is properly configured
- Extraction produces valid JSON with all 25 Equipment tags
- Inline PrintConv generation works (generates `equipment_inline.rs`)

## Remaining Tasks

### High Confidence Implementation Tasks

#### 1. Enhance Tag Structure Generator

**File**: `codegen/src/generators/tag_structure.rs`

**Changes needed**:

```rust
// In generate_tag_structure() function, after main enum generation
// Add subdirectory table detection and lookup function generation

if data.source.table != "Main" {
    // Generate lookup function for subdirectory table
    let lookup_fn_name = format!("get_{}_tag_name", data.source.table.to_lowercase());
    
    code.push_str(&format!(
        "\n/// Get tag name for {} subdirectory\n",
        data.source.table
    ));
    code.push_str(&format!(
        "pub fn {}(tag_id: u16) -> Option<&'static str> {{\n",
        lookup_fn_name
    ));
    code.push_str("    match tag_id {\n");
    
    for tag in &data.tags {
        code.push_str(&format!(
            "        {} => Some(\"{}\"),\n",
            format_tag_id(tag.tag_id_decimal),
            escape_string(&tag.name)
        ));
    }
    
    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n");
}
```

**Helper functions may need**:
- `format_tag_id(id: u16) -> String` - Format as hex (0x0100) or decimal
- Verify `escape_string()` handles tag names correctly

#### 2. Update Equipment Tag Resolution

**File**: `src/exif/ifd.rs` (around line 698)

**Current**:
```rust
use crate::implementations::olympus::equipment_tags::get_equipment_tag_name;
```

**After**:
```rust  
use crate::generated::Olympus_pm::tag_structure::get_equipment_tag_name;
```

#### 3. Remove Manual Implementation

**Action**: Delete `src/implementations/olympus/equipment_tags.rs`

**Update**: Remove from `src/implementations/olympus/mod.rs` exports

### Medium Confidence Tasks (Need Research)

#### 4. Verify Generated Code Integration

**Research needed**: 
- Confirm generated lookup function integrates with IFD processor
- Test function signature matches current usage
- Validate hex vs decimal tag ID formatting

#### 5. Test All Manufacturers

**Research scope**:
- Identify other subdirectory tables needing similar treatment
- Canon: CameraSettings, AFInfo, etc.  
- Nikon: AFInfo, ColorBalance, etc.
- Sony: Tag2010e, Tag2010g, etc.

## Prerequisites

### Before Starting

- **Codegen environment**: `make codegen` must run successfully
- **ExifTool submodule**: Must be at correct commit for extraction
- **Test images**: Olympus ORF file for validation (`test-images/olympus/test.orf`)

### No Blocking Dependencies

This work can proceed independently - all required extraction and configuration is already complete.

## Testing Strategy

### Unit Tests

- Test new lookup function generation logic
- Verify correct hex formatting of tag IDs
- Validate string escaping for tag names

### Integration Tests

```bash
# 1. Regenerate codegen with enhancement
make codegen

# 2. Verify Equipment lookup function exists  
grep -r "get_equipment_tag_name" src/generated/Olympus_pm/

# 3. Test Olympus ORF processing
cargo run -- test-images/olympus/test.orf | grep -E "CameraType2|SerialNumber|LensType"

# 4. Compare with ExifTool
cargo run --bin compare-with-exiftool test-images/olympus/test.orf
```

### Manual Validation

1. **Generated code inspection**: Review generated lookup functions for correctness
2. **Cross-manufacturer testing**: Verify approach works for Canon, Nikon subdirectories  
3. **ExifTool comparison**: Ensure tag names match ExifTool exactly

## Success Criteria & Quality Gates

### Definition of Done

- [ ] Enhanced tag_structure generator handles subdirectory tables
- [ ] Equipment lookup function generated automatically  
- [ ] Manual `equipment_tags.rs` deleted
- [ ] Olympus ORF equipment tags resolve with correct names
- [ ] `make precommit` passes
- [ ] ExifTool compatibility maintained

### Quality Gates

- **Code review**: Enhanced generator follows existing patterns
- **Documentation**: Update CODEGEN.md if architecture changes
- **Testing**: All existing tests pass + new subdirectory tests

## Gotchas & Tribal Knowledge

### Technical Constraints

- **Trust ExifTool**: Generated lookup must match ExifTool tag names exactly
- **Hex formatting**: Tag IDs should be formatted as `0x0100`, not `256`
- **String escaping**: Tag names may contain special characters requiring escaping

### Implementation Details

- **Generator location**: Enhancement goes in `generate_tag_structure()`, not new generator
- **File placement**: Generated functions belong in same file as enum (tag_structure.rs)
- **Naming convention**: Use `get_{table_name}_tag_name` pattern for consistency

### Avoiding Pitfalls

- **Don't break main tables**: Ensure Main table generation unaffected  
- **Preserve enum variants**: Subdirectory variants in main enum must remain
- **Consider all manufacturers**: Solution should work for Canon/Nikon/Sony subdirectories

### Future Considerations

- **Scalability**: This approach should handle 50+ subdirectory tables across manufacturers
- **Maintenance**: Monthly ExifTool updates should regenerate all lookup functions automatically
- **Performance**: Generated functions use simple match statements (O(1) lookup acceptable)

## Implementation Notes

### Priority Order

1. **Olympus Equipment** - Immediate need for Milestone 17c
2. **Canon CameraSettings** - High-frequency usage  
3. **Other subdirectories** - As needed basis

### Risk Mitigation

- **Backup approach**: If enhancement proves complex, create dedicated subdirectory extractor
- **Rollback plan**: Manual `equipment_tags.rs` can be restored if needed
- **Testing early**: Validate Olympus works before tackling other manufacturers