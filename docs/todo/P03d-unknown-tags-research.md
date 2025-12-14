# P03d: Unknown Tags Research

**Status**: COMPLETE
**Completed**: 2025-12-08
**Prerequisites**: P01 complete, familiarity with ExifTool module structure

---

## Part 1: Define Success

**Problem**: ~~20~~ 19 required tags weren't in `tag-metadata.json` - we didn't know their ExifTool source

**Why it matters**: Can't implement tags without knowing their source module

**Solution**: Research each tag to identify its ExifTool source and document findings

**Success test**: `cat docs/analysis/unknown-tags-research.md | grep -c "Source:"` returns 17 (16 tags + 1 deferred)

**Key constraint**: Research only - no implementation in this TPP

**Outcome**:
- 16 tags fully researched with ExifTool source references
- 1 tag (ImageDataHash) deferred - computed value, not extracted metadata
- 2 tags removed from required list:
  - CameraModelName: Already implemented as Model tag (EXIF 0x110)
  - FileVersion: Not required by PhotoStructure

**Follow-up Work** (Completed 2025-12-11):
- Created extraction tests for all 16 researched tags
- Created test XMP files in `test-resources/`:
  - `cc-license-tags.xmp` - Creative Commons namespace tags
  - `xmp-history.xmp` - xmpMM:History tags
  - `iptc-person.xmp` - IPTC PersonInImage tags
- Test Results: 10 passing, 2 ignored (see test file for details)
- **Key Finding**: XMP parser extracts structured data but uses raw XML element names instead of ExifTool's flattened naming convention
  - Example: `Hierarchy: ["A-1", "B-1"]` instead of `HierarchicalKeywords1: A-1, B-1`
  - Example: `RegionList: [8, 8, 0, 0, "Region 1", "Face"]` instead of individual `RegionName`, `RegionType` fields
- **Implementation Gap Identified**: XMP struct flattening required for full ExifTool compatibility
  - Need to concatenate nested struct paths: `xmpMM:History/stEvt:when` → `HistoryWhen`
  - Need to handle `rdf:resource` attributes (affects License, AttributionURL, Permits, Requires, Prohibits, Jurisdiction)
  - Implementation notes documented in chat log at `docs/chats/unknown-tags.md`

---

## Part 2: Share Your Expertise

### A. Unknown Tags

```
AttributionName, AttributionURL, CameraModelName, DNGLensInfo, FileVersion,
HierarchicalKeywords, HistoryWhen, ImageDataHash, Jurisdiction, KeywordInfo,
License, People, Permits, PersonInImageName, PersonInImageWDetails, Prohibits,
RegionList, Requires, UseGuidelines
```

### B. Likely Sources (Hypothesis)

| Suspected Source             | Tags                                                                                                |
| ---------------------------- | --------------------------------------------------------------------------------------------------- |
| XMP-cc (Creative Commons)    | AttributionName, AttributionURL, License, Permits, Prohibits, Requires, UseGuidelines, Jurisdiction |
| XMP-mwg-rs (MWG Regions)     | RegionList, PersonInImageWDetails                                                                   |
| XMP-MP (Microsoft Photo)     | PersonInImageName                                                                                   |
| XMP-xmpMM (Media Management) | HistoryWhen                                                                                         |
| XMP-lr (Lightroom)           | HierarchicalKeywords, KeywordInfo                                                                   |
| XMP-crs (Camera Raw)         | CameraModelName                                                                                     |
| EXIF/DNG                     | DNGLensInfo                                                                                         |
| File module                  | ImageDataHash                                                                                       |

### C. Research Commands

```bash
rg "AttributionName" third-party/exiftool/lib/Image/ExifTool/ --type perl
ls third-party/exiftool/lib/Image/ExifTool/XMP*.pm
rg "cc:|creative.commons" third-party/exiftool/lib/Image/ExifTool/ --type perl -i
```

### D. Learned the Hard Way

1. **XMP namespaces are case-sensitive** - `cc:` vs `CC:` matters
2. **Tags may have multiple sources** - same tag name in different modules
3. **Some tags are synthetic** - calculated by ExifTool, not read from file
4. **Structured tags are complex** - RegionList contains nested structures

---

## Part 3: Tasks

### Task 1: Research Creative Commons Tags

**Tags**: AttributionName, AttributionURL, License, Permits, Prohibits, Requires, UseGuidelines, Jurisdiction

**Success**: Document ExifTool source file:line for each tag

**Implementation**:

```bash
rg -l "cc:" third-party/exiftool/lib/Image/ExifTool/
rg "AttributionName" third-party/exiftool/lib/Image/ExifTool/XMP.pm -B5 -A10
```

**If architecture changed**: ExifTool source locations are stable

---

### Task 2: Research Region Tags

**Tags**: RegionList, PersonInImageWDetails

**Success**: Document MWG region spec source

**Implementation**:

```bash
rg "RegionList|mwg-rs" third-party/exiftool/lib/Image/ExifTool/ --type perl
```

**If architecture changed**: MWG spec is external standard

---

### Task 3: Research Lightroom Tags

**Tags**: HierarchicalKeywords, KeywordInfo

**Success**: Document Lightroom XMP source

**Implementation**:

```bash
rg "HierarchicalKeywords|lr:" third-party/exiftool/lib/Image/ExifTool/ --type perl
```

