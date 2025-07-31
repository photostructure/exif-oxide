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

- **MakerNotes Bypass**: Canon processing completely skips the signature detection path in `process_maker_notes_with_signature_detection()` - debug logs show no "Detected Canon camera" messages
- **Tag Kit vs Main Table**: Generated tag kit extracts many Canon tags successfully, but Canon Main table processing never executes for position-based tags like FileNumber
- **PrintConv Not Applied**: CanonModelID shows raw value 2147484294 instead of "EOS Rebel T3i" because PrintConv evaluation is skipped
- **Generated Code Works**: FileNumber definition exists correctly in `src/generated/Canon_pm/tag_kit/interop.rs:758-778` with proper PrintConv, but extraction never triggers

### Foundation Documents

- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` lines 186-400 (Main table definition)
- **Generated Code**: `src/generated/Canon_pm/tag_kit/interop.rs` (FileNumber tag definition with PrintConv)
- **Processing Logic**: `src/implementations/canon/mod.rs` (`process_canon_makernotes()` function - never called)
- **IFD Router**: `src/exif/ifd.rs:147-150` (signature detection fix that doesn't execute)

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD structure, ExifTool Canon.pm module organization
- **Setup required**: Test image `test-images/canon/eos_rebel_t3i.jpg` with FileNumber=1007632

## Work Completed

- ✅ **Synthetic Tag ID Collision Fixed** → hash-based generation prevents Canon tag crashes
- ✅ **Root Cause Identified** → Canon Main table processing bypassed, tag kit extracts other Canon tags successfully  
- ✅ **Generated Code Verified** → FileNumber definition with PrintConv exists correctly in codegen output
- ✅ **Signature Detection Logic Added** → `process_maker_notes_with_signature_detection()` modified but not executed

## Remaining Tasks

### 1. Task: Ensure process_canon_makernotes() gets called for Canon cameras

**Success Criteria**: Debug log shows "Detected Canon camera, calling Canon-specific MakerNotes processing" when processing Canon image
**Approach**: Find actual MakerNotes processing path that bypasses signature detection, modify to call Canon-specific handler
**Dependencies**: None

**Success Patterns**:
- ✅ `env RUST_LOG=debug cargo run -- test-images/canon/eos_rebel_t3i.jpg 2>&1 | grep "Detected Canon camera"` shows detection log
- ✅ `process_canon_makernotes()` function executes without errors
- ✅ Canon Main table processing debug logs appear

### 2. Task: Verify Canon Main table position 8 extracts FileNumber value

**Success Criteria**: Raw value 1007632 extracted from Canon Main table position 8
**Approach**: Add debug logging to Canon Main table processing, verify position 8 contains expected raw value
**Dependencies**: Task 1 complete

**Success Patterns**:
- ✅ Debug log shows "Canon Main table position 8: 1007632" or similar raw value extraction
- ✅ Canon Main table binary data correctly parsed as 32-bit integers
- ✅ Position-to-tag mapping correctly identifies position 8 as FileNumber

### 3. Task: Apply FileNumber PrintConv expression for human-readable output

**Success Criteria**: `cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep FileNumber` shows `"Canon:FileNumber": "100-7632"`
**Approach**: Ensure PrintConv expression `$_=$val,s/(\d+)(\d{4})/$1-$2/,$_` executes during Canon tag processing
**Dependencies**: Task 2 complete

**Success Patterns**:  
- ✅ Raw value 1007632 → formatted output "100-7632"
- ✅ FileNumber appears in default output without special flags
- ✅ ExifTool comparison shows identical FileNumber output

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