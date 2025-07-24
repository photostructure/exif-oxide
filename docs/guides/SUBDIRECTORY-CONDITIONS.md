# ExifTool Subdirectory Condition Patterns Guide

This guide documents the various condition patterns used in ExifTool's SubDirectory definitions, which are critical for proper tag extraction and processing. Understanding these patterns is essential for implementing and maintaining the subdirectory dispatch system in exif-oxide.

## Overview

### What Are Subdirectory Conditions?

In ExifTool, many tags contain binary data that needs further processing. These tags use SubDirectory definitions with conditions to determine:
1. Which processing table to use
2. When to apply specific parsing logic
3. How to handle manufacturer-specific variations

### Why Are They Critical?

Without proper subdirectory processing:
- Binary data appears as raw arrays: `ColorData1: [10, 789, 1024, ...]`
- Instead of meaningful tags: `WB_RGGBLevelsAsShot: "2241 1024 1024 1689"`

### Current Implementation Status

As of July 2025:
- Tag kit extractor successfully extracts all subdirectory definitions
- Code generator only handles simple `$count == N` conditions
- OR conditions and complex expressions are not properly parsed
- This causes missing variants (e.g., ColorData6 for Canon T3i)

## Expression Pattern Catalog

### 1. Count Comparisons

The most common pattern for binary data dispatch, especially in Canon ColorData tables.

#### Simple Count Comparison
```perl
$count == 582
$count == 653
$count == 5120
```

#### OR Conditions with Count
```perl
# Two values
$count == 1273 or $count == 1275

# Multiple values (Canon ColorData4)
$count == 692  or $count == 674  or $count == 702 or
$count == 1227 or $count == 1250 or $count == 1251 or
$count == 1337 or $count == 1338 or $count == 1346

# Perl OR operator variant (Sony)
$count == 1536 || $count == 2048
```

### 2. Model Regex Matches

Used extensively to apply camera-specific processing logic.

#### Simple Model Match
```perl
$$self{Model} =~ /EOS 5D/
$$self{Model} =~ /EOS D30\\b/
```

#### Model with Word Boundaries
```perl
$$self{Model} =~ /\\b1D$/
$$self{Model} =~ /\\b1DS$/
$$self{Model} =~ /\\b1Ds? Mark II$/  # Optional 's'
```

#### Multiple Models
```perl
$$self{Model} =~ /\\b(450D|REBEL XSi|Kiss X2)\\b/
$$self{Model} =~ /\\b(750D|Rebel T6i|Kiss X8i)\\b/
$$self{Model} =~ /\\b(1200D|REBEL T5|Kiss X70)\\b/
```

#### Model Ranges and Character Classes
```perl
$$self{Model} =~ /\\bEOS R[56]$/
$$self{Model} =~ /\\bEOS (R6m2|R8|R50)$/
```

#### Model Negations
```perl
$$self{Model} !~ /EOS/
$$self{Model} !~ /EOS-1DS?$/
$$self{Model} !~ /^(SLT-|HV|ILCA-)/
```

#### Complex Model Patterns (Panasonic)
```perl
$$self{Model} !~ /^DMC-(FX10|G1|L1|L10|LC80|GF\\d+|G2|TZ10|ZS7)$/ and
# tested for DC-GH6, but rule out other DC- models just in case - PH
$$self{Model} !~ /^DC-/
```

### 3. Value Pointer Patterns ($$valPt)

Sony uses these extensively for binary data pattern matching.

#### Simple Byte Patterns
```perl
$$valPt =~ /^\\xae/
$$valPt =~ /^\\x01/
$$valPt =~ /^[\\x40]/s  # 's' flag for multiline
```

#### Character Class Patterns
```perl
$$valPt =~ /^[\\x01\\x02\\x10\\x20]/
$$valPt =~ /^[\\x07\\x09\\x0a]/
$$valPt =~ /^[\\x23\\x24\\x26\\x28\\x31\\x32\\x33]/
$$valPt =~ /^[\\x3a\\xb3\\x7e\\x9a\\x25\\xe1\\x76\\x8b]/
```

#### Complex Patterns with Negation
```perl
$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/
$$valPt =~ /^[\\x01\\x02\\x10\\x20]/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x02\\x50\\x7c\\x04)/
```

