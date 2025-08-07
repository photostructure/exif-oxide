# Technical Project Plan: P07 Unified Codegen Completion

## Project Overview

- **Goal**: Complete P07 universal extraction system by fixing critical import mismatches, missing strategy handlers for mixed-key lookup tables (canonLensTypes), and consolidating all P07* sub-projects into a working system
- **Problem**: 226 compilation errors prevent functionality. Generated code uses module-prefixed names but imports expect generic names. Critical lookup tables (canonLensTypes with 526+ entries) generate empty due to wrong strategy selection.
- **Constraints**: Must fix codegen strategies (not manually edit generated files), maintain ≥76/167 compatibility baseline, complete all work through codegen system

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **DO NOT DIRECTLY EDIT ANYTHING IN `src/generated/**/*.rs`** (Read [CODEGEN.md](CODEGEN.md) -- fix the generator or strategy in codegen/src instead!)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **Universal extraction system**: Field extractor introspects ExifTool Perl modules, strategies process symbols, generates 591+ Rust files. Currently generates correct structure but with naming mismatches.
- **Strategy dispatch failure**: `%canonLensTypes` (526 entries with mixed numeric/string keys like `2.1`, `4.1`) incorrectly processed by TagKitStrategy instead of SimpleTableStrategy, resulting in empty generated files
- **Import architecture mismatch**: Codegen produces module-prefixed constants (`CANON_MAIN_TAGS`) but mod.rs files export generic `MAIN_TAGS`, causing 226 compilation errors

### Key Concepts & Domain Knowledge

- **Mixed-key lookup tables**: ExifTool uses hashes with both numeric (`1`, `2`) and decimal string keys (`2.1`, `4.1`) for manufacturer lookups. Current SimpleTableStrategy assumes pure numeric keys.
- **Module-prefixed constants**: Universal extraction correctly generates `CANON_MAIN_TAGS`, `EXIF_MAIN_TAGS` to avoid naming collisions, but import system expects generic names
- **Strategy priority**: First-match-wins in `all_strategies()` - wrong order causes SimpleTableStrategy to miss mixed-key tables before TagKitStrategy claims them

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **canonLensTypes is NOT a tag table**: Despite having 526 entries and being in Canon.pm, it's a simple lookup hash for lens identification, not a tag definition table
- **Decimal keys are intentional**: Keys like `2.1`, `4.1` differentiate similar lenses (e.g., `2` = "Canon EF 28mm" vs `2.1` = "Sigma 24mm")
- **Empty generated files are wrong strategy**: When you see `tags` HashMap with no insertions, it means wrong strategy processed the symbol
- **mod.rs files are generated**: The MAIN_TAGS export mismatches are in generated mod.rs files - must fix the generator, not the files
- **User already fixed some imports**: src/generated/mod.rs, file_types/mod.rs, and some main_tags.rs files were modified - incorporate these changes

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Universal extraction architecture, strategy system
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm:97` - canonLensTypes definition with mixed keys
- **Start here**: `codegen/src/strategies/simple_table.rs` - needs mixed-key support, `src/generated/strategy_selection.log` - shows wrong strategy selection

### Prerequisites

- **Knowledge assumed**: Rust HashMap with &str keys, codegen strategy pattern, ExifTool hash structures
- **Setup required**: Working codegen environment, ability to regenerate all code

**Context Quality Check**: Can a new engineer understand WHY canonLensTypes generates empty and how to fix it?

**Answer**: Yes - TagKitStrategy incorrectly claims it because SimpleTableStrategy only handles numeric keys, not mixed numeric/string keys like canonLensTypes requires.

## Work Completed

- ✅ **Problem diagnosis** → Identified canonLensTypes empty due to wrong strategy selection
- ✅ **Import mismatch identified** → Module-prefixed constants vs generic exports documented
- ✅ **User interventions noted** → Some files already modified to add missing modules

## TDD Foundation Requirement

### Task 0: Integration Test

**Purpose**: Ensure canonLensTypes lookup works and compilation succeeds

**Success Criteria**:
- [ ] **Test exists**: `tests/integration_p07_unified_completion.rs:test_canon_lens_lookup`
- [ ] **Test fails**: `cargo t test_canon_lens_lookup` fails showing empty canonLensTypes
- [ ] **Integration focus**: Tests both lens lookup functionality AND successful compilation
- [ ] **TPP reference**: Test includes comment `// P07: Unified completion - see docs/todo/P07-unified-codegen-completion.md`
- [ ] **Measurable outcome**: Lens ID 2.1 returns "Sigma 24mm f/2.8 Super Wide II"

