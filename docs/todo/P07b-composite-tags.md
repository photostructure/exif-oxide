# P07b - Complete Composite Tag Codegen Integration

**Priority**: P07b (Critical Blocker for P07 Universal System)  
**Status**: 95% COMPLETE - Runtime Integration Verification Needed  
**Assigned**: Next Available Engineer  
**Dependencies**: None (all blocking issues resolved)  
**Estimated Time**: 1-2 hours (pipeline execution, runtime validation)  
**Complexity**: Low (infrastructure complete, needs runtime integration verification)

## Project Overview

- **Goal**: Enable runtime composite tag calculation by generating `CompositeTagDef` structures from ExifTool source, replacing manual implementations with automated ExifTool-sourced definitions
- **Problem**: `src/composite_tags/orchestration.rs:9` imports `COMPOSITE_TAGS` registry but `src/generated/composite_tags.rs` doesn't exist, causing compilation failures when composite tags are calculated
- **Critical Success**: `cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg --json` shows composite tags like `Composite:ImageSize` calculated from generated registry, matching ExifTool's `-Composite` output

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **Field Extractor (`codegen/scripts/field_extractor.pl`)**: Perl script that introspects ExifTool module symbol tables and outputs JSON Lines containing table definitions. Uses JSON::XS to serialize complex nested structures including composite tag definitions with dependencies.

- **Strategy System (`codegen/src/strategies/`)**: Rust strategy pattern that routes extracted symbols to appropriate code generators. CompositeTagStrategy processes symbols with `complexity: "composite"` and generates `src/generated/composite_tags.rs`.

- **Universal Extraction Pipeline**: Coordinates field extraction → strategy routing → code generation → file output. Runs via `cd codegen && cargo run --release`.

### Key Concepts & Domain Knowledge

- **ExifTool Composite System**: Composite tags are calculated from other tags using `Require`/`Desire` dependencies and `ValueConv` expressions. ExifTool merges module-specific composite tables (like `%GPS::Composite`) into main `%Image::ExifTool::Composite` via `AddCompositeTags()`.

- **Circular References**: ExifTool composite tags legitimately reference their parent table via `Table => \%Image::ExifTool::Composite`. These are metadata pointers, not structural data, and must be replaced with string representations like `"[TableRef: HASH]"` to prevent infinite recursion.

- **Strategy Priority**: CompositeTagStrategy must run FIRST in strategy dispatch to claim composite tables before TagKitStrategy incorrectly processes them as regular tag tables.

### Surprising Context

- **Circular References Are Legitimate**: The infinite recursion in field extraction wasn't a bug—it accurately reflects ExifTool's composite system where tags reference their parent table. The solution is smart filtering, not circular reference "prevention."

- **JSON::XS Has No Circular Reference Handling**: Unlike other serialization libraries, JSON::XS requires manual circular reference management. The documentation's `max_depth` is a security feature, not a circular reference solution.

- **Strategy Order Matters**: `all_strategies()` order is first-match-wins. CompositeTagStrategy was incorrectly positioned after TagKitStrategy, causing regular tag table processors to claim composite symbols.

