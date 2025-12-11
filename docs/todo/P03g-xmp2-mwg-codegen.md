# P03g: XMP2.pl and MWG.pm Codegen Integration

**Status**: COMPLETE
**Completed**: 2025-12-11
**Priority**: High (blocks 15 required tags)
**Prerequisites**: P03f complete, familiarity with field_extractor.pl

---

## Part 1: Define Success

**Problem**: 15 required tags from "unknown" research are not being extracted because:
1. XMP2.pl tables (cc, MediaPro, iptcExt) are loaded lazily by ExifTool, not when XMP.pm is processed
2. MWG.pm is not in exiftool_modules.json

**Why it matters**: PhotoStructure needs these tags: License, AttributionName, AttributionURL, UseGuidelines, Jurisdiction, Permits, Requires, Prohibits, People, PersonInImageWDetails, PersonInImageName, RegionList, KeywordInfo, HierarchicalKeywords

**Solution**:
1. Modify field_extractor to explicitly require XMP2.pl when processing XMP.pm
2. Add MWG.pm to exiftool_modules.json
3. Update xmp_lookup.rs to route new namespaces

**Success test**:
```bash
# After codegen, check for cc namespace
ls src/generated/XMP_pm/cc_tags.rs
grep -c "XmpTagInfo" src/generated/XMP_pm/cc_tags.rs  # Should return > 0

# Check for MWG modules
ls src/generated/MWG_pm/regions_tags.rs
ls src/generated/MWG_pm/keywords_tags.rs
```

**Key constraint**: Must follow TRUST-EXIFTOOL.md - extract exactly what ExifTool defines

---

## Part 2: Share Your Expertise

### A. Root Cause Analysis

**Verified findings** (2025-12-11):

1. **XMP2.pl is lazily loaded**: ExifTool.pm:8900 requires XMP2.pl only when `GetTagTable` is called for an undefined table. Our field_extractor doesn't trigger this.

2. **Direct require works**: When XMP2.pl is explicitly required, these tables become available:
   - `%Image::ExifTool::XMP::cc` (18 entries) - Creative Commons
   - `%Image::ExifTool::XMP::MediaPro` (13 entries) - iView MediaPro
   - `%Image::ExifTool::XMP::iptcExt` (123 entries) - IPTC Extensions

3. **MWG.pm is independent**: MWG.pm defines its own package (`Image::ExifTool::MWG`) with:
   - `%Regions` (mwg-rs namespace) - RegionList
   - `%Keywords` (mwg-kw namespace) - KeywordInfo, HierarchicalKeywords
   - `%Collections` (mwg-coll namespace)
   - `%Composite` - MWG composite tags

### B. Verification Commands

```bash
# Verify XMP2.pl symbols appear after explicit require
cd $HOME/src/exif-oxide/codegen
perl -e '
use lib "scripts";
use lib "../third-party/exiftool/lib";
require Image::ExifTool;
require Image::ExifTool::XMP;
require "Image/ExifTool/XMP2.pl";
no strict "refs";
print "cc: " . scalar(keys %{"Image::ExifTool::XMP::cc"}) . "\n";
print "MediaPro: " . scalar(keys %{"Image::ExifTool::XMP::MediaPro"}) . "\n";
print "iptcExt: " . scalar(keys %{"Image::ExifTool::XMP::iptcExt"}) . "\n";
'
# Expected: cc: 18, MediaPro: 13, iptcExt: 123
```

### C. Namespace URI Mapping

The ns_uri.rs already has correct URIs (generated from XMP.pm %nsURI):
- `cc` → `http://creativecommons.org/ns#`
- `mediapro` → `http://ns.iview-multimedia.com/mediapro/1.0/`
- `Iptc4xmpExt` → `http://iptc.org/std/Iptc4xmpExt/2008-02-29/`
- `mwg-rs` → `http://www.metadataworkinggroup.com/schemas/regions/`
- `mwg-kw` → `http://www.metadataworkinggroup.com/schemas/keywords/`

### D. Learned the Hard Way

1. **Lazy loading is common in ExifTool** - Many .pl files are loaded on-demand via GetTagTable
2. **Package name matters** - XMP2.pl uses `package Image::ExifTool::XMP;` so its tables appear in XMP:: namespace
3. **MWG uses its own package** - `Image::ExifTool::MWG`, separate from XMP

---

## Part 3: Tasks

### Task 1: Modify field_extractor.pl for XMP2.pl

**Success**: After codegen, `src/generated/XMP_pm/cc_tags.rs` exists with 10+ entries

**Implementation**:

1. Edit `codegen/scripts/field_extractor.pl`
2. After line 68 (`require $package_name`), add:
```perl
# Explicitly load XMP2.pl when processing XMP module
# (ExifTool loads this lazily via GetTagTable, but we need it now)
if ($module_name eq 'XMP') {
    eval { require 'Image/ExifTool/XMP2.pl' };
    if ($@) {
        print STDERR "Note: Failed to load XMP2.pl: $@\n";
    }
}
```

3. Run `make codegen` and verify new files generated

**If architecture changed**: The key is ensuring XMP2.pl is loaded before symbol extraction. Look for where `require $package_name` happens and add explicit loading there.