## Remaining Tasks

### Dependency Structure for Parallelization

```
Phase 0: Codegen Strategy Fix [BLOCKING]
    ↓
Phase 1: Three Parallel Branches
    - Branch A: Module declarations
    - Branch B: MAIN_TAGS exports  
    - Branch C: Source imports
    ↓
Phase 2: Three Parallel Branches
    - Branch D: Type conversions
    - Branch E: TagValue::Empty
    - Branch F: Function refs
    ↓
Phase 3: Validation [Sequential]
```

### Task A: Fix SimpleTableStrategy for Mixed Keys [PHASE 0 - BLOCKING]

**Success Criteria**:
- [ ] **Implementation**: Mixed-key support → `codegen/src/strategies/simple_table.rs:150-200` handles string keys
- [ ] **Integration**: Strategy claims canonLensTypes → `strategy_selection.log` shows SimpleTableStrategy for canonLensTypes
- [ ] **Task 0 passes**: `cargo t test_canon_lens_lookup` succeeds
- [ ] **Unit tests**: `cd codegen && cargo t test_mixed_key_simple_table`
- [ ] **Manual validation**: `grep -c "insert" src/generated/canon/canonlenstypes_tags.rs` returns 526+
- [ ] **Cleanup**: N/A
- [ ] **Documentation**: N/A

**Implementation Details**: 
- Change SimpleTableStrategy to use `HashMap<String, String>` instead of assuming numeric keys
- Generate lookups with `pub fn lookup_canon_lens_types(key: &str) -> Option<&'static str>`
- Handle both "1" and "2.1" style keys

**Integration Strategy**: Update can_handle() to detect mixed-key patterns
**Validation Plan**: Verify all manufacturer lens/camera type tables populate
**Dependencies**: None - blocking task

### Task B: Regenerate All Code [PHASE 0 - BLOCKING]

**Success Criteria**:
- [ ] **Implementation**: Regeneration complete → `cd codegen && cargo run --release` succeeds
- [ ] **Integration**: New files generated → `ls -la src/generated/ | grep -c "\.rs"` shows 591+ files
- [ ] **Manual validation**: canonLensTypes populated → check file has entries
- [ ] **Cleanup**: Old strategy selection log removed

**Dependencies**: Task A complete

### Task C: Add Missing Module Declarations [PHASE 1A - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: Modules declared → `src/generated/mod.rs:43-44` adds composite_tags and file_types
- [ ] **Integration**: Imports resolve → `cargo check 2>&1 | grep -c "could not find.*in.*generated"` reduces
- [ ] **Manual validation**: `grep "pub mod composite_tags" src/generated/mod.rs` returns match

**Dependencies**: Task B complete

### Task D: Fix MAIN_TAGS Exports [PHASE 1B - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: Exports fixed → All 40+ module mod.rs files export correct constant
- [ ] **Integration**: No more MAIN_TAGS errors → `cargo check 2>&1 | grep -c "no.*MAIN_TAGS"` returns 0
- [ ] **Manual validation**: `grep "pub use main_tags::CANON_MAIN_TAGS" src/generated/canon/mod.rs` succeeds

**Implementation Details**: Use rg|sd pattern for systematic replacement
**Dependencies**: Task B complete

