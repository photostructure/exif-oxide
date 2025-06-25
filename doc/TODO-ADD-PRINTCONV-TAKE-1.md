# TODO: Add Missing PrintConv Implementations

## Executive Summary

**Current Status**: 832+ PrintConv functions are set to `PrintConvId::None`, causing raw values to be displayed instead of human-readable strings.

**Impact**: Users see numeric codes like `2` instead of meaningful values like `"Portrait"` for picture modes, exposure settings, etc.

**Goal**: Implement high-priority PrintConv functions to transform raw EXIF values into human-readable strings, matching ExifTool's output exactly.

## Critical Background Reading

**REQUIRED**: Read these documents before starting implementation:

1. **[`doc/PRINTCONV-ARCHITECTURE.md`](PRINTCONV-ARCHITECTURE.md)** - Revolutionary table-driven approach (96% code reduction vs manual porting)
2. **[`doc/EXIFTOOL-SYNC.md`](EXIFTOOL-SYNC.md)** - Synchronization process and attribution requirements  
3. **[`src/core/print_conv.rs:1-583`](../src/core/print_conv.rs)** - Current PrintConv implementation and patterns

## Current Implementation Gaps

### Missing PrintConv by Component

```bash
# Generate current gap analysis
grep -c "print_conv: PrintConvId::None" src/tables/*.rs

# Results (as of 2025-06-24):
src/tables/exif_tags.rs:592        # CRITICAL: Core EXIF metadata
src/tables/panasonic_tags.rs:118   # Panasonic cameras
src/tables/fujifilm_tags.rs:85     # Fujifilm cameras  
src/tables/apple_tags.rs:37        # iPhone/iPad photos
```

**Total Missing**: 832+ PrintConv implementations

### Priority Framework

**High Priority**: Tags marked with `★★★★ ✔` in [`third-party/exiftool-vendored.js/src/Tags.ts`](../third-party/exiftool-vendored.js/src/Tags.ts)

```bash
# Find high-priority tags
grep -c "★★★★" third-party/exiftool-vendored.js/src/Tags.ts
# Result: 67 critical tags need proper PrintConv
```

**Example High-Priority Tags**:
- `ExifToolVersion` - ★★★★ ✔
- `BitsPerSample` - ★★★★ ✔  
- `Directory` - ★★★★ ✔
- `FileName` - ★★★★ ✔
- `FileType` - ★★★★ ✔
- `ImageHeight` - ★★★★ ✔
- `ImageWidth` - ★★★★ ✔

## 4-Phase Implementation Plan

### Phase 1: Core EXIF PrintConv (Week 1) 🎯

**Objective**: Implement universal EXIF conversions used across ALL manufacturers

**Files to Modify**:
- [`src/core/print_conv.rs`](../src/core/print_conv.rs) - Add new PrintConvId variants and implementations
- [`src/tables/exif_tags.rs`](../src/tables/exif_tags.rs) - Update `print_conv` assignments

**Implementation Steps**:

1. **Add PrintConvId Variants** (Days 1-2):
```rust
// Add to PrintConvId enum in src/core/print_conv.rs around line 563
pub enum PrintConvId {
    // ... existing variants ...
    
    // Universal EXIF conversions
    Make,            // Camera manufacturer 
    Model,           // Camera model
    LensModel,       // Lens identification
    Software,        // Software version
    Artist,          // Photographer name
    Copyright,       // Copyright info
    LensInfo,        // Lens info array ("24-70mm f/2.8")
    ISO,             // ISO speed formatting
    Flash,           // Flash mode (16 standard values)
    MeteringMode,    // Spot/Center/Matrix lookup  
    ExposureProgram, // Auto/Manual/Aperture/Shutter Priority
    SceneCaptureType, // Standard/Landscape/Portrait/Night
    WhiteBalance,    // Auto/Manual lookup
    // ... add 10-15 more based on ★★★★ ✔ analysis
}
```

