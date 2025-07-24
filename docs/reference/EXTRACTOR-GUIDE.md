# ExifTool Extractor Selection Guide

This guide provides detailed information about each extractor in the codegen system, helping you choose the right tool for your extraction needs.

## Core Principle: One-Trick Ponies

Each extractor is designed to do one thing well. This makes the codebase easier to understand, maintain, and extend. When faced with a choice, always prefer using a specialized extractor over trying to make a general one handle multiple cases.

## Tag Kit System: The Primary Tag Extraction System

**🎯 STATUS: FULLY OPERATIONAL** - The tag kit system is production-ready and successfully processes 414+ EXIF tags with automated PrintConv implementations.

The tag kit system represents a fundamental architecture shift from manual registry-based tag processing to automated, comprehensive tag extraction. It consolidates multiple extraction patterns into a single, unified approach that eliminates entire classes of bugs.

### What Makes Tag Kit Special

1. **Complete Bundles**: Each tag comes with its ID, name, format, groups, AND PrintConv implementation in a single extraction
2. **Eliminates ID/PrintConv Mismatches**: No more bugs from tag IDs not matching their PrintConv functions - they're extracted together
3. **Self-Contained Processing**: Everything needed to process a tag (definition + conversion logic) in one place
4. **Config-Driven Flexibility**: Works with any ExifTool module, not hardcoded to specific ones like legacy extractors
5. **Automatic ExifTool Updates**: Monthly ExifTool releases update tag definitions and PrintConvs automatically
6. **Three PrintConv Types**: Handles Simple (static lookups), Expression (Perl expressions), and Manual (function references)

### Technical Architecture

**Core Components:**

- **TagKitExtractor** (`codegen/src/extractors/tag_kit.rs`): Calls `tag_kit.pl` once per table, consolidates results
- **Modular Generator** (`codegen/src/generators/tag_kit_modular.rs`): Splits output into category-based modules (core, camera, gps, etc.)
- **Tag Categorization** (`codegen/src/generators/tag_kit_split.rs`): Organizes tags by semantic meaning and ID ranges
- **Schema Definitions** (`codegen/src/schemas/tag_kit.rs`): Rust structures for TagKit, PrintConvType, etc.
- **Perl Extractor** (`codegen/extractors/tag_kit.pl`): Extracts complete tag definitions with embedded PrintConv data

**Generated Structure:**

```rust
// Example: src/generated/Exif_pm/tag_kit/core.rs
pub struct TagKitDef {
    pub id: u32,
    pub name: &'static str,
    pub format: &'static str,
    pub groups: HashMap<&'static str, &'static str>,
    pub print_conv: PrintConvType,
    // ... other fields
}

pub enum PrintConvType {
    None,
    Simple(&'static HashMap<String, &'static str>),  // Static lookup tables
    Expression(&'static str),                        // Perl expressions
    Manual(&'static str),                           // Function names
}
```

**Runtime Integration:**

```rust
// Main API for tag processing
pub fn apply_print_conv(
    tag_id: u32,
    value: &TagValue,
    evaluator: &mut ExpressionEvaluator,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> TagValue
```

### Migration Status (July 2025)

**✅ COMPLETED MODULES:**

- **Exif** (414 tags, 111 PrintConv tables) - Main EXIF tag table
- **Canon** (300+ tags, 97 PrintConv tables) - 17 tables consolidated
- **Sony** (405 tags, 228 PrintConv tables) - 10 tables consolidated
- **Olympus** (351 tags, 72 PrintConv tables) - 8 tables consolidated
- **Panasonic** (129 tags, 59 PrintConv tables) - Complete
- **PanasonicRaw** (48 tags, 1 PrintConv table) - Complete migration from manual registry
- **MinoltaRaw** (35 tags, 5 PrintConv tables) - Complete

**❌ DEPRECATED EXTRACTORS (Being Replaced):**

