# Universal RAW Format Codegen Extractors Implementation and Migration Plan

## 🎯 Executive Summary

This milestone implements **4 universal codegen extractors** that eliminate **1000+ lines of manual maintenance** across ALL RAW format implementations (Canon, Nikon, Olympus, Panasonic, Minolta, Sony, etc.). The extractors automatically generate Rust code from ExifTool source, ensuring perfect compatibility while dramatically reducing future maintenance burden.

**Key Impact**:
- **95% reduction** in manual lookup table maintenance
- **Automatic support** for new ExifTool releases  
- **Universal applicability** across all manufacturers
- **Future-proofs** Nikon, Sony, and other RAW implementations

## 📋 Problem Statement

### Current Manual Maintenance Burden

Every RAW format implementation currently requires extensive manual porting:

**Canon (`canon.rs` - 871 lines)**:
```rust
// ❌ MANUAL MAINTENANCE - 215+ lines of enum variants
pub enum CanonDataType {
    CameraSettings,     // 0x0001
    FocalLength,        // 0x0002
    ShotInfo,           // 0x0003
    // ... 20+ more variants requiring manual updates
}

impl CanonDataType {
    pub fn tag_id(&self) -> u16 {
        match self {
            CanonDataType::CameraSettings => 0x0001,
            CanonDataType::FocalLength => 0x0002,
            // ... 20+ more manual mappings
        }
    }
}
```

**Olympus (`olympus.rs` - 332 lines)**:
```rust
// ❌ MANUAL MAINTENANCE - Hardcoded section mappings
supported_sections.insert(0x2010, "Equipment");
supported_sections.insert(0x2020, "CameraSettings");
supported_sections.insert(0x2030, "RawDevelopment");
// ... 9 hardcoded sections requiring manual updates
```

**Multiplied across ALL manufacturers** = **1000+ lines of manual maintenance**

### The Universal Pattern

After analyzing Canon.pm, Nikon.pm, Olympus.pm, Panasonic.pm, and Minolta.pm, **ALL manufacturers share identical patterns**:

1. **Tag Table Structure**: `%Image::ExifTool::Manufacturer::Main` hash tables
2. **ProcessBinaryData Tables**: Binary data parsing definitions  
3. **Model Detection Patterns**: Regex-based camera model detection
4. **Conditional Tag Definitions**: Array-based multi-variant tags

**This universality enables a single set of extractors to handle ALL manufacturers.**

## 🔧 Technical Analysis: Universal Applicability

### Pattern 1: Main Tag Table Structure (100% Universal)

**Canon.pm**:
```perl
%Image::ExifTool::Canon::Main = (
    0x1 => { Name => 'CanonCameraSettings', SubDirectory => {...} },
    0x2 => { Name => 'CanonFocalLength', SubDirectory => {...} },
    # ... 80+ tag definitions
);
```

**Nikon.pm**:
```perl
%Image::ExifTool::Nikon::Main = (
    0x0001 => { Name => 'MakerNoteVersion', Writable => 'undef', ... },
    0x0002 => { Name => 'ISO', Writable => 'int16u', ... },
    # ... 100+ tag definitions
);
```

**Olympus.pm**:
```perl
%Image::ExifTool::Olympus::Main = (
    0x2010 => { Name => 'Equipment', SubDirectory => {...} },
    0x2020 => { Name => 'CameraSettings', SubDirectory => {...} },
    # ... 576+ tag definitions
);
```

**ALL use identical structure**: `tag_id => { Name, Format, Groups, Conditions, SubDirectory, ... }`

### Pattern 2: ProcessBinaryData Tables (100% Universal)

**Canon.pm**: 169 ProcessBinaryData tables
**Nikon.pm**: 25+ ProcessBinaryData tables  
**Olympus.pm**: 29 ProcessBinaryData tables
**Panasonic.pm**: 20+ ProcessBinaryData tables

