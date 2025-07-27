# Technical Project Plan: BITMASK PrintConv Implementation

## Project Overview

**Goal**: Research and implement support for ExifTool's BITMASK PrintConv pattern, which represents bit flags in human-readable format.

**Problem**: BITMASK is a special PrintConv hash structure where keys are bit positions and values are descriptions. When multiple bits are set, ExifTool concatenates the descriptions.

## Background & Context

ExifTool uses BITMASK for tags that represent multiple boolean flags packed into a single integer. Each bit position has a specific meaning.

### Example from Olympus.pm:
```perl
PrintConv => {
    BITMASK => {
        0 => 'S-AF',
        2 => 'C-AF', 
        4 => 'MF',
        5 => 'Face Detect',
        6 => 'Imager AF',
    }
}
```

For value 0x35 (binary: 00110101):
- Bit 0 set: 'S-AF'
- Bit 2 set: 'C-AF'
- Bit 4 set: 'MF'
- Bit 5 set: 'Face Detect'
- Result: "S-AF, C-AF, MF, Face Detect"

## Research Findings

### Occurrence Analysis
- Found 130 BITMASK occurrences across ExifTool modules
- Used in: Olympus, FujiFilm, Minolta, JPEG, MPF, EXE modules
- Common for: AF modes, flash settings, image processing features

### BITMASK Characteristics
1. Keys are always numeric (bit positions)
2. Values are descriptive strings
3. Multiple bits can be set simultaneously
4. ExifTool joins descriptions with ", " separator
5. Sometimes includes 'OTHER' key for unknown bits

### Implementation Approach

#### Option 1: Generate Specific Functions (Preferred)
- Extract BITMASK data during codegen
- Generate specific bitmask handler for each tag
- Type-safe and efficient

#### Option 2: Generic Runtime Handler
- Single generic bitmask function
- Pass BITMASK data as parameter
- More flexible but requires runtime data

## Implementation Plan

### ✅ **Phase 0: P11 Integration Complete (2025-07-27)**

**🎯 Achievement**: P11 binary data integration has established BITMASK TODO framework.

#### BITMASK Integration Infrastructure Ready:

1. **TODO Placeholder System**:
   - **Location**: `/home/mrm/src/exif-oxide/codegen/src/generators/process_binary_data.rs:106-116`
   - **Function**: Custom serde deserializer handles BITMASK objects in Sony ProcessBinaryData
   - **Placeholder**: All BITMASK entries currently return `"TODO_BITMASK_P15c"`
   - **Search Pattern**: Use `git grep "TODO_BITMASK_P15c"` to find all locations needing P15c implementation

2. **Sony BITMASK Examples Ready**:
   - **File**: `codegen/generated/extract/binary_data/sony__process_binary_data__camerasettings.json:993-998`
   - **Example Structure**:
     ```json
     {
        "key" : "BITMASK",
        "value" : {
           "0" : "Confirmed",
           "1" : "Failed", 
           "2" : "Tracking"
        }
     }
     ```
   - **Context**: FocusStatus tag with bit flag definitions ready for extraction

3. **Integration Points Established**:
   - **Custom Deserializer**: Framework ready to extract BITMASK bit position mappings
   - **Generated Code Path**: Binary data parsers will use BITMASK functions when implemented
   - **Test Data Available**: Real Sony BITMASK examples provide validation targets

#### Next Phase Implementation Strategy:

### Phase 1: Codegen Extraction (Enhanced)
1. **Modify custom deserializer** in `process_binary_data.rs` to extract BITMASK mappings instead of placeholder
2. Store bit position -> description mappings in generated structures
3. Generate BITMASK-specific functions for each tag

### Phase 2: Runtime Implementation
1. Create `bitmask_print_conv` in `implementations/print_conv.rs`
2. Handle bit testing and string concatenation
3. Match ExifTool's output format exactly

### Phase 3: Registry Integration
1. Add BITMASK patterns to `conv_registry.rs`
2. Generate direct function calls
3. Test with real images

## Example Implementation

```rust
pub fn olympus_af_mode_bitmask(value: &TagValue) -> TagValue {
    let Some(val) = value.as_u16() else {
        return value.clone();
    };
    
    let mut descriptions = Vec::new();
    
    if val & (1 << 0) != 0 { descriptions.push("S-AF"); }
    if val & (1 << 2) != 0 { descriptions.push("C-AF"); }
    if val & (1 << 4) != 0 { descriptions.push("MF"); }
    if val & (1 << 5) != 0 { descriptions.push("Face Detect"); }
    if val & (1 << 6) != 0 { descriptions.push("Imager AF"); }
    
    if descriptions.is_empty() {
        TagValue::String(format!("Unknown (0x{:x})", val))
    } else {
        TagValue::String(descriptions.join(", "))
    }
}
```

## Success Criteria

1. BITMASK PrintConv expressions correctly identified
2. Bit flags properly decoded
3. Output matches ExifTool format exactly
4. No performance regression

## Estimated Effort

- Research: ✅ Complete
- Implementation: 2-3 hours
- Testing: 1 hour
- Total: ~4 hours

## Priority Justification

P15c (after P15a/b) because:
- BITMASK is common (~130 occurrences)
- Affects camera feature descriptions
- Required for full PrintConv compatibility