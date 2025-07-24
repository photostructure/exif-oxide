# Technical Project Plan: Tag Kit Migration and Runtime Retrofit

## Project Overview

**Goal**: Complete the tag kit system migration by transitioning from legacy inline_printconv functions to the unified tag_kit system for PrintConv handling.

**Problem**: We currently have two overlapping tag extraction systems running in parallel. The tag kit system provides a unified approach that eliminates tag ID/PrintConv mismatches and simplifies maintenance, but the codebase still references legacy inline functions.

## Background & Context

### Why This Work is Needed

- **Bug Prevention**: Tag kit eliminates offset errors by extracting tag IDs with their PrintConvs together
- **Maintenance Simplification**: One unified extractor instead of multiple overlapping systems
- **ExifTool Updates**: Monthly releases become easier with automated extraction
- **PR Reviews**: Generated code clearly shows tag+PrintConv relationships

### Related Documentation

- [DONE-20250122-tag-kit-codegen.md](../done/DONE-20250122-tag-kit-codegen.md) - Tag kit implementation details
- [EXTRACTOR-GUIDE.md](../reference/EXTRACTOR-GUIDE.md) - Extractor comparisons
- [CODEGEN.md](../CODEGEN.md) - Codegen system overview
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core project principle

## Technical Foundation

### Key Codebases

1. **Tag Kit System** (`src/generated/*/tag_kit/`):
   - Unified tag definitions with embedded PrintConv implementations
   - Modular organization (datetime, thumbnail, interop, other)
   - Static TAG_KITS registry with apply_print_conv functions

2. **Legacy Inline System** (`src/generated/*/inline_*.rs`):
   - Individual lookup functions like `lookup_canon_white_balance`
   - Currently still referenced by source code in `src/implementations/`

3. **Codegen Infrastructure** (`codegen/src/extractors/tag_kit.rs`):
   - Custom TagKitExtractor with table consolidation logic
   - Handles multiple tables per module (e.g., Canon has 17 tables)

### APIs

- **Tag Kit**: `apply_print_conv(tag_id, value, evaluator, errors, warnings) -> TagValue`
- **Legacy**: `lookup_[module]_[table]_[tag](key) -> Option<&'static str>`

## Work Completed

### ✅ Tag Kit Migration (100% Complete)

All 7 inline_printconv configs successfully migrated to tag_kit:
- **Canon** (17 tables) → 300 tags with 97 PrintConv tables
- **Sony** (10 tables) → 405 tags with 228 PrintConv tables  
- **Olympus** (8 tables) → 351 tags with 72 PrintConv tables
- **Panasonic** (1 table) → 129 tags with 59 PrintConv tables
- **MinoltaRaw** (2 tables) → 35 tags with 5 PrintConv tables
- **Exif** (1 table) → 414 tags with 111 PrintConv tables
- **PanasonicRaw** (1 table) → 48 tags with 1 PrintConv table

### ✅ Infrastructure Fixes (100% Complete)

1. **TagKitExtractor consolidation logic** - Fixed to handle multiple tables per module
2. **Runtime integration** - All modules generate tag_kit/ subdirectories with proper mod.rs exports
3. **Compilation errors** - Fixed missing file_types functions and ensured both systems coexist

### ✅ Current State

Both systems now work in parallel:
- Tag kit files generate correctly with consolidated output
- Inline functions still generate for backward compatibility
- `cargo check` passes without errors
- Code generation completes successfully

## Remaining Tasks

### Phase 1: Source Code Migration (2-3 days)

**High Confidence Tasks**:

#### 1.1 Audit Current Usage
```bash
# Find all inline function references
grep -r "inline::" src/
grep -r "lookup_.*__" src/implementations/

# Expected locations based on compilation errors that were fixed:
# - src/implementations/canon/binary_data.rs
# - src/implementations/canon/mod.rs  
# - src/implementations/minolta_raw.rs
```

#### 1.2 Update Canon References
Replace inline function calls in these files:
- `src/implementations/canon/binary_data.rs:210` - focallength_inline usage
- `src/implementations/canon/binary_data.rs:307` - shotinfo_inline usage  
- `src/implementations/canon/binary_data.rs:487` - panorama_inline usage
- `src/implementations/canon/binary_data.rs:541` - mycolors_inline usage
- `src/implementations/canon/mod.rs:888` - camerasettings_inline usage

**Pattern**:
```rust
// OLD:
use crate::generated::Canon_pm::focallength_inline::*;
let result = lookup_focal_length__focal_type(value)?;

// NEW:  
use crate::generated::Canon_pm::tag_kit::apply_print_conv;
let mut evaluator = ExpressionEvaluator::new();
let mut errors = Vec::new();
let mut warnings = Vec::new();
let result = apply_print_conv(tag_id, &TagValue::U8(value), &mut evaluator, &mut errors, &mut warnings);
```

