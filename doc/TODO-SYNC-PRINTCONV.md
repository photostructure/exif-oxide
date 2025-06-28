# TODO: Implement PrintConv Synchronization System

## Overview

This document provides discrete, actionable steps for implementing the PrintConv synchronization system designed in [`SYNC-PRINTCONV-DESIGN.md`](SYNC-PRINTCONV-DESIGN.md). Each phase is a standalone deliverable that can be assigned to different engineers.

## Current State Assessment

### What Exists:

- ✅ `scripts/extract_printconv.pl` - Extracts PrintConv from single modules
- ✅ `src/core/print_conv.rs` - Table-driven PrintConv system with ~100 variants
- ✅ `src/bin/exiftool_sync/extractors/printconv_analyzer.rs` - Pattern analysis
- ✅ `src/bin/exiftool_sync/extractors/printconv_generator.rs` - Code generation
- ✅ `src/bin/exiftool_sync/extractors/printconv_tables.rs` - Table extraction
- ✅ `src/bin/exiftool_sync/analyze_printconv_safety.rs` - Safety analyzer (IN PROGRESS)
- ✅ `emit_sync_issue()` function for tracking manual work needed

### What's Missing:

- ❌ `--all-modules` support in Perl script
- ❌ `sync-printconv` subcommand in main.rs
- ❌ Pattern matching tables (`src/tables/printconv_patterns.rs`)
- ❌ Generated lookup tables directory (`src/tables/generated/`)
- ❌ Integration with build system

## ✅ Phase 1: Enhance Perl Extraction Script (COMPLETED)

### Status: **COMPLETED** - June 26, 2025

### Objective:

Add `--all-modules` support to extract PrintConv from all ExifTool modules in one run.

### Implementation Results:

1. ✅ **Updated `scripts/extract_printconv.pl`** with `--all-modules` support
2. ✅ **Uses ExifTool's static module list** from `@Image::ExifTool::loadAllTables` (exactly as specified)
3. ✅ **Deterministic output** with canonical JSON, sorted arrays and hash keys
4. ✅ **Module count in metadata** (78 successful modules processed)
5. ✅ **Error handling** for missing modules (52 failed modules tracked)
6. ✅ **Comprehensive coverage** (11,057 total tags vs 2,698 from Canon alone)

### Key Results:

- **78 modules successfully processed** (expected since many @loadAllTables entries are virtual/embedded)
- **1,078 tags with PrintConv** extracted across all modules
- **11,057 total tags** processed (4x more than Canon alone)
- **Valid JSON output** with proper metadata structure
- **Failed modules properly tracked** for transparency (52 missing modules are expected)

### Testing Verified:

```bash
# Single module works (existing functionality)
perl scripts/extract_printconv.pl Image::ExifTool::Canon > test_canon.json ✅

# All modules works (new functionality)  
perl scripts/extract_printconv.pl --all-modules > test_all.json ✅

# Module count as expected (78 vs original estimate of ~200)
jq '.metadata.module_count' test_all.json  # Returns: 78 ✅

# Statistics show comprehensive extraction
jq '.statistics.tags_with_printconv' test_all.json  # Returns: 1,078 ✅
```

### Key Discovery:

Many entries in `@loadAllTables` don't exist as separate .pm files - they're embedded in other modules or are placeholders. This is normal ExifTool behavior. The 78 successfully loaded modules represent all actual modules containing PrintConv data.

### Deliverable:

✅ **COMPLETE**: Updated `scripts/extract_printconv.pl` with full `--all-modules` support ready for Phase 2

## ⚠️ ARCHITECTURAL CHANGE REQUIRED

### Issue Discovered: **DRY Violation in Current Architecture**

**Problem**: Current sync extracts expanded data from each tag, creating duplicate tables instead of preserving ExifTool's DRY shared lookup architecture.

**Impact**: Canon lens table (524 entries) extracted 25 times instead of once. No shared lookup tables generated.

**Required Fix**: Complete rewrite of extraction and sync to match ExifTool's architecture.

## ✅ Phase 2: Create PrintConv Pattern Tables (COMPLETED - NEEDS REVISION)

### Status: **COMPLETED** - June 26, 2025 (Architecture needs updating)

### Objective:

Create pattern matching tables to map Perl PrintConv patterns to Rust PrintConvId enum variants.

