# P16b: Universal Model-Based Subdirectory Dispatch Implementation

## Project Overview

- **Goal**: Generate working runtime dispatch code for camera model conditions across ALL manufacturers, enabling proper subdirectory table selection (e.g., Canon `$$self{Model} =~ /EOS 5D$/` → `CameraInfo5D`, Sony `$$self{Model} =~ /^NEX-5N$/` → `Tag2010a`)
- **Problem**: Tag kit generator recognizes model conditions for all manufacturers but generates placeholder comments instead of functional dispatch code, preventing camera-specific processing across Canon, Sony, Nikon, Olympus, and others
- **Constraints**: Must leverage existing expression evaluation system, maintain backward compatibility, trust ExifTool logic exactly, work universally across all camera manufacturers

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **Tag Kit Generator** (`codegen/src/generators/tag_kit_modular.rs`): Universal generator that creates runtime dispatch code from ExifTool tag tables across all manufacturers, converts Perl conditions to Rust match statements and conditional logic
- **Expression Evaluation System** (`src/expressions/`): Runtime engine for evaluating ExifTool conditional expressions, supports model matching, count comparisons, and complex boolean logic universally across manufacturers
- **Universal Model Detection** (`src/generated/*/main_model_detection.rs`): Generated per-manufacturer model detection code - FujiFilm already working, others need subdirectory dispatch
- **Manufacturer-Specific Subdirectories**: Each manufacturer uses model-specific conditions to select different processing tables based on camera model detection (Canon CameraInfo*, Sony Tag2010*, Nikon AFInfo2*, etc.)

### Key Concepts & Domain Knowledge

- **Subdirectory conditions**: ExifTool conditions that determine which subtable to process for a given tag (vs individual tag conditions that determine tag name/format)
- **Universal model pattern matching**: Perl regex patterns work identically across manufacturers - `/\b1DS?$/` (Canon), `/^NEX-5N$/` (Sony), `/^NIKON Z/` (Nikon)
- **Manufacturer-specific dispatch tables**: Canon has CameraInfo* tables, Sony has Tag2010* variants, Nikon has AFInfo2* versions, Olympus has AFPoint configurations
- **Compound conditions**: Some conditions combine multiple checks: `($$self{CameraInfoCount} = $count) and $$self{Model} =~ /\b1DS?$/`
- **ConditionalContext**: Runtime context containing model, count, format, and binary data for condition evaluation - works universally

### Surprising Context

**CRITICAL**: These non-obvious aspects will trip up casual implementers:

- **Individual vs Subdirectory conditions work differently**: Individual conditional tags (like SerialNumber model conditions) work perfectly across manufacturers via `main_conditional_tags.rs`, but subdirectory dispatch is completely unimplemented universally
- **Tag kit generator has explicit universal placeholder code**: `codegen/src/generators/tag_kit_modular.rs:1599-1604` recognizes `SubdirectoryCondition::Model` but deliberately generates comments instead of dispatch code FOR ALL MANUFACTURERS
- **Count conditions work, model conditions don't**: The same generator successfully handles `SubdirectoryCondition::Count` but skips `SubdirectoryCondition::Model` universally
- **Expression system is battle-tested universally**: MILESTONE-11 and conditional tags integration prove the expression evaluation works perfectly across all manufacturers - no need to rewrite, just wire it up
- **Hundreds of conditions waiting across manufacturers**: Canon (~100), Sony (~50), Nikon (~30), Olympus (~15) model-based subdirectory conditions all generate no-op placeholder comments
- **Infrastructure already exists for all manufacturers**: FujiFilm has working `main_model_detection.rs`, proving the system is designed generically
- **Same patterns across manufacturers**: All use `$$self{Model} =~ /pattern/` with manufacturer-specific regex patterns and dispatch table names

### Foundation Documents

