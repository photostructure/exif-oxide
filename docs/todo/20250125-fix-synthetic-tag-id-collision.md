# Technical Project Plan: Fix Synthetic Tag ID Collision

**Last Updated**: 2025-07-25
**Status**: TODO
**Estimated Time**: 4-6 hours
**Priority**: High - Blocking correct subdirectory tag extraction

## Project Overview

**Goal**: Fix the synthetic tag ID collision issue that causes subdirectory tags to overwrite each other.

**Problem**: When extracting multiple tags from a subdirectory (e.g., 27 WB_RGGB tags from Canon ColorData), all tags are assigned the same synthetic ID. This causes only the last tag to appear in the final output, as each overwrites the previous ones in the HashMap storage.

## Background & Context

### Why This Work is Needed

The subdirectory extraction system successfully extracts multiple tags from binary data structures. However, the current synthetic ID generation assigns the same ID to all tags from a single subdirectory:

```rust
// Current code: All tags from subdirectory 0x4001 get ID 0xC001
let synthetic_id = 0x8000 | (tag_id & 0x7FFF);
```

This defeats the purpose of subdirectory extraction, which should expand a single binary blob into multiple human-readable tags.

### Related Documentation

- [Tag Kit Subdirectory Support](../done/20250124-tag-kit-subdirectory-support.md) - The feature that exposed this bug
- [SUBDIRECTORY-COVERAGE.md](../reference/SUBDIRECTORY-COVERAGE.md) - Shows 46.3% Canon coverage
- [API-DESIGN.md](../design/API-DESIGN.md) - Tag storage architecture

## Technical Foundation

### Key Components

**Tag Storage System**:
- `ExifReader.extracted_tags: HashMap<u16, TagValue>` - Main tag storage (u16 ID → value)
- `ExifReader.synthetic_tag_names: HashMap<u16, String>` - Maps synthetic IDs to "Group:TagName"
- `ExifReader.tag_sources: HashMap<u16, TagSourceInfo>` - Metadata about tag origin

**Synthetic ID Ranges**:
- `0x8000-0xBFFF`: Reserved for subdirectory tags (current implementation)
- `0xC000-0xCFFF`: Canon binary data tags
- `0xF000-0xFFFF`: General processor synthetic tags

**Current ID Generation Pattern**:
```rust
// Subdirectory tags (the problematic one)
let synthetic_id = 0x8000 | (tag_id & 0x7FFF);  // Only uses parent tag ID

// Canon binary data tags (working correctly)
let hash = tag_name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);
```

### Architecture Insights

1. **HashMap Storage**: Using u16 as key means we have 65,536 possible IDs
2. **No Collision Detection**: Direct `insert()` silently overwrites existing tags
3. **JSON Output**: Uses `synthetic_tag_names` to resolve display names for synthetic IDs
4. **Precedence System**: `store_tag_with_precedence` exists but isn't used for synthetic tags

## Work Completed

### Investigation Results

1. **Root Cause Confirmed**: All subdirectory tags use parent tag ID for synthetic ID generation
2. **Impact Scope**: Affects all subdirectory extraction (748+ SubDirectory references in ExifTool)
3. **Canon T3i Test Case**: 
   - Extracts 27 tags from ColorData1 (0x4001)
   - All get synthetic ID 0xC001
   - Only last tag ("WB_RGGBLevelsTungsten") survives

### Debug Evidence

```
[DEBUG] Extracted 27 tags from subdirectory 0x4001
[DEBUG] Storing extracted tag 'WB_RGGBLevelsAsShot' from subdirectory 0x4001
[DEBUG] Storing tag 0xc001 with synthetic name mapping: 'MakerNotes:WB_RGGBLevelsAsShot'
[DEBUG] Storing extracted tag 'WB_RGGBLevelsDaylight' from subdirectory 0x4001
[DEBUG] Storing tag 0xc001 with synthetic name mapping: 'MakerNotes:WB_RGGBLevelsDaylight'
... (all 27 tags use 0xc001)
```

## Remaining Tasks

### Task 1: Implement Unique Synthetic ID Generation (High Confidence)

**Location**: `src/implementations/canon/mod.rs:969`

**Current Code**:
```rust
let synthetic_id = 0x8000 | (tag_id & 0x7FFF);
```

**Proposed Solution A: Counter-Based IDs**
```rust
// Add before the subdirectory loop (line ~945)
let mut synthetic_counter: u16 = 0;

// Inside the tag extraction loop (line ~969)
// Generate unique ID for each extracted tag
let synthetic_id = 0x8000 | synthetic_counter;
synthetic_counter += 1;

// Add bounds check to prevent overflow into other ranges
if synthetic_counter >= 0x4000 {
    warn!("Too many synthetic tags extracted, some may be lost");
    break;
}
```

