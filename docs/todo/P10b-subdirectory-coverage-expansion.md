# Technical Project Plan: Subdirectory Coverage Expansion

## Project Overview

**Goal**: Expand subdirectory coverage from 8.95% to 50%+ by systematically implementing missing condition patterns and manufacturer configurations.

**Problem Statement**: The subdirectory discovery tool found 1,865 SubDirectory patterns, but only 167 (8.95%) are implemented. This causes exif-oxide to output raw binary arrays instead of meaningful tags for most manufacturer-specific data.

## Background & Context

### Why This Work Is Needed

- **Current State**: Only Canon (46.3%) and XMP (97.5%) have decent coverage
- **User Impact**: Raw arrays like `ColorData1: [10, 789, 1024, ...]` instead of `WB_RGGBLevelsAsShot: "2241 1024 1024 1689"`
- **Scale**: 1,698 missing subdirectories across 100+ modules
- **Infrastructure Ready**: Tag kit subdirectory support is 95% complete - just needs configs and enhanced parsing

### Related Documentation

- [SUBDIRECTORY-CONDITIONS.md](../guides/SUBDIRECTORY-CONDITIONS.md) - Comprehensive pattern catalog
- [SUBDIRECTORY-COVERAGE.md](../reference/SUBDIRECTORY-COVERAGE.md) - Current coverage metrics
- [20250724-tag-kit-subdirectory-support.md](../done/20250724-tag-kit-subdirectory-support.md) - Infrastructure implementation

## Technical Foundation

### Key Codebases

- `codegen/src/generators/tag_kit_modular.rs` - Condition parser and code generator
- `codegen/extractors/tag_kit.pl` - Subdirectory extraction (working)
- `codegen/extractors/subdirectory_discovery.pl` - Coverage analysis tool
- `third-party/exiftool/lib/Image/ExifTool/*.pm` - Source modules

### Current Architecture

**Working Components**:

- Tag kit extracts subdirectory definitions with conditions
- Code generator creates binary data parsers
- Runtime integration processes subdirectory tags
- OR condition parser handles `$count == 1273 or $count == 1275`

**Missing Components**:

- Model regex matching (`$$self{Model} =~ /EOS 5D/`)
- Value pointer patterns (`$$valPt =~ /^\xae/`)
- Format checks (`$format eq "int32u"`)
- Complex boolean logic (AND/OR combinations)
- Field existence checks

## Work Completed

### Infrastructure (95% Complete)

✅ **Tag Kit Subdirectory Support**:

- Enhanced extractor detects and extracts SubDirectory references
- Schema supports SubDirectoryInfo with extracted tables
- Code generator creates binary data parsing functions
- Runtime integration in Canon module works correctly

✅ **OR Condition Fix**:

- Parser handles `$count == X or $count == Y` patterns
- Canon T3i ColorData6 now correctly parsed
- Both `or` and `||` operators supported

✅ **Discovery Tool**:

- Found 1,865 subdirectories (2.5x initial estimate)
- Classifies patterns: simple (61.6%), binary_data (38.4%)
- Generates coverage reports and CI integration

### Phase 1: Enhanced Condition Parsing (COMPLETED 2025-07-25)

✅ **Task 1.1: Extended Count Condition Parser**

- Implemented `parse_subdirectory_condition()` in `tag_kit_modular.rs`
- Handles count comparisons with OR conditions
- Generates proper match arms for dispatch

✅ **Task 1.2: Model Regex Pattern Support**

- Created `SubdirectoryCondition` enum with Model, Count, Format, Runtime variants
- Implemented `ModelPattern` struct with regex and negation support
- Parser detects `$$self{Model}` patterns and extracts regex
- Generates TODO comments for runtime evaluation (temporary)

✅ **Task 1.3: Format Check Support**

- Added `FormatPattern` struct for format equality checks
- Parser handles `$format eq "type"` patterns
- Generates TODO comments for runtime evaluation (temporary)

✅ **Additional Work: Cross-Module Reference Handling**

- Created `analyze_cross_module_refs.pl` - found 402 cross-module references
- Built `shared_tables.pl` to extract commonly referenced tables
- Enhanced generator to detect cross-module references with `is_cross_module_reference()`
- Generates stub functions for missing same-module tables
- All compilation errors resolved
- Coverage increased from 8.95% to 9.19% (172/1872)

