# P15: Conditional Tag Code Generation Implementation

## Project Overview

- **Goal**: Enable conditional tag resolution by implementing ExifTool's conditional tag extraction in the codegen system
- **Problem**: 50+ ExifTool modules use conditional tags (context-dependent tag resolution) but our codegen doesn't extract them, leaving conditional tag infrastructure disabled
- **Constraints**: Must follow Trust ExifTool principle - exact translation of ExifTool's conditional logic, no manual implementations

## Context & Foundation

### System Overview

- **Conditional Tags**: ExifTool's mechanism where single tag IDs map to different tag definitions based on runtime context (data count, camera model, binary patterns). Example: Canon tag 0x4001 → ColorData1 (count=582) vs ColorData4 (count=692)
- **ExifTool Implementation**: Uses Perl conditional arrays in module definitions with complex expression evaluation for tag resolution
- **Codegen System**: Current codegen processes static tag tables but lacks strategy for conditional tag arrays (`@cond` tables in ExifTool modules)
- **Expression System**: Complete Perl-compatible expression evaluator exists (`src/expressions/`) and works correctly
- **Integration Infrastructure**: IFD processing and tag resolution logic exists but commented out pending code generation

### Key Concepts & Domain Knowledge

- **Conditional Context**: Runtime information needed for resolution - manufacturer, model, data count, format, binary patterns
- **Expression Evaluation**: Perl conditions like `$count == 582` or `$$self{Model} =~ /EOS/` translated to Rust
- **Tag Resolution Pipeline**: Standard tags → IFD processing → conditional tag resolution → final tag assignment
- **Module-Specific Resolvers**: Each manufacturer (Canon, Sony, Nikon) needs separate conditional tag resolver with specific logic

### Surprising Context

- **Canon Complexity**: Canon.pm has 200+ conditional tags, the most complex conditional logic in ExifTool
- **Context Dependency**: Same tag ID can resolve to completely different tag names/meanings based on camera model or data characteristics  
- **Expression Scope**: Conditional expressions access both tag values (`$count`) AND processor state (`$$self{Model}`) requiring dual evaluation contexts
- **Infrastructure Ready**: All integration points exist and work - only missing the generated resolver modules
- **Test Coverage**: Complete test suite exists (`tests/conditional_tag_resolution_tests.rs`) expecting `CanonConditionalTags` and `ConditionalContext` structures

### Foundation Documents