---

### Task 2: Add MWG.pm to exiftool_modules.json

**Success**: `src/generated/MWG_pm/` directory exists with regions_tags.rs and keywords_tags.rs

**Implementation**:

1. Edit `config/exiftool_modules.json`
2. Add to "format" array:
```json
"lib/Image/ExifTool/MWG.pm"
```

3. Run `make codegen`

**If architecture changed**: Check how modules are loaded in codegen/src/main.rs. The MWG module should be processed like any other module.

---

### Task 3: Update xmp_lookup.rs for New Namespaces

**Success**: `lookup_xmp_tag("cc", "license")` returns valid XmpTagInfo

**Implementation**:

1. Edit `src/xmp/xmp_lookup.rs`
2. Add imports for new generated tables (after codegen):
```rust
// In imports section
use crate::generated::XMP_pm::{
    cc_tags::XMP_CC_TAGS,
    media_pro_tags::XMP_MEDIA_PRO_TAGS,
    iptc_ext_tags::XMP_IPTC_EXT_TAGS,
    // ... existing imports
};

// If MWG tables are generated separately
use crate::generated::MWG_pm::{
    regions_tags::MWG_REGIONS_TAGS,
    keywords_tags::MWG_KEYWORDS_TAGS,
};
```

3. Add match arms in `lookup_xmp_tag()`:
```rust
"cc" => XMP_CC_TAGS.get(property),
"mediapro" => XMP_MEDIA_PRO_TAGS.get(property),
"Iptc4xmpExt" | "iptcExt" => XMP_IPTC_EXT_TAGS.get(property),
"mwg-rs" => MWG_REGIONS_TAGS.get(property),
"mwg-kw" => MWG_KEYWORDS_TAGS.get(property),
```

4. Add tests:
```rust
#[test]
fn test_lookup_cc_tags() {
    let license = lookup_xmp_tag("cc", "license");
    assert!(license.is_some());
    assert_eq!(license.unwrap().name, "License");
}

#[test]
fn test_lookup_mediapro_tags() {
    let people = lookup_xmp_tag("mediapro", "People");
    assert!(people.is_some());
}
```

**If architecture changed**: The pattern is the same - import generated table, add match arm, add test.

---

### Task 4: Test With Real Files

**Success**: XMP tags from cc/mediapro/iptcExt namespaces appear in output

**Implementation**:

1. Find or create test image with Creative Commons metadata
2. Run: `cargo run -- test-image.jpg | grep -i "XMP.*license\|XMP.*People"`
3. Compare with: `exiftool -G -j test-image.jpg | grep -i license`

**Verification commands**:
```bash
# After implementation
cargo t test_lookup_cc_tags
cargo t test_lookup_mediapro_tags
make codegen fmt lint t
```

---

### Task 5: Update Documentation

**Success**: P03f and P03-implementation-backlog.md updated to reflect completion

**Implementation**:

1. Update P03f-xmp-namespace-tags.md "Known Limitations" section
2. Update P03-implementation-backlog.md summary table
3. Mark this TPP as COMPLETE

---

## Proof of Completion Checklist

- [ ] `src/generated/XMP_pm/cc_tags.rs` exists with License, AttributionName, etc.
- [ ] `src/generated/XMP_pm/media_pro_tags.rs` exists with People
- [ ] `src/generated/XMP_pm/iptc_ext_tags.rs` exists with PersonInImageWDetails
- [ ] `src/generated/MWG_pm/` exists with regions_tags.rs and keywords_tags.rs
- [ ] `lookup_xmp_tag("cc", "license")` returns valid XmpTagInfo
- [ ] `make codegen fmt lint t` passes
- [ ] Real file test shows CC tags extracted

---

## Emergency Recovery

```bash
# If codegen fails
git checkout HEAD -- codegen/scripts/field_extractor.pl
git checkout HEAD -- config/exiftool_modules.json
git checkout HEAD -- src/xmp/xmp_lookup.rs

# Clean and rebuild
make clean && make codegen
```

---

## Tags Unblocked by This Work

| Tag | Source | Complexity |
|-----|--------|------------|
| License | XMP2.pl cc | Low |
| AttributionName | XMP2.pl cc | Low |
| AttributionURL | XMP2.pl cc | Low |
| UseGuidelines | XMP2.pl cc | Low |
| Jurisdiction | XMP2.pl cc | Low |
| Permits | XMP2.pl cc | Medium (PrintConv) |
| Requires | XMP2.pl cc | Medium (PrintConv) |
| Prohibits | XMP2.pl cc | Medium (PrintConv) |
| People | XMP2.pl MediaPro | Low |
| PersonInImageWDetails | XMP2.pl iptcExt | High (nested struct) |
| PersonInImageName | XMP2.pl iptcExt | Low (flat tag) |
| RegionList | MWG.pm Regions | Very High (nested) |
| KeywordInfo | MWG.pm Keywords | High (nested) |
| HierarchicalKeywords | MWG.pm Keywords | Very High (recursive) |

**Note**: High complexity tags may require additional struct handling beyond basic codegen.
