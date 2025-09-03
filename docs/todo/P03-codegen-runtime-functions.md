# Technical Project Plan: Complete Codegen Runtime Function Library

## Goal Definition

- **What Success Looks Like**: All Perl functions referenced in `codegen/src/ppi/normalizer/passes/expression_precedence.rs:327-351` are implemented with runtime functions, generator support, and integration tests, enabling successful code generation for real ExifTool expressions.
- **Core Problem**: The PPI normalizer lists 24 functions but only 12 are fully implemented, causing code generation failures when ExifTool expressions use the missing functions like `join`, `split`, `ord`, `chr`, `uc`, `lc`, `hex`, `oct`, `pack`, `unpack`, or `defined`.
- **Key Constraints**: Must validate actual usage in ExifTool expressions before implementation, follow `Into<TagValue>` pattern exactly, and replicate Perl's exact behavior including all edge cases.
- **ExifTool Alignment**: Each function must match Perl's behavior precisely as documented in our comprehensive test script `/home/mrm/src/exif-oxide/test_perl_functions.pl`, with no optimization or simplification allowed.
- **Success Validation**: Code generation succeeds for all real ExifTool expressions using these functions, `make precommit` passes, and `cargo run --bin compare-with-exiftool` shows identical outputs.

## Mandatory Context Research Phase

### Step 1: Project Foundation Review

**CLAUDE.md Analysis**: 
You must never edit files in `src/generated/**/*.rs` as these are auto-generated - fix the generators in `codegen/src/` instead (CLAUDE.md:89). You must assume concurrent edits and STOP if build errors aren't from your changes (CLAUDE.md:95-97). You must never create functions unless they're absolutely necessary - ALWAYS prefer editing existing files to creating new ones (important-instruction-reminders:2-4).

**TRUST-EXIFTOOL.md Analysis**:
You must translate ExifTool's implementation exactly, never attempting to "improve" or "optimize" its logic, because every seemingly odd piece of code handles specific camera quirks discovered over 25 years (TRUST-EXIFTOOL.md). The comprehensive Perl test script at `/home/mrm/src/exif-oxide/test_perl_functions.pl` documents the exact edge cases you must replicate, including how `int('hello')` returns `0`, `uc(undef)` returns empty string, and `substr` with negative offsets.

**SIMPLE-DESIGN.md Analysis**:
This project directly applies Rule 1 (passes tests) through comprehensive integration tests in `codegen/tests/config/`, Rule 2 (reveals intention) by using descriptive function names matching Perl exactly, and Rule 4 (fewest elements) by avoiding duplicate implementations when `Into<TagValue>` handles type conversion automatically.

**TDD.md Analysis**:
You must follow the bug-fixing workflow: (1) create integration tests in `codegen/tests/config/print_conv/` or `codegen/tests/config/value_conv/` that fail, (2) validate they fail with the expected error, (3) implement the function following Trust ExifTool, (4) validate tests pass plus no regressions via `cargo t`.

**ARCHITECTURE.md Analysis**:
This integrates with the codegen pipeline at three critical points: (1) runtime function definitions in `codegen-runtime/src/`, (2) function exports in `codegen-runtime/src/lib.rs`, and (3) generator mappings in `codegen/src/ppi/rust_generator/visitor.rs:327+`. Breaking any integration point causes code generation failures that prevent the entire system from working.

### Step 2: Precedent Analysis

**Existing Patterns Analysis**:
You must follow the exact pattern established by successful functions like `length`, `int`, and `substr`. All functions must use `<T: Into<TagValue>>` generics for flexibility (see `codegen-runtime/src/string/extraction.rs:42,77,137,142`), be exported in `lib.rs`, have generator support in `visitor.rs` with proper argument validation, and include integration tests in `codegen/tests/config/`. Deviating from this pattern causes type inference failures and compilation errors.

