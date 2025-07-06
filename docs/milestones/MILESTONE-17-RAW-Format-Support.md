# Milestone 17: RAW Image Format Support (Consolidated)

**Duration**: 4-5 weeks  
**Goal**: Implement comprehensive RAW image support for all mainstream manufacturers

## Overview

This milestone consolidates RAW format support for Canon, Nikon, Sony, Olympus, Fujifilm, and Panasonic into a unified implementation. Analysis shows that despite manufacturer differences, most RAW formats share a common TIFF-based foundation that enables efficient code reuse.

## Background Analysis

**Common TIFF Foundation**: Analysis of ExifTool modules reveals shared patterns:

- **TIFF-based formats**: CR2, NEF, ARW, ORF, RW2 all use TIFF containers
- **Shared infrastructure**: `ProcessTIFF`, `ProcessBinaryData`, standard EXIF processing
- **DNG precedent**: Adobe's DNG format successfully handles multiple RAW types in single module
- **Complexity varies significantly**: Nikon (14K lines) vs Fujifilm (2K lines)

## Implementation Strategy: Foundation-First Approach

### Phase 1: RAW Processing Foundation (Week 1-2)

**Unified RAW Container Parser**:

```rust
pub struct RawProcessor {
    tiff_processor: TiffProcessor,
    maker_note_router: MakerNoteRouter,
    format_handlers: HashMap<RawFormat, Box<dyn RawFormatHandler>>,
}

// Common foundation for all TIFF-based RAW formats
impl RawProcessor {
    pub fn process_raw_file(&mut self, reader: &mut ExifReader, file_type: FileType) -> Result<()> {
        // 1. Detect RAW format variant
        let raw_format = self.detect_raw_format(file_type, reader)?;

        // 2. Process TIFF container (common across CR2, NEF, ARW, ORF, RW2)
        self.tiff_processor.process_tiff_structure(reader)?;

        // 3. Route maker notes to format-specific handler
        if let Some(maker_notes) = reader.get_maker_notes() {
            self.route_maker_notes(raw_format, maker_notes, reader)?;
        }

        // 4. Extract embedded preview images
        self.extract_preview_images(raw_format, reader)?;

        Ok(())
    }
}
```

**Maker Note Routing System**:

```rust
pub trait RawFormatHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()>;
    fn extract_preview_image(&self, reader: &ExifReader) -> Result<Option<Vec<u8>>>;
    fn get_camera_settings(&self, reader: &ExifReader) -> Result<CameraSettings>;
}

// ExifTool's detection patterns
pub fn detect_raw_format(file_type: FileType, reader: &ExifReader) -> Result<RawFormat> {
    let make = reader.get_tag_value("Make").unwrap_or_default();

    match (file_type, make.as_str()) {
        (FileType::CR2, _) | (FileType::CR3, _) => Ok(RawFormat::Canon),
        (FileType::NEF, _) | (FileType::NRW, _) => Ok(RawFormat::Nikon),
        (FileType::ARW, _) | (FileType::SR2, _) => Ok(RawFormat::Sony),
        (FileType::ORF, _) => Ok(RawFormat::Olympus),
        (FileType::RW2, _) | (FileType::RAW, "PANASONIC") => Ok(RawFormat::Panasonic),
        (FileType::RAF, _) => Ok(RawFormat::Fujifilm),
        (FileType::DNG, _) => Ok(RawFormat::DNG),
        _ => Err(ExifError::UnsupportedRawFormat),
    }
}
```

### Phase 2: Manufacturer-Specific Handlers (Week 2-3)

**Priority Order by Complexity** (simplest to most complex):

#### 2.1 Panasonic/Olympus (Simplest - 2.9K + 4.2K lines)

```rust
pub struct PanasonicRawHandler;
impl RawFormatHandler for PanasonicRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Panasonic.pm: 33 ProcessBinaryData entries (simplest)
        // RW2 format: TIFF-based with straightforward maker notes
        process_panasonic_maker_notes(data, reader)
    }
}

pub struct OlympusRawHandler;
impl RawFormatHandler for OlympusRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Olympus.pm: 66 ProcessBinaryData entries (moderate)
        // ORF format: TIFF-based with structured maker notes
        process_olympus_maker_notes(data, reader)
    }
}
```

#### 2.2 Canon Handler (Medium complexity - 10.6K lines)

