# Milestone 17: RAW Image Format Support - Overview

**Total Duration**: 11-15 weeks (divided into 7 sub-milestones)  
**Goal**: Implement comprehensive RAW metadata extraction for all mainstream manufacturers

## Overview

This milestone implements RAW format support through a series of incremental sub-milestones, starting with the simplest formats and building up to the most complex. Each sub-milestone delivers working functionality while building toward comprehensive RAW metadata support.

**Important Scope Clarification**: This milestone focuses exclusively on **metadata extraction**. All binary data extraction (preview images, thumbnails, embedded JPEGs) is handled by [Milestone 19: Binary Data Extraction](MILESTONE-19-Binary-Data-Extraction.md).

## Sub-Milestones

### [17a: RAW Foundation & Kyocera](MILESTONE-17a-RAW-Foundation-Kyocera.md) (1-2 weeks)

- Core RAW detection and routing infrastructure
- Implement simplest format: KyoceraRaw (173 lines)
- Basic TIFF container support for RAW files
- CLI integration foundation

### [17b: Simple TIFF-Based RAW](MILESTONE-17b-Simple-TIFF-RAW.md) (2 weeks)

- MinoltaRaw/MRW support (537 lines)
- PanasonicRaw/RW2 support (956 lines)
- Entry-based offset handling (Panasonic)
- Shared TIFF infrastructure improvements

### [17c: Olympus RAW Support](MILESTONE-17c-Olympus-RAW.md) (2 weeks)

- Olympus ORF format (4,235 lines)
- Handle 15 ProcessBinaryData sections
- Multiple IFD navigation
- Olympus-specific tag processing

### [17d: Canon RAW Support](MILESTONE-17d-Canon-RAW.md) (2-3 weeks)

- Canon CR2 (TIFF-based) - required
- Canon CRW (legacy) - optional
- Canon CR3 (MOV container) - optional
- Canon maker notes (169 ProcessBinaryData entries)

### [17e: Sony RAW Support](MILESTONE-17e-Sony-RAW.md) (2-3 weeks)

- Sony ARW/SR2/SRF formats (11,818 lines)
- Advanced offset management system
- Multi-generation format handling
- Sony IDC corruption recovery

### [17f: Nikon RAW Integration](MILESTONE-17f-Nikon-RAW-Integration.md) (1 week)

- Integrate existing Nikon work from Milestone 14
- NEF/NRW format support
- Ensure consistency with new infrastructure

### [17g: Additional Formats & Testing](MILESTONE-17g-Additional-RAW-Testing.md) (1-2 weeks)

- Fujifilm RAF (non-TIFF format)
- Adobe DNG (multi-manufacturer)
- Comprehensive testing suite
- Performance optimization

## Background Analysis

**ExifTool Complexity Analysis**:

- **Simplest**: KyoceraRaw (173 lines) - just ProcessBinaryData
- **Simple**: MinoltaRaw (537 lines) - clean TIFF structure
- **Medium**: PanasonicRaw (956 lines), Olympus (4,235 lines)
- **Complex**: Canon (10,648 lines), Sony (11,818 lines)
- **Most Complex**: Nikon (14,199 lines)

**Technical Patterns**:

- **TIFF-based**: Most formats (NEF, ORF, ARW, RW2, MRW, CR2)
- **Custom formats**: RAF (Fujifilm), CRW (Canon legacy)
- **Container-based**: CR3 (MOV), DNG (multi-manufacturer)

## Core Success Criteria (All Sub-Milestones)

Each sub-milestone must meet these criteria:

1. **CLI Integration**: The CLI can successfully read and extract metadata from the new format(s)
2. **ExifTool Compatibility**: Output matches `exiftool -j` for all supported tags
3. **Test Coverage**: Integration tests validate against sample files
4. **No Binary Extraction**: Metadata only - preview/thumbnail extraction is Milestone 19's responsibility

## Shared Infrastructure

