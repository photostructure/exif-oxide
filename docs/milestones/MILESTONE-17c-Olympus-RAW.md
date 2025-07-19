# Milestone 17c: Olympus RAW Support

**Duration**: 6-8 hours (Revised from 2 weeks)  
**Goal**: Implement Olympus ORF format leveraging existing RAW infrastructure and generated lookup tables  
**Status**: 95% Complete (Core infrastructure complete, Equipment section extraction pending)

## 🚀 **IMPLEMENTATION STATUS (As of July 18, 2025)**

### ✅ **COMPLETED WORK**

**✅ Core Olympus ORF Support - FULLY IMPLEMENTED**
- ✅ **FixFormat Mechanism**: Solved the core TIFF format type 41015 issue
- ✅ **ORF Detection**: Added Olympus to RawFormat enum and detection logic
- ✅ **Handler Integration**: Implemented OlympusRawHandler following PanasonicRawHandler pattern  
- ✅ **TIFF Processing**: ORF files processed using existing TIFF infrastructure
- ✅ **ExifTool Compatibility**: Generated test snapshot and integration
- ✅ **CLI Integration**: `cargo run -- file.orf` works correctly

**✅ Critical Technical Breakthrough: Olympus FixFormat System**

**Problem**: Olympus MakerNotes use non-standard TIFF format types that violate the TIFF specification. Old Olympus cameras write subdirectory entries with format types like `undef` or `string` instead of `ifd`, causing standard TIFF parsers to reject them as "Invalid TIFF format type".

**ExifTool Solution**: Uses dual-path approach with `FixFormat => 'ifd'` directive:
- Path 1: When format is NOT `ifd` -> process as binary data  
- Path 2: When format is invalid -> apply FixFormat to convert to `ifd` format

**Our Implementation**: 
- Detection: Check if we're in Olympus MakerNotes and tag is known subdirectory (0x2010-0x5000)
- Correction: Convert invalid format types to `TiffFormat::Ifd`
- Processing: Continue with standard IFD processing

This handles Equipment (0x2010), CameraSettings (0x2020), and other Olympus subdirectories.

**✅ Current Working Status**
- **Basic ORF metadata extraction**: Successfully extracts Make, Model, LensModel, ISO, FNumber, ExposureTime, etc.
- **FixFormat mechanism**: Working correctly (see debug logs: "Applying Olympus FixFormat: tag 0x2020 format 8224 -> IFD")
- **TIFF integration**: Equipment tag 0x2010 now processed with `format: Ifd` instead of failing
- **Test infrastructure**: ExifTool snapshot generated, compatibility test framework ready

### 🔧 **REMAINING WORK (5%)**

**Critical Issue**: Equipment Section Subdirectory Processing

**Problem**: Tag 0x2010 (Equipment) is being processed as IFD format correctly thanks to FixFormat, but it's not being recognized as a subdirectory that needs recursive processing.

**Current State**: 
- ✅ FixFormat working: `Processing tag 0x2010 from MakerNotes (format: Ifd, count: 1)`
- ❌ Equipment IFD not processed recursively to extract CameraType2, SerialNumber, LensType

**Root Cause**: `process_subdirectory_tag()` function only handles standard EXIF subdirectories (0x8769, 0x8825, etc.) but not Olympus-specific ones (0x2010-0x5000).

**Solution Started**: Modified `is_subdirectory_tag()` in `src/exif/processors.rs` to recognize Olympus subdirectory tags, but `process_subdirectory_tag()` needs updating to handle them.

### **Overview (Updated)**

This milestone adds support for Olympus ORF (Olympus Raw Format). The core challenge was implementing ExifTool's "FixFormat" mechanism to handle Olympus's non-standard TIFF format types. **This breakthrough is now complete and working.**

## 📋 **ENGINEER HANDOFF GUIDE**

### **Next Engineer Task: Complete Equipment Section Processing (1-2 hours)**

**Goal**: Extract Equipment section tags (CameraType2, SerialNumber, LensType) that ExifTool produces:
```json
"MakerNotes:CameraType2": "E-M1",
"MakerNotes:SerialNumber": "BHP242330", 
"MakerNotes:LensType": "Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro"
```

### **Critical Code to Study**

