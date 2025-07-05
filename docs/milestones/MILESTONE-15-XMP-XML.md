# Milestone 15: XMP/XML Support

**Duration**: 3-4 weeks  
**Goal**: Add comprehensive XMP metadata extraction and basic XML parsing infrastructure

## Overview

XMP (eXtensible Metadata Platform) is Adobe's RDF/XML-based metadata standard, used extensively in JPEG, TIFF, PSD, and other formats. This milestone establishes XMP processing capabilities while addressing a critical architectural decision about XML parsing approach.

## Background Analysis

From ExifTool's XMP implementation:

- **8,773 total lines** across XMP.pm (4,642), XMP2.pl (2,467), WriteXMP.pl (1,664)
- **72 namespace tables** covering Adobe, IPTC, PLUS, industry standards
- **Complex RDF/XML processing** with hierarchical structure preservation
- **No external XML dependencies** - uses pure Perl regex-based parsing

### ⚠️ CRITICAL: Structured Output Mode

**exif-oxide operates exclusively in structured mode** - equivalent to `exiftool -j -struct`. This means:

- **Preserve hierarchical XMP structures** rather than flattening to individual tags
- **Maintain RDF containers** (Bag, Seq, Alt) as arrays and objects
- **Group properties by namespace** (dc, xmp, exif, etc.)
- **Retain language alternatives** with proper lang-key structures
- **Output single XMP TagEntry** containing the entire nested structure

### Structured vs Flattened Comparison

