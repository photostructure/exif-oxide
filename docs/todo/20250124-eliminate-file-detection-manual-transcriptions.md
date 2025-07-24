# Eliminate Manual ExifTool Transcriptions in file_detection.rs

## Project Overview

**High-level goal**: Replace all manually transcribed ExifTool lookup tables in `src/file_detection.rs` with automatically generated tables using the existing codegen infrastructure.

**Problem statement**: The file detection module contains hardcoded lookup tables manually copied from ExifTool source, violating the project's "MANUAL PORTING BANNED" rule. These manual transcriptions create maintenance burden, introduce transcription errors, and become stale with monthly ExifTool releases. Additionally, commented-out `lookup_weakmagic` calls indicate missing functionality.

## Background & Context

**Why this work is needed**: Manual transcription of ExifTool data has caused 100+ bugs historically. The project requires all ExifTool data to be automatically extracted to ensure accuracy and maintainability.

**Key violations identified**:
- RIFF format mapping (lines 360-369) - manual copy of `RIFF.pm:49-53` `%riffType` hash
- ftyp brand mapping (lines 665-675) - manual copy of `QuickTime.pm` `%ftypLookup` hash
- WeakMagic functionality missing - `lookup_weakmagic` calls commented out due to missing generated code

**Links to related design docs**:
- [CODEGEN.md](../CODEGEN.md) - Complete codegen system documentation
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Why manual porting is banned
- [EXTRACTOR-GUIDE.md](../reference/EXTRACTOR-GUIDE.md) - Extractor selection guide

## Technical Foundation

**Key codebases**:
- `src/file_detection.rs` - Contains manual transcriptions to eliminate
- `codegen/extractors/simple_table.pl` - Extractor for basic lookup tables
- `codegen/extractors/boolean_set.pl` - Extractor for boolean membership sets
- `codegen/src/generators/lookup_tables/standard.rs` - Generated lookup table code generator

