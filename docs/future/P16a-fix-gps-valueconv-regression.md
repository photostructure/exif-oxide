# P16a: Fix GPS ValueConv Regression

## Project Overview

- **Goal**: Restore GPS coordinate and timestamp conversion to return decimal degrees instead of missing function placeholders
- **Problem**: Expression compiler incorrectly treats ExifTool GPS functions as compilable, generating missing_print_conv calls instead of delegating to existing implementations
- **Constraints**: Must preserve expression compilation for pure arithmetic while fixing delegation logic

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

- **ValueConv system**: Converts raw tag values to logical values (e.g., rational arrays to decimal degrees). Unlike PrintConv which formats for display, ValueConv maintains precision for calculations.
- **Expression compiler**: Parses ExifTool arithmetic expressions into Rust code at compile time. Located in `codegen/src/expression_compiler/`, supports basic math, functions, ternary operators.
- **ValueConv registry**: Maps ExifTool expressions to Rust function implementations in `codegen/src/conv_registry/valueconv_registry.rs`. Contains 60+ predefined mappings.
- **GPS coordinate conversion**: ExifTool uses `Image::ExifTool::GPS::ToDegrees($val)` to convert rational arrays [deg, min, sec] to decimal degrees. We have working implementation in `src/implementations/value_conv.rs::gps_coordinate_value_conv()`.

### Key Concepts & Domain Knowledge

- **ValueConv vs PrintConv**: ValueConv does logical conversion (maintains precision), PrintConv does display formatting (human-readable)
- **Rational arrays**: GPS coordinates stored as `[(40,1), (26,1), (468,10)]` representing 40° 26' 46.8"
- **ExifTool function delegation**: Some expressions should use our existing implementations rather than being compiled to inline code
- **Registry precedence**: Registry lookups should take precedence over expression compilation for known functions

### Surprising Context

- **Expression compiler overeager**: `is_compilable()` allows single-argument ExifTool functions to be compiled, but doesn't check if we already have implementations
- **Missing function fallback**: When compiler can't generate code for ExifTool functions, it calls `missing_print_conv()` instead of delegating to registry
- **GPS functions work correctly**: Our `gps_coordinate_value_conv()` implementation is correct and tested, but codegen bypasses it
- **Registry exists but unused**: `VALUECONV_REGISTRY` contains correct mappings for GPS functions, but `is_compilable()` doesn't consult it
- **Compilation vs delegation conflict**: System must decide between inline compilation and function delegation based on whether implementations exist

### Foundation Documents

- **ExifTool GPS source**: `third-party/exiftool/lib/Image/ExifTool/GPS.pm` lines 17-21 (`%coordConv`), lines 582-600 (`ToDegrees` function)
- **Our GPS implementation**: `src/implementations/value_conv.rs` lines 11-58 (`gps_coordinate_value_conv`)
- **Registry mappings**: `codegen/src/conv_registry/valueconv_registry.rs` lines 15-16 (GPS function mappings)
- **Expression compiler**: `codegen/src/expression_compiler/mod.rs` lines 77-110 (`is_compilable` method)
- **Broken generated code**: `src/generated/GPS_pm/tag_kit/mod.rs` lines 178+ (shows missing_print_conv calls)

### Prerequisites

- **Knowledge assumed**: Understanding of Rust codegen, ExifTool ValueConv concepts, GPS coordinate mathematics
- **Setup required**: `cargo build` working, test images available in `test-images/` and `third-party/exiftool/t/images/`

## Work Completed

- ✅ **Root cause identified** → Expression compiler classifies GPS functions as compilable instead of checking registry first, in `classify_valueconv_expression()` order bug
- ✅ **Registry verified** → `VALUECONV_REGISTRY` contains correct mappings for GPS functions (`Image::ExifTool::GPS::ToDegrees` → `gps_coordinate_value_conv`)
- ✅ **Implementation validated** → `src/implementations/value_conv.rs::gps_coordinate_value_conv()` works correctly with comprehensive tests
- ✅ **Order fix implemented** → Changed `codegen/src/conv_registry/valueconv_registry.rs:97-100` to check registry before compilation, ensuring GPS functions delegate properly
- ✅ **GPS module regenerated** → `src/generated/GPS_pm/tag_kit/mod.rs:172-177` now calls `gps_coordinate_value_conv` for GPS coordinate tags (IDs 2, 4, 20, 22)
- ✅ **Integration tests added** → `tests/gps_registry_fix_integration_test.rs` validates GPS coordinates return decimal degrees instead of missing function calls

## Remaining Tasks

### ✅ ALL TASKS COMPLETE

**Status**: P16a GPS ValueConv regression has been fully resolved.

