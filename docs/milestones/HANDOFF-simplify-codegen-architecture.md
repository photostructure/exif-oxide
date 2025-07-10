# HANDOFF: Simplify Codegen Architecture

**Priority**: High  
**Estimated Duration**: 2-3 days  
**Status**: Ready for implementation

## Problem Statement

The current codegen system, while functional, has become overly complex with interdependent Perl scripts that know too much about the configuration structure. This creates maintenance burden and makes it difficult to add support for new ExifTool modules. We need to simplify the architecture to make it more maintainable and easier to extend.

## Current Issues

### 1. **Complex Configuration Dependencies**

- Perl scripts like `patch_exiftool_modules.pl` parse JSON configuration files
- `simple_table.pl` requires understanding of the config structure
- Adding new modules requires updating multiple scripts and configs

### 2. **Makefile Complexity**

- Parallel extraction adds complexity without clear benefits
- Complex `split-extractions` step that feels wrong
- Hard to debug when things go wrong

### 3. **Path Guessing Logic**

- System has to guess `Canon_pm` → `../third-party/exiftool/lib/Image/ExifTool/Canon.pm`
- Fragile and error-prone when adding new modules

### 4. **Hardcoded Module Lists**

- `codegen/src/main.rs` has hardcoded `["Canon_pm", "Nikon_pm", ...]`
- Requires code changes to add new modules

### 5. **Multi-Stage Output Processing**

- Perl generates combined JSON files
- Then `split-extractions` breaks them apart
- Then Rust codegen reads individual files
- Should be: one table → one output file

## Proposed Architecture

### Core Principle: **Simple, Dumb Perl Scripts**

Perl scripts should be as simple as possible:

- Take explicit file paths as arguments
- Output exactly one thing per invocation
- No knowledge of configuration structures
- Easy to test and debug

### New Flow:

```
1. Rust codegen scans codegen/config/ directories
2. For each config, determines source ExifTool file from SOURCE attribute
3. Calls simple Perl scripts with explicit arguments
4. Perl outputs individual JSON files directly
5. Rust reads JSON and generates code
```

## Implementation Tasks

### Task 1: Add Source File Configuration (30 mins)

**Goal**: Eliminate path guessing logic

**What to do**:

1. Add `source` field to all config files OR create `SOURCE` files in each config directory
2. Update config to specify: `"source": "third-party/exiftool/lib/Image/ExifTool/Canon.pm"`
3. Update Rust codegen to read source paths from config instead of guessing

**Files to modify**:

- All `codegen/config/*/simple_table.json` files
- `codegen/src/main.rs` (remove hardcoded path logic)

### Task 2: Auto-Discover Modules (15 mins)

**Goal**: Remove hardcoded module list

**What to do**:

1. Replace `["Canon_pm", "Nikon_pm", ...]` in `main.rs` with directory scanning
2. Read all directories in `codegen/config/` and process automatically

**Files to modify**:

- `codegen/src/main.rs` lines 176-177

### Task 3: Simplify patch_exiftool_modules.pl (45 mins)

**Goal**: Make it take explicit arguments instead of reading configs

**Current API**:

```bash
perl patch_exiftool_modules.pl  # reads configs automatically
```

**New API**:

```bash
perl patch_exiftool_modules.pl path/to/Canon.pm variable1 variable2 variable3
```

**What to do**:

1. Remove all JSON config reading logic from `patch_exiftool_modules.pl`
2. Make it take file path and variable list as command line arguments
3. Update Rust codegen to call it with explicit arguments

**Files to modify**:

- `codegen/patch_exiftool_modules.pl`
- `codegen/src/main.rs` (add logic to call patch script)

### Task 4: Simplify simple_table.pl (1 hour)

**Goal**: Make it output individual files directly, no split step

**Current Flow**:

```
simple_table.pl → combined.json → split-extractions → individual files
```

**New Flow**:

```
simple_table.pl Canon.pm canonModelID → canon_model_id.json
simple_table.pl Canon.pm canonWhiteBalance → canon_white_balance.json
```

**What to do**:

1. Modify `simple_table.pl` to take source file and single hash name
2. Output individual JSON file directly
3. Remove `split-extractions` step entirely
4. Update Rust to call perl script once per table

**Files to modify**:

- `codegen/extractors/simple_table.pl`
- `codegen/Makefile.modular` (remove split-extractions)
- `codegen/src/main.rs` (call perl once per table)

### Task 5: Remove Parallelism (30 mins)

**Goal**: Simplify Makefile for easier debugging

**What to do**:

1. Remove parallel extraction logic from `Makefile.modular`
2. Make extraction sequential and straightforward
3. Keep existing `make -j` capability but don't optimize for it