**ALL use identical ProcessBinaryData structure**:
```perl
%Image::ExifTool::Manufacturer::SectionName = (
    PROCESS_PROC => \&Image::ExifTool::ProcessBinaryData,
    FORMAT => 'int16s',
    FIRST_ENTRY => 1,
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    1 => 'TagName1',
    2 => { Name => 'TagName2', PrintConv => {...} },
    # ... field definitions
);
```

### Pattern 3: Model Detection Patterns (100% Universal)

**Canon**: `($model =~ /\b(20D|350D|REBEL XT|Kiss Digital N)\b/) ? 6 : 4`
**Nikon**: `$$self{Model} =~ /E775/` conditionals
**Olympus**: `$$self{Model} =~ /E-M1/` conditionals  
**Panasonic**: `$$self{Make} =~ /^SONY/` brand detection

**ALL use regex-based model detection with conditional processing**

### Pattern 4: Conditional Tag Definitions (100% Universal)

**Canon**:
```perl
0xc => [   # Array syntax for conditionals
    { Name => 'SerialNumber', Condition => '$$self{Model} =~ /EOS D30\b/', ... },
    { Name => 'SerialNumber', Condition => '$$self{Model} =~ /EOS-1D/', ... },
];
```

**Nikon, Olympus, Panasonic ALL use identical array-based conditional syntax**

## 🏗️ Existing Codegen Infrastructure

The universal extractors build upon exif-oxide's mature codegen infrastructure:

### Current Extractors ([`codegen/extractors/`](../../codegen/extractors/))
1. **[simple_table.pl](../../codegen/extractors/simple_table.pl)** - Extracts primitive lookup tables (e.g., %canonLensTypes)
2. **[tag_tables.pl](../../codegen/extractors/tag_tables.pl)** - Extracts EXIF/GPS tag definitions from Main tables
3. **[inline_printconv.pl](../../codegen/extractors/inline_printconv.pl)** - Extracts PrintConv definitions embedded in tag tables
4. **[tag_definitions.pl](../../codegen/extractors/tag_definitions.pl)** - Config-driven tag definition extraction with filtering
5. **[composite_tags.pl](../../codegen/extractors/composite_tags.pl)** - Extracts composite tag definitions
6. **[file_type_lookup.pl](../../codegen/extractors/file_type_lookup.pl)** - File type detection patterns
7. **[regex_patterns.pl](../../codegen/extractors/regex_patterns.pl)** - Regex pattern extraction for file detection
8. **[boolean_set.pl](../../codegen/extractors/boolean_set.pl)** - Boolean set membership extraction

### Infrastructure Components
- **Orchestration**: [`codegen/src/extraction.rs`](../../codegen/src/extraction.rs) auto-discovers configs and runs extractors
- **Utilities**: [`codegen/lib/ExifToolExtract.pm`](../../codegen/lib/ExifToolExtract.pm) provides Perl parsing utilities
- **Patching**: [`codegen/patches/patch_exiftool_modules.pl`](../../codegen/patches/patch_exiftool_modules.pl) converts `my` to `our` variables
- **Config System**: Module-specific configs in [`codegen/config/{Module}_pm/`](../../codegen/config/)
- **Generators**: Rust code generators in [`codegen/src/generators/`](../../codegen/src/generators/)

### Key Design Principles
1. **No Perl parsing in Rust** - Only Perl can parse Perl correctly
2. **Config-driven extraction** - JSON configs specify what to extract
3. **Atomic operations** - Patch, extract, revert in single operation
4. **Type safety** - Strong typing from extraction through generation

The universal extractors follow these established patterns while adding new capabilities for RAW format support.

## 🎯 Solution: 4 Universal Codegen Extractors

### Extractor 1: Tag Table Structure Extractor
**NEW EXTRACTOR**: `tag_table_structure.pl`

**Purpose**: Extract complete %Manufacturer::Main table structures from ANY manufacturer

**Input**: Any ExifTool manufacturer module (Canon.pm, Nikon.pm, etc.)
**Output**: Complete tag enum with all metadata

