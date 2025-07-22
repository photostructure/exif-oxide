# Technical Project Plan: Nikon Required Tags Support

## Project Overview

- **High-level goal**: Enable extraction of all "required: true" tags from Nikon JPEG and RAW (NEF/NRW) files
- **Problem statement**: Currently zero Nikon tags are extracted despite having basic infrastructure in place. Nikon's encryption system blocks access to most valuable metadata.

## Background & Context

- Nikon is ExifTool's largest module (14,191 lines) with extensive encryption
- 25-30 required tags need Nikon-specific implementation
- Unlike Canon, Nikon encrypts most maker notes data requiring decryption keys
- Critical for PhotoStructure compatibility with Nikon cameras

### Links to Related Design Docs
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Must follow ExifTool's implementation exactly
- [CODEGEN.md](../CODEGEN.md) - Leverage for model-specific tables
- [tag-metadata.json](../tag-metadata.json) - Required tags definition

## Technical Foundation

### Key Codebases
- `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` - Canonical implementation
- `src/implementations/nikon/` - Our Nikon module
- `codegen/config/Nikon_pm/` - Codegen configurations

### ExifTool Functions to Study
- `ProcessNikon()` - Main entry point with pre-scan logic
- `ProcessNikonEncrypted()` - Decryption implementation (lines 9084-9244)
- `LensIDConv()` - Complex lens identification system
- Model-specific ShotInfo tables (30+ variants)

### Systems to Familiarize With
- Nikon Format1/2/3 detection
- Serial number and shutter count extraction for keys
- XOR-based encryption with model-specific constants
- Subdirectory and binary data processing

## Work Completed

### Infrastructure
- ✅ Basic module structure (`src/implementations/nikon/mod.rs`)
- ✅ Format detection for Nikon maker notes variants
- ✅ Codegen extraction of 618 lens IDs
- ✅ AF point lookup tables generated
- ✅ Module organization (encryption.rs, af.rs, lens.rs)

### Decisions Made
- Separate modules for encryption, AF, and lens handling
- Using codegen for static lookup tables
- Following ExifTool's exact tag naming

### Issues Resolved
- Module structure matches ExifTool organization
- Codegen framework successfully extracts complex tables

## Remaining Tasks

### Phase 1: Enable Basic Unencrypted Tags (High Confidence)
**Implementation**: Add entries to `src/generated/supported_tags.json`
1. Enable main Nikon IFD tags (0x0001-0x001c, 0x001e-0x00a6)
2. Implement basic tag extraction in `process_nikon_ifd()`
3. Add PrintConv for: CameraID, Quality, WhiteBalance, Sharpness, FocusMode
4. Test with sample Nikon JPEGs

**Expected yield**: 10-15 basic tags without encryption

### Phase 2: Extract Encryption Keys (High Confidence)
**Implementation**: Pre-scan phase in `process_maker_notes()`
1. Extract SerialNumber (tag 0x001d) - critical for key generation
2. Extract ShutterCount (tag 0x00a7) - second part of key
3. Store keys in ExifContext for later use
4. Validate key format per model

**Note**: Keys must be extracted before processing other tags

### Phase 3: Implement Decryption (Research Required)
**Known approach**: Port `ProcessNikonEncrypted()` from Nikon.pm
- XOR decryption with model-specific constants
- Key generation: combine serial, count, and magic numbers
- Different algorithms for different camera generations

**Unknown unknowns**:
- Exact byte ordering for key generation
- Edge cases in serial number formats
- Firmware version variations

**Research needed**:
1. Study lines 9084-9244 of Nikon.pm carefully
2. Create test harness comparing our decryption with ExifTool
3. Start with one modern model (suggest Z9 or D850)

### Phase 4: Model-Specific Processing (Research Required)
**Known requirements**: 30+ ShotInfo table variants
- Each model has unique offsets and tag definitions
- Must detect model and apply correct table
- Some models have multiple encrypted sections

**Implementation sketch**:
```rust
match model {
    "NIKON D850" => process_shot_info_d850(data, offset),
    "NIKON Z 9" => process_shot_info_z9(data, offset),
    // ... 30+ more variants
}
```

**Consider**: Runtime table extraction for model-specific tables
- Add `runtime_table.json` configuration for Nikon
- Extract ShotInfo tables dynamically like Canon

### Phase 5: Enable NEF/NRW Support (Research Required)
**Current state**: Detection commented out in `raw_detector.rs`
- NEF format well-understood (TIFF-based)
- Need to enable and test thoroughly

**Unknown**: 
- NRW format differences
- Encryption in RAW files
- Preview extraction complexity

## Prerequisites

- Complete Canon implementation as reference (mostly done)
- Robust ProcessBinaryData framework (exists)
- Runtime table extraction capability (needs enhancement)

## Testing Strategy

### Unit Tests
- Encryption key extraction for each model
- Decryption validation against known values
- Lens ID resolution for all 618 lenses
- AF point decoding

### Integration Tests
- Compare output with ExifTool for test images
- Verify all required tags extracted
- Test with images from each major model series

### Manual Testing Steps
1. Run `cargo run --bin compare-with-exiftool [nikon.jpg]`
2. Verify encryption keys extracted correctly
3. Check decrypted values match ExifTool
4. Validate lens identification

### Test Image Requirements
- Need samples from: D850, D750, Z6, Z7, Z9
- Both JPEG and NEF files
- Images with known metadata for validation

## Success Criteria & Quality Gates

### Definition of Done
- [ ] All required MakerNotes tags extracted from Nikon JPEGs
- [ ] Encryption/decryption matches ExifTool exactly
- [ ] Support for top 5 current Nikon models (Z9, Z8, Z7II, Z6III, D850)
- [ ] NEF file support enabled and tested
- [ ] Performance within 2x of ExifTool
- [ ] Zero regression in existing tests

### Quality Requirements
- Must handle missing encryption keys gracefully
- Error messages should guide users (e.g., "Cannot decrypt without serial number")
- No panic on malformed data

## Gotchas & Tribal Knowledge

### Known Edge Cases
1. **Serial Number Format**: Some models use different formats/locations
2. **Firmware Updates**: Can change encryption or tag locations
3. **Third-party Lenses**: May not match lens database exactly
4. **Missing Keys**: Some images lack serial/shutter count

### Technical Debt
- ExifTool's Nikon module has 25 years of accumulated fixes
- Many "magic" offsets exist for specific camera quirks
- Some decryption constants derived through reverse engineering

### Critical Insights
1. **Pre-scan is Mandatory**: Must extract keys before processing
2. **Model Detection First**: Everything depends on camera model
3. **Trust ExifTool**: If our output differs, we're wrong
4. **Start Small**: Get unencrypted tags working first

### Nikon-Specific Patterns
- Tags 0x0001-0x00ff are mostly standard across models
- Tags 0x0100+ are often model-specific
- Encrypted sections typically start at known offsets
- ColorBalance format varies by camera generation

### Performance Considerations
- Decryption adds overhead (~10-20%)
- Pre-scan requires two passes through data
- Consider caching decrypted sections

### Historical Context
- Format1: Original Nikon DSLRs (D1, D100)
- Format2: Most DSLRs (D200-D850 era)  
- Format3: Recent mirrorless (Z series)
- Each format has different encryption schemes