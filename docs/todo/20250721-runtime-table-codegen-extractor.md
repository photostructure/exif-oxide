# Technical Project Plan: Runtime Table Codegen Extractor

## Project Overview

**Goal**: Create a new codegen extractor that automatically generates runtime HashMap initialization code for ProcessBinaryData patterns, eliminating manual table maintenance while preserving runtime flexibility.

**Problem Statement**: Current `simple_table.pl` extractor generates static `LazyLock` tables suitable for PrintConv lookups, but cannot handle ProcessBinaryData patterns that require runtime HashMap creation with conditional logic, dynamic values, and binary data processing contexts.

## Background & Context

### Why This Work Is Needed

- **Runtime Flexibility Required**: ProcessBinaryData tables need runtime context (camera model, binary data layout) that static tables cannot provide
- **Manual Table Maintenance**: 15+ runtime HashMap initializations in `src/implementations/canon/binary_data.rs` require manual ExifTool synchronization
- **Binary Processing Complexity**: Tables include conditional expressions, format specifications, and data member references that need dynamic evaluation
- **Trust ExifTool Principle**: Manual maintenance violates TRUST-EXIFTOOL.md - these tables must be generated from ExifTool source

### Related Documentation

- [CODEGEN.md](../CODEGEN.md) - Existing codegen infrastructure
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Must translate ExifTool exactly
- [PROCESSOR-DISPATCH.md](../guides/PROCESSOR-DISPATCH.md) - ProcessBinaryData architecture context

## Technical Foundation

### ExifTool Source Patterns

