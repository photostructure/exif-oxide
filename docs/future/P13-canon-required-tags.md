# Technical Project Plan: Canon Required Tags Implementation

## Project Overview

- **Goal**: Fix corrupted Canon tag extraction and implement all required Canon tags from tag-metadata.json  
- **Problem**: Canon Main table processing is fundamentally broken - firmware versions show garbage, model IDs have wrong format, required tags missing entirely
- **Constraints**: Must exactly follow ExifTool's Canon.pm processing logic without "improvements"

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

**REQUIRED**: Canon is ExifTool's most complex module with 107 tag tables and model-specific processing.

### System Overview

- **Canon Main table (%Canon::Main)**: Primary dispatcher with 200+ tags including all required tags like FileNumber (0x8), CanonModelID (0x10), InternalSerialNumber (0x96)
- **Canon tag kit system**: Generated Rust code from Canon.pm table extraction, handling PrintConv expressions and subdirectory processing
- **Canon binary data**: Complex subdirectories (CameraSettings, ShotInfo, ColorData) with model-specific validation and offset schemes
- **Current implementation**: `src/implementations/canon/mod.rs` coordinates processing but Main table extraction is corrupted

### Key Concepts & Domain Knowledge

- **Canon MakerNotes structure**: Standard IFD with Canon-specific tag processing and absolute/relative offset mixing
- **Tag kit vs Main table**: Generated tag kit handles subdirectories, but basic Main table tags (strings, int32u) need direct processing  
- **Model-specific processing**: Camera info tables vary by firmware version, requiring "FirmwareVersionLookAhead" detection
- **PrintConv expressions**: Perl expressions like `'$_=$val,s/(\d+)(\d{4})/$1-$2/,$_'` for FileNumber formatting must be evaluated exactly

### Surprising Context

**CRITICAL**: Document non-intuitive aspects discovered during research:

- **Canon offset schemes**: Mix absolute file offsets and relative-to-MakerNote offsets in same IFD - violates TIFF standard but required for Canon compatibility
- **String corruption source**: Our binary data extraction shows `"Unknown (���)"` for firmware version indicating byte order or offset calculation errors  
- **Model ID disaster**: ExifTool shows `"EOS Rebel T3i / 600D / Kiss X5"` but we show `"2147483647 mm"` - wrong lookup table and format applied
- **Tag kit ID mismatch**: Generated tag kit uses different IDs than Canon Main table, causing extraction conflicts
- **Required vs generated mismatch**: tag-metadata.json marks FileNumber as required (freq 0.13) but it's missing entirely from our output
- **Firmware dependency complexity**: Tag locations shift based on firmware version requiring dynamic validation

### Foundation Documents