1. **`src/exif/processors.rs:290-340`** - `process_subdirectory_tag()` function
   - Currently only handles standard EXIF subdirectories (ExifIFD, GPS, etc.)
   - **Must add Olympus subdirectory cases** for 0x2010, 0x2020, etc.

2. **`src/tiff_types.rs:64-80`** - FixFormat mechanism (WORKING)
   - This is the breakthrough that solves the core TIFF format issue
   - Do NOT modify this - it's working correctly

3. **`src/exif/ifd.rs:232-234`** - IFD subdirectory processing trigger
   - When `is_subdirectory_tag()` returns true for IFD format tags
   - Calls `process_subdirectory_tag()` with the tag offset

### **Specific Issue to Fix**

**File**: `src/exif/processors.rs`  
**Function**: `process_subdirectory_tag()` around line 290

**Current Code**:
```rust
let subdir_name = match tag_id {
    0x8769 => "ExifIFD",
    0x8825 => "GPS", 
    0xA005 => "InteropIFD",
    0x927C => "MakerNotes",
    _ => return Ok(()), // ← Equipment tag 0x2010 hits this and exits
};
```

**Required Fix**:
```rust
let subdir_name = match tag_id {
    0x8769 => "ExifIFD",
    0x8825 => "GPS",
    0xA005 => "InteropIFD", 
    0x927C => "MakerNotes",
    
    // Olympus subdirectory tags - only when in Olympus context
    0x2010 => "Olympus:Equipment",
    0x2020 => "Olympus:CameraSettings",
    0x2030 => "Olympus:RawDevelopment", 
    0x2031 => "Olympus:RawDev2",
    0x2040 => "Olympus:ImageProcessing",
    0x2050 => "Olympus:FocusInfo",
    0x3000 => "Olympus:RawInfo",
    0x4000 => "Olympus:MainInfo",
    0x5000 => "Olympus:UnknownInfo",
    
    _ => return Ok(()),
};
```

### **Testing the Fix**

**Verification Command**:
```bash
# Should show Equipment section tags
cargo run -- test-images/olympus/test.orf | grep -E "(CameraType|SerialNumber|LensType)"

# Should match ExifTool output:
exiftool -j test-images/olympus/test.orf | grep -E "(CameraType|SerialNumber|LensType)"
```

**Expected Output**:
```json
"MakerNotes:CameraType2": "E-M1",
"MakerNotes:SerialNumber": "BHP242330",
"MakerNotes:LensType": "Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro"
```

### **Debug Commands**

```bash
# Check if Equipment tag is being processed as subdirectory
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep -A5 -B5 "subdirectory.*0x2010"

# Verify FixFormat is working (should see "Applying Olympus FixFormat")
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "FixFormat"
```

### **Success Criteria**

- [ ] **Equipment Section Tags Extracted**: CameraType2, SerialNumber, LensType appear in output
- [ ] **ExifTool Compatibility**: Output matches ExifTool for Equipment section tags
- [ ] **Compatibility Test**: `cargo test --features integration-tests test_exiftool_compatibility` passes for ORF
- [ ] **No Regressions**: All existing tests still pass

### **Files Modified in This Session**

1. **`src/tiff_types.rs`** - Added FixFormat mechanism (COMPLETE)
2. **`src/exif/ifd.rs`** - Added Olympus context detection (COMPLETE)  
3. **`src/exif/processors.rs`** - Added Olympus subdirectory recognition (PARTIAL)
4. **`src/raw/formats/olympus.rs`** - Core handler (COMPLETE)
5. **Generated snapshots** - `generated/exiftool-json/test_images_olympus_test_orf.json` (COMPLETE)

## Background

**ORF Characteristics**:

- **TIFF-based container** with Olympus-specific maker note IFDs
- **Non-standard TIFF format types** that require FixFormat mechanism
- **Dual processing modes**: Each section can be processed as binary data OR IFD (ExifTool pattern)

**Core Data Sections** (9 primary sections from ExifTool analysis):