- `inline_printconv.pl` → Tag kit includes inline PrintConvs as part of complete bundles
- `tag_tables.pl` → Tag kit is config-driven and works with any module (not hardcoded to EXIF/GPS)
- `tag_definitions.pl` → Tag kit provides complete bundles instead of function references

### Key Benefits Over Legacy Systems

1. **Zero Maintenance PrintConvs**: 414+ EXIF tags now have automated PrintConv implementations that update automatically with ExifTool releases
2. **Eliminates Registry Bugs**: No more manual tag ID to function mapping errors
3. **Modular File Organization**: Large tag tables split into semantic categories (core.rs, camera.rs, gps.rs, etc.) for better IDE performance
4. **Deterministic Generation**: ~~Content-based PRINT_CONV naming~~ **(Note: Nondeterministic counter issue being addressed)**
5. **Warning Suppression**: Generated files include `#![allow(unused_imports)]` to prevent clippy warnings

### Configuration Example

```json
// codegen/config/Exif_pm/tag_kit.json
{
  "description": "Complete EXIF tag definitions with embedded PrintConvs",
  "source": "../third-party/exiftool/lib/Image/ExifTool/Exif.pm",
  "tables": [
    {
      "name": "Main",
      "description": "Primary EXIF tag table"
    }
  ]
}
```

### Consolidation Logic

The TagKitExtractor uses custom consolidation logic for modules with multiple tables:

1. **Per-Table Extraction**: Calls `tag_kit.pl` once per table with temporary files
2. **Metadata Aggregation**: Combines total_tags_scanned, skipped_complex counts
3. **Tag Collection**: Merges all tag kits from multiple tables
4. **Single Module Output**: Creates one consolidated `module__tag_kit.json` file
5. **Temporary Cleanup**: Removes individual table files after consolidation

This approach handles complex modules like Canon (17 tables) while maintaining the simple per-table Perl extraction pattern.

## Extractor Comparison Matrix

| Extractor               | Purpose                      | Input Pattern                  | Output                         | Status         | When to Use                  |
| ----------------------- | ---------------------------- | ------------------------------ | ------------------------------ | -------------- | ---------------------------- |
| **tag_kit.pl** ⭐       | **Complete tag definitions** | Any tag table                  | **Tag bundles with PrintConv** | **PRODUCTION** | **PRIMARY: Always for tags** |
| simple_table.pl         | Standalone lookups           | `%hashName = (...)`            | Static HashMap                 | Active         | Manufacturer lookups         |
| runtime_table.pl        | Runtime HashMaps             | ProcessBinaryData tables       | HashMap creators               | Active         | Binary data with conditions  |
| process_binary_data.pl  | Binary structures            | Tables with `%binaryDataAttrs` | Binary parsing info            | Active         | Fixed binary layouts         |
| boolean_set.pl          | Membership tests             | Sets with value 1              | Boolean HashSet                | Active         | Existence checks             |
| composite_tags.pl       | Calculated tags              | Composite definitions          | Tag dependencies               | Active         | Tags from other tags         |
| ~~inline_printconv.pl~~ | ~~PrintConv extraction~~     | ~~Tag tables~~                 | ~~Lookup functions~~           | **DEPRECATED** | ~~Use tag_kit.pl instead~~   |
| ~~tag_tables.pl~~       | ~~EXIF/GPS only~~            | ~~Hardcoded tables~~           | ~~Tag references~~             | **DEPRECATED** | ~~Use tag_kit.pl instead~~   |
| ~~tag_definitions.pl~~  | ~~Filtered tags~~            | ~~Frequency filtered~~         | ~~Function refs~~              | **DEPRECATED** | ~~Use tag_kit.pl instead~~   |

## Detailed Extractor Descriptions

### Tag Extraction

#### tag_kit.pl ⭐ (PRIMARY SYSTEM)

**Status**: **PRODUCTION READY** - Successfully processing 414+ EXIF tags with automated PrintConv implementations

