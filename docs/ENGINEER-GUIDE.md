# Engineer's Guide to exif-oxide

**🚨 CRITICAL: Start with [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - the keystone principle that governs ALL development.**

This guide helps new engineers understand the exif-oxide project and start contributing effectively. Read this after understanding the high-level [ARCHITECTURE.md](ARCHITECTURE.md).

## Essential Background

### What is ExifTool?

ExifTool is a 25-year-old Perl library that reads/writes metadata from image, audio, and video files. It's the de facto standard because it handles thousands of proprietary formats and manufacturer quirks that have accumulated over decades of digital photography.

### Why Translation, Not Innovation?

Every line of ExifTool code exists for a reason - usually to work around a specific camera's bug or non-standard behavior. We must resist the temptation to "improve" or "simplify" the logic. If ExifTool checks for value 0x41 before 0x42, there's a camera somewhere that depends on that order.

## Learning Path

Start with these guides in order:

1. **[TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)** - This is the keystone guide for this project
1. **[EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md)** - Complete guide to working with ExifTool source
1. **[CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md)** - Core system architecture and offset management
1. **[DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md)** - Development workflow and best practices
1. **[PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md)** - Processor dispatch strategy
1. **[TROUBLESHOOTING.md](reference/TROUBLESHOOTING.md)** - Common issues and solutions

## Quick Reference

### Key Principles

- **Trust ExifTool** - We translate, not innovate
- **Test on Real Images** - The spec lies, cameras don't follow it
- **Document Everything** - Include ExifTool source references
- **Embrace the Chaos** - Metadata is messy because cameras are messy

### Common Tasks

- **Adding PrintConv/ValueConv** - See [EXIFTOOL-INTEGRATION.md](design/EXIFTOOL-INTEGRATION.md)
- **Understanding Architecture** - See [CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md)
- **Processor Dispatch** - See [PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md)
- **Development Workflow** - See [DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md)

## Generated Code Policy

**Generated Rust code is committed to git** while intermediate files are ignored:

- **Commit**: Final Rust code in `src/generated/` (tags.rs, conversion_refs.rs, etc.)
- **Ignore**: Intermediate files in `codegen/generated/` (tag_tables.json, etc.)

**Rationale**: This ensures developers can build without requiring Perl + ExifTool while keeping the repository manageable. Generated code is relatively stable and benefits from code review visibility.

**When to regenerate**:

- After modifying extraction scripts (`codegen/extractors/*.pl`)
- After updating ExifTool version
- After adding/modifying configuration files in `codegen/config/`
- When adding new simple table extractions
- Run: `make codegen` then commit the updated `src/generated/` files

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check module docs** - See `third-party/exiftool/doc/modules/` for specific formats
3. **Review concepts** - `third-party/exiftool/doc/concepts/` explains patterns
4. **Check ExifTool forums** - Many quirks are discussed there
5. **Use --show-missing** - Let it guide your implementation priority
6. **Start small** - One tag at a time, one format at a time

Key documentation files:

- [ExifTool documentation](../third-party/exiftool/doc/concepts/) - ExifTool concepts and patterns
- [PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md) - Our dispatch strategy
- [ARCHITECTURE.md](ARCHITECTURE.md) - High-level system overview
- [API-DESIGN.md](design/API-DESIGN.md) - Public API structure and TagEntry design

## Code Maintenance Practices

### Watch for Codegen Opportunities

When reviewing or writing code, be vigilant for manually-maintained lookup tables that should be generated:

- **Red flag**: Any match statement or HashMap with >5 static entries mapping to strings
- **Red flag**: Hardcoded camera/lens names, white balance modes, or other manufacturer settings
- **Action**: Check if it came from ExifTool source (usually `%hashName = (...)`)
- **Solution**: Use the simple table extraction framework (see [EXIFTOOL-INTEGRATION.md](design/EXIFTOOL-INTEGRATION.md))

Remember: Every manually-ported lookup table becomes a maintenance burden with monthly ExifTool updates.

### File Size Guidelines

Keep source files under 500 lines for better maintainability:

- Files >500 lines should be refactored into focused modules
- The Read tool truncates at 2000 lines, hindering code analysis
- Smaller files improve code organization and tool effectiveness

## Remember

- ExifTool compatibility is the #1 priority
- Don't innovate, translate
- Every quirk has a reason
- Test against real images
- Document ExifTool source references

Happy coding! Remember: if it seems weird, it's probably correct. Cameras are weird.