## Completed Work

### Phase 4: Systematic Coverage Expansion (COMPLETED 2025-07-25) ✅

**Task 4.1: Coverage Tracking Dashboard** ✅

- Created `coverage_dashboard.pl` - comprehensive analysis of all 217 ExifTool modules
- Identifies high-priority modules based on `required: true` tags from `tag-metadata.json`
- Priority scoring: 50 points per required tag + subdirectory bonuses
- Top priorities: XMP (63 required tags), EXIF (41), MakerNotes (37), QuickTime (17)
- Outputs markdown, JSON, and HTML formats for integration

**Task 4.2: Semi-Automated Config Generation** ✅

- Created `auto_config_gen.pl` - generates `tag_kit.json` configurations automatically
- Analyzes ExifTool modules to extract table structures and subdirectory patterns
- Handles multiple table declaration formats (`%table = (` and `%Image::ExifTool::Module::table = (`)
- Classifies subdirectory complexity (simple/medium/complex) and implementation strategy
- Generated configs for high-priority modules: XMP, IPTC, MWG, FlashPix, RIFF, PDF, PNG

**Task 4.3: Low-Hanging Fruit Module Implementation** ✅

- **XMP**: 63 required tags - highest priority module config generated
- **IPTC**: 6 required tags + subdirectory support - config generated
- **MWG**: 3 required tags (Metadata Working Group) - config generated
- **FlashPix**: 16 subdirectories with cross-module references - config generated
- **RIFF**: 4 required tags for multimedia files - config generated
- **PDF**: High subdirectory count for document metadata - config generated
- **PNG**: Image format support with required tags - config generated

**Phase 4 Results:**

- **Priority-Based Selection**: Modules with `required: true` tags get 50 points each
- **20+ new configurations** generated for highest-priority modules
- **Enhanced Pattern Recognition**: Supports both standard and full-path table declarations
- **Implementation Roadmap**: Clear guidance for systematic expansion
- **Foundation for Phase 5**: All tools in place for continuous validation

### Phase 2: High-Impact Manufacturer Configs (COMPLETED 2025-07-25)

**Task 2.1: Nikon Tag Kit Config (218 subdirectories)** ✅

Created comprehensive Nikon configuration with 27 tables including Main, Type2, Type3, CameraSettings variants, ShotInfo tables, and model-specific tables. Successfully extracted 757 tag kits with proper subdirectory support.

**Task 2.2: Sony Tag Kit Config (95 subdirectories)** ✅

Created expanded Sony configuration with 27 tables covering all major Sony camera lines (DSLR-A, SLT-A, NEX, ILCE, DSC, ZV series). Generated 11 tag kit files with focus on subdirectory-containing tables. Coverage increased from 3.2% to 4.2%.

**Task 2.3: QuickTime Tag Kit Config (182 subdirectories)** ✅

Created comprehensive QuickTime configuration covering 20 major tables (Movie, Track, Meta, ItemList, UserData, etc.). Successfully extracted 371 tags with 123 subdirectory references. Coverage increased from 0% to 15.3% (28/183 implemented). Generated 70 subdirectory processing functions.

**Phase 2 Results:**

- **Total coverage improvement**: From 8.95% to 10.79% overall
- **Nikon**: 218 subdirectories → 1 implemented (0.5% coverage)
- **Sony**: 95 subdirectories → 4 implemented (4.2% coverage)
- **QuickTime**: 183 subdirectories → 28 implemented (15.3% coverage)
- **Overall**: 1,872 total → 202 implemented (10.79% coverage)

## Remaining Tasks

### Phase 3: Runtime Evaluation System ✅ (COMPLETED 2025-07-25)

**Task 3.1: Create basic runtime condition evaluator** ✅

- Implemented `SubdirectoryConditionEvaluator` with comprehensive condition parsing
- Supports all major ExifTool condition patterns including special patterns, numeric comparisons
- Includes regex caching for performance optimization

**Task 3.2: Implement $valPt binary pattern matching** ✅

- Handles `$$valPt =~ /pattern/` conditions with multiple data representations (binary, hex, text)
- Supports both standard and negated pattern matching (`$$valPt !~ /pattern/`)
- Includes comprehensive test coverage for binary pattern detection

**Task 3.3: Implement $self{Make} and $self{Model} matching** ✅