**Generated Code**:
```rust
// Generated from Canon.pm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonDataType {
    CameraSettings,       // 0x0001
    FocalLength,          // 0x0002  
    ShotInfo,             // 0x0003
    // ... ALL variants auto-generated
}

impl CanonDataType {
    pub fn tag_id(&self) -> u16 { /* generated */ }
    pub fn from_tag_id(tag_id: u16) -> Option<Self> { /* generated */ }
    pub fn name(&self) -> &'static str { /* generated */ }
    pub fn has_subdirectory(&self) -> bool { /* generated */ }
    pub fn groups(&self) -> (&'static str, &'static str) { /* generated */ }
}
```

**Replaces**: 215+ lines in canon.rs, similar amounts in olympus.rs, etc.

### Extractor 2: ProcessBinaryData Table Extractor  
**NEW EXTRACTOR**: `binary_data_tables.pl`

**Purpose**: Extract ALL ProcessBinaryData definitions from ANY manufacturer

**Input**: Any ExifTool manufacturer module
**Output**: Binary data parsing structure definitions

**Generated Code**:
```rust
// Generated from Canon.pm CameraSettings table
pub struct CanonCameraSettingsProcessor {
    fields: HashMap<u16, BinaryFieldDef>,
}

impl CanonCameraSettingsProcessor {
    pub fn new() -> Self {
        let mut fields = HashMap::new();
        fields.insert(1, BinaryFieldDef {
            name: "MacroMode",
            format: BinaryFormat::Int16s,
            print_conv: Some(PrintConvType::MacroMode),
        });
        // ... ALL fields auto-generated
        Self { fields }
    }
}
```

**Replaces**: Future binary processing implementations (saves 100s of lines per manufacturer)

### Extractor 3: Model Detection Pattern Extractor
**NEW EXTRACTOR**: `model_patterns.pl`

**Purpose**: Extract camera model detection patterns from ANY manufacturer

**Input**: ExifTool source files (multiple modules)
**Output**: Model pattern arrays and detection logic

**Generated Code**:
```rust
// Generated from Canon.pm patterns
pub static CANON_6_BYTE_MODELS: &[&str] = &[
    "EOS 20D",
    "EOS 350D", 
    "EOS REBEL XT",
    "EOS Kiss Digital N",
];

pub fn detect_canon_offset_scheme(model: &str) -> CanonOffsetScheme {
    if CANON_6_BYTE_MODELS.iter().any(|&m| model.contains(m)) {
        CanonOffsetScheme::Bytes6
    } else if model.contains("PowerShot") || model.contains("IXUS") {
        CanonOffsetScheme::Bytes16
    } else {
        CanonOffsetScheme::Bytes4
    }
}
```

**Replaces**: Manual pattern detection in CanonOffsetManager (80+ lines)

### Extractor 4: Conditional Tag Definition Extractor
**NEW EXTRACTOR**: `conditional_tags.pl`

**Purpose**: Extract conditional tag definitions using array syntax

**Input**: ExifTool manufacturer modules
**Output**: Conditional tag processing logic

**Generated Code**:
```rust
// Generated from Canon.pm conditional definitions
pub enum CanonConditionalTag {
    SerialNumber {
        model_pattern: &'static str,
        print_conv: SerialNumberFormat,
    },
}

pub fn process_conditional_tag(tag_id: u16, model: &str) -> Option<TagDefinition> {
    match tag_id {
        0x000c => {
            if model.contains("EOS D30") {
                Some(TagDefinition::serial_number_d30())
            } else if model.contains("EOS-1D") {
                Some(TagDefinition::serial_number_1d())
            } else {
                Some(TagDefinition::serial_number_default())
            }
        }
        _ => None,
    }
}
```

**Replaces**: Manual conditional logic throughout manufacturer implementations

## 📋 Configuration Examples

The universal extractors follow the existing config-driven pattern. Place configs in `codegen/config/{Module}_pm/`:

### Tag Table Structure Config (`tag_table_structure.json`)
```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool/Canon.pm",
  "description": "Canon Main tag table structure for enum generation",
  "table": "Main",
  "output": {
    "enum_name": "CanonDataType",
    "include_metadata": true,
    "generate_methods": ["tag_id", "from_tag_id", "name", "has_subdirectory", "groups"]
  }
}
```

### Binary Data Tables Config (`binary_data_tables.json`)
```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool/Canon.pm",
  "description": "Canon ProcessBinaryData table extraction",
  "tables": [
    {
      "table_name": "CameraSettings",
      "processor_name": "CanonCameraSettingsProcessor"
    },
    {
      "table_name": "ShotInfo",
      "processor_name": "CanonShotInfoProcessor"
    }
  ],
  "filters": {
    "process_proc": "ProcessBinaryData"
  }
}
```

### Model Patterns Config (`model_patterns.json`)
```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool/Canon.pm",
  "description": "Canon camera model detection patterns",
  "pattern_types": [
    {
      "name": "6_byte_offset_models",
      "constant_name": "CANON_6_BYTE_MODELS",
      "pattern": "\\b(20D|350D|REBEL XT|Kiss Digital N)\\b"
    }
  ],
  "detection_function": {
    "name": "detect_canon_offset_scheme",
    "return_type": "CanonOffsetScheme"
  }
}
```

### Conditional Tags Config (`conditional_tags.json`)
```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool/Canon.pm",
  "description": "Canon conditional tag definitions",
  "conditional_tags": [
    {
      "tag_id": "0x000c",
      "tag_name": "SerialNumber",
      "conditions": [
        {
          "model_pattern": "EOS D30\\b",
          "variant_name": "serial_number_d30"
        }
      ]
    }
  ]
}
```

## 📊 Implementation Phases

### Phase 1: Tag Table Structure Extractor (Week 1)
1. Create `codegen/extractors/tag_table_structure.pl` based on existing patterns
2. Add config support in `extraction.rs` for `tag_table_structure.json` files
3. Test with Canon.pm (80+ tags), validate enum generation
4. Create Rust generator in `codegen/src/generators/tag_structure.rs`

### Phase 2: ProcessBinaryData Extractor (Week 2)
1. Create `codegen/extractors/binary_data_tables.pl`
2. Handle PROCESS_PROC, FORMAT, FIRST_ENTRY metadata extraction
3. Test with Canon's 169 ProcessBinaryData tables
4. Generate processor structs with field definitions

### Phase 3: Model & Conditional Extractors (Week 3)
1. Create `model_patterns.pl` for camera model detection
2. Create `conditional_tags.pl` for array-based conditionals
3. Test detection patterns across manufacturers
4. Generate model detection functions and conditional logic

### Phase 4: Migration & Validation (Week 4)
1. **Canon**: Replace manual CanonDataType enum (validate with CR2 files)
2. **Olympus**: Replace hardcoded mappings (validate with ORF files)
3. **Full testing**: Compare output with manual implementations
4. **Documentation**: Update integration guide for future manufacturers

## 🔄 Migration Strategy for Existing Code

### Before/After Examples

**Canon Tag Structure (`canon.rs`)**:
```rust
// ❌ Before: 215+ lines of manual enum maintenance
pub enum CanonDataType {
    CameraSettings,     // 0x0001
    FocalLength,        // 0x0002
    ShotInfo,           // 0x0003
    // ... 20+ more variants
}

// ✅ After: Auto-generated from Canon.pm
use crate::generated::canon::tag_structure::CanonDataType;
```

**Olympus Section Mappings (`olympus.rs`)**:
```rust
// ❌ Before: Hardcoded mappings
supported_sections.insert(0x2010, "Equipment");
supported_sections.insert(0x2020, "CameraSettings");

// ✅ After: Generated metadata
use crate::generated::olympus::tag_structure::OlympusTagMetadata;
```

