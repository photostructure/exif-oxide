# P12: Tag Filtering and PrintConv/ValueConv Control

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md).

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!

## Project Overview

- **Goal**: Add tag filtering and PrintConv/ValueConv control to match ExifTool's CLI behavior for selective metadata extraction
- **Problem**: exif-oxide currently extracts all tags and always applies PrintConv; users need filtering and numeric value control like ExifTool
- **Critical Constraints**:
  - 🔧 Must exactly match ExifTool's behavior for tag filtering and numeric value output
  - ⚡ Case-insensitive tag matching like ExifTool (`-MIMEType` == `-mimetype`)
  - 📐 Support all ExifTool filtering patterns: specific tags, groups, wildcards, `-GROUPNAME:all`, and `-all`
  - ⚠️ **CRITICAL**: `-Orientation#` means ONLY extract Orientation tag AND use numeric values (not all tags with numeric formatting)
  - 🚀 Performance optimization: skip expensive parsing for File-only requests (e.g., `-MIMEType` should not parse EXIF data)
  - 🎯 Support glob patterns: `-GPS*`, `-Canon*` for wildcard matching
  - 🔄 Support complex filtering: `-Orientation# -EXIF:all -GPS*` should extract Orientation(numeric) + all EXIF tags + all GPS\* tags

## Background & Context

ExifTool provides sophisticated tag filtering and value formatting control that exif-oxide currently lacks:

1. **Tag Filtering**: Users can specify exactly which tags to extract using tag names, group names, wildcards, or `-all`
2. **PrintConv vs ValueConv**: Users can control whether values are human-readable (PrintConv) or raw numeric (ValueConv) using the `#` suffix
3. **Case Insensitive**: All tag and group names are case-insensitive for user convenience

This feature enables:

- Performance optimization by extracting only needed tags
- Precise control over output format (human vs machine readable)
- Compatibility with existing ExifTool workflows and scripts

## Technical Foundation

### ExifTool CLI Reference

Key ExifTool behaviors to replicate:

**Tag Filtering**:

```bash
# Extract specific tag (case insensitive)
exiftool -MIMEType file.jpg           # extracts File:MIMEType
exiftool -mimetype file.jpg           # same result

# Extract all tags
exiftool -all file.jpg                # extracts everything

# Group filtering
exiftool -EXIF:all file.jpg           # all EXIF group tags
exiftool -File: file.jpg              # all File group tags

# Wildcard/glob patterns (validated July 2025)
exiftool -GPS* file.jpg               # all tags starting with "GPS"
exiftool -*tude file.jpg              # all tags ending with "tude" (GPSLatitude, GPSLongitude, etc.)
exiftool -*Date* file.jpg             # all tags containing "Date"
exiftool -Canon* file.jpg             # all tags starting with "Canon"
```

**PrintConv vs ValueConv Control**:

```bash
# Human-readable (PrintConv) - default
exiftool -Orientation file.jpg        # "Rotate 270 CW"

# Numeric (ValueConv) with # suffix
exiftool '-Orientation#' file.jpg     # 8
```

### Current exif-oxide Architecture

- **CLI**: `src/main.rs` - command line argument parsing
- **API**: `src/lib.rs` - `extract_metadata()` function
- **TagEntry**: Core data structure for metadata entries
- **Registry**: Tag registration and lookup system

## Work Completed

✅ **Phase 1: Research and Architecture** (July 2025)

- Researched ExifTool CLI documentation for tag filtering patterns
- Identified key behaviors: case insensitivity, `-all`, `#` suffix for numeric values
- Located relevant ExifTool documentation sections

✅ **Phase 2: API Design** (July 2025)

- Created `FilterOptions` struct in `src/types/metadata.rs` with comprehensive filtering logic
- Implemented case-insensitive tag matching and group filtering methods
- Added numeric tag control support with `HashSet<String>` for `#` suffix handling
- See: [src/types/metadata.rs:FilterOptions](src/types/metadata.rs)

✅ **Phase 3: Function Signature Updates** (July 2025)

