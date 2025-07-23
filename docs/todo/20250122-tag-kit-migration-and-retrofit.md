# Technical Project Plan: Tag Kit Migration and Runtime Retrofit

## Project Overview

**Goal**: Complete the tag kit system migration by:
1. Migrating all inline_printconv configs to tag kit
2. Wiring tag kit into the runtime system
3. Deprecating redundant extractors
4. Expanding to all manufacturer modules

**Problem**: We have multiple overlapping tag extraction systems. The tag kit system provides a unified approach that eliminates tag ID/PrintConv mismatches and simplifies maintenance.

## Background & Context

### Why This Work is Needed

- **Bug Prevention**: Tag kit eliminates offset errors by extracting tag IDs with their PrintConvs together
- **Maintenance Simplification**: One unified extractor instead of three overlapping ones
- **ExifTool Updates**: Monthly releases become easier with automated extraction
- **PR Reviews**: Generated code clearly shows tag+PrintConv relationships

### Current State

✅ **Completed**:
- Tag kit extraction works - 414 EXIF tags with PrintConvs
- Integration tests pass with 100% parity
- Modular generation splits into manageable files
- Deprecation notices added to old extractors
- Tag kit wired into runtime
- **All 7 inline_printconv configs migrated to tag_kit**:
  - Canon (17 tables) → canon__tag_kit.json
  - Sony (10 tables) → sony__tag_kit.json  
  - Olympus (8 tables) → olympus__tag_kit.json
  - Panasonic (1 table) → panasonic__tag_kit.json
  - MinoltaRaw (2 tables) → minoltaraw__tag_kit.json
  - Exif (1 table) → exif__tag_kit.json (already done)
  - PanasonicRaw (1 table) → panasonicraw__tag_kit.json (already done)
- **Fixed TagKitExtractor** to handle multiple tables per module
- **Runtime integration verified** - all modules have tag_kit/ subdirectories

❌ **Not Complete**:
- Manual implementations still in use (to be removed after validation)

### Related Documentation

- ["Tag kit" Codegen](docs/done/DONE-20250122-tag-kit-codegen.md) - Tag kit implementation details
- [EXTRACTOR-GUIDE.md](../reference/EXTRACTOR-GUIDE.md) - Extractor comparisons
- [CODEGEN.md](../CODEGEN.md) - Codegen system overview

## Technical Foundation

### Systems to Update

1. **inline_printconv configs** (7 modules)
   - Canon, Exif, Sony, MinoltaRaw, Olympus, Panasonic, PanasonicRaw
   - Currently in `codegen/config/*/inline_printconv.json`

2. **Runtime registry** (`src/registry.rs`)
   - Currently looks up PrintConv functions by name
   - Needs to check tag kit first

3. **Tag generator** (`codegen/src/generators/tags.rs`)
   - Currently generates `print_conv_ref` function names
   - Should reference tag kit PrintConvs

4. **Manual implementations** (`src/implementations/print_conv.rs`)
   - Can be gradually removed after validation

## Remaining Tasks

### Phase 1: Migrate inline_printconv Configs (1-2 days)

#### 1.1 Create tag_kit.json for Each Module

For each module with inline_printconv.json:

**Template**:
```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool/[Module].pm",
  "description": "[Module] tag definitions with embedded PrintConv implementations",
  "tables": [
    {
      "name": "[TableName]",
      "description": "Complete [Module] [TableName] tag definitions with PrintConvs"
    }
  ]
}
```

**Migration Steps**:
1. Check existing inline_printconv.json for table name
2. Create corresponding tag_kit.json
3. Run `make codegen`
4. Verify output in `src/generated/[module]_tag_kit/`

**Suggested Order**:
1. Exif (already tested with 414 tags)
2. Canon, Sony (large manufacturer modules)
3. Olympus, Panasonic (medium complexity)
4. MinoltaRaw, PanasonicRaw (specialized formats)

#### 1.2 Verify Coverage

```bash
# Before migration - save current output
cp -r src/generated/[Module]_pm/inline_*.rs /tmp/before/

# After migration - check tag kit output
grep -A5 "print_conv" src/generated/[module]_tag_kit/*.rs

# Ensure all PrintConvs captured
diff -u /tmp/before/ src/generated/[module]_tag_kit/
```

#### 1.3 Update Code References

