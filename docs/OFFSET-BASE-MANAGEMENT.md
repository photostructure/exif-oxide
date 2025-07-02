# TODO: Offset Base Management Design for exif-oxide

## Problem Statement

ExifTool has a sophisticated offset calculation system that handles various base offset schemes used by different manufacturers. We need a design that can handle this complexity without introducing novel parsing logic.

## Research Summary: ExifTool's Offset Management Architecture

Based on comprehensive analysis of ExifTool source code, documentation, and architectural patterns, here's what we discovered:

### 1. Core Directory Info Structure

ExifTool's offset management centers around the **directory information hash** passed to all PROCESS_PROC functions. From `lib/Image/ExifTool/README:42-59` and actual implementation in `Exif.pm:6169+`:

**Key State Variables**:

- **`Base`**: Base offset for all pointers in directory (usually TIFF header position)
- **`DataPos`**: File position of data block containing directory
- **`DirStart`**: Offset to directory start within data block
- **`DataPt`**: Reference to in-memory data block
- **`DataLen`**: Length of data block

**Critical Formula**: `absolute_file_offset = Base + DataPos + relative_offset`

### 2. Multiple Offset Calculation Schemes

From analysis of `ProcessExif` and `MakerNotes.pm`:

#### Standard EXIF/TIFF (Most Common)

```perl
# Exif.pm:6431 - Standard offset calculation
$valuePtr -= $dataPos;  # Convert to data block relative
```

Offsets relative to TIFF header (Base = 0 for main EXIF)

#### Entry-Based Offsets (Panasonic, some others)

```perl
# Exif.pm:6426-6430 - Entry-based detection
if ($$dirInfo{EntryBased} or $$tagTablePtr{$tagID}{EntryBased}) {
    $valuePtr += $entry;  # Add IFD entry position
}
```

Offsets relative to each 12-byte IFD entry position

#### Maker Note Base Fixing

```perl
# Exif.pm:6288-6295 - Automatic base fixing
if (defined $$dirInfo{MakerNoteAddr}) {
    if (Image::ExifTool::MakerNotes::FixBase($et, $dirInfo)) {
        $base = $$dirInfo{Base};      # Updated base
        $dataPos = $$dirInfo{DataPos}; # Updated data position
    }
}
```

### 3. Sophisticated FixBase Algorithm

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

### 4. Special Cases and Edge Cases

#### Dynamic Base Expressions

```perl
# SubDirectory Base can be Perl expressions
Base => '$start + $base - 8',    # Leica M8
Base => '$start',                # Relative to subdirectory
```

#### Tag-Specific Overrides

```perl
# Individual tags can override base calculation
ChangeBase => '$dirStart + $dataPos - 8',  # Leica preview quirk
WrongBase => '$self->{SomeCondition} ? 8 : 0',
```

#### Format-Specific Quirks

- **EntryBased flag**: Per-tag or per-directory
- **FixOffsets expressions**: Runtime offset patching
- **RelativeBase**: Force IFD-relative addressing
- **AutoFix**: Apply corrections without warnings

## Architectural Recommendations for exif-oxide

### Recommendation: Layered Offset Management System

Based on ExifTool's proven architecture and exif-oxide's manual implementation philosophy:

#### 1. Core Offset Context Structure

```rust
/// Directory processing context - mirrors ExifTool's dirInfo hash
#[derive(Debug, Clone)]
pub struct DirectoryContext {
    /// Base offset for all pointers (usually TIFF header position)
    pub base: u32,

    /// File position of data block containing directory
    pub data_pos: u32,

    /// Offset to directory start within data block
    pub dir_start: u32,

    /// Reference to data block
    pub data: &[u8],

    /// Current byte order
    pub byte_order: ByteOrder,

    /// Entry-based addressing flag
    pub entry_based: bool,

    /// Manufacturer-specific offset fixing state
    pub offset_info: OffsetFixingInfo,
}

impl DirectoryContext {
    /// Calculate absolute file offset from relative offset
    /// Core formula: absolute = base + data_pos + relative
    pub fn absolute_offset(&self, relative_offset: u32) -> u32 {
        self.base + self.data_pos + relative_offset
    }

    /// Calculate value pointer for IFD entry
    pub fn value_pointer(&self, value_offset: u32, entry_index: u16) -> u32 {
        if self.entry_based {
            // Entry-based: offset relative to IFD entry position
            value_offset + (entry_index as u32 * 12)
        } else {
            // Standard: offset relative to data block start
            value_offset
        }
    }
}
```

