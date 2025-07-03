# Milestone 8f: Composite Tag Implementation

**Goal**: Complete ExifTool-compatible composite tag system for mainstream metadata tags

**Status**: ✅ Infrastructure Complete - Individual implementations in progress

**Duration**: 4-6 weeks (infrastructure: 2 weeks complete, implementations: 2-4 weeks remaining)

## Summary

ExifTool's composite tag system computes derived metadata by combining multiple extracted tags through complex dependency resolution. Examples include ImageSize (width × height), GPSAltitude (altitude + sea level reference), and complex photography calculations like DOF (depth of field).

This milestone implements ExifTool's sophisticated multi-pass composite building algorithm with full dependency tracking, then systematically implements the ~50 mainstream composite tags identified through frequency analysis.

## Architecture Context

### How This Fits exif-oxide's Design

**Codegen Strategy**: Follows the established pattern of generating tag definitions while manually implementing complex logic:
- ✅ **Generated**: 63 composite tag definitions with dependencies and ValueConv references  
- ✅ **Manual**: Individual computation functions that translate Perl ValueConv expressions verbatim
- ✅ **Mainstream Focus**: Only implements composite tags with >80% frequency or mainstream=true

**Trust ExifTool Principle**: Every computation function directly translates ExifTool's Perl ValueConv expressions without "optimization" or "simplification". Source references included for all implementations.

**Graceful Degradation**: Missing implementations return None rather than panicking, allowing system to remain usable during incremental development.

## Current Status

### ✅ Completed Infrastructure (100% Complete)

1. **Multi-Pass Dependency Resolution** (`src/composite_tags.rs`)
   - ExifTool-compatible BuildCompositeTags algorithm with circular dependency detection
   - Group-prefixed tag resolution (GPS:GPSLatitude vs GPSLatitude)
   - Progress tracking and unresolved composite handling
   - **ExifTool source**: lib/Image/ExifTool.pm BuildCompositeTags() lines 3904-4100

2. **Code Generation Pipeline** (`codegen/extract_tables.pl`)
   - Extracts 63 composite definitions from ExifTool Main, EXIF, and GPS tables
   - Generates `CompositeTagDef` structures with require/desire dependencies
   - Includes ValueConv expressions and PrintConv references
   - Mainstream filtering applied (frequency >8% for composites vs 80% for regular tags)

3. **Runtime Integration** (`src/exif.rs`)
   - `build_composite_tags()` facade calling into composite_tags module
   - "Composite:" prefix integration with `get_all_tags()`
   - PrintConv support via `apply_composite_conversions()`
   - Debug logging for dependency tracking

### ✅ Working Implementations (16% Complete - 10/63 tags)

**Basic Computations**:
- **ImageSize** - `"$val[0] x $val[1]"` → width × height formatting
- **PreviewImageSize** - Same logic for preview dimensions  
- **GPSAltitude** - `"$val[0] $val[1]"` → altitude with sea level reference

**Photography Calculations**:
- **ShutterSpeed** - `"$val[0] || $val[1]"` → fallback between multiple shutter representations
- **Aperture** - `"$val[0] || $val[1]"` → FNumber or ApertureValue fallback
- **FocalLength35efl** - `"$val[0] * $val[1]"` → focal length × scale factor

**Date/Time Combinations**:
- **DateTimeOriginal** - `"$val[0] $val[1]"` → date + time field combination
- **SubSecDateTimeOriginal** - Complex datetime with subseconds and timezone

**Partial Implementations**:
- **ScaleFactor35efl** - Placeholder returning 1.0 (needs full CalcScaleFactor35efl port)
- **CircleOfConfusion** - Basic calculation (needs sensor size refinements)

### ❌ Remaining Work (84% - 53/63 tags)

**High Priority Mainstream Tags** (need implementation):
1. **RedBalance/BlueBalance** - `Image::ExifTool::Exif::RedBlueBalance(1,@val)` white balance calculations
2. **LightValue** - `Image::ExifTool::Exif::CalculateLV($val[0],$val[1],$val[2])` complex photography math
3. **Megapixels** - `$val =~ /(\d+\.?\d*) x (\d+\.?\d*)/ ? $1 * $2 / 1000000` from ImageSize
4. **GPSPosition** - `"$val[0] $val[1]"` coordinate combination
5. **FOV** (Field of View) - Complex trigonometric calculation with focus distance
6. **HyperfocalDistance** - `$val[0] * $val[0] / ($val[1] * $val[2] * 1000)` depth of field math