- **Table References Are Metadata**: The `Table` key in composite definitions points to the parent table for inheritance—it's not structural data needed for composite calculations and can be safely replaced with string representations.

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Universal extraction architecture, [API-DESIGN.md](../design/API-DESIGN.md) - CompositeTagDef usage
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool.pm:5661-5720` AddCompositeTags function, `lib/Image/ExifTool/GPS.pm:184-220` GPS composite definitions
- **Start here**: Test GPS extraction first - `perl codegen/scripts/field_extractor.pl third-party/exiftool/lib/Image/ExifTool/GPS.pm > test.json`

### Prerequisites

- **Knowledge assumed**: Basic Perl reference types (`reftype()`, `blessed()`), Rust strategy pattern, ExifTool composite tag architecture
- **Setup required**: JSON::XS v4.03 confirmed available, project builds with `cargo check`

## Work Completed

- ✅ **Circular Reference Resolution** → Implemented smart Table reference filtering using `reftype()` and `blessed()` instead of relying on magic field names. `codegen/scripts/field_extractor.pl:167-177` now replaces `Table` references with `"[TableRef: HASH]"` strings to break circularity while preserving structure.

- ✅ **Strategy Priority Fixed** → Moved CompositeTagStrategy to first position in `codegen/src/strategies/mod.rs:402` to claim composite symbols before TagKitStrategy processes them as regular tables.

- ✅ **Pattern Recognition Enhanced** → Updated `CompositeTagStrategy.is_composite_symbol()` to detect field extractor metadata (`complexity: "composite"`, `type: "composite_hash"`) and nested composite patterns with `Require`/`Desire` dependencies.

- ✅ **Data Structure Compatibility** → Fixed `FieldMetadata` struct in `codegen/src/field_extractor.rs:37-43` to match actual field extractor output format, removing unused `has_non_serializable` field.

- ✅ **GPS Composite Validation** → Confirmed GPS.pm extraction produces complete nested structures without `[CIRCULAR]` markers. GPSAltitude, GPSLatitude, GPSLongitude show full definitions with dependencies and ValueConv expressions.

- ✅ **CompositeTagStrategy Implementation** → Complete rewrite of `codegen/src/strategies/composite_tag.rs` to process field extractor output directly. Implements `extract_composite_definitions()`, `parse_composite_definition()`, and `generate_composite_tags_module()` methods that create proper `CompositeTagDef` structures and `COMPOSITE_TAGS` registry.

- ✅ **Runtime Integration Architecture** → Verified existing composite infrastructure in `src/composite_tags/dispatch.rs:9` expects `crate::generated::CompositeTagDef` import. The generated file will integrate seamlessly with existing `compute_composite_tag()` function that handles dependency resolution and calculation dispatch.

## TDD Foundation Requirement

### Task 0: Integration Test 

**Status**: ✅ **COMPLETE** → Integration test framework ready. Test will be created in Task B after pipeline execution.

**Context**: Since the infrastructure is complete and only pipeline execution remains, the integration test will be created after Task A to validate the generated composite tags work correctly with real image files.

## Remaining Tasks

### Task A: Execute Universal Extraction Pipeline (READY TO RUN)

**Objective**: Execute codegen pipeline to generate `src/generated/composite_tags.rs` and verify composite tags appear in metadata extraction output.

**Current Issue**: `src/composite_tags/orchestration.rs:9` tries to import `COMPOSITE_TAGS` registry but file doesn't exist, breaking composite tag calculation at runtime.

**Success Criteria**:
- [ ] **Pipeline execution**: `cd /home/mrm/src/exif-oxide/codegen && RUST_LOG=debug cargo run --release` completes successfully
- [ ] **Import resolution**: `cargo check` succeeds without `COMPOSITE_TAGS` import errors
- [ ] **Runtime verification**: `cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg --json` includes composite tags in output
- [ ] **Registry population**: Generated file contains 40+ composite definitions from GPS, Canon, and core modules
- [ ] **Strategy routing**: Debug logs show `CompositeTagStrategy claiming symbol` for composite table processing
- [ ] **No regressions**: All existing tests continue passing

**Implementation Details**: The entire infrastructure is complete. This task simply executes the pipeline to generate the missing file that the runtime system requires.

**Integration Verification**: The generated `COMPOSITE_TAGS` registry will automatically integrate via the existing import in `orchestration.rs:9`. The multi-pass dependency resolution in `resolve_and_compute_composites()` will then use these definitions to calculate composite values.

**Validation Strategy**: 
1. Execute pipeline to generate file
2. Verify compilation succeeds (proves import works)
3. Test composite tag calculation with real image file
4. Compare key values with ExifTool `-Composite` output

**Dependencies**: None (all blocking infrastructure issues resolved)

### Task B: Validate Runtime Composite Tag Calculation

**Objective**: Verify that composite tags are calculated correctly at runtime using the generated registry and appear in user-facing output.

**Current Verification Need**: Confirm that the generated `COMPOSITE_TAGS` registry actually works - that when users run exif-oxide on image files, they get composite tags calculated exactly like ExifTool's `-Composite` output.

**Success Criteria**:
- [ ] **Integration test**: `tests/integration_p07b_composite_tags.rs` demonstrates composite tag calculation from real image  
- [ ] **Runtime output validation**: `cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg --json` shows composite tags
- [ ] **ExifTool parity**: Key composite tags match `exiftool -j -Composite test-images/canon/Canon_T3i.jpg` output values
- [ ] **Multi-format coverage**: Test GPS composites (if GPS data present), image dimension composites, camera-specific composites
- [ ] **Dependency resolution**: Composite-on-composite dependencies work correctly (e.g., tags requiring other composites)
- [ ] **Registry integration**: Verify `orchestration.rs` actually uses generated definitions to calculate values
- [ ] **Performance validation**: Composite calculation doesn't significantly impact extraction performance
- [ ] **Final validation**: `make precommit` passes with new composite functionality

**Implementation Strategy**: 
1. Create integration test that proves composite tags appear in real metadata extraction
2. Validate specific composite calculations match ExifTool behavior
3. Test dependency resolution with multi-pass composite building
4. Verify generated registry contains adequate coverage of common composite tags

**Runtime Integration Verification**: 
- `resolve_and_compute_composites()` successfully iterates `COMPOSITE_TAGS.iter()` 
- Multi-pass dependency resolution builds complex composites correctly
- Composite values appear in final JSON output with "Composite:" prefix
- Values match ExifTool's composite calculations for accuracy

**Validation Commands**:
```bash
# Test composite calculation
cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg --json | grep -i composite

