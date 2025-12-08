# P02: Required Tags Audit

**Prerequisites**:

- Read [GETTING-STARTED.md](../GETTING-STARTED.md) "Current Project State" section
- P01 (Fix the Build) must be complete - `make precommit` passing

---

## Part 1: Define Success

**Problem**: We don't know which of the 124 required tags are actually producing correct values vs broken vs missing.

**Why it matters**: Can't prioritize implementation work without knowing the real gaps. The `supported_tags.json` says what we *claim* to support, but testing reveals what *actually* works.

**Solution**: Cross-reference required tags against actual ExifTool output on test images, produce prioritized backlog with gap analysis.

**Success test**:

```bash
# Deliverables exist and are populated
test -f docs/analysis/required-tags-gap-analysis.json && \
test -f docs/todo/P03-implementation-backlog.md && \
echo "SUCCESS: All deliverables created"
```

**Key constraint**: Use existing tooling (`compare-with-exiftool`, existing analysis JSONs) - don't reinvent scripts.

---

## Part 2: Share Your Expertise

### A. Key Finding: Coverage is Already High

Before diving in, know that a simple cross-reference shows **99.2% coverage**:

```bash
# Run this to see current state
python3 -c "
import json
with open('docs/required-tags.json') as f:
    required = set(json.load(f))
with open('config/supported_tags.json') as f:
    supported = set(tag.split(':')[1] if ':' in tag else tag for tag in json.load(f))
covered = required & supported
print(f'Required: {len(required)}, Covered: {len(covered)}, Missing: {len(required - covered)}')
print('Missing:', sorted(required - supported))
"
# Output: Required: 124, Covered: 123, Missing: 1
# Missing: ['FileAccessDate']
```

**However**: "supported" doesn't mean "working correctly". The real work is validating output matches ExifTool.

### B. Existing Analysis (Pre-Generated)

Analysis already exists at `docs/analysis/expressions/required-expressions-analysis.json`:

| Type      | Unique Expressions | Total Usage |
| --------- | ------------------ | ----------- |
| ValueConv | 41                 | 55          |
| PrintConv | 25                 | 71          |
| Condition | 1                  | 1           |

**Only 67 unique expressions** needed for all required tags (124 base, 178 with transitive composite dependencies).

### C. Required Tags by Group (Corrected)

From cross-referencing `docs/required-tags.json` with `docs/tag-metadata.json`:

| Group        | Count | Status                                    |
| ------------ | ----- | ----------------------------------------- |
| XMP          | 49    | Mostly working (XMP parser exists)        |
| Composite    | 37    | Partial (infrastructure exists)           |
| MakerNotes   | 37    | Partial (Canon/Nikon/Sony exist)          |
| EXIF         | 36    | Mostly working                            |
| UNKNOWN      | 20    | Not in tag-metadata.json - research needed |
| QuickTime    | 18    | **NOT IMPLEMENTED** (blocked: Milestone 18) |
| APP          | 16    | Partial                                   |
| File         | 14    | Working (Milestone File-Meta complete)    |
| IPTC         | 4     | Partial                                   |
| RIFF         | 4     | **NOT IMPLEMENTED** (blocked: Milestone 18) |
| PanasonicRaw | 3     | Partial                                   |
| ExifTool     | 1     | FileAccessDate - needs implementation     |

**Critical blocker**: QuickTime + RIFF = 22 tags require Milestone 18 (Video Format Support).

### D. Key Scripts and Files

```bash
# Existing analysis (already generated - review, don't regenerate)
cat docs/analysis/expressions/required-expressions-analysis.json | jq '.summary'

# Compare exif-oxide output with ExifTool (for individual files)
cargo run --bin compare-with-exiftool image.jpg

# Check composite tag dependencies
jq '.tags | keys | length' docs/analysis/expressions/composite-dependencies.json
# Output: 54 composite tags with dependencies

# List test images available
ls /home/mrm/src/test-images/Canon/*.JPG | head -5
ls third-party/exiftool/t/images/*.jpg | head -10
```

### E. What "Working" Means

A tag is "working" if:

1. It appears in exif-oxide JSON output
2. The value matches ExifTool's output (after normalization)
3. PrintConv produces human-readable format (not raw numbers)

The `compare-with-exiftool` tool handles all normalization automatically.

### F. Learned the Hard Way

1. **Don't regenerate the analysis JSON** - it already exists and is comprehensive. Just review it.

2. **supported_tags.json lies** - it lists what we *claim* to support, not what actually works. The real test is comparing against ExifTool on real images.

3. **20 required tags have no metadata** - tags like `AttributionName`, `ImageDataHash`, `HierarchicalKeywords` aren't in `tag-metadata.json`. These need research into which ExifTool module provides them.

4. **QuickTime/RIFF are blocked** - don't waste time trying to fix these 22 tags. They require Milestone 18 (Video Format Support) first.

---

## Part 3: Tasks

### Task 1: Verify Build is Fixed

