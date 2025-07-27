# P10c - GPS IFD Parsing Bug: Missing GPSLatitude and GPSLongitude

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](../CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!

## Project Overview

- **Goal**: Fix critical IFD parsing bug causing GPS coordinate tags (GPSLatitude, GPSLongitude) to be missing from EXIF output
- **Problem**: IFD0 parsing stops at entry 10, never processes entry 11 (GPSInfo subdirectory tag 0x8825), preventing GPS IFD processing
- **Critical Constraints**:
  - ⚡ Must maintain exact ExifTool compatibility for GPS coordinate formats
  - 🔧 Fix must not break existing tag extraction (confirmed working: GPSAltitude, GPSTimeStamp, etc.)
  - 📐 Must handle all file formats that contain GPS data

## Background & Context

GPS coordinates are among the most critical EXIF tags for photo management applications. The previous work successfully fixed a MakerNotes early return bug, but revealed a deeper IFD parsing issue:

1. **MakerNotes Fix Completed**: Fixed early return in subdirectory processing that broke parent IFD loops
2. **New Bug Discovered**: Main IFD0 parsing loop stops at entry 10, never reaching entry 11 (GPSInfo)
3. **Impact**: GPSLatitude (0x0002) and GPSLongitude (0x0004) are completely missing from output
4. **Partial Success**: Some GPS tags appear (GPSAltitude, GPSTimeStamp) suggesting GPS processing works when triggered

**Related Work**: This builds on the MaxApertureValue double conversion fix and PrintConv registry improvements from the previous session.

## Technical Foundation

**Key Files**:
- [`src/exif/ifd.rs`](../src/exif/ifd.rs) - Main IFD parsing logic where the bug occurs
- [`src/exif/processors.rs`](../src/exif/processors.rs) - Subdirectory processing and dispatch
- [`src/generated/GPS_pm/tag_kit/gps.rs`](../src/generated/GPS_pm/tag_kit/gps.rs) - GPS tag definitions (confirmed working)

**Debug Evidence**:
```
Processing IFD IFD0 entry 0 of 13 at offset 0xa    ✓
Processing IFD IFD0 entry 1 of 13 at offset 0x16   ✓
...
Processing IFD IFD0 entry 10 of 13 at offset 0x82  ✓ (ExifIFD subdirectory)
[Processing stops here - entries 11 and 12 never processed]
```

ExifTool verbose shows:
```
11) GPSInfo (SubDirectory) -->
     - Tag 0x8825 (4 bytes, int32u[1])  <- This is never processed!
```

## Work Completed

✅ **Confirmed GPS tag kit integrity**: GPS tag definitions for GPSLatitude (id: 2) and GPSLongitude (id: 4) are properly generated

✅ **Verified MakerNotes fix effectiveness**: MakerNotes processing no longer breaks parent IFD with early returns

✅ **Isolated the root cause**: IFD0 parsing loop terminates after entry 10 instead of continuing to entries 11-12

✅ **Confirmed GPS processing works**: When GPS IFD is processed, tags extract correctly with proper PrintConv/ValueConv

✅ **CRITICAL DISCOVERY & RESOLUTION**: Root cause was RICOH MakerNotes corrupted entry count (21,097 vs expected 9) causing infinite loop that prevented IFD0 from continuing. Fixed with proper RICOH signature detection using 8-byte offset matching ExifTool's `Start => '$valuePtr + 8'` logic.

✅ **MAJOR BREAKTHROUGH**: IFD parsing now works correctly! After RICOH fix:
- ✅ IFD0 processes all 13 entries including entry 11 (GPSInfo) 
- ✅ GPS IFD processes all 24 entries (tags 0x0-0x1e)
- ✅ GPSLongitude and GPSLongitudeRef appear in output correctly
- ❌ **NEW ISSUE**: GPSLatitude and GPSLatitudeRef are processed but missing from final output

## Remaining Tasks

### 1. ✅ ~~Investigate subdirectory processing control flow bug~~ **COMPLETED**

**✅ RESOLUTION**: Fixed RICOH MakerNotes signature detection in [`src/implementations/ricoh.rs`](../../src/implementations/ricoh.rs). Issue was corrupted entry count causing infinite loop.

### 2. ✅ ~~Fix IFD parsing control flow~~ **COMPLETED**

**✅ VERIFICATION**: IFD0 now processes all 13 entries:
```
Processing IFD IFD0 entry 10 of 13 at offset 0x82  ✓ (ExifIFD)
Processing IFD IFD0 entry 11 of 13 at offset 0x8e  ✓ (GPSInfo) ← FIXED!
Processing IFD IFD0 entry 12 of 13 at offset 0x9a  ✓
```

### 3. **INVESTIGATE**: GPS coordinate conversion bug

**NEW DISCOVERY**: Tags are processed but conversion fails:
- ✅ **Parsing Works**: GPS tags 0x1 (GPSLatitudeRef) and 0x2 (GPSLatitude) are processed
- ✅ **GPS IFD Works**: All 24 GPS entries processed correctly
- ❌ **Conversion Fails**: GPSLatitude/GPSLatitudeRef disappear between parsing and output
- ✅ **Partial Success**: GPSLongitude/GPSLongitudeRef work correctly

**Debug Evidence**:
```
Processing tag 0x1 (1) from GPS (format: Ascii, count: 2)     ← GPSLatitudeRef
Processing tag 0x2 (2) from GPS (format: Rational, count: 3)  ← GPSLatitude  
Processing tag 0x3 (3) from GPS (format: Ascii, count: 2)     ← GPSLongitudeRef
Processing tag 0x4 (4) from GPS (format: Rational, count: 3)  ← GPSLongitude
```

**✅ ROOT CAUSE CONFIRMED**: Tag ID collision between EXIF and GPS namespaces!

**Debug Evidence**:
```
Tag 0x0001: Keeping first encountered EXIF over EXIF  ← GPSLatitudeRef blocked!
Tag 0x0002: Keeping first encountered EXIF over EXIF  ← GPSLatitude blocked!
```

**Testing Confirmation**:
- ✅ `cargo run -- ./t/images/Ricoh2.jpg -GPSLongitude` → Returns 0.5075027777777777
- ❌ `cargo run -- ./t/images/Ricoh2.jpg -GPSLatitude` → Returns nothing  
- ❌ `cargo run -- ./t/images/Ricoh2.jpg -GPSLatitudeRef` → Returns nothing

**Problem**: Tags 0x1 and 0x2 exist in both EXIF and GPS IFDs. Our precedence logic keeps the first value (EXIF) and discards the GPS values. This is why longitude works (tags 0x3, 0x4 are GPS-specific) but latitude fails.

**Solution**: Fix GPS IFD context handling in tag storage. GPS tags need proper namespace isolation to avoid collisions with EXIF tags.

### 4. Verify GPS coordinate extraction

**Acceptance Criteria**: GPSLatitude and GPSLongitude appear in output for test images

**✅ Correct Output**:
```json
{
  "EXIF:GPSLatitude": "42 deg 2' 4.47\"",
  "EXIF:GPSLatitudeRef": "North", 
  "EXIF:GPSLongitude": "0 deg 30' 27.01\"",
  "EXIF:GPSLongitudeRef": "West"
}
```

**❌ Common Mistake**: Tags appear as decimal degrees instead of DMS format (indicates ValueConv working but PrintConv broken)

### 4. Fix remaining GPS PrintConv issues

**Acceptance Criteria**: All GPS tags show ExifTool-compatible formatting

**Issues to Address**:
- GPSAltitudeRef: Currently shows "Above Sea Level", should show "Below Sea Level" (value 0 vs 1 issue)
- GPSLongitude: Shows decimal `0.5075027777777777` instead of DMS `"0 deg 30' 27.01\""`

**Implementation**: Verify PrintConv registry mappings in `codegen/src/conv_registry.rs:54-58`

## Testing Strategy

**Primary Test Image**: `third-party/exiftool/t/images/Ricoh2.jpg` (confirmed to have GPS data)

**Validation Commands**:
```bash
# Our output
cargo run --bin exif-oxide third-party/exiftool/t/images/Ricoh2.jpg | grep GPS

# ExifTool reference
exiftool -j -G third-party/exiftool/t/images/Ricoh2.jpg | jq '.[] | to_entries[] | select(.key | contains("GPS"))'
```

**Additional Test Images**: Verify fix works across multiple GPS-enabled files:
- Test images in `test-images/` directory
- Any files from `third-party/exiftool/t/images/` with GPS data

## Success Criteria & Quality Gates

**Primary Success**: Both GPSLatitude and GPSLongitude appear in output for all GPS-enabled test images

**Secondary Success**: All GPS tags show correct ExifTool-compatible formatting

**Quality Gates**:
1. ✅ `make precommit` passes (lint, typecheck, tests)
2. ✅ GPS coordinate precision matches ExifTool exactly
3. ✅ No regression in existing EXIF tag extraction
4. ✅ Debug logs confirm IFD0 processes all entries

## Gotchas & Tribal Knowledge

**Control Flow Complexity**: The IFD parsing involves nested subdirectory processing that can break parent loops in subtle ways. The MakerNotes fix addressed one case, but there may be other control flow issues.

**GPS Tag ID Mapping**: GPS tags use different IDs than EXIF tags:
- GPSLatitudeRef = 1, GPSLatitude = 2 (in GPS IFD)  
- Not to be confused with EXIF tag IDs

**ExifTool Compatibility**: GPS coordinates must show as DMS format (`"42 deg 2' 4.47\""`) not decimal degrees (`42.034575`) to match ExifTool exactly.

**PrintConv vs ValueConv**: GPS coordinates require both:
- ValueConv: Raw rational values → decimal degrees
- PrintConv: Decimal degrees → DMS string format

**Testing Gotcha**: Some GPS tags may appear in output even when GPS IFD isn't processed properly - this indicates partial/residual processing, not complete success.