2. **Implement Conversion Functions** (Days 3-4):
```rust
// Add to apply_print_conv() match statement around line 700
PrintConvId::Make => exif_value_to_string(value), // Simple string formatting
PrintConvId::Model => exif_value_to_string(value),
PrintConvId::Flash => format_flash_mode(value),    // Lookup table
PrintConvId::ExposureProgram => format_exposure_program(value),
// ... implement remaining conversions
```

3. **Add Helper Functions** (Days 4-5):
```rust
// Add after format_color_space() around line 1125
fn format_flash_mode(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0x00) => "No Flash".to_string(),
        Some(0x01) => "Fired".to_string(),
        Some(0x05) => "Fired, No Return Light".to_string(),
        Some(0x07) => "Fired, Return Light".to_string(),
        // ... add all 16 standard flash modes
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

fn format_exposure_program(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Not Defined".to_string(),
        Some(1) => "Manual".to_string(),
        Some(2) => "Program AE".to_string(),
        Some(3) => "Aperture-priority AE".to_string(),
        Some(4) => "Shutter-priority AE".to_string(),
        // ... complete EXIF standard lookup table
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}
```

4. **Update Tag Tables** (Day 5):
```rust
// Update exif_tags.rs assignments - change from PrintConvId::None to appropriate conversions
ExifTag { id: 0x010f, name: "Make", print_conv: PrintConvId::Make },
ExifTag { id: 0x0110, name: "Model", print_conv: PrintConvId::Model },
ExifTag { id: 0x9003, name: "DateTimeOriginal", print_conv: PrintConvId::DateTime },
```

**Validation Commands**:
```bash
# Test implementation
cargo build && cargo test print_conv

# Compare with ExifTool
exiftool -struct -json test.jpg > exiftool.json
cargo run -- test.jpg > exif-oxide.json
# Verify values match exactly
```

**Expected Impact**: Core photography metadata properly formatted across ALL manufacturers

### Phase 2: Universal Patterns (Week 2) 📈

**Objective**: Implement patterns used by 3+ manufacturers

**Target Patterns** (based on [`doc/PRINTCONV-ARCHITECTURE.md`](PRINTCONV-ARCHITECTURE.md)):
- `LensType` - Universal lens identification
- `DriveMode` - Single/Continuous/Timer/Bracket
- `ImageStabilization` - On/Off/Auto variants
- `NoiseReduction` - On/Off/Auto/High/Low
- `Saturation` - Normal/High/Low/Custom  
- `Contrast` - Normal/High/Low/Custom
- `Sharpness` - Normal/Hard/Soft/Custom
- `AutoBracketing` - Various bracketing modes
- `ColorSpace` - sRGB/Adobe RGB/ProPhoto RGB

**Implementation Approach**:
```bash
# Use automated analysis tools
cargo run --bin exiftool_sync analyze printconv-patterns Exif.pm
cargo run --bin exiftool_sync extract printconv-tables Exif.pm
```

**Reference Files**:
- [`src/bin/exiftool_sync/extractors/printconv_analyzer.rs`](../src/bin/exiftool_sync/extractors/printconv_analyzer.rs) - Pattern analysis
- [`src/bin/exiftool_sync/extractors/printconv_generator.rs`](../src/bin/exiftool_sync/extractors/printconv_generator.rs) - Code generation
- [`src/bin/exiftool_sync/extractors/printconv_tables.rs`](../src/bin/exiftool_sync/extractors/printconv_tables.rs) - Table extraction

**Expected Impact**: 40-50 new PrintConvId patterns covering majority of universal cases

### Phase 3: Manufacturer-Specific Priority (Week 3) 🏭

**Objective**: Implement highest-value manufacturer-specific conversions

**Implementation Order** (by frequency/importance):

1. **Canon** (Days 6-7):
```bash
# Analyze Canon patterns
cargo run --bin exiftool_sync analyze printconv-patterns Canon.pm

# Generate implementation
cargo run --bin exiftool_sync extract printconv-tables Canon.pm
```

2. **Nikon** (Days 8-9):
```bash
cargo run --bin exiftool_sync analyze printconv-patterns Nikon.pm  
cargo run --bin exiftool_sync extract printconv-tables Nikon.pm
```

