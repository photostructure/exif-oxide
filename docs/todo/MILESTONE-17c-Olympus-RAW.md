# Milestone 17c: Olympus RAW Support

**Goal**: Implement Olympus ORF format leveraging existing RAW infrastructure and generated lookup tables  
**Status**: IN PROGRESS ⚠️ - 90% complete, tag conflict issue remaining  
**Updated**: July 21, 2025

## 🚨 **CRITICAL STATUS UPDATE (July 21, 2025)**

Major progress achieved! Equipment discovery and IFD parsing now working correctly. Only tag conflict issue remains.

### 🎯 **CURRENT BLOCKING ISSUE: Tag ID Conflicts**

**Problem**: Equipment tags CameraType2 (0x0100) and SerialNumber (0x0101) conflict with standard EXIF tags ImageWidth/ImageHeight

**Current Behavior:**
```
Tag 0x0100: EXIF ImageWidth stored first → Equipment CameraType2 IGNORED
Tag 0x0101: EXIF ImageHeight stored first → Equipment SerialNumber IGNORED  
Tag 0x0201: No conflict → Equipment LensType extracted ✅
```

**ExifTool Correctly Outputs:**
```json
{
  "ImageWidth": 4640,         // EXIF 0x0100
  "ImageHeight": 3472,        // EXIF 0x0101  
  "CameraType2": "E-M1",      // Equipment 0x0100 (coexists!)
  "SerialNumber": "BHP242330" // Equipment 0x0101 (coexists!)
}
```

## 📋 **WORK COMPLETED IN THIS SESSION**

### ✅ **1. Fixed MakerNotes Processing (COMPLETE)**

**Problem**: MakerNotes were processed with binary data processor instead of standard IFD parsing
**Solution**: Modified `src/exif/processors.rs:456-459` to return `"Exif"` processor for Olympus MakerNotes

```rust
// Before: return Some("Olympus::Main".to_string());  
// After:  return Some("Exif".to_string()); // Use standard IFD parsing
```

### ✅ **2. Fixed Equipment Discovery (COMPLETE)**  

**Problem**: Olympus dispatch rule was preventing MakerNotes IFD parsing
**Solution**: Added MakerNotes case to `src/processor_registry/dispatch.rs:586-591`

```rust
"MakerNotes" => {
    // For Olympus MakerNotes, use standard IFD parsing to discover subdirectories
    debug!("Olympus dispatch rule: MakerNotes should use standard IFD parsing...");
    None  // Forces fallback to standard IFD parsing
}
```

### ✅ **3. Verified Equipment Processing (COMPLETE)**

**Results:**
- ✅ MakerNotes parsed as IFD with 8 entries  
- ✅ Equipment tag 0x2010 discovered
- ✅ Equipment subdirectory processed at offset 0xe66
- ✅ Equipment IFD parsed with 25 entries
- ✅ Equipment tags extracted (Tag_0104, Tag_0201, Tag_0204, etc.)

## 🔧 **REMAINING WORK FOR NEXT ENGINEER**

### Priority 1: Fix Tag ID Conflicts (CRITICAL)

**Issue**: Tag precedence system blocks MakerNotes tags when EXIF tags have same ID

**Key Files to Study:**
- `src/exif/tags.rs` - Tag storage and precedence logic (search for "Ignoring lower priority")
- `src/types/tag_source.rs` - TagSourceInfo priority system
- `src/exif/ifd.rs:280-300` - Where Equipment tags are stored

**Research Findings:**
1. Current system uses simple priority: EXIF > MakerNotes
2. Tags are stored in HashMap by ID only (no namespace consideration)
3. ExifTool namespaces tags allowing duplicates across groups

**Suggested Solutions:**
1. **Option A**: Change tag storage key from `u16` to `(String, u16)` for (namespace, id)
2. **Option B**: Allow duplicate IDs if semantic meanings differ (Equipment vs EXIF)
3. **Option C**: Special-case Equipment tags to bypass conflict checking

### Priority 2: Implement Tag Name Resolution

**Issue**: Equipment tags show as "Tag_0100" instead of "CameraType2"

**Key Files:**
- `src/implementations/olympus/equipment_tags.rs` - Tag name mappings
- `src/generated/Olympus_pm/` - Generated tag definitions
- `src/exif/tag_names.rs` - Tag name resolution logic

**What Needs to Happen:**
1. Map Equipment tag IDs to proper names
2. Ensure PrintConv is applied (e.g., LensType byte array → lens name string)

### Priority 3: Final Validation

Run full compatibility test to ensure output matches ExifTool exactly.

## 🧠 **CRITICAL DISCOVERIES & TRIBAL KNOWLEDGE**

### 1. **Equipment is an IFD, NOT Binary Data!**