#### 1.3 Update MinoltaRaw References
Replace lookup functions in `src/implementations/minolta_raw.rs`:
- Lines 16, 30, 44, 58, 72 - various PRD/RIF lookup functions

### Phase 2: Cleanup and Validation (1-2 days)

**Requires Research**:

#### 2.1 Remove Inline PrintConv Configs
After source code migration is complete:
- Delete `codegen/config/*/inline_printconv.json` files
- Update codegen pipeline to not generate inline files
- Verify no compilation errors

#### 2.2 Add Runtime Integration (OPTIONAL)
Consider wiring tag_kit into the main runtime registry (`src/registry.rs`) to use tag_kit as primary lookup before falling back to manual implementations.

### Phase 3: Testing and Metrics (1 day)

#### 3.1 Integration Testing
```bash
# Run existing tag kit tests
cargo test tag_kit_integration

# Compare outputs with ExifTool
cargo run --bin compare-with-exiftool test-images/canon.cr2
cargo run --bin compare-with-exiftool test-images/sony.arw
```

#### 3.2 Add Usage Metrics (OPTIONAL)
Track tag_kit vs manual registry usage to validate migration success.

## Prerequisites

- Understanding of Rust module system and trait-based extractors
- Familiarity with ExifTool's PrintConv concept
- Read [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) before making changes

## Testing Strategy

### Unit Tests
- Existing `tests/tag_kit_integration.rs` provides comprehensive coverage
- Tests compare tag_kit output against ExifTool reference implementations

### Integration Tests  
```bash
# Validate tag_kit extraction works
make codegen && cargo test tag_kit

# Ensure no regressions
cargo run --bin compare-with-exiftool test-images/*.jpg
```

### Manual Testing
1. Run `make codegen` - should complete without errors
2. Run `cargo check` - should compile cleanly  
3. Test specific manufacturer files with changed PrintConv implementations

## Success Criteria & Quality Gates

**Phase 1 Complete**:
- [ ] All source code references to inline functions updated to use tag_kit
- [ ] `cargo check` passes without compilation errors
- [ ] No change in `compare-with-exiftool` output for test images

**Phase 2 Complete**:
- [ ] Inline PrintConv configs removed from codegen
- [ ] Only tag_kit system generates PrintConv code
- [ ] `make precommit` passes completely

**Phase 3 Complete**:
- [ ] Integration tests pass
- [ ] ExifTool comparison shows no regressions
- [ ] Optional metrics show tag_kit usage

## Gotchas & Tribal Knowledge

### Current Dual System Architecture

**IMPORTANT**: Both tag_kit AND inline_printconv systems currently generate files in parallel. This was intentionally preserved during migration to avoid compilation errors. The next engineer needs to gradually migrate source code references before removing the inline system.

### Tag Kit Function Signatures

Tag kit uses a different API pattern:
```rust
// Tag Kit (new)
apply_print_conv(tag_id: u32, value: &TagValue, evaluator: &mut ExpressionEvaluator, errors: &mut Vec<String>, warnings: &mut Vec<String>) -> TagValue

// Inline (legacy)  
lookup_table_name(key: u8) -> Option<&'static str>
```

### TagKitExtractor Consolidation Logic

The TagKitExtractor has custom consolidation logic in `codegen/src/extractors/tag_kit.rs:47-137` that:
1. Calls `tag_kit.pl` once per table with temporary filenames
2. Consolidates all table results into single module file  
3. Cleans up temporary files

This was necessary because the perl script expects single table names but modules can have multiple tables.

### File Type Generation Issue

During migration, we discovered missing functions in `src/generated/file_types/file_type_lookup.rs`. The fix was to add legacy function aliases:
```rust
pub fn lookup_file_type_by_extension(extension: &str) -> Option<(Vec<&'static str>, &'static str)> {
    resolve_file_type(extension)
}
```

### Complex PrintConv Types

Tag kit handles three PrintConv types:
- **Simple**: Static HashMap lookups (most common)
- **Expression**: Perl expressions (need expression evaluator)  
- **Manual**: Complex functions requiring custom implementation

The current implementation warns for Expression and Manual types rather than failing.

### Testing with ExifTool Comparison

Use the Rust-based comparison tool rather than shell script:
```bash
cargo run --bin compare-with-exiftool image.jpg
# More reliable than scripts/compare-with-exiftool.sh
```

### Never Edit Generated Files

Everything in `src/generated/` is regenerated by `make codegen`. Changes must be made in:
- `codegen/config/*.json` - Configuration
- `codegen/src/extractors/*.rs` - Extraction logic
- `codegen/extractors/*.pl` - Perl extraction scripts

### CRITICAL: Git Submodule

The `third-party/exiftool` directory is a git submodule. Never run git commands directly on files in this directory. The codegen process may temporarily patch ExifTool files but reverts them automatically.