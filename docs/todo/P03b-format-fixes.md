# P03b: Format Fixes

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), P01 complete

---

## Part 1: Define Success

**Problem**: Minor formatting differences - "jpeg" vs "jpg", "1181861" vs "118-1861"

**Why it matters**: String comparisons break when formats don't match exactly

**Solution**: Match ExifTool's exact output format for these specific tags

**Success test**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Apple.jpg 2>/dev/null | grep -E "FileTypeExtension|FileNumber|ShutterSpeedValue|GPSTimeStamp"` shows no differences

**Key constraint**: Copy ExifTool formatting exactly, don't invent "better" formats

---

## Part 2: Share Your Expertise

### A. Tags Affected

| Tag                    | exif-oxide | ExifTool    | Fix                           |
| ---------------------- | ---------- | ----------- | ----------------------------- |
| File:FileTypeExtension | jpeg       | jpg         | Use canonical short extension |
| MakerNotes:FileNumber  | 1181861    | 118-1861    | Apply PrintConv regex         |
| EXIF:ShutterSpeedValue | 4.3        | 1/20        | Use PrintExposureTime         |
| EXIF:GPSTimeStamp      | 17:17:58   | 17:17:58.65 | Preserve subseconds           |

### B. Investigation Commands

```bash
rg "FileTypeExtension|extension" src/file/ --type rust
rg "FileNumber" src/generated/ --type rust -A10
rg "PrintExposureTime" codegen-runtime/src/ --type rust
rg "GPSTimeStamp" src/ --type rust -A10
```

### C. Learned the Hard Way

1. **ExifTool uses short extensions** - jpg/tif/png, NOT jpeg/tiff (see `%fileTypeLookup` in ExifTool.pm)
2. **FileNumber PrintConv is a regex** - `$_=$val;s/(\d+)(\d{4})/$1-$2/;$_` inserts hyphen before last 4 digits
3. **ShutterSpeedValue is APEX** - needs conversion to fraction via PrintExposureTime
4. **GPS timestamp stores rationals** - third value contains fractional seconds

---

## Part 3: Tasks

### Task 1: Fix FileTypeExtension

**Success**: `exif-oxide image.jpg | jq '.FileTypeExtension'` returns "jpg" not "jpeg"

**Implementation**:

```bash
rg "jpeg|extension" src/file/ --type rust
```

Update to use ExifTool's canonical short extensions from `%fileTypeLookup`

**If architecture changed**: Search `rg "file.*type|extension" src/` for extension logic

---

### Task 2: Fix FileNumber PrintConv

**Success**: Canon FileNumber shows "118-1861" not "1181861"

**Implementation**:

1. Verify PrintConv exists: `rg "FileNumber" src/generated/ --type rust -B5 -A10`
2. May be same root cause as P03a - PrintConv not applied

**If architecture changed**: The PrintConv regex `s/(\d+)(\d{4})/$1-$2/` must be applied somewhere

---

### Task 3: Fix ShutterSpeedValue Format

**Success**: ShutterSpeedValue shows "1/20" not "4.3"

**Implementation**:

1. Check for PrintExposureTime: `rg "PrintExposureTime" codegen-runtime/src/ --type rust`
2. ExifTool logic (Exif.pm): `$val >= 1 ? sprintf("%.0f", $val) : sprintf("1/%.0f", 1/$val)`

**If architecture changed**: The conversion formula is constant - find where to apply it

---

### Task 4: Fix GPSTimeStamp Subseconds

**Success**: GPSTimeStamp shows "17:17:58.65" not "17:17:58"

**Implementation**:

```bash
rg "GPSTimeStamp" src/ --type rust -A10
```

GPS timestamp is three rationals (H, M, S). The third may have fractional component.

**If architecture changed**: Find GPS time parsing and preserve fractional seconds

**Proof of completion**:

- [ ] All 4 tags match ExifTool format
- [ ] `make precommit` passes

---

## Emergency Recovery

```bash
git diff HEAD~ > my_changes.patch
git checkout HEAD -- src/
make precommit
```
