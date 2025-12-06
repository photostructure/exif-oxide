# P02: Required Tags Audit

**Prerequisites**:
- Read [GETTING-STARTED.md](../GETTING-STARTED.md) "Current Project State" section
- P01 (Fix the Build) must be complete - `make precommit` passing

## Part 1: Define Success

**Problem**: We don't know which of the 125 required tags are working vs broken vs missing.

**Why it matters**: Can't prioritize implementation work without knowing the gaps.

**Solution**: Cross-reference required tags against current implementation, produce prioritized backlog.

**Success test**:
```bash
# Run analysis and verify against test images
python3 scripts/analyze-required-expressions.py > /tmp/analysis.json
cargo run --bin compare-with-exiftool test-image.jpg
```

**Key constraint**: Use existing tooling where possible - don't reinvent analysis scripts.

---

## Part 2: Share Your Expertise

### A. Existing Analysis

Analysis exists at `docs/analysis/expressions/required-expressions-analysis.json`:

| Type | Unique Expressions | Total Usage |
|------|-------------------|-------------|
| ValueConv | 41 | 55 |
| PrintConv | 25 | 71 |
| Condition | 1 | 1 |

**Only 67 unique expressions** needed for all 125 required tags.

### B. Required Tags by Group

From cross-referencing `docs/required-tags.json` with `docs/tag-metadata.json`:

| Group | Count | Likely Status |
|-------|-------|---------------|
| XMP | 49 | Mostly working (XMP parser exists) |
| Composite | 37 | Partial (infrastructure exists) |
| MakerNotes | 37 | Partial (Canon/Nikon/Sony exist) |
| EXIF | 36 | Mostly working |
| QuickTime | 18 | **NOT IMPLEMENTED** (video) |
| File | 14 | Working (Milestone File-Meta complete) |
| IPTC | 4 | Partial |
| RIFF | 4 | **NOT IMPLEMENTED** (video) |
| PanasonicRaw | 3 | Partial |

### C. Key Scripts

```bash
# Analyze expressions for required tags
python3 scripts/analyze-required-expressions.py

# Compare exif-oxide output with ExifTool
cargo run --bin compare-with-exiftool image.jpg

# Check composite tag dependencies
cat docs/analysis/expressions/composite-dependencies.json
```

### D. What "Working" Means

A tag is "working" if:
1. It appears in exif-oxide JSON output
2. The value matches ExifTool's output (after normalization)
3. PrintConv produces human-readable format (not raw numbers)

Use `compare-with-exiftool` to verify - it handles normalization.

---

## Part 3: Tasks

### Task 1: Verify Build is Fixed

**Success**: `make precommit` passes

**Implementation**:
```bash
make precommit  # Must pass before proceeding
```

### Task 2: Run Existing Analysis

**Success**: Updated analysis JSON with current state

**Implementation**:
```bash
cd /home/mrm/src/exif-oxide
python3 scripts/analyze-required-expressions.py > /tmp/analysis.json
jq '.summary' /tmp/analysis.json
```

### Task 3: Test Against Real Images

**Success**: Gap matrix showing which required tags work for which formats

**Implementation**:
1. Gather test images (JPEG, TIFF, CR2, NEF, ARW, MOV, MP4)
2. For each image:
   ```bash
   cargo run --bin compare-with-exiftool test-image.jpg > /tmp/diff.txt
   # Check which required tags are in diff
   ```
3. Create matrix: tag x format -> working/broken/missing

### Task 4: Categorize Gaps

**Success**: Prioritized backlog in `docs/todo/P03-implementation-backlog.md`

**Categories**:
1. **Quick wins**: Tags that are close to working (small fixes)
2. **Medium effort**: Need expression support or PrintConv
3. **Large effort**: Need new format support (QuickTime)
4. **Blocked**: Need architectural changes

### Task 5: Create Implementation TPPs

**Success**: One TPP per category/group of related work

**Structure**:
- P03a: Quick wins (list specific tags)
- P03b: Missing expressions (list specific expressions)
- P03c: QuickTime/video support (new format)
- etc.

---

## Deliverables

1. `docs/analysis/required-tags-gap-analysis.json` - Machine-readable gap data
2. `docs/todo/P03-implementation-backlog.md` - Prioritized human-readable backlog
3. Individual TPPs for implementation work

---

## Quality Checklist

- [ ] Analysis run on current codebase (after P01 complete)
- [ ] Test images cover all required format groups
- [ ] Each gap categorized by effort level
- [ ] QuickTime/video scope clearly defined
- [ ] Implementation TPPs are actionable and independent
