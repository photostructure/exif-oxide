# Technical Project Plan: P10a EXIF Required Tags

## Project Overview

- **Goal**: Achieve 90%+ tag extraction across 167 required tags for PhotoStructure production readiness
- **Problem**: Current 39% success rate (66/167 tags) with systematic gaps in XMP, MakerNotes, Composite, and IPTC processing
- **Constraints**: Prioritize systematic fixes over manufacturer-specific edge cases to maximize tag coverage wins

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

- **ExifTool compatibility**: Current test suite compares 167 required tags across 303 files using `make compat`
- **Tag extraction pipeline**: File detection → Format parser → Tag extraction → Value conversion → Output formatting
- **Missing systems**: XMP, complete MakerNotes, Composite calculations, IPTC processing represent 75+ missing tags

### Key Concepts & Domain Knowledge

- **Required tags**: Subset of ExifTool's 15,000+ tags prioritized for PhotoStructure DAM use cases
- **Systematic gaps**: Missing entire processing modules vs individual tag bugs indicate different fix strategies
- **Success rate calculation**: Working tags / Total tested tags across all test files

### Surprising Context

- **XMP completely absent**: 20+ XMP tags missing suggests no XMP packet extraction implemented
- **Composite calculations missing**: Dependencies like GPS coordinates, image dimensions not calculated
- **Canon lens identification broken**: 4 type mismatches in lens tags suggest systematic issue, not individual bugs
- **String encoding inconsistent**: Type mismatches in XPKeywords, GPSProcessingMethod indicate encoding problems

### Foundation Documents

- **Compatibility testing**: `tests/exiftool_compatibility_tests.rs` - structured reporting system
- **Test tool**: `cargo run --bin compare-with-exiftool` for file-specific debugging
- **Tag list**: `config/supported_tags_final.json` - 271 comprehensive tags for DAM use
- **ExifTool source**: `/third-party/exiftool/lib/Image/ExifTool/` modules for reference implementations

### Prerequisites

- **Knowledge assumed**: Familiarity with EXIF/metadata standards, Rust async/error patterns
- **Setup required**: `make precommit` passes, test images available in `test-images/` and `third-party/exiftool/t/images/`

## Work Completed

- ✅ **Enhanced compatibility testing** → structured reporting with clear failure categories
- ✅ **Value formatting fixes** → rational values now show decimals instead of arrays  
- ✅ **PrintConv pipeline** → Canon CameraSettings tags now execute conversion functions
- ✅ **String encoding fixes** → SubSec tags show integers instead of floats
- ✅ **Binary data display** → some binary tags show proper placeholder strings

## Remaining Tasks

### 1. Task: Implement XMP packet extraction → See [P18-xmp-required-tags.md](P18-xmp-required-tags.md)

**Current Status**: 20+ XMP tags missing (Country, DateTimeDigitized, LensInfo, FocalLength, etc.)
**Impact**: +20 tags → 51% success rate
**Priority**: High - Comprehensive 332-line TPP already exists with complete implementation plan

### 2. Task: Complete MakerNotes binary data extraction → See P16, P30, P52-P54 TPPs

**Current Status**: 30+ manufacturer-specific tags missing (SonyExposureTime, ShutterCount2/3, Rating, etc.)  
**Impact**: +30 tags → 69% success rate
**Related TPPs**:
- [P16-MILESTONE-19-Binary-Data-Extraction.md](P16-MILESTONE-19-Binary-Data-Extraction.md) - Binary extraction infrastructure
- [P30-MAKERNOTES-TODO.md](P30-MAKERNOTES-TODO.md) - Manufacturer status tracking
- [P52-MILESTONE-17d-Canon-RAW.md](P52-MILESTONE-17d-Canon-RAW.md) - Canon-specific implementation
- [P53-MILESTONE-17e-Sony-RAW.md](P53-MILESTONE-17e-Sony-RAW.md) - Sony-specific implementation
- [P54-MILESTONE-17f-Nikon-RAW-Integration.md](P54-MILESTONE-17f-Nikon-RAW-Integration.md) - Nikon-specific implementation

### 3. Task: Implement missing Composite tag calculations → See [P12](P12-composite-required-tags.md) + [P12b](P12b-complete-composite-lens-media-tags.md)

