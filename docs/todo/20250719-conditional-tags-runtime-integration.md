# Conditional Tags Runtime Integration

**Project**: Wire generated conditional tag resolvers into EXIF parsing pipeline  
**Date**: 2025-07-19  
**Engineer**: Next team member  
**Status**: Ready for implementation  

## Project Overview

**Goal**: Enable runtime conditional tag resolution for Canon, Nikon, and other manufacturers using generated code from MILESTONE-17 extractors.

**Problem**: All 5 universal codegen extractors are complete and generate 1,130+ lines of working Rust code, but **none of the generated conditional logic is used at runtime**. Canon ColorData tags that should resolve differently based on count/model/format currently use static tag names.

## Background & Context

### MILESTONE-17 Status
- ✅ **Universal Codegen Infrastructure Complete**: All 5 extractors working
- ✅ **Generated APIs Available**: `CanonConditionalTags::resolve_tag()` compiles successfully  
- ❌ **Runtime Integration Missing**: Generated code never called during EXIF parsing

### Why This Work Is Critical
ExifTool uses complex conditional logic to resolve tag names dynamically:
- Canon tag ID `16385` → `ColorData1` if count=582, `ColorData4` if count=692
- Canon tag ID `13` → `CanonCameraInfo5D` if model matches `EOS 5D`
- Canon tag ID `16405` → `VignettingCorr` if binary data starts with `\0`

Currently, our parser uses static tag names, missing these manufacturer-specific patterns.

### Related Documentation
- [MILESTONE-17-UNIVERSAL-CODEGEN-EXTRACTORS.md](MILESTONE-17-UNIVERSAL-CODEGEN-EXTRACTORS.md) - Codegen infrastructure details
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core principle: trust ExifTool's logic exactly
- [PROCESSOR-DISPATCH.md](../guides/PROCESSOR-DISPATCH.md) - Existing condition evaluation system

## Technical Foundation

### Key Codebases
- `src/generated/Canon_pm/main_conditional_tags.rs` - Generated conditional tag resolver (842 lines)
- `src/processor_registry/conditions/` - Existing Perl condition evaluation system
- `codegen/src/generators/conditional_tags.rs` - Generator that creates conditional resolvers
- `codegen/extractors/conditional_tags.pl` - Perl extractor that parses ExifTool conditions

### APIs to Understand
```rust
// Generated API (currently unused)
impl CanonConditionalTags {
    pub fn resolve_tag(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag>
}

// Existing condition system (working)
impl ConditionEvaluator {
    pub fn evaluate_condition(&self, condition: &Condition, context: &ProcessorContext) -> bool
}
```

### Current Condition Patterns
The codebase already successfully evaluates Perl conditions in several places:
- `src/implementations/canon/af_info.rs` - Model-based conditions (`$$self{Model} =~ /EOS/`)
- `src/processor_registry/conditions/parser.rs` - Parses Perl expressions into Rust types
- `src/processor_registry/conditions/mod.rs` - Runtime evaluation engine

## Work Completed

### Generated Infrastructure
- **CanonConditionalTags**: 6 conditional arrays, 15 count conditions, 4 binary patterns
- **Condition Data Structures**: `ConditionalContext`, `ResolvedTag`, `ConditionalEntry`
- **Build Integration**: Full codegen pipeline with `make precommit` passing
- **Compilation Success**: All generated APIs compile without errors

### Research Completed
- **Runtime Architecture Analysis**: Identified where tag resolution happens
- **Condition System Study**: Existing Perl expression evaluation infrastructure
- **Parser Research**: PPI/Pest options for parsing Perl expressions to Rust types
- **Fallback Strategy**: Manual implementations for complex conditions

### Critical Discovery
**The generated conditional code stores raw Perl condition strings that can't be evaluated in Rust:**
```rust
condition: "$count == 582",
condition: "$$self{Model} =~ /EOS D30\\b/",
condition: "$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/",
```

