# Design Decision: PrintConv/ValueConv Expression Normalization

## Executive Summary

After extensive analysis, we've decided to **disable expression normalization entirely** in the codegen conv_registry system. Instead, we add multiple registry entries for different formatting variations of the same expression as we encounter them.

## The Problem

The codegen system was calling Perl subprocesses to normalize expressions for consistent registry lookups. With potentially 800 fields × 100 tags each, this meant up to **80,000 subprocess calls** during code generation, causing severe performance degradation.

## Why Normalization Was Originally Needed

ExifTool's Perl code contains PrintConv and ValueConv expressions with inconsistent formatting:
- `sprintf("%.1f mm",$val)` vs `sprintf("%.1f mm", $val)` (space after comma)
- `$val=~/pattern/` vs `$val =~ /pattern/` (spaces around operators)
- Different whitespace patterns around parentheses, brackets, etc.

We needed consistent formatting to look up these expressions in our registry that maps Perl expressions to Rust implementations.

## Approaches Considered and Rejected

### 1. Perl::Tidy via Subprocess (Original Implementation)
**Approach**: Call Perl::Tidy to normalize each expression
**Problem**: 80,000 subprocess calls, extremely slow
**Status**: ❌ Too slow

### 2. Batch Perl Normalization
**Approach**: Send all expressions to Perl in one batch call
**Problem**: Still requires subprocess calls, results were being ignored in implementation
**Status**: ❌ Still slow, implementation was buggy

### 3. Rust-Native Normalization
**Approach**: Implement whitespace normalization rules in pure Rust
**Problem**: Perl expression normalization is extremely complex:
- Must handle single/double quotes correctly
- Must preserve strings while normalizing outside them
- Must handle `q{}`, `qq{}`, `qw{}`, `qr{}` operators with any delimiter
- Must handle regex patterns, here-docs, and other Perl constructs
**Status**: ❌ Too complex and error-prone

### 4. Perl::Tidy Entire ExifTool Source
**Approach**: Run Perl::Tidy on ExifTool source files during patching
**Problem**: Doesn't work - expressions are inside strings from Perl's perspective
**Status**: ❌ Fundamentally won't work

## Current Solution: No Normalization (KISS)

### How It Works
- `normalize_expression()` returns the input unchanged (NO-OP)
- Registry contains entries for each formatting variation we encounter
- Lookups use direct string equality matching

### Example
```rust
// Both entries in registry map to the same function
m.insert("sprintf(\"%.1f mm\",$val)", ("module", "func"));
m.insert("sprintf(\"%.1f mm\", $val)", ("module", "func"));  // Note space after comma
```

### Trade-offs

**Pros:**
- ✅ Zero runtime overhead, no subprocess calls
- ✅ Simple, predictable behavior
- ✅ Easy to debug (what you see is what you get)
- ✅ Instant performance (string equality vs subprocess)

**Cons:**
- ❌ Registry is slightly larger (but still small - ~100 entries)
- ❌ Must manually add variations as encountered
- ❌ Some duplication in registry definitions

### Performance Impact
- **Before**: Potentially 80,000 subprocess calls to Perl
- **After**: Zero subprocess calls, instant string comparisons
- **Speed increase**: 100x-1000x faster for large codegens

## Implementation Details

### Registry Structure
The registry uses exact string matching without any normalization. When a PrintConv/ValueConv expression isn't found, developers should:

1. Copy the expression EXACTLY as it appears in ExifTool source
2. Add it to the appropriate registry (general or tag-specific)
3. If a formatting variation is later found, add that too

### Backwards Compatibility
The normalization functions are kept as NO-OPs for backwards compatibility:
- `normalize_expression(expr)` - returns `expr` unchanged
- `batch_normalize_expressions(exprs)` - returns identity mapping

## Why This Works

In practice, ExifTool is fairly consistent in its formatting. Most expressions appear in only 1-2 variations, making multiple registry entries feasible. The slight duplication (perhaps 10-20 extra entries total) is worth the massive performance gain and code simplicity.

## Lessons Learned

1. **Don't over-engineer**: The complex normalization system was solving a problem that barely existed
2. **Measure first**: We should have counted actual formatting variations before building normalization
3. **KISS principle wins**: Simple string matching beats complex parsing for this use case
4. **Perl is hard to parse**: Even Perl has trouble parsing Perl (expressions in strings)

## Future Considerations

If we ever need to handle many more variations:
1. We could generate registry entries for common variations automatically
2. We could use fuzzy matching (but this adds complexity)
3. We could normalize at extraction time (when reading ExifTool source)

For now, the simple approach is working perfectly.