```rust
pub struct CanonRawHandler {
    lens_database: CanonLensDatabase,
}

impl RawFormatHandler for CanonRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Canon.pm: 169 ProcessBinaryData entries
        // Handle both CR2 (TIFF-based) and CR3 (MOV-based)

        let format = detect_canon_format(reader)?;
        match format {
            CanonFormat::CR2 => self.process_cr2_maker_notes(data, reader),
            CanonFormat::CR3 => self.process_cr3_container(data, reader),
            CanonFormat::CRW => self.process_crw_legacy(data, reader),
        }
    }
}
```

#### 2.3 Sony Handler (High complexity - 11.8K lines)

```rust
pub struct SonyRawHandler {
    // Sony needs sophisticated offset management for multi-generation formats
    offset_manager: AdvancedOffsetManager,
    generation_detector: SonyGenerationDetector,
}

impl RawFormatHandler for SonyRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Sony.pm: 139 ProcessBinaryData entries
        // Multiple generations: ARW, ARQ, SR2, SRF
        // Handle Sony IDC utility corruption issues
        // Uses advanced offset management for complex format variations

        let sony_format = detect_sony_format(reader)?;
        let offset_scheme = self.offset_manager.get_sony_scheme(sony_format)?;
        
        match sony_format {
            SonyFormat::ARW => self.process_arw_maker_notes(data, reader, &offset_scheme),
            SonyFormat::SR2 => self.process_sr2_maker_notes(data, reader, &offset_scheme),
            SonyFormat::SRF => self.process_srf_maker_notes(data, reader, &offset_scheme),
        }
    }
}
```

### Phase 3: Complex Format Support & Sophisticated Offset Management (Week 3-4)

#### 3.0 Advanced Offset Management System

**Context**: While simple manufacturers like Nikon work perfectly with basic offset schemes (as proven in Milestone 14), complex manufacturers (Leica, Panasonic, some Sony) require sophisticated offset management.

**See [OFFSET-BASE-MANAGEMENT.md](../../OFFSET-BASE-MANAGEMENT.md) for comprehensive design details** including:
- ExifTool's offset management architecture research
- Core directory info structure and offset calculation schemes
- Manufacturer-specific patterns (Canon, Nikon, Sony, Leica)
- Implementation recommendations for exif-oxide

**Advanced Offset Management Foundation**:

```rust
/// ExifTool's sophisticated offset management for complex manufacturers
pub struct AdvancedOffsetManager {
    manufacturer_schemes: HashMap<RawFormat, ManufacturerOffsetScheme>,
    entry_based_processors: HashMap<String, EntryBasedOffsetProcessor>,
    corruption_recovery: CorruptionRecoveryEngine,
    validation_engine: OffsetValidationEngine,
}

/// Manufacturer-specific offset complexity (Leica has 9 formats!)
#[derive(Debug, Clone)]
pub struct ManufacturerOffsetScheme {
    pub base_calculation_rules: Vec<BaseCalculationRule>,
    pub format_specific_adjustments: HashMap<String, FormatAdjustment>,
    pub model_specific_overrides: HashMap<String, ModelOverride>,
    pub offset_dependency_chains: Vec<OffsetDependencyChain>,
}

/// Handle entry-based offsets (Panasonic maker notes)
pub struct EntryBasedOffsetProcessor {
    entry_detection_patterns: Vec<EntryPattern>,
    offset_extraction_rules: HashMap<u16, OffsetExtractionRule>,
    validation_constraints: Vec<EntryValidationConstraint>,
}

#[derive(Debug, Clone)]
pub struct OffsetExtractionRule {
    pub tag_id: u16,
    pub offset_location: OffsetLocation,      // Where in the entry the offset is
    pub calculation_method: CalculationMethod, // How to interpret the offset
    pub base_reference: OffsetBaseReference,  // What the offset is relative to
    pub validation_requirements: Vec<ValidationRequirement>,
}

/// ExifTool's complex base reference system
#[derive(Debug, Clone)]
pub enum OffsetBaseReference {
    MakerNoteStart,                           // Simple case (like Nikon)
    TiffHeaderStart,                          // TIFF-relative
    IfdStart(String),                         // Relative to specific IFD
    EntryValueOffset,                         // Relative to entry's value offset
    DynamicCalculation(String),               // Complex expression evaluation
    ManufacturerSpecific(String),             // Manufacturer-defined scheme
}

/// Corruption detection and recovery for offset issues
pub struct CorruptionRecoveryEngine {
    corruption_detectors: Vec<Box<dyn CorruptionDetector>>,
    recovery_strategies: HashMap<CorruptionType, RecoveryStrategy>,
    heuristic_validators: Vec<HeuristicValidator>,
}

pub trait CorruptionDetector {
    fn detect_corruption(&self, offsets: &[CalculatedOffset], data: &[u8]) -> Option<CorruptionAnalysis>;
    fn assess_severity(&self, corruption: &CorruptionAnalysis) -> CorruptionSeverity;
    fn suggest_recovery(&self, corruption: &CorruptionAnalysis) -> Vec<RecoveryOption>;
}

#[derive(Debug)]
pub struct CorruptionAnalysis {
    pub corruption_type: CorruptionType,
    pub affected_offsets: Vec<usize>,
    pub confidence_score: f64,               // 0.0 to 1.0
    pub recovery_feasibility: f64,           // 0.0 to 1.0
    pub alternative_interpretations: Vec<AlternativeInterpretation>,
}

#[derive(Debug)]
pub enum CorruptionType {
    PointerMisalignment,                     // Pointers don't point to valid data
    CircularDependency,                      // Offset chains that loop
    OutOfBoundsReference,                    // Offsets beyond valid data
    InconsistentBaseCalculation,             // Conflicting base calculations
    ManufacturerSpecificViolation(String),   // Format-specific rule violations
}
```