**Dependencies Analysis**:
Changes affect the entire codegen pipeline: (1) `codegen-runtime` functions get compiled into the runtime library, (2) `visitor.rs` generator mappings enable code generation, (3) integration tests validate end-to-end behavior, (4) generated code in `src/generated/functions/` calls these functions. If any piece is missing, expressions using these functions will fail to generate or compile.

**Integration Points Analysis**: 
Critical integration at `codegen-runtime/src/lib.rs:31-37` for function exports, `codegen/src/ppi/rust_generator/visitor.rs:327+` for generator mappings with exact function name matching, and `codegen/tests/config/` for validation. The `expression_precedence.rs:327` list is the authoritative source - any function listed there must have complete implementation or expressions will fail.

**Generated Code Analysis**:
The system auto-generates function call sites in `src/generated/functions/hash_*.rs` files. These call `codegen_runtime::function_name(args)` directly, so runtime functions must be exported and generator must produce correct syntax. Manual transcription of lookup tables is banned - use the existing codegen system for any data tables needed.

### Step 3: ExifTool Research

**Source Analysis**:
You must research actual ExifTool expressions that use each function before implementing. Use `rg -r 'function_name' third-party/exiftool/lib/` to find real usage patterns. The comprehensive test script `/home/mrm/src/exif-oxide/test_perl_functions.pl` documents exact Perl behavior including edge cases: `hex('xyz')` returns `0`, `oct('')` returns `0`, `defined(undef)` returns false, `split(',', 'a,')` returns `('a')`.

**Critical Edge Cases**:
You must handle Perl's specific behaviors exactly: `ord('')` returns `0` (not undef), `chr(0)` returns null byte, `uc(undef)` returns empty string, `pack`/`unpack` format strings have complex parsing rules, `join` with empty separator still joins, `split` with empty pattern splits every character. These behaviors exist to handle camera firmware quirks and file format edge cases.

**Test Cases**:
Use sample expressions from real ExifTool modules found via `rg 'function_name.*\$' third-party/exiftool/lib/`. Validate against the comprehensive test results in `/home/mrm/src/exif-oxide/test_perl_functions.pl` output. Every function must pass identical behavior tests before integration.

**Output Format Requirements**:
Output must match ExifTool exactly. Use `cargo run --bin compare-with-exiftool` to validate after implementation. No deviation allowed from Perl's exact return values, type conversions, or edge case handling.

### Step 4: Risk Assessment

**What Could Go Wrong**:
You could implement functions without validating they're actually used, creating shelf-ware that adds maintenance burden. You could deviate from Perl's exact behavior, breaking compatibility with real camera files. You could forget generator support, causing code generation failures. You could use wrong type signatures, causing compilation errors in generated code.

**Emergency Recovery Plan**:
If functions break existing code generation: (1) `git checkout HEAD~1 -- codegen-runtime/src/` to revert runtime changes, (2) `git checkout HEAD~1 -- codegen/src/ppi/rust_generator/visitor.rs` to revert generator changes, (3) `make codegen && cargo t` to validate system works, (4) debug individual functions in isolation.

**Validation Strategy**:
Every function must pass: (1) `cargo t --package codegen-runtime` for unit tests, (2) `cargo t --package codegen` for integration tests, (3) `make codegen` for code generation, (4) `cargo run --bin compare-with-exiftool test-image.jpg` for ExifTool compatibility, (5) `make precommit` for final validation.

**Integration Testing Requirements**:
You must prove each function works in real ExifTool expressions, not just unit tests. Create integration tests in `codegen/tests/config/` that match actual usage patterns found in ExifTool modules. Test must demonstrate complete pipeline: ExifTool expression → PPI parsing → normalization → code generation → runtime execution → correct output.

## TDD Integration Test (Task 0)

### When Required: Failing Integration Test

- [ ] **Test exists**: A file in `codegen/tests/config/*_conv/*.json` includes a perl expression that contains the given function
- [ ] **Test fails**: `cargo t --package codegen test_function_name` fails before implementation
- [ ] **End-to-end focus**: Tests complete pipeline from ExifTool expression to runtime output  
- [ ] **Success criteria clear**: Test shows exact output matching Perl behavior

