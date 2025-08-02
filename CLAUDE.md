# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with exif-oxide.

## 🚨 CRITICAL: ALWAYS USE ABSOLUTE PATHS 🚨

**NEVER use `cd ..` or `cd ../..` - there have been devastating mistakes due to directory confusion.**

**ALWAYS:**

1. Run `pwd` first to check your current directory
2. Use absolute paths: `cd /home/mrm/src/exif-oxide` or `cd /home/mrm/src/exif-oxide/codegen`
3. When in doubt, ask the user to confirm the intended directory

## Project Overview

As much as possible, exif-oxide is a _translation_ of [ExifTool](https://exiftool.org/) from perl to Rust.

The biggest complexifier for this project is that ExifTool has monthly
releases. New parsers, file types, and bugfixes accompany every new release.

If our codebase is manually ported over, examining thousands of lines of diff to
keep up to date with releases will become sisyphean and untenable.

This project attempts to balance manually-written components that are stitch
together code from our automated [docs/CODEGEN.md](docs/CODEGEN.md) ExifTool
perl-to-rust code generation system. This is discussed in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## ⚠️ CRITICAL: Trust ExifTool

**This is the #1 rule for all work on exif-oxide.**

See [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) for the complete guidelines.

The key principle: **wholly and completely trust the ExifTool implementation.**

Any time we stray from ExifTool's logic and heuristics will introduce defects in real-world tasks.

Almost every task will involve studying some part of the ExifTool codebase and validating that we are doing something exactly equivalent.

**Note on Unknown Tags**: We follow ExifTool's default behavior of omitting tags marked with `Unknown => 1`. These tags are only shown in ExifTool when using the `-u` flag. This keeps our output clean and matches user expectations.

## ⚠️ CRITICAL: Assume Concurrent Edits

There are several other engineers working _on the same copy of the source tree_ at the same time you are. If you ever encounter a build error that you are positive is not a side-effect from code that you've edited, **STOP** and tell the user the issue. The user will fix the build nd tell you when you can resume your work.

## Essential Documentation

This project has many non-intuitive aspects. Before starting any task, **READ THE RELEVANT DOCUMENTATION**.

If you skip this step, your work will likely be spurious, wrong, and rejected.

### Our Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - High-level system overview and philosophy
- [MILESTONES.md](docs/MILESTONES.md) - Active development milestones
- [EXCLUDED-TAGS.md](docs/EXCLUDED-TAGS.md) - Tags excluded from implementation scope
- [TPP.md](docs/TPP.md) - Technical Project Plan template with priority naming conventions

#### Design Documents

- [API-DESIGN.md](docs/design/API-DESIGN.md) - Public API structure and TagEntry design
- [CODEGEN.md](docs/CODEGEN.md) - Unified code generation and implementation guide
- [PRINTCONV-VALUECONV-GUIDE.md](docs/guides/PRINTCONV-VALUECONV-GUIDE.md) - PrintConv/ValueConv implementation guide and design decisions

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

## Critical Development Principles

### 0. Ask clarifying questions

If you have any clarifying questions for any aspects that are odd, nebulous, confusing, inadequately specific, or otherwise unclear, **please ask the user**.

The user assumes every task will need at least a couple clarifying questions before starting work!

### 1. Trust ExifTool

READ [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md)

The Trust ExifTool principle is the fundamental law of the exif-oxide project: we translate ExifTool's implementation exactly, never attempting to "improve," "optimize," or "simplify" its logic, because every seemingly odd piece of code exists to handle specific camera quirks discovered over 25 years of development. The only changes allowed are syntax translations required for Rust (like string formatting or type conversions), but the **underlying logic must remain identical**. This principle exists because no camera follows the spec perfectly, and ExifTool's battle-tested code handles millions of real-world files from thousands of camera models with their unique firmware bugs and non-standard behaviors.

Whenever possible, our rust code should include a comment pointing back to the ExifTool source code, function or variable name, and line number range.

Or better: use CODEGEN!

### 2. Only `perl` can parse `perl`

WE CANNOT INTERPRET PERL CODE IN RUST.

The perl interpreter is the only competent perl parser! There are too many gotchas and surprising perl-isms--any perl parser we make in rust needs to be super conservative and strict with its allowed inputs.

### 3. Incremental improvements with a focus on common, mainstream tags

To maintain a manageable scope:

- We are initially targeting support for tags with >80% frequency or marked `mainstream: true` in TagMetadata.json
- This reduces scope from ExifTool's 15,000+ tags to approximately 500-1000
- See [TagMetadata.json](docs/tag-metadata.json) for tag popularity data

### 4. Look for codegen opportunities

ExifTool releases new versions monthly. The more our code can be generated automatically from ExifTool source, the better.

**CRITICAL**: If you ever see any manually-ported static maps or sets in our non-`src/generated/**/*.rs` code, **immediately look for where that came from in the ExifTool source, and ask the user to rewrite it with the codegen infrastructure**. See [CODEGEN.md](docs/CODEGEN.md) "Simple Table Extraction Framework" for details.

#### What to Look For

- **HashMap/match statements** with >5 static entries
- **Lens identification databases** (should use simple table framework)
- **Camera model mappings** (should be generated)
- **Mode/setting lookup tables** (white balance, picture styles, etc.)
- **Any hardcoded string constants** that map values to names

#### How to Address

1. **Find the ExifTool source** - Usually a `%hashName = (...)` pattern
2. **Check if primitive** - Only numbers/strings, no Perl expressions
3. **Add to module config** - Add to appropriate `codegen/config/$ModuleName_pm/simple_table.json`
4. **Regenerate codegen** - `make codegen`
5. **Replace manual code** - Use generated lookup functions

See [CODEGEN.md](docs/CODEGEN.md) for more details.

#### Red Flags

If you see ANY of these, immediately suggest codegen extraction:

- Files with hundreds of manual constant definitions
- Match statements mapping numbers to camera/lens names
- Static arrays of string literals that look like they came from ExifTool
- TODO comments about "add more lens types when we have time"
- Version-specific model lists that need manual updates

**Remember**: Manually translated lookup tables are a minefield of bugs -- they're difficult to compare with the source material, frequently contain subtle translation mistakes, and are a substantial maintenance burden that grows with each ExifTool release. The codegen system automates hundreds of perl-encoded tables with zero ongoing maintenance costs.

### 5. DO NOT EDIT THE FILES THAT SAY DO NOT EDIT

Everything in `src/generated` **is generated code** -- if you edit the file directly, the next time `make codegen` is run, your edit will be deleted. Fix the generating code in `codegen/src` instead.

YOU WILL BE IMMEDIATELY FIRED IF YOU IGNORE THIS WARNING, as you've obviously not read this manual, and all your other work will be circumspect.

### Choosing the Right Extractor

When working with the codegen system, use the right extractor for each task:

1. **Extracting tags with PrintConvs?** → Use `tag_kit.pl` (the unified tag extraction system)
2. **Extracting standalone lookups?** → Use `simple_table.pl` (for manufacturer lookup tables)
3. **Extracting binary data tables?** → Use `process_binary_data.pl` or `runtime_table.pl`

**Important**: We're migrating to the tag kit system for all tag-related extraction. If you see configs for `inline_printconv.pl`, `tag_tables.pl`, or `tag_definitions.pl`, suggest converting them to tag kit instead.

See [EXTRACTOR-GUIDE.md](docs/reference/EXTRACTOR-GUIDE.md) for detailed extractor comparisons and [CODEGEN.md](docs/CODEGEN.md) for the complete extractor selection guide.

### Bug Fixing

**MANDATORY**: When a bug is discovered, follow the test-driven debugging workflow in [TDD.md](docs/TDD.md):

1. Create a breaking test that reproduces the issue
2. Validate that the test fails for the expected reason
3. Fix the bug following "Trust ExifTool" principles
4. Validate that the test now passes and no regressions occur

### When a task is complete

1. Verify and validate! No task is complete until `make precommit`
   passes.

2. Concisely update any impacted and related docs, including reference
   documentation, todo lists, milestone planning, and architectural design.

### The user is a rust newbie...

...so explaining things as we go would be wonderful. We want to make this
project be as idiomatic rust as possible, so please web search and examine the
rust language documentation to validate structures, setup, naming conventions,
module interactions, and any other aspects that the rust community has adopted
as a best practice, and explain those aspects to the user as we embrace them.

### Task prioritization and naming

When creating Technical Project Plans (TPPs) or TODO documents, use the priority naming convention defined in [TPP.md](docs/TPP.md):

- `P00-P09` - Critical blockers
- `P10-P19` - Maximum required tag impact (JPEG + Video ecosystem, binary extraction)
- `P20-P29` - Technical debt
- `P30-P39` - Architecture improvements
- `P40-P49` - Video format support (if not required tag related)
- `P50-P59` - RAW format support (low required tag impact)
- `P60+` - Long-term/speculative work

Add letter suffixes (a, b, c) only for strong prerequisites.
When moving to `docs/done/`, prefix with completion date: `YYYYMMDD-P10a-description.md`

**Priority Rationale**: Focus on extracting all required tags from docs/tag-metadata.json. P10-P19 covers ~97% of required tags (JPEG ecosystem + video). RAW formats (P50s) only add 3 required tags but become useful once binary extraction (P16) enables preview/thumbnail extraction.

## Development guidance

### Running Tests

**Use `cargo t` instead of `cargo test`** - Integration tests require the `test-helpers` feature to access test helper methods like `add_test_tag()`. We've configured a cargo alias for convenience:

- `cargo t` - Run all tests with test features enabled (shorthand)
- `cargo t pattern` - Run tests matching "pattern"
- `cargo t test_png_pattern_directly` - Run specific test

The alias is defined in `.cargo/config.toml` and automatically includes `--features test-helpers,integration-tests`.

**Why not regular `cargo test`?** The `test-helpers` feature enables test-only public methods on `ExifReader` that integration tests need, and `integration-tests` enables tests requiring external test assets. We don't include these in default features to keep them out of release builds.

### ⚠️ IMPORTANT: ExifTool is a Git Submodule

The `third-party/exiftool` directory is a **git submodule**. This means:

- **NEVER run `git checkout`, `git add`, or any git commands directly on files in this directory**
- The submodule tracks a specific commit of the ExifTool repository
- Any changes to files in `third-party/exiftool/` will affect the submodule state
- The codegen process may temporarily patch ExifTool files, but these changes should be reverted automatically
- If you need to update or modify anything in the ExifTool directory, coordinate with the user first

### Watch for manually-ported hashes that could use codegen

Be vigilant for manually-maintained lookup tables and hash mappings that could be automatically generated. If you encounter any static mappings, immediately:

1. Check if it came from ExifTool source (usually a `%hashName = (...)` pattern)
2. Suggest converting it to use the codegen infrastructure
3. See [CODEGEN.md](docs/CODEGEN.md) for the ExifTool code extraction framework

This is critical for maintainability as ExifTool releases monthly updates.

### Refactor large source files

When working with source files that exceed 500 lines:

1. Suggest refactoring into smaller, focused modules using semantic grouping (completed for generated files in July 2025)
2. The Read tool will truncate files larger than 2000 lines, which can cause incomplete code analysis
3. Breaking up large files improves:
   - Code readability and maintenance
   - Tool effectiveness for analysis
   - Module organization and separation of concerns
   - IDE performance and compile times

### Mark where the code smells

While reviewing or editing code, if there are components that feel like a temporary hack or otherwise have a bad "code smell", add a TODO comment into the code that tersely describes why it smells, along with either a link to a MILESTONES.md stage when it will be fixed, or a terse description of how it should be fixed in the future.

### Safety rules

- **NEVER use `rm -rf` in scripts** - it's too dangerous and can accidentally delete important files. Use specific file patterns with `rm -f` instead (e.g., `rm -f "$DIR/*.json"`)
- Always prefer targeted cleanup over recursive deletion
- **Use existing dependencies** - prefer already-imported crates (like `std::sync::LazyLock`) instead of adding new external dependencies unless really necessary

### Wondering what's going on?

Check the debug logging -- and if a component is missing debug logging, feel free to add it.
We use `tracing`, and there's lots of examples in `src/main.rs`.

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

#### 2. Shell script (simple diff)

The `scripts/compare-with-exiftool.sh` script provides a basic JSON diff:

```bash
# Compare all tags
./scripts/compare-with-exiftool.sh image.jpg

# Compare only specific group tags
./scripts/compare-with-exiftool.sh image.jpg File:
```

Environment variables:

- `DEBUG=1` - Keep the raw outputs for debugging
- `DIFF_CONTEXT=3` - Show more context lines in diff (default is 0 for minimal diff)

### Git commit messages

All commit messages must follow the Conventional Commits specification (https://www.conventionalcommits.org/en/v1.0.0/). Use the format: `<type>[optional scope]: <description>` where type is `feat` (new features, MINOR version), `fix` (bug patches, PATCH version), or other types like `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`. Breaking changes are indicated with `!` after type/scope or with a `BREAKING CHANGE:` footer. The scope should reference the most significant file/module changed. Keep descriptions concise and avoid enumerating every change unless crucial for understanding.

### Is this tag prevalent?

Tag "popularity" ranges widely. First check `docs/tag-metadata.json`, but if it's not there, use `exiftool` to do your own research! For the `ISOSpeed` tag, for example:

```
exiftool -j -struct -G -r -if '$ISOSpeed' -ISOSpeed test-images/ third-party/exiftool/t/images/ ../test-images/
```

Which shows it's only in the EXIF group, and in 20/10693 of our sample files.

### Test images

There are many test images in `third-party/exiftool/t/image/` -- but they've all had their image content stripped out, so they're all 8x8. Don't test things like dimensions with those files -- we need proper, original out-of-camera examples to test with. Those live in `test-images/${manufacturer name}`
