# Processor Dispatch Guide

**🚨 CRITICAL: This dispatch system faithfully translates ExifTool patterns per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).**

**Understanding exif-oxide's processor selection system and its relationship to ExifTool**

## Overview

Processor dispatch is the system that determines which processing function to use for a given piece of EXIF data. This guide explains how exif-oxide's sophisticated trait-based system maps to ExifTool's simpler function-pointer approach, and why this translation is both necessary and faithful to ExifTool's design.

## ExifTool's Processor Dispatch

### Core Dispatch Logic

ExifTool's processor selection follows a simple three-tier fallback pattern at `ExifTool.pm:8950`:

```perl
$proc or $proc = $$tagTablePtr{PROCESS_PROC} || \&Image::ExifTool::Exif::ProcessExif;
```

This single line implements the entire dispatch hierarchy:

1. **Explicit ProcessProc** - Function passed as parameter (highest priority)
2. **Table PROCESS_PROC** - Function specified in tag table definition
3. **Default ProcessExif** - Fallback processor for standard EXIF data

### Table-Driven Processor Assignment

Tag tables specify their processors through the `PROCESS_PROC` field:

```perl
%Image::ExifTool::Canon::AFInfo = (
    PROCESS_PROC => \&ProcessSerialData,
    # ... tag definitions
);

%Image::ExifTool::Canon::SensorInfo = (
    PROCESS_PROC => \&Image::ExifTool::ProcessBinaryData,
    # ... tag definitions
);
```

### Conditional Dispatch

SubDirectories can specify conditional processors:

```perl
SubDirectory => {
    TagTable => 'Image::ExifTool::Canon::AFInfo',
    ProcessProc => \&ProcessSerialData,
    Condition => '$$self{Model} =~ /EOS R5/',
}
```

### Key Characteristics

- **Simple function pointers** - Direct function dispatch
- **Table-driven** - Each tag table defines its processor
- **Implicit complexity** - Sophisticated logic hidden inside processors
- **Dynamic conditions** - Runtime evaluation of Perl expressions
- **Context passing** - `$self` and `$dirInfo` carry all state

## exif-oxide's Processor Dispatch

### Architecture Overview

exif-oxide implements a trait-based registry system that captures ExifTool's patterns while adding type safety and explicit structure:

```rust
pub trait BinaryDataProcessor: Send + Sync {
    fn assess_capability(&self, context: &ProcessorContext) -> ProcessorCapability;
    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult>;
}
```

### Core Components

#### 1. ProcessorRegistry

Central registry managing all available processors:

```rust
pub struct ProcessorRegistry {
    processors: HashMap<ProcessorKey, Arc<dyn BinaryDataProcessor>>,
    dispatch_rules: Vec<Box<dyn DispatchRule>>,
}
```

#### 2. ProcessorContext

Structured context carrying all necessary information:

```rust
pub struct ProcessorContext {
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub table_name: String,
    pub tag_id: Option<u16>,
    pub parent_tags: HashMap<String, TagValue>,
    // ... other fields
}
```

#### 3. ProcessorCapability

Four-tier capability assessment system:

```rust
pub enum ProcessorCapability {
    Perfect,     // Exact match for manufacturer/model/format
    Good,        // Compatible but not optimal
    Fallback,    // Can process but with limitations
    Incompatible // Cannot process
}
```

#### 4. DispatchRule Trait

Sophisticated conditional dispatch logic:

```rust
pub trait DispatchRule: Send + Sync {
    fn applies_to(&self, context: &ProcessorContext) -> bool;
    fn priority(&self) -> u8;
}
```

### Dispatch Flow

1. **Context Creation** - Build ProcessorContext from ExifReader state
2. **Rule Evaluation** - Check which dispatch rules apply to context
3. **Capability Assessment** - Evaluate each applicable processor's capability
4. **Selection** - Choose processor with highest capability + priority
5. **Processing** - Execute selected processor's `process_data()` method

## Architectural Mapping

| ExifTool Pattern | exif-oxide Equivalent | Relationship |
|------------------|----------------------|--------------|
| `$proc` parameter | Direct registry lookup | 1:1 correspondence |
| `PROCESS_PROC` field | DispatchRule system | Table-driven selection |
| `ProcessExif` fallback | StandardExifProcessor | Default processor |
| `$self` + `$dirInfo` | ProcessorContext | Structured context |
| Condition expressions | ProcessorCapability | Enhanced assessment |
| Function pointers | Trait objects | Type-safe dispatch |

## Implementation Comparison

### ExifTool Approach: Simple and Direct

**Pros:**
- **Minimal overhead** - Direct function calls
- **Simple to understand** - Clear function pointer dispatch
- **Flexible conditions** - Full Perl expression power
- **Dynamic dispatch** - Runtime function selection

**Cons:**
- **Runtime errors** - No compile-time safety
- **Implicit complexity** - Hard to debug dispatch logic
- **Limited introspection** - Can't easily inspect available processors
- **Maintenance burden** - Condition expressions scattered throughout code