- **Trust ExifTool**: [docs/TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core principle requiring exact ExifTool translation
- **Codegen Architecture**: [docs/CODEGEN.md](../CODEGEN.md) - Strategy pattern for processing ExifTool structures
- **Expression System**: `src/expressions/mod.rs` - Working Perl expression evaluator
- **ExifTool Reference**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm:2847-3200` - Canon conditional tag arrays
- **Start Here**: `codegen/src/strategies/mod.rs` - Where to add `ConditionalTagStrategy`

### Prerequisites

- **Knowledge assumed**: Familiarity with codegen strategy pattern, Perl regex basics, ExifTool module structure
- **Setup required**: Standard development environment, `third-party/exiftool/` submodule available

## TDD Foundation Requirement

### Task 0: Integration Test

**Purpose**: Verify conditional tag resolution works end-to-end with real ExifTool data patterns.

**Success Criteria**:

- [ ] **Test exists**: `tests/integration_p15_conditional_tags.rs:test_canon_conditional_resolution`
- [ ] **Test fails**: `cargo t test_canon_conditional_resolution` fails with "CanonConditionalTags not found" 
- [ ] **Integration focus**: Tests full Canon ColorData resolution pipeline with count-based switching
- [ ] **TPP reference**: Test includes comment `// P15: Conditional Tag Codegen - see docs/todo/P15-conditional-tag-codegen-implementation.md`
- [ ] **Measurable outcome**: Test shows Canon tag 0x4001 correctly resolves to ColorData1 vs ColorData4 based on count context

**Requirements**:
- Must test actual Canon conditional tag resolution using count values from real camera files
- Should fail specifically because `CanonConditionalTags` struct doesn't exist yet
- Must demonstrate context-dependent tag resolution working when implementation completes

## Remaining Tasks

### Task A: Extract Canon Conditional Tag Arrays from ExifTool

**Success Criteria**:

- [ ] **Implementation**: ConditionalTagStrategy processes Canon.pm → `codegen/src/strategies/conditional_tag.rs:45-120` extracts `@cond` arrays
- [ ] **Integration**: Strategy registered → `codegen/src/strategies/mod.rs:23` includes `ConditionalTagStrategy` 
- [ ] **Generation**: Canon conditionals extracted → `make codegen` generates `src/generated/canon/main_conditional_tags.rs`
- [ ] **Unit tests**: `cargo t test_conditional_tag_extraction` passes
- [ ] **Manual validation**: `grep -r "ColorData1\|ColorData4" src/generated/canon/` shows conditional tag definitions
- [ ] **Cleanup**: N/A
- [ ] **Documentation**: N/A

**Implementation Details**: Parse Canon.pm conditional arrays, extract tag conditions and mappings, generate Rust conditional resolver structure  
**Integration Strategy**: Add to codegen pipeline as standard strategy, auto-generates during `make codegen`  
**Validation Plan**: Compare generated conditional mappings against Canon.pm source arrays  
**Dependencies**: None

**Success Patterns**:
- ✅ All Canon conditional tags from ExifTool extracted with exact condition logic preserved
- ✅ Generated resolver handles count-based, model-based, and binary pattern conditions
- ✅ `CanonConditionalTags::resolve_tag()` method exists and compiles

### Task B: Implement Conditional Context Generation

**Success Criteria**:

- [ ] **Implementation**: ConditionalContext structure generated → `src/generated/canon/main_conditional_tags.rs:15-35` defines context struct
- [ ] **Integration**: Context passed through resolution pipeline → `src/exif/ifd.rs:895` builds and uses conditional context  
- [ ] **Task 0 passes**: `cargo t test_canon_conditional_resolution` now succeeds
- [ ] **Unit tests**: `cargo t test_conditional_context_building` passes
- [ ] **Manual validation**: `cargo run -- test-images/canon/colordata.jpg` resolves to ColorData1 or ColorData4 based on data count
- [ ] **Cleanup**: Remove commented TODO lines → `grep -r "TODO.*conditional.*tag" src/` returns empty for implemented areas
- [ ] **Documentation**: N/A

**Implementation Details**: Generate ConditionalContext with make, model, count, format, binary_data fields matching ExifTool's resolution context  
**Integration Strategy**: Wire into IFD processing pipeline, build context from current parsing state  
**Validation Plan**: Test context building and resolution with real Canon camera files  
**Dependencies**: Task A complete

**Success Patterns**:
- ✅ Context correctly captures all information needed for ExifTool conditional expressions
- ✅ Resolution works with same logic as ExifTool's conditional tag evaluation
- ✅ Real camera files resolve conditional tags correctly

### Task C: Enable Conditional Tag Tests and Integration

**Success Criteria**:

- [ ] **Implementation**: Tests uncommented and working → `tests/conditional_tag_resolution_tests.rs:12-15` imports working structures
- [ ] **Integration**: All conditional tag infrastructure active → `src/exif/ifd.rs:874-902` functions uncommented and working
- [ ] **Task 0 passes**: `cargo t test_integration_p15_conditional_tags` succeeds  
- [ ] **Unit tests**: `cargo t conditional_tag` passes (all conditional tag tests)
- [ ] **Manual validation**: `./scripts/compare-with-exiftool.sh test-images/canon/eos5d.jpg` shows matching ColorData resolution
- [ ] **Cleanup**: All conditional tag TODOs resolved → `grep -r "TODO.*Re-enable.*conditional" src/` returns empty
- [ ] **Documentation**: N/A

**Implementation Details**: Uncomment and activate all conditional tag infrastructure, verify end-to-end functionality  
**Integration Strategy**: Enable conditional resolution in production IFD processing pipeline  
**Validation Plan**: Full comparison testing against ExifTool output for conditional tags  
**Dependencies**: Task A and Task B complete

**Success Patterns**:
- ✅ All existing conditional tag tests pass without modification
- ✅ Integration with ExifTool comparison shows exact match for conditional tag resolution
- ✅ No remaining TODO comments about conditional tag generation

### Task D: RESEARCH - Extend to Sony and Nikon Conditional Tags

**Objective**: Determine feasibility and approach for implementing conditional tags for other manufacturers
**Success Criteria**: Research document `docs/research/multi-manufacturer-conditional-tags.md` with implementation strategy for Sony/Nikon  
**Done When**: Document exists with specific conditional tag counts and complexity analysis for top 3-5 manufacturers

## Integration Requirements

### Mandatory Integration Proof

- [ ] **Activation**: Conditional tags resolved automatically → Canon images show ColorData1/ColorData4 in normal processing
- [ ] **Consumption**: IFD processing uses conditional resolution → `grep -r "resolve_tag" src/exif/` shows production usage  
- [ ] **Measurement**: ExifTool output matches → `cargo run image.jpg | jq '.["Canon:ColorData1"]'` shows resolved conditional tags
- [ ] **Cleanup**: Test infrastructure no longer commented → All conditional tag tests run in CI

### Integration Verification Commands

**Production Usage Proof**:
- `grep -r "CanonConditionalTags\|ConditionalContext" src/generated/` → Should show generated resolver structures
- `grep -r "resolve_tag" src/exif/` → Should show conditional tag resolution in IFD processing
- `cargo run -- test-images/canon/eos5d.jpg | jq '.["Canon:ColorData1"]'` → Should show resolved conditional tag data

## Working Definition of "Complete"

A conditional tag system is complete when:

- ✅ **Canon camera files** resolve ColorData1/ColorData4 based on data count automatically
- ✅ **ExifTool compatibility** maintained - same conditional tag resolution logic and results
- ✅ **Codegen integrated** - conditional tags regenerate with `make codegen` from ExifTool updates  
- ✅ **Infrastructure active** - no commented-out conditional tag code remains

## Prerequisites

None - this is foundational infrastructure

## Testing

- **Unit**: Test conditional tag extraction from Canon.pm, context building, and tag resolution logic
- **Integration**: Verify end-to-end conditional tag resolution with real Canon camera files  
- **Manual check**: Run `cargo run -- canon_image.jpg` and confirm ColorData tags resolve correctly

## Definition of Done

- [ ] `cargo t conditional_tag` passes (all conditional tag tests)
- [ ] `make precommit` clean  
- [ ] Canon conditional tag resolution matches ExifTool exactly for ColorData1/ColorData4 cases
- [ ] No remaining TODO comments about conditional tag generation

## Implementation Guidance

### Recommended Patterns
- **Strategy Pattern**: Follow existing codegen strategy structure for consistency
- **ExifTool Parsing**: Use proven regex patterns from existing strategies for Perl array extraction
- **Context Building**: Mirror ExifTool's conditional evaluation context exactly

### Tools to Leverage  
- **Expression System**: `src/expressions/` provides working Perl expression evaluation
- **Codegen Framework**: Existing strategy pattern handles ExifTool processing consistently
- **Test Infrastructure**: Complete conditional tag test suite exists

### ExifTool Translation Notes
- **Conditional Arrays**: ExifTool `@cond` arrays map directly to Rust resolver match statements
- **Context Variables**: `$count`, `$$self{Model}` patterns need context struct field mapping
- **Expression Evaluation**: Preserve exact ExifTool conditional logic without "optimization"

## Additional Gotchas & Tribal Knowledge

- **Generated file location** → Canon conditionals go in `src/generated/canon/main_conditional_tags.rs`, not root canon module
- **Expression complexity** → Canon has most complex conditional logic, Sony/Nikon simpler - start with Canon to handle hardest case
- **Context scope** → Conditional context needs both tag values AND processor state for full ExifTool compatibility
- **Test expectations** → Existing tests are comprehensive and expect exact ExifTool-compatible API structure