```bash
# Find usage
grep -r "inline_printconv" src/

# Update imports and function calls
```

#### 1.4 Remove Old Configs

After verification:
1. Delete inline_printconv.json files
2. Remove from codegen pipeline
3. Consider removing inline_printconv.pl entirely

### Phase 2: Wire Tag Kit into Runtime (HIGH PRIORITY)

#### 2.1 Update Registry

In `src/registry.rs`:
```rust
// Add tag kit lookup before manual registry
pub fn apply_print_conv(tag_id: u16, value: &TagValue) -> Result<TagValue> {
    // Check tag kit first
    let mut evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    if let Some(result) = exif_tag_kit::apply_print_conv(
        tag_id, value, &mut evaluator, &mut errors, &mut warnings
    ) {
        // Handle errors/warnings as needed
        return Ok(result);
    }
    
    // Fall back to manual registry
    // ... existing code
}
```

#### 2.2 Add Metrics

Track usage to verify tag kit working:
```rust
static TAG_KIT_HITS: AtomicUsize = AtomicUsize::new(0);
static MANUAL_HITS: AtomicUsize = AtomicUsize::new(0);
```

### Phase 3: Expand Coverage (1-2 weeks)

#### 3.1 Create Tag Kit Configs for All Modules

- Canon, Nikon, Sony, Olympus (major manufacturers)
- GPS, Composite (standard modules)
- Each gets its own tag_kit.json

#### 3.2 Expression Support

Wire expression evaluator for GPS tags:
- GPSAltitude has: `$val =~ /^(inf|undef)$/ ? $val : "$val m"`
- Connect to existing expressions system

#### 3.3 Remove Manual Implementations

After validation:
1. Start with tested tags (ResolutionUnit, Orientation)
2. Remove in batches with testing
3. Keep complex "Manual" type (37 functions)

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_tag_kit_parity() {
    // Compare tag kit vs manual for all tags
    for (tag_id, expected) in known_values {
        let tag_kit_result = /* tag kit */;
        let manual_result = /* manual */;
        assert_eq!(tag_kit_result, manual_result);
    }
}
```

### Integration Tests
- Use existing `tests/tag_kit_integration.rs`
- Add tests for each migrated module
- Compare with ExifTool output

### Validation
```bash
# Compare outputs
cargo run --bin compare-with-exiftool test.jpg
```

## Success Criteria & Quality Gates

**Phase 1 Complete**:
- [x] All 7 inline_printconv configs migrated
- [x] Generated code includes all PrintConvs
- [x] Tests pass with new code
- [ ] Old configs deleted (cleanup pending)

**Phase 2 Complete**:
- [x] Tag kit wired into runtime
- [ ] Metrics show tag kit being used (needs verification)
- [ ] No regression in tag extraction (validation pending)

**Phase 3 Complete**:
- [ ] All modules have tag kit configs
- [ ] Expression PrintConvs working
- [ ] Manual implementations removed where possible

## Gotchas & Tribal Knowledge

### Table Name Discovery

Check inline_printconv.json:
```json
"table_name": "Main"  // Extract this table
```

### Multiple Tables per Module

Some modules have multiple tables:
```json
"tables": [
  { "name": "Main", "description": "..." },
  { "name": "CameraSettings", "description": "..." }
]
```

### Known Issues

1. **Large "other" category** - 3245 lines in modular generation
2. **Empty groups** - Tag groups not populated yet
3. **Expression evaluation** - Extracted but not evaluated
4. **Complex PrintConvs** - 37 still need manual implementation

### Migration Differences

- inline_printconv: Only PrintConv hashes
- tag_kit: Complete tag definitions with PrintConv
- More data but eliminates mismatches

## Metrics & Benefits

**Current State**:
- 414 EXIF tags extracted
- 74 Simple PrintConvs automated
- 6 Expression PrintConvs ready
- 37 Manual PrintConvs identified

**Expected Benefits**:
- Zero offset errors
- 90% reduction in manual PrintConv code
- Automated ExifTool updates
- Better PR reviewability

## Next Steps

1. **Immediate**: Start Phase 1 migrations (Exif module first)
2. **This Week**: Complete all inline_printconv migrations
3. **Next Week**: Wire into runtime and validate
4. **Future**: Expand to all manufacturer modules

This consolidation is essential to avoid maintaining parallel extraction systems!