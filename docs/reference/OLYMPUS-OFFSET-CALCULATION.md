# Olympus MakerNotes Subdirectory Offset Calculation

## The Problem

Olympus cameras have a unique quirk in how they store subdirectory offsets within their MakerNotes. This document explains the issue and how exif-oxide handles it, following ExifTool's implementation.

## Background

When Olympus cameras write MakerNotes, they include:
1. A signature header ("OLYMPUS\0" + padding = 12 bytes)
2. TIFF-structured data containing various subdirectories (Equipment, CameraSettings, etc.)

## The Offset Calculation Issue

### What You'd Expect

Normally, when processing subdirectory offsets within a TIFF structure:
- Subdirectory offset is relative to the current IFD's data start
- You'd add: `current_ifd_offset + subdirectory_offset`

### What Olympus Actually Does

Olympus subdirectory offsets are relative to the **original MakerNotes position in the file**, not the TIFF data start after the signature.

### Real Example from test.orf

```
File positions:
- MakerNotes tag at: 0xdf4
- Olympus signature: "OLYMPUS\0" (12 bytes including padding)
- TIFF data starts: 0xdf4 + 12 = 0xe00
- Equipment offset in IFD: 0x72

Incorrect calculation:
0xe00 (TIFF start) + 0x72 (offset) = 0xe72 ❌
This points to the middle of the first IFD entry!

Correct calculation:
0xdf4 (MakerNotes position) + 0x72 (offset) = 0xe66 ✅
This correctly points to the Equipment IFD start!
```

## ExifTool Reference

From `lib/Image/ExifTool/Olympus.pm` lines 1157-1168:

> "Olympus really screwed up the format of the following subdirectories (for the E-1 and E-300 anyway). Not only is the subdirectory value data not included in the size, but also the count is 2 bytes short for the subdirectory itself (presumably the Olympus programmers forgot about the 2-byte entry count at the start of the subdirectory)."

## How We Handle It

1. **Store Original Offset**: When processing MakerNotes, we store the original file position before any signature adjustment
2. **Use Original for Subdirectories**: When calculating subdirectory offsets within MakerNotes, we use the original position, not the adjusted TIFF data position

## Code Location

The implementation is in `src/exif/ifd.rs`:
- `process_maker_notes_with_signature_detection()` - stores original offset
- `parse_ifd_entry()` - uses original offset for subdirectory calculations

## Why This Matters

Without this adjustment:
- Equipment tags (CameraType2, SerialNumber, LensType) won't be extracted
- Other Olympus subdirectories will fail to parse
- You'll see errors about invalid entry counts (like 12336 instead of 25)

## Testing

To verify the fix works:
```bash
cargo run -- test-images/olympus/test.orf | grep -E "(CameraType|SerialNumber|LensType)"
```

Should show:
- CameraType2: "E-M1"
- SerialNumber: "BHP242330"
- LensType: "Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro"

## Other Manufacturers

This issue is specific to Olympus. Other manufacturers:
- **Canon**: Uses different offset calculation methods
- **Nikon**: Has encryption but standard offset calculation
- **Sony**: Complex but follows TIFF offset rules

Always check ExifTool source for manufacturer-specific quirks!