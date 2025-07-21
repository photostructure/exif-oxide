# Milestone 17c: Olympus RAW Support

**Goal**: Implement Olympus ORF format leveraging existing RAW infrastructure and generated lookup tables  
**Status**: IN PROGRESS ⚠️ - 85% complete, tag conflict and codegen integration remaining  

## 🚨 CRITICAL STATUS UPDATE

Major progress achieved! Equipment discovery and IFD parsing now working correctly. Two critical issues remain:
1. Tag ID conflict system preventing Equipment tags from coexisting with EXIF tags
2. Equipment tag name resolution needs proper codegen integration

### 🎯 **CURRENT BLOCKING ISSUES**

#### 1. Tag ID Conflicts

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

#### 2. Equipment Tag Name Resolution

**Problem**: `src/implementations/olympus/equipment_tags.rs` was manually created instead of being generated through codegen

**Current State**:
- Equipment extraction IS configured in `codegen/config/Olympus_pm/equipment_tag_table_structure.json`
- Equipment inline PrintConv tables ARE being generated in `src/generated/Olympus_pm/equipment_inline.rs`
- Equipment tag structure enum and lookup functions are NOT being generated

## 📋 **WORK COMPLETED IN PREVIOUS SESSIONS**

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

## 🔧 **CURRENT SESSION PROGRESS (January 21, 2025)**

### 1. Investigated Tag Conflict System

**Finding**: The tag storage system uses `HashMap<u16, TagValue>` where only the tag ID is the key. This prevents tags with the same ID from different contexts (EXIF vs MakerNotes) from coexisting.

**Code Location**: `src/exif/tags.rs:16-55` - `store_tag_with_precedence()` method

**Current Logic**:
```rust
// Current behavior in src/exif/tags.rs
if existing_priority > new_priority {
    debug!("Tag 0x{:04x}: Ignoring lower priority {} (existing: {})", 
           tag_id, new_source, existing_source);
    return; // Tag ignored!
}
```

### 2. Equipment Codegen Investigation

**Problem**: Equipment tag name resolution uses manual code instead of codegen

**What I Found**:
1. Equipment tag structure CAN be extracted:
   ```bash
   cd codegen
   perl extractors/tag_table_structure.pl ../third-party/exiftool/lib/Image/ExifTool/Olympus.pm Equipment
   ```
   This successfully creates `olympus_equipment_tag_structure.json` with all 25 tags

2. But the codegen system isn't generating:
   - Equipment enum (like `OlympusEquipmentDataType`)
   - Tag name lookup functions
   - Integration with the tag resolution system

3. Manual workaround exists in `src/implementations/olympus/equipment_tags.rs` but violates CODEGEN.md principles

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

### 4. **Tag Conflict is System-Wide**

This issue affects ALL manufacturers with MakerNotes subdirectories:
- Canon: CameraSettings tags may conflict
- Nikon: Various subdirectory tags may conflict  
- Sony: Similar issues likely exist

## 🔍 **DEBUG COMMANDS FOR NEXT ENGINEER**

```bash
# See full Equipment processing flow
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep -E "(Equipment|0x2010|MakerNotes.*entries)"

# Check tag conflicts
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "Ignoring.*priority"

# Compare with ExifTool
exiftool -j test-images/olympus/test.orf | jq '{CameraType2, SerialNumber, LensType}'

# See what Equipment tags ARE extracted (but with wrong names)
cargo run -- test-images/olympus/test.orf | grep "Tag_0"

# Check if Equipment structure is being extracted
cd codegen && ls -la generated/extract/*equipment*
```

## 🚀 **QUICK START FOR NEXT ENGINEER**

### Step 1: Fix Equipment Codegen Integration

1. **Investigate why Equipment tag structure isn't being generated:**
   ```bash
   cd codegen
   # Check if Equipment is being processed
   grep -r "Equipment" src/
   ```

2. **The extraction works but generation doesn't:**
   ```bash
   # This extracts successfully:
   perl extractors/tag_table_structure.pl ../third-party/exiftool/lib/Image/ExifTool/Olympus.pm Equipment
   # Creates: generated/extract/olympus_equipment_tag_structure.json
   # But no Equipment enum is generated in src/generated/Olympus_pm/
   ```