**Current Status**: 7 missing composite dependencies (GPS calculations, lens specs, file numbers)
**Impact**: +20 tags → 81% success rate  
**Priority**: Medium - P12 validation shows most composites already working, P12b addresses remaining lens/media tags

### 4. Task: Add IPTC metadata processing → See [P19-IPTC-metadata-processing.md](P19-IPTC-metadata-processing.md)

**Current Status**: 6 IPTC tags missing (Keywords, City, Source, ObjectName, DateTimeCreated, FileVersion)
**Impact**: +6 tags → 42% success rate
**Priority**: High - New dedicated TPP created with complete IPTC implementation plan

### 5. Task: Fix string encoding type mismatches → See [P17a-value-formatting-consistency.md](P17a-value-formatting-consistency.md)

**Current Status**: 7 type mismatches (XPKeywords, GPSProcessingMethod showing byte arrays instead of strings)
**Impact**: +7 tags → 46% success rate
**Focus Areas**: UTF-16/ASCII encoding conversion, Canon lens identification fixes

## Implementation Guidance

**Coordination Strategy**: P10a serves as coordination TPP - track overall progress while detailed work happens in dedicated TPPs

**Priority Order**: 
1. **P19 (IPTC)** - Simple binary format, clear +6 tag win, good learning project
2. **P18 (XMP)** - Largest impact (+20 tags), comprehensive TPP already exists  
3. **P17a (String encoding)** - Fixes 7 type mismatches, complements other work
4. **P16/P30/P52-54 (MakerNotes)** - Complex manufacturer-specific work, highest technical complexity
5. **P12/P12b (Composite)** - Mostly working, remaining dependency issues

**Testing Strategy**: Use `make compat` to track overall progress, individual TPPs handle detailed testing

## Testing

- **Progress tracking**: Run `make compat` after each TPP completion to measure success rate improvements
- **Integration**: Each referenced TPP includes comprehensive testing strategies
- **Cross-TPP validation**: Ensure tag extraction from one system (e.g., XMP) doesn't break composite calculations

## Definition of Done

- [ ] **P19 completion** → IPTC tags extracting (+6 tags, ~42% success rate)
- [ ] **P18 completion** → XMP tags extracting (+20 tags, ~54% success rate)  
- [ ] **P17a completion** → String encoding fixed (+7 tags, ~58% success rate)
- [ ] **P16/P30/P52-54 completion** → MakerNotes complete (+30 tags, ~78% success rate)
- [ ] **P12b completion** → All composites working (+20 tags, ~90% success rate target)
- [ ] `make precommit` clean across all implementations
- [ ] PhotoStructure integration validation with 90%+ tag coverage

## Gotchas & Tribal Knowledge

- **src/generated/ files**: Never edit directly, fix codegen configs instead
- **Missing Composite tags**: Check if prerequisites (GPS, MakerNotes data) are available first
- **XMP namespace complexity**: ExifTool handles multiple XMP schema, our parser must match this flexibility
- **Binary data endianness**: Manufacturer-specific byte order handling required for Sony/Canon/Panasonic tags
- **String encoding variety**: EXIF uses multiple string formats (ASCII, UTF-16, undefined) requiring careful detection

## Current Status (2025-07-30)

**Latest Compatibility Report**:
- **Files tested**: 303  
- **Unique tags tested**: 167
- **Success rate**: 39% (66/167 tags working)

**Failure Breakdown**:
- **86 missing tags**: XMP (20+), MakerNotes (30+), Composite (20+), IPTC (5+), other (11+)
- **7 missing composite dependencies**: GPS calculations, lens specs, file numbers
- **7 type mismatches**: String encoding issues, Canon lens identification
- **1 extra tag**: Only in exif-oxide, not ExifTool

**Coordinated TPP Impact Analysis**:
- **Current baseline**: 39% success rate (66/167 tags working)
- **P19 (IPTC)**: +6 tags → ~42% success rate
- **P18 (XMP)**: +20 tags → ~54% success rate  
- **P17a (String encoding)**: +7 tags → ~58% success rate
- **P16/P30/P52-54 (MakerNotes)**: +30 tags → ~78% success rate
- **P12/P12b (Composite)**: +20 tags → **90% success rate target achieved**

**Strategic Approach**: This coordination model leverages existing comprehensive TPPs instead of duplicating detailed implementation plans, focusing P10a on progress tracking and impact measurement across the entire required tags initiative.