**Success**: `make precommit` passes completely

**Implementation**:

```bash
cd /home/mrm/src/exif-oxide
make precommit  # Must pass before proceeding
```

**If it fails**: Complete P01 first.

---

### Task 2: Review Existing Analysis

**Success**: Understand expression complexity breakdown

**Implementation** (analysis already exists - just review):

```bash
cd /home/mrm/src/exif-oxide

# Review pre-generated analysis
jq '.summary' docs/analysis/expressions/required-expressions-analysis.json

# See expression patterns
jq '.expressions.ValueConv.patterns | to_entries | sort_by(-.value[1]) | .[0:5]' \
  docs/analysis/expressions/required-expressions-analysis.json

# Check composite dependencies
jq '.tags | keys | length' docs/analysis/expressions/composite-dependencies.json
```

**Proof of completion**: Can explain what the top 5 expression patterns are.

---

### Task 3: Test Against Real Images

**Success**: `docs/analysis/required-tags-gap-analysis.json` exists with format-specific results

**Implementation**:

1. Create gap-matrix script (doesn't exist yet):

```bash
# scripts/generate-gap-matrix.py - NEW SCRIPT NEEDED
# Inputs: test images across formats (JPEG, TIFF, CR2, NEF, ARW)
# Process: Run compare-with-exiftool, filter to required tags
# Output: JSON matrix of tag × format → status
```

2. Gather representative test images:

```bash
# ExifTool test images (diverse formats)
ls third-party/exiftool/t/images/*.jpg

# Manufacturer-specific
ls /home/mrm/src/test-images/Canon/*.CR2 | head -3
ls /home/mrm/src/test-images/Nikon/*.NEF | head -3
```

3. Run gap analysis (once script exists):

```bash
python3 scripts/generate-gap-matrix.py > docs/analysis/required-tags-gap-analysis.json
```

**If architecture changed**: The `compare-with-exiftool` tool uses `src/compat/` module. If that changes:
```bash
rg "analyze.*differences|normalize.*comparison" src/
```

---

### Task 4: Categorize Gaps

**Success**: Gaps organized by effort level in `docs/todo/P03-implementation-backlog.md`

**Categories**:

| Category | Criteria | Example Tags |
|----------|----------|--------------|
| **Quick wins** | Tag exists but value format differs | Most EXIF tags |
| **Medium effort** | Need expression or PrintConv | Composite tags |
| **Large effort** | Need new format support | QuickTime, RIFF (22 tags) |
| **Blocked** | Need architectural changes | Milestone 18 dependencies |
| **Research needed** | Not in tag-metadata.json | 20 UNKNOWN tags |

**Implementation**:

```bash
# Create backlog from gap analysis
# Group by: effort level, then by ExifTool module
cat > docs/todo/P03-implementation-backlog.md << 'EOF'
# P03: Implementation Backlog

## Quick Wins (estimated: X tags)
...

## Medium Effort (estimated: Y tags)
...

## Blocked on Milestone 18 (22 tags)
- All QuickTime tags (18)
- All RIFF tags (4)

## Research Needed (20 tags)
Tags not in tag-metadata.json - need to find source module...
EOF
```

---

### Task 5: Create Implementation TPPs

**Success**: One TPP per category/group of related work, following [TPP-GUIDE.md](../TPP-GUIDE.md)

**Structure**:

| TPP | Scope | Dependencies |
|-----|-------|--------------|
| P03a | Quick wins - value format fixes | None |
| P03b | Missing PrintConv expressions | Codegen knowledge |
| P03c | Composite tag fixes | P03a, P03b |
| P03d | UNKNOWN tag research | None (research only) |

**Note**: Do NOT create a TPP for QuickTime/RIFF - those are blocked on Milestone 18.

---

## Deliverables

1. `docs/analysis/required-tags-gap-analysis.json` - Machine-readable gap matrix
2. `docs/todo/P03-implementation-backlog.md` - Prioritized human-readable backlog
3. Individual TPPs for implementation work (P03a, P03b, etc.)

---

## Quality Checklist

- [ ] P01 complete (`make precommit` passes)
- [ ] Existing analysis reviewed (not regenerated unnecessarily)
- [ ] Gap matrix generated from real image tests
- [ ] Test images cover: JPEG, TIFF, CR2, NEF, ARW (skip MOV/MP4 - blocked)
- [ ] Each gap categorized by effort level
- [ ] QuickTime/RIFF (22 tags) explicitly marked as blocked on Milestone 18
- [ ] 20 UNKNOWN tags identified for research
- [ ] Implementation TPPs are actionable and follow TPP-GUIDE.md

---

## Emergency Recovery

```bash
# If something breaks during analysis
git status  # Check what changed
git diff docs/  # Review doc changes

# Revert if needed
git checkout docs/analysis/required-tags-gap-analysis.json
git checkout docs/todo/P03-implementation-backlog.md

# Validate before declaring success
make precommit
```