**Complex Function Dependencies** (require manual porting):
1. **CalcScaleFactor35efl** - 150-line ExifTool function handling sensor size calculations
2. **RedBlueBalance** - White balance array processing with fallback logic  
3. **CalculateLV** - Light value calculation: `log(aperture² × 100 / (shutter × ISO)) / log(2)`

**Medium Priority Tags**:
- **CFAPattern** - Color Filter Array validation and formatting
- **LensID** - Complex lens identification from multiple fallback sources
- **DOF** (Depth of Field) - Multi-parameter photography calculation
- **ThumbnailImage/PreviewImage** - Binary data extraction composites

**Lower Priority Tags** (less common):
- Various binary data composites (JpgFromRaw, OtherImage)
- Specialized photography calculations (ModifiedSensorSize, etc.)

**Technical Debt**:
- Test snapshot updates for "Composite:" prefix (causing 2 compatibility test failures)
- PrintConv implementations for composite tags that reference complex functions

## Key Deliverables

### Phase 1: High-Priority Mainstream Tags (2 weeks)
1. **RedBalance/BlueBalance** - Port RedBlueBalance() function for white balance calculations
2. **Megapixels** - Simple regex parsing from ImageSize composite  
3. **GPSPosition** - Coordinate combination with proper formatting
4. **LightValue** - Port CalculateLV() mathematical function
5. **HyperfocalDistance** - Photography depth calculation

### Phase 2: Complex Mathematical Functions (2 weeks)
1. **CalcScaleFactor35efl** - Port full 150-line ExifTool sensor calculation function
2. **FOV** (Field of View) - Trigonometric calculations with focus distance correction  
3. **DOF** (Depth of Field) - Multi-parameter photography calculations
4. **CFAPattern/LensID** - Complex identification and validation logic

### Phase 3: Polish & Validation (1 week)
1. Fix compatibility test snapshots for "Composite:" prefix
2. Implement missing PrintConv functions for composite tags
3. Comprehensive validation testing with real-world images
4. Performance optimization pass

## Success Criteria

**Technical**:
- All 6 high-priority mainstream composite tags implemented
- CalcScaleFactor35efl function fully ported with sensor size logic
- Zero panics or crashes when processing composite tags
- `make compat` tests pass with composite tag support

**Compatibility**:
- Output matches ExifTool for all implemented composite tags
- Graceful degradation for unimplemented tags (raw value display)
- Debug logging shows clear progress on missing vs available dependencies

**Quality**:
- All complex calculations include ExifTool source references (file:line)
- Test coverage for edge cases (missing dependencies, invalid values)
- Documentation updated with implementation examples

## Implementation Guide

### Adding a New Composite Tag

1. **Find the Definition in Generated Code**
   ```rust
   // In src/generated/composite_tags.rs, locate your tag:
   CompositeTagDef {
       name: "Megapixels",
       value_conv: Some("$val =~ /(\\d+\\.?\\d*) x (\\d+\\.?\\d*)/ ? $1 * $2 / 1000000 : undef"),
       require: &[],
       desire: &[(0, "ImageSize")],
       // ... dependencies and metadata
   }
   ```

2. **Add to compute_composite_tag Match**
   ```rust
   // In src/composite_tags.rs around line 154
   match composite_def.name {
       // ... existing implementations ...
       "Megapixels" => compute_megapixels(available_tags),
       _ => { /* fallback for unimplemented */ }
   }
   ```

3. **Implement Computation Function**
   ```rust
   /// Compute Megapixels from ImageSize
   /// ExifTool: lib/Image/ExifTool/Composite.pm line 45
   /// ValueConv: $val =~ /(\d+\.?\d*) x (\d+\.?\d*)/ ? $1 * $2 / 1000000 : undef
   fn compute_megapixels(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
       let image_size = available_tags.get("ImageSize")?.as_string()?;
       
       // Trust ExifTool: translate regex exactly as written
       let re = regex::Regex::new(r"(\d+\.?\d*) x (\d+\.?\d*)").ok()?;
       let caps = re.captures(&image_size)?;
       
       let width: f64 = caps.get(1)?.as_str().parse().ok()?;
       let height: f64 = caps.get(2)?.as_str().parse().ok()?;
       
       Some(TagValue::F64(width * height / 1_000_000.0))
   }
   ```

### Trust ExifTool Translation Patterns

| Perl Expression | Rust Translation | Notes |
|----------------|------------------|--------|
| `$val[0] \|\| $val[1]` | First non-None fallback | Use `.or()` or conditional logic |
| `"$val[0] $val[1]"` | `format!("{} {}", val0, val1)` | String interpolation → format macro |
| `$val[0] * $val[1]` | Arithmetic with as_f64() | Handle type conversion explicitly |
| `$val =~ /(\d+)\.(\d+)/` | `regex::Regex::new(r"(\d+)\.(\d+)")` | Translate regex patterns exactly |
| `defined($val)` | `Option::is_some()` | Perl's defined → Rust Option |
| `$val{FocalLength}` | `available_tags.get("FocalLength")` | Hash access → HashMap lookup |