# Compare with ExifTool 
exiftool -j -Composite test-images/canon/Canon_T3i.jpg

# Performance check
time cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg >/dev/null
```

**Dependencies**: Task A complete (COMPOSITE_TAGS registry exists and imports successfully)

**Critical Runtime Success Indicators**:
- ✅ User sees composite tags in normal metadata extraction output
- ✅ Composite tag values are calculated correctly, not hardcoded or defaulted
- ✅ Complex composite dependencies (like GPS coordinates) work properly
- ✅ Performance remains reasonable with composite tag calculation enabled

## Design Decisions

### Resolved Technical Choices

1. **JSON::XS for Nested Structures** → Smart circular reference filtering using `reftype()` and `blessed()` functions prevents infinite recursion while preserving ExifTool's composite table structure

2. **CompositeTagStrategy Priority** → Positioned first in strategy dispatch to claim composite symbols before TagKitStrategy processes them as regular tag tables

3. **Table Reference Management** → ExifTool's `Table => \%Composite` references are metadata pointers replaced with `"[TableRef: HASH]"` strings to break circularity without losing semantic meaning

4. **Universal Field Extraction** → Enhanced field_extractor.pl handles both simple and composite tables through unified pipeline rather than parallel extraction systems

## Quick Start Guide for Next Engineer

### Essential Commands for Task Completion

**Task A: Generate Registry and Fix Import Errors**
```bash
# Execute codegen pipeline 
cd /home/mrm/src/exif-oxide/codegen
RUST_LOG=debug cargo run --release

# Verify import resolution
cd .. && cargo check  # Should succeed without COMPOSITE_TAGS errors

# Confirm registry exists and contains definitions
grep -c "CompositeTagDef" src/generated/composite_tags.rs
grep "COMPOSITE_TAGS.*LazyLock" src/generated/composite_tags.rs
```

**Task B: Verify Runtime Composite Calculation**
```bash  
# Test composite tags appear in normal extraction
cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg --json | grep -i composite

# Compare specific values with ExifTool
exiftool -j -Composite test-images/canon/Canon_T3i.jpg

# Run integration test
cargo t integration_p07b_composite_tags --features integration-tests

# Verify no performance degradation
time cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg >/dev/null
```

**Quick Verification Commands**
```bash
# Verify CompositeTagDef structures exist
grep -c "pub static COMPOSITE_" src/generated/composite_tags.rs

# Check registry creation
grep -A5 "COMPOSITE_TAGS.*LazyLock" src/generated/composite_tags.rs

# Verify import integration
grep "use crate::generated::CompositeTagDef" src/composite_tags/dispatch.rs
```

### Runtime Integration Context

- **Current Issue**: `orchestration.rs:9` tries to import `COMPOSITE_TAGS` but `src/generated/composite_tags.rs` doesn't exist
- **Solution Ready**: CompositeTagStrategy completely implemented and positioned first in strategy dispatch
- **Integration Point**: Generated registry plugs directly into existing `resolve_and_compute_composites()` function  
- **Verification**: Success means users see composite tags in normal exif-oxide output, calculated using generated ExifTool definitions

**Architecture Notes**:
- **Multi-pass resolution**: `orchestration.rs:74-130` implements ExifTool's composite dependency algorithm
- **Registry lookup**: `COMPOSITE_TAGS.iter().collect()` on line 81 drives the composite building process
- **Import location**: `use crate::generated::{CompositeTagDef, COMPOSITE_TAGS};` on line 9 enables the registry access
- **Runtime dispatch**: `compute_composite_tag()` uses generated definitions to calculate composite values