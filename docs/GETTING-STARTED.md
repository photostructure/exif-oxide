# Getting Started with exif-oxide

**15-minute onboarding path for new engineers.**

Welcome to exif-oxide! This guide gets you productive quickly by focusing on the essential knowledge first.

## Quick Start (15 minutes)

### 1. Project Foundation (5 minutes)

**CRITICAL:** Read [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) first. This is the #1 rule governing ALL development on this project. Everything else builds on this principle.

**Key takeaway:** We translate ExifTool exactly - we don't "improve" or "simplify" its parsing logic.

**What is ExifTool?** ExifTool is a 25-year-old Perl library that reads/writes metadata from image, audio, and video files. It's the de facto standard because it handles thousands of proprietary formats and manufacturer quirks that have accumulated over decades of digital photography. Every line of ExifTool code exists for a reason - usually to work around a specific camera's bug or non-standard behavior.

Then skim [ARCHITECTURE.md](ARCHITECTURE.md) for the system overview. Focus on:

- The extract → generate → implement cycle
- Manual excellence over code generation for complex logic
- Runtime fallback system (never panics)

### 2. Understanding ExifTool (5 minutes)

Read [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) sections 1-3:

- **Section 1:** Core concepts (PROCESS_PROC, tag tables, conversion pipeline)
- **Section 2:** How to read ExifTool source code
- **Section 3:** Common pitfalls to avoid

**Key takeaway:** ExifTool's complexity exists for good reasons - every quirk handles a specific camera's bug.

### 3. Development Setup (5 minutes)

Follow the TPP-driven development workflow:

- **Create TPPs:** Write Technical Project Plans for coherent feature sets using [TPP.md](TPP.md)
- **Use `--show-missing`:** Let it guide your implementation priorities

Then run your first test:

```bash
# Build the project
make precommit

# Test on a real image
cargo run -- test-images/Canon/Canon_T3i.jpg --show-missing

# See what ExifTool extracts for comparison
exiftool -j test-images/Canon/Canon_T3i.jpg
```

## Choose Your Learning Path

After the 15-minute foundation, pick your path based on your task:

### 🔧 **I'm implementing a specific tag/conversion**

→ Write a TPP using [TPP.md](TPP.md) template to plan your implementation
→ Reference [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) sections 4-5 as needed

### 🏗️ **I'm working on core architecture**

→ Read [guides/CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md) (state management & offset calculations)
→ Study [CODEGEN.md](CODEGEN.md) (code generation system)

### 📷 **I'm adding manufacturer support**

→ Check [reference/MANUFACTURER-FACTS.md](reference/MANUFACTURER-FACTS.md) for existing patterns
→ Study [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) section 5 (manufacturer quirks)

### 🔍 **I'm debugging or troubleshooting**

→ Jump to [reference/TROUBLESHOOTING.md](reference/TROUBLESHOOTING.md)
→ Reference [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) section 3 (common pitfalls)

### 🧪 **I'm working on tests**

→ Follow the test-driven debugging workflow in [TDD.md](TDD.md)
→ Check test image collection in `test-images/` vs `third-party/exiftool/t/images/`

## Essential Principles

Keep these in mind throughout development:

1. **Trust ExifTool** - The fundamental law of this project
2. **Test on real images** - Use `test-images/` for full camera files
3. **Document everything** - Include ExifTool source references (file:line)
4. **Ask clarifying questions** - The user expects this for complex tasks
5. **Use runtime fallback** - Missing implementations return raw values, never panic
6. **Watch for codegen opportunities** - Manually-maintained lookup tables should use the simple table extraction framework
7. **Keep files focused** - Files >500 lines should be refactored into smaller, focused modules

## Quick Reference Links

### Core Documentation

- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - The prime directive
- [ARCHITECTURE.md](ARCHITECTURE.md) - System overview
- [MILESTONES.md](MILESTONES.md) - Current development priorities

### Design Documents

- [design/API-DESIGN.md](design/API-DESIGN.md) - Public API structure
- [CODEGEN.md](CODEGEN.md) - Code generation system
- [guides/PRINTCONV-VALUECONV-GUIDE.md](guides/PRINTCONV-VALUECONV-GUIDE.md) - PrintConv/ValueConv implementation guide and design decisions

### Development Guides

- [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) - Complete ExifTool reference
- [TPP.md](TPP.md) - Technical Project Plan template for feature development
- [guides/CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md) - State management & offset calculations

### Reference Materials

- [reference/SUPPORTED-FORMATS.md](reference/SUPPORTED-FORMATS.md) - File formats and MIME types
- [reference/MANUFACTURER-FACTS.md](reference/MANUFACTURER-FACTS.md) - Manufacturer-specific quirks
- [reference/TROUBLESHOOTING.md](reference/TROUBLESHOOTING.md) - Debugging guide

## Generated Code Policy

**Generated Rust code is committed to git** while intermediate files are ignored:

- **Commit**: Final Rust code in `src/generated/` (tags.rs, conversion_refs.rs, etc.)
- **Ignore**: Intermediate files in `codegen/generated/` (tag_tables.json, etc.)

**When to regenerate**: After modifying extraction scripts, updating ExifTool, or adding new configurations. Run `make codegen` then commit the updated `src/generated/` files.

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check existing documentation** - Use the learning paths above
3. **Ask clarifying questions** - Especially for vague or complex requirements
4. **Use `--show-missing`** - Let it guide your implementation priorities
5. **Test against ExifTool** - Our compatibility tests compare outputs

## Next Steps

Once you've completed the 15-minute foundation:

1. **Pick your learning path** based on your current task
2. **Try the extract-generate-implement cycle** on a simple tag
3. **Read ExifTool source code** for your specific area
4. **Ask the user** for clarification on any confusing aspects

Remember: If something seems weird or wrong, it's probably correct for some camera model. When in doubt, **trust ExifTool**.

Happy coding! 📷
