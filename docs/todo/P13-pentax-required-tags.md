# Technical Project Plan: Pentax Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive Pentax MakerNotes tag extraction to support all required tags from supported_tags.json
- **Problem**: No Pentax-specific infrastructure exists - missing codegen extraction, tag kit system, and MakerNotes processor implementation
- **Constraints**: Must support Pentax K-mount DSLR legacy and recent models, handle complex AF point detection system and extensive lens database

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **No existing infrastructure**: Unlike other manufacturers, zero Pentax-specific codegen or processing exists in current codebase
- **DSLR specialist**: Pentax remains focused on K-mount DSLR cameras while others moved to mirrorless, requiring strong legacy support
- **Complex AF system**: Advanced AF point detection with model-specific configurations (K-3 III has 101-point system)
- **Market position**: Smaller but dedicated user base, significant archive of images requiring metadata extraction

### Key Concepts & Domain Knowledge

- **CryptShutterCount system**: Pentax uses encrypted shutter count requiring special decryption (CryptShutterCount function in Pentax.pm)
- **Extensive lens database**: 200+ lens entries in pentaxLensTypes covering K, A, F, FA, DFA, DA, DA* series plus third-party lenses
- **Multi-series lens identification**: Complex 2-number system (series, model) with firmware-dependent recognition issues
- **AF point complexity**: Multiple AF point layouts requiring model-specific DecodeAFPoints functions

### Surprising Context

- **Zero infrastructure**: Most other manufacturers have partial codegen/infrastructure, but Pentax has none - complete ground-up implementation needed
- **DSLR focus advantage**: Unlike mirrorless complexity, Pentax standardized on K-mount DSLR provides consistent MakerNotes structure
- **Firmware compatibility issues**: Older cameras may not recognize newer lens series properly, requiring fallback logic
- **Ricoh ownership**: Pentax is owned by Ricoh but maintains separate MakerNotes structure (different from src/implementations/ricoh.rs)

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Pentax.pm` - Main tag table, lens database lines 75-400+, AF point functions
- **No existing codegen**: `src/generated/Pentax_pm/` directory doesn't exist - needs complete extraction
- **Market context**: DSLR specialist with dedicated user base, strong in wildlife/outdoor photography
- **Start here**: Need to create complete extraction pipeline: codegen → tag kit → processor implementation

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD processing, DSLR vs mirrorless differences, K-mount lens system
- **Setup required**: Pentax test images (K-3 series, K-5 series, K-1) for comprehensive validation

**Context Quality Check**: Can a new engineer understand WHY Pentax requires complete ground-up implementation unlike other manufacturers?

## Work Completed

- ✅ **File type detection** → Basic PEF format detection exists in file_type_lookup
- (No other infrastructure exists)

## Remaining Tasks

### 1. Task: Research supported_tags.json requirements for Pentax MakerNotes

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from Pentax cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags, cross-reference with Pentax.pm Main table capabilities
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags that Pentax should provide
- ✅ Priority ranking based on tag frequency and importance for DSLR metadata
- ✅ Understanding of Pentax-specific tags vs generic MakerNotes requirements

### 2. Task: Implement complete Pentax codegen extraction

**Success Criteria**: `src/generated/Pentax_pm/` directory created with comprehensive tag kit system
**Approach**: Add Pentax.pm to codegen extraction system, generate tag tables, lens database, and AF point lookup tables
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/Pentax_pm/tag_kit/ directory with complete tag definitions
- ✅ pentaxLensTypes database extracted with 200+ lens entries
- ✅ AF point detection tables generated for major camera models
- ✅ CryptShutterCount function requirements documented

### 3. Task: Create Pentax MakerNotes processor implementation

**Success Criteria**: `src/implementations/pentax/mod.rs` processes Pentax MakerNotes using generated tag kit system
**Approach**: Follow Canon/Sony implementation patterns, handle DSLR-specific processing requirements
**Dependencies**: Task 2 (codegen extraction)

**Success Patterns**:
- ✅ process_pentax_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement critical Pentax-specific features

**Success Criteria**: CryptShutterCount decryption, lens identification, AF point detection working with ExifTool-compatible output
**Approach**: Translate key Pentax.pm functions (CryptShutterCount, DecodeAFPoints) to Rust following Trust ExifTool principles
**Dependencies**: Task 3 (MakerNotes processor)

**Success Patterns**:
- ✅ ShutterCount properly decrypted from encrypted value
- ✅ LensType extracted with proper lens names from pentaxLensTypes database
- ✅ AF point information extracted for supported camera models

### 5. Task: Validate against major Pentax DSLR models

**Success Criteria**: Tag extraction working across K-1, K-3, K-5 series cameras with comprehensive MakerNotes support
**Approach**: Test with sample files from different Pentax DSLR generations, ensure broad compatibility
**Dependencies**: Task 4 (Pentax-specific features)

**Success Patterns**:
- ✅ K-1 (full-frame flagship) extracts complete MakerNotes
- ✅ K-3 series (APS-C flagship) extracts complete MakerNotes with AF point detection
- ✅ K-5 series (legacy popular model) extracts complete MakerNotes
- ✅ Various PEF and JPEG samples work correctly

### 6. RESEARCH: Investigate Ricoh integration possibilities

**Objective**: Understand relationship between Pentax and Ricoh MakerNotes, ensure no conflicts with existing ricoh.rs
**Success Criteria**: Clear separation documented between Pentax and Ricoh processing
**Done When**: No conflicts between Pentax processor and existing Ricoh implementation

## Implementation Guidance

### Recommended Patterns

- **Ground-up implementation**: Unlike other manufacturers, Pentax needs complete extraction pipeline from scratch
- **DSLR-focused processing**: Leverage DSLR consistency advantage vs mirrorless complexity
- **Legacy compatibility**: Ensure processing works across wide range of Pentax DSLR generations

### Tools to Leverage

- **Codegen framework**: Use existing tag kit extraction system for Pentax.pm processing
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available
- **DSLR patterns**: Follow patterns from other DSLR-focused manufacturers where applicable

### Architecture Considerations

- **Complete separation from Ricoh**: Pentax processor must be independent from existing ricoh.rs implementation
- **Namespace isolation**: Pentax MakerNotes must use proper "MakerNotes" group
- **PEF format integration**: Ensure MakerNotes processing works for both JPEG and PEF files

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Pentax support.

Every feature must include:
- [ ] **Activation**: Pentax MakerNotes processing enabled by default for Pentax cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Pentax tag extraction with grep on output
- [ ] **Cleanup**: Complete infrastructure built, no placeholder stubs remaining

**Red Flag Check**: If Pentax images show zero MakerNotes tags, implementation is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Pentax images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Pentax tag extraction
- ✅ **Old path removed** - No placeholder infrastructure, complete extraction system working
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Partial implementation *(example: "basic tags work but lens identification missing")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Codegen system → verify with existing Canon/Sony tag kit generation

## Testing

- **Unit**: Test Pentax MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from K-1, K-3, K-5 sample files
- **Manual check**: Run `cargo run test-images/pentax/k1.pef` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t pentax` passes (if Pentax-specific tests exist)
- [ ] `make precommit` clean
- [ ] Complete Pentax codegen extraction in src/generated/Pentax_pm/
- [ ] Pentax MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] No conflict with existing Ricoh implementation