### Migration Targets by Manufacturer
- **Canon**: Replace CanonDataType enum and CanonOffsetManager (295+ lines)
- **Olympus**: Replace hardcoded section mappings (80+ lines)
- **Minolta**: Replace manual PRD/WBG/RIF processors (400+ lines)
- **Panasonic**: Replace manual tag definitions (150+ lines)

## 📚 Updated Milestone Documentation

### MILESTONE-17-RAW-Format-Support.md Updates

**Current Prerequisites Section**:
```markdown
### Critical Prerequisites

- **⚠️ MANDATORY: MILESTONE-17-PREREQUISITE-Codegen.md MUST BE COMPLETED FIRST**
```

**Updated Prerequisites Section**:
```markdown
### Critical Prerequisites

- **⚠️ MANDATORY: MILESTONE-17-UNIVERSAL-CODEGEN-EXTRACTORS.md MUST BE COMPLETED FIRST**
  - **Why**: Universal extractors eliminate 1000+ lines of manual maintenance across ALL manufacturers
  - **Impact**: Canon, Nikon, Olympus, Panasonic implementations become 95% automated
  - **Scope**: Works for ALL manufacturers, not just Canon
  - **Generated Code**: Tag structures, binary data processors, model detection, conditional logic
```

### MILESTONE-17c-Olympus-RAW.md Updates

**Add New Section**:
```markdown
## 🔧 INTEGRATION WITH UNIVERSAL EXTRACTORS

**Post-Extractor Implementation**: This milestone's manual implementations will be replaced with generated code:

**Generated Replacements**:
- `supported_sections` HashMap → `crate::generated::olympus::tag_structure::OlympusSubdirectories`
- `get_olympus_tag_name()` → `crate::generated::olympus::tag_structure::OlympusTagMetadata::tag_name()`
- Hardcoded 0x2010-0x5000 ranges → Auto-generated from Olympus.pm Main table

**Migration Timeline**: Phase 2B (post-extractor completion)
```

### MILESTONE-17d-Canon-RAW.md Updates

**Add New Section**:
```markdown
## 🔧 INTEGRATION WITH UNIVERSAL EXTRACTORS

**Generated Replacements for Manual Code**:

**169 ProcessBinaryData Sections** → `crate::generated::canon::binary_data::*`
- Each ProcessBinaryData table becomes a generated processor struct
- Automatic field definitions, format specifications, PrintConv handling

**CanonDataType Enum** → `crate::generated::canon::tag_structure::CanonDataType`  
- All 23+ variants automatically generated from Canon.pm Main table
- tag_id(), from_tag_id(), name() methods auto-generated

**Model Detection Patterns** → `crate::generated::canon::model_patterns::*`
- detect_canon_offset_scheme() function generated from ExifTool patterns
- CANON_6_BYTE_MODELS, CANON_16_BYTE_MODELS arrays auto-generated

**Implementation Note**: Manual implementations serve as bridge until extractors complete.
```

### New: MILESTONE-17-PREREQUISITE-Universal-Codegen.md

**Replaces**: MILESTONE-17-PREREQUISITE-Codegen.md (Canon-specific)

**Content**:
```markdown
# Universal RAW Format Codegen Extractors

**Duration**: 3-4 weeks  
**Scope**: ALL manufacturers (Canon, Nikon, Olympus, Panasonic, Sony, etc.)
**Impact**: Eliminates 1000+ lines of manual maintenance

## Extractors to Implement

1. **Tag Table Structure Extractor** (`tag_table_structure.pl`)
2. **ProcessBinaryData Table Extractor** (`binary_data_tables.pl`)  
3. **Model Detection Pattern Extractor** (`model_patterns.pl`)
4. **Conditional Tag Definition Extractor** (`conditional_tags.pl`)

## Generated Output Structure

```
src/generated/
├── canon/
│   ├── tag_structure.rs        # CanonDataType enum + metadata
│   ├── binary_data/            # 169 ProcessBinaryData processors
│   ├── model_patterns.rs       # Camera model detection
│   └── conditional_tags.rs     # Multi-variant tag logic
├── nikon/
│   ├── tag_structure.rs        # NikonDataType enum + metadata  
│   ├── binary_data/            # 25+ ProcessBinaryData processors
│   └── ...
├── olympus/
│   └── ... (similar structure)
└── [manufacturer]/
    └── ... (universal pattern)
