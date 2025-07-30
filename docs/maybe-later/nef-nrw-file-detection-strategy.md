# P21b: NEF/NRW File Type Detection Strategy

## Project Overview

- **Goal**: Implement predictable, documented NEF/NRW file type detection by trusting file extensions
- **Problem**: ExifTool's multi-stage detection requires MakerNotes parsing, causing false positives in our incomplete implementation
- **Constraints**: Must eliminate test failures while maintaining simple, fast file detection

## Context & Foundation

### The NEF/NRW Detection Problem

**ExifTool's Complex Detection Process**:
1. **Initial Detection**: Based on file extension (.nef → NEF, .nrw → NRW)
2. **IFD0 Override**: If NEF file has JPEG compression (value 6) in IFD0 → change to NRW
3. **MakerNotes Override**: If NRW file has NEFLinearizationTable (tag 0x0096) → change back to NEF

**Why This Is Hard**:
- Both NEF and NRW use identical TIFF structure
- Many NEF files have JPEG compression in IFD0 (for thumbnails) 
- Only NEFLinearizationTable in MakerNotes definitively identifies NEF
- Parsing MakerNotes during file detection adds significant complexity

**Real-World Examples**:
- `test-images/nikon/z6_3_01.nef`: Has JPEG compression in IFD0, but ExifTool detects as NRW based on missing NEFLinearizationTable
- These are actually NRW files with .nef extensions (camera firmware quirk)

**Key ExifTool Sources**:
- ExifTool Exif.pm:1138-1141 (NRW detection from JPEG compression)
- ExifTool.pm:8672-8674 (NEF recovery from NEFLinearizationTable)
- Nikon.pm:2672 (NEFLinearizationTable tag 0x0096 definition)

### Our Design Decision: Trust Extensions

**Rationale**:
- **Predictable behavior** - Users know exactly what to expect
- **Avoids false positives** - Our incomplete content analysis incorrectly identified NEF as NRW
- **Industry standard** - Most software trusts extensions for initial type detection
- **Performance** - No need to parse TIFF/MakerNotes during file detection
- **Maintainability** - Simpler code with clear, documented behavior

**Trade-offs**:
- Some files with "wrong" extensions will be detected differently than ExifTool
- But these represent camera firmware quirks, not user error

## Work Completed

- ✅ **Removed content-based NEF/NRW detection** → Eliminated false positives where NEF files were detected as NRW
- ✅ **Updated documentation** → README.md, MANUFACTURER-FACTS.md, TROUBLESHOOTING.md explain the strategy
- ✅ **Simplified file_detection.rs** → Removed `correct_nef_nrw_type()` function and IFD0 reading
- ✅ **Updated test infrastructure** → Added `ExtensionTrusted` known difference type

### Decision Log

**Rejected Alternatives**:
- **Option 1: Keep current implementation** → Still had false positives, required maintenance of known difference list
- **Option 2: Enhanced detection** → Would require MakerNotes parsing during file detection (too complex)

**Chose Option 3: Trust Extensions** → Provides predictable, documented behavior

## Remaining Tasks

### Task: Fix Remaining Test Failures

**Problem**: New test images have different naming patterns than what was configured

**Current Failures**:
```
test-images/nikon/z_6_3_02.nef: ExifTool='image/x-nikon-nrw', Ours='image/x-nikon-nef'
test-images/nikon/z_6_3_01.nef: ExifTool='image/x-nikon-nrw', Ours='image/x-nikon-nef'
test-images/nikon/z_8_73.nef: ExifTool='image/x-nikon-nrw', Ours='image/x-nikon-nef'
test-images/nikon/z_6_3_03.nef: ExifTool='image/x-nikon-nrw', Ours='image/x-nikon-nef'
```

**Success**: All MIME type compatibility tests pass

**Approach**: 
1. Identify all NEF files that ExifTool detects as NRW
2. Add them to `KNOWN_DIFFERENCES` with `ExtensionTrusted` classification
3. Verify that these are genuine NRW files with .nef extensions

**Failures to avoid**:
- ❌ Adding files that are actually NEF → Would mask real detection problems
- ❌ Missing new test images → Tests will continue to fail

### RESEARCH: Verify File Classification

**Questions**: Are these files actually NRW content with .nef extensions?

**Method**:
1. Check ExifTool output: `exiftool -FileType -Compression -G1 [file]`
2. Look for JPEG compression in IFD0 (value 6)
3. Confirm missing NEFLinearizationTable
4. Document findings in test comments

**Done when**: All failing files are properly classified and documented

## Testing

- **Integration**: `cargo test --test mime_type_compatibility_tests` must pass
- **Manual check**: Run `exiftool -FileType [failing_file]` and confirm it returns NRW
- **Regression**: Ensure no previously passing files now fail

## Definition of Done

- [ ] All MIME type compatibility tests pass
- [ ] All failing NEF files properly classified as `ExtensionTrusted`
- [ ] Documentation updated with file-specific explanations
- [ ] `make precommit` clean

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

- **Test names don't match config** → New test images added with different naming → Update `KNOWN_DIFFERENCES` with exact file paths
- **ExifTool says it's NRW but extension is .nef** → Camera firmware quirk, some models output NRW content with .nef extension → This is the expected behavior we're documenting
- **File detection seems broken** → We now trust extensions by design → Check documentation to understand the trade-offs

## Quick Debugging

Stuck? Try these:

1. `exiftool -FileType -Compression -G1 [file]` - See what ExifTool detects
2. `grep -r "z_6_3" tests/` - Find test configurations
3. `cargo test --test mime_type_compatibility_tests -- --nocapture` - See actual vs expected
4. Check `test-images/nikon/` directory for actual file names

## File Status Investigation

**Files to investigate**:
- `test-images/nikon/z_6_3_01.nef` (underscore format)
- `test-images/nikon/z_6_3_02.nef` 
- `test-images/nikon/z_6_3_03.nef`
- `test-images/nikon/z_8_73.nef`

**vs Previously Fixed**:
- `test-images/nikon/z6_3_01.nef` (no underscore format)
- `test-images/nikon/z6_3_02.nef`
- `test-images/nikon/z6_3_03.nef`

This suggests test image renaming or addition of new files with different naming conventions.