3. **Sony** (Day 10):
```bash
cargo run --bin exiftool_sync analyze printconv-patterns Sony.pm
cargo run --bin exiftool_sync extract printconv-tables Sony.pm
```

**Reference Implementation**: See [`src/maker/pentax.rs:88-110`](../src/maker/pentax.rs) for table-driven parser pattern

**Expected Impact**: 90%+ coverage of high-priority manufacturer-specific tags

### Phase 4: Remaining Gaps & GPMF (Week 4) 🎯

**Objective**: Complete coverage and handle GoPro metadata

**Recently Added GPMF Support**: 
- 102+ `GpmfXxx` PrintConvId variants added to [`src/core/print_conv.rs:564-667`](../src/core/print_conv.rs)
- Currently return raw values (lines 708-811)
- Need implementation based on `third-party/exiftool/lib/Image/ExifTool/GoPro.pm`

**Remaining Manufacturers**:
- **Fujifilm**: 85 missing PrintConv patterns
- **Panasonic**: 118 missing PrintConv patterns  
- **Apple**: 37 missing PrintConv patterns
- **Olympus**: Various missing patterns

**GPMF Implementation**:
```rust
// Example GPMF conversion (around line 708)
PrintConvId::GpmfAutoRotation => match as_u32(value) {
    Some(0) => "Off".to_string(),
    Some(1) => "On".to_string(), 
    Some(2) => "Up".to_string(),
    _ => format!("Unknown ({})", exif_value_to_string(value)),
},
```

## Tools and Automation 🛠️

### PrintConv Analysis Tools

**Located in**: [`src/bin/exiftool_sync/extractors/`](../src/bin/exiftool_sync/extractors/)

1. **Pattern Analyzer**: [`printconv_analyzer.rs`](../src/bin/exiftool_sync/extractors/printconv_analyzer.rs)
   ```bash
   # Analyze PrintConv patterns for reusability
   cargo run --bin exiftool_sync analyze printconv-patterns Canon.pm
   ```

2. **Code Generator**: [`printconv_generator.rs`](../src/bin/exiftool_sync/extractors/printconv_generator.rs)  
   ```bash
   # Generate PrintConv implementations
   cargo run --bin exiftool_sync generate printconv-functions Canon.pm
   ```

3. **Table Extractor**: [`printconv_tables.rs`](../src/bin/exiftool_sync/extractors/printconv_tables.rs)
   ```bash
   # Extract complete tag tables with PrintConv mappings
   cargo run --bin exiftool_sync extract printconv-tables Canon.pm
   ```

### Gap Analysis Commands

```bash
# Count missing implementations
find src/tables -name "*.rs" -exec grep -c "print_conv: PrintConvId::None" {} +

# Find high-priority tags needing implementation
grep -A1 -B1 "★★★★.*✔" third-party/exiftool-vendored.js/src/Tags.ts

# Check what's already implemented
grep -E "PrintConvId::[A-Z]" src/core/print_conv.rs | grep -v None | wc -l
```

### Validation Process

```bash
# Build and test
cargo build && cargo test print_conv

# Compare with ExifTool output
exiftool -struct -json test_image.jpg > exiftool_output.json  
cargo run -- test_image.jpg > exif_oxide_output.json

# Verify exact match of converted values
# Focus on human-readable strings, not raw numeric values
```

## Code Examples and Templates

### 1. Adding New PrintConvId Variant

```rust
// In src/core/print_conv.rs, add to PrintConvId enum:
pub enum PrintConvId {
    // ... existing variants ...
    MyNewConversion,  // Brief description of conversion purpose
}
```

### 2. Implementing Simple Lookup Conversion

```rust
// In apply_print_conv() match statement:
PrintConvId::MyNewConversion => match as_u32(value) {
    Some(0) => "Off".to_string(),
    Some(1) => "On".to_string(),
    Some(2) => "Auto".to_string(),
    _ => format!("Unknown ({})", exif_value_to_string(value)),
},
```