**Purpose**: Extract complete, self-contained tag definitions with embedded PrintConv implementations, eliminating tag ID/function mismatch bugs.

**ExifTool Pattern**:

```perl
0x0128 => {
    Name => 'ResolutionUnit',
    Format => 'int16u',
    PrintConv => { 1 => 'None', 2 => 'inches', 3 => 'cm' },
}
```

**Generated Output**:

```rust
// src/generated/Exif_pm/tag_kit/core.rs
TagKitDef {
    id: 296,  // 0x0128
    name: "ResolutionUnit",
    format: "int16u",
    print_conv: PrintConvType::Simple(&PRINT_CONV_73),
}

// Accompanying static lookup table
static PRINT_CONV_73: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("1".to_string(), "None");
    map.insert("2".to_string(), "inches");
    map.insert("3".to_string(), "cm");
    map
});
```

**Key Architecture Features**:

- **Consolidation Logic**: Handles modules with multiple tables (e.g., Canon has 17 tables)
- **Modular Output**: Splits into semantic categories (core, camera, gps, etc.) for manageable file sizes
- **Three PrintConv Types**: Simple (static), Expression (Perl code), Manual (function references)
- **Runtime Integration**: `apply_print_conv()` function for seamless tag processing
- **No Patching Required**: Unlike other extractors, doesn't need ExifTool patching

**When to Use**: **Always** for any tag table extraction. This is the primary system replacing all tag-related extractors.

#### ~~inline_printconv.pl~~ (DEPRECATED)

**Status**: **REPLACED BY TAG KIT SYSTEM** - No longer maintained

**Original Purpose**: Extract PrintConv hashes embedded in tag definitions separately from tag IDs.

**Why Deprecated**:

- Created tag ID/PrintConv mismatch bugs (different extraction timing)
- Tag kit extracts PrintConvs as part of complete tag bundles
- 414+ EXIF tags successfully migrated to tag kit system (July 2025)

**Migration**: All inline_printconv configs converted to tag_kit configs. Source code migration from manual registry in progress.

#### ~~tag_tables.pl~~ (DEPRECATED)

**Status**: **REPLACED BY TAG KIT SYSTEM** - No longer maintained

**Original Purpose**: Hardcoded extraction of EXIF and GPS tags only.

**Why Deprecated**:

- Hardcoded to specific modules instead of config-driven
- Tag kit works with any ExifTool module through JSON configuration
- Limited to basic tag extraction without PrintConv integration

#### ~~tag_definitions.pl~~ (DEPRECATED)

**Status**: **REPLACED BY TAG KIT SYSTEM** - No longer maintained

**Original Purpose**: Extract tags with frequency filtering and generate function name references for manual registry.

**Why Deprecated**:

- Creates dependency on manual registry maintenance
- Function reference approach prone to registry bugs
- Tag kit embeds everything needed for tag processing in unified bundles

### Lookup Tables

#### simple_table.pl

**Purpose**: Extract standalone key-value lookup tables not associated with specific tags.

**ExifTool Pattern**:

```perl
%canonWhiteBalance = (
    0 => 'Auto',
    1 => 'Daylight',
    2 => 'Cloudy',
    3 => 'Tungsten',
);
```

**When to Use**:

- Manufacturer-specific lookups referenced by multiple tags
- Standalone conversion tables
- Any `%hashName` that isn't inside a tag definition

#### boolean_set.pl

**Purpose**: Extract sets used for membership testing.

**ExifTool Pattern**:

```perl
%isDatChunk = (
    IDAT => 1,
    JDAT => 1,
    JDAA => 1,
);
```

**When to Use**:

- Fast existence checks in ExifTool code
- Sets where only membership matters, not values

### Binary Data Processing

#### process_binary_data.pl

**Purpose**: Extract ProcessBinaryData table structures for fixed-format binary parsing.

**ExifTool Pattern**:

```perl
%Canon::CameraSettings = (
    %binaryDataAttrs,
    FORMAT => 'int16s',
    FIRST_ENTRY => 1,
    1 => { Name => 'MacroMode', PrintConv => {...} },
);
```

