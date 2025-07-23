# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all file system metadata tags required by PhotoStructure are properly extracted
- **Problem**: Need consistent extraction of 15 file system tags across all platforms

## Background & Context

- File system tags come from OS file metadata, not image data
- Critical for file management and tracking
- Platform-specific implementations needed (Windows/macOS/Linux)


## Technical Foundation

Study the entirety of the documentation, and study referenced relevant source code.

- [CLAUDE.md](CLAUDE.md)
- [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) -- follow their dimension extraction algorithm **precisely**.
- [CODEGEN.md](docs/CODEGEN.md) -- if there's any tabular data, or perl code that you think we could automatically extract and use, **strongly** prefer that to any manual porting effort.
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

### Timestamps (4 tags)
- **FileModifyDate** - Last modification time - freq 1.000
- **FileAccessDate** - Last access time - freq 1.000
- **FileCreateDate** - Creation time (platform-specific) - freq 0.000

```
$ exiftool test-images/sony/sony_a7c_ii_02.jpg  -File:\* -G -j -struct
[{
  "SourceFile": "test-images/sony/sony_a7c_ii_02.jpg",
  "File:FileName": "sony_a7c_ii_02.jpg",
  "File:Directory": "test-images/sony",
  "File:FileSize": "25 MB",
  "File:FileModifyDate": "2025:07:22 17:52:59-07:00",
  "File:FileAccessDate": "2025:07:22 17:53:10-07:00",
  "File:FileInodeChangeDate": "2025:07:22 17:52:59-07:00",
  "File:FilePermissions": "-rw-rw-r--",
  "File:FileType": "JPEG",
  "File:FileTypeExtension": "jpg",
  "File:MIMEType": "image/jpeg",
  "File:ExifByteOrder": "Little-endian (Intel, II)",
  "File:ImageWidth": 7008,
  "File:ImageHeight": 4672,
  "File:EncodingProcess": "Baseline DCT, Huffman coding",
  "File:BitsPerSample": 8,
  "File:ColorComponents": 3,
  "File:YCbCrSubSampling": "YCbCr4:2:2 (2 1)"
}]
```

(we're handling ImageWidth, ImageHeight, EncodingProcess,BitsPerSample, ColorComponents,and YCbCrSubSampling in a parallel session)

##  Tasks

### High Priority - Core Implementation

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

- All referenced file tags extracting correctly
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
- **FilePermissions**: Unix uses octal + symbolic (e.g., "-rw-r--r--"), Windows uses attributes -- FOLLOW THE ExifTool SOURCE!
- **Path Separators**: Normalize to forward slashes for consistency
- **Unicode**: Windows paths may have different encoding than Unix, but we should normalize everything to UTF-8

### MIME Types
- RAW formats lack standard MIME types -- MATCH WHAT EXIFTOOL SAYS
  - ORF: "image/x-olympus-orf"
  - CR2: "image/x-canon-cr2"
  - NEF: "image/x-nikon-nef"
- Use file extension as primary detection

### Special Cases
- **ExifByteOrder**: "II" = Little-endian (Intel), "MM" = Big-endian (Motorola)
- **FileSize**: Report actual bytes, not "human readable" format
- **FileName**: Just the base name, not the full path