```
```

## 🚀 Future Nikon RAW Integration

### Automatic Nikon Support Benefits

When MILESTONE-17f (Nikon RAW Integration) is implemented, the universal extractors will provide:

**Generated Nikon Components**:
```rust
// Auto-generated from Nikon.pm Main table (100+ tags)
pub enum NikonDataType {
    MakerNoteVersion,    // 0x0001
    ISO,                 // 0x0002
    ColorMode,           // 0x0003
    Quality,             // 0x0004
    WhiteBalance,        // 0x0005
    // ... 100+ variants auto-generated
}

// Auto-generated from 25+ ProcessBinaryData tables
pub struct NikonAFInfoProcessor { /* generated */ }
pub struct NikonPictureControlProcessor { /* generated */ }
pub struct NikonISOInfoProcessor { /* generated */ }
// ... 25+ processors auto-generated

// Auto-generated model detection
pub fn detect_nikon_format(model: &str) -> NikonFormatVariant {
    if model.contains("D70") {
        NikonFormatVariant::D70
    } else if model.contains("E775") {
        NikonFormatVariant::E775
    }
    // ... all patterns auto-generated
}
```

**Implementation Acceleration**: 
- **80% less manual coding** for Nikon RAW support
- **Automatic compatibility** with ExifTool updates
- **Perfect tag structure** generated from Nikon.pm

## 🔧 Integration Instructions

### For Engineers Implementing New Manufacturers

**Instead of manual porting**:
```rust
// ❌ DON'T DO THIS - Manual maintenance nightmare
pub enum NewManufacturerDataType {
    Tag1,    // 0x0001  
    Tag2,    // 0x0002
    // ... 100+ manual variants
}
```

**Use generated extractors**:
```bash
# 1. Add manufacturer to extractor config
echo 'NewManufacturer.pm' >> codegen/config/modules.list

# 2. Run universal extractors  
make codegen-universal

# 3. Use generated code
use crate::generated::new_manufacturer::tag_structure::NewManufacturerDataType;
```


## 🎯 Success Criteria

### Code Reduction Metrics

**Per-Manufacturer Savings**:
- **Canon**: 215+ lines (CanonDataType) + 80+ lines (offset patterns) = 295+ lines
- **Olympus**: 50+ lines (section mappings) + 30+ lines (tag functions) = 80+ lines  
- **Minolta**: 400+ lines (manual processors) = 400+ lines
- **Panasonic**: 150+ lines (manual tag definitions) = 150+ lines
- **Future Nikon**: 300+ lines (estimated manual work) = 300+ lines

**Total Savings**: **1000+ lines of manual maintenance eliminated**

### Compatibility Validation

**Test Requirements**:
- [ ] Generated code produces identical output to manual implementations
- [ ] ExifTool compatibility maintained across all manufacturers
- [ ] Performance equivalent or better than manual implementations
- [ ] Memory usage equivalent or better than manual implementations

### Maintainability Validation

**Update Process**:
- [ ] New ExifTool release → Run `make codegen-universal` → Automatic updates
- [ ] New camera model → Automatic detection pattern generation
- [ ] New tag definition → Automatic enum variant generation
- [ ] Zero manual maintenance required for lookup tables

## 🔧 Testing Strategy

### Phase-by-Phase Testing

**Phase 1 (Extractor Development)**:
- Unit tests for each extractor against known ExifTool patterns
- Validate generated Rust code compiles and functions correctly
- Compare generated lookup tables against ExifTool reference data

