# P03c: Composite Tags

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [ARCHITECTURE.md](../ARCHITECTURE.md), [composite-dependencies.json](../analysis/expressions/composite-dependencies.json), P01 complete

---

## Part 1: Define Success

**Problem**: Composite tags like Aperture, ShutterSpeed, Megapixels are "supported" but not generated

**Why it matters**: These are the most commonly used metadata tags - without them, exif-oxide is incomplete

**Solution**: Implement composite tag evaluation that calculates derived values from extracted tags

**Success test**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep "Composite"` shows zero missing

**Key constraint**: Composite tags depend on other tags being extracted first - evaluation order matters

---

## Part 2: Share Your Expertise

### A. What Are Composite Tags?

Composite tags combine or transform other tags. ExifTool defines them in `lib/Image/ExifTool/Composite.pm`.

Example: `Composite:Aperture` = first non-empty of (FNumber, ApertureValue)

### B. Key Composite Tags

| Tag          | Dependencies                                  | Expression                              |
| ------------ | --------------------------------------------- | --------------------------------------- |
| Aperture     | FNumber, ApertureValue                        | `$val[0] \|\| $val[1]`                  |
| ShutterSpeed | ExposureTime, ShutterSpeedValue, BulbDuration | Ternary selection                       |
| Megapixels   | ImageSize                                     | `$d[0] * $d[1] / 1000000`               |
| ImageSize    | ImageWidth, ImageHeight                       | `"$val[0] $val[1]"`                     |
| GPSLatitude  | GPSLatitude, GPSLatitudeRef                   | `$val[1] =~ /^S/i ? -$val[0] : $val[0]` |
| GPSLongitude | GPSLongitude, GPSLongitudeRef                 | `$val[1] =~ /^W/i ? -$val[0] : $val[0]` |
| GPSPosition  | GPSLatitude, GPSLongitude                     | `"$val[0] $val[1]"`                     |
| GPSDateTime  | GPSDateStamp, GPSTimeStamp                    | `"$val[0] $val[1]Z"`                    |

### C. Investigation

```bash
rg "composite|Composite" src/ --type rust -l
rg "depend|require|desire" src/ --type rust
```

### D. Learned the Hard Way

1. **Order matters** - Megapixels depends on ImageSize which depends on ImageWidth/ImageHeight
2. **Desire vs Require** - `desire` dependencies are optional, `require` are mandatory
3. **Inhibit** - Some composites should NOT exist if certain tags exist (LensID-2 inhibited by LensID)
4. **$prt[] references** - Some PrintConv use `$prt[]` (printed values of deps), not `$val[]`

---

## Part 3: Tasks

### Task 1: Audit Existing Infrastructure

**Success**: Document what composite support exists in codebase

**Implementation**:

```bash
rg -l "composite|Composite" src/ --type rust
rg "composite" src/generated/ --type rust | head -20
rg "struct ExifReader" src/ --type rust -A50
```

**If architecture changed**: Search `rg "calculate|derive|compute" src/` for derived tag logic

---

### Task 2: Implement Aperture (Proof-of-Concept)

**Success**: `Composite:Aperture` appears in output when FNumber or ApertureValue exists

**Implementation**:

```rust
// Simplest composite - first available value
fn compute_aperture(tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    tags.get("FNumber").or_else(|| tags.get("ApertureValue")).cloned()
}
```

**Verification**: `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep "Composite:Aperture"`

**If architecture changed**: Goal unchanged - find where derived tags should be added

---

### Task 3: Implement GPS Composites

**Success**: GPSLatitude, GPSLongitude, GPSPosition, GPSDateTime appear in output

**Implementation**:

```rust
fn compute_gps_latitude(lat: f64, lat_ref: &str) -> f64 {
    if lat_ref.to_uppercase().starts_with('S') { -lat.abs() } else { lat }
}
```

**If architecture changed**: GPS sign logic is constant - find where to apply it

---

### Task 4: Implement ImageSize and Megapixels

**Success**: `Composite:ImageSize` and `Composite:Megapixels` appear in output

**Implementation**: ImageSize has complex selection (see composite-dependencies.json). Megapixels = width \* height / 1000000

**If architecture changed**: The math is constant - find output assembly point

---

### Task 5: Implement ShutterSpeed

**Success**: `Composite:ShutterSpeed` appears in output

**Implementation**: Ternary selection: BulbDuration if >0, else ExposureTime if defined, else ShutterSpeedValue

**If architecture changed**: Selection logic is constant

**Proof of completion**:

- [ ] At least Aperture working
- [ ] GPS composites working
- [ ] ImageSize/Megapixels working
- [ ] `make precommit` passes

---

## Emergency Recovery

```bash
git diff HEAD~ > my_changes.patch
git checkout HEAD -- src/
make precommit
```
