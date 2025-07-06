# Milestone 15: XMP/XML Support

**Duration**: 3-4 weeks  
**Goal**: Add comprehensive XMP metadata extraction with structured output (equivalent to `exiftool -j -struct`)  
**XML Parser**: `quick-xml` v0.36 (validated - see implementation guide)

## Task List Summary

### Phase 0: Codegen Infrastructure (Week 1)
1. Extract XMP namespace tables (`%nsURI`, `%xmpNS`) via simple table framework
2. Extract XML character escape tables (`%charName`, `%charNum`)
3. Extract 50+ PrintConv lookup tables from XMP.pm/XMP2.pl
4. Validate generated code compiles and matches ExifTool data

### Phase 1: TagValue Architecture Enhancement (Week 1-2)
1. Add `Object(HashMap<String, TagValue>)` variant to TagValue enum
2. Add `Array(Vec<TagValue>)` variant to TagValue enum
3. Update JSON serialization to handle nested structures
4. Write unit tests for new TagValue variants
5. Update existing code to handle new variants gracefully

### Phase 2: Core XMP Processing (Week 2-3)
1. Implement standalone .xmp file reader (simplest case)
2. Create XmpProcessor with namespace resolution
3. Implement RDF/XML parsing using quick-xml
4. Build structure tree (no flattening)
5. Convert RDF containers to appropriate types (Bag/Seq→Array, Alt→Object)

### Phase 3: Format Integration (Week 3)
1. Add JPEG APP1 XMP segment extraction
2. Add TIFF IFD0 XMP tag (0x02bc) processing  
3. Handle Extended XMP in JPEG (multi-segment)
4. Integrate with processor dispatch system

### Phase 4: Testing & Validation (Week 4)
1. Generate reference outputs with `exiftool -j -struct`
2. Compare our output for semantic equivalence
3. Performance profiling vs ExifTool
4. Fix compatibility issues

## Overview

XMP (eXtensible Metadata Platform) is Adobe's RDF/XML-based metadata standard, used extensively in JPEG, TIFF, PSD, and other formats. This milestone establishes XMP processing capabilities using structured output exclusively.

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

**Decision**: **Option B** - Use `quick-xml` crate (already in dependencies) with fail-fast approach for malformed XML. Analysis of ExifTool's XMP tests shows focus on well-formed XML rather than malformed edge cases. We should trust ExifTool's namespace definitions and tag structures, not necessarily its parsing implementation.

### XML Parser: quick-xml v0.36

**Validation completed**: Comprehensive testing confirms `quick-xml` meets all XMP requirements:

- **Namespace Resolution**: ✅ Full support via `NsReader` and `resolve_element()`/`resolve_attribute()`
- **RDF Containers**: ✅ Successfully parsed Bag/Seq/Alt with language alternatives
- **Performance**: ✅ 1.6 million events/second on 26KB document
- **UTF-8/Unicode**: ✅ Handles Japanese/Chinese/Arabic/emoji perfectly
- **Streaming**: ✅ Low memory usage, no DOM construction
- **Error Handling**: ✅ Properly rejects malformed XML

See `examples/validate_quick_xml_xmp_v3.rs` for working validation code

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
      "rights": { "x-default": "Copyright notice" }
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

### Phase 0: Codegen Infrastructure Setup (Week 1)

**Extract XMP Tables via Simple Table Framework**:

Before writing any XMP processing code, we need the lookup tables:

```bash
# Priority extractions:
1. %nsURI - 100+ namespace URI mappings (XMP.pm)
2. %xmpNS - ExifTool group translations (XMP.pm) 
3. %charName/%charNum - XML escapes (XMP.pm)
4. PrintConv tables from XMP.pm/XMP2.pl
```

**Implementation Steps**:
1. Analyze XMP.pm to identify all simple hash tables
2. Add entries to codegen/simple_tables.json
3. Run `make codegen-simple-tables`
4. Verify generated code in src/generated/xmp/

### Phase 1: TagValue Enhancement for Structured Data (Week 1-2)

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

### Phase 2: Core XMP Processing Implementation (Week 2-3)

**Processing Order** (simplest to complex):
1. **Start with standalone .xmp files** - Pure XML, no container extraction needed
2. **Then JPEG APP1 segments** - Single XMP packet in APP1 marker
3. **Finally Extended XMP** - Multi-segment handling in JPEG

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

