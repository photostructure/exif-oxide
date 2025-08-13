# P08: Codegen Function Deduplication - Engineering Handoff

## Issue Description

The P07 unified codegen system successfully replaced runtime expression evaluation with PPI AST-based code generation, but has a critical remaining issue: **duplicate function names** are being generated, causing compilation failures.

**Current Error:**
```
error[E0428]: the name `canon_camera_info650d_file_index_399b60f4_value_ast` is defined multiple times
```

**Root Cause:** Multiple ExifTool tags use identical expressions (e.g., `$val + 1`) but the codegen generates separate functions for each tag instead of reusing functions with identical AST structure.

## Completed Work

✅ **P07 Foundation Complete:**
- Removed runtime expression evaluator ("thrown into Mount Doom")
- All Perl interpretation now happens via PPI at build time
- Fixed function signatures: ValueConv returns `Result<TagValue, ExifError>`
- Fixed arithmetic operations with proper f64 literals (`10.0` not `10`)
- Fixed dereference syntax (`*v` not `v` for TagValue matches)
- Added hash-based function naming to prevent collisions
- Fixed fallback cases to wrap ValueConv returns in `Ok()`

✅ **Infrastructure:**
- PPI AST parsing working correctly
- Function generation working for simple expressions
- Tag table generation working
- Error handling and type safety complete

## Current State

**Working:** 
- Simple arithmetic expressions (`$val + 1`, `$val / 100`)
- String interpolation (`"$val mm"`)
- Function calls (`sprintf`, `int`, `abs`)
- Complex fallback with proper error messages

**Broken:**
- Compilation fails due to duplicate function names
- Same AST generates multiple identical functions

## Remaining Work

### Immediate Priority: Fix Function Deduplication

**Problem:** The tag kit strategy generates duplicate functions when multiple tags use the same expression.

**Current Implementation:**
```rust
// In codegen/src/strategies/tag_kit.rs:27
ppi_generated_functions: Vec<String>, // Allows duplicates!
```

**Solution Started:**
```rust
// Changed to HashMap for deduplication
ppi_generated_functions: std::collections::HashMap<String, String>, // AST hash -> function code
```

**Next Steps:**
1. **Complete AST-based hashing** (in progress in tag_kit.rs:480-490)
2. **Update function storage logic** to check HashMap before generating
3. **Test deduplication** works correctly
4. **Regenerate all code** and verify compilation

### Task Breakdown

#### Task 1: Implement AST-based Function Deduplication
**Files:** `codegen/src/strategies/tag_kit.rs`
**Lines:** 480-520 (hash generation and function storage)

```rust
// Replace string-based hash with AST-based hash:
let ast_hash = {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    // Hash the AST structure instead of expression text
    serde_json::to_string(&ppi_ast).unwrap().hash(&mut hasher);
    format!("{:x}", hasher.finish()).chars().take(8).collect::<String>()
};
```

#### Task 2: Update Function Storage Logic
**Files:** `codegen/src/strategies/tag_kit.rs`
**Lines:** 515-520

```rust
// Check if function already exists before generating
if let Some(existing_function) = self.ppi_generated_functions.get(&ast_hash) {
    // Reuse existing function, don't generate duplicate
    return Ok(format!("Some(ValueConv::Function({function_name}))"));
}

// Generate new function and store by AST hash
let generated_function = generator.generate_function(&ppi_ast)?;
self.ppi_generated_functions.insert(ast_hash.clone(), generated_function);
```

#### Task 3: Update Function Output Logic
**Files:** `codegen/src/strategies/tag_kit.rs`
**Lines:** 180-190

```rust
// Change from Vec iteration to HashMap values
for func in self.ppi_generated_functions.values() {
    writeln!(content, "{}", func)?;
}
```

## Key Files to Study

### Core Logic:
- `codegen/src/strategies/tag_kit.rs` - Main strategy implementation
- `codegen/src/ppi/rust_generator.rs` - PPI to Rust conversion
- `codegen/src/ppi/types.rs` - AST node definitions

### Test Cases:
- `src/generated/Canon_pm/camera_info650d_tags.rs` - Shows duplicate function issue
- `src/generated/Canon_pm/camera_info_power_shot2_tags.rs` - Another duplicate case

### Documentation:
- `docs/CODEGEN.md` - Overall codegen architecture
- `docs/TRUST-EXIFTOOL.md` - Core principles

## Success Criteria

1. **Compilation Success:** `cargo check --package exif-oxide` passes without duplicate function errors
2. **Deduplication Working:** Same AST expressions generate single shared function
3. **Function Correctness:** Generated functions produce correct results for all tag types
4. **No Regressions:** All existing functionality continues working

## Testing Strategy

```bash
# 1. Test compilation
cargo check --package exif-oxide

# 2. Test specific cases that were failing
rg "canon_camera_info650d_file_index_399b60f4_value_ast" src/generated/
# Should find only ONE function definition

# 3. Run full test suite
cargo t

# 4. Test with ExifTool comparison
cargo run --bin compare-with-exiftool test-images/canon/IMG_1234.CR2
```

## Context for Success

### Why AST-based Deduplication?
- **Semantic correctness:** Two expressions with identical AST have identical behavior
- **Robust:** Handles whitespace, formatting differences automatically  
- **Future-proof:** Works for complex expressions as PPI parsing improves

### Trust ExifTool Principle
- Never modify the generated expression logic
- Always preserve ExifTool's exact behavior
- Only deduplicate when AST is truly identical

### Performance Considerations
- AST hashing is done once at codegen time (not runtime)
- Reduced binary size from fewer duplicate functions
- Faster compilation due to less code duplication

## Future Refactoring Ideas

### Short Term:
1. **Improve error messages** when PPI parsing fails
2. **Add validation** that deduplicated functions have compatible signatures
3. **Better handling** of complex sprintf expressions

### Medium Term:
1. **Extract common patterns** (arithmetic, string formatting) into reusable templates
2. **Optimize hash algorithm** for better performance and collision resistance
3. **Add metrics** on deduplication effectiveness

### Long Term:
1. **Template-based generation** for common expression patterns
2. **Incremental codegen** that only regenerates changed functions
3. **Cross-module deduplication** for expressions shared across ExifTool modules

## Known Edge Cases

1. **Complex sprintf:** Multi-line expressions may need special handling
2. **Function signatures:** Ensure deduplicated functions have compatible return types
3. **Hash collisions:** Extremely unlikely but should be detected
4. **Empty AST:** Handle parsing failures gracefully

## Getting Started

1. **Read this document** and study the key files
2. **Run the failing tests** to see current errors
3. **Implement Task 1** (AST-based hashing) first
4. **Test incrementally** after each change
5. **Ask questions** about ExifTool behavior if needed

The foundation is solid - this is primarily a data structure change from Vec to HashMap with proper key generation. The hard work of PPI parsing and function generation is already complete.