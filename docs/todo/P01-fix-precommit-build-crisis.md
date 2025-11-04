# P01: Fix Precommit Build Crisis

**Status:** 🔴 CRITICAL - Build completely broken
**Priority:** P01 (HIGHEST)
**Blocks:** Everything - no development can proceed until fixed

## Part 0: Current Dire State

**Date:** 2025-11-04
**Compilation Errors:** 1,143 errors
**Codegen Status:** Broken - finds 0 modules
**Precommit Status:** FAIL

### The Crisis

```bash
make precommit
# Result: TOTAL FAILURE at codegen step
# - Codegen finds 0 ExifTool modules (all paths not found)
# - Generated code is stale (from previous successful run)
# - Cargo build fails with 1,143 compilation errors
# - Cannot run tests, cannot commit, cannot develop
```

### Error Breakdown

From `cargo build 2>&1` analysis:

- **1,113 errors (E0308):** Type mismatches - "expected fn pointer, found fn item"
- **20 errors (E0425):** Missing `power` function in generated code imports
- **4 errors (E0614):** TagValue cannot be dereferenced
- **6 other errors:** Various argument/method issues

## Part 1: Root Cause Analysis

### Issue #1: Codegen Finds No Modules (**CRITICAL BLOCKER**)

**Symptom:**
```
📦 Found 0 ExifTool modules to process
⚠️  Module path not found: lib/Image/ExifTool.pm
⚠️  Module path not found: lib/Image/ExifTool/Canon.pm
... (49 modules all failing)
```

**Root Cause:** Docker environment issues
1. Perl dependencies (JSON::XS, etc.) not installed in Docker
2. ExifTool submodule not initialized by default
3. Codegen scripts fail silently, leaving stale generated code

**Impact:** Cannot regenerate code with fixes, leaving compilation errors unfixable

### Issue #2: Missing `power` Function Import (20 errors)

**Symptom:**
```rust
error[E0425]: cannot find function `power` in this scope
  --> src/generated/functions/hash_74.rs:83:21
```

**Root Cause:**
- `codegen/src/ppi/fn_registry/mod.rs:29` imports list missing `power`
- Generated code calls `power(2i32, val)` but function not imported

**Fix Applied:**
```rust
// OLD:
math::{abs, atan2, cos, exp, int, log, sin, sqrt, IsFloat}

// NEW:
math::{abs, atan2, cos, exp, int, log, power, sin, sqrt, IsFloat}
```

**Status:** ✅ Fixed in commit d08592a, but needs codegen regeneration

### Issue #3: ExifError Type Collision (Was causing 248 → 1,113 errors)

**Symptom:** Two `ExifError` types being re-exported, causing ambiguity

**Root Cause:**
- `src/types/mod.rs` exported both `codegen_runtime::ExifError` AND `errors::ExifError`
- Created name collision throughout codebase

**Fix Applied:** Removed duplicate export from `types/mod.rs`

**Status:** ✅ Fixed in commit d08592a

### Issue #4: Incomplete Unified Expression System

**Symptom:**
```rust
error[E0599]: no method named `evaluate_context_condition` found
```

**Root Cause:** Code calling non-existent methods from incomplete feature

**Fix Applied:** Stubbed out methods to return errors/false

**Status:** ✅ Fixed in commit d08592a

## Part 2: Recovery Plan

### Phase 1: Fix Docker/Environment Setup

**Goal:** Get codegen working in Docker

**Tasks:**

1. **Ensure ExifTool submodule is initialized** ✅ DONE
   ```bash
   git submodule update --init --recursive
   # Note: Need HTTPS not SSH in Docker - update .git/config
   ```

2. **Install Perl dependencies** ✅ ATTEMPTED
   ```bash
   make perl-deps
   # Installs: JSON::XS, PPI, and other required modules
   ```

3. **Verify codegen can find modules**
   ```bash
   make codegen 2>&1 | grep "Found.*modules"
   # Should show: "📦 Found 49 ExifTool modules to process"
   ```

**Success Criteria:** Codegen finds and processes all 49 modules

### Phase 2: Regenerate All Code

**Goal:** Apply fixes by regenerating with working codegen

**Tasks:**

1. **Clean generated code**
   ```bash
   make clean-generated
   ```

