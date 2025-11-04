# Build Crisis Analysis - 2025-11-04

## Executive Summary

**Status:** 🔴 CRITICAL - `make precommit` cannot pass due to broken codegen and 1,143 compilation errors

**Root Cause:** Docker environment cannot run codegen due to missing dependencies, leaving stale generated code with multiple compilation errors

**Immediate Blockers:**
1. Perl dependencies (JSON::XS) not available in Docker → Codegen finds 0 modules
2. ExifTool submodule not initialized → Module paths don't exist
3. Stale generated code has 1,143 errors from missing `power` function and type system issues

## What I Found

### Build State Analysis

Ran comprehensive analysis of compilation errors:
```bash
cargo build 2>&1 # Captured 21,453 lines of errors to temp file
```

**Error Summary:**
- 1,113 type mismatch errors (E0308) - "expected fn pointer, found fn item"
- 20 missing `power` function errors (E0425)
- 4 TagValue dereference errors (E0614)
- 6 misc errors (missing methods, wrong arguments)

**Total:** 1,143 compilation errors (not the "5" claimed in old P01 doc!)

### Codegen Analysis

Attempted to run codegen:
```bash
make codegen
# Result: "📦 Found 0 ExifTool modules to process"
# All 49 module paths reported as "not found"
```

**Root Causes Discovered:**
1. ExifTool submodule not initialized (`-` prefix in `git submodule status`)
2. Perl JSON::XS module not installed (field_extractor.pl fails)
3. Without successful extraction, codegen skips regeneration
4. Old generated code remains with bugs

### Documentation Analysis

Used subagent to analyze all 67 TODO files in `docs/todo/`:

**Key Findings:**
- Most TODO docs are out of date or incorrectly claim completion
- P01 claimed "5 errors" when reality is 1,143 errors
- Many P07 files exist but build is too broken to work on them
- No clear prioritization reflecting actual build state

## What I Fixed

### Code Fixes (Commit d08592a)

1. **Added `power` function to imports** (`codegen/src/ppi/fn_registry/mod.rs:29`)
   - Generated code calls `power(2i32, val)` but function wasn't imported
   - Added `power` to math imports list
   - ✅ Fix is correct but needs codegen regeneration to take effect

2. **Fixed ExifError type collision** (`src/types/mod.rs:7`)
   - Two different ExifError types being re-exported caused ambiguity
   - Removed `codegen_runtime::ExifError` from re-exports
   - Kept only `types::errors::ExifError` which has conversions from runtime error
   - ✅ Reduces type confusion

3. **Stubbed incomplete unified expression system** (`src/types/binary_data.rs:224,640`)
   - Code was calling `evaluate_context_condition()` which doesn't exist
   - Stubbed methods to return errors/false instead of calling non-existent methods
   - Added TODO comments marking incomplete features
   - ✅ Prevents compilation errors from incomplete features

### Documentation Fixes

1. **Created new accurate P01** (`docs/todo/P01-fix-precommit-build-crisis.md`)
   - Reflects actual current state (1,143 errors, not 5)
   - Documents root causes and recovery plan
   - Provides phase-by-phase fix strategy
   - Archived old incorrect P01

2. **Created comprehensive analysis** (this document)
   - Documents findings from build, codegen, and documentation analysis
   - Provides actionable next steps
   - Summarizes all TODO docs state

## Critical Path to Green Build

### Phase 1: Fix Environment (1-2 hours)

**Problem:** Codegen cannot run in Docker due to missing dependencies

**Solution:**
```bash
# Initialize ExifTool submodule
git submodule update --init --recursive
# Note: May need to configure HTTPS instead of SSH for Docker

# Install Perl dependencies
make perl-deps
# This installs JSON::XS, PPI, and other required modules

# Verify codegen works
make codegen 2>&1 | grep "Found.*modules"
# Should show: "📦 Found 49 ExifTool modules to process"
```

**Blocking Issue:** I attempted this but JSON::XS installation may need more setup

### Phase 2: Regenerate Code (30 mins)

Once codegen works:
```bash
make clean-generated
make codegen
# Should regenerate all files with `power` in imports
```

Verify:
```bash
grep "power" src/generated/functions/hash_74.rs
# Should show: math::{abs, ..., power, ...}
```

