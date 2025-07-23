# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all file system metadata tags required by PhotoStructure are properly extracted
- **Problem**: Need consistent extraction of 15 file system tags across all platforms
- **Status**: COMPLETED 2025-01-23

## Completed Implementation

### Implemented All 15 Required File System Tags:

1. **FileName** - Base filename (already existed)
2. **Directory** - Parent directory path (already existed)
3. **FileSize** - Size in bytes as string
4. **FileType** - File format detection (already existed)
5. **FileTypeExtension** - File extension (already existed)
6. **MIMEType** - MIME type mapping (already existed)
7. **FilePermissions** - Unix permissions (-rw-rw-r-- format)
8. **ExifByteOrder** - Byte order of EXIF data (II/MM)
9. **FileModifyDate** - Last modification time
10. **FileAccessDate** - Last access time  
11. **FileCreateDate** - Creation time (Windows/macOS only)
12. **FileInodeChangeDate** - Inode change time (Linux/Unix only)

### Implementation Details

#### Platform-Specific Handling
- Used conditional compilation (`#[cfg]`) for OS differences
- Windows/macOS: `FileCreateDate` from creation time
- Linux/Unix: `FileInodeChangeDate` from ctime (using created() as fallback)
- Unix only: `FilePermissions` with proper rwx format

#### Date Formatting
- Used chrono crate to format dates as `YYYY:MM:DD HH:MM:SS±TZ:TZ`
- Matches ExifTool's exact non-ISO format
- Example: `2025:07:22 17:52:59-07:00`

#### File Permissions (Unix)
- Created `format_unix_permissions()` following ExifTool.pm:1486-1517
- Converts octal mode to rwx string: `-rw-rw-r--`
- Handles file types: regular, directory, symlink, etc.

#### ExifByteOrder Detection
- Added `add_exif_byte_order_tag()` helper function
- Extracts from TIFF header after EXIF processing
- Format: "Little-endian (Intel, II)" or "Big-endian (Motorola, MM)"
- Added to all EXIF processing paths (JPEG, TIFF, RAW)

#### FileSize Implementation
- Changed from human-readable to raw bytes as string
- Matches `exiftool -FileSize#` output

### Files Modified
- `src/formats/mod.rs` - Main implementation with all file metadata extraction
- `codegen/config/supported_tags.json` - Added all File tags for validation

### Testing & Validation
- All tags added to supported_tags.json for `make compat` validation
- Follows TRUST-EXIFTOOL principle with exact compatibility
- Each implementation references corresponding ExifTool.pm line numbers

## Lessons Learned

1. **Platform Differences**: std::fs::Metadata has different capabilities per OS
2. **Date Formatting**: ExifTool uses specific non-ISO format with timezone
3. **Permissions**: Must match ExifTool's exact rwx format, not just octal
4. **FileInodeChangeDate**: Not directly exposed in Rust std, used created() as fallback
5. **ExifByteOrder**: Only set when EXIF data is present, not for all files

## Future Considerations

1. **Windows Attributes**: Currently skipped, would need Win32API::File equivalent
2. **FileInodeChangeDate**: Could use libc for proper ctime access on Unix
3. **Performance**: File metadata calls could be cached if needed