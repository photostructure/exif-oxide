# P01: Fix the Build

**Prerequisites**: Read [GETTING-STARTED.md](../GETTING-STARTED.md) first, especially the "Current Project State" section.

## Part 1: Define Success

**Problem**: Build fails with compilation errors, blocking all development.

**Why it matters**: Nothing else can proceed - no testing, no new features, no integration.

**Solution**: Unify function signatures in codegen to match type system expectations.

**Success test**:
```bash
make precommit  # MUST pass with zero errors
```

**Key constraint**: Don't break existing working code. Generated functions that can't be fixed should emit placeholders, not broken syntax.

---

## Part 2: Share Your Expertise

### A. Root Cause Analysis

The errors typically break down as:
- **Type mismatches** (E0308) - cascading from signature issues
- **Dereference errors** (E0614)
- **Method not found** (E0599)
- **Argument count errors** (E0061)

**The core issue**: `PrintConv::Function` and `ValueConv::Function` expect:
```rust
fn(&TagValue, Option<&ExifContext>) -> TagValue
```

But generated functions may have inconsistent signatures:
1. Some missing context parameter: `fn(&TagValue) -> TagValue`
2. Some returning Result: `fn(&TagValue, Option<&ExifContext>) -> Result<TagValue, ExifError>`
3. Some are `fn item` not `fn pointer` (needs explicit cast)

### B. Key Files

```bash
# The type definitions (what signatures SHOULD be)
src/types/tag_info.rs          # PrintConv::Function definition

# Where signatures are generated (what to FIX)
codegen/src/ppi/rust_generator/   # Function generation
codegen/src/strategies/           # Strategy output templates

# Sample generated code locations
src/generated/Exif_pm/           # EXIF module tags
src/generated/Canon_pm/          # Canon manufacturer tags
```

### C. The Fix Strategy

**Option A**: Make all generated functions match expected signature
- Add `_ctx: Option<&ExifContext>` parameter to all functions
- Unwrap Result to TagValue (return original value on error)
- Cast fn items to fn pointers where needed

**Option B**: Change type definitions to accept what's generated
- Make PrintConv/ValueConv accept Result-returning functions
- Add wrapper that handles the unwrapping
- More invasive but might be cleaner long-term

**Recommendation**: Start with Option A - it's more surgical and less risky.

### D. Landmines

1. **NEVER edit `src/generated/` directly** - changes will be overwritten by `make codegen`
2. **Many errors are cascading** - fixing the root cause (signature mismatch) should resolve most of them
3. **Some errors are in generated lookup tables** - these are separate codegen bugs

### E. Learned the Hard Way

From previous build fixes:
- The `cast_\` backslash disaster took 3 hours - `\$val` pattern not handled
- sprintf argument explosion took 2 hours - binary ops inside function args
- unpack/pack/split chain took 4 hours - multi-function chains without parens

**Golden rule**: If codegen can't generate valid Rust, throw `CodeGenError::UnsupportedStructure` to trigger fallback placeholder. A placeholder that compiles is infinitely better than malformed code.

---

## Part 3: Tasks

### Task 1: Audit the Error Categories

**Success**: Documented breakdown of all errors by root cause.

**Implementation**:
```bash
# Categorize errors
cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn

# Find most problematic files
cargo check 2>&1 | grep -E "^\s+-->" | sort | uniq -c | sort -rn | head -20

# Sample each error type
cargo check 2>&1 | grep -A10 "error\[E0308\]" | head -50
```

**Deliverable**: Update this TPP with specific error counts and root causes.

### Task 2: Fix Function Signature Generation

**Success**: Generated functions have correct signature `fn(&TagValue, Option<&ExifContext>) -> TagValue`

**Implementation**:
1. Find where signatures are templated:
   ```bash
   rg "fn.*TagValue.*ExifContext" codegen/src/
   rg "PrintConv::Function" codegen/src/
   ```
2. Update templates to always include context parameter
3. For Result-returning functions, add unwrap wrapper
4. Run `make codegen` to regenerate
5. Check error count: `cargo check 2>&1 | grep "^error" | wc -l`

**If architecture changed**: Search for new signature pattern:
```bash
rg "type.*PrintConv|type.*ValueConv" src/types/
```

### Task 3: Fix Lookup Table Type Errors

**Success**: No "expected &str, found integer" or similar type errors

**Implementation**:
1. Find broken tables:
   ```bash
   cargo check 2>&1 | grep "expected.*found"
   ```
2. Trace to codegen strategy that produced them
3. Fix type inference in strategy
4. Regenerate and verify

### Task 4: Verify Full Build

**Success**: `make precommit` passes

**Implementation**:
```bash
make codegen          # Regenerate all code
cargo check           # Should have 0 errors
cargo t               # Tests should pass
make precommit        # Full validation
```

---

## Emergency Recovery

```bash
# If changes break more than they fix
git checkout HEAD -- codegen/src/
make codegen
cargo check 2>&1 | grep "^error" | wc -l  # Back to baseline

# Incremental approach
git stash
# Apply one change at a time
git stash pop
make codegen && cargo check
```

---

## Quality Checklist

- [ ] Error count reduced to 0
- [ ] `make precommit` passes
- [ ] No manual edits to `src/generated/`
- [ ] Fallback placeholders used for unfixable expressions
- [ ] Changes documented in commit message