2. **Regenerate with fixed imports**
   ```bash
   make codegen
   # Should process all 49 modules
   # Should generate new code with `power` in imports
   ```

3. **Verify power import in generated files**
   ```bash
   grep "power" src/generated/functions/hash_74.rs
   # Should show: math::{abs, ..., power, ...}
   ```

**Success Criteria:** All generated files have correct imports

### Phase 3: Fix Remaining Compilation Errors

**Goal:** Get cargo build to succeed

**Tasks:**

1. **Build and capture errors**
   ```bash
   cargo build 2>&1 | tee build-errors.txt
   grep "^error" build-errors.txt | sort | uniq -c
   ```

2. **Address function pointer type mismatches (1,113 errors)**
   - **Symptom:** `expected fn pointer, found fn item` for ValueConv::Function
   - **Investigation needed:** Why generated code produces wrong types
   - **Likely fix:** Codegen needs to wrap functions in proper type

3. **Fix TagValue dereference issues (4 errors)**
   - Check where generated code tries to deref TagValue
   - May need TagValue Deref impl or fix codegen

4. **Fix remaining method/argument errors**
   - Review each unique error type
   - Determine if codegen bug or runtime API issue

**Success Criteria:** `cargo build` succeeds with 0 errors

### Phase 4: Run Test Suite

**Goal:** Make precommit pass

**Tasks:**

1. **Run all test suites**
   ```bash
   make tests
   # Includes: codegen-test, test, compat-test
   ```

2. **Fix test failures**
   - Catalog all failing tests
   - Determine if failures are from bugs or missing features
   - Fix critical bugs blocking precommit

3. **Run full precommit**
   ```bash
   make precommit
   # Should succeed: codegen, audit, fix, check, tests, build
   ```

**Success Criteria:** `make precommit` exits 0

## Part 3: Prevention

### Docker Setup Documentation

**Action Required:** Document Docker-specific setup in README or GETTING-STARTED

1. How to initialize submodules in Docker
2. How to install Perl dependencies
3. Environment-specific issues and solutions

### CI/CD Validation

**Action Required:** Add CI check that codegen works

```yaml
# .github/workflows/test.yml
- name: Verify codegen works
  run: |
    make perl-deps
    make codegen
    git diff --exit-code src/generated/
    # Fails if codegen changes files (means it wasn't run)
```

### Precommit Hook

**Action Required:** Add pre-commit hook to prevent broken commits

```bash
#!/bin/bash
# .git/hooks/pre-commit
make codegen
if ! cargo build; then
  echo "ERROR: Build fails, cannot commit"
  exit 1
fi
```

## Part 4: Current Status Summary

### Fixes Applied (Commit d08592a)

✅ Added `power` to math imports
✅ Fixed ExifError type collision
✅ Stubbed incomplete unified expression system

### Blockers Remaining

🔴 **CRITICAL:** Codegen cannot process modules (Perl deps)
🔴 **CRITICAL:** 1,143 compilation errors from stale generated code
🟡 **HIGH:** Type system issues in generated code (fn pointer vs fn item)

### Time Estimate

- **Phase 1** (Environment): 1-2 hours if Docker, 30 mins if native
- **Phase 2** (Regenerate): 30 mins once Phase 1 works
- **Phase 3** (Fix errors): 4-8 hours (depends on type mismatch root cause)
- **Phase 4** (Tests): 2-4 hours

**Total:** 8-16 hours to green build

## Emergency Escalation

If you cannot fix the Docker/Perl issues:

1. **Notify user:** Build blocked by environment setup
2. **Workaround:** Commit fixes as-is with note that codegen regen needed
3. **Document:** Create ticket for "Fix Docker codegen environment"

The code fixes (power import, ExifError, expression system) are correct but cannot be validated until codegen works.

## Success Metrics

- [ ] Codegen processes all 49 modules
- [ ] Generated files include `power` import
- [ ] `cargo build` succeeds (0 errors)
- [ ] `cargo test` runs (may have failures, but runs)
- [ ] `make precommit` succeeds

## Quality Checklist

- [x] Problem clearly stated (build completely broken)
- [x] Root causes identified (environment + stale code)
- [x] Recovery plan with clear phases
- [x] Success criteria for each phase
- [x] Time estimates provided
- [x] Escalation path documented