---

## Task Definition

### Task A: Validate Function Usage Requirements

**What works after this task**: Complete documentation of which functions are actually used in ExifTool expressions and which can be safely skipped.

**Implementation approach**: Research all 19 remaining functions - 12 unimplemented (`join`, `split`, `unpack`, `pack`, `ord`, `chr`, `uc`, `lc`, `hex`, `oct`, `defined`) plus 7 implemented math functions missing tests (`abs`, `sqrt`, `sin`, `cos`, `atan2`, `exp`, `log`) - by searching ExifTool modules for real usage. Document usage patterns and prioritize by frequency. Skip functions with no real usage to avoid shelf-ware.

**Validation commands**: 
- `rg -r 'join\s*\(' third-party/exiftool/lib/ | head -10` - shows real join() usage
- `rg -r 'split\s*\(' third-party/exiftool/lib/ | head -10` - shows real split() usage  
- `rg -r 'abs\s*\(' third-party/exiftool/lib/ | head -10` - shows real abs() usage
- `rg -r 'sqrt\s*\(' third-party/exiftool/lib/ | head -10` - shows real sqrt() usage
- `find third-party/exiftool/lib -name "*.pm" -exec grep -l "function_name" {} \;` - files using each function

**Dependencies**: None

**Completion checklist**:
- [ ] **Usage documented** → `docs/research/function-usage-analysis.md` with frequency data for all 19 functions
- [ ] **Priority assigned** → Functions ranked by actual usage in ExifTool expressions  
- [ ] **Implementation scope** → Clear list of functions to implement vs skip
- [ ] **Test cases identified** → Real ExifTool expressions using each priority function
- [ ] **Math function validation** → JSON configs exist in `codegen/tests/config/` or new ones created for `abs`, `sqrt`, `sin`, `cos`, `atan2`, `exp`, `log`

### Task B: Complete Math Function Integration Tests

**What works after this task**: All implemented math functions (`abs`, `sqrt`, `sin`, `cos`, `atan2`, `exp`, `log`) have integration tests and work in generated ExifTool expressions.

**Implementation approach**: Research actual usage of math functions in ExifTool expressions using Task A results. Create JSON test configs in `codegen/tests/config/print_conv/` and `codegen/tests/config/value_conv/` for each function that's actually used. Use `cargo run --package codegen --bin generate-expression-tests` to validate tests work correctly.

**Validation commands**: 
- `find codegen/tests/config -name "*math*.json" -o -name "*abs*.json"` - shows existing math test configs
- `cargo t --package codegen math_function` - integration tests pass for each implemented function
- `rg 'abs\|sqrt\|sin\|cos\|atan2\|exp\|log' src/generated/functions/` - shows generated usage in real expressions

**Dependencies**: Task A (usage validation showing which math functions are actually used)

**Completion checklist**:
- [ ] **Usage validated** → Task A shows which math functions are used in real ExifTool expressions
- [ ] **Tests created** → JSON configs in `codegen/tests/config/` for each used math function
- [ ] **Tests passing** → `cargo t --package codegen` passes all new math function tests
- [ ] **Production integration** → `rg 'math_function_name' src/generated/` shows generated calls from real expressions

### Task C: Implement High-Priority String Functions (ord, chr, uc, lc)

**What works after this task**: Character conversion and case transformation functions work in generated ExifTool expressions.

**Implementation approach**: Implement in `codegen-runtime/src/string/transformation.rs` following the `Into<TagValue>` pattern established by existing functions. Add generator support in `visitor.rs` with proper argument validation. Create integration tests matching real usage patterns.

**Validation commands**: 
- `cargo t --package codegen-runtime test_ord_chr_functions` - unit tests pass
- `cargo t --package codegen test_character_conversion` - integration tests pass
- `rg 'ord\|chr\|uc\|lc' src/generated/functions/` - shows generated usage