### Phase 3: Format Integration (Week 3)

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

**Format Integration Priority**:

1. **Standalone .xmp files** (simplest - pure XML):
   - Direct file reading
   - No container format complications
   - Test files: XMP.xmp, XMP2.xmp, etc.

2. **JPEG APP1 XMP segments**:
   - Extract from APP1 marker (0xFFE1)
   - Identifier: "http://ns.adobe.com/xap/1.0/\0"
   - Test files: XMP.jpg, PhotoMechanic.jpg

3. **TIFF IFD0 XMP tag**:
   - Tag 0x02bc (700) in IFD0
   - Contains XMP packet as string
   - Integration with existing TIFF processor

4. **Extended XMP** (JPEG only):
   - Multiple APP1 segments
   - GUID-based reassembly
   - Test file: ExtendedXMP.jpg

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
      "rights": { "x-default": "Copyright notice" }
    },
    "ContactInfo": {
      "CiAdrCity": "New York",
      "CiEmailWork": "contact@example.com"
    }
  }
}
```

## Detailed Codegen Requirements

### Phase 0 Implementation Details

Based on analysis of XMP.pm and XMP2.pl, we need to extract **80+ lookup tables** before implementing XMP processing:

### Critical Tables for Initial Implementation

**Week 1 Codegen Tasks**:

1. **Namespace Tables** (MUST HAVE):
```json
{
  "module": "XMP.pm",
  "hash_name": "%nsURI",
  "output_file": "xmp/namespace_uris.rs",
  "constant_name": "NAMESPACE_URIS",
  "key_type": "string",
  "value_type": "string",
  "description": "XMP namespace prefix to URI mappings"
}
```

2. **Group Name Translations**:
```json
{
  "module": "XMP.pm", 
  "hash_name": "%xmpNS",
  "output_file": "xmp/group_names.rs",
  "constant_name": "XMP_GROUP_NAMES",
  "key_type": "string",
  "value_type": "string"
}
```

3. **XML Character Escapes**:
```json
{
  "module": "XMP.pm",
  "hash_name": "%charName",
  "output_file": "xmp/char_names.rs",
  "constant_name": "XML_CHAR_NAMES",
  "key_type": "string",
  "value_type": "string"
}
```

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

### Complex Structure Handling Strategy

**Current Limitation**: Some XMP structures in ExifTool (like `%sCorrectionMask`, `%sTime`) contain nested hash references and metadata that exceed our simple table framework.

**Milestone 15 Approach**:
1. Extract simple lookup tables only (namespaces, PrintConv mappings)
2. Complex structures will be parsed dynamically from XML
3. Focus on mainstream namespaces that don't require complex structure definitions
4. Leave complex Adobe-specific structures for future milestones

**Rationale**: The XMP processor reads structure directly from the XML, so we don't need to pre-define every possible structure. ExifTool's structure definitions are mainly for writing XMP, which is out of scope for this milestone.

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

- XMP metadata extraction with structured output only
- Core namespace and structure support via codegen
- Hierarchical JSON output matching `exiftool -j -struct`
- Essential PrintConv functions for value display

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
- [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md) - Unified code generation and implementation guide

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

1. **Standalone XMP files** (10 test files available):
   - `XMP.xmp` through `XMP9.xmp` - Various namespace and structure tests
   - `PLUS.xmp` - PLUS licensing metadata
   
2. **JPEG with XMP** (4 test files available):
   - `XMP.jpg` - Basic XMP in APP1 segment
   - `ExtendedXMP.jpg` - Multi-segment extended XMP
   - `PhotoMechanic.jpg` - Real-world Photo Mechanic XMP
   - `MWG.jpg` - Metadata Working Group compliance

3. **Structure validation**:
   - Nested structures (ContactInfo, LocationCreated)
   - RDF containers (Bag, Seq, Alt)
   - Language alternatives (dc:title with xml:lang)
   - Namespace grouping (dc:*, xmp:*, exif:*)

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

## quick-xml Implementation Guide

### Key API Patterns

**Using NsReader for namespace-aware parsing**:
```rust
use quick_xml::reader::NsReader;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;

let mut reader = NsReader::from_str(xmp_data);
reader.config_mut().trim_text(true);

