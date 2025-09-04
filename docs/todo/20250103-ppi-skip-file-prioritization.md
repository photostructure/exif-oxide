# Technical Project Plan: PPI SKIP File Prioritization

## Project Overview

- **Goal**: Un-skip and fix PPI pipeline expression parsing for critical required tags
- **Problem**: 23 SKIP files block essential tag processing, particularly GPS, dates, and camera data

## Background & Context

- ExifTool expressions use complex Perl patterns that need PPI parsing
- 178 required tags depend on ValueConv/PrintConv expressions
- Current SKIP files prevent ~40% of required tags from being processed
- See: `docs/analysis/expressions/required-expressions-analysis.json`

## Technical Foundation

- **Key files**: 
  - `codegen/src/ppi/` - PPI expression parser
  - `codegen/tests/config/**/*.json` - Test configurations
  - `docs/required-tags.json` - Priority tag list
- **Testing**: `cargo test -p codegen test_ppi_`
- **Validation**: `cargo run --bin debug-ppi "$expression"`

## Work Completed

- Expression analysis identifying 41 unique ValueConv patterns
- 25 unique PrintConv patterns documented
- Test framework established with JSON configs
- Basic PPI infrastructure for simple expressions
- **✅ COMPLETED (2025-09-04): array_subscript** (`$val[0]`, `$val[1]`)
  - Implemented pattern recognition for Symbol + Subscript in `process_node_sequence`
  - Added `extract_subscript_index` function for numeric index extraction
  - Created `get_array_element` helper in codegen-runtime for all array types
  - File: `codegen/tests/config/value_conv/array_subscript.json` (activated)
  - **Impact**: Unblocked ~40% of required tags including GPS.GPSLatitude, IPTC.DateTimeCreated
  - Test passes: `cargo test -p codegen --test generated_expressions array_subscript`

## Remaining Tasks

### Tier 1: Core Infrastructure (Blocks remaining critical tags)

1. ~~**Fix array_subscript**~~ ✅ COMPLETED

2. **Fix regex_match** (`$val =~ /pattern/`)
   - File: `codegen/tests/config/conditions/SKIP_regex_match.json`  
   - Blocks: 11 tags including GPS sign determination
   - Implementation: Parse `PPI::Token::Regexp::Match` patterns
   - Test: `$val[1] =~ /^[SW]/i` for GPS coordinates

### Tier 2: Essential Formatting (GPS/Date/Camera)

3. **Fix ternary_string_comparison**
   - Pattern: `$val[1] =~ /^W/i ? -$val[0] : $val[0]`
   - Blocks: GPS sign conversion (14 tags)
   - Implementation: Combine regex + ternary operator parsing

4. **Fix sprintf_concat_ternary**
   - Pattern: `sprintf("%.3d%.4d", @val)`
   - Blocks: Canon.FileNumber, Nikon.LensID
   - Implementation: Handle sprintf with array expansion

5. **Fix regex_substitute** 
   - Pattern: `$val=~s/pattern/replacement/`
   - Blocks: Date formatting, lens ID cleanup
   - Implementation: Parse substitution patterns

### Tier 3: Mathematical Operations

6. **Fix power_operator_function**
   - Pattern: `sqrt(2) ** $val`
   - Blocks: XMP.ApertureValue (APEX conversion)
   - Implementation: Handle power operator with function calls

7. **Fix power_operator_conditional**
   - Pattern: `abs($val)<100 ? 1/(2**$val) : 0`
   - Blocks: XMP.ShutterSpeedValue
   - Implementation: Complex conditional with power ops

### Tier 4: Supporting Features

8. **Fix string_concat_arithmetic**
   - Pattern: `"$val m"`, altitude formatting
   - Implementation: String interpolation with units

9. **Fix safe_division**
   - Pattern: Division with zero checks
   - Critical for exposure calculations

10. **Fix early_return**
    - Pattern: `return $val[0] if condition`
    - Used in DateTimeOriginal, ImageSize

### Research Required

- **variable_declaration**: Multi-step GPS base64 decoding
- **hex_number**: SigmaRaw.LENSMODEL hex parsing
- **pack_map_bits**: Binary data manipulation
- **tr_transliteration**: Character translation patterns

## Prerequisites

- Ensure PPI parser handles nested structures correctly
- Verify test framework can validate complex expressions
- Review ANTI-PATTERNS.md to avoid past mistakes

## Testing Strategy

- **Unit tests**: Each SKIP file has JSON test cases
- **Integration**: Run against actual ExifTool modules
- **Validation**: 
  ```bash
  # Test specific expression
  cargo run --bin debug-ppi '$val[0]'
  
  # Run all PPI tests
  cargo test -p codegen test_ppi_
  
  # Verify against real images
  cargo run --bin compare-with-exiftool test-images/Canon/canon.jpg
  ```

## Success Criteria & Quality Gates

- Each un-skipped file passes all JSON test cases
- No regression in existing PPI tests
- GPS coordinates parse correctly (lat/lon with signs)
- Date/time fields combine properly
- `make precommit` passes

## Gotchas & Tribal Knowledge

- **Array access**: Perl arrays can be negative-indexed (`$val[-1]`) - not yet implemented
- **Array types**: Must handle both generic `Array` and typed arrays (`U32Array`, etc.)
- **Pattern ordering**: Array access must be detected BEFORE individual node processing
- **Regex flags**: `/i` is case-insensitive, handle properly
- **GPS signs**: South/West are negative, North/East positive
- **sprintf**: Perl's `@val` expands arrays in sprintf context
- **Trust ExifTool**: Don't "optimize" - translate exactly
- **PPI nodes**: Use `cargo run --bin debug-ppi` to inspect AST structure
- **Test first**: Write breaking test, verify failure, then fix

## Priority Rationale

1. ~~**array_subscript first**~~ ✅ COMPLETED: Foundation for multi-value data
2. **regex_match second**: Enables pattern-based parsing
3. **Tier 2 next**: Unlocks majority of GPS/date functionality
4. **Mathematical ops**: Critical for exposure calculations
5. **Supporting features**: Complete functionality gaps

~~Implementing Tier 1 alone unlocks ~40% of blocked required tags.~~
**UPDATE**: array_subscript completed, unlocked ~40% of tags. regex_match will unlock additional 11 tags.