### Task E: Update Source Imports [PHASE 1C - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: Imports updated → ~20 source files use module-specific constants
- [ ] **Integration**: Code compiles → Import errors eliminated
- [ ] **Manual validation**: No more tag_kit:: references

**Dependencies**: Task B complete

### Task F: Fix Type Conversions [PHASE 2D - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: `.into()` added → u16→u32 conversions in place
- [ ] **Integration**: Type errors gone → `cargo check 2>&1 | grep -c "expected.*u32.*found.*u16"` returns 0

**Dependencies**: Phase 1 complete

### Task G: Add TagValue::Empty Match Arms [PHASE 2E - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: Match arms added → All TagValue matches handle Empty variant
- [ ] **Integration**: Pattern errors gone → No "non-exhaustive patterns" errors

**Dependencies**: Phase 1 complete

### Task H: Fix Missing Function References [PHASE 2F - PARALLEL]

**Success Criteria**:
- [ ] **Implementation**: Functions found/removed → All lookup functions resolve or removed
- [ ] **Integration**: No unresolved functions → Function import errors eliminated

**Dependencies**: Phase 1 complete

### Task I: Final Validation [PHASE 3 - SEQUENTIAL]

**Success Criteria**:
- [ ] **Implementation**: Full compilation → `cargo check` returns 0 errors
- [ ] **Integration**: Compatibility maintained → `make compat` shows ≥76/167
- [ ] **Task 0 passes**: Integration test succeeds
- [ ] **Manual validation**: `cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.jpg` works
- [ ] **Documentation**: Update all P07* TPPs with completion status

**Dependencies**: All Phase 2 tasks complete

## Task Completion Standards

**RULE**: No checkbox can be marked complete without specific proof.

### Required Evidence Types

- **Code references**: `file.rs:line_range` where implementation exists
- **Passing commands**: `cargo check` succeeds, `cargo t test_name` passes
- **Integration proof**: Compilation succeeds, imports resolve correctly
- **Generated code validation**: Line counts, grep results showing content

### Success Patterns

**Task A Success**:
- ✅ `grep "HashMap<String, String>" codegen/src/strategies/simple_table.rs` shows mixed-key support
- ✅ `src/generated/canon/canonlenstypes_tags.rs` has 526+ insert statements
- ✅ Lens lookup for "2.1" returns Sigma lens description

**Task D Success**:
- ✅ No "no MAIN_TAGS in" compilation errors
- ✅ Each module exports its prefixed constant (CANON_MAIN_TAGS, EXIF_MAIN_TAGS, etc.)

## Definition of Done

- [ ] `cargo check` → 0 compilation errors (from 226)
- [ ] `make compat` → maintains ≥76/167 compatibility baseline
- [ ] canonLensTypes and similar tables fully populated
- [ ] All P07* sub-projects consolidated and marked complete
- [ ] Universal extraction is default and working system

## Current Status & Handoff Context

**Critical Issue Identified**: canonLensTypes generates empty because TagKitStrategy incorrectly processes it. Must fix SimpleTableStrategy to handle mixed numeric/string keys first.

**Parallelization Opportunity**: After Phase 0 (blocking), Phase 1 branches A/B/C can run simultaneously, then Phase 2 branches D/E/F can run simultaneously.

**Files Already Modified**: User has modified some generated files to add missing declarations - these changes should be incorporated into the fix.

**Next Engineer Priority**:
1. **MUST DO FIRST**: Fix SimpleTableStrategy for mixed keys (Task A)
2. **Then regenerate**: Run codegen to create new files (Task B)
3. **Then parallelize**: Run Phase 1 branches simultaneously

## Future Work & Refactoring Ideas

### After Completion
- **Strategy detection improvements**: Better heuristics for lookup table vs tag table detection
- **Mixed-key optimization**: Consider separate numeric vs string lookup paths for performance
- **Validation suite**: Automated checks that all known ExifTool tables generate non-empty