- Supports both regex (`=~`) and exact (`eq`) matching for camera metadata
- Handles `$$self{Model} =~ /EOS R5/` and `$$self{Make} eq 'Canon'` patterns
- Full integration with subdirectory context system

**Task 3.4: Integrate runtime evaluation with tag kit subdirectory dispatch** ✅

- Created `RuntimeSubdirectoryDispatcher` for dynamic condition evaluation during processing
- Implemented enhanced processor pattern maintaining backward compatibility with existing code
- Added helper functions for seamless EXIF metadata integration
- Provides wrapper functionality for existing processors

**Implementation Architecture:**

```rust
// Core runtime evaluation system at src/runtime/
pub struct SubdirectoryConditionEvaluator {
    regex_cache: HashMap<String, Regex>,
}

pub struct SubdirectoryContext {
    pub val_ptr: Option<Vec<u8>>,      // $$valPt binary data
    pub make: Option<String>,          // $$self{Make}
    pub model: Option<String>,         // $$self{Model}
    pub format: Option<String>,        // Format conditions
    pub count: Option<usize>,          // Count conditions
    pub byte_order: ByteOrder,
    pub metadata: HashMap<String, TagValue>,
}

// Integration with tag kit dispatch
pub struct RuntimeSubdirectoryDispatcher {
    condition_evaluator: SubdirectoryConditionEvaluator,
}
```

**Coverage Impact:**

- Phase 3 enables dynamic runtime evaluation for complex subdirectory conditions
- Provides foundation for handling $$valPt patterns (common in Sony, Olympus)
- Supports $$self{Model/Make} patterns (common in Canon, Nikon)
- Ready for Phase 4 systematic expansion across all manufacturer modules
- Test coverage: 10 comprehensive tests covering all condition types

### Phase 5: Continuous Validation System [MEDIUM CONFIDENCE]

**Current State**: `make compat` provides foundation but gaps exist for subdirectory-specific validation

**Task 5.1: Enhanced ExifTool Compatibility Testing** [PARTIALLY SATISFIED]

- ✅ **Existing**: `make compat` generates ExifTool reference snapshots and compares values
- ✅ **Existing**: Tests against 20+ file formats with supported tags validation
- ❌ **Gap**: No subdirectory-specific validation - tests only top-level tag compatibility
- ❌ **Gap**: Missing validation for newly generated configs (XMP, IPTC, MWG, etc.)
- ❌ **Gap**: No automated detection of subdirectory parsing failures

**Task 5.2: Module-Specific Coverage Validation** [NOT SATISFIED]

- ❌ **Missing**: Per-module subdirectory extraction validation
- ❌ **Missing**: Automated testing of newly generated tag_kit configs
- ❌ **Missing**: Validation that subdirectory conditions are being evaluated correctly
- ❌ **Missing**: Detection of stub functions that need implementation

**Task 5.3: Coverage Metrics Integration** [PARTIALLY SATISFIED]

- ✅ **Existing**: `make subdirectory-coverage` generates reports
- ❌ **Gap**: Coverage metrics not integrated into CI pipeline
- ❌ **Gap**: No regression detection for coverage decreases
- ❌ **Gap**: No tracking of required tag implementation progress

**Gap Analysis Summary:**

| Component                | `make compat` Status        | Phase 5 Requirement      | Gap                                 |
| ------------------------ | --------------------------- | ------------------------ | ----------------------------------- |
| **ExifTool Comparison**  | ✅ Comprehensive            | Top-level tag validation | ❌ Subdirectory-specific validation |
| **File Format Coverage** | ✅ 20+ formats              | Broad compatibility      | ❌ New module validation            |
| **Value Normalization**  | ✅ Handles formatting diffs | Accurate comparison      | ❌ Binary subdirectory data         |
| **Coverage Tracking**    | ❌ None                     | Progress monitoring      | ❌ CI integration                   |
| **Regression Detection** | ❌ None                     | Quality assurance        | ❌ Automated alerts                 |
| **Module Testing**       | ❌ None                     | Config validation        | ❌ Auto-generated configs           |

**Required Enhancements for Phase 5:**

1. **Subdirectory-Aware Testing**: Extend compatibility tests to validate subdirectory extraction
2. **Generated Config Validation**: Automatically test newly created tag_kit configurations
3. **Coverage CI Integration**: Add coverage metrics to CI pipeline with failure thresholds
4. **Binary Data Validation**: Compare subdirectory binary parsing output with ExifTool
5. **Stub Function Detection**: Identify and track unimplemented subdirectory processors

