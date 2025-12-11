# P03f: XMP Tag Codegen Support

**Status**: NOT STARTED
**Priority**: High (critical tags blocked)
**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [CODEGEN.md](../CODEGEN.md)

---

## Problem Statement

XMP tags cannot be extracted because `TagKitStrategy` skips string keys:

```rust
// codegen/src/strategies/tag_kit.rs:315-323
let tag_id = if let Ok(id) = tag_key.parse::<u16>() {
    id
} else {
    return Ok(None);  // ALL XMP TAGS DISCARDED
};
```

**Result**: All 72 XMP namespace tables generate empty `HashMap::new()`.

**Required tags blocked**: License, Permits, RegionList, HierarchicalKeywords, PersonInImageWDetails, etc.

---

## Solution

Create `XmpTagStrategy` that extracts string-keyed XMP tag definitions and generates `HashMap<&'static str, XmpTagInfo>` tables.

**Key simplification**: No struct flattening. Return full XML tree as `TagValue::Object` for struct tags (RegionList, HierarchicalKeywords). The XMP processor already parses nested structures - we just need tag name mappings.

---

## Success Criteria

```bash
# 1. Generated tables are populated (not empty)
rg "license.*XmpTagInfo" src/generated/XMP2_pl/

# 2. CC tags work with PrintConv
cargo run -- cc-image.jpg | grep -E "Permits|Requires"
# Shows "Sharing, Derivative Works" not "cc:Sharing"

# 3. Struct tags return nested objects
cargo run -- face-tagged.jpg | grep RegionList
# Shows nested JSON structure

# 4. All tests pass
cargo t xmp && make precommit
```

---

## Design

### New Type: XmpTagInfo

```rust
// src/core/xmp_tag_info.rs

/// XMP tag definition extracted from ExifTool
#[derive(Debug, Clone)]
pub struct XmpTagInfo {
    /// Display name (e.g., "License", "RegionList")
    pub name: &'static str,
    /// Writable type: "string", "lang-alt", "integer", "real", "boolean"
    pub writable: Option<&'static str>,
    /// RDF container type
    pub list: Option<XmpListType>,
    /// True if value is a URI resource (not plain string)
    pub resource: bool,
    /// PrintConv lookup table
    pub print_conv: Option<&'static PrintConv>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmpListType {
    Bag,  // Unordered
    Seq,  // Ordered
    Alt,  // Language alternatives
}
```

### New Strategy: XmpTagStrategy

```rust
// codegen/src/strategies/xmp_tag.rs

pub struct XmpTagStrategy;

impl ExtractionStrategy for XmpTagStrategy {
    fn can_handle(&self, symbol: &FieldSymbol) -> bool {
        // XMP tables have NAMESPACE key (not numeric tag IDs)
        symbol.data.get("NAMESPACE").is_some()
    }

    fn process(&mut self, symbol: &FieldSymbol, ctx: &mut ExtractionContext) -> Result<()> {
        // Extract namespace name
        let namespace = symbol.data["NAMESPACE"].as_str()?;

        // Process each property in the table
        for (prop_name, prop_data) in symbol.data.as_object()? {
            if is_metadata_key(prop_name) { continue; }

            let tag_info = self.build_xmp_tag_entry(prop_name, prop_data, ctx)?;
            // ... accumulate entries
        }

        // Generate: HashMap<&'static str, XmpTagInfo>
        self.generate_xmp_tag_table(namespace, entries, ctx)
    }
}
```

**Detection**: XMP tables have `NAMESPACE` key. EXIF tables have numeric keys.

### Generated Output

```rust
// src/generated/XMP2_pl/cc_tags.rs (generated)

pub static XMP_CC_TAGS: LazyLock<HashMap<&'static str, XmpTagInfo>> = LazyLock::new(|| {
    HashMap::from([
        ("license", XmpTagInfo {
            name: "License",
            writable: None,
            list: None,
            resource: true,
            print_conv: None,
        }),
        ("permits", XmpTagInfo {
            name: "Permits",
            writable: None,
            list: Some(XmpListType::Bag),
            resource: true,
            print_conv: Some(&CC_PERMITS_PRINTCONV),
        }),
        ("requires", XmpTagInfo {
            name: "Requires",
            writable: None,
            list: Some(XmpListType::Bag),
            resource: true,
            print_conv: Some(&CC_REQUIRES_PRINTCONV),
        }),
        // ... all CC tags
    ])
});

pub static CC_PERMITS_PRINTCONV: PrintConv = PrintConv::HashMap(&[
    ("cc:Sharing", "Sharing"),
    ("cc:DerivativeWorks", "Derivative Works"),
    ("cc:Reproduction", "Reproduction"),
    ("cc:Distribution", "Distribution"),
]);
```

### Processor Integration

