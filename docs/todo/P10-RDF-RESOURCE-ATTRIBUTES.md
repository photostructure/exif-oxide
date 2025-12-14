# P10: RDF Resource Attributes Support

**Status**: Phase 1 (P10a) COMPLETE - 2024-12-13

## Problem

XMP uses RDF attributes to reference external URIs and values. When an XMP element has an empty body, the value comes from `rdf:resource`, `rdf:value`, or `rdf:about` attributes. Currently, exif-oxide ignores these attributes, causing loss of:

- Creative Commons license URLs (`cc:license rdf:resource="..."`)
- Homepage URLs (`ph:homePage rdf:resource="..."`)
- See-also references (`rdfs:seeAlso rdf:resource="..."`)
- Structure type declarations (`rdf:type rdf:resource="..."`)

**Why it matters**: Users cannot extract CC license info, author URLs, or resource references from XMP metadata.

**Success test**:
```bash
# ExifTool shows these values:
third-party/exiftool/exiftool t/images/XMP3.xmp -G | grep -i "HomePage"
# XMP:ProgrammerHomePage: https://exiftool.org/

# exif-oxide should match:
cargo run -- third-party/exiftool/t/images/XMP3.xmp | grep -i "HomePage"
# Expected: ProgrammerHomePage: https://exiftool.org/
```

**Key constraint**: Must match ExifTool's XMP.pm:4136-4139 attribute priority order exactly.

## ExifTool Reference

### Core Logic (XMP.pm:4136-4143)

```perl
# if element value is empty, take value from RDF 'value' or 'resource' attribute
# (preferentially) or 'about' attribute (if no 'value' or 'resource')
if ($val eq '' and ($attrs =~ /\brdf:(?:value|resource)=(['"])(.*?)\1/ or
                    $attrs =~ /\brdf:about=(['"])(.*?)\1/))
{
    $val = $2;
    $wasEmpty = 1;
}
```

### Priority Order
1. `rdf:value` - explicit RDF value attribute (highest)
2. `rdf:resource` - URI reference attribute
3. `rdf:about` - document subject identifier (fallback)

### Tags with Resource => 1

Some XMP tags always use `rdf:resource` for their values (XMP2.pl:1420-1470):
- `cc:license` - Creative Commons license URL
- `cc:attributionURL` - Attribution URL
- `cc:morePermissions` - Additional permissions URL
- `cc:permits`, `cc:requires`, `cc:prohibits` - License properties (List + Resource)
- `rdfs:seeAlso` - External reference (MWG.pm:445)

### Recognized RDF Attributes (XMP.pm:275-282)

```perl
our %recognizedAttrs = (
    'rdf:about' => [ 'Image::ExifTool::XMP::rdf', 'about', 'About' ],
    'x:xmptk'   => [ 'Image::ExifTool::XMP::x',   'xmptk', 'XMPToolkit' ],
    'rdf:parseType' => 1,
    'rdf:nodeID' => 1,
    ...
);
```

## Test Cases

### XMP3.xmp - rdf:resource for URI value

```xml
<rdf:Description rdf:nodeID="abc" ph:fullName="Phil Harvey">
  <ph:homePage rdf:resource="https://exiftool.org/"/>
</rdf:Description>
```

ExifTool output: `XMP:ProgrammerHomePage: https://exiftool.org/`

### XMP5.xmp - rdfs:seeAlso with rdf:resource

```xml
<rdfs:seeAlso rdf:resource='plus:Licensee'/>
```

ExifTool output: `XMP:RegionSeeAlso: plus:Licensee`

### XMP5.xmp - rdf:type for structure typing

```xml
<myXMPns:BTestTag rdf:parseType='Resource'>
  <rdf:type rdf:resource='myXMPns:SomeFunnyType'/>
  ...
</myXMPns:BTestTag>
```

Note: ExifTool skips empty `rdf:type` values (XMP.pm:4152-4153).

## Current Behavior

The XMP processor in `src/xmp/processor.rs` only extracts:
- Text content between element tags
- `xml:lang` attribute for language alternatives
- `xmlns:*` namespace declarations

It does **not** check for `rdf:resource`, `rdf:value`, or `rdf:about` attributes.

## Implementation Plan

### Phase 1: Core rdf:resource Support (P10a)

**Scope**: Extract values from `rdf:resource`/`rdf:value`/`rdf:about` when element text is empty.

#### Task 1: Add attribute extraction to ElementContext

**File**: `src/xmp/processor.rs`

**Changes**:
1. Add `rdf_resource: Option<String>` field to `ElementContext` struct
2. In `process_start_element()`, extract the attribute value with priority:
   - First check for `rdf:value`
   - Then `rdf:resource`
   - Finally `rdf:about`

```rust
// In process_start_element(), after existing attribute processing:
let mut rdf_resource_value = None;
for attr in element.attributes() {
    let attr = attr?;
    let key = std::str::from_utf8(attr.key.as_ref())?;

    // Check for rdf:value first (highest priority)
    if key == "rdf:value" && rdf_resource_value.is_none() {
        rdf_resource_value = Some(std::str::from_utf8(&attr.value)?.to_string());
    }
    // Then rdf:resource
    else if key == "rdf:resource" && rdf_resource_value.is_none() {
        rdf_resource_value = Some(std::str::from_utf8(&attr.value)?.to_string());
    }
    // Finally rdf:about as fallback
    else if key == "rdf:about" && rdf_resource_value.is_none() {
        rdf_resource_value = Some(std::str::from_utf8(&attr.value)?.to_string());
    }
}
```

