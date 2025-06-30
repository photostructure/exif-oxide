# RESEARCH: Processor Dispatch Design for exif-oxide

## Problem Statement

ExifTool uses a flexible dispatch system for PROCESS_PROC that supports both table-level defaults and SubDirectory-specific overrides. We need to determine if static dispatch is sufficient or if we need dynamic dispatch.

## Research Findings

### 1. ExifTool's Dispatch Hierarchy

**Priority Order** (from `WriteExif.pl:163`):

```perl
my $proc = $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif;
```

1. **SubDirectory ProcessProc**: Highest priority (tag-level override)
2. **Table PROCESS_PROC**: Table default
3. **Fallback**: ProcessExif (or ProcessMOV for QuickTime)

**Key Finding**: The actual dispatch is simple - it's just a 3-level fallback chain. No complex conditionals.

### 2. SubDirectory ProcessProc Override Patterns

**Canon Serial Data Example**:

```perl
# Table level: ProcessBinaryData is default
%Image::ExifTool::Canon::AFInfo = (
    PROCESS_PROC => \&ProcessSerialData,  # Override for entire table
    VARS => { ID_LABEL => 'Sequence' },
    FORMAT => 'int16u',
    # ... tag definitions
);
```

**Nikon Encrypted SubDirectory Example**:

```perl
# Tag-level override within a tag definition
SubDirectory => {
    TagTable => 'Image::ExifTool::Nikon::LensData01',
    ProcessProc => \&ProcessNikonEncrypted,  # Override for this subdirectory only
    WriteProc => \&ProcessNikonEncrypted,
    DecryptStart => 4,                       # Custom parameter
    ByteOrder => 'LittleEndian',            # Another custom parameter
},
```

**Key Finding**: Parameters like DecryptStart, ByteOrder, DirOffset are passed through the SubDirectory hash, not as function arguments.

### 3. Dynamic Dispatch Requirements Analysis

**Conditional Processing Examples**:

1. **Nikon Encrypted Sections** - Runtime data-driven dispatch:

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
{
    Condition => '$$valPt =~ /^0402/', # Different data pattern
    Name => 'LensData0402',
    SubDirectory => {
        TagTable => 'Image::ExifTool::Nikon::LensData0402',
        ProcessProc => \&ProcessNikonEncrypted,
        DecryptStart => 4,
    },
},
```

2. **Canon Model-Based Conditions**:

```perl
{
    Condition => '$$self{Model} =~ /\b1DS?$/',
    SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraInfo1D' },
},
{
    Condition => '$$self{Model} =~ /\b1Ds? Mark II$/',
    SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraInfo1DmkII' },
},
```

**Key Finding**: Conditions are evaluated at RUNTIME based on:

- File data content (`$$valPt =~ /pattern/`)
- Camera model (`$$self{Model}`)
- Other extracted metadata

### 4. Processor Enumeration Analysis

**Core Processors** (from PROCESS_PROC.md analysis):

- ProcessExif (default fallback)
- ProcessBinaryData (most common)
- ProcessTIFF
- ProcessXMP
- ProcessMOV

**Custom Processors** (~50 total):

- ProcessSerialData (Canon AF info)
- ProcessNikonEncrypted (Nikon encrypted sections)
- ProcessJVCText, ProcessFLIRText (text formats)
- ProcessX3FDirectory (Sigma RAW)
- Plus ~40+ more manufacturer-specific processors

**Key Finding**: While there are ~50 custom processors, they're used in a finite number of contexts. Each has a specific signature based on (manufacturer, format, condition).

### 5. Parameter Passing Mechanism

Parameters are passed through the SubDirectory hash, not function signatures:

```perl
SubDirectory => {
    TagTable => 'Image::ExifTool::Nikon::LensData01',
    ProcessProc => \&ProcessNikonEncrypted,
    # These are passed to the processor:
    DecryptStart => 4,
    ByteOrder => 'LittleEndian',
    DirOffset => 14,
    # Plus standard directory info
}
```

**Key Finding**: ExifTool's processor functions have a fixed signature `($self, $dirInfo, $tagTablePtr)` but receive parameters through the `$dirInfo` hash.

## Implementation Recommendations

### Recommendation: Hybrid Approach with Static Enumeration

Based on the research, I recommend a **hybrid approach** that combines static enumeration with runtime condition evaluation:

### 1. Static Processor Registry

```rust
// Generated at build time - enumerate all known processors
pub enum ProcessorType {
    Exif,
    BinaryData,
    TIFF,
    XMP,
    Canon(CanonProcessor),
    Nikon(NikonProcessor),
    // ... ~50 variants total
}

