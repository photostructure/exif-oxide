# CLAUDE.md - MANDATORY CHECKLIST

## 🚨 CRITICAL: ALWAYS USE ABSOLUTE PATHS 🚨

**NEVER use `cd ..` or `cd ../..` - there have been devastating mistakes due to directory confusion.**

**ALWAYS:**

1. Run `pwd` first to check your current directory
2. Use absolute paths: `cd /home/mrm/src/exif-oxide` or `cd /home/mrm/src/exif-oxide/codegen`
3. When in doubt, ask the user to confirm the intended directory

## 🚨 BEFORE ANY CODE CHANGES - RUN THESE CHECKS 🚨

```bash
# 1. Are you in the right directory?
pwd  # MUST show /home/mrm/src/exif-oxide or subdirectory

# 2. Are you about to edit generated code?
echo "Files in src/generated/ are AUTO-GENERATED. Edit codegen/src/ instead."

# 3. Check for forbidden patterns in your changes:
rg "split_whitespace|\.join.*split" codegen/src/ppi/  # MUST return empty
```

## 🔴 INSTANT REJECTION TRIGGERS

These will get your PR reverted immediately:

1. **Editing any file in `**/generated/`** → These are generated. Fix `codegen/src/` instead.
2. **Using `split_whitespace()` on AST nodes** → Breaks Perl parsing
3. **Deleting ExifTool patterns** → Breaks camera support 
4. **"Improving" ExifTool logic** → We translate EXACTLY. No optimizations.
5. **Manual data transcription** → Use codegen for ALL ExifTool data

## ✅ MANDATORY BEFORE EVERY PR

```bash
make precommit  # MUST pass
cargo t         # MUST pass (not cargo test - needs test-helpers)
```

## 📁 Directory Safety

**NEVER** use `cd ..` or relative paths. **ALWAYS** use absolute paths:
```bash
cd /home/mrm/src/exif-oxide          # ✅ GOOD
cd /home/mrm/src/exif-oxide/codegen  # ✅ GOOD  
cd ../..                              # ❌ WILL CAUSE DISASTERS
```

## 🚨 CRITICAL: stderr redirects are broken in the Bash tool 🚨

**You can't use `2>&1` in your bash commands** -- your Bash tool will mangle the stderr redirect and pass a "2" as an arg, and you won't see stderr.

**Workarounds**:

- Use `./scripts/capture.sh command args` to redirect output to temp files (useful for large outputs) - it will echo the file paths you can then grep/rg/awk through if the stream was non-trivial.
- Don't use either of these tools if you don't care about stderr and/or the output is expected to be 100 lines or less.

See https://github.com/anthropics/claude-code/issues/4711 for details.

## 📚 Critical Documentation

**READ THESE FIRST:**
- [ANTI-PATTERNS.md](docs/ANTI-PATTERNS.md) - What NOT to do (with horror stories)
- [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) - Core principle: translate EXACTLY
- [CODEGEN.md](docs/CODEGEN.md) - How code generation works

### Our Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - High-level system overview and philosophy
- [MILESTONES.md](docs/MILESTONES.md) - Active development milestones
- [EXCLUDED-TAGS.md](docs/EXCLUDED-TAGS.md) - Tags excluded from implementation scope
- [TPP.md](docs/TPP.md) - Technical Project Plan template with priority naming conventions

#### Design Documents

- [API-DESIGN.md](docs/design/API-DESIGN.md) - Public API structure and TagEntry design
- [CODEGEN.md](docs/CODEGEN.md) - Unified code generation and implementation guide
- [PRINTCONV-VALUECONV-GUIDE.md](docs/guides/PRINTCONV-VALUECONV-GUIDE.md) - PrintConv/ValueConv implementation guide and design decisions
- [TDD.md](docs/TDD.md) - **TL;DR**: Mandatory bug-fixing workflow: (1) write breaking test, (2) validate it fails, (3) fix bug following Trust ExifTool, (4) validate test passes + no regressions
- [SIMPLE-DESIGN.md](docs/SIMPLE-DESIGN.md) - **TL;DR**: Kent Beck's Four Rules of Simple Design in priority order: (1) passes tests, (2) reveals intention, (3) no duplication, (4) fewest elements