### 3. Implementing Complex Helper Function

```rust
// Add after existing helper functions:
fn format_my_complex_conversion(value: &ExifValue) -> String {
    // Handle various input formats
    match value {
        ExifValue::U16Array(arr) if arr.len() >= 2 => {
            format!("{}x{}", arr[0], arr[1])
        }
        ExifValue::Rational(num, den) => {
            if *den == 0 {
                "undef".to_string()
            } else {
                let result = *num as f64 / *den as f64;
                format!("{:.1}", result)
            }
        }
        _ => exif_value_to_string(value),
    }
}
```

### 4. Updating Tag Table

```rust
// In src/tables/exif_tags.rs, change PrintConvId::None to appropriate conversion:
ExifTag {
    id: 0x1234,
    name: "MyTag",
    print_conv: PrintConvId::MyNewConversion, // Changed from PrintConvId::None
},
```

### 5. Shared Lookup Optimization

```rust
// For patterns used by multiple tags, create shared functions:
fn shared_quality_lookup(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "Best".to_string(),
        Some(2) => "Better".to_string(),  
        Some(3) => "Good".to_string(),
        Some(4) => "Normal".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

// Then reuse in multiple PrintConvId variants:
PrintConvId::CanonQuality => shared_quality_lookup(value),
PrintConvId::NikonQuality => shared_quality_lookup(value),
PrintConvId::SonyQuality => shared_quality_lookup(value),
```

## Testing Strategy

### Unit Tests

Add tests to [`src/core/print_conv.rs:1127-1264`](../src/core/print_conv.rs):

```rust
#[test]
fn test_my_new_conversion() {
    assert_eq!(
        apply_print_conv(&ExifValue::U32(0), PrintConvId::MyNewConversion),
        "Off"
    );
    assert_eq!(
        apply_print_conv(&ExifValue::U32(1), PrintConvId::MyNewConversion), 
        "On"
    );
}
```

### Integration Tests

```bash
# Test with real images
cargo run -- test_images/canon.jpg | jq '.Make'
cargo run -- test_images/nikon.jpg | jq '.Model' 

# Compare with ExifTool
exiftool -Make -Model test_images/canon.jpg
```

### ExifTool Synchronization

**CRITICAL**: Every PrintConv implementation MUST match ExifTool's output exactly.

```bash
# Validation script template
#!/bin/bash
IMAGE="test.jpg"
TAG="Flash"

EXIFTOOL_VALUE=$(exiftool -$TAG -s3 "$IMAGE")
OXIDE_VALUE=$(cargo run -- "$IMAGE" | jq -r ".$TAG")

if [ "$EXIFTOOL_VALUE" = "$OXIDE_VALUE" ]; then
    echo "✅ $TAG: $OXIDE_VALUE" 
else
    echo "❌ $TAG: ExifTool='$EXIFTOOL_VALUE' Oxide='$OXIDE_VALUE'"
fi
```

## Success Metrics 📊

### Phase 1 Success
- [ ] 67 ★★★★ ✔ priority tags have proper PrintConv (vs `PrintConvId::None`)
- [ ] Core EXIF metadata displays human-readable values
- [ ] All tests pass: `cargo test print_conv`

### Phase 2 Success  
- [ ] 200+ universal PrintConv patterns implemented
- [ ] Cross-manufacturer compatibility verified
- [ ] Shared lookup optimizations in place

### Phase 3 Success
- [ ] 300+ manufacturer-specific patterns implemented
- [ ] Canon, Nikon, Sony high-priority tags covered
- [ ] ExifTool output matching verified

### Phase 4 Success
- [ ] <100 `PrintConvId::None` entries remaining (90%+ coverage)
- [ ] GPMF metadata properly formatted
- [ ] All manufacturer tag tables complete

## File Reference Quick Guide 📁

### Core Implementation Files
- **[`src/core/print_conv.rs`](../src/core/print_conv.rs)** - Main PrintConv implementation
- **[`src/core/types.rs`](../src/core/types.rs)** - ExifValue definitions

