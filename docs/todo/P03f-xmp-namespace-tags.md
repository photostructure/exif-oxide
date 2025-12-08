# P03f: XMP Namespace Tags Implementation

**Status**: BLOCKED on XMP tag extraction infrastructure

**Prerequisites**:
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- P03d (Unknown Tags Research) complete
- XMP tag extraction working in codegen (see Blocker section)

**Research Reference**: [unknown-tags-research.md](../analysis/unknown-tags-research.md)

---

## Part 1: Define Success

**Problem**: 15 required XMP tags have been researched but cannot be implemented because XMP tag definitions aren't being extracted by codegen

**Why it matters**: Creative Commons licensing, face/region detection, hierarchical keywords, and person metadata are critical for photo management

**Solution**:
1. First: Fix codegen to extract XMP tag definitions from XMP.pm, XMP2.pl, MWG.pm
2. Then: Wire extracted tags through XMP processor

**Success test**:
```bash
# After XMP infrastructure fixed:
cargo run --bin compare-with-exiftool -- test-images/xmp/cc-licensed.jpg 2>/dev/null | grep -E "License|Attribution"
# Should show matching Creative Commons tags
```

**Key constraint**: XMP tags use string names (not numeric IDs) - codegen must handle this differently than EXIF tags

---

## Blocker: XMP Tag Extraction Gap

### Current State

**Namespace URIs exist** (`src/generated/XMP_pm/ns_uri.rs`):
```rust
("cc", "http://creativecommons.org/ns#"),
("mwg-rs", "http://www.metadataworkinggroup.com/schemas/regions/"),
("mwg-kw", "http://www.metadataworkinggroup.com/schemas/keywords/"),
("Iptc4xmpExt", "http://iptc.org/std/Iptc4xmpExt/2008-02-29/"),
("mediapro", "http://ns.iview-multimedia.com/mediapro/1.0/"),
```

**Tag definition tables are empty**:
```rust
// src/generated/XMP_pm/dc_tags.rs
pub static XMP_DC_TAGS: LazyLock<HashMap<u16, TagInfo>> = LazyLock::new(HashMap::new);
```

### Root Cause

`TagKitStrategy` is designed for EXIF-style numeric tag IDs. XMP tags use string property names within namespaces - they don't have numeric IDs.

### Required Infrastructure Changes

1. **New strategy or TagKitStrategy enhancement** to handle XMP tables:
   - XMP tags keyed by property name (string), not tag ID (u16)
   - Extract from `%Image::ExifTool::XMP::cc`, `%Image::ExifTool::XMP::MediaPro`, etc.

2. **Generated table format change**:
   ```rust
   // Instead of HashMap<u16, TagInfo>:
   pub static XMP_CC_TAGS: LazyLock<HashMap<&'static str, XmpTagInfo>> = ...
   ```

3. **XMP processor integration**: Wire tag definitions to `src/xmp/processor.rs`

### Investigation Commands

```bash
# Check what codegen does with XMP modules
RUST_LOG=trace make codegen 2>&1 | grep -i "XMP"

# See what strategies handle XMP
rg "XMP|xmp" codegen/src/strategies/ --type rust

# Check field_extractor output for XMP
./codegen/scripts/field_extractor.pl XMP 2>/dev/null | head -50
```

---

## Part 2: Share Your Expertise (For Future Implementation)

### A. Tag Categories by Namespace

| Namespace | Tags | Source | Complexity |
|-----------|------|--------|------------|
| XMP-cc | 8 | XMP2.pl:1420-1470 | Low-Medium |
| XMP-mwg-rs | 1 | MWG.pm:473-481 | Very High (struct) |
| XMP-mwg-kw | 2 | MWG.pm:499-523 | Very High (recursive) |
| XMP-Iptc4xmpExt | 2 | XMP2.pl:619-640 | Medium-High |
| XMP-xmpMM | 1 | XMP.pm:341 | Medium |
| XMP-mediapro | 1 | XMP2.pl:1523 | Low |

### B. Creative Commons Tags (8) - Easiest to Start

These are simple string/URI properties with optional PrintConv hash maps:

```perl
# XMP2.pl:1420-1470
%Image::ExifTool::XMP::cc = (
    license         => { Resource => 1 },           # Simple URI
    attributionName => { },                          # Simple string
    attributionURL  => { Resource => 1 },           # Simple URI
    useGuidelines   => { Resource => 1 },           # Simple URI
    jurisdiction    => { Resource => 1 },           # Simple URI
    permits => {                                     # Bag with PrintConv
        List => 'Bag',
        Resource => 1,
        PrintConv => {
            'cc:Sharing' => 'Sharing',
            'cc:DerivativeWorks' => 'Derivative Works',
            # ...
        },
    },
    requires => { ... },  # Similar to permits
    prohibits => { ... }, # Similar to permits
);
```

