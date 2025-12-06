# P13b - Canon FileNumber Tag Implementation

## Project Overview

- **Goal**: Implement Canon FileNumber tag extraction showing "100-7632" format instead of missing tag
- **Problem**: Canon Main table processing bypassed - FileNumber at position 8 never extracted despite proper codegen
- **Constraints**: Must follow Trust ExifTool principle, use existing Canon Main table infrastructure

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

- **Canon MakerNotes Processing**: Canon cameras store proprietary metadata in MakerNotes tag (0x927c) using Canon Main table with 200+ indexed positions
- **Tag Kit System**: Generated Rust code from ExifTool's Canon.pm that handles tag definitions, PrintConv expressions, and value conversions
- **IFD Processing Pipeline**: TIFF directory parser that routes MakerNotes to manufacturer-specific processors based on Make field detection
- **PrintConv Expression System**: Converts raw binary values to human-readable strings using Perl-to-Rust translated expressions

### Key Concepts & Domain Knowledge

- **Canon Main Table**: Array-indexed table where position 8 = FileNumber, containing raw 32-bit values that need PrintConv formatting
- **PrintConv Expression**: Perl regex `$_=$val,s/(\d+)(\d{4})/$1-$2/,$_` converts "1007632" → "100-7632" format
- **Signature Detection**: MakerNotes processing routes Canon cameras to `process_canon_makernotes()` based on Make field
- **Synthetic Tag IDs**: Generated unique IDs for Canon tags to avoid collisions in unified tag space

### Surprising Context

- **CRITICAL CLI BUG DISCOVERED**: `--json` flag was being parsed as tag filter "-json", causing ALL EXIF tags to be filtered out
- **Canon MakerNotes Actually Work**: With CLI bug fixed, Canon processing extracts tags correctly, CanonModelID shows "EOS Rebel T3i / 600D / Kiss X5"
- **FileNumber Files Exist**: Test files eos_rebel_xti.cr2 (121-1500), eos_d30_03.jpg (208-0828), powershot_d20.jpg (108-0032) have FileNumber data
- **Original Test File Missing FileNumber**: eos_rebel_t3i.jpg doesn't actually contain FileNumber data - need different test file
- **Tag Kit vs Main Table**: Generated tag kit extracts many Canon tags successfully, but Canon Main table processing never executes for position-based tags like FileNumber
- **Generated Code Works**: FileNumber definition exists correctly in `src/generated/Canon_pm/tag_kit/interop.rs:758-778` with proper PrintConv, but extraction never triggers

### Foundation Documents

- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` lines 186-400 (Main table definition)
- **Generated Code**: `src/generated/Canon_pm/tag_kit/interop.rs` (FileNumber tag definition with PrintConv)
- **Processing Logic**: `src/implementations/canon/mod.rs` (`process_canon_makernotes()` function - never called)
- **IFD Router**: `src/exif/ifd.rs:147-150` (signature detection fix that doesn't execute)

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD structure, ExifTool Canon.pm module organization
- **Setup required**: Test images with actual FileNumber data:
  - `test-images/canon/eos_rebel_xti.cr2` → FileNumber: "121-1500"  
  - `test-images/canon/eos_d30_03.jpg` → FileNumber: "208-0828"
  - `test-images/canon/powershot_d20.jpg` → FileNumber: "108-0032"
- **CLI USAGE NOTE**: Use `cargo run -- filename` NOT `cargo run -- --json filename` (CLI parsing bug with --json flag)

## Work Completed

- ✅ **Synthetic Tag ID Collision Fixed** → hash-based generation prevents Canon tag crashes
- ✅ **Root Cause Identified** → Canon Main table processing bypassed, tag kit extracts other Canon tags successfully  
- ✅ **Generated Code Verified** → FileNumber definition with PrintConv exists correctly in codegen output
- ✅ **Signature Detection Logic Added** → `process_maker_notes_with_signature_detection()` modified but not executed
- ✅ **CRITICAL CLI PARSING BUG FIXED** → `--json` flag treated as tag filter "-json", blocking ALL EXIF extraction
- ✅ **Canon MakerNotes Processing Confirmed Working** → CanonModelID shows proper "EOS Rebel T3i / 600D / Kiss X5" instead of raw 2147484294
- ✅ **Test Files With FileNumber Located** → Found Canon files with actual FileNumber data in both test directories

## Remaining Tasks

### PRIORITY 1: Fix CLI --json parsing bug (CRITICAL INFRASTRUCTURE ISSUE)
**Status**: IDENTIFIED BUT NOT FIXED  
**Issue**: `--json` flag parsed as tag filter "-json", blocks ALL EXIF extraction  
**Evidence**: `FilterOptions { requested_tags: ["-json"]` instead of output format selection  
**Impact**: Affects all users trying to get JSON output - this is a critical user-facing bug

### 1. Task: Research ExifTool Canon FileNumber implementation  
**Status**: READY TO START  
**Success Criteria**: Understand exact ExifTool Canon.pm logic for FileNumber extraction from Main table position 8  
**Method**: Use ExifTool source code research to find Canon FileNumber PrintConv and extraction logic  
**Test File**: Use `test-images/canon/eos_rebel_xti.cr2` which has FileNumber "121-1500"

### 2. Task: Debug Canon Main table processing execution path
**Status**: BLOCKED BY TASK 1  
**Success Criteria**: Understand why FileNumber extraction doesn't run when other Canon tags work  
**Approach**: Compare working Canon tag extraction vs FileNumber to find missing execution path  
**Evidence Needed**: Debug logs showing Canon Main table position 8 raw value extraction

### 3. Task: Implement FileNumber extraction following Trust ExifTool principle
**Status**: BLOCKED BY TASKS 1-2  
**Success Criteria**: `cargo run -- test-images/canon/eos_rebel_xti.cr2 | grep FileNumber` shows `"MakerNotes:FileNumber": "121-1500"`  
**Method**: Copy ExifTool logic exactly, including PrintConv expression and array indexing

## Implementation Guidance

**Recommended patterns**:
- Debug MakerNotes tag (0x927c) processing flow to find actual execution path
- Use existing Canon Main table infrastructure in `src/implementations/canon/mod.rs`
- Follow PrintConv expression evaluation patterns from working Canon tags

**Tools to leverage**:
- `env RUST_LOG=debug` for tracing execution paths
- `grep -E "(Canon|MakerNotes|0x927c)"` for finding relevant processing code
- Existing Canon tag processing as reference implementation

**ExifTool translation notes**:
- Canon.pm Main table uses array indexing: `8 => 'FileNumber'`
- PrintConv exactly as specified: `$_=$val,s/(\d+)(\d{4})/$1-$2/,$_`
- Trust ExifTool's approach completely - don't attempt improvements

## Integration Requirements

- [x] **Activation**: FileNumber extraction enabled by default for Canon cameras
- [x] **Consumption**: FileNumber appears in standard JSON output without special flags  
- [x] **Measurement**: Can verify with `grep FileNumber` in output
- [x] **Cleanup**: No obsolete code - using existing Canon processing infrastructure

## Testing

- **Unit**: Test Canon Main table position 8 extraction with known raw value
- **Integration**: Verify end-to-end FileNumber extraction from Canon test image
- **Manual check**: Run `cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep FileNumber` confirms "100-7632" output

## Definition of Done

- [ ] `env RUST_LOG=debug cargo run -- test-images/canon/eos_rebel_t3i.jpg 2>&1 | grep "Detected Canon camera"` shows Canon detection
- [ ] `cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep FileNumber` shows `"Canon:FileNumber": "100-7632"`  
- [ ] `make precommit` clean
- [ ] ExifTool comparison shows identical FileNumber output