**If architecture changed**: Lightroom namespace is stable

---

### Task 4: Research Remaining Tags

**Tags**: CameraModelName, DNGLensInfo, FileVersion, HistoryWhen, ImageDataHash, People, PersonInImageName

**Success**: Document source for each

**Implementation**:

```bash
for tag in CameraModelName DNGLensInfo FileVersion HistoryWhen ImageDataHash People PersonInImageName; do
  echo "=== $tag ===" && rg "$tag" third-party/exiftool/lib/Image/ExifTool/ --type perl | head -3
done
```

**If architecture changed**: ExifTool sources are stable

---

### Task 5: Create Research Document

**Success**: `docs/analysis/unknown-tags-research.md` exists with all 20 tags documented

**Implementation**: Create file with format:

```markdown
## AttributionName

- **Source**: XMP.pm:XXX
- **Namespace**: cc (Creative Commons)
- **ValueConv**: none
- **PrintConv**: none
- **Implementation complexity**: Low
```

**Proof of completion**:

- [x] All 16 tags have identified sources (2 removed, 1 deferred)
- [x] Each tag has file:line reference
- [x] `docs/analysis/unknown-tags-research.md` created
- [x] `docs/required-tags.json` updated (CameraModelName, FileVersion removed)
- [x] `docs/analysis/required-tags-gap-analysis.json` updated with confirmed sources
- [x] Extraction tests created in `tests/xmp_namespace_integration_test.rs`
- [x] Test resources created in `test-resources/` directory
- [x] 10 of 12 extraction tests passing (2 blocked on XMP struct flattening implementation)

---

## Part 4: Test Coverage Results (2025-12-11)

### Extraction Tests Created

All 16 researched tags now have extraction tests in `tests/xmp_namespace_integration_test.rs`:

| Tag | Test Status | Test Image | Notes |
|-----|-------------|------------|-------|
| DNGLensInfo | ✅ Pass | third-party/exiftool/t/images/DNG.dng | EXIF tag extraction working |
| PersonInImage | ✅ Pass | test-resources/iptc-person.xmp | IPTC namespace extraction working |
| AttributionName | ✅ Pass | third-party/exiftool/t/images/XMP3.xmp | CC namespace extraction working |
| License | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` support |
| AttributionURL | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` support |
| UseGuidelines | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` support |
| Permits | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` + Bag support |
| Requires | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` + Bag support |
| Prohibits | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` + Bag support |
| Jurisdiction | ⏸️ Ignored | test-resources/cc-license-tags.xmp | Blocked: needs `rdf:resource` support |
| RegionList | ✅ Pass | third-party/exiftool/t/images/XMP5.xmp | Extracted as array with element names |
| HierarchicalKeywords | ✅ Pass | third-party/exiftool/t/images/XMP5.xmp | Extracted as Hierarchy/Children arrays |
| KeywordInfo | ✅ Pass | Via lookup test | MWG namespace lookup working |
| HistoryWhen | ✅ Pass | test-resources/xmp-history.xmp | Extracted as History array |
| People | ⏸️ Ignored | third-party/exiftool/t/images/ExifTool.jpg | Blocked: mediapro namespace not extracted |
| PersonInImageWDetails | ✅ Pass | Via lookup test | IPTC namespace lookup working |

### Test Summary

- **10 tests passing**: Core XMP extraction working, tags present in generated code
- **2 tests ignored**:
  - CC tags with `rdf:resource` attributes (6 tags affected)
  - mediapro:People namespace extraction issue

### Key Architectural Finding

The XMP parser successfully extracts structured XMP data but uses **raw XML element names** rather than **ExifTool's flattened naming convention**:

**ExifTool Output:**
```
HierarchicalKeywords1: A-1, B-1, C-1
HierarchicalKeywords2: A-2, B-2
RegionName: Region 1
RegionType: Face
HistoryWhen: 2024-01-15T10:30:00+00:00
HistoryAction: created
```

**exif-oxide Current Output:**
```json
"XMP:Hierarchy": ["A-1", "B-1", "C-1"],
"XMP:Children": ["A-2", "B-2"],
"XMP:RegionList": [8, 8, 0, 0, "Region 1", "Face", 1, 2],
"XMP:History": ["created", "Test Application 1.0", "2024-01-15T10:30:00+00:00"]
```

### Next Steps

To achieve full ExifTool compatibility for these tags, the XMP processor needs:

1. **Struct path flattening**: Concatenate nested element paths with ucfirst
   - `xmpMM:History/stEvt:when` → `HistoryWhen`
   - `mwg-rs:Regions/RegionStruct/Name` → `RegionName`
   - `mwg-kw:Keywords/KeywordStruct/Keyword[1]` → `HierarchicalKeywords1`

2. **rdf:resource attribute extraction**: Extract URI values from resource attributes
   - Affects: License, AttributionURL, UseGuidelines, Jurisdiction, Permits, Requires, Prohibits

3. **Namespace investigation**: Determine why mediapro namespace isn't extracted from embedded XMP

Implementation approach documented in `docs/chats/unknown-tags.md` (lines 2600-2713).

---

## Emergency Recovery

```bash
# Research-only TPP - no code to revert
git checkout HEAD -- docs/analysis/
```
