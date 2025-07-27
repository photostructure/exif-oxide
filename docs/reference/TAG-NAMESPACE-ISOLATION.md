# Tag Namespace Isolation Pattern

**Purpose**: This document explains ExifTool's dynamic tag table switching mechanism and how exif-oxide implements namespace-aware tag storage to prevent tag ID collisions across different IFD contexts.

**Audience**: Engineers working on EXIF parsing, subdirectory processing, or tag extraction systems.

**Prerequisites**: Understanding of EXIF IFD structure, ExifTool's ProcessExif function, and Rust HashMap usage.

## The Problem: Tag ID Collisions

EXIF metadata uses numeric tag IDs that **conflict across different contexts**:

- **GPS IFD**: Tag 0x0002 = `GPSLatitude`
- **EXIF IFD**: Tag 0x0002 = `InteroperabilityVersion`  
- **Canon MakerNotes**: Tag 0x0002 = `FocalLength`

When using a simple `HashMap<u16, TagValue>` storage, **later tags overwrite earlier ones** with the same ID, causing critical metadata loss.

## ExifTool's Solution: Dynamic Tag Table Switching

ExifTool solves this through **context-specific tag tables** during subdirectory processing:

```perl
# Exif.pm line ~8825: GPS subdirectory definition  
0x8825 => {
    SubDirectory => {
        TagTable => 'Image::ExifTool::GPS::Main',  # ← Context switch!
    },
},

# GPS.pm lines 51-82: GPS-specific tag table
%Image::ExifTool::GPS::Main = (
    GROUPS => { 1 => 'GPS' },
    0x0002 => { Name => 'GPSLatitude' },      # Different from EXIF 0x0002
    0x0004 => { Name => 'GPSLongitude' },     # GPS context
);
```

**Key Insight**: ExifTool doesn't use namespace prefixing or collision resolution. It uses **completely different tag lookup tables** based on the current processing context.

## exif-oxide Implementation: Namespace-Aware Storage

Since we can't dynamically switch Rust HashMaps like Perl hashes, we implement **namespace isolation** through composite keys:

### Core Storage Change

```rust
// Before: Tag ID collisions
pub(crate) extracted_tags: HashMap<u16, TagValue>,

// After: Namespace-aware keys  
pub(crate) extracted_tags: HashMap<(u16, String), TagValue>,
pub(crate) tag_sources: HashMap<(u16, String), TagSourceInfo>,
```

### Storage Isolation

```rust
// GPS tags stored with GPS namespace
(0x0002, "GPS".to_string()) → TagValue::F64Array([42.034575, 2.0, 4.47])  // GPSLatitude

// EXIF tags stored with EXIF namespace  
(0x0002, "EXIF".to_string()) → TagValue::String("0100")  // InteroperabilityVersion
```

### API Methods

```rust
impl ExifReader {
    /// Store tag with namespace context
    pub fn store_tag_with_precedence(
        &mut self,
        tag_id: u16, 
        value: TagValue,
        source_info: TagSourceInfo,
    ) {
        let key = (tag_id, source_info.namespace.clone());
        self.extracted_tags.insert(key.clone(), value);
        self.tag_sources.insert(key, source_info);
    }

    /// Legacy access across namespaces for backward compatibility
    pub(crate) fn get_tag_across_namespaces(&self, tag_id: u16) -> Option<&TagValue> {
        let namespaces = ["EXIF", "GPS", "MakerNotes"];
        for namespace in namespaces {
            let key = (tag_id, namespace.to_string());
            if let Some(value) = self.extracted_tags.get(&key) {
                return Some(value);
            }
        }
        None
    }
}
```

## Migration Pattern

When updating code from old to new API:

```rust
// Old: Direct HashMap access (causes collisions)
reader.extracted_tags.get(&tag_id)
reader.extracted_tags.insert(tag_id, value)

// New: Namespace-aware access
reader.get_tag_across_namespaces(tag_id)  
reader.store_tag_with_precedence(tag_id, value, source_info)

// Old: Iteration pattern
for (&tag_id, value) in &reader.extracted_tags

// New: Iteration pattern  
for (&(tag_id, _namespace), value) in &reader.extracted_tags
```

## Real-World Impact

**Before Fix** (GPS coordinates missing):
```json
{
  "EXIF:InteroperabilityVersion": "0100"  // Tag 0x0002 from EXIF overwrote GPS
}
```

**After Fix** (GPS coordinates present):
```json
{
  "EXIF:InteroperabilityVersion": "0100",       // Tag (0x0002, "EXIF")
  "GPS:GPSLatitude": 42.034575,                 // Tag (0x0002, "GPS") 
  "GPS:GPSLongitude": 0.5075027777777777
}
```

## Common Pitfalls

1. **Forgetting namespace context**: Always use `TagSourceInfo` to provide proper namespace when storing tags

2. **Legacy code assumptions**: Code expecting `HashMap<u16, TagValue>` needs updating to handle composite keys

3. **Iteration patterns**: Direct HashMap iteration requires pattern updates for the new key structure

4. **Backwards compatibility**: Use `get_tag_across_namespaces()` for code that doesn't know the specific namespace

## When to Use This Pattern

- **Multi-context EXIF parsing**: Any system processing multiple IFD types (GPS, EXIF, MakerNotes)
- **ExifTool compatibility**: When exact tag extraction behavior must match ExifTool
- **Legacy metadata preservation**: Systems where tag ID conflicts would cause data loss

## Source Code References

- **Core implementation**: [`src/exif/mod.rs`](../../src/exif/mod.rs) lines 36-41, 242-333
- **Tag storage**: [`src/exif/tags.rs`](../../src/exif/tags.rs) `store_tag_with_precedence` method  
- **Migration example**: [`src/exif/processors.rs`](../../src/exif/processors.rs) Make/Model tag access updates
- **Research documentation**: [`docs/todo/P10c-gps-ifd-parsing-bug.md`](../todo/P10c-gps-ifd-parsing-bug.md)

## Related Documentation

- **ExifTool behavior**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- **IFD processing**: [CORE-ARCHITECTURE.md](../guides/CORE-ARCHITECTURE.md)  
- **Tag tables**: [EXIFTOOL-GUIDE.md](../guides/EXIFTOOL-GUIDE.md)

*Document created: 2025-01-27 | Subject matter expert: GPS IFD parsing implementation team*