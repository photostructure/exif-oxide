# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all file system metadata tags required by PhotoStructure are properly extracted
- **Problem**: Need consistent extraction of 15 file system tags across all platforms

## Background & Context

- File system tags come from OS file metadata, not image data
- Critical for file management and tracking
- Platform-specific implementations needed (Windows/macOS/Linux)

## Technical Foundation

- **Key files**:
  - `src/file_metadata.rs` - File system metadata extraction
  - Standard library `std::fs` module
- **Platform considerations**: Unix vs Windows file attributes

## Required File System Tags (15 total)

### Core File Properties (8 tags)
- **FileName** - Base filename - freq 1.000
- **Directory** - Parent directory path - freq 1.000
- **FileSize** - Size in bytes - freq 1.000
- **FileType** - File format detection - freq 1.000
- **FileTypeExtension** - File extension - freq 1.000
- **MIMEType** - MIME type mapping - freq 1.000
- **FilePermissions** - Unix permissions or Windows attributes - freq 1.000
- **ExifByteOrder** - Byte order of EXIF data (II/MM) - freq 0.990

### Image Dimensions (2 tags)
- **ImageWidth** - Image width from file analysis - freq 1.000
- **ImageHeight** - Image height from file analysis - freq 1.000

### Timestamps (4 tags)
- **FileModifyDate** - Last modification time - freq 1.000
- **FileAccessDate** - Last access time - freq 1.000
- **FileCreateDate** - Creation time (platform-specific) - freq 0.000
- **FileInodeChangeDate** - Inode change time (Unix only) - freq 1.000

### Special (1 tag)
- **ImageDataMD5** - MD5 hash of image data (excluding metadata) - freq 0.000

## Work Completed

- ✅ Basic file metadata extraction exists
- ✅ FileType detection via magic numbers
- ✅ Some timestamps already extracted

## Remaining Tasks

### High Priority - Core Implementation

1. **Image Dimensions from File**
   - Extract width/height without full EXIF parse
   - Read from JPEG SOF markers
   - Handle RAW format dimensions
   - Must work even if EXIF corrupted

2. **Complete Timestamp Extraction**
   - Handle platform differences for creation time
   - Ensure timezone handling is correct
   - Format as ExifTool-compatible strings (YYYY:MM:DD HH:MM:SS±TZ)

3. **MIME Type Mapping**
   - Map file extensions to MIME types
   - Use magic number detection as fallback
   - Handle RAW format MIME types (e.g., "image/x-olympus-orf")

4. **Permissions Formatting**
   - Unix: Format as rwx string (e.g., "-rw-r--r--")
   - Windows: Handle file attributes (ReadOnly, Hidden, System)

### Medium Priority - Cross-Platform

1. **Platform Abstraction**
   - Abstract OS-specific calls
   - Handle missing timestamps gracefully
   - Consistent error handling

2. **Path Handling**
   - Normalize path separators
   - Handle Unicode in paths
   - Relative vs absolute paths

### Low Priority - Performance

1. **ImageDataMD5 Calculation**
   - Extract only image data (skip metadata)
   - Cache results if possible
   - Make optional for performance

## Prerequisites

- Verify file type detection is working
- Ensure timezone handling is implemented
- Cross-platform testing environment

## Testing Strategy

- Test on Windows, macOS, and Linux
- Verify timestamps are consistent
- Check special characters in filenames
- Compare with ExifTool file metadata

## Success Criteria

- All 15 file tags extracting correctly
- Cross-platform compatibility
- Consistent timestamp formatting
- Proper error handling for missing data
- Image dimensions extracted even without valid EXIF

## Gotchas & Tribal Knowledge

### Timestamp Issues
- **FileCreateDate**: Not reliable on Unix (often shows inode creation, not file creation)
- **FileInodeChangeDate**: Not available on Windows
- **Timezone**: ExifTool uses local timezone with offset (e.g., "2024:01:15 14:30:00-08:00")
- **macOS**: Has birth time which is true creation time

### Platform Differences
- **FilePermissions**: Unix uses octal + symbolic (e.g., "-rw-r--r--"), Windows uses attributes
- **Path Separators**: Normalize to forward slashes for consistency
- **Unicode**: Windows paths may have different encoding than Unix

### Image Dimensions
- **JPEG**: Read from SOF0/SOF2 markers, not EXIF
- **RAW**: May need format-specific parsing
- **Orientation**: File dimensions are pre-rotation values

### MIME Types
- RAW formats lack standard MIME types:
  - ORF: "image/x-olympus-orf"
  - CR2: "image/x-canon-cr2"
  - NEF: "image/x-nikon-nef"
- Use file extension as primary detection

### Special Cases
- **ExifByteOrder**: "II" = Little-endian (Intel), "MM" = Big-endian (Motorola)
- **ImageDataMD5**: Expensive operation, requires stripping all metadata
- **FileSize**: Report actual bytes, not "human readable" format
- **FileName**: Just the base name, not the full path