### exif-oxide Approach: Structured and Safe

**Pros:**
- **Type safety** - Compile-time verification of processor interfaces
- **Explicit complexity** - Clear dispatch rules and capabilities
- **Rich debugging** - Full introspection of processor selection
- **Maintainability** - Centralized dispatch logic
- **Performance optimization** - Registry can cache processor lookups

**Cons:**
- **Runtime overhead** - Registry lookup and capability assessment
- **Implementation complexity** - More code than simple function pointers
- **Learning curve** - Requires understanding trait system
- **Potential over-abstraction** - May be more complex than needed

## Alternative Approaches for Future Consideration

### 1. Hybrid Function-Pointer System

Closer to ExifTool's approach while maintaining type safety:

```rust
pub struct ProcessorTable {
    process_func: fn(&[u8], &ProcessorContext) -> Result<ProcessorResult>,
    condition: Option<fn(&ProcessorContext) -> bool>,
}

pub static PROCESSOR_TABLES: &[(&str, ProcessorTable)] = &[
    ("Canon::AFInfo", ProcessorTable {
        process_func: canon_serial_data_processor,
        condition: Some(|ctx| ctx.manufacturer == Some("Canon")),
    }),
    // ... more tables
];
```

**Pros:**
- Simpler dispatch logic
- Lower runtime overhead
- Closer to ExifTool's pattern

**Cons:**
- Less type safety
- Harder to unit test individual processors
- Limited introspection capabilities

### 2. Macro-Generated Dispatch

Use Rust macros to generate dispatch logic from table definitions:

```rust
processor_table! {
    Canon::AFInfo => {
        processor: canon_serial_data_processor,
        condition: manufacturer == "Canon",
    },
    Canon::SensorInfo => {
        processor: process_binary_data,
        condition: manufacturer == "Canon" && table_name.starts_with("Canon::"),
    },
}
```

**Pros:**
- Declarative table definitions
- Compile-time generation
- Close to ExifTool's table syntax

**Cons:**
- Complex macro implementation
- Debugging difficulty
- Limited runtime flexibility

### 3. Codegen from ExifTool Tables

Generate dispatch code directly from ExifTool's Perl tag tables:

```rust
// Generated from ExifTool's Canon.pm
pub static CANON_PROCESSORS: &[ProcessorEntry] = &[
    ProcessorEntry {
        table_name: "Canon::AFInfo",
        processor: canon_af_info_processor,
        condition: "$$self{Model} =~ /EOS R5/",
    },
    // ... auto-generated from Canon.pm
];
```

**Pros:**
- Perfect fidelity to ExifTool
- Automatic updates with ExifTool releases
- No manual translation needed

**Cons:**
- Requires Perl condition parsing
- Complex codegen infrastructure
- Potential for subtle translation errors

## Current Implementation Assessment

### Strengths

1. **Faithful to ExifTool's patterns** - Captures the three-tier fallback logic
2. **Type safety** - Prevents many runtime errors
3. **Excellent debugging** - Rich introspection capabilities
4. **Maintainable** - Clear separation of concerns
5. **Extensible** - Easy to add new processors and manufacturers

### Areas for Improvement

1. **Performance optimization** - Registry lookup could be faster
2. **Condition language** - ProcessorCapability is simpler than Perl conditions
3. **Table fidelity** - Could be closer to ExifTool's table definitions
4. **Processor variants** - May need more dynamic processor selection

## Recommendations

### Short Term (Current Implementation)

1. **Continue with trait-based system** - It's working well and provides good debugging
2. **Optimize registry performance** - Cache processor lookups for common cases
3. **Enhance condition evaluation** - Add more sophisticated context matching
4. **Improve documentation** - Help engineers understand the dispatch flow

### Long Term (Future Considerations)

1. **Evaluate hybrid approach** - Consider function-pointer system for hot paths
2. **Explore codegen options** - Generate dispatch tables from ExifTool source
3. **Performance benchmarking** - Compare against simpler dispatch methods
4. **Condition language enhancement** - Support more complex conditional logic

## Conclusion

exif-oxide's processor dispatch system successfully captures ExifTool's patterns while adding type safety and explicit structure. The trait-based approach is **architecturally equivalent** to ExifTool's function-pointer system and maintains the **Trust ExifTool principle** by preserving the underlying dispatch logic.

The current implementation prioritizes correctness, maintainability, and debugging capability over raw performance. This is appropriate for the current development phase, but future optimizations may consider simpler approaches for performance-critical paths.

**Key Insight**: Both systems follow the same logical pattern - context evaluation, capability assessment, processor selection, and processing execution. The difference is in implementation philosophy: ExifTool uses implicit complexity with simple dispatch, while exif-oxide uses explicit complexity with structured dispatch.

The architecture successfully bridges the gap between Perl's dynamic dispatch and Rust's type-safe systems while maintaining fidelity to ExifTool's proven patterns.