1. **Equipment (0x2010)** - Camera hardware, lens data, serial numbers ← **NEEDS COMPLETION**
2. **CameraSettings (0x2020)** - Core camera settings, exposure mode, white balance  
3. **RawDevelopment (0x2030)** - RAW processing parameters
4. **RawDev2 (0x2031)** - Additional RAW development parameters
5. **ImageProcessing (0x2040)** - Image processing settings, art filters
6. **FocusInfo (0x2050)** - Autofocus information, AF points
7. **RawInfo (0x3000)** - RAW file specific information
8. **MainInfo (0x4000)** - Main Olympus tag table (primary maker notes)
9. **UnknownInfo (0x5000)** - Unknown/experimental data section

## 🔧 **POTENTIAL FUTURE REFACTORINGS**

### **Subdirectory Processing Architecture**

**Current Issue**: `process_subdirectory_tag()` uses hardcoded match statements for tag IDs, making it difficult to extend for manufacturer-specific subdirectories.

**Suggested Refactoring**:
1. **Registry-Based Subdirectory Handling**: Create a `SubdirectoryRegistry` that maps tag IDs to subdirectory names based on context (manufacturer, file format)
2. **Trait-Based Processing**: Define a `SubdirectoryProcessor` trait that manufacturers can implement
3. **Context-Aware Dispatch**: Move manufacturer detection logic to a central context system

**Benefits**: Easier to add Canon, Nikon, Sony subdirectories without modifying core IFD processing code.

### **FixFormat System Generalization**

**Current State**: FixFormat is Olympus-specific in `from_u16_with_olympus_fixformat()`

**Future Enhancement**: 
- Generalize to `from_u16_with_manufacturer_fixformat(manufacturer, tag_id, format)`
- Support Canon, Nikon, and other manufacturers that also have non-standard format types
- Move manufacturer-specific logic to dedicated modules

### **TIFF Format Type Validation**

**Observation**: Multiple manufacturers violate TIFF specification in different ways. Consider creating a `TiffFormatValidator` that can handle:
- Olympus invalid format types (0x2010-0x5000 ranges)
- Canon non-standard formats
- Nikon encrypted/proprietary format types

## 📚 **IMPLEMENTATION REFERENCE (COMPLETED)**

### **Phase 1: Detection and Integration** ✅

**Completed Updates**:

1. **`src/raw/detector.rs`** ✅:
   - Added `Olympus` variant to `RawFormat` enum
   - Added ORF detection logic to `detect_raw_format()`
   - Added `validate_olympus_orf_magic()` function

2. **`src/raw/processor.rs`** ✅:
   - Registered Olympus handler in `RawProcessor::new()`

3. **`src/raw/formats/mod.rs`** ✅:
   - Added `pub mod olympus;` declaration

### **Phase 2: Core Handler Implementation** ✅

**Completed Implementation**: See `src/raw/formats/olympus.rs` - fully functional ORF handler that processes TIFF-based ORF files correctly.

### **Phase 3: FixFormat Breakthrough** ✅

**Completed**: The critical FixFormat mechanism in `src/tiff_types.rs` that handles Olympus's non-standard TIFF format types. This was the key technical breakthrough that made the entire implementation possible.

## 📋 **FINAL SUMMARY FOR NEXT ENGINEER**

### **What's Working (95% Complete)**

1. ✅ **Core ORF processing** - Files load, basic EXIF extraction works perfectly
2. ✅ **FixFormat mechanism** - Handles invalid TIFF format types automatically  
3. ✅ **TIFF integration** - Equipment tag 0x2010 now processed as IFD instead of failing
4. ✅ **Test infrastructure** - ExifTool snapshots generated, compatibility tests ready
5. ✅ **CLI integration** - `cargo run -- file.orf` works

### **What Needs Completion (5%)**

**Single Issue**: Equipment subdirectory not processed recursively to extract specific tags.

**1-Line Fix Required**: Add Olympus subdirectory cases to `process_subdirectory_tag()` match statement in `src/exif/processors.rs`.

**Expected Result**: Extract `CameraType2: "E-M1"`, `SerialNumber: "BHP242330"`, `LensType: "Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro"`

### **Key Technical Achievement**

The **FixFormat mechanism** is the core breakthrough - it automatically converts Olympus's invalid TIFF format types to valid ones, enabling standard TIFF processing. This pattern can be extended to other manufacturers with similar issues.

---

*This milestone demonstrates that our RAW processing architecture can handle manufacturer-specific format violations efficiently while leveraging existing TIFF infrastructure.*