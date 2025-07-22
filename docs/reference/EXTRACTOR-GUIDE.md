# ExifTool Extractor Selection Guide

This guide provides detailed information about each extractor in the codegen system, helping you choose the right tool for your extraction needs.

## Core Principle: One-Trick Ponies

Each extractor is designed to do one thing well. This makes the codebase easier to understand, maintain, and extend. When faced with a choice, always prefer using a specialized extractor over trying to make a general one handle multiple cases.

## Tag Kit System: The Future of Tag Extraction

The tag kit system represents a fundamental improvement in how we extract and manage tag definitions. It consolidates multiple extraction patterns into a single, unified approach.

### What Makes Tag Kit Special

1. **Complete Bundles**: Each tag comes with its ID, name, format, groups, AND PrintConv implementation
2. **Eliminates Mismatches**: No more bugs from tag IDs not matching their PrintConv functions
3. **Self-Contained**: Everything needed to process a tag in one place
4. **Config-Driven**: Works with any ExifTool module, not hardcoded to specific ones

### Migration Path

We're actively migrating from these deprecated extractors to tag kit:
- `inline_printconv.pl` → Tag kit includes inline PrintConvs
- `tag_tables.pl` → Tag kit is more flexible and config-driven
- `tag_definitions.pl` → Tag kit provides complete bundles

## Extractor Comparison Matrix

| Extractor | Purpose | Input Pattern | Output | When to Use |
|-----------|---------|---------------|--------|-------------|
| **tag_kit.pl** | Complete tag definitions | Any tag table | Tag bundles with PrintConv | **Always for tags** |
| simple_table.pl | Standalone lookups | `%hashName = (...)` | Static HashMap | Manufacturer lookups |
| runtime_table.pl | Runtime HashMaps | ProcessBinaryData tables | HashMap creators | Binary data with conditions |
| process_binary_data.pl | Binary structures | Tables with `%binaryDataAttrs` | Binary parsing info | Fixed binary layouts |
| boolean_set.pl | Membership tests | Sets with value 1 | Boolean HashSet | Existence checks |
| composite_tags.pl | Calculated tags | Composite definitions | Tag dependencies | Tags from other tags |

## Detailed Extractor Descriptions

### Tag Extraction

#### tag_kit.pl (PRIMARY)

**Purpose**: Extract complete tag definitions with everything needed to process them.

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
TagKitDef {
    id: 296,
    name: "ResolutionUnit",
    format: "int16u",
    print_conv: PrintConvType::Simple(&PRINT_CONV_55),
}
```

**Key Benefits**:
- Single source of truth for tag processing
- Automatic handling of inline PrintConvs
- Supports Simple, Expression, and Manual PrintConv types

#### inline_printconv.pl (DEPRECATED)

**Status**: Being replaced by tag kit system.

**Original Purpose**: Extract PrintConv hashes embedded in tag definitions.

**Why Deprecated**: Tag kit extracts the same data as part of complete tag bundles, eliminating the need for separate PrintConv extraction.

#### tag_tables.pl (DEPRECATED)

**Status**: Being replaced by tag kit system.

**Original Purpose**: Hardcoded extraction of EXIF and GPS tags only.

**Why Deprecated**: Tag kit is config-driven and can extract from any module, not just EXIF/GPS.

#### tag_definitions.pl (DEPRECATED)

**Status**: Being replaced by tag kit system.

**Original Purpose**: Extract tags with frequency filtering and generate function name references.

**Why Deprecated**: Creates dependency on manual registry. Tag kit embeds everything needed.

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
├─ Tag definitions with PrintConv?
│  └─ Use tag_kit.pl ✓
│
├─ Standalone lookup table (%hashName)?
│  └─ Use simple_table.pl
│
├─ Boolean membership set?
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
└─ Something else?
   └─ Check specialized extractors
```

## Common Pitfalls

### 1. Using inline_printconv.pl for New Work
**Don't**: Add new inline_printconv configs
**Do**: Use tag_kit.pl for all tag extraction needs

### 2. Confusing Runtime vs Static Tables
**runtime_table.pl**: Creates functions that build HashMaps at runtime
**simple_table.pl**: Creates static LazyLock HashMaps

### 3. Manual Tag Definition Extraction
**Don't**: Write custom extraction for tag tables
**Do**: Configure tag_kit.pl for your module

## Future Direction

The tag kit system is the future of tag extraction in exif-oxide. We're working toward:

1. **Complete Migration**: All tag extraction through tag kit
2. **Deprecation**: Remove redundant extractors
3. **Simplification**: Fewer extractors to understand and maintain
4. **Reliability**: Eliminate entire classes of bugs through unified extraction

## Examples

### Migrating from inline_printconv to tag_kit

**Old Config** (inline_printconv.json):
```json
{
  "tables": [{
    "table_name": "Main",
    "description": "EXIF main table inline PrintConvs"
  }]
}
```

**New Config** (tag_kit.json):
```json
{
  "tables": [{
    "name": "Main",
    "description": "Complete EXIF Main tag definitions with PrintConvs"
  }]
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

Choose extractors based on what you're extracting, not how you want to use it. The tag kit system represents the future of tag extraction, providing complete, self-contained bundles that eliminate entire classes of bugs. When in doubt, prefer specialized extractors over general ones, and always consider whether tag kit can handle your use case.