**When to Use**:

- Tables marked with `%binaryDataAttrs`
- Fixed binary data layouts
- When you need offset/format information

#### runtime_table.pl

**Purpose**: Generate code that creates HashMaps at runtime based on context.

**Key Distinction from process_binary_data.pl**:

- Generates runtime HashMap creation functions
- Handles conditional logic based on camera model/firmware
- For dynamic table construction, not static definitions

**When to Use**:

- ProcessBinaryData tables with model-specific variations
- Tables requiring runtime context
- When static extraction isn't sufficient

### Specialized Extractors

#### composite_tags.pl

**Purpose**: Extract composite tag definitions that calculate values from other tags.

**ExifTool Pattern**:

```perl
FOV => {
    Require => {
        0 => 'FocalLength',
        1 => 'ScaleFactor35efl',
    },
    ValueConv => '2*atan($val[1]/$val[0]/2)*180/3.14159',
}
```

**When to Use**:

- Tags calculated from other tag values
- Complex dependencies between tags

#### conditional_tags.pl

**Purpose**: Extract tag arrays with complex conditional logic.

**When to Use**:

- Model/firmware-specific tag variations
- Complex conditional structures in tag tables

#### file_type_lookup.pl & regex_patterns.pl

**Purpose**: Extract file type detection patterns and magic numbers.

**When to Use**:

- File format identification
- Binary signature detection

## Decision Flowchart

```
Start: What are you extracting?
│
├─ Tag definitions (ANY module/table)?
│  └─ ⭐ Use tag_kit.pl ✓ (PRIMARY SYSTEM)
│
├─ Standalone lookup table (%hashName, not in tag def)?
│  └─ Use simple_table.pl
│
├─ Boolean membership set (existence checks)?
│  └─ Use boolean_set.pl
│
├─ Binary data structure?
│  ├─ Need runtime construction?
│  │  └─ Use runtime_table.pl
│  └─ Fixed format?
│     └─ Use process_binary_data.pl
│
├─ Composite/calculated tags?
│  └─ Use composite_tags.pl
│
├─ ❌ PrintConv separate from tag IDs?
│  └─ DON'T: Use tag_kit.pl instead (eliminates mismatch bugs)
│
└─ Something else?
   └─ Check specialized extractors (file_type_lookup, etc.)
```

**Rule of Thumb**: If it's a tag table in ExifTool, use `tag_kit.pl`. Everything else uses specialized extractors.

## Common Pitfalls

### 1. Using Deprecated Extractors for New Work ❌

**Don't**: Add new `inline_printconv.pl`, `tag_tables.pl`, or `tag_definitions.pl` configs
**Do**: Use `tag_kit.pl` for all tag extraction needs

### 2. Extracting Tags and PrintConvs Separately ❌

**Don't**: Try to extract tag IDs and PrintConv functions separately (creates mismatch bugs)
**Do**: Use tag_kit.pl which extracts them together as complete bundles

### 3. Confusing Runtime vs Static Tables

**runtime_table.pl**: Creates functions that build HashMaps at runtime (binary data with conditions)
**simple_table.pl**: Creates static LazyLock HashMaps (standalone lookups)
**tag_kit.pl**: Creates static tag definitions WITH embedded PrintConv tables

### 4. Manual Tag Definition Extraction ❌

**Don't**: Write custom extraction for tag tables or try to parse Perl manually
**Do**: Configure tag_kit.pl for your module - it handles the complexity

### 5. Adding Manual Registry Dependencies ❌

**Don't**: Create function references that require manual registry mapping
**Do**: Use tag_kit's self-contained bundles that include everything needed

### 6. Manual Porting Instead of Improving Tag Kit ❌

**Don't**: Manually port individual tags, PrintConvs, or tables from ExifTool
**Do**: Improve the tag kit system to handle edge cases automatically