### Implementation Results:

1. ✅ **Created `src/tables/printconv_patterns.rs`** with comprehensive pattern tables:
   - `PERL_STRING_PATTERNS`: Maps 12+ common string expressions to PrintConvId 
   - `HASH_PATTERNS`: Maps 15+ normalized hash patterns to PrintConvId
   - `FUNCTION_PATTERNS`: Maps 10+ ExifTool functions to PrintConvId
   - `HASH_REF_PATTERNS`: Maps shared lookup table references to PrintConvId
   - `determine_printconv_id()`: Main pattern matching function
   - `normalize_hash_pattern()`: Hash normalization utility

2. ✅ **Updated `src/tables/mod.rs`** to include printconv_patterns module

3. ✅ **Added missing PrintConvId variants** to `src/core/print_conv.rs`:
   - `TrueFalse`, `EnableDisable` for binary toggles
   - `ImageQuality`, `TargetImageType`, `ImageFormat` for image metadata
   - `DriveMode` for camera settings
   - `Millimeters`, `Float1Decimal`, `Float2Decimal`, `RoundToInt`, `Integer`, `Hex` for formatting
   - `Fraction`, `Duration`, `ConvertBinary`, `UnixTime`, `GPSCoordinate` for functions

4. ✅ **Added phf dependency** to Cargo.toml for efficient pattern matching

### Key Features Implemented:

- **Comprehensive coverage**: Patterns for string expressions, hash lookups, function calls, and shared references
- **Deterministic matching**: Hash patterns normalized by sorting for consistent recognition
- **Integration ready**: Works with existing PrintConvId enum and print_conv system
- **Test coverage**: 9 comprehensive tests verify all pattern matching functionality
- **Performance optimized**: Uses PHF (Perfect Hash Functions) for O(1) lookups

### Testing Results:

```bash
# Pattern matching tests pass successfully
cargo test --lib printconv_patterns
running 9 tests
test tables::printconv_patterns::tests::test_determine_printconv_id_code_ref ... ok
test tables::printconv_patterns::tests::test_determine_printconv_id_hash_ref ... ok  
test tables::printconv_patterns::tests::test_determine_printconv_id_hash ... ok
test tables::printconv_patterns::tests::test_function_pattern_lookup ... ok
test tables::printconv_patterns::tests::test_hash_pattern_lookup ... ok
test tables::printconv_patterns::tests::test_hash_ref_pattern_lookup ... ok
test tables::printconv_patterns::tests::test_normalize_hash_pattern ... ok
test tables::printconv_patterns::tests::test_determine_printconv_id_string ... ok
test tables::printconv_patterns::tests::test_string_pattern_lookup ... ok

# All existing print_conv tests continue to pass (32 tests)
cargo test --lib print_conv  
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured
```

### Deliverable:

✅ **COMPLETE**: 
- New `src/tables/printconv_patterns.rs` with 50+ patterns mapped  
- Updated `src/tables/mod.rs` integration
- 9 comprehensive tests validating all functionality
- Ready for Phase 3 sync extractor to consume

## Phase 3: Rewrite PrintConv Sync Architecture (REQUIRED)

### Task Owner: Engineer familiar with exiftool_sync architecture

### Objective:

Completely rewrite sync to preserve ExifTool's DRY shared lookup architecture.

### Dependencies:

- Phase 1 completed (Perl script with `--all-modules`)
- Phase 2 completed (Pattern tables)

### Implementation Steps:

1. **Rewrite `scripts/extract_printconv.pl` for two-phase extraction**:
   - Phase 1: Extract module-level shared tables (`%canonLensTypes`)
   - Phase 2: Extract tag PrintConv references (not expanded data)

2. **Create new `src/bin/exiftool_sync/extractors/shared_tables_sync.rs`**:
   - Extract shared lookup tables from Phase 1 data
   - Generate single `canon_lens_types.rs`, `nikon_lens_types.rs` files
   - Handle deduplication and conflict detection

3. **Update `src/bin/exiftool_sync/extractors/printconv_sync.rs`**:

   - Process tag references instead of expanded data
   - Map `\%canonLensTypes` references to `PrintConvId::CanonLensTypes`
   - Remove duplicate table generation logic