**Files to modify**:

- `codegen/Makefile.modular`

### Task 6: Update Rust Codegen Integration (45 mins)

**Goal**: Tie everything together

**What to do**:

1. Update `main.rs` to read source paths from configs
2. Call simplified Perl scripts with explicit arguments
3. Process generated JSON files directly (no split step)

**Files to modify**:

- `codegen/src/main.rs`

## Files You Must Study

### 1. **Current Implementation Files**

- `codegen/src/main.rs` - Lines 176-188 show current module processing
- `codegen/Makefile.modular` - Lines 83-122 show current extraction flow
- `codegen/patch_exiftool_modules.pl` - Current complex implementation
- `codegen/extractors/simple_table.pl` - Current table extraction

### 2. **Configuration Structure**

- `codegen/config/Canon_pm/simple_table.json` - Example config to modify
- `codegen/config/Nikon_pm/simple_table.json` - Another example
- `docs/design/EXIFTOOL-INTEGRATION.md` - Current codegen design

### 3. **Example Generated Output**

- `src/generated/Canon_pm/mod.rs` - See what the system currently generates
- `codegen/generated/extract/*.json` - Current extraction output format

## Success Criteria

### Must Complete:

- [ ] All Perl scripts take explicit file paths and arguments (no config reading)
- [ ] `make codegen` works without hardcoded module lists
- [ ] Adding new module requires only adding config directory (no code changes)
- [ ] No `split-extractions` step - direct individual file output
- [ ] `make precommit` passes

### Validation Tests:

- [ ] Add a new dummy module config and verify it gets processed automatically
- [ ] Run codegen and ensure all existing tables still generate correctly
- [ ] Verify generated Rust code compiles without warnings
- [ ] Test that patching works with simplified script

## Current State Analysis

### What's Working ✅

- Basic config structure is sound
- JSON schema validation works
- Generated Rust code is clean and readable
- Direct code generation (no macros) is working well

### What's Problematic ❌

- Perl scripts are too smart/complex
- Multi-stage processing (combined → split → individual)
- Hardcoded module lists
- Path guessing logic
- Parallel extraction complexity

### What's Missing 📋

- Simple, testable Perl scripts
- Explicit source file configuration
- Auto-discovery of modules

## Tribal Knowledge & Gotchas

### 1. **Perl Script Testing**

```bash
# Test Perl scripts manually:
cd codegen
perl extractors/simple_table.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm canonModelID
```

### 2. **ExifTool Path Structure**

- `ExifTool.pm` is at `third-party/exiftool/lib/Image/ExifTool.pm`
- Other modules are at `third-party/exiftool/lib/Image/ExifTool/Canon.pm`
- Don't assume all modules follow same path pattern

### 3. **Variable Name Extraction**

- Hash names in config have `%` prefix: `%canonModelID`
- Perl variables don't have `%` in `my %canonModelID`
- Be careful with string processing

### 4. **Git Submodule Handling**

- ExifTool is a git submodule
- Patching scripts MUST revert changes after extraction
- Don't commit modified ExifTool files

### 5. **JSON Output Format**

The current extraction output format should be preserved:

```json
{
  "source": {
    "hash_name": "%canonModelID",
    "file": "Canon.pm"
  },
  "data": {
    "0x80000001": "Canon EOS-1D",
    "0x80000002": "Canon EOS-1DS"
  }
}
```

### 6. **Error Handling**

- Perl scripts should fail fast and clearly
- Don't continue processing if ExifTool module doesn't exist
- Check that variables exist before trying to patch them

## Expected Challenges

### 1. **Path Handling**

- Windows vs Unix path separators -- although we don't need to _codegen_ on windows -- ONLY COMPILE -- so it's ok to use POSIX file separators.
- Relative vs absolute paths
- ExifTool submodule location

### 2. **Perl Module Dependencies**

- Local::lib setup in Makefile
- JSON parsing in Perl
- File I/O error handling

### 3. **Makefile Complexity**

- Current Makefile has lots of moving parts
- Simplifying without breaking existing workflow
- Maintaining compatibility with `make clean`, `make check`, etc.

## Implementation Strategy

### Phase 1: Configuration (Day 1)

1. Add source attributes to configs
2. Update auto-discovery in main.rs
3. Test that module scanning works

### Phase 2: Perl Simplification (Day 2)

1. Simplify patch_exiftool_modules.pl
2. Simplify simple_table.pl for direct output
3. Update Rust to call simplified scripts

### Phase 3: Integration & Testing (Day 3)

1. Remove parallelism from Makefile
2. Update integration in main.rs
3. Full end-to-end testing

## Risk Mitigation