**Key Discovery**: Equipment has `WRITE_PROC => WriteExif` in ExifTool (Olympus.pm:1588), meaning it's an IFD structure. The binary Equipment processor was wrong - it must use standard IFD parsing.

### 2. **MakerNotes Processing Flow**

**Correct Flow:**
1. ExifIFD discovers MakerNotes at 0x927c  
2. MakerNotes processed as standard IFD (NOT binary processor)
3. MakerNotes IFD discovers Equipment at 0x2010
4. Equipment processed as standard IFD  
5. Equipment tags extracted

### 3. **Dispatch Rule Gotchas**

- Returning `None` from dispatch rule = use fallback processing
- MakerNotes must use `"Exif"` processor for standard IFD parsing
- Equipment dispatch rule correctly returns `None` to force IFD parsing

### 4. **Tag Conflict Issue Details**

```rust
// Current behavior in src/exif/tags.rs
if existing_priority > new_priority {
    debug!("Tag 0x{:04x}: Ignoring lower priority {} (existing: {})", 
           tag_id, new_source, existing_source);
    return; // Tag ignored!
}
```

This blocks ALL MakerNotes tags that share IDs with EXIF tags, even when they have completely different meanings.

## 🔍 **DEBUG COMMANDS FOR NEXT ENGINEER**

```bash
# See full Equipment processing flow
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep -E "(Equipment|0x2010|MakerNotes.*entries)"

# Check tag conflicts
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "Ignoring.*priority"

# Compare with ExifTool
exiftool -j test-images/olympus/test.orf | jq '{CameraType2, SerialNumber, LensType}'

# See what Equipment tags ARE extracted  
cargo run -- test-images/olympus/test.orf | grep "Tag_0"
```

## 💡 **REFACTORING OPPORTUNITIES CONSIDERED**

### 1. **Namespace-Aware Tag Storage**

Current: `HashMap<u16, TagValue>`  
Better: `HashMap<(String, u16), TagValue>` or `HashMap<TagKey, TagValue>`

This would eliminate ALL tag conflict issues permanently.

### 2. **Remove Binary Equipment Processor**

The `OlympusEquipmentProcessor` in `src/processor_registry/processors/olympus.rs` is unnecessary since Equipment uses IFD format. It should be removed entirely.

### 3. **Centralize Subdirectory Detection**

Currently hardcoded checks for 0x2010, 0x2020, etc. Could use a table-driven approach:
```rust
const OLYMPUS_SUBDIRECTORIES: &[(u16, &str)] = &[
    (0x2010, "Equipment"),
    (0x2020, "CameraSettings"),
    (0x2030, "RawDevelopment"),
    // etc...
];
```

### 4. **Improve Debug Output**

Add Equipment-specific debug logging to track tag extraction:
```rust
if context.directory == "Olympus:Equipment" {
    debug!("Equipment tag 0x{:04x} ({}): {:?}", tag_id, tag_name, value);
}
```

## 📊 **FINAL STATUS SUMMARY**

**What Works:**
- ✅ ORF file detection and loading
- ✅ Basic EXIF tag extraction  
- ✅ MakerNotes discovery and IFD parsing
- ✅ Equipment subdirectory discovery (tag 0x2010)
- ✅ Equipment IFD parsing (25 entries found)
- ✅ Partial Equipment tag extraction (tags without conflicts)

**What's Broken:**
- ❌ CameraType2 (0x0100) - blocked by ImageWidth conflict
- ❌ SerialNumber (0x0101) - blocked by ImageHeight conflict
- ❌ Tag name resolution (shows Tag_0201 instead of LensType)

**Overall Progress**: ~90% complete

## 📚 **SUCCESS CRITERIA**

The milestone is complete when:

1. ✅ **Equipment Discovery**: Tag 0x2010 found and processed (DONE)
2. ✅ **Equipment IFD Parsing**: 25 entries parsed correctly (DONE)
3. ❌ **Tag Extraction**: CameraType2, SerialNumber, LensType extracted (PARTIAL)
4. ❌ **ExifTool Compatibility**: Output matches ExifTool exactly (BLOCKED)

**Current Status**: 2/4 complete with 1 partial

## 🚀 **QUICK START FOR NEXT ENGINEER**

1. **Read the tag conflict debug output:**
   ```bash
   RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "0x0100\|0x0101"
   ```

2. **Find the conflict logic in** `src/exif/tags.rs` (search for "Ignoring lower priority")

3. **Implement one of the suggested solutions** (namespace-aware storage recommended)

4. **Verify all Equipment tags extract:**
   ```bash
   cargo run -- test-images/olympus/test.orf | jq '{CameraType2, SerialNumber, LensType}'
   ```

5. **Run compatibility test** to ensure ExifTool match

The infrastructure is solid - just need to fix the tag conflict system to allow MakerNotes Equipment tags to coexist with standard EXIF tags!