**ProcessBinaryData Tables**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm`

- Lines 2172-2400: Camera settings table definitions
- Lines 2640-2800: AF info table structures
- Lines 2719-2900: My colors processing tables

**Key Characteristics**:

- Perl hashes with complex value structures: `{ Name => 'TagName', PrintConv => { 0 => 'Off' } }`
- Conditional expressions: `Condition => '$self->{Model} =~ /EOS/'`
- Format specifications: `Format => 'int16s'`
- Data member references: `DataMember => 'CanonImageWidth'`

### Current Codegen Infrastructure

**Available Extractors**:

- `simple_table.pl` - Static LazyLock tables ✅
- `process_binary_data.pl` - Table structure extraction ✅
- `conditional_tags.pl` - Conditional logic ✅

**Gap**: No extractor for **runtime HashMap initialization code** that preserves ExifTool's complex table logic.

## Work Completed

### Analysis Phase ✅

**Pattern Identification**:

- **15+ runtime HashMap patterns** in Canon binary data processing
- **Dynamic table creation** in AF info processing
- **Category-based grouping** in Nikon lens database
- **Parameter mapping** in encryption contexts

**Key Finding**: These patterns require **generated code that creates HashMaps at runtime** rather than static lookup tables.

## Remaining Tasks

### Phase 1: Research & Design (High Confidence)

1. **Analyze ExifTool ProcessBinaryData structures** in Canon.pm and Nikon.pm

   - Map Perl hash patterns to Rust HashMap initialization
   - Identify conditional logic patterns
   - Document format specifications and data member usage

2. **Design `runtime_table.pl` extractor**

   - Extract ProcessBinaryData table definitions from ExifTool
   - Parse complex value structures with PrintConv, Condition, Format
   - Generate JSON intermediate representation

3. **Create configuration schema** for runtime table extraction
   - Define `runtime_table.json` structure in `codegen/config/*/`
   - Specify table names, conditional logic, format handling
   - Support data member references and complex expressions

### Phase 2: Implementation (Requires Research)

4. **Implement Rust code generation** for runtime HashMap patterns

   - Generate `create_*_table()` functions that build HashMaps at runtime
   - Preserve conditional logic and dynamic value computation
   - Maintain compatibility with existing ProcessBinaryData architecture

5. **Integrate with build system**
   - Add `runtime_table` generator to `codegen/src/generators/`
   - Wire into orchestration system
   - Update configuration discovery and processing

### Phase 3: Migration (Requires Implementation Design)

6. **Migrate Canon binary data tables** using new extractor

   - Replace manual HashMap initialization with generated functions
   - Validate binary data processing compatibility
   - Test with real Canon image files

7. **Extend to other manufacturers** (Nikon, Sony, Olympus)
   - Apply runtime table extraction to other ProcessBinaryData patterns
   - Validate manufacturer-specific processing logic

## Prerequisites

### Before Starting

- **ExifTool submodule** at correct commit for extraction
- **Codegen environment** fully functional (`make codegen` passes)
- **Understanding of ProcessBinaryData architecture** from PROCESSOR-DISPATCH.md
- **Canon test images** for binary data processing validation

### No Blocking Dependencies

This work can proceed independently - it extends the existing codegen infrastructure without breaking changes.

## Testing Strategy

### Unit Tests

- Test runtime table generation with known Canon ProcessBinaryData examples
- Validate conditional logic extraction and code generation
- Compare generated vs manual HashMap initialization results

### Integration Tests

```bash
# Generate runtime tables
make codegen

# Verify generated functions exist
ls src/generated/Canon_pm/runtime_*

# Test Canon binary data processing
cargo run -- test-images/canon/test.cr2 | grep -E "MacroMode|AFMode|Quality"

# Compare with ExifTool
cargo run --bin compare-with-exiftool test-images/canon/test.cr2
```

### Manual Testing Steps

1. **Extract test table** from Canon.pm using new extractor
2. **Generate runtime HashMap code** and verify compilation
3. **Process Canon binary data** with generated tables
4. **Compare tag extraction** with ExifTool reference output

## Success Criteria & Quality Gates

**Definition of Done**:

- [ ] `runtime_table.pl` extractor successfully extracts Canon ProcessBinaryData tables
- [ ] Generated code compiles and produces functionally equivalent runtime HashMaps
- [ ] Binary data processing produces identical results to manual implementation
- [ ] `make precommit` passes with generated code
- [ ] Documentation updated with new extraction patterns

**Quality Gates**:

- Code review focusing on ExifTool translation accuracy
- Binary compatibility testing with representative Canon files
- Performance validation (generated tables should not degrade processing speed)

## Gotchas & Tribal Knowledge

### ExifTool ProcessBinaryData Complexity

**Complex Value Structures**: ExifTool tables contain nested Perl structures that need careful parsing:

```perl
# Example from Canon.pm
1 => { Name => 'MacroMode', PrintConv => { 1 => 'Macro', 2 => 'Normal' } }
```

**Conditional Logic**: Tables include model-specific conditions that affect table creation:

```perl
Condition => '$self->{Model} =~ /EOS/'
```

**Format Dependencies**: Binary data format affects value interpretation:

```perl
Format => 'int16s'  # Signed 16-bit integers
```

### Runtime vs Static Table Trade-offs

**Runtime Flexibility**: ProcessBinaryData requires runtime context that static tables cannot provide
**Performance**: Generated runtime tables should maintain equivalent performance to manual implementation
**Memory Usage**: Consider table creation frequency and memory lifecycle

### Integration Points

**ProcessBinaryData Architecture**: Generated tables must integrate with existing `BinaryDataProcessor` trait
**Expression Evaluation**: Complex ExifTool expressions may require `expressions/` module integration
**Data Member References**: Tables reference computed values that need runtime availability

## Next Steps for Implementation

1. **Start with Canon camera settings table** (simplest ProcessBinaryData pattern)
2. **Focus on PrintConv extraction first** (defer conditional logic to later phases)
3. **Generate minimal viable runtime HashMap** before adding complexity
4. **Test incrementally** with single table before scaling to full Canon module

This work extends the successful codegen pattern to a new class of ExifTool structures requiring runtime generation.