- **ExifTool source examples**:
  - Canon: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` lines 1276-1422 (CameraInfo model conditions)
  - Sony: `third-party/exiftool/lib/Image/ExifTool/Sony.pm` lines 823-1127 (Tag2010* model conditions)
  - Nikon: `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` scattered model conditions for Z-series cameras
- **Working individual conditions**: `src/generated/*/main_conditional_tags.rs` shows model conditions working for individual tags across manufacturers
- **Working universal model detection**: `src/generated/FujiFilm_pm/main_model_detection.rs` proves the generic infrastructure works
- **Expression evaluation**: `docs/done/MILESTONE-11-Conditional-Dispatch.md` and `docs/done/20250719-conditional-tags-runtime-integration.md` document the working universal infrastructure
- **Start here**: `codegen/src/generators/tag_kit_modular.rs:1580-1620` contains the universal placeholder code that needs completion

### Prerequisites

- **Knowledge assumed**: Understanding of Rust pattern matching, ExifTool Perl syntax across manufacturers, regex compilation, and the existing expression evaluation system
- **Setup required**: Working `cargo t` environment, ability to run `make codegen` and test with images from multiple manufacturers

## Work Completed

- ✅ **Universal expression evaluation system** → MILESTONE-11 implemented comprehensive condition evaluation with ExpressionEvaluator, ProcessorContext, and regex caching working across all manufacturers
- ✅ **Individual conditional tags universal** → FujiFilm, Canon model conditions work perfectly for individual tag resolution across manufacturers
- ✅ **Count-based subdirectory dispatch universal** → Tag kit generator successfully handles count conditions across all manufacturers
- ✅ **Universal infrastructure research** → Confirmed all required components exist: ModelPattern parsing, ConditionalContext, runtime dispatch framework work generically
- ✅ **Universal gap identification** → Located exact source of problem: `SubdirectoryCondition::Model` case generates placeholder comments universally instead of functional code
- ✅ **Cross-manufacturer validation** → Confirmed Canon, Sony, Nikon, Olympus, Sigma, Panasonic all need model-based subdirectory dispatch

## Remaining Tasks

### 1. Task: Implement Universal Model Condition Code Generation

**Success Criteria**: Tag kit generator produces working Rust dispatch code for model conditions across ALL manufacturers, `cargo check` passes, generated code for Canon, Sony, Nikon contains `if` statements instead of placeholder comments
**Approach**: Replace placeholder code in `tag_kit_modular.rs:1599-1604` with universal runtime dispatch generation using existing expression evaluation patterns
**Dependencies**: None - all infrastructure exists and works universally

**Success Patterns**:
- ✅ Generated code contains `if context.model.as_ref().map(|m| model_regex.is_match(m)).unwrap_or(false)` patterns universally
- ✅ Complex conditions like `(count check) and (model check)` generate proper boolean logic across manufacturers
- ✅ Fallback default case handles unmatched models gracefully for all manufacturers
- ✅ Canon 100+ conditions, Sony 50+ conditions, Nikon 30+ conditions all generate functional dispatch code

### 2. Task: Handle Universal Complex Compound Conditions

**Success Criteria**: Compound conditions like `($$self{CameraInfoCount} = $count) and $$self{Model} =~ /\b1DS?$/` generate correct boolean expressions across all manufacturers
**Approach**: Parse compound conditions into separate count and model checks, combine with `&&` operator in generated code universally
**Dependencies**: Task 1 (basic model conditions)

**Success Patterns**:
- ✅ Assignment expressions `$$self{CameraInfoCount} = $count` handled correctly universally
- ✅ Boolean operators `and` converted to Rust `&&` across all manufacturers
- ✅ Generated code evaluates both conditions and combines results universally
- ✅ Variable assignments maintain ExifTool side-effect semantics across manufacturers

### 3. Task: Regenerate and Validate Multi-Manufacturer Code

**Success Criteria**: `make codegen` produces working tag kits for all manufacturers, Canon/Sony/Nikon test images properly dispatch to camera-specific tables
**Approach**: Run codegen, test with various manufacturer images, compare output with ExifTool using existing comparison tools
**Dependencies**: Tasks 1 and 2 (universal code generation implementation)

**Success Patterns**:
- ✅ `src/generated/Canon_pm/tag_kit/mod.rs` contains working dispatch code instead of comments
- ✅ `src/generated/Sony_pm/tag_kit/mod.rs` contains working Sony-specific model dispatch
- ✅ `src/generated/Nikon_pm/tag_kit/mod.rs` contains working Nikon Z-series dispatch
- ✅ Canon 1D/5D/7D, Sony A7/NEX, Nikon Z6/Z7/Z8 images dispatch to correct tables
- ✅ `compare-with-exiftool.sh` shows improved compatibility across all tested manufacturers
- ✅ No compilation errors in any generated manufacturer code

## Implementation Guidance

### Universal Code Generation Patterns

**Model Condition Template** (replace placeholder in `tag_kit_modular.rs` for ALL manufacturers):
```rust
SubdirectoryCondition::Model(pattern) => {
    let regex_pattern = &pattern.regex;
    let condition_check = if pattern.negated {
        format!("!context.model.as_ref().map(|m| model_regex_{}.is_match(m)).unwrap_or(false)", hash_id)
    } else {
        format!("context.model.as_ref().map(|m| model_regex_{}.is_match(m)).unwrap_or(false)", hash_id)
    };
    
    code.push_str(&format!("        if {} {{\n", condition_check));
    code.push_str(&format!("            debug!(\"Model condition matched: {}\");\n", regex_pattern));
    code.push_str(&format!("            return process_{}(data, byte_order);\n", table_fn_name));
    code.push_str("        }\n");
}
```

**Universal Regex Compilation**: Use lazy static patterns across all manufacturers:
```rust
static MODEL_REGEX_123: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b1DS?$").unwrap());
```

### ExifTool Translation Notes (Universal)

- **Perl `=~` operator** → `regex.is_match(string)` in Rust (same across all manufacturers)
- **Perl boolean `and`** → Rust `&&` (universal)
- **Assignment in conditions** → Handle `$$self{CameraInfoCount} = $count` as side effect + comparison (universal pattern)
- **Model field access** → `context.model` contains the camera model string from EXIF parsing (universal)

### Architecture Considerations

- **Leverage existing universal ConditionalContext**: Reuse model field from conditional tag system across manufacturers
- **Maintain universal fallback patterns**: Always include default case for unmatched conditions across all manufacturers
- **Universal error handling**: Invalid regex patterns should fail compilation, not runtime (applies to all manufacturers)
- **Universal performance**: Cache compiled regex patterns using LazyLock across all manufacturers

## Integration Requirements

- [x] **Activation**: Model conditions are processed automatically during tag kit generation for all manufacturers
- [ ] **Consumption**: Generated code across Canon, Sony, Nikon, Olympus actively uses model-based dispatch for manufacturer-specific tables
- [ ] **Measurement**: Images from multiple manufacturers show camera-specific tag extraction, `compare-with-exiftool` improves compatibility universally
- [ ] **Cleanup**: Placeholder comments replaced with functional dispatch code across all manufacturers

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes**: Images from Canon, Sony, Nikon, Olympus dispatch to correct camera-specific tables based on model
- ✅ **Default usage**: Model conditions work automatically across all manufacturers without configuration changes
- ✅ **Old path removed**: Placeholder comment generation replaced with functional dispatch universally
- ❌ Code exists but generates comments *(current universal state)*
- ❌ Model conditions "work if you call them directly" *(they need to be generated first for all manufacturers)*

## Prerequisites

- Expression evaluation system → MILESTONE-11 → verify with `cargo t expression`
- Conditional tags integration → conditional-tags-runtime-integration → verify with `cargo t conditional_tag_resolution`

## Testing

- **Unit**: Test model pattern parsing, regex compilation, compound condition handling across manufacturers
- **Integration**: Verify Canon/Sony/Nikon/Olympus images dispatch to correct manufacturer-specific tables based on model
- **Manual check**: Run `compare-with-exiftool.sh` on images from multiple manufacturers and confirm improved tag coverage universally

## Definition of Done

- [ ] `cargo check` passes after regenerating code for all manufacturers
- [ ] `make codegen` produces functional model dispatch code universally
- [ ] `compare-with-exiftool.sh` shows improved compatibility across Canon, Sony, Nikon, Olympus
- [ ] All placeholder "Model condition not yet supported" comments replaced with working code across ALL manufacturers
- [ ] Canon 1D/5D/7D, Sony A7/NEX/DSC, Nikon Z6/Z7/Z8, Olympus E-M families show camera-specific tag extraction

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

- **Generated code looks complex** → Model dispatch requires regex compilation and conditional logic → Use existing LazyLock pattern for cached regex, follow count condition template (universal approach)
- **Compound conditions confusing** → ExifTool mixes assignment and comparison in single expression → Parse assignment `=` separately from comparison `and`, handle both side effects and boolean result (universal pattern)
- **Some models not matching** → Each manufacturer has multiple name variants (Canon EOS 5D vs 5D, Sony NEX-5N vs NEX5N) → Trust ExifTool regex patterns exactly per manufacturer, don't try to "improve" them
- **Performance concerns** → Hundreds of regex patterns across manufacturers seems expensive → Regex compilation cached via LazyLock universally, evaluation is fast, only compiled once per manufacturer
- **Integration not working** → Model dispatch generated but not called → Verify ConditionalContext has model field populated from EXIF parsing state (universal issue)
- **Wrong manufacturer dispatch** → Canon image dispatching to Sony table → Ensure manufacturer detection works correctly before model-specific dispatch

## Quick Debugging

Stuck? Try these:

1. `grep -r "Model condition not yet supported" src/generated/` - Find remaining placeholder comments across all manufacturers
2. `rg "Model.*=~" third-party/exiftool/lib/Image/ExifTool/{Canon,Sony,Nikon}.pm` - See ExifTool source patterns across manufacturers
3. `cargo t model_detection -- --nocapture` - Test universal model detection logic
4. `RUST_LOG=debug compare-with-exiftool.sh canon-image.jpg` - See Canon dispatch decisions
5. `RUST_LOG=debug compare-with-exiftool.sh sony-image.arw` - See Sony dispatch decisions
6. `RUST_LOG=debug compare-with-exiftool.sh nikon-image.nef` - See Nikon dispatch decisions

## Manufacturer Impact Assessment

**High Impact** (100+ model conditions):
- ✅ Canon: ~100 CameraInfo model conditions for different camera families
- ✅ Sony: ~50 Tag2010* model conditions for A-series, NEX, DSC families

**Medium Impact** (10-50 model conditions):
- ✅ Nikon: ~30 Z-series and DSLR model conditions for AFInfo2*, Custom Settings
- ✅ Olympus: ~15 model conditions for different E-series and OM families

**Low Impact** (5-10 model conditions):
- ✅ Sigma: ~5 model conditions for SD1/DP Merrill/Quattro series
- ✅ Panasonic: ~8 model conditions for DMC/Leica series

**Expected Results**: Universal implementation will improve ExifTool compatibility significantly across all major camera manufacturers, with largest impact on Canon and Sony cameras.