#### 2. Manufacturer-Specific Offset Managers

Manual implementations following exif-oxide's pattern:

```rust
/// Canon-specific offset management
/// ExifTool reference: Canon.pm, MakerNotes.pm GetMakerNoteOffset
pub mod canon {
    pub fn detect_offset_scheme(model: &str) -> CanonOffsetScheme {
        match model {
            m if m.contains("20D") || m.contains("350D") => CanonOffsetScheme::SixByte,
            m if m.contains("PowerShot") => CanonOffsetScheme::SixteenByte,
            m if m.contains("FV") || m.contains("OPTURA") => CanonOffsetScheme::TwentyEightByte,
            _ => CanonOffsetScheme::FourByte,
        }
    }

    pub fn fix_maker_note_base(
        ctx: &mut DirectoryContext,
        maker_note_data: &[u8],
        scheme: CanonOffsetScheme,
    ) -> Result<(), ExifError> {
        // Manual port of Canon-specific FixBase logic
        // References: MakerNotes.pm:1257-1459

        // 1. Validate TIFF footer
        if let Some(footer_offset) = validate_canon_footer(maker_note_data)? {
            ctx.base = footer_offset;
            return Ok(());
        }

        // 2. Apply scheme-specific offset
        let expected_offset = match scheme {
            CanonOffsetScheme::FourByte => 4,
            CanonOffsetScheme::SixByte => 6,
            CanonOffsetScheme::SixteenByte => 16,
            CanonOffsetScheme::TwentyEightByte => 28,
        };

        // 3. Validate and adjust
        if validate_offset_scheme(ctx, expected_offset)? {
            ctx.base += expected_offset;
        }

        Ok(())
    }
}

/// Nikon-specific offset management
pub mod nikon {
    pub fn process_encrypted_section(/* ... */) { /* ... */ }
    pub fn fix_nikon_base(/* ... */) { /* ... */ }
}
```

#### 3. Offset Validation System

```rust
/// Offset validation following ExifTool's approach
pub struct OffsetValidator {
    file_size: u32,
    processed_ranges: Vec<(u32, u32)>, // (start, length) pairs
}

impl OffsetValidator {
    pub fn validate_value_offset(
        &mut self,
        ctx: &DirectoryContext,
        offset: u32,
        size: u32,
        tag_name: &str,
    ) -> Result<(), ExifError> {
        let absolute_offset = ctx.absolute_offset(offset);

        // ExifTool validation patterns from Exif.pm:6390+

        // 1. Check file bounds
        if absolute_offset + size > self.file_size {
            return Err(ExifError::invalid_offset(tag_name, absolute_offset));
        }

        // 2. Check for overlaps with directory
        let dir_start = ctx.absolute_offset(ctx.dir_start);
        let dir_end = dir_start + calculate_ifd_size(ctx.data, ctx.dir_start)?;

        if offset_ranges_overlap(absolute_offset, size, dir_start, dir_end - dir_start) {
            return Err(ExifError::minor(format!("Value for {} overlaps IFD", tag_name)));
        }

        // 3. Check for overlaps with previous values
        for (prev_start, prev_size) in &self.processed_ranges {
            if offset_ranges_overlap(absolute_offset, size, *prev_start, *prev_size) {
                return Err(ExifError::minor(format!("Value for {} overlaps previous value", tag_name)));
            }
        }

        self.processed_ranges.push((absolute_offset, size));
        Ok(())
    }
}
```

