# Unknown Tag Handling in exif-oxide

**Last Updated**: 2025-07-25

## Overview

exif-oxide follows ExifTool's default behavior by omitting tags marked as "Unknown" in the source. This provides cleaner output and matches what most users expect from metadata extraction tools.

## Current Behavior

### ExifTool Comparison

**ExifTool (default)**:
```bash
$ exiftool image.jpg | grep "WB RGGB" | wc -l
10  # Shows only known/documented tags
```

**ExifTool with -u flag**:
```bash
$ exiftool -u image.jpg | grep "WB RGGB" | wc -l
25  # Shows all tags including Unknown ones
```

**exif-oxide**:
```bash
$ exif-oxide image.jpg | grep "WB_RGGB" | wc -l
10  # Shows only known tags, matching ExifTool default
```

### Why We Filter Unknown Tags

1. **Clean Output**: Most users don't need undocumented tag data
2. **ExifTool Compatibility**: Matches ExifTool's default behavior
3. **Reduced Noise**: Prevents cluttering output with experimental or manufacturer-internal tags
4. **Performance**: Slightly faster by skipping unknown tag processing

## Tag Examples

From Canon ColorData extraction, we show only:
```json
{
  "MakerNotes:WB_RGGBLevelsAsShot": "2241 1024 1024 1689",        // ✓ Shown
  "MakerNotes:WB_RGGBLevelsDaylight": "2217 1024 1024 1631",      // ✓ Shown
  // WB_RGGBLevelsUnknown tags are filtered out
}
```

Tags we filter out:
- `WB_RGGBLevelsUnknown`
- `WB_RGGBLevelsUnknown10`
- `WB_RGGBLevelsUnknown11`
- Any tag with "Unknown" in its name

## Implementation Details

### Tag Definition Structure

In ExifTool source, unknown tags are marked with:
```perl
0x1d => { Name => 'WB_RGGBLevelsUnknown', Format => 'int16s[4]', Unknown => 1 },
```

### Current Implementation

The tag_kit.pl extractor captures all tags including those with `Unknown => 1`, but we filter them during runtime:

```rust
// Skip tags marked as Unknown (matching ExifTool's default behavior)
if tag_name.contains("Unknown") {
    debug!("Skipping unknown tag: {}", tag_name);
    continue;
}
```

This simple string-based filter catches all tags with "Unknown" in their name, which ExifTool uses consistently for undocumented tags.

## Future Considerations

### Potential -u Flag Support

To match ExifTool's `-u` flag, we could:

1. **Add CLI flag**: `--show-unknown` or `-u` to include unknown tags
2. **Make filtering conditional**: Skip the "Unknown" check when flag is set
3. **Library API**: Add `include_unknown_tags` option to extraction functions

### API Design

For library users:
```rust
pub struct ExtractionOptions {
    pub include_unknown_tags: bool,  // Default: false to match ExifTool
    // ... other options
}
```

## Recommendations

1. **Document Current Behavior**: Make it clear we extract all tags by default
2. **Consider Filtering Options**: Add ability to filter unknown tags for ExifTool compatibility
3. **Preserve Tag Metadata**: Keep track of which tags are marked as Unknown for future filtering

## Related Documentation

- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Our principle of following ExifTool exactly
- [API-DESIGN.md](./API-DESIGN.md) - Overall API design including tag extraction