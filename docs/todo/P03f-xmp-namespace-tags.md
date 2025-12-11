# P03f: XMP Tag Codegen Support

**Status**: COMPLETE (Tasks 1-5 complete)
**Priority**: High (critical tags blocked)
**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [CODEGEN.md](../CODEGEN.md)

---

## Progress Summary

| Task                         | Status      | Notes                                        |
| ---------------------------- | ----------- | -------------------------------------------- |
| Task 1: XmpTagInfo type      | ✅ COMPLETE | `src/core/xmp_tag_info.rs` created           |
| Task 2: XmpTagStrategy       | ✅ COMPLETE | `codegen/src/strategies/xmp_tag.rs` created  |
| Task 3: PrintConv extraction | ✅ COMPLETE | Inlined in XmpTagInfo (not separate statics) |
| Task 4: Wire processor       | ✅ COMPLETE | `src/xmp/xmp_lookup.rs` + processor wiring   |
| Task 5: Test critical tags   | ✅ COMPLETE | 22 XMP tests pass, 402 unit tests pass       |
| Task 6: Documentation        | ✅ COMPLETE | This document updated                        |

**Codegen Results**:

- 40 XMP namespace tables now populated (was 0)
- 719 XmpTagInfo entries generated
- PrintConv tables generated for namespaces with lookup data
- `make codegen` succeeds, `make lint` passes, all tests pass

---

## What Was Built

### 1. XmpTagInfo Type (COMPLETE)

**File**: `src/core/xmp_tag_info.rs`

```rust
pub struct XmpTagInfo {
    pub name: &'static str,
    pub writable: Option<&'static str>,
    pub list: Option<XmpListType>,
    pub resource: bool,
    pub print_conv: Option<PrintConv>,
}

pub enum XmpListType { Bag, Seq, Alt }
```

Exported from `src/core/mod.rs`.

### 2. XmpTagStrategy (COMPLETE)

**File**: `codegen/src/strategies/xmp_tag.rs`

- Detects XMP tables by `NAMESPACE` key (vs numeric keys for EXIF)
- Extracts: name, writable, list type, resource flag, PrintConv
- Generates `HashMap<&'static str, XmpTagInfo>` tables
- Registered in `mod.rs` BEFORE TagKitStrategy

**Generated constant naming**: `XMP_{namespace}_TAGS` (e.g., `XMP_DC_TAGS`, `XMP_TIFF_TAGS`)

### 3. PrintConv Extraction (COMPLETE)

PrintConv is **inlined** in XmpTagInfo, not as separate static constants. This is required because `HashMap::from()` with `.to_string()` cannot be called in static initializers.

**Example generated code** (`src/generated/XMP_pm/tiff_tags.rs`):

```rust
pub static XMP_TIFF_TAGS: LazyLock<HashMap<&'static str, XmpTagInfo>> = LazyLock::new(|| {
    HashMap::from([
        ("Compression", XmpTagInfo {
            name: "Compression",
            writable: Some("integer"),
            list: None,
            resource: false,
            print_conv: Some(PrintConv::Simple(std::collections::HashMap::from([
                ("1".to_string(), "Uncompressed"),
                ("2".to_string(), "CCITT 1D"),
                // ... 50+ compression types
            ]))),
        }),
        // ...
    ])
});
```

---

## Completed Implementation

### Task 4: Wire Processor to Generated Tables (COMPLETE)

**New file**: `src/xmp/xmp_lookup.rs`

Created a dedicated lookup module that:
- Imports all 30+ generated XMP namespace tables
- Provides `lookup_xmp_tag(namespace, property)` for tag info lookup
- Provides `get_xmp_tag_name(namespace, property)` for canonical name lookup
- 7 unit tests covering dc, tiff, xmp, and IPTC lookups

**Changes to `src/xmp/processor.rs`**:

1. **`flatten_xmp_structure()`** now:
   - Looks up tag info from generated tables via `lookup_xmp_tag()`
   - Uses tag info name when available, falls back to `map_property_to_tag_name()`
   - Applies PrintConv via new `apply_xmp_print_conv()` function

