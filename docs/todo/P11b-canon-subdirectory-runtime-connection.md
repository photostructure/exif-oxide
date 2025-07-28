# P11b - Fix Canon Binary Data Runtime Connection

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

---

## Project Overview

- **Goal**: Fix the final 5% of Canon MakerNotes binary data extraction so individual tags display like ExifTool
- **Problem**: Canon binary data extracts as raw arrays `[18576, 255, 8192...]` instead of meaningful tags `MacroMode: Normal, Quality: Fine`
- **Critical Constraints**:
  - 🔧 Infrastructure is 95% complete - DO NOT rebuild anything
  - 📐 Must use existing tag kit system and binary data parsers
  - ⚡ Binary data parsers are already generated and working

## Background & Context

This is the **final task** for P11 - Complete SubDirectory Binary Data Parsers. Previous engineers have built all the infrastructure:

- ✅ Canon MakerNotes extraction (50+ tags working)
- ✅ Binary data parsers generated (`measuredcolor_binary_data.rs`, `processing_binary_data.rs`, `previewimageinfo_binary_data.rs`)
- ✅ Tag kit integration functions exist
- ✅ Canon subdirectory processing pipeline exists

**The ONLY missing piece**: Canon binary data tags are not recognized as "having subdirectory processing" so they aren't parsed into individual tags.

See [@docs/todo/P11-complete-subdirectory-binary-parsers.md](P11-complete-subdirectory-binary-parsers.md) for full context.

## Technical Foundation

**Key Files:**
- `src/generated/Canon_pm/tag_kit/mod.rs` - Tag kit with `has_subdirectory()` and `process_subdirectory()` functions
- `src/implementations/canon/mod.rs:811` - `process_canon_subdirectory_tags()` calls tag kit
- `codegen/generated/extract/tag_kits/canon__tag_kit.json` - Source of truth for Canon tag definitions

**Current Flow:**
1. Canon MakerNotes → `process_canon_makernotes()` → `process_canon_subdirectory_tags()`
2. `process_canon_subdirectory_tags()` calls `tag_kit::has_subdirectory(tag_id)`
3. **BUG**: `has_subdirectory()` returns `false` for Canon binary data tags
4. Result: Binary arrays not processed into individual tags

## Work Completed

- **Tag Kit Generation**: All Canon binary data parsers generated and compiling
- **Binary Data Integration**: Generated functions like `process_canon_camerasettings()` exist
- **Runtime Testing**: Confirmed binary data extracted as arrays: `CanonCameraSettings: [18576, 255, 8192, ...]`
- **Root Cause Identified**: `CANON_PM_TAG_KITS` HashMap missing Main table tags with subdirectories
- **Root Cause Analysis**: Tag kit extraction only included tags FROM subdirectory tables, not Main table tags WITH subdirectories
- **Fix Implemented**: Modified `codegen/src/generators/lookup_tables/mod.rs` to integrate Main table subdirectory info
- **Tag Kit Regenerated**: Main table tags (0x1, 0x2, etc.) now have subdirectory entries in generated tag kit

## Remaining Tasks

### ✅ 1. Identify Missing Canon Tag IDs - COMPLETED

Found that Main table tags (0x1, 0x2, etc.) were missing from tag kit. These tags have subdirectories that parse binary data.

### ✅ 2. Fix Tag Kit JSON Configuration - COMPLETED

Modified codegen to integrate Main table tag structure data. Canon tags now have proper subdirectory entries:
```json
{
  "tag_id": "1",
  "name": "CanonCameraSettings",
  "subdirectory": {
    "tag_table": "CameraSettings",
    "is_binary_data": true
  }
}
```

### ✅ 3. Verify Binary Data Parser Connection - COMPLETED

`has_subdirectory()` now returns true for Canon binary data tags after tag kit regeneration.

### 4. Test End-to-End Canon Tag Extraction

**Acceptance Criteria**: Canon tags show individual values like ExifTool

**✅ Correct Output:**
```json
{
  "MakerNotes:MacroMode": "Normal",
  "MakerNotes:SelfTimer": "Off", 
  "MakerNotes:Quality": "Fine",
  "MakerNotes:CanonFlashMode": "Off"
}
```

**❌ Current Output:**
```json
{
  "MakerNotes:CanonCameraSettings": [18576, 255, 8192, 0, 0, ...]
}
```

**Implementation**: Run full test after fixes to verify individual tag extraction

## Prerequisites

- P11a must be complete (binary data parsers generated) - ✅ DONE
- Understanding of tag kit system - see `src/generated/Canon_pm/tag_kit/mod.rs`

## Testing Strategy

**Unit Tests:**
- Test `has_subdirectory()` for Canon tag IDs 0x1, 0x93, 0x26
- Verify `CANON_PM_TAG_KITS` contains subdirectory entries

**Integration Tests:**
- Compare Canon tag extraction with ExifTool output
- Test file: `test-images/canon/canon_eos_r5_mark_ii_10.jpg`

**Manual Testing:**
```bash
# Before fix
cargo run test-images/canon/canon_eos_r5_mark_ii_10.jpg | grep "CanonCameraSettings"
# Should show: "CanonCameraSettings": [18576, 255, ...]

# After fix  
cargo run test-images/canon/canon_eos_r5_mark_ii_10.jpg | grep "MacroMode"
# Should show: "MacroMode": "Normal"
```

## Success Criteria & Quality Gates

**Definition of Done:**
1. ✅ Canon individual tags extracted (MacroMode, Quality, etc.)
2. ✅ No regression in existing Canon tag extraction
3. ✅ Tests pass: `cargo t canon`
4. ✅ Precommit passes: `make precommit`

**Quality Gates:**
- Compare output with ExifTool for same image
- Verify 20+ individual Canon MakerNotes tags extracted

## Gotchas & Tribal Knowledge

- **Don't regenerate everything**: The infrastructure is working, this is a configuration/mapping issue
- **Tag Kit System**: The `subdirectory` field in JSON maps to `SubDirectoryType::Binary` in Rust
- **Canon Tag IDs**: Canon uses decimal IDs in JSON but hex in debug output (0x1 = 1)
- **Binary Data Parsers**: Already exist as `*_binary_data.rs` files, don't need regeneration
- **Trust ExifTool**: Compare final output with ExifTool to verify correctness

## Expected Time Investment

- **Research/Debug**: 1-2 hours to identify missing tag mappings
- **Implementation**: 30 minutes to fix tag kit JSON or codegen
- **Testing**: 30 minutes to verify all Canon tags working
- **Total**: 2-3 hours maximum

This should be a quick win since all infrastructure exists!