**Documentation**:
- [CODEGEN.md Simple Tables](../CODEGEN.md#simple-tables) - How simple_table.pl works
- [EXTRACTOR-GUIDE.md boolean_set.pl](../reference/EXTRACTOR-GUIDE.md#boolean_setpl) - Boolean set extraction

**APIs**:
- Generated lookup functions: `lookup_table_name(key) -> Option<&'static str>`
- Boolean set functions: `lookup_set_name(key) -> bool`

**Systems to familiarize with**:
- Codegen configuration format in `config/*.json`
- ExifTool Perl source structure and hash definitions
- Generated code patterns in `src/generated/`

## Work Completed

**Research and analysis completed**:
- ✅ **Binary data handling validated** - Research confirmed `simple_table.pl` and generation pipeline correctly preserve trailing spaces (e.g., `'crx '` brand)
- ✅ **ExifTool source locations identified** - All manual transcriptions mapped to specific ExifTool hashes
- ✅ **Test infrastructure validated** - `make compat` provides comprehensive regression testing
- ✅ **WeakMagic status clarified** - Missing functionality, not broken imports

**Key findings from research**:
- Extension normalization (`TIF` → `TIFF`) is hardcoded logic in `ExifTool.pm:9013`, not extractable hash table
- `validate_primitive_value()` in extraction system accepts all string data including binary characters
- `escape_string()` in generation properly preserves regular characters and trailing spaces
- Comprehensive test coverage exists via `make compat` and integration tests

**Decision rationale**:
- Use `simple_table.pl` for RIFF/ftyp extractions (proven reliable for string tables)
- Use `boolean_set.pl` for WeakMagic (single MP3 entry, membership test pattern)
- Keep extension normalization as manual code with source attribution (not extractable)

## Remaining Tasks

### High Confidence Tasks (Ready for Implementation)

#### 1. Generate Missing WeakMagic Lookup
**Implementation instructions**:
```bash
# Add %weakMagic to existing boolean_set.json
cat >> codegen/config/ExifTool_pm/boolean_set.json << 'EOF'
    ,{
      "hash_name": "%weakMagic",
      "constant_name": "WEAK_MAGIC_FILE_TYPES",
      "key_type": "String",
      "description": "File types with weak magic number recognition (MP3)"
    }
EOF

# Generate code
make codegen

# Verify generated file
ls src/generated/ExifTool_pm/weakmagic.rs
```

#### 2. Restore WeakMagic Functionality
**Implementation instructions**:
```rust
// In src/file_detection.rs, uncomment line 18:
use crate::generated::ExifTool_pm::lookup_weakmagic;

// In src/file_detection.rs, replace line 114:
// Change: if false { // TODO: Re-enable when lookup_weakmagic is generated
// To:     if lookup_weakmagic(candidate) {
```

#### 3. Extract RIFF Format Mapping
**Implementation instructions**:
```bash
# Create new config directory
mkdir -p codegen/config/RIFF_pm

# Create configuration
cat > codegen/config/RIFF_pm/simple_table.json << 'EOF'
{
  "source": "third-party/exiftool/lib/Image/ExifTool/RIFF.pm",
  "description": "RIFF format type mappings",
  "tables": [
    {
      "hash_name": "%riffType",
      "constant_name": "RIFF_TYPE_MAPPING",
      "key_type": "String",
      "description": "RIFF format ID to file type mapping (8 entries)"
    }
  ]
}
EOF

# Generate and validate
make codegen
ls src/generated/RIFF_pm/riff_type_mapping.rs
```

#### 4. Replace RIFF Hardcoded Matches
**Implementation instructions**:
```rust
// In src/file_detection.rs, add import:
use crate::generated::RIFF_pm::lookup_riff_type_mapping;

// Replace hardcoded match in detect_riff_type() (lines 360-369):
let detected_type = lookup_riff_type_mapping(&String::from_utf8_lossy(format_id))
    .unwrap_or("RIFF")
    .to_string();

// Replace hardcoded match in validate_riff_format() (lines 428-441):
let detected_type = lookup_riff_type_mapping(&String::from_utf8_lossy(format_id))
    .unwrap_or("");
expected_type == detected_type
```

### Medium Confidence Tasks (Require Careful Subset Selection)

#### 5. Extract ftyp Brand Mapping
**Research needed**: Determine correct subset of `%ftypLookup` entries for file type detection (not all entries are relevant).

**Implementation approach**:
```bash
# Create configuration (subset needs verification)
cat > codegen/config/QuickTime_pm/simple_table.json << 'EOF'
{
  "source": "third-party/exiftool/lib/Image/ExifTool/QuickTime.pm",
  "description": "QuickTime ftyp brand to file type mappings",
  "tables": [
    {
      "hash_name": "%ftypLookup",
      "constant_name": "FTYP_BRAND_MAPPING",
      "key_type": "String", 
      "description": "MOV/MP4 ftyp brand to file type mapping (subset for detection)"
    }
  ]
}
EOF
```

**Critical**: Test with files containing `'crx '` brand (trailing space) to validate binary data preservation.

### Low Confidence Tasks (Documentation Updates)

#### 6. Add Source Attribution
**Implementation**: Add comments to non-extractable manual code:
```rust
// Extension normalization - ExifTool.pm:9013 GetFileExtension()
// Hardcoded logic (TIF->TIFF only), not extractable as hash table
$fileExt eq 'TIF' and $fileExt = 'TIFF';
```

#### 7. MIME Type Fallback Audit
**Analysis required**: Compare `file_detection.rs:778-816` with `ExifTool_pm/mimetype.rs` to identify duplicates vs. legitimate fallbacks.

## Prerequisites

None - all required infrastructure exists and is operational.

## Testing Strategy

**Integration tests**:
```bash
# Test WeakMagic restoration with MP3 files
cargo test test_real_file_detection -- --nocapture

# Test RIFF format detection 
# Verify AVI, WAV, WEBP files in test-images/ directory
```

**Regression testing**:
```bash
# CRITICAL: Run after each change
make compat

# Validates both file type detection AND MIME type accuracy
# Zero tolerance for detection regressions
```

**Manual testing steps**:
1. Test MP3 files use extension-based detection (weak magic behavior)
2. Test RIFF files: AVI, WAV, WEBP detection accuracy
3. Test ftyp brands: HEIC, AVIF, CR3, MP4 with different brands
4. Test binary data: files with trailing spaces in headers
5. Performance comparison (optional): HashMap vs match statement speed

## Success Criteria & Quality Gates

**Definition of done**:
- [ ] WeakMagic functionality restored - MP3 detection via extension fallback works
- [ ] All hardcoded RIFF format matches replaced with generated lookups
- [ ] ftyp brand detection uses generated lookups (if extraction successful)
- [ ] `make compat` passes with zero detection regressions
- [ ] No manual ExifTool transcriptions remain without generated alternative or source attribution
- [ ] Integration tests pass for all affected file formats

**Quality gates**:
- `make compat` validation required after each extraction
- Code review focusing on manual transcription elimination
- Performance validation (ensure no significant slowdown)

## Gotchas & Tribal Knowledge

**Binary data handling**:
- ftyp brands like `'crx '` have trailing spaces that must be preserved
- Research confirmed extraction system handles this correctly, but test with real files
- If extraction fails, keep manual code with proper source attribution

**Extension rules are NOT extractable**:
- Extension normalization (`TIF` → `TIFF`) is hardcoded logic, not hash table
- Extension aliases may also be hardcoded logic - investigate before attempting extraction
- These are legitimate manual transcriptions when properly documented

**Test file locations**:
- Use `test-images/` directory for real file testing, not `third-party/exiftool/t/images/` (8x8 stripped images)
- HEIC test files available: `test-images/apple/IMG_9757.heic`
- Integration tests already exist in `tests/file_detection_integration.rs`

**Performance considerations**:
- HashMap lookups vs match statements should have negligible performance difference
- File detection is not a performance-critical path
- Focus on correctness over micro-optimizations

**WeakMagic behavior**:
- Only MP3 has weak magic currently (single entry in `%weakMagic`)
- Weak magic types defer to extension-based detection as fallback
- Must preserve exact ExifTool behavior for MP3 detection

**Codegen configuration patterns**:
- `simple_table.pl` requires `hash_name` with `%` prefix in config
- Generated functions follow pattern: `lookup_{constant_name_lowercase}()`
- Key types: use `"String"` for text keys, specific types for numeric keys

**Known risks**:
- ftyp brand extraction may require careful subset selection from large `%ftypLookup` hash
- Binary data in file headers requires testing, not just assumption of correctness
- API refactoring freedom exists (no users) but changes should be surgical and well-tested

**Success pattern from previous work**:
- Similar extractions completed successfully for Canon, Nikon, Sony modules
- MIME type extraction already working as reference implementation
- Follow proven patterns: extract → test → replace → validate