#### Task 2: Handle empty elements with resource attributes

**File**: `src/xmp/processor.rs`

**Changes**:
1. On `Event::End`, check if:
   - Element has no text content captured yet
   - Current `ElementContext` has `rdf_resource` value
2. If both true, use `rdf_resource` as the element value

#### Task 3: Add integration test

**File**: `tests/xmp_rdf_resource_test.rs`

```rust
#[test]
fn test_rdf_resource_extraction() {
    let xmp = r#"<?xpacket begin='...'?>
<x:xmpmeta xmlns:x='adobe:ns:meta/'>
<rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'
         xmlns:ph='https://exiftool.org/ph/1.0/'>
 <rdf:Description>
  <ph:homePage rdf:resource="https://exiftool.org/"/>
 </rdf:Description>
</rdf:RDF>
</x:xmpmeta>"#;

    let mut processor = XmpProcessor::new();
    let tags = processor.process_xmp_data_individual(xmp.as_bytes()).unwrap();

    let homepage = tags.iter().find(|t| t.name == "HomePage").unwrap();
    assert_eq!(homepage.value.as_str(), Some("https://exiftool.org/"));
}
```

**Success criteria**:
- [x] `cargo t test_rdf_resource_extraction` passes
- [x] `cargo run -- third-party/exiftool/t/images/XMP5.xmp | grep SeeAlso` shows value

### Implementation Complete (2024-12-13)

**Files modified**:
- `src/xmp/processor.rs`:
  - Added `rdf_resource: Option<String>` and `has_text_content: bool` to `ElementContext`
  - Updated `process_start_element()` to extract `rdf:value`, `rdf:resource`, `rdf:about` with priority
  - Added `Event::Empty` handling for self-closing elements
  - Added `process_rdf_resource_value()` method
  - Added unit test `test_rdf_resource_extraction`

**Test results**:
- `cargo t test_rdf_resource_extraction` - PASS
- `cargo run --bin compare-with-exiftool t/images/XMP5.xmp` - 7/7 tags match

### Phase 2: rdf:about as XMP:About Tag (P10b) - Lower Priority

**Scope**: Extract `rdf:about` from `rdf:Description` as its own tag (not just as fallback).

ExifTool output for XMP3.xmp:
```
XMP:About: http://www.w3.org/TR/rdf-syntax-grammar
```

This is a separate tag extraction, not the fallback logic.

### Phase 3: rdf:nodeID Blank Node References (P10c) - Future

**Scope**: Handle blank node ID references for property sharing.

This is complex and marked as "not strictly XMP" in ExifTool's test file. Defer until needed.

**What it does**: When multiple elements reference the same `rdf:nodeID`, they share properties. ExifTool flattens these into the referencing element's namespace.

Example:
```xml
<ph:supervisor rdf:nodeID="abc"/>
<ph:programmer rdf:nodeID="abc"/>
<rdf:Description rdf:nodeID="abc" ph:fullName="Phil Harvey">
  <ph:homePage rdf:resource="https://exiftool.org/"/>
</rdf:Description>
```

Results in:
- `SupervisorFullName: Phil Harvey`
- `SupervisorHomePage: https://exiftool.org/`
- `ProgrammerFullName: Phil Harvey`
- `ProgrammerHomePage: https://exiftool.org/`

## Architecture Notes

### Current XMP Processing Flow

```
parse_xmp_xml()
  ├── Event::Start → process_start_element()
  │   └── Pushes ElementContext to stack
  ├── Event::Text → process_text_content()
  │   └── Uses element stack to build tag path
  └── Event::End → pops element stack
```

### Required Changes

The key insight is that `rdf:resource` values need to be captured in `process_start_element()` but emitted in `Event::End` (or a new `Event::Empty` handler).

**Option A**: Store in ElementContext, emit on End
- Pro: Simple, matches current architecture
- Con: Need to track "has text" separately

**Option B**: Handle in Start for empty elements
- Pro: More immediate
- Con: Can't distinguish empty-with-resource from empty-without

Recommend **Option A** as it matches ExifTool's approach of checking value at the end of element processing.

## Verification Commands

```bash
# Build and run tests
cargo t xmp_rdf

# Compare with ExifTool
cargo run --bin compare-with-exiftool third-party/exiftool/t/images/XMP3.xmp XMP:
cargo run --bin compare-with-exiftool third-party/exiftool/t/images/XMP5.xmp XMP:

# Check specific tags
cargo run -- third-party/exiftool/t/images/XMP3.xmp | grep -E "HomePage|About"
cargo run -- third-party/exiftool/t/images/XMP5.xmp | grep -E "SeeAlso"
```

## Known Limitations

1. **rdf:nodeID** is deferred - this is complex blank node reference handling
2. **rdf:type** extraction may need special handling (ExifTool skips empty structure types)
3. **Tags with Resource => 1** require codegen support for proper semantics (writing back)

## Related Files

- [XMP.pm](../../third-party/exiftool/lib/Image/ExifTool/XMP.pm) - Lines 4136-4143
- [XMPStruct.pl](../../third-party/exiftool/lib/Image/ExifTool/XMPStruct.pl) - Lines 519-526
- [XMP2.pl](../../third-party/exiftool/lib/Image/ExifTool/XMP2.pl) - Lines 1420-1470 (cc namespace)
- [processor.rs](../../src/xmp/processor.rs) - Current XMP implementation