## Remaining Tasks

### Phase 1: Fix Condition Generation (High Confidence)
**Problem**: Generator outputs unparseable Perl strings  
**Solution**: Use Pest grammar parser for simple expressions + manual implementations for complex ones

1. **Add Pest dependency** to `codegen/Cargo.toml`
2. **Create simple grammar** for common condition patterns:
   ```pest
   condition = { term ~ (logical ~ term)* }
   term = { variable ~ operator ~ value | "(" ~ condition ~ ")" }
   variable = { "$count" | "$format" | "$$self{Model}" }
   operator = { "==" | "eq" | "=~" | "!~" }
   logical = { "and" | "or" }
   ```
3. **Modify conditional_tags.rs generator** to:
   - Parse simple conditions with Pest at codegen time
   - Generate proper Rust condition enums
   - Create fallback for complex conditions

**Expected Coverage**: 80% of conditions (simple count/format/model checks)

### Phase 2: Manual Implementation Registry (Medium Confidence - Requires Design)
**Problem**: Complex Perl conditions can't be auto-parsed  
**Solution**: Hand-written Rust implementations for complex patterns

1. **Create manual implementation modules**:
   ```rust
   // src/implementations/canon/conditional_manual.rs
   pub fn complex_vignetting_check(ctx: &ConditionalContext) -> bool {
       // Hand-implement: $$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/
   }
   ```
2. **Registry system** mapping condition names to manual functions
3. **Generator integration** to reference manual implementations in generated code

### Phase 3: Runtime Integration (Requires Research)
**Problem**: Generated APIs exist but are never called during EXIF parsing  
**Solution**: Wire conditional resolution into tag parsing pipeline

**Research Required**: 
- Find where tag IDs are resolved to tag names in current parsing
- Determine where model/count/format/binary_data context is available
- Identify exact integration points for conditional resolution

**Implementation**: Add conditional tag resolution calls in parsing pipeline

### Phase 4: Testing & Validation (High Confidence)
1. **Unit tests** for Pest grammar parser
2. **Integration tests** with Canon EXIF files containing ColorData tags
3. **Compatibility tests** ensuring ExifTool behavior match
4. **Performance validation** measuring conditional resolution overhead

## Prerequisites

### Dependencies
- Add `pest` and `pest_derive` to `codegen/Cargo.toml`
- Ensure `PPI` Perl module available for alternative parsing approach (if needed)

### Test Assets
- Canon EXIF files with various ColorData counts (582, 653, 796, etc.)
- Images from different Canon models (EOS 5D, EOS R5, etc.)
- Files with VignettingCorr binary patterns

## Testing Strategy

### Primary Test Case: Canon ColorData
**Tag ID 16385** with count-based conditions:
- Count 582 → `ColorData1`
- Count 653 → `ColorData2` 
- Count 796 → `ColorData3`
- Count 692/674/702/1227/etc. → `ColorData4`

**Validation**:
```rust
let context = ConditionalContext {
    model: Some("Canon EOS 5D".to_string()),
    count: Some(582),
    format: Some("int32u".to_string()),
    binary_data: Some(raw_bytes),
};
let resolved = conditional_tags.resolve_tag("16385", &context);
assert_eq!(resolved.unwrap().name, "ColorData1");
```

### Integration Testing
1. **End-to-end parsing**: `cargo run -- canon_colordata.jpg --show-tags`
2. **Compatibility verification**: Compare resolved tag names with ExifTool output
3. **Performance testing**: Measure conditional resolution overhead

### Unit Testing
1. **Grammar parser**: Test Pest parsing of simple conditions
2. **Manual implementations**: Test complex condition functions
3. **Context creation**: Test building ConditionalContext from parsing state

## Success Criteria & Quality Gates