#### 4. Expression Evaluation Strategy

Instead of eval(), use pattern matching:

```rust
/// Base offset calculation patterns from ExifTool SubDirectory definitions
#[derive(Debug, Clone)]
pub enum BaseCalculation {
    /// Standard: same as parent directory
    Inherit,

    /// Fixed offset: Base = parent_base + offset
    Fixed(i32),

    /// Relative to subdirectory start: Base = subdirectory_start
    RelativeToStart,

    /// Relative to parent: Base = parent_base + subdirectory_start + offset
    RelativeToParent(i32),

    /// Manufacturer-specific calculation
    Custom(CustomBaseCalc),
}

#[derive(Debug, Clone)]
pub enum CustomBaseCalc {
    /// Leica M8: different for DNG vs JPEG
    LeicaM8,

    /// Canon: use TIFF footer
    CanonFooter,

    /// Nikon: TIFF header at offset 0x0a
    NikonTiffHeader,
}

impl BaseCalculation {
    pub fn calculate(
        &self,
        parent_base: u32,
        subdirectory_start: u32,
        context: &ProcessingContext,
    ) -> Result<u32, ExifError> {
        match self {
            Self::Inherit => Ok(parent_base),
            Self::Fixed(offset) => Ok((parent_base as i32 + offset) as u32),
            Self::RelativeToStart => Ok(subdirectory_start),
            Self::RelativeToParent(offset) => {
                Ok((parent_base as i32 + subdirectory_start as i32 + offset) as u32)
            }
            Self::Custom(calc) => calc.calculate(parent_base, subdirectory_start, context),
        }
    }
}
```

### Implementation Strategy

#### Phase 1: Core Infrastructure

1. Implement `DirectoryContext` with basic offset calculations
2. Create `OffsetValidator` with ExifTool's validation patterns
3. Build foundation for manufacturer-specific managers

#### Phase 2: Manufacturer-Specific Implementations

1. **Canon**: Complete offset scheme detection and footer validation
2. **Nikon**: TIFF header handling and encryption support
3. **Sony**: Multi-format detection
4. **Others**: Add as needed based on test image requirements

#### Phase 3: Advanced Features

1. Entry-based offset detection and handling
2. Complex base calculation patterns
3. Write support with offset fixup

### Key Design Principles

1. **Manual Excellence**: Each manufacturer's quirks manually implemented with ExifTool source references
2. **No Dynamic Evaluation**: Replace Perl eval() with pattern matching and manual logic
3. **Layered Validation**: Multiple validation levels following ExifTool's approach
4. **State Isolation**: Clear separation between global and directory-specific state
5. **Error Classification**: Mirror ExifTool's minor vs. fatal error handling

### Integration with ARCHITECTURE.md

This offset management system integrates with the planned architecture:

- **Implementations Palette**: Manufacturer offset managers indexed by patterns
- **Registry System**: Map maker note headers to offset fixing strategies
- **Runtime Fallback**: Graceful degradation when offset fixing fails
- **TODO Tracking**: Log missing offset implementations by manufacturer

The offset management system becomes a core part of the implementation palette, with each manufacturer's offset quirks treated as manual implementations that reference specific ExifTool source locations.

## Conclusion

ExifTool's offset management represents 20+ years of handling real-world camera firmware quirks. The recommended approach for exif-oxide:

1. **Embrace the Complexity**: Don't try to simplify - port ExifTool's proven patterns
2. **Manual Implementation**: Each manufacturer's offset scheme is manually coded
3. **Layered Architecture**: Context → Validation → Manufacturer-specific logic
4. **Reference Everything**: Every quirk includes ExifTool source references
5. **Test Extensively**: Offset bugs are subtle and data-corrupting

This approach maintains ExifTool compatibility while providing the performance and safety benefits of Rust, following exif-oxide's core philosophy of manual excellence over automatic generation.
