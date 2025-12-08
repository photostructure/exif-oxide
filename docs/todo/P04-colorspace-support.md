# P04: ColorSpace Group1 Assignment Bug

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [ARCHITECTURE.md](../ARCHITECTURE.md)

---

## Part 1: Define Success

**Problem**: The EXIF ColorSpace tag (0xA001) is not being extracted from ExifIFD, resulting in only the Canon MakerNotes ColorSpace (0xb4) appearing in output. When `get_tag_by_name("ColorSpace")` is called, it returns the Canon version (group1="Canon") instead of the EXIF version (group1="ExifIFD").

**Why it matters**: ColorSpace is a key EXIF IFD tag used to determine color profile (sRGB vs Adobe RGB). Users expect the standard EXIF ColorSpace, not manufacturer-specific versions.

**Solution**: Investigate why EXIF ColorSpace (0xA001) is not being parsed from ExifIFD and fix the root cause.

**Success test**:
```bash
# ExifTool shows ColorSpace in ExifIFD:
exiftool -G1 -ColorSpace test-images/canon/eos_rebel_t3i.jpg
# [ExifIFD]       Color Space                     : sRGB

# Our tool should match:
cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep ColorSpace
# Should show: "EXIF:ColorSpace": ... (from ExifIFD, not MakerNotes)
```

**Key constraint**: Must preserve Canon's separate ColorSpace tag (0xb4) for users who need manufacturer-specific data, but EXIF ColorSpace should have higher priority when accessed by name.

---

## Part 2: Share Your Expertise

### A. Current Behavior

```bash
# What we currently output:
cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep ColorSpace
# "MakerNotes:ColorSpace": 1

# What ExifTool outputs:
exiftool -G1 -ColorSpace test-images/canon/eos_rebel_t3i.jpg
# [ExifIFD]       Color Space                     : sRGB
```

### B. Key Investigation Points

1. **Two different ColorSpace tags exist**:
   - EXIF ColorSpace: tag ID 0xA001, in ExifIFD, standard EXIF tag
   - Canon ColorSpace: tag ID 0xb4, in Canon MakerNotes, manufacturer-specific

2. **Priority infrastructure exists** - [src/types/metadata.rs:600-605](../../src/types/metadata.rs#L600-L605):
   ```rust
   // get_tag_by_name uses priority selection
   matching_tags.into_iter()
       .max_by_key(|tag| SourcePriority::from_namespace(&tag.group))
   ```

3. **Group1 override exists** - [src/types/metadata.rs:848-864](../../src/types/metadata.rs#L848-L864):
   ```rust
   pub fn get_group1_with_tag_override(&self, tag_id: u16) -> String {
       match tag_id {
           0xA001 => "ExifIFD".to_string(), // ColorSpace - Always in ExifIFD
           // ...
       }
   }
   ```

4. **The issue**: EXIF ColorSpace (0xA001) is NOT being parsed at all. Only Canon ColorSpace (0xb4) appears in output.

### C. Likely Root Causes to Investigate

1. **ExifIFD parsing may be incomplete** - Check if ExifIFD subdirectory is being parsed:
   ```bash
   rg "ExifIFD" src/exif/
   ```

2. **Tag may be filtered out** - Check for filters that exclude tag 0xA001:
   ```bash
   rg "0xa001|0xA001" src/
   ```

3. **Subdirectory processing** - ExifIFD is a subdirectory of IFD0 pointed to by tag 0x8769:
   ```bash
   rg "0x8769|ExifOffset" src/
   ```

### D. Key Files to Examine

| File | Purpose |
|------|---------|
| `src/exif/ifd.rs` | IFD parsing, subdirectory handling |
| `src/exif/subdirectory_processing.rs` | Nested IFD processing |
| `src/exif/mod.rs:663-674` | Group0/Group1 assignment |
| `src/types/metadata.rs:584-605` | `get_tag_by_name` priority logic |

### E. Test File

The failing test is in [tests/enhanced_exiftool_group_compatibility_tests.rs:414-490](../../tests/enhanced_exiftool_group_compatibility_tests.rs#L414-L490):

```rust
#[test]
fn test_key_exif_ifd_tag_grouping() {
    // Tests that ColorSpace has group1="ExifIFD"
    // Currently fails: ours='Canon' vs ExifTool='ExifIFD'
}
```

---

## Part 3: Tasks

### Task 1: Diagnose Why EXIF ColorSpace is Missing

**Success**: Understand why tag 0xA001 from ExifIFD is not in our output

**Implementation**:

1. Add debug logging to ExifIFD parsing:
   ```bash
   RUST_LOG=debug cargo run -- test-images/canon/eos_rebel_t3i.jpg 2>&1 | grep -i "exififd\|0xa001\|colorspace"
   ```

2. Check if ExifIFD subdirectory is being followed:
   ```bash
   rg "process.*subdirectory|parse.*ifd" src/exif/
   ```

3. Verify the ExifIFD pointer (0x8769) is being processed:
   ```bash
   exiftool -v3 test-images/canon/eos_rebel_t3i.jpg 2>&1 | grep -A5 "ExifIFD"
   ```

### Task 2: Fix ExifIFD ColorSpace Extraction

**Success**: `cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep ColorSpace` shows EXIF:ColorSpace

**Implementation**: Based on Task 1 findings, fix the root cause:
- If ExifIFD not parsed: Fix subdirectory following
- If tag filtered: Remove incorrect filter
- If priority wrong: Fix `SourcePriority::from_namespace`

### Task 3: Ensure Both ColorSpace Tags Coexist

**Success**: Both EXIF ColorSpace (0xA001) and Canon ColorSpace (0xb4) are accessible

**Implementation**:
- EXIF ColorSpace should have group="EXIF", group1="ExifIFD"
- Canon ColorSpace should have group="MakerNotes", group1="Canon"
- `get_tag_by_name("ColorSpace")` returns EXIF version (higher priority)
- Both should be in the full tag list

### Task 4: Verify Test Passes

**Success**: `cargo t test_key_exif_ifd_tag_grouping` passes

---

## Proof of Completion

- [ ] `cargo run -- test-images/canon/eos_rebel_t3i.jpg | grep ColorSpace` shows EXIF:ColorSpace
- [ ] `cargo t test_key_exif_ifd_tag_grouping` passes
- [ ] Both EXIF and Canon ColorSpace tags accessible in output
- [ ] `make precommit` passes

---

## Notes from Initial Investigation (2025-12-08)

During composite tag work, this bug was discovered:

```
Key ExifIFD tag grouping verification:
  ExifVersion: ours='ExifIFD' vs ExifTool='ExifIFD'
  FlashpixVersion: ours='ExifIFD' vs ExifTool='ExifIFD'
  ColorSpace: ours='Canon' vs ExifTool='ExifIFD'  <-- BUG
  ExifImageWidth: ours='ExifIFD' vs ExifTool='ExifIFD'
  ExifImageHeight: ours='ExifIFD' vs ExifTool='ExifIFD'
```

The EXIF ColorSpace tag (0xA001) is not appearing in our output at all. Only the Canon MakerNotes ColorSpace (0xb4) is being extracted.

Infrastructure to fix this (Group1 overrides, priority selection) already exists - the issue is the EXIF tag not being parsed in the first place.