**Complex Manufacturer Implementations**:

```rust
/// Leica's 9 maker note formats with complex expressions
pub struct LeicaOffsetProcessor {
    format_detection: LeicaFormatDetector,
    expression_evaluator: OffsetExpressionEvaluator,
    format_specific_handlers: HashMap<LeicaFormat, Box<dyn LeicaFormatHandler>>,
}

/// Handle complex offset expressions like "(entry_count * 12 + base_offset)"
pub struct OffsetExpressionEvaluator {
    expression_parser: ExpressionParser,
    variable_resolver: VariableResolver,
    calculation_cache: HashMap<String, CalculatedValue>,
}

/// Panasonic entry-based offset processing
pub struct PanasonicEntryProcessor {
    entry_based_manager: EntryBasedOffsetProcessor,
    panasonic_specific_rules: Vec<PanasonicOffsetRule>,
    rw2_format_handler: RW2FormatHandler,
}

/// Sony multi-generation offset handling
pub struct SonyOffsetProcessor {
    generation_detector: SonyGenerationDetector,
    arw_processor: ARWOffsetProcessor,
    sr2_processor: SR2OffsetProcessor,
    format_migration_handler: FormatMigrationHandler,
}
```

### Phase 3: Complex Format Support (Week 3-4)

#### 3.1 Nikon Handler (Leverages Milestone 14)

```rust
pub struct NikonRawHandler {
    // Simple delegation to proven Milestone 14 implementation
    nikon_processor: NikonProcessor,
}

impl RawFormatHandler for NikonRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // Leverage complete Nikon implementation from Milestone 14
        // No complex offset management needed - simple schemes work perfectly
        self.nikon_processor.process_nikon_makernotes(reader, offset)
    }
}
```

#### 3.2 Non-TIFF Formats

```rust
pub struct FujifilmRawHandler;
impl RawFormatHandler for FujifilmRawHandler {
    fn process_maker_notes(&self, data: &[u8], reader: &mut ExifReader) -> Result<()> {
        // FujiFilm.pm: 19 ProcessBinaryData entries (simple)
        // RAF format: Custom binary format (not TIFF-based)
        process_raf_custom_format(data, reader)
    }
}
```

### Phase 4: Integration and Testing (Week 4-5)

**Preview Image Extraction**:

```rust
impl RawProcessor {
    pub fn extract_preview_images(&self, format: RawFormat, reader: &ExifReader) -> Result<Vec<PreviewImage>> {
        // Extract embedded JPEG previews from RAW files
        // Handle manufacturer-specific preview locations
        // Support multiple preview sizes (thumbnail, medium, large)

        match format {
            RawFormat::Canon => extract_canon_previews(reader),
            RawFormat::Nikon => extract_nikon_previews(reader),
            RawFormat::Sony => extract_sony_previews(reader),
            // ... other manufacturers
        }
    }
}
```

## MIMETYPES.md Format Coverage

**Supported RAW Formats**:

