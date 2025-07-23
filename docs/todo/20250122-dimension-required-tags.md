# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all image dimension tags required by PhotoStructure are properly extracted

## Background & Context

- Critical as image metadata

## Technical Foundation

Study the entirety of the documentation, and study referenced relevant source code.

- [CLAUDE.md](CLAUDE.md)
- [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) -- follow their dimension extraction algorithm **precisely**.
- [CODEGEN.md](docs/CODEGEN.md) -- if there's any tabular data, or perl code that you think we could automatically extract and use, **strongly** prefer that to any manual porting effort.

- **Key files**:
  - `src/file_metadata.rs` - File system metadata extraction

## Required File System Tags (15 total)

### Core Media Properties (8 tags)
- **ImageWidth** 
- **ImageHeight**

### Nice-to-have (4 tags)
- **BitsPerSample**
- **ColorComponents**
  
## Not really needed

- **YCbCrSubSampling**
- **EncodingProcess**

## Remaining Tasks

### High Priority - Core Implementation

1. **Image Dimensions from File**
   - Extract width/height without full EXIF parse
   - Read from JPEG SOF markers
   - Handle RAW format dimensions
   - Must work even if EXIF corrupted

## Testing Strategy

- Compare with ExifTool file metadata

## Success Criteria

- All file tags extracting correctly
- Cross-platform compatibility
- Consistent timestamp formatting
- Proper error handling for missing data
- Image dimensions extracted even without valid EXIF

## Gotchas & Tribal Knowledge

### Image Dimensions
- **JPEG**: Read from SOF0/SOF2 markers, not EXIF
- **RAW**: May need format-specific parsing
- **Orientation**: File dimensions are pre-rotation values

### Special Cases
- **ExifByteOrder**: "II" = Little-endian (Intel), "MM" = Big-endian (Motorola)