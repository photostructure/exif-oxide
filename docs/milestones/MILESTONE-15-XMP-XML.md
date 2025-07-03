# Milestone 15: XMP/XML Support

**Duration**: 3-4 weeks  
**Goal**: Add comprehensive XMP metadata extraction and basic XML parsing infrastructure

## Overview

XMP (eXtensible Metadata Platform) is Adobe's RDF/XML-based metadata standard, used extensively in JPEG, TIFF, PSD, and other formats. This milestone establishes XMP processing capabilities while addressing a critical architectural decision about XML parsing approach.

## Background Analysis

From ExifTool's XMP implementation:

- **8,773 total lines** across XMP.pm (4,642), XMP2.pl (2,467), WriteXMP.pl (1,664)
- **72 namespace tables** covering Adobe, IPTC, PLUS, industry standards
- **Complex RDF/XML processing** with structure flattening and namespace management
- **No external XML dependencies** - uses pure Perl regex-based parsing

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
- May reject malformed XML that ExifTool accepts
- Different error behavior

**Recommendation**: **Option B** with fail-fast approach for malformed XML. Analysis of ExifTool's XMP tests shows focus on well-formed XML rather than malformed edge cases. We should trust ExifTool's namespace definitions and tag structures, not necessarily its parsing implementation.

## Implementation Strategy

### Phase 1: XML Infrastructure (Week 1)

**Core XML Processing**:

```rust
// Use quick-xml or similar crate for robust parsing
use quick_xml::{Reader, events::Event};

pub struct XmpProcessor {
    namespace_resolver: NamespaceResolver,
    tag_tables: HashMap<String, XmpTagTable>,
    structure_flattener: StructureFlattener,
}

impl XmpProcessor {
    pub fn process_xmp_data(&self, data: &[u8]) -> Result<Vec<TagEntry>> {
        // 1. Detect encoding (UTF-8/16/32)
        // 2. Parse XML with proper parser
        // 3. Extract namespaces and resolve URIs
        // 4. Process RDF elements
        // 5. Apply structure flattening
    }
}
```

**Namespace Management**:

```rust
// Port ExifTool's 72 namespace definitions verbatim
pub struct NamespaceResolver {
    // ExifTool's %nsURI lookup table
    uri_to_prefix: HashMap<String, String>,
    // ExifTool's %stdXlatNS standardization
    prefix_translations: HashMap<String, String>,
}
```

### Phase 2: Core XMP Processing (Week 2)

**RDF Element Processing**:

- Parse RDF/XML structure with proper XML parser
- Extract properties, attributes, and containers (Bag, Seq, Alt)
- Handle language alternatives (`xml:lang`)
- Process blank nodes and references

**Tag Table Integration**:

```rust
// Port ExifTool's tag tables verbatim
pub static XMP_DC_TAGS: XmpTagTable = XmpTagTable {
    namespace: "dc",
    uri: "http://purl.org/dc/elements/1.1/",
    tags: &[
        XmpTag { name: "title", writable: true, list_type: Some(LangAlt) },
        XmpTag { name: "creator", writable: true, list_type: Some(Seq) },
        // ... port all 72 namespace tables
    ],
};
```

### Phase 3: Structure and Format Support (Week 3)

**Structure Flattening**:

```rust
// Port ExifTool's structure flattening logic
pub fn flatten_xmp_structures(
    structures: &HashMap<String, XmpValue>
) -> Vec<TagEntry> {
    // Convert nested structures to flat tags
    // ContactInfo.CiAdrCity = "New York"
    // Preserve namespace prefixes
    // Handle arrays of structures
}
```

**Format Integration**:

- JPEG APP1 XMP segment extraction
- TIFF IFD0 XMP tag processing
- Standalone .xmp sidecar file support
- Basic PSD XMP resource extraction

### Phase 4: Validation and Polish (Week 4)

**Testing and Compatibility**:

- Port ExifTool's XMP test cases (54 tests)
- Validate against known XMP samples
- Compare output with ExifTool for equivalency
- Handle embedded vs sidecar XMP priority

## Success Criteria

### Core Requirements

- [ ] **Namespace Resolution**: All 72 ExifTool namespaces supported
- [ ] **Tag Extraction**: Dublin Core, EXIF, TIFF, Photoshop namespaces working
- [ ] **Structure Flattening**: Nested structures properly flattened to tag names
- [ ] **Format Support**: JPEG, TIFF, standalone .xmp files processed
- [ ] **Compatibility**: Output semantically equivalent to ExifTool for mainstream tags

### Validation Tests

- Process `t/images/XMP.jpg` and extract standard metadata
- Handle `t/images/ExtendedXMP.jpg` (multi-segment XMP)
- Parse standalone `.xmp` sidecar files
- Extract structured data (ContactInfo, Location) with proper flattening

## Implementation Dependencies

### Required ExifTool Analysis

- **Namespace Definitions**: Extract all 72 namespace URI mappings from XMP.pm/XMP2.pl
- **Tag Tables**: Port tag definitions with writable flags and list types
- **Structure Definitions**: Port %s[StructName] structure definitions
- **Conversion Functions**: Identify required PrintConv/ValueConv functions

### External Dependencies

- **XML Parser**: `quick-xml` or `roxmltree` for robust XML processing
- **Encoding Detection**: UTF-8/16/32 BOM detection and conversion
- **URI Handling**: Namespace URI validation and normalization

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

### XML Parser Choice Risk

- **Risk**: Different XML parser behavior vs ExifTool's regex approach
- **Mitigation**: Comprehensive testing against ExifTool's XMP test suite
- **Fallback**: Can implement fail-fast for malformed XML rather than attempting complex regex replication

### Namespace Complexity Risk

- **Risk**: 72 namespaces create maintenance burden
- **Mitigation**: Focus on mainstream namespaces first (Dublin Core, EXIF, TIFF, Photoshop)
- **Implementation**: Use codegen to extract namespace definitions from ExifTool source

### Performance Risk

- **Risk**: XML parsing overhead for large XMP blocks
- **Mitigation**: Stream-based parsing for large files, optional XMP extraction flag

## Related Documentation

### Required Reading

- [XMP.md](../../third-party/exiftool/doc/modules/XMP.md) - ExifTool XMP architecture
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - When to deviate from ExifTool implementation
- [IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md) - Manual implementation patterns

### Missing Documentation to Create

- **XML_PARSING_STRATEGY.md** - Detailed comparison of regex vs proper XML parsing approaches
- **XMP_NAMESPACE_EXTRACTION.md** - Guide to extracting namespace definitions from ExifTool
- **XMP_STRUCTURE_FLATTENING.md** - Deep dive on ExifTool's structure flattening algorithm

This milestone establishes XMP as an enhancement layer for metadata richness while maintaining a pragmatic approach to implementation complexity. The XML parser choice represents a key architectural decision that balances ExifTool compatibility with modern Rust development practices.
