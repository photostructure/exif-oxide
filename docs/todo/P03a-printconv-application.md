# P03a: PrintConv Application

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [PRINTCONV-VALUECONV-GUIDE.md](../guides/PRINTCONV-VALUECONV-GUIDE.md), P01 complete

---

## Part 1: Define Success

**Problem**: Tags like MeteringMode show "5" instead of "Multi-segment"

**Why it matters**: Users expect readable metadata, not cryptic numbers

**Solution**: Apply existing PrintConv lookup tables during tag output

**Success test**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep MeteringMode` shows matching values

**Key constraint**: Must match ExifTool output exactly per Trust ExifTool

---

## Part 2: Share Your Expertise

### A. The Pattern

PrintConv tables exist in `src/generated/` but aren't being applied to output:

```bash
# Verify tables exist
rg "MeteringMode" src/generated/ --type rust | head -5

# Find where PrintConv should be applied
rg "print_conv|PrintConv" src/ --type rust -l
```

### B. Tags Affected

| Tag                   | Raw | Expected            | ExifTool Source |
| --------------------- | --- | ------------------- | --------------- |
| EXIF:MeteringMode     | 5   | Multi-segment       | Exif.pm         |
| EXIF:ResolutionUnit   | 2   | inches              | Exif.pm         |
| EXIF:Orientation      | 1   | Horizontal (normal) | Exif.pm         |
| EXIF:Flash            | 0   | No Flash            | Exif.pm         |
| EXIF:YCbCrPositioning | 1   | Centered            | Exif.pm         |
| EXIF:ExposureProgram  | 2   | Program AE          | Exif.pm         |
| EXIF:GPSLatitudeRef   | N   | North               | GPS.pm          |
| EXIF:GPSLongitudeRef  | W   | West                | GPS.pm          |
| EXIF:GPSAltitudeRef   | 0   | Above Sea Level     | GPS.pm          |

### C. Learned the Hard Way

1. **Don't edit generated code** - if PrintConv tables are wrong, fix `codegen/src/` instead
2. **Check both paths** - JSON output and CLI output may use different pipelines
3. **GPS refs are NOT raw values** - "N"/"S" vs "North"/"South" is a PrintConv transformation

---

## Part 3: Tasks

### Task 1: Locate PrintConv Application Point

**Success**: Identify the exact file/function where PrintConv should be applied

**Implementation**:

```bash
rg "TagEntry|tag.*output" src/ --type rust -l
rg "Serialize|serde" src/ --type rust -l | xargs rg "TagEntry"
```

**If architecture changed**: Search `rg "format|display" src/ --type rust` for any output formatting system

---

### Task 2: Fix MeteringMode as Proof-of-Concept

**Success**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep MeteringMode` shows no difference

**Implementation**:

1. Find generated PrintConv: `rg "MeteringMode" src/generated/ --type rust -A5`
2. Wire PrintConv into output path identified in Task 1
3. Verify with comparison tool

**If architecture changed**: Goal unchanged (5 → "Multi-segment"), find new output path

---

### Task 3: Apply Pattern to All 9 Tags

**Success**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep -E "(MeteringMode|ResolutionUnit|Orientation|Flash|YCbCrPositioning|ExposureProgram|GPS.*Ref)"` shows no differences

**Implementation**: Apply Task 2 pattern to each tag in the table above

**If architecture changed**: Same pattern applies - find PrintConv, wire to output

**Proof of completion**:

- [ ] Test passes: comparison tool shows 0 differences for these 9 tags
- [ ] Integration shown: `rg "print_conv" src/` finds usage in output path
- [ ] `make precommit` passes

---

## Emergency Recovery

```bash
git diff HEAD~ > my_changes.patch
git checkout HEAD -- src/
make precommit
```