- Updated `extract_metadata()` in `src/formats/mod.rs` to accept `Option<FilterOptions>`
- Maintained backward compatibility with `None` parameter defaulting to extract all tags
- Added performance optimization check for File-only requests
- See: [src/formats/mod.rs:extract_metadata](src/formats/mod.rs)

✅ **Phase 4: CLI Parsing** (July 2025)

- Replaced basic `parse_mixed_args()` with comprehensive `parse_exiftool_args()` in `src/main.rs`
- Added support for all ExifTool patterns: `-TagName`, `-TagName#`, `-GROUPNAME:all`, `-all`
- Implemented case-insensitive parsing and numeric tag detection
- Fixed mixed file/tag argument handling for ExifTool compatibility
- See: [src/main.rs:parse_exiftool_args](src/main.rs)

✅ **Phase 5: Performance Optimization Framework** (July 2025)

- Added `extract_file_tags_only()` function for early return optimization
- Implemented `is_file_group_only()` detection to skip expensive EXIF parsing
- Created smart filtering to avoid processing unneeded data for simple requests
- See: [src/formats/mod.rs:extract_file_tags_only](src/formats/mod.rs)

✅ **Phase 6: Test Infrastructure Updates** (July 2025)

- Updated all test files to use new 4-parameter `extract_metadata()` signature
- Fixed compilation errors in multiple test modules
- Maintained test coverage while adding new functionality

✅ **Phase 7: Critical Bug Fix - Central Filtering Architecture** (July 2025)

- **🚨 FIXED**: Discovered and fixed critical bug where specific tag requests extracted ALL tags instead of filtered subset
- **Root Cause**: Filtering was applied at CLI level but ignored during format-specific extraction
- **Solution**: Implemented central filtering chokepoint in `src/formats/mod.rs` matching ExifTool's FoundTag architecture
- **Architecture**: Applied post-processing filtering with allowlist-based tag inclusion using `should_extract_tag()`
- **Performance**: Fixed performance optimization logic that incorrectly categorized glob patterns as "file only"
- **Result**: `-Orientation#` now correctly extracts only 1 tag instead of 96+ tags

✅ **Phase 8: Comprehensive Glob Pattern Support** (July 2025)

- **Validated ExifTool Patterns**: Tested all wildcard types against real ExifTool output
- **Prefix Wildcards**: `-GPS*`, `-Canon*`, `-File*` - matches tags starting with pattern
- **Suffix Wildcards**: `-*tude`, `-*Date`, `-*Mode` - matches tags ending with pattern  
- **Middle Wildcards**: `-*Date*`, `-*Size*` - matches tags containing pattern
- **Case Insensitive**: All patterns work with lowercase input (e.g., `-gps*`)
- **Implementation**: Added `matches_glob_pattern()` method to FilterOptions with comprehensive wildcard support
- **Edge Cases**: Non-matching patterns return only SourceFile (matches ExifTool behavior)

✅ **Phase 9: Integration Testing & Validation** (July 2025)

- **Created 15 comprehensive integration tests** in `tests/p12_tag_filtering_integration_test.rs`
- **Test Coverage**: All filtering patterns, numeric control, complex combinations, performance optimization
- **Manual Testing**: Validated against ExifTool output using real image files
- **Performance Validation**: Confirmed File-only requests skip expensive EXIF parsing
- **Edge Case Testing**: Non-matching patterns, case insensitivity, multiple glob patterns
- **All Tests Passing**: 15/15 integration tests + comprehensive unit tests

✅ **Phase 10: CLI Help System** (July 2025)

- **Added comprehensive --help documentation** with examples and pattern reference
- **ExifTool Pattern Guide**: Documents all supported filtering patterns with real examples
- **Usage Examples**: Shows common use cases from simple tags to complex combinations
- **Multiple Filter Examples**: Demonstrates `-Orientation# -GPS* -EXIF:all` syntax

✅ **Phase 11: Public API Enhancement** (July 2025)