2. **`map_property_to_tag_name()`** simplified:
   - Special cases for namespaces not in generated tables (mwg-rs, plus, cc)
   - Delegates to `get_xmp_tag_name()` for generated table lookup
   - Fallback capitalizes first letter

3. **New `apply_xmp_print_conv()`**:
   - Handles `PrintConv::Simple` lookups for String, numeric, and Array values
   - Returns value unchanged for unsupported PrintConv types

### Task 5: Test Critical Tags (COMPLETE)

**Test results**:
- 22 XMP-specific tests pass
- 402 total unit tests pass
- 8 integration tests pass
- All lint checks pass

**Test coverage includes**:
- `test_lookup_dc_tags` - verifies dc namespace lookup (title → Title)
- `test_lookup_tiff_tags` - verifies tiff namespace with PrintConv (Orientation)
- `test_lookup_xmp_tags` - verifies xmp namespace (Rating, CreateDate)
- `test_required_xmp_tags_coverage` - comprehensive XMP extraction test

### Task 6: Documentation (COMPLETE)

This document updated to reflect completed status

---

## Known Limitations / Future Work

### 1. XMP2.pl Not Processed

The Creative Commons (`cc`) namespace is defined in `XMP2.pl`, not `XMP.pm`. The field_extractor only processes `.pm` files by default.

**Impact**: CC tags (License, Permits, Requires, Prohibits) are NOT in generated tables.

**Fix**: Either:

- Extend field_extractor to process `.pl` files
- Or manually add XMP2.pl to the module list in codegen

**Workaround**: The hardcoded `map_property_to_tag_name()` already handles CC tags. Keep those mappings as fallback until XMP2.pl is processed.

### 2. MWG.pm Not Processed

MWG (Metadata Working Group) namespaces (`mwg-rs`, `mwg-kw`) are in `MWG.pm`.

**Check**: Run `ls src/generated/MWG_pm/` to see if MWG tables exist.

### 3. Namespace Prefix Mapping

The namespace prefixes in XMP XML may differ from ExifTool's internal names. The processor's `uri2ns` tables handle this, but the lookup function needs to use the **normalized** namespace prefix.

**Example**: XML might use `xmlns:photoshop="..."` but lookup needs `"photoshop"` key.

---

## Verification Commands

```bash
# Check generated tables exist and have content
grep -c "XmpTagInfo {" src/generated/XMP_pm/*.rs | grep -v ":0"
# Should show ~40 files with non-zero counts

# Check total tag count
grep -c "XmpTagInfo {" src/generated/XMP_pm/*.rs | awk -F: '{sum+=$2} END {print sum}'
# Currently: 719 tags

# Check PrintConv is present
grep "print_conv: Some" src/generated/XMP_pm/tiff_tags.rs | head -3
# Should show entries with PrintConv::Simple(...)

# Verify codegen still works
make codegen && cargo check --lib
```

---

## Files Summary

| File                                | Status        | Description                                |
| ----------------------------------- | ------------- | ------------------------------------------ |
| `src/core/xmp_tag_info.rs`          | ✅ Created    | XmpTagInfo, XmpListType types              |
| `src/core/mod.rs`                   | ✅ Modified   | Exports xmp_tag_info module                |
| `codegen/src/strategies/xmp_tag.rs` | ✅ Created    | XmpTagStrategy implementation              |
| `codegen/src/strategies/mod.rs`     | ✅ Modified   | Registers XmpTagStrategy                   |
| `src/generated/XMP_pm/*.rs`         | ✅ Generated  | 40 namespace tables, 719 tags              |
| `src/xmp/xmp_lookup.rs`             | ✅ Created    | Lookup functions for generated tables      |
| `src/xmp/mod.rs`                    | ✅ Modified   | Exports xmp_lookup module                  |
| `src/xmp/processor.rs`              | ✅ Modified   | Wired to generated tables + PrintConv      |

---

## Related Work

- **P03c**: Composite Tags - similar codegen pattern
- **P03e**: DNGLensInfo - EXIF tag, independent
- **[XMP.md](../../third-party/exiftool/doc/modules/XMP.md)**: ExifTool XMP module reference
