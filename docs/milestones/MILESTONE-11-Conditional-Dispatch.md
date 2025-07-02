# Milestone 11: Conditional Dispatch

**Duration**: 2 weeks  
**Goal**: Runtime condition evaluation for processor selection

## Overview

ExifTool uses conditional logic to select different processors based on runtime data like camera model or data patterns. This milestone implements that capability in exif-oxide.

## Background

From [PROCESSOR-PROC-DISPATCH.md](../PROCESSOR-PROC-DISPATCH.md), ExifTool supports conditions like:

```perl
{
    Condition => '$$valPt =~ /^0204/', # Based on actual data content
    Name => 'LensData0204',
    SubDirectory => {
        TagTable => 'Image::ExifTool::Nikon::LensData0204',
        ProcessProc => \&ProcessNikonEncrypted,
        DecryptStart => 4,
    },
},
```

## Deliverables

### 1. Condition Expression Types

Implement condition evaluation system:

```rust
pub enum Condition {
    DataPattern(Regex),           // $$valPt =~ /pattern/
    ModelMatch(Regex),           // $$self{Model} =~ /pattern/
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

impl Condition {
    pub fn evaluate(&self, context: &EvalContext) -> bool {
        match self {
            Condition::DataPattern(regex) => {
                regex.is_match(&context.data)
            }
            Condition::ModelMatch(regex) => {
                context.model
                    .as_ref()
                    .map(|m| regex.is_match(m))
                    .unwrap_or(false)
            }
            Condition::And(conditions) => {
                conditions.iter().all(|c| c.evaluate(context))
            }
            Condition::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(context))
            }
        }
    }
}
```

### 2. Conditional Processor Dispatch

Enhance processor selection with conditions:

```rust
pub struct ConditionalProcessor {
    pub condition: Option<Condition>,
    pub processor: ProcessorType,
    pub parameters: HashMap<String, Value>,
}

impl ExifReader {
    fn select_processor(
        &self,
        table: &TagTable,
        tag_id: Option<TagId>,
        data: &[u8],
    ) -> (ProcessorType, HashMap<String, Value>) {
        // Build evaluation context
        let context = EvalContext {
            data,
            model: self.values.get("Model").and_then(|v| v.as_string()),
            // ... other context fields
        };

        // Check conditional processors
        if let Some(tag_id) = tag_id {
            if let Some(conditionals) = table.conditionals.get(&tag_id) {
                for conditional in conditionals {
                    if let Some(condition) = &conditional.condition {
                        if condition.evaluate(&context) {
                            return (conditional.processor.clone(), 
                                   conditional.parameters.clone());
                        }
                    }
                }
            }
        }

        // Fallback to default
        (table.default_processor, HashMap::new())
    }
}
```

### 3. Integration Examples

#### Canon Model-Specific Tables

```rust
// In codegen output
ConditionalProcessor {
    condition: Some(Condition::ModelMatch(
        Regex::new(r"\b1DS?$").unwrap()
    )),
    processor: ProcessorType::Canon(CanonProcessor::CameraInfo1D),
    parameters: HashMap::new(),
}
```

#### Nikon Encrypted Sections

```rust
ConditionalProcessor {
    condition: Some(Condition::DataPattern(
        Regex::new(r"^0204").unwrap()
    )),
    processor: ProcessorType::Nikon(NikonProcessor::Encrypted),
    parameters: hashmap! {
        "DecryptStart" => Value::Integer(4),
        "ByteOrder" => Value::String("LittleEndian"),
    },
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

1. Define `Condition` enum and evaluation logic
2. Create `ConditionalProcessor` structure
3. Add condition parsing to codegen
4. Update processor dispatch logic

### Phase 2: Integration & Testing (Week 2)

1. Integrate with Canon processor selection
2. Add Nikon encrypted section dispatch
3. Performance optimization (regex caching)
4. Comprehensive testing

## Success Criteria

- [ ] Canon FileNumber extraction works correctly per camera model
- [ ] Correct processor selected based on conditions
- [ ] No performance regression vs static dispatch
- [ ] All existing processors continue working

## Manual Implementations Required

### Condition Evaluation

```rust
// src/implementations/conditions/mod.rs
pub mod evaluator {
    use regex::Regex;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    // Cache compiled regexes
    static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = 
        Lazy::new(|| Mutex::new(HashMap::new()));

    pub fn evaluate_data_pattern(pattern: &str, data: &[u8]) -> bool {
        let mut cache = REGEX_CACHE.lock().unwrap();
        let regex = cache.entry(pattern.to_string())
            .or_insert_with(|| Regex::new(pattern).unwrap());
        
        // Convert first few bytes to string for pattern matching
        let data_str = String::from_utf8_lossy(&data[..data.len().min(16)]);
        regex.is_match(&data_str)
    }
}
```

### Model-Specific Dispatch Rules

```rust
// src/implementations/canon.rs
pub fn select_camera_info_table(model: &str) -> &'static str {
    // ExifTool: Canon.pm:7234-7298
    if model.contains("1D") && !model.contains("Mark") {
        "Canon::CameraInfo1D"
    } else if model.contains("1D") && model.contains("Mark II") {
        "Canon::CameraInfo1DmkII"
    } else if model.contains("1D") && model.contains("Mark III") {
        "Canon::CameraInfo1DmkIII"
    } else {
        "Canon::CameraInfoUnknown"
    }
}
```

## Testing Strategy

1. **Unit Tests**: Test condition evaluation in isolation
2. **Integration Tests**: Test processor selection with real camera files
3. **Performance Tests**: Ensure regex caching works effectively
4. **Compatibility Tests**: Compare output with ExifTool

## Future Extensions

- Support for more complex conditions (firmware version, lens type)
- Condition debugging/logging for troubleshooting
- GUI tool for visualizing condition evaluation

## Related Documentation

- [PROCESSOR-PROC-DISPATCH.md](../PROCESSOR-PROC-DISPATCH.md) - Detailed dispatch design
- [IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md) - How to add processors
- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - Access to model and other state