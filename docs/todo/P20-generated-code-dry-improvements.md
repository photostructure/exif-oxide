# Technical Project Plan: Generated Code DRY Improvements

## Project Overview

- **Goal**: Reduce generated code size by 40-50% through strategic use of macros and shared types while maintaining zero runtime overhead
- **Problem**: Generated code contains massive duplication - 42+ lookup tables with identical structure, 574+ PRINT_CONV HashMap declarations, resulting in the largest files being 14,390 lines. Total generated code is ~112,000 lines with significant redundancy.
- **Critical Constraints**:
  - ⚡ Zero runtime overhead (macros must expand to identical code)
  - 🔧 Macros must live outside `src/generated/` (which gets wiped during codegen)
  - 📐 Must maintain exact same public API for existing code
  - 🚀 Changes must be in code generators, not manual edits to generated files

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md)

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!

## Background & Context

The codegen system generates Rust code from ExifTool's Perl source. While this automation is critical for maintainability (ExifTool releases monthly), the generated code contains significant duplication:

- **Lookup tables**: 42+ files use identical HashMap + LazyLock pattern (~40 lines each)
- **PrintConv tables**: 574+ PRINT_CONV_ declarations with identical structure
- **Tag kit types**: 11 modules duplicate TagKitDef, PrintConvType, etc.
- **File sizes**: Largest generated files exceed 14,000 lines (Canon/Nikon tag_kit/other.rs)

This duplication causes:
- Slower compilation times
- Larger binary size
- Harder to navigate codebase
- IDE performance issues with large files

## Technical Foundation

### Current Patterns

1. **Lookup Table Pattern** (in 42+ files):
```rust
// src/generated/Canon_pm/canonwhitebalance.rs
static CANON_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[...];
pub static CANON_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> = 
    LazyLock::new(|| CANON_WHITE_BALANCE_DATA.iter().cloned().collect());
pub fn lookup_canon_white_balance(key: u8) -> Option<&'static str> {
    CANON_WHITE_BALANCE.get(&key).copied()
}
```

2. **PrintConv Pattern** (574+ occurrences):
```rust
// src/generated/Canon_pm/tag_kit/other.rs
static PRINT_CONV_36: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Full auto");
    // ... many more entries
    map
});
```

3. **Tag Kit Types** (duplicated in 11 modules):
```rust
// Repeated in every tag_kit/mod.rs
pub struct TagKitDef { ... }
pub enum PrintConvType { ... }
pub enum SubDirectoryType { ... }
```

### Code Generators

- `codegen/src/generators/lookup_tables/standard.rs` - Generates lookup tables
- `codegen/src/generators/tag_kit_modular.rs` - Generates tag kit files
- `codegen/src/generators/tag_kit.rs` - Original tag kit generator

## Work Completed

- Analyzed generated code structure and identified duplication patterns
- Measured impact: ~112,000 total lines with 40-50% duplication potential
- Identified safe macro storage location: `src/implementations/macros/`
- Prioritized improvements by risk/reward ratio

## Remaining Tasks

### Phase 1: Lookup Table Macro (Highest Impact, Lowest Risk)

**Acceptance Criteria**: Replace 42+ lookup table implementations with macro invocations

**✅ Correct Output:**
```rust
// In src/generated/Canon_pm/canonwhitebalance.rs
use crate::implementations::macros::define_lookup_table;

define_lookup_table! {
    name: CANON_WHITE_BALANCE,
    key_type: u8,
    data: [
        (0, "Auto"),
        (1, "Daylight"),
        // ...
    ]
}
```

**❌ Common Mistake:**
```rust
// Putting macro in src/generated/ - THIS IS WRONG (gets wiped)
#[macro_export]
macro_rules! define_lookup_table { ... }
```

**Implementation**:
1. Create `src/implementations/macros/mod.rs` and `lookup_table.rs`
2. Write macro that generates the DATA array, LazyLock HashMap, and lookup function
3. Modify `codegen/src/generators/lookup_tables/standard.rs` to emit macro calls
4. Test with one module first (e.g., Canon_pm)

**Expected Impact**: 1,680 lines → 420 lines (75% reduction)

### Phase 2: PrintConv HashMap Macro

**Acceptance Criteria**: Replace 574+ PRINT_CONV_ declarations with macro invocations

**✅ Correct Output:**
```rust
// In tag_kit files
use crate::implementations::macros::define_print_conv;

define_print_conv! {
    name: PRINT_CONV_36,
    entries: {
        "0" => "Full auto",
        "1" => "Manual",
        // ...
    }
}
```

**❌ Common Mistake:**
```rust
// Using proc macros - TOO COMPLEX for this use case
#[proc_macro]
pub fn define_print_conv(input: TokenStream) -> TokenStream { ... }
```

**Implementation**:
1. Add `print_conv.rs` to macros module
2. Create macro similar to lookup_table but for String key HashMaps
3. Modify tag_kit generators to emit macro calls
4. Consider grouping related PrintConvs to reduce static count

**Expected Impact**: 11,000+ lines → 2,000 lines (80% reduction)

### Phase 3: Tag Kit Common Types

**Acceptance Criteria**: Extract shared types to common module

**✅ Correct Output:**
```rust
// In src/implementations/tag_kit_common.rs
pub struct TagKitDef { ... }
pub enum PrintConvType { ... }

// In generated files
use crate::implementations::tag_kit_common::*;
```

**❌ Common Mistake:**
```rust
// Circular dependency - generated code depending on generated code
use crate::generated::common_types::*; // WRONG
```

**Implementation**:
1. Create `src/implementations/tag_kit_common.rs`
2. Move shared types and helper functions
3. Update all tag_kit generators to import from common module
4. Ensure no circular dependencies

**Expected Impact**: 1,100 lines → 200 lines

### Phase 4: Small Optimizations

**Tasks**:
1. **Import prelude**: Common imports for tag_kit files
2. **Const arrays**: Use for tables with <10 entries
3. **Module consolidation**: Group similar patterns

**Implementation**: Evaluate each after phases 1-3 complete

## Prerequisites

- Understanding of Rust macros (macro_rules!)
- Familiarity with codegen system (see [@docs/CODEGEN.md](docs/CODEGEN.md))
- `make codegen` working locally

## Testing Strategy

1. **Compilation Tests**:
   - Generated code must compile identically
   - No changes to public API
   
2. **Runtime Tests**:
   - Existing tests must pass unchanged
   - Benchmark to verify zero performance impact
   
3. **Size Verification**:
   ```bash
   # Before changes
   find src/generated -name "*.rs" | xargs wc -l > before.txt
   
   # After changes
   find src/generated -name "*.rs" | xargs wc -l > after.txt
   
   # Compare
   diff before.txt after.txt
   ```

## Success Criteria & Quality Gates

- [ ] Generated code reduced by at least 15,000 lines
- [ ] All existing tests pass
- [ ] No runtime performance regression
- [ ] Code generators produce deterministic output
- [ ] `make precommit` passes

## Gotchas & Tribal Knowledge

1. **Macro Location**: Must be in `src/implementations/macros/` not `src/generated/`
2. **Incremental Approach**: Test with one module before converting all
3. **Generator Determinism**: Sort all collections for consistent output
4. **LazyLock**: Part of std library, no external dependencies needed
5. **Type Keys**: Some lookups use String keys, others use u8/u16/u32 - macro must handle all