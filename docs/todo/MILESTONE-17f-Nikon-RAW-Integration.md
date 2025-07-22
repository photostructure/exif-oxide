# Technical Project Plan: Milestone 17f - Nikon RAW Integration

## Project Overview

- **High-level goal**: Integrate Nikon NEF/NRW support with the RAW infrastructure from milestones 17a-e
- **Problem statement**: Nikon has existing module structure but zero tag extraction. Need to prove NEF integration viable and establish foundation for full Nikon support.

## Background & Context

- Unlike other manufacturers requiring from-scratch implementation, Nikon has existing work to integrate
- NEF is a TIFF-based format similar to ORF/ARW, but with extensive encryption
- This milestone proves the concept - full implementation tracked in [20250122-nikon-required-tags.md](20250122-nikon-required-tags.md)
- Critical for enabling Nikon support in PhotoStructure

### Links to Related Design Docs
- [20250122-nikon-required-tags.md](20250122-nikon-required-tags.md) - Comprehensive Nikon implementation plan
- [MILESTONE-17-RAW-Format-Support.md](MILESTONE-17-RAW-Format-Support.md) - Overall RAW strategy
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Implementation principles

## Technical Foundation

### Key Codebases
- `src/implementations/nikon/` - Existing Nikon module structure
- `src/raw/detector.rs` - RAW format detection (NEF commented out)
- `src/raw/processors/` - RAW processor infrastructure
- `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` - Reference implementation

### APIs/Systems
- TIFF container processing (shared with ORF, ARW)
- Manufacturer routing system from 17a
- Offset management patterns from 17b
- ProcessBinaryData framework

### NEF Format Structure
```
NEF File Layout:
├── TIFF Header
├── IFD0 (Main Image)
│   ├── ImageWidth/Height
│   ├── Make/Model
│   └── ExifIFD Offset
├── IFD1 (Thumbnail)
├── ExifIFD
│   ├── Standard EXIF tags
│   └── MakerNote Offset
└── MakerNote IFD (Nikon-specific)
    ├── Unencrypted tags (0x0001-0x001c)
    ├── SerialNumber (0x001d) - CRITICAL
    ├── ShutterCount (0x00a7) - CRITICAL
    └── Encrypted sections (0x0091+)
```

## Work Completed

### From Previous Development
- ✅ Module structure (encryption.rs, af.rs, lens.rs)
- ✅ Format1/2/3 detection logic
- ✅ 618 lens IDs via codegen
- ✅ AF point tables generated
- ✅ NikonEncryptionKeys structure defined

### What's Missing
- ❌ NEF detection disabled in raw_detector.rs
- ❌ No tag extraction implementation
- ❌ Encryption/decryption not implemented
- ❌ No integration with RAW pipeline

## Remaining Tasks

### Task 1: Enable NEF Detection (2-3 hours, High Confidence)
**Implementation**: Update `src/raw/detector.rs`
```rust
// Uncomment and verify NEF detection
b"II*\0\x1c\0\0\0HEAPCCDR" => Ok(FileType::Nef),
b"II*\0\x08\0\0\0CR" => Ok(FileType::Nef), // Some NEF variants

// In process_raw(), add routing:
FileType::Nef => {
    let processor = NikonProcessor::new();
    processor.process_raw(reader, context)
}
```

**Validation**: NEF files recognized by CLI

### Task 2: Basic Tag Extraction (4-6 hours, High Confidence)
**Implementation**: Enable unencrypted tags
1. Add Nikon tags to `supported_tags.json`:
   ```json
   {
     "0x0001": { "name": "MakerNoteVersion", "group": "Nikon" },
     "0x0002": { "name": "ISO", "group": "Nikon" },
     // ... tags 0x0001-0x001c (unencrypted)
   }
   ```
2. Implement `process_nikon_ifd()` in existing module
3. Hook into TIFF processing pipeline

**Expected Output**: 10-15 basic tags (Make, Model, ISO, Quality)

### Task 3: Key Extraction Pre-Scan (6-8 hours, High Confidence)
**Implementation**: Two-pass processing
```rust
impl NikonProcessor {
    fn pre_scan_for_keys(&mut self, reader: &mut impl Read) -> Result<()> {
        // First pass: find SerialNumber (0x001d) and ShutterCount (0x00a7)
        // Store in self.encryption_keys
        // Critical: must happen before main processing
    }
    
    fn process_raw(&mut self, reader: &mut impl Read, context: &mut Context) -> Result<()> {
        self.pre_scan_for_keys(reader)?;
        // Now process normally with keys available
    }
}
```

**Validation**: Keys visible in debug output

### Task 4: Proof-of-Concept Decryption (8-10 hours, Research Required)
**Goal**: Decrypt ONE section from ONE model
**Implementation approach**:
1. Port simplified XOR from Nikon.pm (lines 9084-9244)
2. Focus on D850 or Z9 (well-documented)
3. Target ShotInfo (0x0091) first
4. Create test harness comparing with ExifTool

