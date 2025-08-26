# Expression Analysis Files

This directory contains analysis of ExifTool expressions used by required tags in exif-oxide.

## Files

### composite-dependencies.json

Extracted dependencies for composite tags from ExifTool modules. Shows which base tags each composite tag requires or desires.

**Generate/Update:** `make expression-analysis`

### required-expressions-analysis.json

Analysis of all ValueConv, PrintConv, and Condition expressions used by required tags (including transitive dependencies from composite tags).

**Generate/Update:** `make expression-analysis`

### all-value-conv.json

(Legacy) Analysis of all ValueConv expressions across the entire codebase.

## Usage

To regenerate these files after updating field_extractor.pl, tag-metadata.json, or the ExifTool source:

```bash
make expression-analysis-force
```

## Key Findings

The analysis shows that for required tags, the PPI AST codegen needs to support:

1. **Function calls** (58 PrintConv, 29 ValueConv) - Most common pattern
2. **Arithmetic operations** (39 PrintConv, 26 ValueConv) - Critical for calculations
3. **Ternary operators** (14 ValueConv, 3 PrintConv) - Conditional logic
4. **String operations** - Concatenation and sprintf formatting
5. **Regex operations** - Pattern matching and substitution

See the generated JSON files for detailed breakdowns of expressions and which tags use them.
