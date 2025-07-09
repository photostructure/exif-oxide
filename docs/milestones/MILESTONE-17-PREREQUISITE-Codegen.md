# Milestone 18: RAW Format Codegen Extraction

**Duration**: 1-2 weeks  
**Goal**: Add all RAW format lookup tables to the codegen extraction system

## Overview

This milestone implements codegen extraction for all lookup tables identified in the RAW format analysis. Rather than manually maintaining thousands of lookup table entries across all manufacturers, we'll use the simple table extraction framework to generate them automatically from ExifTool source.

**Impact**: This will eliminate **~3000+ manually maintained lookup table entries** across all RAW format implementations, dramatically reducing maintenance burden and ensuring automatic updates with each ExifTool release.

## Background

Analysis of RAW format milestones revealed extensive lookup tables across all manufacturers:

- **Canon**: ~140 lookup tables (1000+ lens types, 367 camera models, 130+ smaller tables)
- **Nikon**: ~85 lookup tables (618 lens IDs, multiple AF point mappings, 80+ camera settings)
- **Sony**: ~35 lookup tables (white balance, AF points, exposure programs, picture effects)
- **Olympus**: ~25 lookup tables (lens types, camera types, filters, scene modes, 15+ others)
- **Panasonic**: ~15 lookup tables (white balance, CFA patterns, compression modes)
- **Minolta**: ~10 lookup tables (WB mode, storage methods, program modes)
- **Kyocera**: ~5 lookup tables (ISO settings, basic modes)

All of these are simple key-value mappings perfect for the simple table extraction framework.

## Implementation Strategy

### Phase 1: High Priority Large Tables (Week 1)

**Massive Lookup Tables (1000+ entries)**:

```json
// Add to codegen/extract.json
{
  "module": "Canon.pm",
  "hash_name": "%canonLensTypes",
  "output_file": "canon/lens_types.rs",
  "constant_name": "CANON_LENS_TYPES",
  "key_type": "u16",
  "extraction_type": "simple_table",
  "description": "Canon lens type IDs to lens names"
},
{
  "module": "Canon.pm", 
  "hash_name": "%canonModelID",
  "output_file": "canon/camera_models.rs",
  "constant_name": "CANON_CAMERA_MODELS",
  "key_type": "u32",
  "extraction_type": "simple_table",
  "description": "Canon camera model IDs to camera names"
},
{
  "module": "Nikon.pm",
  "hash_name": "%nikonLensIDs", 
  "output_file": "nikon/lens_ids.rs",
  "constant_name": "NIKON_LENS_IDS",
  "key_type": "string",
  "extraction_type": "simple_table",
  "description": "Nikon lens signatures to lens names"
},
{
  "module": "Olympus.pm",
  "hash_name": "%olympusCameraTypes",
  "output_file": "olympus/camera_types.rs", 
  "constant_name": "OLYMPUS_CAMERA_TYPES",
  "key_type": "string",
  "extraction_type": "simple_table",
  "description": "Olympus camera type codes to camera names"
},
{
  "module": "Olympus.pm",
  "hash_name": "%olympusLensTypes",
  "output_file": "olympus/lens_types.rs",
  "constant_name": "OLYMPUS_LENS_TYPES", 
  "key_type": "string",
  "extraction_type": "simple_table",
  "description": "Olympus lens type codes to lens names"
}
```

### Phase 2: Medium Priority Tables (Week 1-2)

**Camera-Specific Settings (20-100 entries)**:

