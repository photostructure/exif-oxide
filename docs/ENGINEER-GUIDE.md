# Engineer's Guide to exif-oxide

This guide helps new engineers understand the exif-oxide project and start contributing effectively. Read this after understanding the high-level [ARCHITECTURE.md](ARCHITECTURE.md).

## Essential Background

### What is ExifTool?

ExifTool is a 25-year-old Perl library that reads/writes metadata from image, audio, and video files. It's the de facto standard because it handles thousands of proprietary formats and manufacturer quirks that have accumulated over decades of digital photography.

### Why Translation, Not Innovation?

Every line of ExifTool code exists for a reason - usually to work around a specific camera's bug or non-standard behavior. We must resist the temptation to "improve" or "simplify" the logic. If ExifTool checks for value 0x41 before 0x42, there's a camera somewhere that depends on that order.

## Learning Path

Start with these guides in order:

1. **[TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)** - This is the keystone guide for this project
1. **[TESTING.md](guides/TESTING.md)** - No test mocking for this project!
1. **[EXIFTOOL-CONCEPTS.md](guides/EXIFTOOL-CONCEPTS.md)** - Critical concepts like PROCESS_PROC, tag tables, and the conversion pipeline
1. **[READING-EXIFTOOL-SOURCE.md](guides/READING-EXIFTOOL-SOURCE.md)** - How to navigate ExifTool's Perl source code
1. **[DEVELOPMENT-WORKFLOW.md](guides/DEVELOPMENT-WORKFLOW.md)** - The extract-generate-implement cycle
1. **[COMMON-PITFALLS.md](guides/COMMON-PITFALLS.md)** - Mistakes to avoid and debugging tips
1. **[TRIBAL-KNOWLEDGE.md](guides/TRIBAL-KNOWLEDGE.md)** - Undocumented quirks and mysteries

## Quick Reference

### Key Principles

- **Trust ExifTool** - We translate, not innovate
- **Test on Real Images** - The spec lies, cameras don't follow it
- **Document Everything** - Include ExifTool source references
- **Embrace the Chaos** - Metadata is messy because cameras are messy

### Common Tasks

- **Adding PrintConv/ValueConv** - See [IMPLEMENTATION-PALETTE.md](design/IMPLEMENTATION-PALETTE.md)
- **Understanding State** - See [STATE-MANAGEMENT.md](STATE-MANAGEMENT.md)
- **Processor Dispatch** - See [PROCESSOR-PROC-DISPATCH.md](PROCESSOR-PROC-DISPATCH.md)
- **Offset Calculations** - See [OFFSET-BASE-MANAGEMENT.md](OFFSET-BASE-MANAGEMENT.md)

## Generated Code Policy

**Generated Rust code is committed to git** while intermediate files are ignored:

- **Commit**: Final Rust code in `src/generated/` (tags.rs, conversion_refs.rs, etc.)
- **Ignore**: Intermediate files in `codegen/generated/` (tag_tables.json, etc.)

**Rationale**: This ensures developers can build without requiring Perl + ExifTool while keeping the repository manageable. Generated code is relatively stable and benefits from code review visibility.

**When to regenerate**:

- After modifying extraction scripts (`codegen/extract_tables.pl`)
- After updating ExifTool version
- When adding new tag implementations to MILESTONE_COMPLETIONS
- Run: `make codegen` then commit the updated `src/generated/` files

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check module docs** - See `third-party/exiftool/doc/modules/` for specific formats
3. **Review concepts** - `third-party/exiftool/doc/concepts/` explains patterns
4. **Check ExifTool forums** - Many quirks are discussed there
5. **Use --show-missing** - Let it guide your implementation priority
6. **Start small** - One tag at a time, one format at a time

Key documentation files:

- [FILE_TYPES.md](../third-party/exiftool/doc/concepts/FILE_TYPES.md) - File format detection
- [WRITE_PROC.md](../third-party/exiftool/doc/concepts/WRITE_PROC.md) - Writing (future milestone)
- [PROCESSOR-PROC-DISPATCH.md](PROCESSOR-PROC-DISPATCH.md) - Our dispatch strategy

## Remember

- ExifTool compatibility is the #1 priority
- Don't innovate, translate
- Every quirk has a reason
- Test against real images
- Document ExifTool source references

Happy coding! Remember: if it seems weird, it's probably correct. Cameras are weird.