**Phase 2 (Migration)**:
- Before/after compatibility tests for each manufacturer
- Regression tests ensuring no functionality loss
- Performance benchmarks to validate equivalent performance

**Phase 3 (Integration)**:
- Full compatibility test suite across all manufacturers
- Integration tests with existing test-images/
- End-to-end CLI tests for all supported RAW formats

### Rollback Procedures

**If Issues Arise During Migration**:

1. **Per-Manufacturer Rollback**: Keep manual implementations until generated versions validated
2. **Feature Flags**: Use conditional compilation to switch between manual/generated code
3. **Incremental Migration**: Migrate one manufacturer at a time to isolate issues
4. **Compatibility Validation**: Extensive before/after testing for each migration phase

## ⚠️ Technical Gotchas and Challenges

### ProcessBinaryData Complexity
- **Variable-length fields**: Fields can have dynamic sizes based on previous values
- **DataMember tags**: Must be processed first to determine subsequent field sizes
- **FIRST_ENTRY offset**: Canon uses 1-based offsets, not 0-based
- **Format overrides**: Each field can override table's default format
- **Reference**: See [`src/exif/binary_data.rs`](../../src/exif/binary_data.rs) for implementation patterns

### Conditional Tag Handling
- **Perl regex limitations**: Rust regex crate doesn't support lookaround/backreferences
- **Model patterns**: Handle variations like "EOS 1D" vs "EOS 1Ds"
- **Condition types**: Model-based, data pattern, count-based, format-based
- **Reference**: See [`src/conditions.rs`](../../src/conditions.rs) for evaluation framework

### ExifTool Module Patching
- **Requirement**: Convert `my %hash` to `our %hash` for extraction
- **Git submodule**: NEVER modify third-party/exiftool directly
- **Atomic operations**: Patch → Extract → Revert in single operation
- **Tool**: [`codegen/patches/patch_exiftool_modules.pl`](../../codegen/patches/patch_exiftool_modules.pl)

### Manufacturer-Specific Quirks
- **Canon**: 4/6/16/28 byte offset schemes based on model
- **Nikon**: TIFF header at offset 0x0a in maker notes
- **Sony**: Double UTF-8 encoding in some models
- **Olympus**: Complex subdirectory structures with 576+ tags

### String Handling Gotchas
- **Null termination**: Scan for nulls, don't assume clean strings
- **Garbage data**: Cameras often leave junk after string terminators
- **Encoding**: Some cameras encode UTF-8 twice (Sony)
- **Magic values**: "n/a" often used instead of null

### Debugging Tools
```bash
# Compare with ExifTool
exiftool -v3 image.jpg > exiftool_verbose.txt
exiftool -htmlDump image.jpg > dump.html

# Enable trace logging
RUST_LOG=trace cargo run -- test.jpg

# Check missing implementations
cargo run -- image.jpg --show-missing

# Generate reference data
make compat-gen
```

### Common Pitfalls
1. **Don't parse Perl with regex** - Use Perl interpreter via extractors
2. **Don't "fix" odd behavior** - It handles real camera quirks
3. **Document references** - Include ExifTool file:line numbers
4. **Test with real files** - Not synthetic test data
5. **Handle errors gracefully** - One tag failure shouldn't stop processing

## 📚 Implementation Guide for Engineers

### Getting Started

**Prerequisites**:
```bash
# 1. Ensure ExifTool submodule is current
cd third-party/exiftool && git pull origin main

# 2. Understand existing codegen architecture  
study docs/design/EXIFTOOL-INTEGRATION.md

# 3. Review universal patterns analysis
study this document's "Universal Pattern" sections
```

**Implementation Order**:
1. Start with **Tag Table Structure Extractor** - highest impact, clearest patterns
2. Implement **Model Detection Pattern Extractor** - medium complexity, clear patterns  
3. Add **Conditional Tag Definition Extractor** - handles complex multi-variant cases
4. Complete **ProcessBinaryData Table Extractor** - most complex, saves most future work

### Code Study Requirements

