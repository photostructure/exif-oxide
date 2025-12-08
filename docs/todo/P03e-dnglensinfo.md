# P03e: DNGLensInfo Tag Implementation

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), P03d (Unknown Tags Research) complete

**Research Reference**: [unknown-tags-research.md](../analysis/unknown-tags-research.md#dnglensinfo)

---

## Part 1: Define Success

**Problem**: DNGLensInfo tag (0xc630) is in required-tags.json but not being extracted or formatted

**Why it matters**: DNG files store lens information in this tag; photographers need lens metadata for cataloging

**Solution**: Add DNGLensInfo to EXIF tag tables with PrintLensInfo formatting

**Success test**:
```bash
# Find a DNG file with lens info and verify output
cargo run --bin compare-with-exiftool -- test-images/dng/sample.dng 2>/dev/null | grep DNGLensInfo
# Should show matching values
```

**Key constraint**: Must use ExifTool's `PrintLensInfo` function pattern from Exif.pm:3475-3483

---

## Part 2: Share Your Expertise

### A. ExifTool Source

**Location**: `lib/Image/ExifTool/Exif.pm:3475-3483`

```perl
0xc630 => {
    Name => 'DNGLensInfo',
    Groups => { 2 => 'Camera' },
    Writable => 'rational64u',
    WriteGroup => 'IFD0',
    Count => 4,
    PrintConv =>\&PrintLensInfo,
    PrintConvInv => \&ConvertLensInfo,
},
```

**Data format**: 4-element rational64u array:
- `[0]` = Minimum focal length (mm)
- `[1]` = Maximum focal length (mm)
- `[2]` = Minimum f-number at min focal length
- `[3]` = Maximum f-number at max focal length

### B. PrintLensInfo Function

**Location**: `lib/Image/ExifTool/Exif.pm:5197-5255`

```bash
# Find the function
rg "sub PrintLensInfo" third-party/exiftool/lib/Image/ExifTool/Exif.pm -A 30
```

The function formats the 4 rationals into human-readable form like:
- `"24-70mm f/2.8"` (zoom lens)
- `"50mm f/1.8"` (prime lens - min==max focal length)

### C. Similar Pattern to Follow

```bash
# Check if LensInfo tag exists (similar pattern)
rg "LensInfo|0xc630" src/generated/Exif_pm/ --type rust
```

LensInfo (0xa432) is a similar tag - check if it's implemented and copy the pattern.

### D. Learned the Hard Way

1. **Rational arrays**: EXIF rational values are `(numerator, denominator)` pairs. Division by zero is possible if denominator is 0.

2. **PrintConv functions**: ExifTool's `PrintLensInfo` handles edge cases like:
   - `0/0` (unknown value) → display nothing or "?"
   - Prime lens detection (min focal == max focal)
   - Variable aperture formatting

3. **Generated code**: Check if tag is already in generated tables but just missing PrintConv wiring.

---

## Part 3: Tasks

### Task 1: Verify Tag Extraction

**Success**: Raw DNGLensInfo values appear in output (even without formatting)

**Implementation**:

```bash
# Check if tag is in generated Exif tables
rg "0xc630|DNGLensInfo" src/generated/Exif_pm/ --type rust

# If not found, check codegen config
rg "c630|DNGLensInfo" codegen/
```

**If tag not being extracted**:
1. Verify Exif.pm is in `exiftool_modules.json`
2. Check TagKitStrategy handles DNG tags
3. Run `make codegen` and re-check

**If architecture changed**: The goal is extracting 4 rational values from IFD0 tag 0xc630.

---

### Task 2: Implement PrintLensInfo

**Success**: `cargo run image.dng | grep DNGLensInfo` shows formatted lens string

**Implementation**:

1. Check if `PrintLensInfo` already exists in codegen-runtime:
   ```bash
   rg "PrintLensInfo|print_lens_info" src/ codegen-runtime/
   ```

2. If not, add to `codegen-runtime/src/exif_functions.rs` (or similar):
   ```rust
   /// Format lens info array as human-readable string
   /// ExifTool: lib/Image/ExifTool/Exif.pm:5197-5255
   pub fn print_lens_info(val: &TagValue) -> TagValue {
       // Extract 4 rationals: min_focal, max_focal, min_fnum, max_fnum
       // Format: "24-70mm f/2.8-4" or "50mm f/1.8"
   }
   ```

3. Wire the function to DNGLensInfo tag's PrintConv

**If architecture changed**: Find where other EXIF PrintConv functions live and follow that pattern.

---

### Task 3: Test with Real DNG Files

**Success**: Output matches ExifTool for test DNG files

**Implementation**:

```bash
# Find DNG test files
find third-party/exiftool/t/images -name "*.dng" -o -name "*.DNG"

# Compare output
for f in $(find third-party/exiftool/t/images -iname "*.dng"); do
  echo "=== $f ==="
  cargo run --bin compare-with-exiftool -- "$f" 2>/dev/null | grep -i lens
done
```

**Edge cases to verify**:
- Prime lens (min focal == max focal)
- Variable aperture zoom
- Missing/zero values

**If architecture changed**: The test goal is output parity with ExifTool.

---

## Proof of Completion

- [ ] `rg "DNGLensInfo|0xc630" src/generated/` shows tag definition
- [ ] `rg "print_lens_info|PrintLensInfo" src/ codegen-runtime/` finds implementation
- [ ] `cargo run --bin compare-with-exiftool -- [dng_file] | grep DNGLensInfo` shows match
- [ ] `cargo t` passes (no regressions)
- [ ] `make precommit` passes

---

## Emergency Recovery

```bash
# If changes break things
git diff src/ codegen-runtime/
git checkout HEAD -- src/generated/  # Regenerate with make codegen
make codegen && cargo t
```
