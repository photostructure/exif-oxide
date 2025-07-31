# Core Architecture: State Management & Offset Calculations

**🚨 CRITICAL: This document builds on [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - the fundamental law of this project.**

This guide covers the two most critical aspects of exif-oxide's internal architecture: how we manage state during processing and how we handle the complex offset calculations that make metadata extraction work. Both systems faithfully reproduce ExifTool's proven patterns.

**📋 Document Status**: This document describes both implemented features in the current codebase and planned enhancements. Implementation status is marked throughout with ✅ (implemented), 🔄 (in progress), or 🔮 (planned).

## Section 1: State Management Patterns

> **Foundation:** ExifTool's stateful approach is proven for complex nested metadata. We translate this exactly per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### 1.1 Research Summary

After extensive analysis of ExifTool's source code, I've identified the critical state management patterns and can now provide architectural recommendations for the Rust implementation.

### 1.2 Key State Components in ExifTool (Research Findings)

#### PROCESSED Hash - Infinite Loop Prevention

- **Location**: `lib/Image/ExifTool.pm:4279, 8964-8970`
- **Purpose**: Prevents infinite loops in circular directory references
- **Implementation**:

  ```perl
  $$self{PROCESSED} = { };  # init
  my $addr = $$dirInfo{DirStart} + $$dirInfo{DataPos} + ($$dirInfo{Base}||0) + $$self{BASE};
  $$self{PROCESSED}{$addr} = $dirName unless $$tagTablePtr{VARS}{ALLOW_REPROCESS};
  ```

- **Address calculation**: Combines DirStart + DataPos + Base + global BASE offset
- **Critical for**: Maker notes that reference other sections

#### VALUE Hash - Extracted Tag Storage

- **Location**: `lib/Image/ExifTool.pm:4273, 9356, 9527-9530`
- **Purpose**: Stores all extracted tag values indexed by tag key
- **Implementation**:

  ```perl
  $$self{VALUE} = { };  # initialization
  $$self{VALUE}{$vtag} = $$self{VALUE}{$tag};  # duplicate handling
  ```

- **Features**: Supports duplicate tag handling, deletion, and metadata association

#### DataMember Dependencies - Complex Interdependencies

- **Location**: Multiple locations, key processing in ProcessBinaryData and ProcessSerialData
- **Purpose**: Earlier tags determine format/count/behavior of later tags
- **Examples**:
  - Canon AF data: `NumAFPoints` determines array sizes for AF positions
  - Format expressions: `int16s[$val{0}]` where `$val{0}` is from tag 0
  - Conditional extraction: Tags only extracted if dependency conditions met
- **Resolution Strategy**: Sequential processing with `%val` hash accumulating values

#### Directory Processing Context - Nested State Management

- **Location**: `lib/Image/ExifTool.pm:4287, 8977, 8985, 7159-7161`
- **Components**:
  - **PATH stack**: `$$self{PATH}` tracks current directory hierarchy
  - **Directory Info**: Hash with Base, DataPos, DirStart, DirLen
  - **Base calculations**: Complex offset arithmetic for nested structures
- **State transitions**: Push/pop on directory entry/exit

### 1.3 Current Implementation Analysis

#### Stateful Reader Object Pattern ✅ **IMPLEMENTED**

**Current Implementation**: We have a stateful `ExifReader` object in `src/exif/mod.rs:31-67` that closely mirrors ExifTool's `$self`

```rust
// Current implementation in src/exif/mod.rs:36-79
pub struct ExifReader {
    // Core state - equivalent to ExifTool's member variables
    pub(crate) extracted_tags: HashMap<(u16, String), TagValue>, // ✅ VALUE hash with namespace
    pub(crate) tag_sources: HashMap<(u16, String), TagSourceInfo>, // ✅ Enhanced conflict resolution
    pub(crate) header: Option<TiffHeader>,                  // ✅ TIFF header with byte order
    pub(crate) data: Vec<u8>,                              // ✅ Raw EXIF data buffer
    pub(crate) warnings: Vec<String>,                       // ✅ Parse errors (non-fatal)

    // Stateful processing features
    pub(crate) processed: HashMap<u64, String>,            // ✅ PROCESSED hash (ExifTool $$self{PROCESSED})
    pub(crate) path: Vec<String>,                          // ✅ PATH stack (ExifTool $$self{PATH})
    pub(crate) data_members: HashMap<String, DataMemberValue>, // ✅ DataMember storage
    pub(crate) base: u64,                                  // ✅ Current base offset
    pub(crate) processor_dispatch: ProcessorDispatch,      // ✅ PROCESS_PROC system
    pub(crate) maker_notes_original_offset: Option<usize>, // ✅ MakerNotes offset tracking
    pub(crate) composite_tags: HashMap<String, TagValue>,  // ✅ Composite tag computation
    pub(crate) original_file_type: Option<String>,         // ✅ File type detection
    pub(crate) overridden_file_type: Option<String>,       // ✅ Content-based override
    pub(crate) synthetic_tag_names: HashMap<u16, String>,  // ✅ Synthetic tag ID mapping
}
```

**Implementation Analysis**:

- ✅ **Fully implemented** - ExifTool's stateful approach with memory safety
- ✅ **Namespace-aware storage** - Tags stored with (tag_id, namespace) keys for conflict resolution
- ✅ **Enhanced source tracking** - TagSourceInfo includes namespace, IFD name, and processor context
- 📍 **Reference** - See `src/exif/mod.rs:88-107` for initialization logic

#### DataMember Resolution Strategy ✅ **IMPLEMENTED**

**Current Implementation**: Sequential processing with dependency tracking exists in the processor system

```rust
// Current DataMember support in ExifReader (src/exif/mod.rs:59)
pub struct ExifReader {
    // ...
    pub(crate) data_members: HashMap<String, DataMemberValue>,  // ✅ Implemented
    // ...
}

// The data_members field stores resolved values for use by dependent tags.
// Specific processing logic is handled by the processor dispatch system
// and subdirectory processing modules.
```

**Implementation Status**:

- ✅ **DataMember storage** - HashMap for storing resolved dependency values
- ✅ **Processor dispatch** - System for handling different directory types
- 📍 **Reference** - See `src/exif/binary_data.rs` for binary data processing
- 📍 **Reference** - See `src/exif/subdirectory_processing.rs` for subdirectory implementations

**Note**: The specific two-phase processing pattern shown in the original proposal represents an approach that could be implemented within the existing processor framework as needed for specific manufacturers requiring complex DataMember dependencies.

#### State Isolation Strategy ✅ **IMPLEMENTED**

**Current Implementation**: Shared mutable state with controlled access through method boundaries

```rust
impl ExifReader {
    fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        // Calculate unique address for recursion prevention
        let addr = self.calculate_directory_address(dir_info);

        // Check for infinite loops
        if let Some(prev_dir) = self.processed.get(&addr) {
            return Err(ExifError::circular_reference(prev_dir, &dir_info.name));
        }

        // Enter subdirectory context
        self.path.push(dir_info.name.clone());
        self.processed.insert(addr, dir_info.name.clone());

        // Process with current context
        let result = self.process_directory_contents(dir_info);

        // Exit subdirectory context
        self.path.pop();
        if !dir_info.allow_reprocess {
            // Keep PROCESSED entry for recursion prevention
        }

        result
    }

    fn calculate_directory_address(&self, dir_info: &DirectoryInfo) -> u64 {
        // Equivalent to ExifTool's addr calculation
        dir_info.dir_start + dir_info.data_pos + dir_info.base + self.base
    }
}
```

**Benefits**:

- **Compatible behavior**: Matches ExifTool's recursion prevention
- **Context preservation**: PATH and offset state properly managed
- **Memory safety**: Rust ownership prevents dangling references

#### Thread Safety Approach ✅ **IMPLEMENTED**

**Current Implementation**: Single-threaded per Reader, thread-safe for multiple readers

```rust
// Current ExifReader design (src/exif/mod.rs:88-107)
impl ExifReader {
    pub fn new() -> Self {
        Self {
            extracted_tags: HashMap::new(),
            tag_sources: HashMap::new(),
            header: None,
            data: Vec::new(),
            warnings: Vec::new(),
            // Milestone 5: Initialize stateful features
            processed: HashMap::new(),
            path: Vec::new(),
            data_members: HashMap::new(),
            base: 0,
            processor_dispatch: ProcessorDispatch::default(),
            maker_notes_original_offset: None,
            composite_tags: HashMap::new(),
            original_file_type: None,
            overridden_file_type: None,
            synthetic_tag_names: HashMap::new(),
        }
    }
}

// Thread safety is achieved through the high-level API in src/formats/mod.rs
// Each call to extract_metadata creates its own processing context
```

**Implementation Notes**:

- ✅ **Single-threaded design** - Each ExifReader is stateful and not thread-safe
- ✅ **Parallel processing** - Achieved by creating separate ExifReader instances
- 📍 **API Reference** - See `src/formats/mod.rs:63` for the public `extract_metadata` function
- 📍 **Usage Pattern** - Each file processing gets its own reader instance

## Section 2: Offset Calculation Systems

> **Critical:** ExifTool's offset management represents 20+ years of handling real-world camera firmware quirks. Per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), we embrace this complexity rather than simplify it.

### 2.1 Problem Statement

ExifTool has a sophisticated offset calculation system that handles various base offset schemes used by different manufacturers. We need a design that can handle this complexity without introducing novel parsing logic.

### 2.2 Implementation Approach

Offset management is implemented incrementally through manufacturer-specific milestones. Simple schemes suffice for some manufacturers (like Nikon), while others require sophisticated base fixing algorithms.

### 2.3 Research Summary: ExifTool's Offset Management Architecture

Based on comprehensive analysis of ExifTool source code, documentation, and architectural patterns, here's what we discovered:

#### Core Directory Info Structure

ExifTool's offset management centers around the **directory information hash** passed to all PROCESS_PROC functions. From `lib/Image/ExifTool/README:42-59` and actual implementation in `Exif.pm:6169+`:

**Key State Variables**:

- **`Base`**: Base offset for all pointers in directory (usually TIFF header position)
- **`DataPos`**: File position of data block containing directory
- **`DirStart`**: Offset to directory start within data block
- **`DataPt`**: Reference to in-memory data block
- **`DataLen`**: Length of data block

**Critical Formula**: `absolute_file_offset = Base + DataPos + relative_offset`

#### Multiple Offset Calculation Schemes

From analysis of `ProcessExif` and `MakerNotes.pm`:

##### Standard EXIF/TIFF (Most Common)

```perl
# Exif.pm:6431 - Standard offset calculation
$valuePtr -= $dataPos;  # Convert to data block relative
```

Offsets relative to TIFF header (Base = 0 for main EXIF)

##### Entry-Based Offsets (Panasonic, some others)

```perl
# Exif.pm:6426-6430 - Entry-based detection
if ($$dirInfo{EntryBased} or $$tagTablePtr{$tagID}{EntryBased}) {
    $valuePtr += $entry;  # Add IFD entry position
}
```

Offsets relative to each 12-byte IFD entry position

##### Maker Note Base Fixing

```perl
# Exif.pm:6288-6295 - Automatic base fixing
if (defined $$dirInfo{MakerNoteAddr}) {
    if (Image::ExifTool::MakerNotes::FixBase($et, $dirInfo)) {
        $base = $$dirInfo{Base};      # Updated base
        $dataPos = $$dirInfo{DataPos}; # Updated data position
    }
}
```

#### Sophisticated FixBase Algorithm

From `MAKERNOTE.md` and source analysis, ExifTool's `FixBase()` implements:

1. **Offset Expectation Calculation**: Based on manufacturer and model
2. **Directory Validation**: Entry count, format validation
3. **Value Block Analysis**: Detect overlapping values, negative gaps
4. **Entry-Based Detection**: Look for -12 byte gaps (12 = IFD entry size)
5. **Automatic Correction**: Adjust Base and DataPos

**Manufacturer-Specific Patterns**:

- **Canon**: 4, 6, 16, or 28 byte offsets depending on model
- **Nikon**: TIFF header at offset 0x0a from maker note start
- **Sony**: Offset 0 or 4 depending on model era
- **Leica**: 9 different maker note formats!

#### Special Cases and Edge Cases

##### Dynamic Base Expressions

```perl
# SubDirectory Base can be Perl expressions
Base => '$start + $base - 8',    # Leica M8
Base => '$start',                # Relative to subdirectory
```

##### Tag-Specific Overrides

```perl
# Individual tags can override base calculation
ChangeBase => '$dirStart + $dataPos - 8',  # Leica preview quirk
WrongBase => '$self->{SomeCondition} ? 8 : 0',
```

##### Format-Specific Quirks

- **EntryBased flag**: Per-tag or per-directory
- **FixOffsets expressions**: Runtime offset patching
- **RelativeBase**: Force IFD-relative addressing
- **AutoFix**: Apply corrections without warnings

### 2.4 Current Implementation Analysis for exif-oxide

#### Implementation Status: Foundational Offset Management

Based on ExifTool's proven architecture, our current implementation provides:

##### Current Offset Management ✅ **PARTIALLY IMPLEMENTED**

```rust
// Current DirectoryInfo structure (src/types/metadata.rs)
pub struct DirectoryInfo {
    pub name: String,        // ✅ Directory name for debugging and PATH tracking
    pub dir_start: usize,    // ✅ Start offset of directory within data
    pub dir_len: usize,      // ✅ Length of directory data
    pub base: u64,          // ✅ Base offset for pointer calculations (ExifTool's Base)
    pub data_pos: u64,      // ✅ File position of data block (ExifTool's DataPos)
    pub allow_reprocess: bool, // ✅ Whether this directory allows reprocessing
}

// Current ExifReader offset tracking (src/exif/mod.rs:55)
pub struct ExifReader {
    pub(crate) base: u64,              // ✅ Current base offset tracking
    pub(crate) header: Option<TiffHeader>, // ✅ Includes byte order
    // ...
}

// TIFF header management (src/tiff_types.rs)
pub struct TiffHeader {
    pub byte_order: ByteOrder, // ✅ Byte order tracking
    pub magic: u16,           // ✅ TIFF validation (42 for TIFF, 85 for RW2)
    pub ifd0_offset: u32,     // ✅ Offset to first IFD
}
```

**Implementation Status**:

- ✅ **Basic offset tracking** - DirectoryInfo and base offset management
- ✅ **TIFF header handling** - Byte order and validation
- ✅ **Endianness support** - Throughout the parsing pipeline
- 🔄 **Advanced manufacturer-specific offset fixing** - Planned for manufacturer-specific milestones
- 📍 **Reference** - See `src/tiff_types.rs` for TIFF structures

##### Manufacturer-Specific Offset Managers 🔄 **PLANNED**

Manufacturer-specific offset handling is implemented through dedicated modules:

- **Canon**: 4, 6, 16, or 28 byte offsets depending on model
- **Nikon**: TIFF header at offset 0x0a from maker note start
- **Sony**: Offset 0 or 4 depending on model era
- **Others**: Added incrementally based on milestone requirements

##### Offset Validation System 🔄 **PLANNED**

Offset validation following ExifTool's approach:

- File bounds checking
- Directory overlap detection
- Value overlap prevention
- Absolute offset calculations

##### Expression Evaluation Strategy 🔄 **PLANNED**

Base offset calculation patterns replace Perl eval() with pattern matching:

- **Inherit**: Use parent directory base
- **Fixed**: Parent base + fixed offset
- **RelativeToStart**: Relative to subdirectory start
- **Custom**: Manufacturer-specific calculations (Canon footer, Nikon TIFF header, etc.)

#### Implementation Strategy and Current Status

##### Phase 1: Core Infrastructure ✅ **COMPLETED**

1. ✅ `DirectoryInfo` with basic offset calculations - Implemented in `src/types/`
2. ✅ Basic validation patterns - Integrated in EXIF processing
3. ✅ Foundation for manufacturer-specific managers - Processor dispatch system

##### Phase 2: Manufacturer-Specific Implementations 🔄 **IN PROGRESS**

1. **Canon**: Planned for Canon-specific milestone
2. **Nikon**: Basic support exists, encryption support planned
3. **Sony**: Planned for Sony-specific milestone
4. **Others**: Driven by milestone requirements and test coverage

##### Phase 3: Advanced Features 🔄 **PLANNED**

1. Entry-based offset detection and handling
2. Complex base calculation patterns
3. Write support with offset fixup

## Section 3: Integration Between State & Offsets

> **Synthesis:** State management and offset calculations work together to handle ExifTool's complex directory traversal patterns, following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### 3.1 How State and Offsets Interact ✅ **IMPLEMENTED**

Our current state management and offset calculation systems are integrated as follows:

```rust
// Current integration pattern in ExifReader (conceptual - specific implementations
// are distributed across src/exif/ifd.rs and src/exif/processors.rs)

impl ExifReader {
    // The ExifReader contains both state management and offset tracking:
    pub(crate) processed: HashMap<u64, String>,     // ✅ Recursion prevention
    pub(crate) path: Vec<String>,                   // ✅ Directory hierarchy
    pub(crate) base: u64,                          // ✅ Offset calculations

    // Actual subdirectory processing combines these elements
    // See src/exif/ifd.rs for IFD processing implementations
    // See src/exif/processors.rs for specific directory processors
}

// The integration happens through the processing pipeline where:
// 1. PROCESSED tracking prevents infinite loops during recursion
// 2. PATH management maintains current directory context
// 3. Base offset calculations ensure correct pointer resolution
// 4. DirectoryInfo carries the offset context between processing levels
```

**Current Implementation References**:

- 📍 **IFD Processing** - `src/exif/ifd.rs` - Core directory processing logic
- 📍 **Subdirectory Processing** - `src/exif/subdirectory_processing.rs` - Subdirectory and binary data handling
- 📍 **State Management** - `src/exif/mod.rs:52-66` - PROCESSED and PATH tracking
- 📍 **Offset Types** - `src/types/metadata.rs` - DirectoryInfo and related structures

### 3.2 Key Design Principles

Following exif-oxide's architecture principles and [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):

1. **Manual Excellence**: Each manufacturer's quirks manually implemented with ExifTool source references
2. **No Dynamic Evaluation**: Replace Perl eval() with pattern matching and manual logic
3. **Layered Validation**: Multiple validation levels following ExifTool's approach
4. **State Isolation**: Clear separation between global and directory-specific state
5. **Error Classification**: Mirror ExifTool's minor vs. fatal error handling

### 3.3 Integration with Current Architecture

This state and offset management system integrates with our current architecture:

- ✅ **Processor Dispatch**: Manufacturer-specific processors handle different directory types
- ✅ **Registry System**: Runtime registration of PrintConv/ValueConv implementations
- ✅ **Graceful Degradation**: Error handling preserves extracted data when processing fails
- ✅ **Implementation Tracking**: Missing implementations logged for milestone planning

The state and offset management systems are core parts of the ExifReader, with manufacturer-specific quirks handled through the processor dispatch system that references specific ExifTool source locations.

## Implementation Priority and Status

1. **Phase 1**: ✅ **COMPLETED** - Basic stateful reader with PROCESSED hash implemented
2. **Phase 2**: ✅ **COMPLETED** - VALUE hash equivalent and directory context management
3. **Phase 3**: ✅ **FOUNDATION** - DataMember dependency resolution infrastructure exists
4. **Phase 4**: ✅ **COMPLETED** - Comprehensive error handling and graceful degradation
5. **Phase 5**: 🔄 **IN PROGRESS** - Manufacturer-specific offset managers (milestone-driven)
6. **Phase 6**: 🔮 **PLANNED** - Advanced offset validation and fixing (write support milestone)

## Compatibility Benefits ✅ **ACHIEVED**

Our current architecture maintains behavioral compatibility with ExifTool while leveraging Rust's safety features:

- ✅ **Memory safety**: No risk of dangling pointers or use-after-free
- ✅ **Error handling**: Explicit Result types with graceful degradation vs Perl's undefined behavior
- ✅ **Performance**: Zero-cost abstractions and optimized parsing vs Perl's runtime overhead
- ✅ **Maintainability**: Strong typing and module organization vs Perl's dynamic typing

## Conclusion

Our core architecture successfully captures ExifTool's complex state management and offset calculation patterns while providing the performance and safety benefits of Rust. By following [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), we have preserved 20+ years of camera-specific knowledge while building a maintainable, type-safe system.

**Key Insight**: ExifTool's value isn't in its Perl code, but in the accumulated knowledge of metadata formats. We preserve this knowledge through careful manual translation of both state management and offset calculation patterns, implemented incrementally through our milestone-driven development approach.

**Current Status**: The foundational architecture is complete and operational, with manufacturer-specific enhancements being added milestone by milestone as we encounter specific camera requirements in our test image corpus.