- **Canon**: CR2 (`image/x-canon-cr2`), CR3 (`image/x-canon-cr3`), CRW (`image/x-canon-crw`)
- **Nikon**: NEF (`image/x-nikon-nef`), NRW (`image/x-nikon-nrw`)
- **Sony**: ARW (`image/x-sony-arw`), ARQ (`image/x-sony-arq`), SR2 (`image/x-sony-sr2`), SRF (`image/x-sony-srf`)
- **Fujifilm**: RAF (`image/x-fujifilm-raf`)
- **Olympus**: ORF (`image/x-olympus-orf`)
- **Panasonic**: RAW (`image/x-panasonic-raw`), RW2 (`image/x-panasonic-rw2`)
- **Adobe**: DNG (`image/x-adobe-dng`)
- **Other**: ERF, GPR, 3FR, DCR, K25, KDC, RWL, MEF, MRW, PEF, IIQ, SRW, X3F

## Success Criteria

### Core Requirements

- [ ] **TIFF Foundation**: Robust TIFF container parsing for all TIFF-based RAW formats
- [ ] **Maker Note Routing**: Automatic detection and routing to format-specific handlers
- [ ] **Advanced Offset Management**: Sophisticated offset handling for complex manufacturers (Leica, Panasonic, Sony)
- [ ] **Simple Offset Integration**: Seamless integration with simple manufacturers (Nikon via Milestone 14)
- [ ] **Corruption Recovery**: Graceful handling of offset corruption with heuristic recovery
- [ ] **Basic Metadata**: Camera make/model, exposure settings, lens information
- [ ] **Preview Extraction**: Embedded JPEG preview images for all formats
- [ ] **Error Handling**: Graceful handling of corrupted or unsupported RAW variants

### Validation Tests

- Process sample files from each manufacturer in `t/images/`
- Extract core metadata (ISO, shutter speed, aperture, focal length)
- Compare output with ExifTool for equivalency
- Handle edge cases (older camera models, firmware variations)

## Implementation Boundaries

### Goals (Milestone 17)

- Basic RAW metadata extraction for all MIMETYPES.md formats
- Embedded preview image extraction
- Camera settings and technical metadata
- Robust error handling for unsupported variants

### Non-Goals (Future Milestones)

- **Advanced manufacturer features**: Complex encryption (beyond basic Nikon), proprietary lens corrections
- **RAW image decoding**: Only metadata, not actual image data processing
- **Write support**: Read-only RAW processing
- **Sidecar XMP**: Focus on embedded metadata only

## Dependencies and Prerequisites

### Milestone Prerequisites

- **Milestone 14**: Nikon implementation (completed) - can leverage for NEF format
- **Milestone 16**: File type detection - RAW format identification

### Enables Future Milestones

- **Advanced manufacturer features**: Extended lens databases, advanced correction data
- **Write support**: Metadata writing to RAW files (complex)
- **Professional workflows**: RAW processing pipeline integration

## Risk Mitigation

### Format Complexity Risk

- **Risk**: Manufacturer-specific formats too complex for unified approach
- **Mitigation**: Phase-based implementation, simplest formats first
- **Evidence**: DNG format proves multiple manufacturers can share infrastructure

### Encryption Complexity (Nikon)

- **Risk**: Nikon encryption requires substantial additional complexity
- **Mitigation**: Leverage existing Milestone 14 Nikon implementation
- **Boundary**: Focus on basic encrypted data detection, not full decryption

### Preview Extraction Complexity

- **Risk**: Each manufacturer stores previews differently
- **Mitigation**: Implement basic preview extraction, enhanced in future milestones

## Related Documentation

### Required Reading

- **Existing Nikon Implementation**: Milestone 14 patterns for complex manufacturer handling
- **TIFF Processing**: Existing TIFF infrastructure in exif-oxide
- **MIMETYPES.md**: Complete list of formats to support
- **[OFFSET-BASE-MANAGEMENT.md](../../OFFSET-BASE-MANAGEMENT.md)**: Critical offset calculation patterns for complex manufacturers

### Implementation References

- **Canon.pm**: 169 ProcessBinaryData patterns for CR2/CR3 handling
- **Sony.pm**: Multiple generation handling for ARW/SR2/SRF
- **DNG.pm**: Multi-manufacturer container approach

This milestone establishes comprehensive RAW format support through a unified foundation while respecting manufacturer-specific requirements. The phased approach ensures rapid delivery of basic functionality while building toward complete RAW ecosystem support.

**Note**: For detailed offset management implementation guidance, see [OFFSET-BASE-MANAGEMENT.md](../../OFFSET-BASE-MANAGEMENT.md).