4. **Update `src/core/print_conv.rs`**:
   - Add runtime support for shared lookup tables
   - Update `PrintConvId` enum with shared table variants
   - Implement lookup logic for `CANON_LENS_TYPES`, etc.

### Key Architectural Decisions:

- **Two-phase extraction**: Shared tables first, then tag references
- **Single source of truth**: One file per shared lookup table
- **DRY compliance**: Tag definitions reference shared tables, don't duplicate data
- **ExifTool alignment**: Match ExifTool's module-level variable architecture

### Testing:

```bash
# Test shared table extraction
cargo run --bin exiftool_sync extract shared-tables

# Test tag reference extraction  
cargo run --bin exiftool_sync extract printconv-sync

# Verify single shared lookup files generated
ls src/tables/generated/canon_lens_types.rs
ls src/tables/generated/nikon_lens_types.rs

# Test Canon lens name conversion
cargo run -- test-images/canon/canon_eos_r50v_01.jpg | grep LensType
```

### Deliverable:

- Rewritten two-phase extraction architecture
- Single shared lookup table files (no duplicates)
- Tag definitions that reference shared tables
- Canon lens names correctly converted to human-readable strings

## Phase 4: Build System Integration (Simple Task)

### Task Owner: Any engineer

### Objective:

Integrate PrintConv sync with the build system and make it part of `extract-all`.

### Dependencies:

- Phase 3 completed (Sync extractor implemented)

### Implementation Steps:

1. **Update `Makefile`**:

   ```makefile
   # Add new target
   sync-printconv:
   	cargo run --bin exiftool_sync extract printconv-sync

   # Update sync target to include printconv
   sync: extract-all

   # Or if sync has specific targets:
   sync: sync-tags sync-printconv sync-other
   ```

2. **Already integrated with `extract-all`**:
   - Phase 3 adds to components list
   - Will run automatically with `make sync`

### Testing:

```bash
# Test integration
make sync-printconv

# Test as part of extract-all
cargo run --bin exiftool_sync extract-all
```

### Deliverable:

- Updated Makefile
- PrintConv sync runs as part of standard sync workflow

## Phase 5: Runtime Integration

### Task Owner: Engineer familiar with print_conv.rs

### Objective:

Connect generated lookup tables to the runtime PrintConv system.

### Dependencies:

- Phase 3 completed (Generated tables exist)
- Phase 2 completed (Pattern tables for mapping)

### Implementation Steps:

1. **Create conditional imports in `src/core/print_conv.rs`**:

   ```rust
   // At top of file, after existing imports:

   // Generated lookup tables (may not exist on first build)
   #[cfg(feature = "generated-tables")]
   mod generated_imports {
       pub use crate::tables::generated::canon_lens_types::CANON_LENS_TYPES;
       pub use crate::tables::generated::nikon_lens_types::NIKON_LENS_TYPES;
       // Add more as they are generated
   }

   #[cfg(not(feature = "generated-tables"))]
   mod generated_imports {
       use phf::phf_map;
       // Stub implementations to allow compilation
       pub static CANON_LENS_TYPES: phf::Map<u16, &'static str> = phf_map! {};
       pub static NIKON_LENS_TYPES: phf::Map<u16, &'static str> = phf_map! {};
   }

   use generated_imports::*;
   ```

2. **Update `apply_print_conv()` implementation**:

   ```rust
   // In the match statement, add new cases:
   PrintConvId::CanonLensType => {
       if let Some(lens_id) = value.as_u16() {
           if let Some(lens_name) = CANON_LENS_TYPES.get(&lens_id) {
               lens_name.to_string()
           } else {
               format!("Unknown ({})", lens_id)
           }
       } else {
           format!("{}", value)
       }
   },

   PrintConvId::NikonLensType => {
       // Similar implementation
   },
   ```

3. **Update `Cargo.toml` to add feature flag**:

   ```toml
   [features]
   default = []
   generated-tables = []

   # After first sync, enable by default:
   # default = ["generated-tables"]
   ```

### Handling Missing Generated Files:

Option 1 - **Build script approach**:

```rust
// In build.rs:
fn main() {
    // Check if generated tables exist
    let generated_dir = Path::new("src/tables/generated");
    if generated_dir.exists() && generated_dir.read_dir()?.count() > 0 {
        println!("cargo:rustc-cfg=feature=\"generated-tables\"");
    }
}
```