#### Multi-byte Patterns
```perl
$$valPt =~ /^[\\x01\\x08\\x1b].[\\x08\\x1b]/s
$$valPt =~ /^[\\x40\\x7d]..\\x01/
$$valPt =~ /^[\\xe7\\xea\\xcd\\x8a\\x70]..\\x08/
$$valPt =~ /^\\xb6..\\x01/
```

#### String Comparisons
```perl
$$valPt ne "\\0\\0\\0\\0"
$$valPt ne "\\xff\\xff"
$$valPt ne "NORMAL"
```

### 4. Format Checks

Used to validate data format before processing.

#### Exact Format Match
```perl
$format eq "int32u"
$format eq "int16u"
```

#### Format with Count Conditions
```perl
$format eq "int32u" and ($count == 138 or $count == 148)

$format eq "int32u" and ($count == 156 or $count == 162 or
$count == 167 or $count == 171 or $count == 264)
```

#### Regex Format Match
```perl
$format =~ /^int32/
$format =~ /^int16/
```

### 5. Complex Boolean Logic

#### AND/OR Combinations
```perl
$$self{Model} =~ /EOS/ and $$self{Model} !~ /EOS-1DS?$/

$$self{Model} !~ /EOS/ or
$$self{Model} =~ /\\b(1DS?|5D|D30|D60|10D|20D|30D|K236)$/ or
$$self{Model} =~ /\\b((300D|350D|400D) DIGITAL|REBEL( XTi?)?|Kiss Digital( [NX])?)$/
```

#### Field Existence and Logic
```perl
$$self{Model} !~ /EOS/ and not $$self{AFInfo3}
$$self{Model} !~ /EOS/ and (not $$self{AFInfoCount} or $$self{AFInfoCount} != 36)
```

#### Complex Field Comparisons
```perl
$$self{LensType} and $$self{LensType} == 124 and
$$self{Model} !~ /\\b(40D|450D|REBEL XSi|Kiss X2)\\b/
```

### 6. Field Checks and Assignments

#### Simple Field Existence
```perl
$$self{FocusDistanceUpper}
$$self{FocusDistanceUpper2}
```

#### Field Assignment with Return Value
```perl
# Sets AFInfo3 to 1 and uses that as the condition
$$self{AFInfo3} = 1

# Sets CameraInfoCount and checks model
($$self{CameraInfoCount} = $count) and $$self{Model} =~ /\\b1DS?$/

# Sets Panorama based on pattern match
$$self{Panorama} = ($$valPt =~ /^(\\0\\0)?\\x01\\x01/)
```

### 7. Function Calls

```perl
GetByteOrder() eq "MM"
```

### 8. Special Complex Conditions (Sony)

```perl
# Combined model and valPt check
$$self{Model} =~ /^(ILCE-(6700|7CM2|7CR)|ZV-(E1|E10M2))\\b/ or 
($$self{Model} =~ /^(ILCE-1M2)/ and $$valPt =~ /^\\x00/)

# Cipher flag with valPt check
$$valPt =~ /^[\\x07\\x09\\x0a]/ or
($$valPt =~ /^[\\x5e\\xe7\\x04]/ and $$self{DoubleCipher} = 1)
```

## Manufacturer-Specific Patterns

### Canon

Canon primarily uses count-based dispatch for ColorData tables:

- **ColorData1**: `$count == 582`
- **ColorData2**: `$count == 653`
- **ColorData3**: `$count == 796`
- **ColorData4**: 9 different count values with OR
- **ColorData5**: `$count == 5120`
- **ColorData6**: `$count == 1273 or $count == 1275` (Canon 600D/T3i)
- **ColorData7-12**: Various count combinations

Also uses model matching for camera-specific processing:
- CameraInfo tables based on model (1D, 5D, 40D, etc.)
- AFInfo variants based on model patterns

### Sony

Sony makes extensive use of $$valPt pattern matching:

- Binary signature detection for table selection
- Complex byte patterns for encryption detection
- Model-based conditions combined with data patterns
- DoubleCipher flag affecting processing

Examples:
- `Tag9400a`: Selected when `$$valPt =~ /^[\\x07\\x09\\x0a]/`
- `Tag9404b`: Selected when `$$valPt =~ /^[\\xe7\\xea\\xcd\\x8a\\x70]..\\x08/`

### Olympus

Olympus uses mixed conditions:

- Model checks combined with count: `$$self{Model} =~ /E-(1|M5)\\b/ || $count != 1`
- String value checks: `$$valPt ne "NORMAL"`

### Panasonic

Panasonic focuses on model-specific conditions:

- Complex model exclusions with negations
- Format checks combined with value patterns
- Model ranges using regex patterns

## Implementation Requirements

### What a Parser Needs to Support

1. **Variable References**
   - `$count` - byte count of the data
   - `$format` - data format string
   - `$$self{field}` - access to other tag values
   - `$$valPt` - pointer to raw value data

2. **Operators**
   - Comparison: `==`, `!=`, `eq`, `ne`
   - Regex: `=~`, `!~`
   - Boolean: `and`, `or`, `||`, `not`
   - Assignment: `=` (that returns value)

3. **Data Types**
   - Numbers for count comparisons
   - Strings for format and model checks
   - Regex patterns with escape sequences
   - Byte patterns in hex notation

4. **Expression Features**
   - Parentheses for grouping
   - Multi-line expressions
   - Mixed operator precedence
   - Function calls

### Current Limitations

The current implementation in `tag_kit_modular.rs`:
- Only parses simple `$count == N` conditions
- Splits on `==` and expects a single number
- Ignores OR conditions completely
- Cannot handle model matches, format checks, or $$valPt patterns

### Future Considerations

1. **Full Expression Parser**
   - Needed for complex boolean logic
   - Runtime evaluation of model matches
   - Support for field assignments

2. **Shared Expression System**
   - Consider extracting to shared crate
   - Reuse between codegen and runtime
   - Cache parsed expressions

3. **Performance Optimization**
   - Pre-compile regex patterns
   - Optimize common count comparisons
   - Fast path for simple conditions

## Examples and Test Cases

### Canon T3i ColorData Issue

The Canon T3i uses ColorData6 with count=1273:
```perl
# Condition in ExifTool
$count == 1273 or $count == 1275

# Current parser only handles:
$count == 1273  # Misses the OR condition

# Result: No ColorData6 variant generated for count 1273
```

### Sony Complex Dispatch

```perl
# Multiple variants selected by binary signature
Tag9400:
  variant_0: $$valPt =~ /^[\\x07\\x09\\x0a]/
  variant_1: $$valPt =~ /^\\x0c/
  variant_2: $$valPt =~ /^[\\x23\\x24\\x26\\x28\\x31\\x32\\x33]/
```

### Common Pitfalls

1. **OR Operator Variations**
   - Perl uses both `or` and `||`
   - Parser must handle both forms

2. **Escape Sequences**
   - `\\b` for word boundaries in regex
   - `\\x` for hex bytes in patterns
   - `\\0` for null bytes

3. **Multi-line Conditions**
   - Conditions can span multiple lines
   - Whitespace handling is important

4. **Assignment Side Effects**
   - Some conditions set fields while evaluating
   - Parser needs to track these for runtime

## Parser Implementation Options

### Option 1: Enhanced Simple Parser (Immediate Fix)

For count-based OR conditions:
```rust
fn parse_count_conditions(condition: &str) -> Vec<usize> {
    let mut counts = Vec::new();
    
    // Normalize OR operators
    let normalized = condition.replace("||", "or");
    
    // Split on OR
    for part in normalized.split(" or ") {
        if let Some(eq_pos) = part.find("==") {
            let count_str = part[eq_pos + 2..].trim();
            if let Ok(count) = count_str.parse::<usize>() {
                counts.push(count);
            }
        }
    }
    
    counts
}
```

### Option 2: Shared Expression Parser

Extract existing expression parser to shared crate:
- Move `src/expressions/` to `exif-oxide-expressions`
- Enhance for Perl-like syntax
- Use in both codegen and runtime

### Option 3: External Parser Library

Evaluated options:
- **pest**: Grammar-based, good for complex syntax
- **nom**: Fast combinators, full control
- **meval/reval**: Too limited for Perl expressions

Recommendation: Start with Option 1 for immediate fix, plan for Option 2 long-term.

## Conclusion

Subdirectory conditions are a critical part of ExifTool's tag processing system. While the current implementation handles simple cases, full support requires:

1. Immediate: Fix OR conditions for count comparisons
2. Short-term: Handle model matches and format checks
3. Long-term: Full expression parser for complex conditions

This guide should help future engineers understand the complexity and implement robust solutions.