### Definition of Done
- [ ] Canon ColorData tags resolve correctly based on count (primary success metric)
- [ ] Model-based conditions work for Canon EOS variants
- [ ] Binary pattern conditions work for VignettingCorr tags
- [ ] Graceful fallback for unimplemented conditions
- [ ] Performance impact <10% on EXIF parsing
- [ ] All tests pass, including compatibility tests

### Quality Gates
- **Code Review**: Architecture review with focus on condition evaluation safety
- **Integration Testing**: End-to-end tests with real Canon files
- **Performance Review**: Benchmark conditional resolution impact
- **Documentation**: Update MILESTONE-17 status and create handoff docs

## Gotchas & Tribal Knowledge

### Condition Complexity Spectrum
**Simple (auto-parseable)**:
- `$count == 582` - Direct count check
- `$format eq "int32u"` - Format string comparison
- `$$self{Model} =~ /EOS/` - Basic model regex

**Complex (manual implementation needed)**:
- `$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/` - Binary pattern logic
- `$format eq "int32u" and ($count == 156 or $count == 162 or $count == 167...)` - Multi-line conditions
- `($$self{CameraInfoCount} = $count) and $$self{Model} =~ /\\b1DS?\$/` - Assignment + condition

### Fallback Strategy
**For unparseable conditions**:
1. **Development**: Generate compile warnings pointing to missing manual implementations
2. **Runtime**: Log detailed warnings + use conservative fallback (AlwaysFalse)
3. **Monitoring**: Track which unimplemented conditions are hit in real usage

### Integration Architecture
**Existing condition infrastructure is battle-tested** - reuse rather than rebuild:
- `src/processor_registry/conditions/parser.rs` already parses many Perl patterns
- `src/processor_registry/conditions/mod.rs` provides runtime evaluation
- Follow patterns in `src/implementations/canon/af_info.rs` for condition enums

### Trust ExifTool Principle
**Critical**: Never "optimize" or "improve" ExifTool's condition logic. If ExifTool has seemingly redundant conditions, implement them exactly. The conditions exist to handle specific camera firmware quirks discovered over 25 years.

## Future Work: Universal Perl Expression System

### Vision
Expand the hybrid grammar + manual implementation approach to handle **all Perl expressions** in ExifTool:

**Additional Expression Types**:
- **ValueConv**: `'$val * 100'`, `'2**($val / 16)'`, `'$val / 256'`
- **PrintConv**: `'"$val mm"'`, `'sprintf("%.1f",$val)'`, `'lc $val'`
- **Function Calls**: `'Image::ExifTool::Exif::PrintExposureTime($val)'`
- **Array/Hash Access**: `'$val[0] * $val[1]'`, `'$$self{TIFF_TYPE}'`

### Expanded Grammar Requirements
```pest
expression = { assignment | conditional | arithmetic | function_call | string_format }
arithmetic = { term ~ (operator ~ term)* }
function_call = { module ~ "::" ~ function ~ "(" ~ args ~ ")" }
string_format = { "sprintf" ~ "(" ~ format ~ "," ~ expression ~ ")" }
array_access = { variable ~ "[" ~ index ~ "]" }
hash_access = { "$$self{" ~ key ~ "}" }
```

### Strategic Impact
- **Eliminate months of manual porting** for ValueConv/PrintConv implementations
- **Universal infrastructure** reused across all extractors (ValueConv, PrintConv, Conditional, etc.)
- **Automatic ExifTool updates** - regenerate expression handlers with each ExifTool release

### Future Milestones
1. **ValueConv Expression Parser** - Handle mathematical expressions and simple conversions
2. **PrintConv Expression Parser** - Handle string formatting and display conversions  
3. **Function Call Registry** - Manual implementations for ExifTool module functions
4. **Advanced Pattern Matching** - Complex array/hash access and string operations

**Note**: The conditional tags work establishes the architectural foundation and proves the hybrid approach. Future expression parsers will reuse the same grammar parser + manual implementation registry pattern.