## Gotchas & Tribal Knowledge

### Pentax-Specific Considerations

- **Complete ground-up build**: Unlike other manufacturers with partial infrastructure, Pentax needs everything built from scratch
- **CryptShutterCount complexity**: Shutter count requires special decryption algorithm from ExifTool
- **Lens database size**: 200+ lens entries in pentaxLensTypes covering decades of K-mount lenses
- **AF point model dependency**: Different cameras have different AF point layouts requiring model-specific detection

### Implementation Challenges

- **Firmware compatibility**: Older cameras may not recognize newer lens series, requiring fallback logic
- **AF point complexity**: DecodeAFPoints function varies significantly between camera models
- **Legacy DSLR support**: Need to support wide range of Pentax DSLR generations from film-era compatibility to modern
- **Ricoh separation**: Ensure no conflicts with existing Ricoh implementation despite corporate ownership

### Codegen Requirements

- **Main tag table**: Extract comprehensive Main table from Pentax.pm
- **Lens database**: Extract pentaxLensTypes with all series and model numbers
- **AF point tables**: Extract model-specific AF point layouts
- **Special functions**: Document requirements for CryptShutterCount and DecodeAFPoints

## Quick Debugging

Stuck? Try these:

1. `ls src/generated/Pentax_pm/` - Verify codegen extraction completed
2. `cargo run test-images/pentax/sample.pef | grep MakerNotes | wc -l` - Count MakerNotes tags
3. `exiftool -j -G test-images/pentax/sample.pef | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_pentax" src/` - Check if Pentax processor is integrated into main pipeline
5. `rg "CryptShutterCount" third-party/exiftool/` - Reference ExifTool implementation for shutter count decryption