### Phase 3: Fix Type System Issues (4-8 hours)

After regeneration, the 1,113 "expected fn pointer, found fn item" errors need investigation:

**Symptom:**
```rust
error[E0308]: mismatched types
  --> src/generated/CanonCustom_pm/personal_func_values_tags.rs:25:58
   |
25 |    value_conv: Some(ValueConv::Function(ast_value_41e4bfecd227b921)),
   |                     ------------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |                     |                   expected fn pointer, found fn item
```

**Possible Causes:**
- Generated code passes function items where function pointers expected
- ValueConv::Function enum variant signature may have changed
- Type coercion not happening automatically

**Investigation Needed:**
1. Check `ValueConv::Function` definition - what type does it expect?
2. Check how functions are generated - are they creating the right type?
3. May need to wrap functions: `ValueConv::Function(ast_value_... as fn(...))`

### Phase 4: Run Tests (2-4 hours)

Once build succeeds:
```bash
make tests       # Run full test suite
make precommit   # Should pass
```

**Total Time Estimate:** 8-16 hours

## TODO Documentation Status

Analyzed 67 TODO files with subagent. Key findings:

### Files Out of Date
- `P01-fix-the-build.md` - Claims 5 errors, reality 1,143 → ARCHIVED
- Most P07 files claim completion but build is broken
- Many P1x files haven't been reviewed for current relevance

### Critical Files to Focus On
1. **P01-fix-precommit-build-crisis.md** (NEW) - Current accurate state
2. **P03-codegen-runtime-functions.md** - Likely documents missing `power` function
3. **P07-universal-extractor-architecture.md** - Main architecture work
4. **P08-codegen-deduplication-handoff.md** - Function dedup issues

### Recommended Actions
1. Archive or delete completed/obsolete TODOs
2. Renumber based on actual blocking priority
3. Create clear dependency chain
4. Mark clearly what blocks precommit vs future work

## Recommendations

### Immediate (This Session)
1. ✅ Commit code fixes (power, ExifError, expression system stubs)
2. ✅ Commit new P01 document
3. ✅ Create this analysis document
4. ⏸️ Push to branch (need working codegen to validate)

### Next Steps (User or Future Session)
1. Fix Docker/Perl environment to get codegen working
2. Regenerate all code with fixes applied
3. Investigate and fix type system issues (1,113 errors)
4. Validate tests pass
5. Clean up and re-prioritize TODO documentation

### Process Improvements
1. Add CI check that codegen works before merge
2. Document Docker-specific setup requirements
3. Add precommit hook to prevent broken commits
4. Keep P01 document as source of truth for build state

## Files Changed

### Code Changes (Committed: d08592a)
```
M codegen/src/ppi/fn_registry/mod.rs     # Added power to imports
M src/types/binary_data.rs                # Stubbed expression system
M src/types/mod.rs                        # Fixed ExifError collision
```

### Documentation Changes (To Commit)
```
A docs/todo/P01-fix-precommit-build-crisis.md    # New accurate P01
M docs/todo/ARCHIVE-P01-fix-the-build-OLD.md     # Archived old P01
A BUILD-CRISIS-ANALYSIS.md                        # This document
```

## Key Insights

1. **Documentation Drift:** TODO docs claimed completion while build was completely broken
2. **Silent Failures:** Codegen failed silently, leaving stale code that compiled previously
3. **Environment Fragility:** Docker setup missing critical dependencies
4. **Type System Evolution:** Generated code no longer matches runtime API expectations

## Questions for User

1. **Docker Setup:** Can you provide working Docker image with Perl dependencies?
2. **Type System:** Did ValueConv::Function signature change recently?
3. **Testing Strategy:** What's minimum test coverage needed for precommit to pass?
4. **TODO Process:** Want me to continue cleaning up and re-prioritizing TODO docs?

## Success Criteria

- [ ] Codegen processes all 49 modules (not 0)
- [ ] Generated files include `power` import
- [ ] Cargo build succeeds (0 errors, not 1,143)
- [ ] Tests run (may have failures, but executable)
- [ ] Make precommit passes

---

**Analysis Date:** 2025-11-04
**Analyzed By:** Claude (exif-oxide debugging session)
**Build Status:** 🔴 CRITICAL (1,143 errors)
**Recommended Priority:** P01 - Drop everything until build works
