# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with exif-oxide.

## Project Overview

As much as possible, exif-oxide is a _translation_ of [ExifTool](https://exiftool.org/) from perl to Rust.

The biggest complexifier for this project is that ExifTool has monthly
releases. New parsers, file types, and bugfixes accompany every new release.

If our codebase is manually ported over, examining thousands of lines of diff to
keep up to date with releases will become sisyphean and untenable.

The current hypothesis involves a balance of manually-written components that
are stitched together by a code generator that reads and parses ExifTool's
largely tabular codebase. This is discussed in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## ⚠️ CRITICAL: Trust ExifTool

**This is the #1 rule for all work on exif-oxide.**

See [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) for the complete guidelines.

The key principle: **wholly and completely trust the ExifTool implementation.**

Any time we stray from ExifTool's logic and heuristics will introduce defects in real-world tasks.

Almost every task will involve studying some part of the ExifTool codebase and validating that we are doing something exactly equivalent.

## Essential Documentation

Before starting work on exif-oxide, familiarize yourself with:

### Our Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - High-level system overview and philosophy
- [MILESTONES.md](docs/MILESTONES.md) - Active development milestones
- [ENGINEER-GUIDE.md](docs/ENGINEER-GUIDE.md) - Starting point for new contributors

#### Design Documents

- [API-DESIGN.md](docs/design/API-DESIGN.md) - Public API structure and TagEntry design
- [EXIFTOOL-INTEGRATION.md](docs/design/EXIFTOOL-INTEGRATION.md) - Unified code generation and implementation guide

#### Technical Deep Dives

- [STATE-MANAGEMENT.md](docs/STATE-MANAGEMENT.md) - How we handle stateful processing
- [PROCESSOR-PROC-DISPATCH.md](docs/PROCESSOR-PROC-DISPATCH.md) - Processor dispatch strategy
- [OFFSET-BASE-MANAGEMENT.md](docs/OFFSET-BASE-MANAGEMENT.md) - Critical offset calculation patterns

#### Guides

- [EXIFTOOL-CONCEPTS.md](docs/guides/EXIFTOOL-CONCEPTS.md) - Critical ExifTool concepts
- [READING-EXIFTOOL-SOURCE.md](docs/guides/READING-EXIFTOOL-SOURCE.md) - Navigating ExifTool's Perl code
- [TESTING.md](docs/guides/TESTING.md) - Testing details
- [DEVELOPMENT-WORKFLOW.md](docs/guides/DEVELOPMENT-WORKFLOW.md) - Day-to-day development process
- [COMMON-PITFALLS.md](docs/guides/COMMON-PITFALLS.md) - Common mistakes and debugging
- [TRIBAL-KNOWLEDGE.md](docs/guides/TRIBAL-KNOWLEDGE.md) - Undocumented quirks
- [EXIFTOOL-UPDATE-WORKFLOW.md](docs/guides/EXIFTOOL-UPDATE-WORKFLOW.md) - Updating to new ExifTool versions

### ExifTool Documentation

- [MODULE_OVERVIEW.md](third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md) - Overview of ExifTool's module structure
- [PROCESS_PROC.md](third-party/exiftool/doc/concepts/PROCESS_PROC.md) - How ExifTool processes different data formats
- [VALUE_CONV.md](third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable output conversions
- [PATTERNS.md](third-party/exiftool/doc/concepts/PATTERNS.md) - Common patterns across modules

## Critical Development Principles

### 0. Ask the user clarifying questions

If you have any clarifying questions for any aspects that are odd, nebulous,
confusing, inadequately specific, or otherwise unclear, **please ask the user**.

The user assumes every task will need at least a couple clarifying questions
before starting work!

### 1. Trust ExifTool

READ [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md)

The Trust ExifTool principle is the fundamental law of the exif-oxide project: we translate ExifTool's implementation exactly, never attempting to "improve," "optimize," or "simplify" its logic, because every seemingly odd piece of code exists to handle specific camera quirks discovered over 25 years of development. The only changes allowed are syntax translations required for Rust (like string formatting or type conversions), but the **underlying logic must remain identical**. This principle exists because no camera follows the spec perfectly, and ExifTool's battle-tested code handles millions of real-world files from thousands of camera models with their unique firmware bugs and non-standard behaviors.

Whenever possible, our rust code should include a comment pointing back to the ExifTool source code, function or variable name, and line number range.

### 2. Only `perl` can parse `perl`

WE CANNOT INTERPRET PERL CODE IN RUST.

The perl interpreter is the only competent perl parsing! There are too many gotchas and surprising perl-isms--any perl parser we make in rust or regex is a bad idea, be brittle, lead us to ruin, and haunt us in the future.

### 3. Incremental improvements with a focus on common, mainstream tags

To maintain a manageable scope:

- We are initially targeting support for tags with >80% frequency or marked `mainstream: true` in TagMetadata.json
- This reduces scope from ExifTool's 15,000+ tags to approximately 500-1000
- See [TagMetadata.json](third-party/exiftool/doc/TagMetadata.json) for tag popularity data

### 4. Look for easy codegen wins

ExifTool releases new versions monthly. The more our code can be generated automatically from ExifTool source, the better.

**CRITICAL**: If you ever see any simple, static mapping in our code, **immediately look for where that came from in the ExifTool source, and ask the user to rewrite it with the codegen infrastructure**. See [EXIFTOOL-INTEGRATION.md](docs/design/EXIFTOOL-INTEGRATION.md) "Simple Table Extraction Framework" for details.

#### Simple Table Detection

Be especially vigilant for these patterns that should NEVER be manually maintained:

❌ **Manual lookup tables** (should be generated):
```rust
// BAD - This should be generated from ExifTool!
fn canon_white_balance_lookup(value: u8) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Daylight", 
        2 => "Cloudy",
        3 => "Tungsten",
        _ => "Unknown",
    }
}
```

✅ **Using generated tables**:
```rust
// GOOD - Using simple table extraction framework
use crate::generated::canon::white_balance::lookup_canon_white_balance;

fn canon_white_balance_print_conv(value: &TagValue) -> Result<String> {
    if let Some(wb_value) = value.as_u8() {
        if let Some(description) = lookup_canon_white_balance(wb_value) {
            return Ok(description.to_string());
        }
    }
    Ok(format!("Unknown ({})", value))
}
```

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
4. **Regenerate codegen** - `make codegen-extract`
5. **Replace manual code** - Use generated lookup functions

See [EXIFTOOL-INTEGRATION.md](docs/design/EXIFTOOL-INTEGRATION.md#simple-table-extraction-framework) for the complete HOWTO guide.

#### Red Flags

If you see ANY of these, immediately suggest codegen extraction:
- Files with hundreds of manual constant definitions
- Match statements mapping numbers to camera/lens names  
- Static arrays of string literals that look like they came from ExifTool
- TODO comments about "add more lens types when we have time"
- Version-specific model lists that need manual updates

**Remember**: Every manually maintained lookup table is a maintenance burden that grows with each ExifTool release. The simple table extraction framework can automate hundreds of these tables with zero ongoing maintenance cost.

### 5. When a task is complete

1. Verify and validate! No task is complete until `make precommit`
   passes.

2. Concisely update any impacted and related docs, including reference
   documentation, todo lists, milestone planning, and architectural design.

### 6. The user is a rust newbie...

...so explaining things as we go would be wonderful. We want to make this
project be as idiomatic rust as possible, so please web search and examine the
rust language documentation to validate structures, setup, naming conventions,
module interactions, and any other aspects that the rust community has adopted
as a best practice, and explain those aspects to the user as we embrace them.

## Development guidance

### Watch for manually-ported hashes that could use codegen

Be vigilant for manually-maintained lookup tables and hash mappings that could be automatically generated. If you encounter any static mappings, immediately:

1. Check if it came from ExifTool source (usually a `%hashName = (...)` pattern)
2. Suggest converting it to use the codegen infrastructure
3. See [EXIFTOOL-INTEGRATION.md](docs/design/EXIFTOOL-INTEGRATION.md) for the simple table extraction framework

This is critical for maintainability as ExifTool releases monthly updates.

### Refactor large source files

When working with source files that exceed 500 lines:

1. Suggest refactoring into smaller, focused modules
2. The Read tool will truncate files larger than 2000 lines, which can cause incomplete code analysis
3. Breaking up large files improves:
   - Code readability and maintenance
   - Tool effectiveness for analysis
   - Module organization and separation of concerns

### Mark where the code smells

While reviewing or editing code, if there are components that feel like a
temporary hack or otherwise have a bad "code smell", ask the user to add a TODO
comment into the code that tersely describes why it smells, along with either a
link to a MILESTONES.md stage when it will be fixed, or a terse description of
how it should be fixed in the future.

### Safety rules

- **NEVER use `rm -rf` in scripts** - it's too dangerous and can accidentally delete important files. Use specific file patterns with `rm -f` instead (e.g., `rm -f "$DIR/*.json"`)
- Always prefer targeted cleanup over recursive deletion

### Wondering what's going on?

Check the debug logging -- and if a component is missing debug logging, feel free to add it.
We use `tracing`, and there's lots of examples in `src/main.rs`.

### Git commit messages

All commit messages must follow the Conventional Commits specification
(https://www.conventionalcommits.org/en/v1.0.0/). Use the format:
`<type>[optional scope]: <description>` where type is `feat` (new features,
MINOR version), `fix` (bug patches, PATCH version), or other types like `docs`,
`style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`. Breaking changes
are indicated with `!` after type/scope or with a `BREAKING CHANGE:` footer. The
scope should reference the most significant file/module changed. Keep
descriptions concise and avoid enumerating every change unless crucial for
understanding.