```json
// Sony lookup tables
{
  "module": "Sony.pm",
  "hash_name": "%whiteBalanceSetting",
  "output_file": "sony/white_balance.rs",
  "constant_name": "SONY_WHITE_BALANCE_SETTING",
  "key_type": "u16",
  "extraction_type": "simple_table",
  "description": "Sony white balance settings with fine-tune adjustments"
},
{
  "module": "Sony.pm",
  "hash_name": "%afPoints79",
  "output_file": "sony/af_points_79.rs",
  "constant_name": "SONY_AF_POINTS_79",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Sony 79-point AF system point mappings"
},
{
  "module": "Sony.pm",
  "hash_name": "%isoSetting2010",
  "output_file": "sony/iso_setting_2010.rs",
  "constant_name": "SONY_ISO_SETTING_2010",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Sony ISO setting values to actual ISO numbers"
},
{
  "module": "Sony.pm",
  "hash_name": "%sonyExposureProgram",
  "output_file": "sony/exposure_program.rs",
  "constant_name": "SONY_EXPOSURE_PROGRAM",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Sony exposure program modes"
},
{
  "module": "Sony.pm",
  "hash_name": "%pictureProfile2010",
  "output_file": "sony/picture_profile_2010.rs",
  "constant_name": "SONY_PICTURE_PROFILE_2010",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Sony picture profile settings"
},

// Nikon lookup tables
{
  "module": "Nikon.pm",
  "hash_name": "%afPoints153",
  "output_file": "nikon/af_points_153.rs",
  "constant_name": "NIKON_AF_POINTS_153",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Nikon 153-point AF system point mappings"
},
{
  "module": "Nikon.pm",
  "hash_name": "%afPoints135",
  "output_file": "nikon/af_points_135.rs",
  "constant_name": "NIKON_AF_POINTS_135",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Nikon 135-point AF system point mappings"
},
{
  "module": "Nikon.pm",
  "hash_name": "%afPoints105",
  "output_file": "nikon/af_points_105.rs",
  "constant_name": "NIKON_AF_POINTS_105",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Nikon 105-point AF system point mappings"
},
{
  "module": "Nikon.pm",
  "hash_name": "%iSOAutoShutterTimeZ9",
  "output_file": "nikon/iso_auto_shutter_time_z9.rs",
  "constant_name": "NIKON_ISO_AUTO_SHUTTER_TIME_Z9",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Nikon Z9 ISO auto shutter time settings"
},

// Olympus lookup tables
{
  "module": "Olympus.pm",
  "hash_name": "%filters",
  "output_file": "olympus/filters.rs",
  "constant_name": "OLYMPUS_FILTERS",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Olympus art filter modes"
},

// Canon lookup tables
{
  "module": "Canon.pm",
  "hash_name": "%canonWhiteBalance",
  "output_file": "canon/white_balance.rs",
  "constant_name": "CANON_WHITE_BALANCE",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Canon white balance settings"
},
{
  "module": "Canon.pm",
  "hash_name": "%pictureStyles",
  "output_file": "canon/picture_styles.rs",
  "constant_name": "CANON_PICTURE_STYLES",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Canon picture style modes"
}
```

### Phase 3: Comprehensive Manufacturer Coverage (Week 2)

**All Remaining Tables by Manufacturer**:

```json
// Panasonic tables
{
  "module": "PanasonicRaw.pm",
  "hash_name": "%panasonicWhiteBalance",
  "output_file": "panasonic/white_balance.rs",
  "constant_name": "PANASONIC_WHITE_BALANCE",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Panasonic white balance settings"
},

// Minolta tables
{
  "module": "MinoltaRaw.pm",
  "hash_name": "%mrwWB",
  "output_file": "minolta/wb_mode.rs",
  "constant_name": "MINOLTA_WB_MODE",
  "key_type": "u8", 
  "extraction_type": "simple_table",
  "description": "Minolta white balance mode conversion"
},

// Kyocera tables
{
  "module": "KyoceraRaw.pm",
  "hash_name": "%isoConv",
  "output_file": "kyocera/iso_conversion.rs",
  "constant_name": "KYOCERA_ISO_CONVERSION",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Kyocera ISO value conversion"
},

// Additional RAW format tables
{
  "module": "CanonRaw.pm",
  "hash_name": "%targetImageType",
  "output_file": "canon_raw/target_image_type.rs",
  "constant_name": "CANON_RAW_TARGET_IMAGE_TYPE",
  "key_type": "u8",
  "extraction_type": "simple_table",
  "description": "Canon RAW target image type"
},
{
  "module": "SigmaRaw.pm",
  "hash_name": "%driveMode",
  "output_file": "sigma_raw/drive_mode.rs",
  "constant_name": "SIGMA_RAW_DRIVE_MODE",
  "key_type": "string",
  "extraction_type": "simple_table",
  "description": "Sigma RAW drive mode settings"
}
```

### Phase 4: PrintConv Table Extraction (Week 2)

**Inline PrintConv Tables** (extract from within tag definitions):

```json
// Extract PrintConv blocks from tag definitions
{
  "module": "Olympus.pm",
  "tag_id": "0x0403",
  "field": "PrintConv",
  "output_file": "olympus/scene_mode.rs",
  "constant_name": "OLYMPUS_SCENE_MODE",
  "key_type": "u16",
  "extraction_type": "printconv_table",
  "description": "Olympus scene mode settings"
},
{
  "module": "Sony.pm",
  "tag_id": "0x1002",
  "field": "PrintConv", 
  "output_file": "sony/picture_effect.rs",
  "constant_name": "SONY_PICTURE_EFFECT",
  "key_type": "u16",
  "extraction_type": "printconv_table",
  "description": "Sony picture effect modes"
}
```

## Implementation Details

### Build System Updates

```bash
# Update Makefile to include new extractions
make codegen-extract  # Extract all new lookup tables
make codegen-gen      # Generate Rust code from extractions
make precommit        # Verify everything compiles
```

### Generated Code Structure