**Essential ExifTool Analysis**:
- `Canon.pm:1186-2165` - %Canon::Main table structure (representative)
- `Nikon.pm:1773-1872` - %Nikon::Main table structure (compare patterns)
- `Olympus.pm` - Main table structure (validate universality)
- `MakerNotes.pm:1136-1141` - Model detection patterns (cross-manufacturer)

**Essential exif-oxide Code**:
- [`codegen/src/generators/lookup_tables/`](../../codegen/src/generators/lookup_tables/) - Existing simple table generation
- [`codegen/extractors/simple_table.pl`](../../codegen/extractors/simple_table.pl) - Existing extractor pattern
- [`src/raw/formats/canon.rs:368-583`](../../src/raw/formats/canon.rs#L368) - Manual enum requiring replacement
- [`src/raw/formats/olympus.rs:42-52`](../../src/raw/formats/olympus.rs#L42) - Manual mappings requiring replacement

### Common Pitfalls to Avoid

**1. Over-Engineering**:
- Don't try to handle every edge case initially
- Start with 80% of patterns, refine based on real-world testing
- ExifTool has 25 years of edge case handling - trust its patterns

**2. Under-Engineering**:  
- Don't skip model detection patterns - they're critical for camera compatibility
- Don't ignore conditional tag definitions - they handle firmware variations
- Don't assume simple patterns - manufacturer differences are significant

**3. Compatibility Issues**:
- Always validate generated code against ExifTool reference output
- Test with real camera files, not just synthetic data
- Consider backward compatibility for older camera models

## 🎯 Long-Term Vision

### Impact on Development Time
- **Current**: 2-3 months per new manufacturer (manual porting)
- **With Extractors**: 1-2 weeks per manufacturer (mostly automated)
- **Maintenance**: Monthly ExifTool updates become trivial (`make codegen-universal`)

### Supported Manufacturers
- **Tier 1**: Canon, Nikon, Sony (most complex, 100s of ProcessBinaryData tables)
- **Tier 2**: Olympus, Panasonic, Fujifilm (medium complexity)  
- **Tier 3**: Minolta, Pentax, Leica (simpler patterns)
- **Future**: Any manufacturer ExifTool supports

## 📚 Related Documentation

### Essential Reading
- **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)** - Core principle: never "improve" ExifTool logic
- **[EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)** - Existing codegen architecture
- **[ARCHITECTURE.md](../ARCHITECTURE.md)** - Overall system design principles

### Milestone Integration  
- **[MILESTONE-17-RAW-Format-Support.md](MILESTONE-17-RAW-Format-Support.md)** - Overall RAW format strategy
- **[MILESTONE-17c-Olympus-RAW.md](MILESTONE-17c-Olympus-RAW.md)** - Olympus-specific implementation
- **[MILESTONE-17d-Canon-RAW.md](MILESTONE-17d-Canon-RAW.md)** - Canon-specific implementation

### Implementation References
- **ExifTool Canon.pm**: 169 ProcessBinaryData patterns for complex binary processing
- **ExifTool Nikon.pm**: Model detection patterns and conditional tag logic  
- **ExifTool Olympus.pm**: 29 tag tables with extensive subdirectory structures
- **ExifTool patterns across modules**: Universal applicability validation

## 🎯 Conclusion

The Universal RAW Format Codegen Extractors represent a fundamental shift from manual porting to automated generation. By recognizing and exploiting the universal patterns across ALL ExifTool manufacturer modules, we can eliminate 1000+ lines of manual maintenance while ensuring perfect compatibility and automatic updates.

**This is not just about Canon RAW support - this is about scaling exif-oxide to support every manufacturer ExifTool supports, both now and in the future.**

The investment in these 4 universal extractors pays dividends immediately (Canon, Olympus migration) and compound benefits for every future manufacturer implementation (Nikon, Sony, Fuji, etc.).

**Implementation Ready**: All analysis complete, patterns validated, migration strategy defined. Ready for engineering execution.