**Unknown factors**:
- Exact key concatenation order
- Byte alignment requirements
- Model-specific constants location

**Success metric**: One encrypted tag matches ExifTool output

### Task 5: Integration Testing (4-6 hours, High Confidence)
**Implementation**: Validate integration
1. Update compatibility test script:
   ```bash
   # In tools/generate_exiftool_json.sh
   SUPPORTED_EXTENSIONS=("jpg" "jpeg" "nef" "orf" "arw" ...)
   ```
2. Add NEF test files to `test-images/nikon/`
3. Run `make compat-gen && make compat-test`
4. Compare output with ExifTool

**Required test files**:
- Modern DSLR with encryption (D850)
- Z-series mirrorless (Z9)
- Older format (D200)
- Coolpix without encryption

## Prerequisites

- Milestones 17a-e completed (RAW infrastructure) ✓
- TIFF container processing working ✓
- Manufacturer routing system in place ✓
- NEF sample files with known metadata

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_nef_detection() {
    let data = include_bytes!("../test-images/nikon/d850.nef");
    assert_eq!(detect_file_type(data), Ok(FileType::Nef));
}

#[test]
fn test_key_extraction() {
    let keys = extract_nikon_keys(test_data);
    assert_eq!(keys.serial_number, Some("12345678"));
    assert_eq!(keys.shutter_count, Some(1000));
}
```

### Integration Tests
- NEF files process without panic
- Basic EXIF tags extracted
- Keys extracted when present
- Graceful handling of missing keys

### Manual Testing
```bash
# Test NEF recognition
cargo run -- test-images/nikon/d850.nef

# Compare with ExifTool
cargo run --bin compare-with-exiftool test-images/nikon/d850.nef

# Verify encryption keys (debug mode)
RUST_LOG=debug cargo run -- test-images/nikon/z9.nef | grep -i serial
```

## Success Criteria & Quality Gates

### Minimum Viable Integration
- [ ] NEF files detected by raw_detector
- [ ] Basic tags extracted (Make, Model, DateTime)
- [ ] SerialNumber and ShutterCount extracted
- [ ] No panics on any NEF variant
- [ ] Clear error messages for unsupported features

### Stretch Goals (Time Permitting)
- [ ] One encrypted section decrypted
- [ ] Support for 2-3 camera models
- [ ] NRW format detection

### Non-Goals for This Milestone
- Full encryption implementation (40-60 hour effort)
- All model support (30+ variants)
- Complete PrintConv implementations
- Binary data extraction (previews, thumbnails)

## Gotchas & Tribal Knowledge

### Critical Implementation Order
1. **Detection First**: Must update raw_detector.rs before anything works
2. **Keys Before Tags**: Pre-scan MUST complete before processing
3. **Format Variations**: D850 uses Format2, Z9 uses Format3 - different offsets

### NEF-Specific Patterns
- Magic bytes: `II*\0` followed by various markers
- Some NEF files use `CR` marker at offset 8
- Maker note always starts with "Nikon\0"
- Format version at offset 10 in maker note

### Common Pitfalls
- **Forgetting Pre-Scan**: Without keys, everything appears corrupted
- **Wrong Offset Base**: Format1/2/3 use different offset calculations
- **Missing SUPPORTED_EXTENSIONS**: Compat tests will fail silently

### Performance Notes
- Pre-scan adds ~10% overhead (two passes)
- Decryption adds another 10-20%
- Cache decrypted sections to avoid re-decryption

### Edge Cases to Handle
- Images without serial numbers (some Coolpix models)
- Corrupted maker notes (third-party software)
- Firmware variations within same model
- Professional models with dual card slots (different serial number formats)

## Risk Mitigation

### Encryption Complexity
- **Risk**: Can't decrypt anything in one week
- **Mitigation**: Focus on key extraction only, prove concept with one section
- **Fallback**: Deliver unencrypted tags only

### Time Constraints
- **Risk**: Full implementation needs 40-60 hours
- **Mitigation**: Clearly scope to integration only
- **Success**: Foundation laid for future work

### Test File Availability
- **Risk**: No access to encrypted NEF samples
- **Mitigation**: Use ExifTool sample images repository
- **Alternative**: Generate test files with known values

## Future Work

This milestone establishes the foundation. Full implementation requires:
- Complete encryption/decryption system (Phase 3 of TPP)
- Model-specific processing (Phase 4 of TPP)
- Binary data extraction (Phase 5 of TPP)
- Full NEF/NRW format support (Phase 6 of TPP)

See [20250122-nikon-required-tags.md](20250122-nikon-required-tags.md) for comprehensive plan.

## Summary

This 1-week milestone proves Nikon NEF integration is viable within our RAW framework. We'll enable detection, extract basic tags, and demonstrate that encryption keys can be extracted. While full Nikon support requires 40-60 hours, this milestone validates the approach and unblocks future development.