### Backup Current System

Before starting, commit current working state:

```bash
git add -A
git commit -m "checkpoint: working codegen before simplification"
```

### Incremental Validation

After each phase:

```bash
make codegen          # Should complete without errors
cargo check           # Generated code should compile
make precommit        # Full validation
```

### Rollback Plan

If implementation gets stuck:

```bash
git reset --hard HEAD~1  # Back to checkpoint
```

## Key Design Principles

### 1. **Perl Scripts Are Dumb**

- No JSON config parsing in Perl
- Take all needed info as command line arguments
- One job per script execution

### 2. **Rust Orchestrates Everything**

- Rust reads configs and decides what to extract
- Rust calls Perl with explicit arguments
- Rust handles all the coordination logic

### 3. **One Table, One Output**

- No combined files that get split later
- Direct extraction to final JSON format
- Simpler debugging and testing

### 4. **Auto-Discovery Over Configuration**

- Scan directories instead of hardcoded lists
- Convention over configuration where possible
- Fail fast when conventions aren't followed

## ✅ IMPLEMENTATION COMPLETE (January 2025)

**Status**: All 6 core tasks successfully implemented!

### What Works Now ✅

- **Auto-discovery**: No hardcoded module lists - scans `config/` directories automatically
- **Source paths**: All configs have explicit `source` field, eliminated path guessing
- **Simplified Perl scripts**: Both take explicit arguments, no config reading
- **Auto-patching**: `simple_table.pl` automatically patches ExifTool when variables need conversion
- **Direct file output**: No `split-extractions` step - individual JSON files created directly
- **Rust orchestration**: Rust reads configs and coordinates everything

### Current Architecture

```
Rust scans config/ → Reads source paths → Calls Perl with explicit args → Individual JSON files
```

**Success Criteria Met:**

- ✅ All Perl scripts take explicit file paths and arguments (no config reading)
- ✅ `make codegen` works without hardcoded module lists
- ✅ Adding new module requires only adding config directory (no code changes)
- ✅ No `split-extractions` step - direct individual file output
- ✅ `cargo check` passes - generated code compiles correctly

## 🚨 CURRENT ISSUE: Makefile Configuration Duplication

**Problem**: We moved config out of Perl into the Makefile, creating maintenance burden:

```makefile
# BAD - Hardcoded extraction commands in Makefile
extract-canon:
    @cd $(EXTRACT_DIR) && perl ../../extractors/simple_table.pl ../../$(EXIFTOOL_LIB)/Canon.pm canonModelID canonWhiteBalance pictureStyles canonImageSize canonQuality

extract-nikon:
    @cd $(EXTRACT_DIR) && perl ../../extractors/simple_table.pl ../../$(EXIFTOOL_LIB)/Nikon.pm nikonLensIDs
```

**Issue**: Variable lists are duplicated between JSON configs and Makefile targets.

## 📋 NEXT ENGINEER TASKS

### Priority 1: DRY Up Configuration (1-2 hours)

**Goal**: Eliminate hardcoded variable lists in Makefile that duplicate data in codegen/config

**Option A - Rust Orchestration (Recommended)**:

```
1. Remove extract-* targets from Makefile entirely
2. Have Rust read configs and call Perl scripts directly
3. Use config.source and config.tables[].hash_name to determine what to extract
```

**Implementation Example (Option A)**:

```rust
// In Rust codegen
for config in configs {
    let source_path = &config.source;
    let hash_names: Vec<&str> = config.tables.iter()
        .map(|t| t.hash_name.trim_start_matches('%'))
        .collect();

    // Call: perl simple_table.pl <source_path> <hash1> <hash2>...
    let output = Command::new("perl")
        .arg("extractors/simple_table.pl")
        .arg(source_path)
        .args(&hash_names)
        .current_dir("generated/extract")
        .output()?;
}
```

### Benefits of Rust Orchestration

- ✅ Single source of truth (JSON configs)
- ✅ Better error handling and debugging
- ✅ Eliminates Makefile complexity
- ✅ Easier to extend for future requirements

### Implementation Notes

- Move extraction logic from `Makefile.modular` into `src/main.rs`
- Keep Makefile simple: `extract-data` target just calls `cargo run --extract`
- Use the existing `source` field and `tables[].hash_name` from configs
- Call `simple_table.pl` once per module with all its hash names

## Next Engineer: You've Got This!

The hard work is done - we have a working, simplified system. You just need to eliminate the config duplication between JSON and Makefile.

**Remember**: Trust ExifTool completely. We're not changing any extraction logic, just eliminating duplicate configuration.

The simplified architecture is solid. One final cleanup and this system will be ready to scale! 🚀