### C. Structured Tags (4) - Require XMP Struct Parsing

**RegionList** (MWG.pm:473-481):
- Bag of MWG RegionStruct
- Each region has: Area (x/y/w/h), Type, Name, Description
- Deeply nested - requires full XMP struct support

**HierarchicalKeywords** (MWG.pm:506-523):
- Recursive KeywordStruct (Keyword, Applied, Children)
- ExifTool unrolls to 6 levels to prevent infinite recursion
- Very complex - defer until basic XMP works

**PersonInImageWDetails** (XMP2.pl:619):
- Bag of PersonDetails structs
- Contains lang-alt fields (multi-language support)
- Medium complexity

### D. Implementation Order Recommendation

**Phase 1**: Fix codegen XMP extraction (prerequisite)

**Phase 2**: Simple string tags (9 tags)
- License, AttributionName, AttributionURL, UseGuidelines, Jurisdiction
- People, PersonInImageName, HistoryWhen

**Phase 3**: Tags with PrintConv (3 tags)
- Permits, Requires, Prohibits

**Phase 4**: Nested struct tags (4 tags) - may need separate TPP
- KeywordInfo, PersonInImageWDetails
- RegionList, HierarchicalKeywords

---

## Part 3: Tasks (After Blocker Resolved)

### Task 1: Verify XMP Tag Extraction Works

**Success**: Generated XMP tag tables are populated, not empty

**Implementation**:
```bash
# After codegen infrastructure fixed:
grep -v "HashMap::new()" src/generated/XMP_pm/cc_tags.rs
# Should show actual tag definitions
```

**If not working**: This TPP is still blocked - return to blocker investigation.

---

### Task 2: Implement Simple CC Tags (5)

**Tags**: License, AttributionName, AttributionURL, UseGuidelines, Jurisdiction

**Success**:
```bash
# With a CC-licensed image:
cargo run --bin compare-with-exiftool -- cc-image.jpg | grep -E "License|Attribution|Jurisdiction"
```

**Implementation**:
1. Verify tags extracted from XMP2.pl:1420-1470
2. Wire to XMP processor namespace handling
3. Handle `Resource => 1` (URI) vs plain string

---

### Task 3: Implement CC Tags with PrintConv (3)

**Tags**: Permits, Requires, Prohibits

**Success**:
```bash
cargo run -- cc-image.jpg | grep -E "Permits|Requires|Prohibits"
# Should show "Sharing, Derivative Works" not "cc:Sharing, cc:DerivativeWorks"
```

**Implementation**:
1. Extract PrintConv hash maps from XMP2.pl
2. Apply during output (URI → human-readable)
3. Handle Bag (list) type

---

### Task 4: Implement Remaining Simple Tags (4)

**Tags**: People, PersonInImageName, HistoryWhen

**Success**:
```bash
cargo run --bin compare-with-exiftool -- image.jpg | grep -E "People|PersonInImage|HistoryWhen"
```

**Implementation**:
- People: Simple Bag of strings (XMP2.pl:1523)
- PersonInImageName: Flattened from PersonInImageWDetails (XMP2.pl:634)
- HistoryWhen: DateTime within stEvt struct (XMP.pm:341)

---

### Task 5: Implement Struct Tags (Separate TPP Recommended)

**Tags**: KeywordInfo, PersonInImageWDetails, RegionList, HierarchicalKeywords

These require XMP struct parsing infrastructure and should likely be a separate TPP after basic XMP tags work.

---

## Proof of Completion

### Blocker Resolution
- [ ] Codegen extracts XMP tag definitions (not empty HashMaps)
- [ ] `rg "license|attributionName" src/generated/XMP_pm/` finds definitions

### Tag Implementation
- [ ] `cargo run -- cc-image.jpg | grep License` shows value
- [ ] `cargo run -- cc-image.jpg | grep Permits` shows "Sharing" not "cc:Sharing"
- [ ] `cargo run --bin compare-with-exiftool -- cc-image.jpg` shows XMP matches
- [ ] `cargo t` passes
- [ ] `make precommit` passes

---

## Emergency Recovery

```bash
# If codegen changes break things
git status
git diff codegen/
git checkout HEAD -- src/generated/
make codegen && cargo t
```

---

## Related Work

- **P03c**: Composite Tags - similar codegen challenges
- **P03e**: DNGLensInfo - EXIF tag, can proceed independently
- **Milestone ?**: XMP struct parsing infrastructure