pub enum CanonProcessor {
    SerialData,
    ProcessCMT3,
    // ... Canon-specific processors
}

pub enum NikonProcessor {
    Encrypted,
    // ... Nikon-specific processors
}
```

### 2. Runtime Dispatch with Conditions

```rust
// Generated from tag table analysis
pub struct ProcessorDispatch {
    pub table_processor: Option<ProcessorType>,
    pub subdirectory_overrides: HashMap<TagId, Vec<ConditionalProcessor>>,
}

pub struct ConditionalProcessor {
    pub condition: Option<Condition>,  // None = unconditional
    pub processor: ProcessorType,
    pub parameters: HashMap<String, Value>,
}

pub enum Condition {
    DataPattern(Regex),           // $$valPt =~ /pattern/
    ModelMatch(Regex),           // $$self{Model} =~ /pattern/
    And(Vec<Condition>),
    Or(Vec<Condition>),
}
```

### 3. Dispatch Implementation

```rust
impl ExifReader {
    fn select_processor(
        &self,
        table: &TagTable,
        tag_id: Option<TagId>,
        data: &[u8],
    ) -> (ProcessorType, HashMap<String, Value>) {

        // 1. Check for subdirectory-specific processor with conditions
        if let Some(tag_id) = tag_id {
            if let Some(overrides) = table.dispatch.subdirectory_overrides.get(&tag_id) {
                for conditional in overrides {
                    if self.evaluate_condition(&conditional.condition, data) {
                        return (conditional.processor.clone(), conditional.parameters.clone());
                    }
                }
            }
        }

        // 2. Fall back to table-level processor
        if let Some(processor) = &table.dispatch.table_processor {
            return (processor.clone(), HashMap::new());
        }

        // 3. Final fallback to ProcessExif
        (ProcessorType::Exif, HashMap::new())
    }

    fn evaluate_condition(&self, condition: &Option<Condition>, data: &[u8]) -> bool {
        match condition {
            None => true,
            Some(Condition::DataPattern(regex)) => {
                regex.is_match(data)
            },
            Some(Condition::ModelMatch(regex)) => {
                self.get_value("Model")
                    .and_then(|v| v.as_str())
                    .map(|s| regex.is_match(s))
                    .unwrap_or(false)
            },
            // ... handle other condition types
        }
    }
}
```

### 4. Benefits of This Approach

1. **Fully Enumerable**: All processors are known at build time
2. **Handles Conditions**: Runtime evaluation of data/model-based conditions
3. **Parameter Passing**: Clean parameter mechanism through HashMap
4. **Fallback Logic**: Clear 3-level fallback hierarchy
5. **Extensible**: Easy to add new processors and conditions
6. **Performance**: Static dispatch within each processor type

### 5. Implementation Strategy

1. **Extract All Processors**: Codegen identifies all PROCESS_PROC and ProcessProc references
2. **Build Condition Database**: Parse all Condition expressions into structured format
3. **Generate Dispatch Tables**: Create static dispatch configuration per tag table
4. **Implement Runtime**: Condition evaluation and processor selection logic
5. **Manual Processor Implementation**: Implement the ~50 custom processors as needed

### 6. Addressing Original Design Questions

1. **Static vs Dynamic**: **Hybrid** - Static enumeration, dynamic condition evaluation
2. **Override Mechanism**: **Generated dispatch tables** with condition-based selection
3. **Parameter Passing**: **HashMap-based** parameter passing mimicking ExifTool's approach
4. **Fallback Logic**: **3-level hierarchy** exactly as ExifTool implements

## Conclusion

ExifTool's dispatch system is more structured than initially apparent. While it supports dynamic conditions, the set of processors is finite and enumerable. The hybrid approach provides the flexibility needed for condition-based dispatch while maintaining the performance benefits of static analysis.

The key insight is that ExifTool's complexity comes from the _conditions_ that select processors, not from the dispatch mechanism itself. By capturing these conditions in a structured format at build time, we can replicate ExifTool's behavior while maintaining Rust's type safety and performance characteristics.