- **Added FilterOptions to public API** via `lib.rs` exports
- **New Functions**:
  - `extract_metadata_json_with_filter()` - JSON output with filtering
  - `extract_metadata_with_filter()` - Direct TagEntry access with filtering
- **Full API Compatibility**: Supports all CLI filtering features programmatically
- **Comprehensive Examples**: Documentation with real-world usage patterns for all filter types
- **Test Coverage**: 6 additional API tests covering all filtering scenarios

## Remaining Tasks

**🎉 ALL TASKS COMPLETED! 🎉**

P12 Tag Filtering and PrintConv/ValueConv Control is fully implemented and tested:

✅ **Core Functionality Complete**:
- ExifTool-compatible tag filtering with all pattern types
- Case-insensitive tag matching
- Numeric value control with `#` suffix  
- Performance optimization for File-only requests
- Central filtering architecture matching ExifTool's FoundTag approach

✅ **Pattern Support Complete**:
- Specific tags: `-MIMEType`, `-Orientation#`
- Group patterns: `-EXIF:all`, `-File:all`
- Prefix wildcards: `-GPS*`, `-Canon*`, `-File*`
- Suffix wildcards: `-*tude`, `-*Date`, `-*Mode`
- Middle wildcards: `-*Date*`, `-*Size*`
- Complex combinations: `-Orientation# -GPS* -EXIF:all`

✅ **Quality Assurance Complete**:
- 15 comprehensive integration tests (all passing)
- Comprehensive unit tests for FilterOptions methods
- Manual testing against ExifTool reference output
- Performance validation and edge case testing
- CLI help system with comprehensive documentation
- Public API with full filtering support

## Post-Completion Tasks

### 1. **Monitor for Edge Cases in Production** (Ongoing)

**Status**: Ready for production use

**Validation**: All major ExifTool filtering patterns work identically to reference implementation

### 2. **Documentation Maintenance** (Ongoing)

**Status**: Complete with comprehensive examples

**Coverage**: CLI help, API documentation, and TPP documentation all updated

### 3. **Performance Monitoring** (Future)

**Status**: Optimizations implemented

**Implemented**: File-only request optimization, early termination patterns, efficient glob matching

## Prerequisites

- Understanding of current TagEntry and TagRegistry architecture
- Familiarity with ExifTool CLI argument patterns
- Knowledge of Rust's case-insensitive string matching

## Testing Strategy

### Unit Tests

- Case-insensitive tag matching functions
- FilterOptions parsing and validation
- Tag filtering logic with various patterns
- **Wildcard pattern matching (Validated July 2025)**:
  - Prefix wildcards: `-GPS*`, `-Canon*`, `-File*`
  - Suffix wildcards: `-*tude`, `-*Date`, `-*Mode`
  - Middle wildcards: `-*Date*`, `-*Size*`
  - Complex combinations: multiple wildcards in single request

### Integration Tests

- CLI argument parsing with tag filters
- End-to-end filtering with real image files
- JSON output format validation
- **Wildcard integration tests (July 2025)**:
  - Test against actual ExifTool output for wildcard patterns
  - Validate complex wildcard combinations: `-*tude -EXIF:all -Canon*`
  - Test edge cases: non-matching wildcards, empty results

### Compatibility Tests

```rust
#[test]
fn test_exiftool_compatibility() {
    // Compare output with ExifTool for same filtering options
    let exiftool_output = run_exiftool(&["-MIMEType", "image.jpg"]);
    let oxide_output = extract_with_filter("image.jpg", &["MIMEType"]);
    assert_eq!(normalize_output(exiftool_output), normalize_output(oxide_output));
}
```

### Manual Testing Steps

1. Test case insensitivity: `-MIMEType` vs `-mimetype`
2. Test numeric values: `-Orientation` vs `-Orientation#`
3. Test group filtering: `-EXIF:all`, `-File:`
4. Test `-all` flag behavior
5. **Test wildcard patterns (Validated July 2025)**:
   - Prefix: `-GPS*`, `-Canon*`, `-File*`
   - Suffix: `-*tude`, `-*Date`, `-*Mode`
   - Middle: `-*Date*`, `-*Size*`
   - Complex: `-Orientation# -*tude -EXIF:all`