3. **Look at how other subdirectory tables are handled:**
   - Check if CameraSettings, RawDevelopment etc. have separate enums
   - Or if they're part of the main OlympusDataType enum
   - Follow the same pattern for Equipment

4. **Replace manual equipment_tags.rs with generated code:**
   - Delete `src/implementations/olympus/equipment_tags.rs`
   - Ensure codegen creates proper Equipment tag lookup functions
   - Update `src/exif/ifd.rs:698` to use generated function

### Step 2: Fix Tag Conflict System

1. **Implement namespace-aware storage** (recommended approach):
   ```rust
   // Option A: Compound key
   pub struct ExifReader {
       extracted_tags: HashMap<(String, u16), TagValue>, // (namespace, tag_id)
       // ...
   }
   
   // Option B: Custom key type
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct TagKey {
       namespace: String,
       tag_id: u16,
   }
   ```

2. **Update all code that accesses extracted_tags**:
   - Search for all uses of `extracted_tags.get(&tag_id)`
   - Update to include namespace in lookup
   - Consider backward compatibility approach

3. **Test the fix:**
   ```bash
   RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "0x0100\|0x0101"
   # Should show BOTH ImageWidth AND CameraType2
   ```

### Step 3: Verify Everything Works

```bash
# Check Equipment tags extract with proper names
cargo run -- test-images/olympus/test.orf | grep -E "CameraType2|SerialNumber|LensType"

# Run compatibility test
cargo run --bin compare-with-exiftool test-images/olympus/test.orf
```

## 📚 **KEY CODE LOCATIONS**

### Tag Conflict System
- `src/exif/tags.rs:16-55` - `store_tag_with_precedence()` - Core conflict logic
- `src/exif/mod.rs:37` - `extracted_tags: HashMap<u16, TagValue>` field
- `src/types/metadata.rs:340-400` - `TagSourceInfo` and priority system

### Equipment Tag Resolution
- `src/exif/ifd.rs:694-701` - Equipment tag name resolution
- `src/implementations/olympus/equipment_tags.rs` - MANUAL implementation (should be GENERATED)
- `src/generated/Olympus_pm/equipment_inline.rs` - Generated PrintConv tables (working)

### Codegen Configuration
- `codegen/config/Olympus_pm/equipment_tag_table_structure.json` - Equipment extraction config
- `codegen/extractors/tag_table_structure.pl` - Extracts tag structure from ExifTool
- `codegen/src/generators/tag_structure.rs` - Should generate Equipment enum/functions

## 💡 **REFACTORING OPPORTUNITIES CONSIDERED**

### 1. **Namespace-Aware Tag Storage** (CRITICAL)

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

### 4. **Unified Subdirectory Tag Generation**

Enhance codegen to automatically:
- Detect all tables referenced in SubDirectory definitions
- Generate lookup functions for each subdirectory table
- Create a registry of subdirectory tag resolvers

### 5. **Enhanced Debug Output**

Add Equipment-specific debug logging:
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
- ❌ Equipment codegen integration

**Overall Progress**: ~85% complete

## 📚 **SUCCESS CRITERIA**

The milestone is complete when:

1. ✅ **Equipment Discovery**: Tag 0x2010 found and processed (DONE)
2. ✅ **Equipment IFD Parsing**: 25 entries parsed correctly (DONE)
3. ❌ **Tag Extraction**: CameraType2, SerialNumber, LensType extracted with proper names
4. ❌ **ExifTool Compatibility**: Output matches ExifTool exactly
5. ❌ **Codegen Integration**: Equipment tags generated not manually maintained

**Current Status**: 2/5 complete

## 🎯 **RECOMMENDED APPROACH**

1. **Fix codegen first** - This ensures we're following project principles
2. **Then fix tag conflicts** - This benefits all manufacturers
3. **Finally verify Olympus** - Should "just work" after the above

The infrastructure is solid - these two architectural improvements would complete this milestone and benefit all manufacturers!