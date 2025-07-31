# Technical Project Plan: Nikon Required Tags Implementation

## Project Overview

- **Goal**: Implement core decryption algorithms and model-specific processing to extract all required tags from Nikon JPEG and NEF files
- **Problem**: Infrastructure 85% complete but missing actual decryption algorithms - encrypted sections detected but not processed
- **Constraints**: Must translate ExifTool's ProcessNikonEncrypted exactly, focus on mainstream camera models

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

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **Nikon Implementation Architecture**: 14 implementation files covering detection, encryption, IFD processing, AF systems, lens database (618 entries) - following proven Canon module pattern with complete modular organization
- **Tag Kit System**: 17+ generated files provide comprehensive tag definitions with embedded PrintConv implementations, automatically extracted from ExifTool's 135 Nikon tag tables
- **Encryption Framework**: Complete key management system with pre-scan capability, encrypted section detection, and validation - but missing actual decryption algorithms (the core gap)
- **Subdirectory Processing**: Integration with generic subdirectory processing system enables complex binary data extraction once decryption is implemented

### Key Concepts & Domain Knowledge

- **Nikon Encryption System**: Uses serial number (tag 0x001d) + shutter count (tag 0x00a7) as XOR decryption keys, with model-specific constants in lookup tables (`$xlat[0]` and `$xlat[1]`)
- **Two-Pass Processing**: Pre-scan extracts keys, then main processing decrypts and extracts tags - critical architectural requirement that's already implemented
- **Model-Specific Processing**: 30+ ShotInfo table variants for different camera models, each with unique binary data structures and offsets
- **Format Detection**: Three Nikon format versions (Format1/2/3) with different offset calculation schemes, already implemented in `offset_schemes.rs`

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Extensive Infrastructure Already Built**: Original TPP assumed 0% completion, but analysis shows 85% infrastructure complete - remaining work is focused on specific decryption algorithms
- **ProcessNikonEncrypted is Small**: The core missing piece is just 120 lines in ExifTool (lines 13892-14011), plus 35-line Decrypt function (lines 13554-13588)
- **Encrypted Section Detection Works**: Current implementation correctly identifies encrypted tags (0x0088, 0x0091, 0x0097, 0x0098, etc.) and validates keys - but doesn't decrypt
- **Tag Kit Migration Complete**: Unlike original TPP assumptions, the tag kit system is fully operational with comprehensive PrintConv implementations generated

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Current tag kit system, [ARCHITECTURE.md](../ARCHITECTURE.md) - Overall system design
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` lines 13554-13588 (Decrypt), 13892-14011 (ProcessNikonEncrypted), 14144-14172 (ProcessNikon)
- **Start here**: `src/implementations/nikon/mod.rs` (main coordinator), `src/implementations/nikon/encryption.rs` (key management framework)

### Prerequisites

- **Knowledge assumed**: Understanding of XOR encryption, binary data processing, Rust borrowing/ownership for data processing
- **Setup required**: Working ExifTool installation for comparison testing, Nikon test images with known serial numbers

**Context Quality Check**: Can a new engineer understand WHY this approach is needed after reading this section?

## Work Completed

- ✅ **Modular Architecture** → Built 14 implementation files following Canon pattern with complete separation of concerns
- ✅ **Tag Kit Migration** → 17+ generated files with comprehensive tag definitions and PrintConv implementations 
- ✅ **Encryption Framework** → Complete key management, pre-scan logic, encrypted section detection and validation
- ✅ **Format Detection** → Three Nikon format versions with proper offset calculation schemes implemented
- ✅ **Lens Database** → 618-entry lens ID lookup system automatically generated from ExifTool source
- ✅ **AF Processing** → Complete AF point systems for 105, 135, 153 point grids with lookup tables
- ✅ **System Integration** → Proper integration into broader exif-oxide architecture with subdirectory processing
- ✅ **Generated Lookup Tables** → Extensive codegen-produced tables for compression types, metering modes, focus modes
- ✅ **Core Decryption Algorithms** → Complete implementation of ExifTool's Decrypt() and ProcessNikonEncrypted functions with XLAT lookup tables, XOR algorithm, and state management (71 tests passing)
- ✅ **Model-Specific Processing** → Complete ProcessBinaryData dispatch for D850, Z8, Z9, Z7 cameras with encrypted section processing (ShotInfo, LensData, ColorBalance) and automatic integration (79 tests passing)

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Implement Core Decryption Algorithms ✅ **COMPLETED**

**Success Criteria**: `cargo t nikon_decrypt_test` passes, ExifTool comparison shows identical decryption for test files
**Approach**: Translate ExifTool's Decrypt() function (lines 13554-13588) and ProcessNikonEncrypted (lines 13892-14011) to Rust
**Dependencies**: None - encryption framework already provides key management

**Success Patterns**:

- ✅ XOR decryption algorithm matches ExifTool's `$xlat[0]` and `$xlat[1]` lookup tables exactly
- ✅ Decryption state management (`$ci0`, `$cj0`, `$ck0` variables) properly implemented  
- ✅ Encrypted sections (ShotInfo, LensData, ColorBalance) successfully decrypted for test images
- ✅ All 71 Nikon tests passing including 6 comprehensive decryption algorithm tests
- ✅ XLAT lookup tables extracted and validated against ExifTool source (lines 13505-13538)
- ✅ decrypt_nikon_data() function handles initialization, state management, and offset calculations
- ✅ process_nikon_encrypted() performs actual decryption instead of detection only

### 2. Task: Add Model-Specific Processing for Popular Cameras ✅ **COMPLETED**

**Success Criteria**: D850, Z9, Z8, Z7 samples extract all required tags with identical values to ExifTool
**Approach**: Implement ProcessBinaryData dispatch for 4-5 most popular models using existing tag kit system
**Dependencies**: Task 1 (decryption algorithms)

**Success Patterns**:

- ✅ Model detection correctly selects appropriate ShotInfo table variant
- ✅ Binary data extraction works for each model's specific offset schemes
- ✅ All required tags (ISO, Aperture, Lens info, etc.) extracted from encrypted sections
- ✅ NikonCameraModel enum with D850 (0243), Z8 (0806), Z9 (0805), Z7Series (080x) detection
- ✅ ModelOffsetConfig handles model-specific offset table positions (0x0c for D850, 0x24 for Z-series)
- ✅ process_encrypted_shotinfo(), process_encrypted_lensdata(), process_encrypted_colorbalance() functions implemented
- ✅ Integration with main Nikon pipeline - encrypted sections automatically processed when keys available
- ✅ All 79 Nikon tests passing including 8 new encrypted processing tests

### 3. Task: Complete Binary Data Extraction Integration

**Success Criteria**: Encrypted binary sections (ShotInfo, LensData, ColorBalance) fully processed with tag extraction
**Approach**: Integrate decrypted data processing with existing subdirectory processing system
**Dependencies**: Tasks 1 & 2 (decryption + model support)

**Success Patterns**:

- ✅ Encrypted ShotInfo sections processed to extract exposure data (ISO, aperture, shutter speed)
- ✅ LensData sections processed to extract lens identification and specifications
- ✅ ColorBalance sections processed for white balance and color space information

### 4. Task: End-to-End Integration Testing and Validation

**Success Criteria**: `make compat` passes for Nikon test images, no regressions in existing functionality
**Approach**: Comprehensive testing with real Nikon files and ExifTool comparison validation
**Dependencies**: Tasks 1, 2 & 3 (complete decryption pipeline)

**Success Patterns**:

- ✅ All required tags extracted from representative Nikon JPEG and NEF files
- ✅ Output matches ExifTool exactly for supported tags (using comparison tool)
- ✅ Error handling graceful when encryption keys unavailable

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [ ] **Activation**: Decryption algorithms are used by default when processing Nikon maker notes
- [ ] **Consumption**: Existing Nikon processing pipeline actively uses decrypted data for tag extraction
- [ ] **Measurement**: Can prove decryption working via ExifTool comparison and extracted tag values
- [ ] **Cleanup**: Encrypted section detection stubs replaced with actual processing, debug placeholders removed

**Red Flag Check**: If a task seems like "build decryption tool but don't wire it anywhere," ask for clarity. We're not writing algorithms to sit unused - everything must get us closer to "ExifTool compatibility for PhotoStructure."

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - Nikon files now extract required tags instead of showing raw values
- ✅ **Default usage** - Decryption happens automatically for encrypted Nikon sections, not opt-in
- ✅ **Old path removed** - Encrypted section detection placeholders eliminated, actual processing implemented
- ❌ Code exists but isn't used *(example: "decryption implemented but encryption framework still uses stubs")*
- ❌ Feature works "if you call it directly" *(example: "Decrypt function works but ProcessNikonEncrypted doesn't use it")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

None - all required infrastructure already implemented.

## Testing

- **Unit**: Test decryption algorithms with known encrypted/decrypted pairs from ExifTool verbose output
- **Integration**: Verify end-to-end tag extraction from real Nikon files (D850, Z9, Z8, Z7 samples)
- **Manual check**: Run `cargo run --bin compare-with-exiftool nikon_sample.jpg` and confirm identical output

## Definition of Done

- [ ] `cargo t nikon` passes for all decryption and model-specific processing tests
- [ ] `make precommit` clean
- [ ] Required tags extracted from D850, Z9, Z8, Z7 sample files with ExifTool-identical values
- [ ] Integration tests pass - no regressions in existing functionality

## Implementation Guidance

### Recommended Patterns

- **XOR Decryption**: Follow ExifTool's exact algorithm with lookup tables - don't optimize or simplify
- **State Management**: Use struct to manage decryption state (`ci0`, `cj0`, `ck0`) across function calls
- **Error Handling**: Graceful fallback when keys unavailable - return raw values, never panic
- **Binary Processing**: Leverage existing `value_extraction` module for consistent data parsing

### Tools to Leverage

- **Existing encryption framework**: `NikonEncryptionKeys` struct and validation logic
- **Tag kit system**: Generated PrintConv implementations for value formatting
- **Subdirectory processing**: Generic binary data extraction once decryption complete
- **Comparison tools**: `compare-with-exiftool` binary for validation

### ExifTool Translation Notes

- **Lines 13554-13588**: Core Decrypt function - translate `$xlat` lookups and XOR logic exactly
- **Lines 13892-14011**: ProcessNikonEncrypted - focus on decryption dispatch and binary data handling
- **Key Generation**: SerialKey function handles model-specific serial number processing (lines 13594-13601)

## Gotchas & Tribal Knowledge

### Known Edge Cases

1. **Serial Number Formats**: Some models use different formats - already handled in SerialKey function
2. **Missing Keys**: Some images lack serial/shutter count - framework already handles gracefully
3. **Firmware Updates**: Can change encryption parameters - focus on mainstream firmware versions first
4. **Model Detection**: Must detect exact camera model before selecting ShotInfo table variant

### ExifTool Translation Challenges

- **Perl Variables**: `$ci0`, `$cj0`, `$ck0` must become struct fields for state management in Rust
- **Lookup Tables**: `$xlat[0]` and `$xlat[1]` arrays must be static constants in Rust
- **XOR Operations**: Perl's byte manipulation must use explicit u8 operations in Rust
- **String vs Binary**: ExifTool mixes string/binary operations - be explicit about data types

### Performance Considerations

- **Decryption Overhead**: Adds ~15% processing time but necessary for most valuable tags
- **Pre-scan Impact**: Two-pass processing already implemented and optimized
- **Memory Usage**: Large ShotInfo tables (Z9 has several KB) - use borrowed data where possible

## Quick Debugging

Stuck? Try these:

1. `exiftool -v3 nikon_file.jpg` - See ExifTool's decryption process and keys
2. `cargo t nikon_decrypt -- --nocapture` - See debug prints from decryption attempts  
3. `xxd encrypted_section.bin` - Hex dump to verify data patterns
4. Compare with ExifTool's `HexDump` output in verbose mode

---

## Summary

Nikon implementation is 85% complete with comprehensive infrastructure already built. The remaining 15% requires focused translation of ExifTool's 155-line decryption core (Decrypt + ProcessNikonEncrypted functions) plus model-specific binary data processing for 4-5 popular cameras. Success means Nikon files extract required tags automatically, with identical output to ExifTool for supported models.

**Estimated Effort**: 15-25 hours focused on decryption algorithms and popular model support (vs. original 40-60 hour estimate that assumed no infrastructure).