### Tag Table Files  
- **[`src/tables/exif_tags.rs`](../src/tables/exif_tags.rs)** - EXIF tags (592 None entries)
- **[`src/tables/canon_tags.rs`](../src/tables/canon_tags.rs)** - Canon maker note tags
- **[`src/tables/nikon_tags.rs`](../src/tables/nikon_tags.rs)** - Nikon maker note tags
- **[`src/tables/sony_tags.rs`](../src/tables/sony_tags.rs)** - Sony maker note tags
- **[`src/tables/pentax_tags.rs`](../src/tables/pentax_tags.rs)** - Pentax tags (example implementation)
- **[`src/tables/olympus_tags.rs`](../src/tables/olympus_tags.rs)** - Olympus maker note tags
- **[`src/tables/panasonic_tags.rs`](../src/tables/panasonic_tags.rs)** - Panasonic tags (118 None entries)
- **[`src/tables/fujifilm_tags.rs`](../src/tables/fujifilm_tags.rs)** - Fujifilm tags (85 None entries)
- **[`src/tables/apple_tags.rs`](../src/tables/apple_tags.rs)** - Apple tags (37 None entries)

### Automation Tools
- **[`src/bin/exiftool_sync/extractors/printconv_analyzer.rs`](../src/bin/exiftool_sync/extractors/printconv_analyzer.rs)** - Pattern analysis
- **[`src/bin/exiftool_sync/extractors/printconv_generator.rs`](../src/bin/exiftool_sync/extractors/printconv_generator.rs)** - Code generation
- **[`src/bin/exiftool_sync/extractors/printconv_tables.rs`](../src/bin/exiftool_sync/extractors/printconv_tables.rs)** - Table extraction

### Reference Documentation
- **[`doc/PRINTCONV-ARCHITECTURE.md`](PRINTCONV-ARCHITECTURE.md)** - Complete technical architecture
- **[`doc/EXIFTOOL-SYNC.md`](EXIFTOOL-SYNC.md)** - Synchronization process
- **[`third-party/exiftool-vendored.js/src/Tags.ts`](../third-party/exiftool-vendored.js/src/Tags.ts)** - Priority markers

### ExifTool Reference
- **[`third-party/exiftool/lib/Image/ExifTool/Exif.pm`](../third-party/exiftool/lib/Image/ExifTool/Exif.pm)** - Core EXIF conversions
- **[`third-party/exiftool/lib/Image/ExifTool/Canon.pm`](../third-party/exiftool/lib/Image/ExifTool/Canon.pm)** - Canon conversions
- **[`third-party/exiftool/lib/Image/ExifTool/Nikon.pm`](../third-party/exiftool/lib/Image/ExifTool/Nikon.pm)** - Nikon conversions
- **[`third-party/exiftool/lib/Image/ExifTool/GoPro.pm`](../third-party/exiftool/lib/Image/ExifTool/GoPro.pm)** - GPMF conversions

## Getting Started Checklist ✅

Before starting implementation:

- [ ] Read [`doc/PRINTCONV-ARCHITECTURE.md`](PRINTCONV-ARCHITECTURE.md) completely
- [ ] Read [`doc/EXIFTOOL-SYNC.md`](EXIFTOOL-SYNC.md) for attribution requirements  
- [ ] Review current implementation in [`src/core/print_conv.rs`](../src/core/print_conv.rs)
- [ ] Understand tag table structure in [`src/tables/pentax_tags.rs`](../src/tables/pentax_tags.rs) (working example)
- [ ] Set up test environment with sample images
- [ ] Verify ExifTool comparison workflow

Start with Phase 1 (Core EXIF PrintConv) for maximum impact across all manufacturers.

**Next Engineer**: This document provides complete technical context and step-by-step implementation guidance. The table-driven PrintConv architecture enables rapid implementation with automated tools - this is a well-architected system ready for systematic completion.

---
**Document Version**: 2025-06-24  
**Estimated Effort**: 4 weeks (1 week per phase)  
**Impact**: Transform raw numeric metadata into human-readable strings across entire application