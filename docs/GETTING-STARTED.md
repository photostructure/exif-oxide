# Getting Started with exif-oxide

**15-minute onboarding path for new engineers.**

Welcome to exif-oxide! This guide gets you productive quickly by focusing on the essential knowledge first.

## Quick Start (15 minutes)

### 1. Project Foundation (5 minutes)

**CRITICAL:** Read [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) first. This is the #1 rule governing ALL development on this project. Everything else builds on this principle.

**Key takeaway:** We translate ExifTool exactly - we don't "improve" or "simplify" its parsing logic.

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

Read [guides/DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md) sections 1-2:

- **Section 1:** The extract-generate-implement cycle
- **Section 2:** Using `--show-missing` to find what needs implementation

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

→ Continue with [guides/DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md) section 4 (implementation workflow)
→ Reference [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) sections 4-5 as needed

### 🏗️ **I'm working on core architecture**

→ Read [guides/CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md) (state management & offset calculations)
→ Study [design/EXIFTOOL-INTEGRATION.md](design/EXIFTOOL-INTEGRATION.md) (code generation system)

### 📷 **I'm adding manufacturer support**

→ Check [reference/MANUFACTURER-FACTS.md](reference/MANUFACTURER-FACTS.md) for existing patterns
→ Study [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) section 5 (manufacturer quirks)

### 🔍 **I'm debugging or troubleshooting**

→ Jump to [reference/TROUBLESHOOTING.md](reference/TROUBLESHOOTING.md)
→ Reference [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) section 3 (common pitfalls)

### 🧪 **I'm working on tests**

→ Read [guides/DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md) section 2 (testing strategy)
→ Check test image collection in `test-images/` vs `third-party/exiftool/t/images/`

## Essential Principles

Keep these in mind throughout development:

1. **Trust ExifTool** - The fundamental law of this project
2. **Test on real images** - Use `test-images/` for full camera files
3. **Document everything** - Include ExifTool source references (file:line)
4. **Ask clarifying questions** - The user expects this for complex tasks
5. **Use runtime fallback** - Missing implementations return raw values, never panic

## Quick Reference Links

### Core Documentation

- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - The prime directive
- [ARCHITECTURE.md](ARCHITECTURE.md) - System overview
- [MILESTONES.md](MILESTONES.md) - Current development priorities

### Design Documents

- [design/API-DESIGN.md](design/API-DESIGN.md) - Public API structure
- [design/EXIFTOOL-INTEGRATION.md](design/EXIFTOOL-INTEGRATION.md) - Code generation system
- [design/PRINTCONV-DESIGN-DECISIONS.md](design/PRINTCONV-DESIGN-DECISIONS.md) - Why we diverge from ExifTool's JSON output

### Development Guides

- [guides/EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) - Complete ExifTool reference
- [guides/DEVELOPMENT-GUIDE.md](guides/DEVELOPMENT-GUIDE.md) - Daily development workflow
- [guides/CORE-ARCHITECTURE.md](guides/CORE-ARCHITECTURE.md) - State management & offset calculations

### Reference Materials

- [reference/SUPPORTED-FORMATS.md](reference/SUPPORTED-FORMATS.md) - File formats and MIME types
- [reference/MANUFACTURER-FACTS.md](reference/MANUFACTURER-FACTS.md) - Manufacturer-specific quirks
- [reference/TROUBLESHOOTING.md](reference/TROUBLESHOOTING.md) - Debugging guide

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