6. Test edge cases: `-XYZ*` (no matches), `-*` (should match everything?)
7. Verify JSON output format matches ExifTool for all wildcard patterns

## Success Criteria & Quality Gates

- [ ] All ExifTool tag filtering patterns work identically
- [ ] Case-insensitive matching matches ExifTool behavior exactly
- [ ] Numeric value output with `#` suffix matches ExifTool
- [ ] `-all` flag extracts same tags as ExifTool `-all`
- [ ] **Wildcard patterns work identically to ExifTool (Validated July 2025)**:
  - [ ] Prefix wildcards: `-GPS*`, `-Canon*`, `-File*`
  - [ ] Suffix wildcards: `-*tude`, `-*Date`, `-*Mode`
  - [ ] Middle wildcards: `-*Date*`, `-*Size*`
  - [ ] Complex combinations: `-Orientation# -*tude -EXIF:all`
- [ ] JSON output format compatible with existing tools
- [ ] Performance impact minimal for filtered extraction
- [ ] All compatibility tests pass
- [ ] `make precommit` passes

## Gotchas & Tribal Knowledge

### 🚨 **Critical Discovery**: Filtering Architecture Bug (July 2025)

- **Problem**: Tag filtering is applied at CLI level but ignored during format-specific extraction
- **Root Cause**: `src/formats/mod.rs` calls `exif_reader.get_all_tag_entries()` and ignores FilterOptions
- **Impact**: `-Orientation#` extracts all 96+ tags instead of just Orientation
- **Fix Required**: Apply filtering during tag collection, not just CLI parsing
- **Test Evidence**: Manual test showed numeric control works but filtering completely broken

### 🎯 **ExifTool Behavior Notes** (Validated July 2025)

- Tag names are case-insensitive everywhere: `-MIMEType` == `-mimetype`
- **CRITICAL**: `-Orientation#` means ONLY extract Orientation tag AND use numeric values (two operations)
- Group patterns: `-EXIF:all` matches all EXIF group tags, `-File:all` matches all File group tags
- `-all` is special - enables extraction of all available tags (not just a pattern)
- **Wildcard patterns (Validated July 2025)**:
  - **Prefix**: `-GPS*` matches GPSLatitude, GPSLongitude, GPSAltitude, etc.
  - **Suffix**: `-*tude` matches GPSLatitude, GPSLongitude, GPSAltitude
  - **Middle**: `-*Date*` matches FileModifyDate, DateTimeOriginal, GPSDateStamp, etc.
  - **Edge case**: `-XYZ*` (no matches) returns only SourceFile
  - **Combinations work**: `-*tude -EXIF:all -Canon*` extracts all matching patterns
- Must use `exiftool -j -struct -G` format for compatibility testing

### 🔧 **Implementation Decisions** (July 2025)

- Used `HashSet<String>` for `numeric_tags` to handle `#` suffix efficiently
- Chose `Vec<String>` over `HashSet` for `requested_tags` to preserve order
- Added performance optimization with `is_file_group_only()` check for early return
- Case-insensitive matching uses `to_lowercase()` for consistency across ASCII tag names

### ⚡ **Performance Optimization Strategy** (July 2025)

- File-only requests (e.g., `-MIMEType`, `-FileSize`) skip expensive EXIF parsing
- Early return from `extract_metadata()` when only File group tags requested
- Filter during extraction phase, not post-processing phase, to minimize I/O

### 🧪 **Testing Strategy** (July 2025)

- Use actual test images, not synthetic data (follows CLAUDE.md guidance)
- Compare with ExifTool using `third-party/exiftool/exiftool -j -struct -G` format
- Test complex scenarios: `-Orientation# -EXIF:all -GPS*` (overlapping filters)
- Validate case insensitivity: `-mimetype` vs `-MIMEType`