#### Guides

- [GETTING-STARTED.md](docs/GETTING-STARTED.md) - Quick start guide for new contributors
- [CORE-ARCHITECTURE.md](docs/guides/CORE-ARCHITECTURE.md) - Core system architecture and offset management
- [EXIFTOOL-GUIDE.md](docs/guides/EXIFTOOL-GUIDE.md) - Complete guide to working with ExifTool source
- [PROCESSOR-DISPATCH.md](docs/guides/PROCESSOR-DISPATCH.md) - Processor dispatch strategy

#### Reference

- [MANUFACTURER-FACTS.md](docs/reference/MANUFACTURER-FACTS.md) - Manufacturer-specific implementation facts
- [SUPPORTED-FORMATS.md](docs/reference/SUPPORTED-FORMATS.md) - Currently supported file formats
- [TROUBLESHOOTING.md](docs/reference/TROUBLESHOOTING.md) - Common issues and solutions

### ExifTool Documentation

- [MODULE_OVERVIEW.md](third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md) - Overview of ExifTool's module structure
- [PROCESS_PROC.md](third-party/exiftool/doc/concepts/PROCESS_PROC.md) - How ExifTool processes different data formats
- [VALUE_CONV.md](third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable output conversions
- [PATTERNS.md](third-party/exiftool/doc/concepts/PATTERNS.md) - Common patterns across modules


## ⚠️ IMPORTANT: ExifTool is a Git Submodule

The `third-party/exiftool` directory is a **git submodule**. This means:

- **NEVER run `git checkout`, `git add`, or any git commands directly on files in this directory**
- The submodule tracks a specific commit of the ExifTool repository
- Any changes to files in `third-party/exiftool/` will affect the submodule state
- The codegen process may temporarily patch ExifTool files, but these changes should be reverted automatically
- If you need to update or modify anything in the ExifTool directory, coordinate with the user first

## Running Tests

**Use `cargo t` instead of `cargo test`** - Integration tests require the `test-helpers` feature to access test helper methods like `add_test_tag()`. We've configured a cargo alias for convenience:

- `cargo t` - Run all tests with test features enabled (shorthand)
- `cargo t pattern` - Run tests matching "pattern"
- `cargo t test_png_pattern_directly` - Run specific test

The alias is defined in `.cargo/config.toml` and automatically includes `--features test-helpers,integration-tests`.

**Why not regular `cargo test`?** The `test-helpers` feature enables test-only public methods on `ExifReader` that integration tests need, and `integration-tests` enables tests requiring external test assets. We don't include these in default features to keep them out of release builds.

## ⚠️ CRITICAL: Bug Fixing

When a bug is discovered, follow the test-driven debugging workflow documented in [TDD.md](docs/TDD.md):

1. **Create a breaking test** that reproduces the issue with minimal test data
2. **Validate test explodes** - confirm it fails for the exact expected reason
3. **Address the bug** following "Trust ExifTool" principles (check ExifTool's implementation)
4. **Validate test passes** and run full test suite (`cargo t`) for regressions

This workflow ensures bugs are properly isolated, fixed at root cause, and protected against future regressions. See [TDD.md](docs/TDD.md) for complete details, examples, and test organization best practices.

### Comparing with ExifTool

Two tools are available for comparing exif-oxide output with ExifTool:

#### 1. Rust-based comparison tool (recommended)

The `compare-with-exiftool` binary uses the same value normalization logic as our compatibility tests:

```bash
# Build the tool
cargo build --bin compare-with-exiftool

# Compare all tags
cargo run --bin compare-with-exiftool image.jpg

# Compare only File: group tags
cargo run --bin compare-with-exiftool image.jpg File:

# Compare only EXIF: group tags
cargo run --bin compare-with-exiftool image.jpg EXIF:
```

This tool:

- Normalizes values using the same logic as our test suite (e.g., "25 MB" → "26214400")
- Shows only actual differences, not formatting variations
- Groups differences into: tags only in ExifTool, tags only in exif-oxide, and tags with different values
- Handles ExifTool's inconsistent formatting across different modules


---

**IF YOU IGNORE ANY ASPECT IN THIS DOCUMENT:** Your work will be reverted, and you'll have to start over.
