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

---

## Emergency Recovery

```bash
# Research-only TPP - no code to revert
git checkout HEAD -- docs/analysis/
```