let mut buf = Vec::new();
loop {
    match reader.read_event_into(&mut buf)? {
        Event::Start(e) => {
            let (ns_result, local) = reader.resolve_element(e.name());
            if let ResolveResult::Bound(ns) = ns_result {
                let namespace_uri = str::from_utf8(ns.as_ref())?;
                let local_name = str::from_utf8(local.as_ref())?;
                // Process element with namespace
            }
        }
        Event::Text(e) => {
            let text = e.unescape()?.into_owned();
            // Process text content
        }
        Event::Eof => break,
        _ => {}
    }
    buf.clear();
}
```

**Extracting attributes (e.g., xml:lang)**:
```rust
for attr in element.attributes() {
    let attr = attr?;
    let (_, attr_local) = reader.resolve_attribute(attr.key);
    let attr_name = str::from_utf8(attr_local.as_ref())?;
    if attr_name == "lang" {
        let lang_code = str::from_utf8(&attr.value)?;
        // Process language code
    }
}
```

### RDF Container Detection Patterns

- **rdf:Bag** → Track state, collect items, output as `TagValue::Array`
- **rdf:Seq** → Same as Bag (order preserved by XML parsing)
- **rdf:Alt** → Track language codes from `xml:lang`, output as `TagValue::Object`

### Helpful Resources

- **quick-xml docs**: https://docs.rs/quick-xml/0.36/
- **Example code**: `/examples/validate_quick_xml_xmp_v3.rs`
- **ExifTool XMP tests**: `/third-party/exiftool/t/images/XMP*.jpg`

## Implementation Notes for Engineers

### Starting Implementation

1. **First Steps**:
   - Run codegen for XMP tables (Phase 0)
   - Implement TagValue::Object and TagValue::Array
   - Start with standalone .xmp file parsing (simplest case)

2. **Key Design Decisions**:
   - We output ONLY structured mode (no flattening)
   - Use quick-xml for parsing (already in dependencies)
   - Single XMP TagEntry containing entire structure
   - Trust ExifTool's namespace definitions via codegen

3. **Common Pitfalls**:
   - Don't try to flatten XMP structures
   - Don't manually maintain namespace tables
   - Don't parse XML with regex
   - Remember to handle UTF-8/16/32 BOMs
   - Use `read_event_into()` with buffer reuse for performance
   - Always call `buf.clear()` after each event to avoid memory growth
   - Use `e.unescape()?.into_owned()` for text content to handle XML entities

4. **Testing Approach**:
   - Generate reference with: `exiftool -j -struct test.xmp > ref.json`
   - Compare semantic structure, not exact string format
   - Use available test files in third-party/exiftool/t/images/

### XMP Packet Detection

**Standalone .xmp files**:
- Direct XML parsing, no packet extraction needed
- May have XML declaration: `<?xml version="1.0"?>`
- Root element: `<x:xmpmeta xmlns:x="adobe:ns:meta/">`

**JPEG APP1 segments**:
- Marker: 0xFFE1 (APP1)
- Identifier: "http://ns.adobe.com/xap/1.0/\0" (29 bytes)
- XMP packet follows identifier
- Packet format: `<?xpacket begin="﻿" id="W5M0MpCehiHzreSzNTczkc9d"?>...<?xpacket end="r"?>`

**Extended XMP in JPEG**:
- Multiple APP1 segments with identifier "http://ns.adobe.com/xmp/extension/\0"
- Contains MD5 digest (32 bytes) and offset/length info
- Reassemble segments in order before parsing

**TIFF IFD0**:
- Tag 0x02bc (XMP tag)
- Contains complete XMP packet as byte array
- Same packet format as JPEG

### Expected Output Structure

Our XMP processor should return a single TagEntry:
```rust
TagEntry {
    tag_id: "XMP",
    value: TagValue::Object({
        "dc" => TagValue::Object({
            "creator" => TagValue::Array([...]),
            "title" => TagValue::Object({
                "x-default" => TagValue::String("Title")
            })
        }),
        "xmp" => TagValue::Object({...}),
        // ... other namespaces
    }),
    // ... other fields
}
```

This milestone establishes XMP as a structured metadata enhancement while maintaining compatibility with ExifTool's battle-tested approach. The focus on structured output (`-struct` mode) provides richer metadata access while the simple table codegen framework eliminates maintenance burden for the 80+ lookup tables.
