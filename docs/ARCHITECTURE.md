# exif-oxide Architecture

**🚨 CRITICAL: This architecture is built on [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - the fundamental law of this project.**

## Overview

exif-oxide is a Rust translation of [ExifTool](https://exiftool.org/), focusing on mainstream metadata extraction from image files. This document provides a high-level overview of the architecture and design decisions.

**Scope**: This project targets mainstream metadata tags only (frequency > 80% in TagMetadata.json), reducing the implementation burden from 15,000+ tags to approximately 500-1000. We explicitly do not support ExifTool's custom tag definitions or user-defined tags.

## Core Philosophy

1. **No Novel Parsing**: ExifTool has already solved every edge case - we port, not reinvent. Follow [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)
2. **Manual Excellence**: Complex logic is manually ported with references to ExifTool source
3. **Simple Codegen**: Generator handles only straightforward, unambiguous translations
4. **Always Working**: System produces compilable code at every stage
5. **Transparent Progress**: Clear visibility into what's implemented vs TODO
6. **Mainstream Focus**: Only implement tags with >80% frequency or marked mainstream
7. **Streaming First**: All binary data handled via streaming to minimize memory usage

## Key Insights from ExifTool Analysis

### PROCESS_PROC Complexity

- 121+ uses of ProcessBinaryData across manufacturers
- Custom processors for encrypted data (Nikon), serial data (Canon), text formats (JVC)
- Sophisticated dispatch with table-level and SubDirectory overrides
- No code sharing between processors - each is self-contained

### ProcessBinaryData Sophistication

- Variable-length formats with offset tracking (`var_string`, `var_int16u`)
- Hook mechanism for dynamic format assignment
- Bit-level extraction with Mask/BitShift
- Complex format expressions like `string[$val{3}]`

### Error Handling Excellence

- MINOR_ERRORS classification system
- Graceful degradation with corruption
- Manufacturer-specific quirk handling
- Size limits and validation boundaries

### Non-UTF-8 Data Handling

- Binary magic numbers with raw bytes (e.g., BPG format: `BPG\xfb`)
- Pattern matching on binary data streams
- Proper escaping for Rust string literals
- JSON-safe representation of non-UTF-8 content

### State Management Requirements

- PROCESSED hash for recursion prevention
- DataMember dependencies between tags
- VALUE hash for extracted data
- Directory context (Base, DataPos, PATH)

## System Architecture

### Build Pipeline

The simplified codegen architecture uses Rust orchestration with minimal Perl scripts:

```
1. Rust scans codegen/config/ directories
2. Reads source paths from config files
3. Patches ExifTool modules (temporary)
4. Calls simple Perl scripts with explicit arguments
5. Perl outputs individual JSON files directly
6. Rust reads JSON and generates code
   - Handles non-UTF-8 bytes in patterns
   - Escapes binary data for Rust string literals
7. Reverts ExifTool patches

ExifTool Source → [Rust Orchestration] → Perl Extractors → JSON → Generated Code
                     ↓                        ↓                         ↓
              Config-driven            Explicit arguments        Direct output
                                      No config reading         No split step
```

### Key Components

1. **ExifTool Integration** ([CODEGEN.md](CODEGEN.md))

   - Code generation for tag tables and lookups
   - Manual implementation patterns
   - PrintConv/ValueConv functions
   - Manufacturer-specific processors

2. **Public API** ([design/API-DESIGN.md](design/API-DESIGN.md))

   - Streaming-first design
   - TagEntry with value/print fields
   - ExifTool compatibility

3. **Core Architecture** ([guides/CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md))

   - Stateful ExifReader object
   - DataMember dependencies
   - Offset management and state tracking

4. **Processor Dispatch** ([guides/PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md))

   - Conditional processor selection
   - Runtime evaluation
   - Manufacturer-specific routing

## Development Workflow

1. **Extract**: Parse ExifTool source into JSON
2. **Generate**: Create Rust code from definitions
3. **Discover**: Use `--show-missing` to find needed implementations
4. **Implement**: Port complex logic manually
5. **Validate**: Test against ExifTool output

See [guides/DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md) for detailed steps.

## Implementation Status

### Completed

- Basic JPEG/EXIF parsing
- PrintConv infrastructure
- Canon MakerNote support
- Sony MakerNote basics
- Composite tag framework
- ProcessBinaryData for simple formats

### In Progress

- See [MILESTONES.md](MILESTONES.md) for current work

### Future

- Additional manufacturers (Nikon, Panasonic)
- Video metadata (MP4, QuickTime)
- Write support
- Advanced encryption

## Key Design Decisions

### Manual Implementation Over Code Generation

We manually implement complex logic because:

- Perl expressions are too complex to parse reliably
- Manual code can reference ExifTool source directly
- Easier to debug and understand
- Better performance characteristics

### Runtime Fallback Over Compile-Time Stubs

Missing implementations return raw values instead of panicking:

- System remains usable during development
- Clear visibility into what's missing
- No stub function explosion

### Streaming Over In-Memory

Binary data uses streaming references:

- Handles large embedded images/videos
- Minimal memory footprint
- Efficient for partial extraction

## Benefits of This Architecture

1. **Realistic**: No Perl parsing, just manual porting of what matters
2. **Incremental**: Ship working code immediately, improve coverage over time
3. **Maintainable**: Clear separation between generated and manual code
4. **Traceable**: Every manual implementation references ExifTool source
5. **Robust**: Handles ExifTool's full complexity through manual excellence
6. **No Panic**: System never crashes on missing implementations
7. **Demand-Driven**: Only implement what's actually used in real files
8. **Zero Stub Spam**: No thousands of TODO functions cluttering the codebase

## Documentation Guide

### Design Documents

- [API Design](design/API-DESIGN.md) - Public API structure
- [ExifTool Integration](CODEGEN.md) - Unified code generation and implementation guide

### Guides

- [Engineer's Guide](ENGINEER-GUIDE.md) - Getting started with the codebase
- [Development Guide](guides/DEVELOPMENT-GUIDE.md) - Development workflow and best practices
- [ExifTool Guide](guides/EXIFTOOL-GUIDE.md) - Complete guide to working with ExifTool source

### Milestones

- [Current Milestones](MILESTONES.md) - Active development work
- [Completed Milestones](archive/DONE-MILESTONES.md) - Historical progress

### Technical Deep Dives

- [Core Architecture](guides/CORE-ARCHITECTURE.md) - Core system architecture and offset management
- [Processor Dispatch](guides/PROCESSOR-DISPATCH.md) - Processor dispatch strategy

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check module docs** - See `third-party/exiftool/doc/`
3. **Use --show-missing** - Let it guide your implementation priority
4. **Ask clarifying questions** - The codebase is complex by necessity

## Conclusion

This architecture embraces the reality of ExifTool's complexity. Rather than trying to automatically handle Perl's flexibility, we:

1. Use codegen only for unambiguous translations
2. Manually port complex logic with full traceability
3. Build an implementation palette that grows over time
4. Maintain ExifTool compatibility through careful porting

This approach is more labor-intensive but results in:

- Correct handling of manufacturer quirks
- Predictable performance
- Maintainable code
- Clear progress tracking

The key insight: ExifTool's value isn't in its Perl code, but in the accumulated knowledge of metadata formats. We preserve this knowledge through careful manual translation, not automatic parsing.