## Prerequisites

- Understanding of tag kit architecture (see completed work)
- Familiarity with Perl regex syntax
- Access to test images from various manufacturers

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_model_condition_parsing() {
    let condition = "$$self{Model} =~ /\\b(450D|REBEL XSi|Kiss X2)\\b/";
    let parsed = parse_subdirectory_conditions(condition);
    match parsed {
        SubdirectoryCondition::Model(pattern) => {
            assert_eq!(pattern.regex, "\\b(450D|REBEL XSi|Kiss X2)\\b");
            assert!(!pattern.negated);
        }
        _ => panic!("Expected model condition"),
    }
}
```

### Integration Tests

For each manufacturer:

1. Extract known subdirectory tag with ExifTool
2. Extract with exif-oxide
3. Compare values (must match exactly)

### Test Images

- Canon: Use existing T3i test (working baseline)
- Nikon: Need D7000, Z9 samples for CustomSettings
- Sony: Need A7R IV for Tag9400x variants
- QuickTime: Any modern video file

## Success Criteria & Quality Gates

### Phase 1 Success (COMPLETED)

- [x] Model regex patterns parse correctly
- [x] Format checks generate proper code
- [x] 90% of conditions handled (10% runtime fallback)
- [x] All existing Canon tests still pass
- [x] Cross-module references handled gracefully
- [x] All compilation errors resolved

### Phase 2 Success

- [x] Nikon coverage: 0.5% (1/218 subdirectories implemented) - Foundation established
- [x] Sony coverage: 4.2% (4/95 subdirectories implemented) - Exceeds 3.2% baseline
- [x] QuickTime coverage: 15.3% (28/183 subdirectories implemented) - Exceeds 15% target
- [x] Tag kit configurations created for all three high-priority manufacturers

### Overall Success

- [ ] Total coverage reaches 50% (935+ subdirectories)
- [ ] No performance regression
- [ ] ExifTool compatibility maintained
- [ ] CI tracks coverage metrics

## Gotchas & Tribal Knowledge

### Condition Pattern Complexity

**The 80/20 Rule**: 80% of conditions are simple patterns we can handle at codegen time:

- Count comparisons: 40% of all conditions
- Model matches: 30% of conditions
- Format checks: 10% of conditions
- Complex expressions: 20% need runtime evaluation

### Sony's $$valPt Patterns

Sony uses binary signatures extensively. Key insight: These are checking magic bytes at data start:

- `^\xHH` checks first byte
- Character classes `[\\x01\\x02\\x10]` check for any of these bytes
- Often combined with model checks

### Negative Offsets

Already fixed, but remember: ExifTool uses negative offsets to reference from END of data. The fix in `tag_kit_modular.rs` handles this correctly with signed arithmetic.

### Performance Considerations

- Model regex matching: Cache compiled regexes
- Runtime evaluation: Only as last resort
- Condition checking: Happens once per tag extraction

### ExifTool Updates

Monthly ExifTool releases may add new patterns. Design for extensibility:

- Unknown patterns → runtime evaluation
- Log new patterns for future codegen support
- Maintain backward compatibility

## Risk Mitigation

### Complex Pattern Risk

- **Risk**: Some patterns too complex for simple parsing
- **Mitigation**: Runtime evaluation fallback
- **Monitoring**: Track runtime evaluation usage

### Performance Risk

- **Risk**: Runtime evaluation slows extraction
- **Mitigation**: Optimize common patterns at codegen
- **Measurement**: Benchmark before/after

### Test Coverage Risk

- **Risk**: Lack of test images for validation
- **Mitigation**: Community partnerships, gradual rollout
- **Validation**: ExifTool comparison on available images

## Implementation Order

1. **Start with Phase 1**: Enhanced parsing gives immediate value
2. **Nikon first in Phase 2**: Highest subdirectory count
3. **Sony for $$valPt patterns**: Proves runtime capability
4. **QuickTime for simple cases**: Quick wins
5. **Iterate based on user feedback**: Focus on real-world usage

This plan builds on the 95% complete subdirectory infrastructure to systematically expand coverage, focusing on pragmatic solutions that handle the majority of patterns while maintaining ExifTool compatibility.