- **Design docs**: [API-DESIGN.md](../design/API-DESIGN.md) for TagEntry structure, [CODEGEN.md](../CODEGEN.md) for tag kit system
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` Main table line ~1375, CameraSettings line ~2166
- **Canon module overview**: `third-party/exiftool/doc/modules/Canon.md` - comprehensive structure documentation  
- **Start here**: `src/implementations/canon/mod.rs` process_canon_makernotes() function and `src/generated/Canon_pm/tag_kit/mod.rs`

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD structure, MakerNotes processing, binary data extraction patterns
- **Setup required**: Canon test images in `test-images/canon/`, ExifTool installed for comparison testing

**Context Quality Check**: A new engineer can understand WHY Canon processing is complex and why exact ExifTool translation is critical.

## Work Completed

- ✅ **Canon tag kit generation** → chose automated extraction over manual translation because ExifTool releases monthly updates
- ✅ **Binary data infrastructure** → implemented CameraSettings, ShotInfo, AFInfo processors following ExifTool patterns
- ✅ **MakerNotes namespace** → Canon tags properly grouped under "MakerNotes:" prefix for PhotoStructure compatibility
- ✅ **Test infrastructure** → Canon test images available and comparison tools working

## Remaining Tasks

### 1. Task: Fix Canon Main Table Tag Extraction Corruption

**Success Criteria**: 
- `cargo run test-images/canon/eos_rebel_t3i.cr2 | grep CanonFirmwareVersion` shows `"Firmware Version 1.0.1"` not `"Unknown (���)"`
- `cargo run test-images/canon/eos_rebel_t3i.cr2 | grep CanonModelID` shows `"EOS Rebel T3i / 600D / Kiss X5"` not `"2147483647 mm"`
- No corrupted string data or wrong format applications

**Approach**: 
1. Debug binary data extraction in `src/implementations/canon/mod.rs` process_canon_makernotes()
2. Verify byte order handling and offset calculations for Main table tags
3. Check if generated canonModelID lookup table is corrupted or lookup logic broken
4. Compare our IFD parsing with ExifTool's Main table processing logic

**Dependencies**: None - this is blocking other Canon work

**Success Patterns**:
- ✅ String tags show readable text, not garbage characters
- ✅ Model ID uses correct lookup table and returns human-readable camera name
- ✅ Firmware version parsed as simple string without format conversion errors

### 2. Task: Implement Missing FileNumber Tag (Required, Freq 0.13)

**Success Criteria**: 
- `cargo run test-images/canon/eos_rebel_t3i.cr2 | grep FileNumber` returns formatted value like `"123-4567"`
- ExifTool comparison shows identical FileNumber format
- Tag appears in default output without special flags

**Approach**:
1. Verify FileNumber (0x8) extraction from Canon Main table
2. Implement PrintConv expression `'$_=$val,s/(\d+)(\d{4})/$1-$2/,$_'` evaluation  
3. Ensure tag is marked as Group 2 'Image' per ExifTool specification
4. Test with multiple Canon images to verify directory-file numbering

**Dependencies**: Task 1 (Main table extraction must work first)

**Success Patterns**:
- ✅ FileNumber appears in JSON output under MakerNotes group
- ✅ Format matches ExifTool exactly (directory-file format with hyphen)
- ✅ Value updates correctly for different Canon images

### 3. RESEARCH: Investigate LensType Extraction Complexity

**Objective**: Understand why LensType shows in our output but needs verification against ExifTool accuracy
**Success Criteria**: Document all Canon LensType extraction locations and methods used by ExifTool
**Done When**: Clear mapping of CameraSettings vs CameraInfo vs Main table LensType sources with model dependencies

ExifTool shows: `"Canon EF 24-105mm f/4L IS USM"` for test image - verify our extraction matches exactly.

### 4. Task: Implement InternalSerialNumber Tag (Required per PhotoStructure)

**Success Criteria**:
- `cargo run test-images/canon/eos_rebel_t3i.cr2 | grep InternalSerialNumber` shows `"ZA0740300"`  
- Matches ExifTool output exactly
- ValueConv processing removes trailing 0xFF bytes per Canon.pm specification

**Approach**:
1. Extract from Main table 0x96 with proper string processing
2. Implement ValueConv: `'$val=~s/\xff+$//; $val'` for trailing cleanup
3. Handle model-specific conditions (EOS 5D uses SerialInfo subdirectory)
4. Verify string encoding and null termination handling

**Dependencies**: Task 1 (Main table extraction)

**Success Patterns**:
- ✅ Serial number matches camera body exactly
- ✅ No trailing garbage or 0xFF bytes in output
- ✅ Model-specific processing routes correctly

### 5. Task: Research and Document CameraID Requirements

**Success Criteria**: Complete analysis of CameraID extraction requirements across Canon camera models  
**Approach**: Study Canon CameraInfo tables and model-specific extraction patterns
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Document all CameraInfo table locations for CameraID
- ✅ Identify model detection requirements  
- ✅ Map firmware version dependencies

## Implementation Guidance

### ExifTool Translation Notes

**Canon offset handling pattern**:
```perl
# ExifTool Canon.pm uses mixed offset schemes
IsOffset => '$val and $$et{FILE_TYPE} ne "JPEG"'  # Absolute for some tags
# vs relative-to-MakerNote for others
```

**String extraction with cleanup**:
```perl  
RawConv => '$val=~/^.{4}([^\0]+)/s ? $1 : undef'  # Skip 4-byte header
ValueConv => '$val=~s/\xff+$//; $val'              # Remove trailing 0xFF
```

**Model-specific conditionals**:
```perl
Condition => '$$self{Model} =~ /EOS 5D/'  # Route by camera model
```

### Tools to Leverage

- **Existing Canon processors**: `extract_camera_settings()`, `extract_shot_info()` in `src/implementations/canon/binary_data.rs`
- **Tag kit system**: Generated PrintConv expressions in `src/generated/Canon_pm/tag_kit/`
- **Comparison tool**: `cargo run --bin compare-with-exiftool image.jpg` for validation
- **Debug logging**: `tracing::debug!()` calls throughout Canon processing for investigation

### Architecture Considerations

- **Main table vs tag kit separation**: Basic types (string, int32u) process directly, complex subdirectories use tag kit
- **PrintConv evaluation**: Perl expressions must be evaluated in Rust expression evaluator system
- **Model detection**: Camera model and firmware version determine processing paths
- **Precedence handling**: Main table tags override subdirectory tags when conflicts occur

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept shelf-ware.

Every Canon tag implementation must include:
- [ ] **Activation**: Tag extraction enabled by default in Canon MakerNotes processing
- [ ] **Consumption**: Tags appear in standard `cargo run image.jpg` output without special flags  
- [ ] **Measurement**: Can verify tag presence with `grep TagName` on output
- [ ] **Cleanup**: Remove manual workarounds, obsolete extraction code deleted

**Red Flag Check**: If implementation requires special flags or doesn't appear in default output, integration failed.

## Working Definition of "Complete"

A Canon tag is complete when:
- ✅ **System behavior changes** - tag appears in JSON output for Canon images
- ✅ **Default usage** - extraction works automatically, not opt-in
- ✅ **Old path removed** - no "Unknown" values or corruption artifacts remain  
- ❌ Code exists but shows corrupted data *(example: "Unknown (���)" firmware version)*
- ❌ Feature works "if you call it directly" *(example: "tag kit has entry but Main table broken")*

## Prerequisites

- **P10a EXIF Foundation** → [EXIF foundation TPP](P10a-exif-foundation.md) → verify with `cargo t exif_basic`

## Testing

- **Unit**: Test Canon Main table extraction with `cargo t canon_main_table`
- **Integration**: Verify end-to-end with `cargo run test-images/canon/eos_rebel_t3i.cr2`
- **Manual check**: Compare with ExifTool using `cargo run --bin compare-with-exiftool test-images/canon/eos_rebel_t3i.cr2`

## Definition of Done

- [ ] `cargo run test-images/canon/eos_rebel_t3i.cr2 | grep -v "Unknown\|���"` shows no corruption
- [ ] `make precommit` clean  
- [ ] FileNumber, CanonModelID, CanonFirmwareVersion, InternalSerialNumber all extract correctly
- [ ] ExifTool comparison shows <5 tag differences for Canon images
- [ ] All Canon required tags from tag-metadata.json implemented or documented as future work