**Critical**: Never "optimize" or "improve" the Perl logic. If ExifTool checks for the same condition twice, do it twice in Rust.

### Development Workflow

1. **Find Missing Implementation**
   ```bash
   # Use trace logging to see what's missing
   RUST_LOG=trace cargo run -- test-images/canon.jpg 2>&1 | grep "not yet implemented"
   ```

2. **Research ExifTool Source**
   ```perl
   # In ExifTool's lib/Image/ExifTool/Composite.pm, find:
   Megapixels => {
       Require => 'ImageSize',
       ValueConv => '$val =~ /(\d+\.?\d*) x (\d+\.?\d*)/ ? $1 * $2 / 1000000 : undef',
   },
   ```

3. **Implement with Source Reference**
   ```rust
   /// ExifTool: lib/Image/ExifTool/Composite.pm line 45
   /// ValueConv: $val =~ /(\d+\.?\d*) x (\d+\.?\d*)/ ? $1 * $2 / 1000000 : undef
   fn compute_megapixels(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
       // Implementation here...
   }
   ```

4. **Validate Against ExifTool**
   ```bash
   # Compare outputs
   cargo run -- test.jpg | grep "Composite:Megapixels"
   exiftool -Composite:Megapixels test.jpg
   ```

## Complex Function Porting Strategy

### CalcScaleFactor35efl (Priority 1)

**ExifTool Source**: lib/Image/ExifTool/Exif.pm lines 5345-5500

**Complexity**: 150-line function handling:
- Canon-specific sensor size calculations
- Focal plane resolution unit conversions  
- Image dimension validation with fallbacks
- Multiple manufacturer-specific quirks

**Implementation Approach**: Port function verbatim with extensive ExifTool source comments.

### RedBlueBalance (Priority 2) 

**ExifTool Source**: lib/Image/ExifTool/Exif.pm RedBlueBalance()

**Complexity**: White balance level array processing with manufacturer-specific format handling.

**Implementation Approach**: Translate Perl array manipulation logic directly, preserving all edge case handling.

## Risk Mitigation

**Incremental Progress**: Infrastructure complete means system remains functional throughout development. Each new tag implementation provides immediate value.

**Scope Management**: Focus on 63 mainstream composite tags (vs ExifTool's 200+ total). Frequency-based prioritization ensures maximum impact.

**Compatibility Assurance**: Direct Perl-to-Rust translation with ExifTool source references ensures exact behavior match. Comprehensive test validation catches regressions.

## References

### ExifTool Source Files
- **Primary**: `lib/Image/ExifTool/Composite.pm` - All composite tag definitions
- **Functions**: `lib/Image/ExifTool/Exif.pm` - CalcScaleFactor35efl, RedBlueBalance, CalculateLV
- **Algorithm**: `lib/Image/ExifTool.pm` BuildCompositeTags() lines 3904-4100

### exif-oxide Implementation Files  
- **Generated Definitions**: `src/generated/composite_tags.rs` - 63 CompositeTagDef structures
- **Runtime Logic**: `src/composite_tags.rs` - Multi-pass resolution and computation dispatch
- **Integration**: `src/exif.rs` - ExifReader facade methods

### Documentation
- **Architecture**: `docs/ARCHITECTURE.md` - How composites fit the overall design
- **Trust ExifTool**: `docs/TRUST-EXIFTOOL.md` - Translation principles  
- **Testing**: Test images in `test-images/` for validation

## Dependencies

**Blocked by**: None - Infrastructure complete

**Blocks**: 
- Milestone 15 (Performance Analysis) - needs composite coverage metrics
- Advanced photography applications - depend on FOV, DOF, HyperfocalDistance calculations

## Completion Criteria

1. ✅ **Infrastructure** - Multi-pass dependency resolution working
2. ⏳ **High Priority Tags** - RedBalance, Megapixels, GPSPosition, LightValue, HyperfocalDistance  
3. ⏳ **Complex Functions** - CalcScaleFactor35efl, RedBlueBalance ported
4. ⏳ **Test Compatibility** - All `make compat` tests pass with composite tags
5. ⏳ **Quality Assurance** - ExifTool source references for all implementations

**Next Steps**: Begin with Megapixels implementation (simplest regex parsing) to establish pattern, then tackle RedBalance (most complex function dependency).