The sub-milestones will build these shared components:

- **RAW Format Detection**: Robust identification of RAW file types
- **Manufacturer Routing**: Clean dispatch to format-specific handlers
- **TIFF Container Support**: Enhanced TIFF processing for RAW variants
- **Offset Management**: Simple to advanced offset handling as needed
- **Test Framework**: Comprehensive RAW format testing infrastructure

## Implementation Approach

**Critical Principle**: All RAW format implementations must strictly follow the [Trust ExifTool](../TRUST-EXIFTOOL.md) principle. We translate ExifTool's logic exactly, preserving all quirks, special cases, and seemingly "odd" code that handles real-world camera behavior.

1. **Start Simple**: Begin with 173-line KyoceraRaw to validate architecture
2. **Build Incrementally**: Each format adds complexity and capability
3. **Reuse Infrastructure**: Shared TIFF and offset management code
4. **Test Continuously**: Every milestone includes compatibility tests
5. **Document Patterns**: Extract common patterns for future formats
6. **Trust ExifTool**: Study and follow ExifTool's implementation precisely - no "improvements" or "optimizations"

## Overall Success Criteria

### Core Requirements (Across All Sub-Milestones)

- [ ] **TIFF Foundation**: Robust TIFF container parsing for all TIFF-based RAW formats
- [ ] **Maker Note Routing**: Automatic detection and routing to format-specific handlers
- [ ] **Advanced Offset Management**: Sophisticated offset handling for complex manufacturers (Sony, Panasonic)
- [ ] **Simple Offset Integration**: Seamless integration with simple manufacturers (Kyocera, Minolta)
- [ ] **Nikon Integration**: Leverage existing Milestone 14 implementation
- [ ] **Metadata Extraction**: Camera make/model, exposure settings, lens information
- [ ] **Error Handling**: Graceful handling of corrupted or unsupported RAW variants

### Validation Tests

- Process sample files from each manufacturer in `test-images/`
- Extract core metadata (ISO, shutter speed, aperture, focal length)
- Compare output with ExifTool for equivalency
- Handle edge cases (older camera models, firmware variations)

## Implementation Boundaries

### Goals (Milestone 17 - All Sub-Milestones)

- RAW metadata extraction for all mainstream formats
- Camera settings and technical metadata
- Manufacturer-specific tags and maker notes
- Robust error handling for unsupported variants

### Non-Goals (Handled by Other Milestones)

- **Binary Data Extraction**: Preview/thumbnail extraction is Milestone 19's responsibility
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

### Offset Management Complexity

- **Risk**: Complex manufacturers require sophisticated offset handling
- **Mitigation**: Start with simple formats, build advanced offset system incrementally
- **Reference**: See [OFFSET-BASE-MANAGEMENT.md](../OFFSET-BASE-MANAGEMENT.md) for detailed patterns

## Related Documentation

### Required Reading

- **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)**: Critical - we must follow ExifTool's implementation exactly
- **[OFFSET-BASE-MANAGEMENT.md](../OFFSET-BASE-MANAGEMENT.md)**: Critical offset calculation patterns for complex manufacturers
- **Existing Nikon Implementation**: Milestone 14 patterns for complex manufacturer handling
- **TIFF Processing**: Existing TIFF infrastructure in exif-oxide
- **MIMETYPES.md**: Complete list of formats to support

### Implementation References

- **Canon.pm**: 169 ProcessBinaryData patterns for CR2/CR3 handling
- **Sony.pm**: Multiple generation handling for ARW/SR2/SRF
- **DNG.pm**: Multi-manufacturer container approach
- **KyoceraRaw.pm**: Simplest format (173 lines) - good starting point

## Summary

This milestone establishes comprehensive RAW format support through incremental sub-milestones, starting with the simplest formats and building toward complex manufacturer implementations. By dividing the work into manageable chunks, we can deliver value quickly while building robust infrastructure for all RAW formats.