Option 2 - **Stub files approach**:
Create minimal stub files in `src/tables/generated/` with empty tables to allow initial compilation.

### Testing:

```bash
# Build without generated tables
cargo build

# Run sync to generate tables
make sync

# Build with generated tables
cargo build --features generated-tables

# Test PrintConv functionality
cargo test print_conv
```

### Deliverable:

- Updated `src/core/print_conv.rs` with conditional imports
- New PrintConvId implementations
- Build system handles missing generated files gracefully

## Phase 6: Testing & Validation

### Task Owner: QA engineer or developer

### Objective:

Comprehensive testing of the PrintConv sync system.

### Dependencies:

- All previous phases completed

### Test Suite:

1. **Unit Tests** (`src/bin/exiftool_sync/extractors/printconv_sync.rs`):

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_idempotent_sync() {
           // Run sync twice, verify same output
       }

       #[test]
       fn test_pattern_matching() {
           // Test PrintConvId::from_extraction
       }

       #[test]
       fn test_cache_detection() {
           // Verify cache prevents regeneration
       }
   }
   ```

2. **Integration Tests** (`tests/printconv_sync_test.rs`):

   ```rust
   #[test]
   fn test_end_to_end_sync() {
       // Run full sync pipeline
       // Verify generated files
       // Test PrintConv output
   }

   #[test]
   fn test_exiftool_compatibility() {
       // Compare output with ExifTool
       // for known test images
   }
   ```

3. **Performance Benchmarks**:
   ```bash
   # Add to benches/printconv_bench.rs
   cargo bench --bench printconv_bench
   ```

### Validation Checklist:

- [ ] All existing tests pass: `cargo test`
- [ ] Generated code compiles: `cargo build --features generated-tables`
- [ ] Idempotent: Running sync twice produces identical output
- [ ] Sync issues tracked: Check `sync-todos.jsonl` populated correctly
- [ ] PrintConv output matches ExifTool for test images
- [ ] Performance regression < 10%

### Deliverable:

- Complete test suite
- Validation report
- Performance comparison

## Phase 7: Documentation & Monitoring

### Task Owner: Any engineer

### Objective:

Document the system and provide monitoring tools.

### Implementation:

1. **Update documentation**:

   - Remove "Unknown Aspects" section from this file (all resolved)
   - Update `SYNC-PRINTCONV-DESIGN.md` with implementation results
   - Add examples of adding new PrintConv patterns

2. **Create monitoring dashboard** (`sync-report.md`):

   ```markdown
   # PrintConv Sync Status Report

   Generated: [date]

   ## Coverage Statistics

   - Total ExifTool tags: X
   - Tags with PrintConv: Y (Z%)
   - Successfully mapped: A (B%)
   - Unmapped patterns: C

   ## Top Unmapped Patterns

   1. Pattern X (used by N tags)
   2. Pattern Y (used by M tags)

   ## Next Steps

   - Add PrintConvId::NewPattern for...
   ```

3. **Automate reporting**:
   ```rust
   // Add to printconv_sync.rs
   fn generate_report(&self, data: &ExtractedData) -> Result<(), String> {
       // Generate markdown report
       // Include statistics
       // List top unmapped patterns
   }
   ```

### Deliverable:

- Updated documentation
- Automated reporting system
- Clear guide for adding new patterns

## Summary

### Resolved Unknowns:

1. ✅ **Module Discovery**: ExifTool uses static list `@loadAllTables`
2. ✅ **Update Strategy**: Generate new files, preserve manual edits via pattern tables
3. ✅ **Build Order**: Use feature flags or stub files
4. ✅ **Safety Analysis**: Separate tool handles PrintConv type analysis

### Key Innovation:

Using `emit_sync_issue()` to track unmapped patterns creates a feedback loop for continuous improvement.

### Success Metrics:

- Zero manual work lost during sync
- 95%+ PrintConv patterns mapped automatically
- Clear path for handling the remaining 5%
- Single command (`make sync`) updates everything

### Timeline:

- Phase 1-2: Can be done in parallel by different engineers (1-2 days each)
- Phase 3: Depends on 1-2 (2-3 days)
- Phase 4-5: Can be done in parallel after 3 (1 day each)
- Phase 6-7: Final validation and documentation (2-3 days)

**Total: 1-2 weeks with parallel work**