**Final Verification**:
- [x] **Classification fix**: `codegen/src/conv_registry/valueconv_registry.rs:97-100` checks registry before compilation
- [x] **GPS delegation**: `src/generated/GPS_pm/tag_kit/mod.rs:172-177` calls `gps_coordinate_value_conv` for GPS coordinate tags
- [x] **Integration tests**: `tests/gps_registry_fix_integration_test.rs` validates GPS coordinates return decimal degrees
- [x] **Manual validation**: GPS coordinates now return `TagValue::F64(decimal)` instead of missing function calls
- [x] **Cleanup**: Removed redundant registry check from `is_compilable()` method
- [x] **Regression prevention**: GPS ValueConv functions no longer incorrectly treated as compilable expressions

**Completed Tasks Summary**:

### ✅ 1. Task: Fix classification logic to prioritize registry over compilation
- **Implementation**: Changed order in `classify_valueconv_expression()` to check registry first
- **Integration**: GPS module automatically uses corrected logic after regeneration
- **Testing**: GPS coordinates now return decimal degrees (40.446333, 118.2394444, etc.)

### ✅ 2. Task: Regenerate GPS_pm module with corrected delegation logic  
- **Implementation**: Generated GPS module now calls `gps_coordinate_value_conv` for GPS coordinate tags
- **Integration**: `apply_value_conv()` function properly delegates to existing implementations
- **Testing**: No more `missing_print_conv` calls for GPS functions

### ✅ 3. Task: Add comprehensive integration tests for GPS ValueConv delegation
- **Implementation**: Created `tests/gps_registry_fix_integration_test.rs` with multiple GPS test scenarios
- **Integration**: Tests validate GPS coordinates, destinations, and timestamps through generated module
- **Testing**: All GPS ValueConv functions verified to return proper decimal values

## Implementation Guidance

### Recommended Patterns

- **Registry-first delegation**: Always check registry before attempting compilation for ExifTool functions
- **Preserve arithmetic compilation**: Keep compiling pure arithmetic expressions like `$val / 8`, `2**(-$val)`
- **Graceful fallback**: Unknown functions should still compile to `missing_print_conv` for debugging
- **Module-scoped lookups**: Use module parameter in registry lookups for context-sensitive mappings

### Architecture Considerations

- **Compilation priority**: Registry delegation > Expression compilation > Missing function fallback
- **Performance**: Registry lookup is O(1) HashMap access, minimal overhead
- **Maintainability**: Centralized decision logic prevents inconsistent behavior across modules

### ExifTool Translation Notes

- **Trust GPS.pm exactly**: Our `gps_coordinate_value_conv` already implements ExifTool's `ToDegrees` formula precisely
- **Preserve all precision**: GPS coordinates need full floating-point precision for mapping applications
- **Handle edge cases**: Zero denominators, missing values handled per ExifTool behavior

## Integration Requirements

- [x] **Activation**: Fix is automatically used when processing GPS tags (no opt-in required)
- [x] **Consumption**: All GPS tag processing uses corrected delegation logic 
- [x] **Measurement**: GPS coordinates return decimal degrees instead of missing function calls
- [x] **Cleanup**: Remove broken compiled arithmetic expressions from generated GPS module

## Testing

- **Unit**: Test modified `is_compilable()` method with GPS function expressions
- **Integration**: Verify GPS coordinates extracted correctly from test images
- **Manual check**: Run `cargo run -- test-images/canon/Canon_T3i.jpg -GPSLatitude` and confirm decimal output

## Definition of Done

- [x] **No regressions**: GPS ValueConv fix doesn't break existing expression compilation
- [x] **GPS decimal output**: GPS tags return decimal degrees (40.446333) instead of missing function calls  
- [x] **Function delegation**: Generated `GPS_pm` module calls `gps_coordinate_value_conv` instead of `missing_print_conv`
- [x] **Comprehensive testing**: Integration tests validate GPS coordinates, destinations, and timestamps
- [x] **Expression compatibility**: Expression compiler still compiles pure arithmetic expressions correctly
- [x] **Registry priority**: All registry functions take precedence over expression compilation

**Status**: ✅ COMPLETE - All Definition of Done criteria met

## Additional Gotchas & Tribal Knowledge

- **Don't edit generated GPS module directly** → It gets overwritten by codegen → Fix the expression compiler logic instead
- **Registry contains both exact and normalized expressions** → Use `lookup_valueconv()` helper which handles both cases
- **GPS coordinates need unsigned values** → Our `gps_coordinate_value_conv` returns unsigned degrees, composite tags apply hemisphere signs
- **Missing function calls are debugging features** → Don't suppress them for truly unknown functions, only fix delegation for known ones
- **Codegen runs automatically in CI** → Changes to expression compiler affect multiple modules, test thoroughly

## Quick Debugging

Stuck? Try these:

1. `cargo run -- test-images/canon/Canon_T3i.jpg -GPSLatitude` - See current GPS output
2. `rg "Image::ExifTool::GPS::ToDegrees" third-party/exiftool/` - Find ExifTool implementation
3. `cargo t gps_coordinate_value_conv -- --nocapture` - Test our implementation
4. `git log -S "missing_print_conv"` - Find when missing function calls were introduced