**Dependencies**: Task A (usage validation)

**Completion checklist**:
- [ ] **Code implemented** → `codegen-runtime/src/string/transformation.rs:1-200` with all 4 functions
- [ ] **Tests passing** → `cargo t ord_chr_uc_lc` succeeds with comprehensive edge cases  
- [ ] **Production integration** → `rg 'ord\|chr\|uc\|lc' src/generated/` shows generated calls
- [ ] **Cleanup complete** → Functions exported in `lib.rs`, generator support in `visitor.rs`

### Task D: Implement High-Priority Conversion Functions (hex, oct)

**What works after this task**: Hexadecimal and octal conversion functions work in generated ExifTool expressions.

**Implementation approach**: Implement in `codegen-runtime/src/string/conversion.rs` with exact Perl behavior including edge cases where invalid input returns 0. Add comprehensive tests covering empty strings, invalid characters, and mixed valid/invalid input.

**Validation commands**: 
- `cargo t --package codegen-runtime test_hex_oct_conversion` - unit tests pass
- `perl -e "print hex('xyz')"` vs Rust output - behavior matches exactly
- `cargo t --package codegen test_conversion_integration` - integration tests pass

**Dependencies**: Task A (usage validation)  

**Completion checklist**:
- [ ] **Code implemented** → `codegen-runtime/src/string/conversion.rs:1-150` with hex/oct functions
- [ ] **Tests passing** → `cargo t hex_oct` covers all edge cases from Perl test script
- [ ] **Production integration** → `rg 'hex\|oct' src/generated/` shows generated usage
- [ ] **Cleanup complete** → Perfect match with `/home/mrm/src/exif-oxide/test_perl_functions.pl` results

### Task E: Implement Array/Binary Functions (if required by usage analysis)

**What works after this task**: Functions like `join`, `split`, `pack`, `unpack` work if actually used in ExifTool expressions.

**Implementation approach**: Only implement if Task A shows significant usage. These are complex functions requiring careful handling of array/binary data. Research exact usage patterns and implement minimal viable version matching real needs.

**Validation commands**: 
- `cargo t --package codegen-runtime test_array_functions` - if implemented
- `make codegen && grep -r 'join\|split\|pack\|unpack' src/generated/` - shows usage
- `cargo run --bin compare-with-exiftool` - validates against ExifTool

**Dependencies**: Task A (usage validation showing these are actually needed)

**Completion checklist**:
- [ ] **Usage justified** → Task A shows substantial real usage requiring implementation
- [ ] **Code implemented** → Functions in appropriate modules with full test coverage
- [ ] **Tests passing** → All unit and integration tests pass  
- [ ] **Production integration** → Actual usage in generated code proven

## Validation Requirements

**RULE**: Every checkbox must have verifiable proof with specific commands and file locations.

### Required Evidence

- **Commands that pass**: `cargo t function_name`, `make precommit`, `cargo run --bin compare-with-exiftool`
- **Code locations**: `codegen-runtime/src/module/file.rs:line_range` where each function is implemented  
- **Integration proof**: `rg 'function_name' src/generated/functions/` shows generated code using functions
- **Behavior changes**: Before/after comparison showing expressions now work that previously failed

### Anti-Vandalism Validation

**Integration Requirements**: Every implemented function must prove it's actually used.

- ✅ **Production Usage**: `rg 'function_name' src/generated/` shows real usage in generated code
- ✅ **Behavior Change**: Expressions that previously failed to generate now work correctly
- ✅ **Cleanup Complete**: All functions exported in `lib.rs`, generator support complete
- ❌ **Shelf-ware**: Functions implemented but no ExifTool expressions actually use them
- ❌ **Half-integrated**: Runtime functions exist but missing generator support

**Definition of Complete**: ExifTool expressions using implemented functions generate working Rust code, execute correctly, and produce output identical to ExifTool, with all validation commands passing.