```rust
// Example: src/generated/canon/lens_types.rs
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Canon lens type IDs to lens names
/// Generated from ExifTool Canon.pm %canonLensTypes
pub static CANON_LENS_TYPES: Lazy<HashMap<u16, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "Canon EF 50mm f/1.8");
    map.insert(2, "Canon EF 28mm f/2.8");
    map.insert(3, "Canon EF 135mm f/2.8 Soft");
    // ... 1000+ more entries
    map
});

/// Lookup Canon lens name by lens type ID
pub fn lookup_canon_lens_type(id: u16) -> Option<&'static str> {
    CANON_LENS_TYPES.get(&id).copied()
}
```

### Usage in Implementation Code

```rust
// In implementations/canon/print_conv.rs
use crate::generated::canon::lens_types::lookup_canon_lens_type;

pub fn canon_lens_type_print_conv(value: &TagValue) -> TagValue {
    if let Some(lens_id) = value.as_u16() {
        if let Some(lens_name) = lookup_canon_lens_type(lens_id) {
            return TagValue::string(lens_name);
        }
    }
    TagValue::string(format!("Unknown lens type ({})", value))
}
```

## Success Criteria

### Core Requirements

- [ ] **Large Table Extraction**: All 5 major lookup tables (1000+ entries) successfully extracted
- [ ] **Medium Table Extraction**: 20+ medium-sized tables (20-100 entries) extracted
- [ ] **Comprehensive Coverage**: All manufacturer lookup tables included
- [ ] **PrintConv Integration**: Inline PrintConv tables properly extracted
- [ ] **Build System**: Automated extraction and generation process
- [ ] **Generated Code**: All tables generate proper Rust code

### Validation Tests

- [ ] **Extract All Tables**: `make codegen-extract` completes without errors
- [ ] **Generate Code**: `make codegen-gen` produces valid Rust code
- [ ] **Compilation**: All generated code compiles without warnings
- [ ] **Lookup Functions**: Generated lookup functions work correctly
- [ ] **Table Completeness**: Verify all identified tables are extracted

## Implementation Boundaries

### Goals (Milestone 18)

- Extract all RAW format lookup tables to codegen
- Eliminate manual maintenance of 3000+ lookup entries
- Set up automated generation from ExifTool source
- Create complete manufacturer coverage

### Non-Goals

- Modify RAW format implementations (future milestones)
- Add new extraction types beyond simple_table/printconv_table
- Handle complex conversion logic (only static mappings)

## Dependencies and Prerequisites

- Working codegen extraction system
- Simple table extraction framework
- ExifTool source code access
- Understanding of manufacturer lookup patterns

## Technical Notes

### Extraction Strategy

1. **Hash Variables**: Use existing simple_table extraction for `my %name = (...)` patterns
2. **PrintConv Blocks**: Extend system to extract PrintConv tables from tag definitions
3. **Key Type Detection**: Automatically detect appropriate key types (u8, u16, u32, string)
4. **Validation**: Ensure all extracted tables contain only static mappings

### Build Process

```bash
# Full extraction pipeline
make codegen-extract     # Extract all tables from ExifTool
make codegen-gen         # Generate Rust lookup code
make test-codegen        # Verify generated code works
make precommit           # Final validation
```

### Error Handling

- **Missing Tables**: Gracefully handle tables that don't exist in ExifTool version
- **Format Changes**: Detect and report format changes in ExifTool
- **Invalid Keys**: Skip entries with non-static keys
- **Validation**: Ensure all generated code compiles

## Benefits

### Immediate Benefits

1. **Eliminated Manual Work**: No more manual maintenance of 3000+ lookup entries
2. **Automatic Updates**: Tables update automatically with each ExifTool release
3. **Reduced Errors**: No more typos or missed entries in manual tables
4. **Consistency**: All tables follow same generation pattern

### Long-term Benefits

1. **Scalability**: Easy to add new manufacturers and tables
2. **Maintainability**: Single source of truth (ExifTool)
3. **Reliability**: Generated code is always in sync with ExifTool
4. **Development Speed**: Faster RAW format implementation

## Risk Mitigation

### Table Format Changes

- **Risk**: ExifTool changes table format
- **Mitigation**: Robust parsing with error handling
- **Recovery**: Graceful fallback to manual extraction

### Key Type Mismatches

- **Risk**: Generated key types don't match usage
- **Mitigation**: Type validation during generation
- **Testing**: Compilation errors catch mismatches

### Missing Dependencies

- **Risk**: Required ExifTool variables not accessible
- **Mitigation**: Auto-patching system makes variables accessible
- **Validation**: Check all required variables exist

## Summary

This milestone eliminates the largest source of manual maintenance in the RAW format implementation by automatically generating all lookup tables from ExifTool source. The ~3000+ lookup table entries across all manufacturers will be generated automatically, ensuring they stay in sync with ExifTool updates and eliminating a massive maintenance burden.

Success here enables rapid implementation of all RAW format milestones since the lookup tables won't need to be manually maintained.