**🚨 CRITICAL**: We've had **100+ bugs** from manual porting errors. Manual transcription of ExifTool data is **BANNED**. See [CODEGEN.md](../CODEGEN.md#never-manual-port-exiftool-data) for details.

**Why This Matters**: Manual porting creates a maintenance burden that grows with every ExifTool release. A 1-hour fix to tag kit's extraction logic can automatically handle hundreds of tags across all modules, while manual porting benefits only that specific case and requires ongoing maintenance. More critically, manual transcription introduces subtle bugs that are nightmare to debug.

## Migration Status & Future Direction

**Current State (July 2025):**

- ✅ **Tag Kit System: PRODUCTION READY** - 7 modules migrated, 414+ EXIF tags automated
- ✅ **Source Code Migration: IN PROGRESS** - Updating implementations to use tag_kit instead of inline functions
- ✅ **Legacy Deprecation: ACTIVE** - Deprecated extractors marked but still generate for backward compatibility

**Immediate Next Steps:**

1. **Complete Source Code Migration** - Update remaining `src/implementations/` files to use tag_kit APIs
2. **Remove Legacy Configs** - Delete deprecated `inline_printconv.json` files after source migration
3. **Fix PRINT_CONV Determinism** - Address nondeterministic counter issue for stable git diffs

**Long-term Vision:**

1. **Unified Tag Processing** - All tag extraction through tag kit system
2. **Simplified Architecture** - Fewer extractors to understand and maintain
3. **Zero Manual Registry** - Complete automation of tag processing
4. **Expression System** - Implement Perl expression evaluation for complex PrintConvs
5. **Automated Everything** - Prefer improving extraction over manual porting for maximum scalability

## Examples

### Migrating from inline_printconv to tag_kit

**Old Config** (inline_printconv.json):

```json
{
  "tables": [
    {
      "table_name": "Main",
      "description": "EXIF main table inline PrintConvs"
    }
  ]
}
```

**New Config** (tag_kit.json):

```json
{
  "tables": [
    {
      "name": "Main",
      "description": "Complete EXIF Main tag definitions with PrintConvs"
    }
  ]
}
```

The tag kit automatically extracts everything inline_printconv did, plus more.

### When to Keep Using simple_table.pl

If you have a standalone lookup table like:

```perl
%nikonLensIDs = (
    0x01 => 'AF Nikkor 35-70mm f/3.3-4.5',
    0x02 => 'AF Zoom-Nikkor 80-200mm f/2.8 ED',
    # ... hundreds more entries
);
```

This should remain with simple_table.pl because:

- It's not part of a tag definition
- Multiple tags might reference it
- It's a pure lookup table

## Conclusion

**The tag kit system is now the primary and recommended approach for all tag extraction.** It has proven successful with 414+ EXIF tags and 7 manufacturer modules in production, providing complete, self-contained bundles that eliminate entire classes of bugs.

**For New Engineers:**

1. **Default to tag_kit.pl** for any tag table extraction
2. **Use specialized extractors** for non-tag data (lookup tables, binary structures, etc.)
3. **Avoid deprecated extractors** - they're being phased out
4. **Read the migration docs** if working with legacy inline function references
5. **🚨 CRITICAL: Manual porting is BANNED** - We've had 100+ bugs from manual transcription. Always use codegen.
6. **⚠️ REQUIRED READING**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) and [CODEGEN.md manual porting section](../CODEGEN.md#never-manual-port-exiftool-data)

**Key Success Metrics:**

- ✅ **Zero maintenance PrintConvs** - 414+ tags update automatically with ExifTool releases
- ✅ **Eliminated registry bugs** - No more manual tag ID to function mapping errors
- ✅ **Modular organization** - Large tables split into manageable categories
- ✅ **Production validation** - Real image testing confirms human-readable output

The tag kit system represents a fundamental shift from manual, error-prone registry systems to automated, reliable tag processing. When in doubt, choose tag_kit.pl - it handles the complexity so you don't have to.