**Flattened mode** (what we DON'T do):
```
ContactInfo.CiAdrCity = "New York"
dc:creator[1] = "John Doe"  
dc:creator[2] = "Jane Smith"
dc:title[x-default] = "Photo Title"
```

**Structured mode** (our target):
```json
{
  "XMP": {
    "ContactInfo": {
      "CiAdrCity": "New York"
    },
    "dc": {
      "creator": ["John Doe", "Jane Smith"],
      "title": { "x-default": "Photo Title" }
    }
  }
}
```

## Key Architectural Decision

**⚠️ CRITICAL DESIGN CHOICE**: XML Parsing Implementation

ExifTool uses regex-based XML parsing to avoid external dependencies. For exif-oxide, we have two options:

### Option A: Follow ExifTool's Regex Approach

**Pros**:

- Exact behavioral compatibility
- No external dependencies
- Handles any XML edge cases ExifTool handles

**Cons**:

- Complex regex XML parsing (4,000+ lines)
- Maintenance burden
- Non-standard XML handling

### Option B: Use Rust XML Parsing Crate

**Pros**:

- Robust, standards-compliant XML parsing
- Maintainable code
- Better error handling
- Follows Rust ecosystem best practices

**Cons**:

- External dependency
- May reject malformed XML that ExifTool accepts (this is acceptable, but needs to be noted in the README)
- Different error behavior

**Recommendation**: **Option B** with fail-fast approach for malformed XML. Analysis of ExifTool's XMP tests shows focus on well-formed XML rather than malformed edge cases. We should trust ExifTool's namespace definitions and tag structures, not necessarily its parsing implementation.

## ExifTool Structured Mode Analysis

### How ExifTool `-struct` Mode Works

ExifTool's `-struct` option preserves the hierarchical nature of XMP data structures rather than flattening them into individual tags. Key behaviors:

**RDF Container Mappings**:
- **rdf:Bag** → JSON Array: `["item1", "item2", "item3"]`
- **rdf:Seq** → JSON Array: `["first", "second", "third"]` 
- **rdf:Alt** → JSON Object: `{"x-default": "value", "en-US": "English value"}`

**Namespace Grouping**:
```json
{
  "XMP": {
    "dc": {
      "creator": ["Photographer Name"],
      "subject": ["keyword1", "keyword2"],
      "rights": {"x-default": "Copyright notice"}
    },
    "xmp": {
      "CreateDate": "2024-01-01T12:00:00Z",
      "ModifyDate": "2024-01-01T12:30:00Z"
    },
    "exif": {
      "DateTimeOriginal": "2024:01:01 12:00:00",
      "ISO": 100
    }
  }
}
```

**Nested Structure Preservation**:
```json
{
  "ContactInfo": {
    "CiAdrExtadr": "123 Main St",
    "CiAdrCity": "New York", 
    "CiAdrCtry": "USA",
    "CiEmailWork": "contact@example.com"
  },
  "LocationCreated": [
    {
      "City": "Paris",
      "CountryName": "France",
      "GPS": {
        "Latitude": 48.8566,
        "Longitude": 2.3522
      }
    }
  ]
}
```

### Implementation Requirements

**TagValue Extensions Needed**:
```rust
pub enum TagValue {
    // ... existing variants
    Object(HashMap<String, TagValue>),  // For nested structures
    Array(Vec<TagValue>),               // For RDF containers
}
```

**Structure Building Algorithm**:
1. Parse XML with namespace awareness
2. Build hierarchical object tree preserving nesting
3. Convert RDF containers to appropriate JSON types
4. Group properties by namespace prefix
5. Return single XMP TagEntry with nested TagValue::Object

## Implementation Strategy

### Phase 1: TagValue Enhancement for Structured Data (Week 1)

**Extend TagValue for Nested Structures**:

```rust
pub enum TagValue {
    String(String),
    Number(i64), 
    Float(f64),
    Object(HashMap<String, TagValue>),  // NEW: For nested XMP structures
    Array(Vec<TagValue>),               // NEW: For RDF containers
    // ... existing variants
}
```

**XMP Structure Types**:

```rust
pub enum XmpContainer {
    Bag(Vec<TagValue>),     // Unordered collection → Array
    Seq(Vec<TagValue>),     // Ordered sequence → Array  
    Alt(HashMap<String, TagValue>), // Language alternatives → Object
}

pub struct XmpStructure {
    namespace: String,
    properties: HashMap<String, TagValue>,
}
```

### Phase 2: Structured XMP Processor (Week 2)

**Core Architecture**:

```rust
pub struct XmpProcessor {
    namespace_resolver: NamespaceResolver,
    structure_builder: StructureBuilder,  // NOT flattener!
}

impl XmpProcessor {
    pub fn process_xmp_data(&self, data: &[u8]) -> Result<TagEntry> {
        // Returns single TagEntry with TagValue::Object containing:
        // {
        //   "dc": { "creator": ["John"], "subject": ["kw1", "kw2"] },
        //   "xmp": { "CreateDate": "2024-01-01T12:00:00Z" },
        //   "exif": { "DateTimeOriginal": "2024:01:01 12:00:00" }
        // }
    }
}
```

**Structure Building (No Flattening)**:

```rust
pub struct StructureBuilder {
    // Builds nested TagValue::Object from RDF/XML
    // Preserves:
    // - Namespace grouping: dc:creator → "dc": { "creator": [...] }
    // - RDF containers: rdf:Bag → Array, rdf:Alt → Object with lang keys
    // - Nested structures: ContactInfo → Object with sub-properties
}
```

### Phase 3: RDF Container and Namespace Handling (Week 3)

**Codegen for XMP Lookup Tables**:

```bash
# Add 80+ XMP lookup tables to simple table extraction framework
# Priority tables identified:
# - %nsURI (100+ namespace URI mappings) 
# - %xmpNS (ExifTool group name translations)
# - %charName/%charNum (XML character escapes)
# - All PrintConv tables from XMP.pm/XMP2.pl (even in struct mode, some values need conversion)

# Example simple_tables.json entries:
{
  "module": "XMP.pm",
  "hash_name": "%nsURI", 
  "output_file": "xmp/namespaces.rs",
  "constant_name": "NAMESPACE_URIS",
  "key_type": "string",
  "value_type": "string",
  "description": "XMP namespace prefix to URI mappings"
}
```

**Namespace Management**:

```rust
// Use generated lookup tables from simple table extraction
use crate::generated::xmp::namespaces::lookup_namespace_uri;
use crate::generated::xmp::char_escapes::lookup_xml_char_name;

pub struct NamespaceResolver {
    // Generated from %nsURI via codegen
    // No manual maintenance required!
}
```

**RDF Container Processing**:

```rust
// Map ExifTool's RDF containers to structured output:
// rdf:Bag/rdf:Seq → TagValue::Array([item1, item2, ...])
// rdf:Alt → TagValue::Object({"x-default": "value", "en-US": "English"})
```

**Format Integration**:

- JPEG APP1 XMP segment extraction  
- TIFF IFD0 XMP tag processing
- Standalone .xmp sidecar file support
- Extended XMP multi-segment handling

### Phase 4: Integration and Compatibility Testing (Week 4)

**Processor Dispatch Integration**:

```rust
// Add XmpDispatchRule to existing system
// Returns single XMP TagEntry with nested structure
pub fn select_processor() -> TagEntry {
    TagEntry {
        tag_id: "XMP",
        value: TagValue::Object(xmp_structure), // Entire XMP tree
        // ...
    }
}
```

**Testing Strategy**:

- Compare with `exiftool -j -struct` output for semantic equivalence
- Validate RDF container representations (Bag/Seq/Alt)
- Test namespace grouping preservation
- Verify nested structure handling

## Success Criteria

### Core Requirements

- [ ] **Structured Output**: Single XMP TagEntry with nested TagValue::Object structure
- [ ] **Namespace Grouping**: Properties grouped by namespace (dc, xmp, exif, etc.)
- [ ] **RDF Container Support**: Bag/Seq → Arrays, Alt → Objects with lang keys
- [ ] **Hierarchy Preservation**: Nested structures maintained (ContactInfo, LocationCreated)
- [ ] **Format Support**: JPEG APP1, TIFF IFD0, standalone .xmp files processed
- [ ] **Codegen Integration**: 80+ lookup tables generated automatically

### Validation Tests

- **JSON Compatibility**: Output matches `exiftool -j -struct` format semantically
- **Structure Preservation**: Nested objects/arrays correctly represented
- **Namespace Resolution**: All 72 ExifTool namespaces supported via codegen
- **Container Handling**: RDF Bag/Seq/Alt properly converted to JSON structures
- **Multi-format Support**: Process `t/images/XMP.jpg`, `ExtendedXMP.jpg`, standalone `.xmp` files

### Technical Validation

```json
// Expected output structure:
{
  "XMP": {
    "dc": {
      "creator": ["Photographer Name"],
      "subject": ["keyword1", "keyword2"], 
      "rights": {"x-default": "Copyright notice"}
    },
    "ContactInfo": {
      "CiAdrCity": "New York",
      "CiEmailWork": "contact@example.com"
    }
  }
}
```

## Simple Table Codegen Opportunities

Based on comprehensive analysis of XMP.pm and XMP2.pl, identified **80+ lookup tables** perfect for simple table extraction:

### High-Priority Tables (Week 1)

**Core Infrastructure Tables**:
- `%nsURI` (100+ entries) - Namespace prefix to URI mappings
- `%xmpNS` (7 entries) - ExifTool group name translations  
- `%charName`/`%charNum` (5 entries each) - XML character escapes

**PrintConv Lookup Tables** (50+ tables):
- Color Mode mappings (CMYK, RGB, LAB)
- Photoshop ColorMode (9 entries)
- Urgency levels (10 entries) 
- White Balance settings (9 entries)
- Exposure Program modes (9 entries)
- Metering Mode types (7 entries)
- Flash settings (multiple small tables)
- GPS direction references (multiple tables)
- Time format mappings (10 entries)
- Audio/Video type lookups (20+ entries)

### Codegen Implementation

```bash
# Add to codegen/simple_tables.json:
{
  "tables": [
    {
      "module": "XMP.pm",
      "hash_name": "%nsURI",
      "output_file": "xmp/namespaces.rs", 
      "constant_name": "NAMESPACE_URIS",
      "key_type": "string",
      "value_type": "string"
    },
    {
      "module": "XMP.pm", 
      "hash_name": "%charName",
      "output_file": "xmp/xml_chars.rs",
      "constant_name": "XML_CHAR_NAMES",
      "key_type": "string",
      "value_type": "string"
    }
    // ... 78+ more tables
  ]
}

# Generate with:
make codegen-simple-tables
```

**Estimated Maintenance Savings**: 400+ manually maintained key-value pairs eliminated.

### Codegen Enhancement Required: Nested Hash Support

Many XMP structures contain **nested hash definitions** that exceed current simple table framework:

```perl
# ExifTool XMP.pm - Complex nested structures
%sCorrectionMask = (
    STRUCT_NAME => 'CorrectionMask',
    NAMESPACE   => 'crs',
    What => { List => 0 },
    MaskActive => { Writable => 'boolean', List => 0 },
    Masks => { Struct => \%sOtherMask, NoSubStruct => 1 },
    # Nested reference to another structure
);

%sTime = (
    STRUCT_NAME => 'Time', 
    NAMESPACE   => 'xmpDM',
    scale => { Writable => 'rational' },
    value => { Writable => 'integer' },
);
```

**Enhancement Needed**: Extend simple table framework to handle:
- Nested hash references (`Struct => \%otherHash`)
- Complex value types (`Writable => 'boolean'`, `List => 0`)
- Structure metadata (`STRUCT_NAME`, `NAMESPACE`)

**Alternative Approach**: Extract nested structures to separate generated files with cross-references:
```rust
// Generated from nested hash extraction:
pub struct CorrectionMask {
    pub what: Option<String>,
    pub mask_active: Option<bool>, 
    pub masks: Option<Vec<OtherMask>>,
}
```

## Implementation Dependencies

### Codegen Infrastructure

- **Simple table extraction framework** for 80+ XMP lookup tables
- **Namespace URI generation** from ExifTool %nsURI definitions
- **PrintConv table generation** for value conversion in structured mode

### External Dependencies

- **XML Parser**: `quick-xml` (already in Cargo.toml) for robust XML processing
- **Encoding Detection**: UTF-8/16/32 BOM detection and conversion
- **Generated lookups**: Auto-generated namespace and conversion tables

## Scope Boundaries

### Goals (Milestone 15)

- Basic XMP metadata extraction from mainstream formats
- Core namespace and structure support
- Flattened tag output compatible with ExifTool
- Essential PrintConv functions for common tags

### Non-Goals (Future Milestones)

- XMP writing/modification (Milestone 21)
- Complex validation and MWG compliance
- Advanced RDF features (collections, reification)
- Extended format support (SVG, PLIST, complex video XMP)

## Dependencies and Future Work

### Prerequisite for Other Milestones

- **Not required** for RAW format support (Milestone 17)
- **Optional enhancement** for JPEG/TIFF metadata completeness
- **Foundation** for advanced metadata workflow features

### Future Milestone Integration

- **Milestone 17**: RAW formats may contain embedded XMP for editing metadata
- **Milestone 21**: Write support will need XMP modification capabilities
- **Video formats**: May contain XMP in metadata tracks

## Risk Mitigation

### TagValue Architecture Complexity Risk

- **Risk**: Adding Object/Array variants to TagValue affects entire codebase
- **Mitigation**: Implement incrementally with comprehensive tests, ensure backward compatibility
- **Fallback**: Start with basic nested support, expand gradually

### Structured Output Compatibility Risk

- **Risk**: Our structured output might differ from ExifTool's `-struct` format
- **Mitigation**: Direct comparison testing with `exiftool -j -struct` output
- **Validation**: Semantic equivalence testing rather than exact string matching

### Codegen Framework Enhancement Risk

- **Risk**: Simple table framework may need significant extension for nested hashes
- **Mitigation**: Assess complexity early, consider manual implementation for complex structures
- **Alternative**: Use simple tables for basic lookups, manual code for nested structures

### XML Parser Choice Risk

- **Risk**: quick-xml behavior differs from ExifTool's regex approach
- **Mitigation**: Comprehensive testing against ExifTool's 54 XMP test cases
- **Benefit**: Better error handling and standards compliance than regex parsing

### Memory Usage Risk

- **Risk**: Nested structures consume more memory than flattened tags
- **Mitigation**: Lazy structure building, stream-based parsing for large XMP blocks
- **Monitoring**: Memory profiling during development

## Related Documentation

### Required Reading

- [XMP.md](../../third-party/exiftool/doc/modules/XMP.md) - ExifTool XMP architecture
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - When to deviate from ExifTool implementation
- [IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md) - Manual implementation patterns

### Missing Documentation to Create

- **XML_PARSING_STRATEGY.md** - Detailed comparison of regex vs proper XML parsing approaches
- **XMP_NAMESPACE_EXTRACTION.md** - Guide to extracting namespace definitions from ExifTool
- **XMP_STRUCTURE_FLATTENING.md** - Deep dive on ExifTool's structure flattening algorithm

## Testing Strategy and ExifTool Compatibility

### Comparison Testing Methodology

**Primary Validation**: Compare with `exiftool -j -struct` output
```bash
# Generate reference output for testing
exiftool -j -struct t/images/XMP.jpg > reference_output.json
exiftool -j -struct t/images/ExtendedXMP.jpg > reference_extended.json

# Test semantic equivalence (not exact string matching)
cargo run -- t/images/XMP.jpg --output-format json > our_output.json
# Compare structure, not formatting
```

**Key Test Cases**:
1. **Basic XMP extraction**: `t/images/XMP.jpg` - standard Dublin Core, EXIF metadata
2. **Extended XMP**: `t/images/ExtendedXMP.jpg` - multi-segment XMP handling
3. **Standalone XMP**: `.xmp` sidecar files - pure XMP document processing
4. **Nested structures**: Files with ContactInfo, LocationCreated hierarchies
5. **RDF containers**: Files with Bag, Seq, Alt collections
6. **Language alternatives**: Multi-language title/description tags

### Structure Validation Tests

**RDF Container Mapping**:
```rust
#[test]
fn test_rdf_bag_to_array() {
    // <rdf:Bag><rdf:li>item1</rdf:li><rdf:li>item2</rdf:li></rdf:Bag>
    // Should become: ["item1", "item2"]
}

#[test] 
fn test_rdf_alt_to_object() {
    // <rdf:Alt><rdf:li xml:lang="x-default">Title</rdf:li></rdf:Alt>
    // Should become: {"x-default": "Title"}
}
```

**Namespace Grouping**:
```rust
#[test]
fn test_namespace_grouping() {
    // Verify dc:creator and dc:subject both appear under "dc" object
    // Verify xmp:CreateDate appears under "xmp" object
}
```

### Performance Benchmarks

**Memory Usage**: Profile memory consumption vs ExifTool for large XMP blocks
**Processing Speed**: Compare parsing speed with ExifTool (target: within 2x)
**Scalability**: Test with files containing extensive XMP structures

### Integration Testing

**Processor Dispatch**: Verify XMP processor integrates with existing system
**TagValue Compatibility**: Ensure Object/Array variants work with JSON output
**Error Handling**: Test malformed XML rejection with proper error messages

This milestone establishes XMP as a structured metadata enhancement while maintaining compatibility with ExifTool's battle-tested approach. The focus on structured output (`-struct` mode) provides richer metadata access while the simple table codegen framework eliminates maintenance burden for the 80+ lookup tables.