```rust
// src/xmp/processor.rs - replace hardcoded map_property_to_tag_name()

fn lookup_xmp_tag(&self, namespace: &str, property: &str) -> Option<&'static XmpTagInfo> {
    // Generated lookup tables indexed by namespace
    match namespace {
        "cc" => XMP_CC_TAGS.get(property),
        "dc" => XMP_DC_TAGS.get(property),
        "mwg-rs" => XMP_MWG_RS_TAGS.get(property),
        "mwg-kw" => XMP_MWG_KW_TAGS.get(property),
        // ... all 72 namespaces
        _ => None,
    }
}

fn process_xmp_property(&self, namespace: &str, property: &str, value: TagValue) -> TagEntry {
    let tag_info = self.lookup_xmp_tag(namespace, property);

    let name = tag_info.map(|t| t.name).unwrap_or(property);
    let value = self.apply_xmp_print_conv(tag_info, value);

    TagEntry::new(name, value)
}

fn apply_xmp_print_conv(&self, tag_info: Option<&XmpTagInfo>, value: TagValue) -> TagValue {
    let Some(info) = tag_info else { return value };
    let Some(print_conv) = info.print_conv else { return value };

    match &value {
        TagValue::String(s) => print_conv.lookup(s).map(TagValue::string).unwrap_or(value),
        TagValue::Array(items) => {
            // Apply PrintConv to each item in Bag/Seq
            TagValue::Array(items.iter().map(|v| {
                if let TagValue::String(s) = v {
                    print_conv.lookup(s).map(TagValue::string).unwrap_or_else(|| v.clone())
                } else {
                    v.clone()
                }
            }).collect())
        }
        _ => value,
    }
}
```

**Struct tags**: No special handling. RegionList returns `TagValue::Object` with nested structure from XML parser.

---

## Files to Modify

| File | Action | Lines |
|------|--------|-------|
| `src/core/xmp_tag_info.rs` | Create | ~40 |
| `src/core/mod.rs` | Edit | +2 |
| `codegen/src/strategies/xmp_tag.rs` | Create | ~250 |
| `codegen/src/strategies/mod.rs` | Edit | +5 |
| `src/xmp/processor.rs` | Edit | ~80 (replace hardcoded map) |
| `src/generated/XMP_pm/*.rs` | Generated | Populated |
| `src/generated/XMP2_pl/*.rs` | Generated | Populated |
| `src/generated/MWG_pm/*.rs` | Generated | Populated |

---

## Tasks

### Task 1: Create XmpTagInfo Type

**File**: `src/core/xmp_tag_info.rs`

Create the type as shown above. Export from `src/core/mod.rs`.

**Validation**: `cargo check`

### Task 2: Create XmpTagStrategy

**File**: `codegen/src/strategies/xmp_tag.rs`

1. Implement `can_handle()` - detect NAMESPACE key
2. Implement `process()` - extract properties, skip metadata keys (GROUPS, NOTES, etc.)
3. Implement `build_xmp_tag_entry()` - extract name, writable, list, resource
4. Implement `generate_xmp_tag_table()` - emit Rust HashMap code
5. Register in `mod.rs` before TagKitStrategy (so XMP tables don't fall through)

**Key metadata keys to skip**: `NAMESPACE`, `GROUPS`, `NOTES`, `WRITE_GROUP`, `TABLE_DESC`, `STRUCT_NAME`

**Validation**: `make codegen && rg "XmpTagInfo" src/generated/`

### Task 3: Extract PrintConv

Extend `XmpTagStrategy` to handle PrintConv:

1. Detect `PrintConv` hash in property definition
2. Generate static `PrintConv::HashMap` table
3. Reference from `XmpTagInfo.print_conv`

**Source patterns** (XMP2.pl:1437-1466):
```perl
permits => {
    List => 'Bag',
    Resource => 1,
    PrintConv => {
        'cc:Sharing' => 'Sharing',
        'cc:DerivativeWorks' => 'Derivative Works',
    },
},
```

**Validation**: `rg "CC_PERMITS_PRINTCONV" src/generated/`

### Task 4: Wire Processor to Generated Tables

**File**: `src/xmp/processor.rs`

1. Remove hardcoded `map_property_to_tag_name()` (lines 144-252)
2. Add `lookup_xmp_tag()` function using generated tables
3. Add `apply_xmp_print_conv()` for PrintConv application
4. Update `process_xmp_property()` to use new lookup

**Validation**:
```bash
cargo run -- third-party/exiftool/t/images/XMP.jpg | head -20
# Should show XMP tags with correct names
```

### Task 5: Test Critical Tags

Write/update tests for:

1. **CC namespace**: License, Permits (with PrintConv), Requires, Prohibits
2. **MWG-rs**: RegionList (returns nested object)
3. **MWG-kw**: HierarchicalKeywords (returns nested object)
4. **IPTC4xmpExt**: PersonInImageWDetails

**Test approach**: Compare against ExifTool JSON output structure.

**Validation**: `cargo t xmp`

### Task 6: Documentation

Update this file:
- Change status from NOT STARTED to COMPLETE
- Document the XmpTagStrategy pattern
- Note that struct tags return full XML tree (not flattened)

---

## Proof of Completion

- [ ] `rg "NAMESPACE" src/generated/ | wc -l` shows 72+ namespaces
- [ ] `cargo run -- cc-image.jpg | grep Permits` shows "Sharing" not "cc:Sharing"
- [ ] `cargo run -- face-image.jpg | grep RegionList` shows nested JSON
- [ ] `map_property_to_tag_name()` removed from processor.rs
- [ ] `cargo t` passes
- [ ] `make precommit` passes

---

## Risk Mitigation

**If PrintConv extraction is complex**: Ship without PrintConv first, add in follow-up. Tags will work but show raw values.

**If namespace detection conflicts with TagKitStrategy**: XmpTagStrategy must run BEFORE TagKitStrategy in strategy order. Both have `can_handle()` - first match wins.

**If generated code is too large**: Consider lazy loading or splitting by namespace prefix. Measure first.

---

## Related Work

- **P03c**: Composite Tags - similar codegen pattern
- **P03e**: DNGLensInfo - EXIF tag, independent
- **[XMP.md](../../third-party/exiftool/doc/modules/XMP.md)**: ExifTool XMP module reference