**Proposed Solution B: Hash-Based IDs (Like Canon Binary)**
```rust
// Generate ID based on parent tag + extracted tag name
let combined = format!("{:04x}:{}", tag_id, tag_name);
let hash = combined.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
let synthetic_id = 0x8000 + ((hash as u16) & 0x3FFF); // 14-bit range

// Check for collisions
if exif_reader.extracted_tags.contains_key(&synthetic_id) {
    // Linear probe to find free slot
    let mut probe_id = synthetic_id;
    loop {
        probe_id = 0x8000 + (((probe_id & 0x3FFF) + 1) & 0x3FFF);
        if !exif_reader.extracted_tags.contains_key(&probe_id) {
            break;
        }
        if probe_id == synthetic_id {
            warn!("No free synthetic IDs available");
            continue; // Skip this tag
        }
    }
    synthetic_id = probe_id;
}
```

**Recommendation**: Start with Solution A (counter-based) as it's simpler and guaranteed unique.

### Task 2: Add Collision Detection (Medium Confidence)

Add a debug assertion to catch future collisions:

```rust
// Before inserting synthetic tag
debug_assert!(
    !exif_reader.synthetic_tag_names.contains_key(&synthetic_id),
    "Synthetic tag ID collision detected: 0x{:04x}", synthetic_id
);
```

### Task 3: Consider Architectural Improvements (Requires Research)

**Option 1: Composite Keys**
Instead of flattening to u16, use composite keys:
```rust
type TagKey = (u16, Option<String>); // (tag_id, optional_subtag_name)
extracted_tags: HashMap<TagKey, TagValue>
```

**Option 2: Separate Synthetic Storage**
```rust
struct ExifReader {
    extracted_tags: HashMap<u16, TagValue>,
    synthetic_tags: HashMap<String, TagValue>, // "Group:TagName" → value
    ...
}
```

**Option 3: Larger ID Space**
Use u32 IDs internally, only convert to u16 for legacy APIs.

## Prerequisites

None - the subdirectory extraction system is complete and working.

## Testing Strategy

### Unit Test
Add to `src/implementations/canon/mod.rs`:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_unique_synthetic_ids() {
        // Create mock ExifReader
        // Process ColorData with multiple tags
        // Verify each tag gets unique synthetic ID
        // Verify all tags appear in final output
    }
}
```

### Integration Test

1. **Canon T3i Test**:
   ```bash
   cargo run --release test-images/canon/Canon_T3i.jpg | jq -r '.[0] | to_entries[] | select(.key | test("WB_RGGB")) | .key' | sort | wc -l
   # Should output: 11 (not 1)
   ```

2. **Compare with ExifTool**:
   ```bash
   # Our output should have same WB_RGGB tags as ExifTool
   exiftool -j test-images/canon/Canon_T3i.jpg | jq -r '.[0] | to_entries[] | select(.key | test("WB_RGGB")) | .key' | sort > exiftool.txt
   cargo run --release test-images/canon/Canon_T3i.jpg | jq -r '.[0] | to_entries[] | select(.key | test("WB_RGGB")) | .key' | sort > ours.txt
   diff exiftool.txt ours.txt
   ```

### Manual Testing

1. Enable debug logging: `RUST_LOG=debug cargo run ...`
2. Verify unique synthetic IDs in logs
3. Check final JSON output has all extracted tags

## Success Criteria & Quality Gates

### Required Outcomes

1. **All subdirectory tags appear in output** - No overwrites due to ID collision
2. **Canon T3i shows 11 WB_RGGB tags** - Matching ExifTool output
3. **No performance regression** - Extraction time remains similar
4. **Backward compatibility** - Existing non-subdirectory tags unaffected

### Quality Checks

- [ ] `make precommit` passes
- [ ] No new clippy warnings
- [ ] Debug logs show unique synthetic IDs
- [ ] Integration test matches ExifTool output

## Gotchas & Tribal Knowledge

### ID Space Constraints

- **u16 Limitation**: We only have 65,536 possible tag IDs
- **Reserved Ranges**: Must stay within assigned ranges to avoid conflicts
- **Binary Compatibility**: Some tools may expect specific ID ranges

### HashMap Behavior

- **Silent Overwrites**: `HashMap::insert()` returns old value but doesn't error
- **Iteration Order**: HashMap iteration is non-deterministic
- **Key Types**: Must be `Eq + Hash`, u16 is ideal for performance

### Subdirectory Patterns

- **Multiple Tables**: One parent tag can reference multiple subdirectory tables
- **Conditional Selection**: Table chosen based on data size/content
- **Nested Subdirectories**: Some subdirectories contain further subdirectories

### Performance Considerations

- **Collision Detection**: Checking for existing IDs adds overhead
- **Linear Probing**: Can degrade to O(n) in worst case
- **Memory Usage**: More unique IDs means larger HashMaps

### Future Considerations

- **Sony/Nikon**: Will have same issue when subdirectory support added
- **Scalability**: Current approach may not scale to 1000s of tags
- **Error Handling**: Need graceful degradation when ID space exhausted

## Implementation Checklist

- [ ] Choose ID generation strategy (counter vs hash)
- [ ] Implement unique ID generation
- [ ] Add collision detection (debug mode)
- [ ] Update debug logging to show synthetic IDs
- [ ] Add unit test for uniqueness
- [ ] Test with Canon T3i
- [ ] Compare output with ExifTool
- [ ] Update documentation if needed
- [ ] Consider long-term architectural improvements