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
- [CODEGEN-STRATEGY.md](docs/design/CODEGEN-STRATEGY.md) - Code generation approach
- [IMPLEMENTATION-PALETTE.md](docs/design/IMPLEMENTATION-PALETTE.md) - Manual implementation patterns

#### Technical Deep Dives

- [STATE-MANAGEMENT.md](docs/STATE-MANAGEMENT.md) - How we handle stateful processing
- [PROCESSOR-PROC-DISPATCH.md](docs/PROCESSOR-PROC-DISPATCH.md) - Processor dispatch strategy
- [OFFSET-BASE-MANAGEMENT.md](docs/OFFSET-BASE-MANAGEMENT.md) - Critical offset calculation patterns

#### Guides

- [EXIFTOOL-CONCEPTS.md](docs/guides/EXIFTOOL-CONCEPTS.md) - Critical ExifTool concepts
- [READING-EXIFTOOL-SOURCE.md](docs/guides/READING-EXIFTOOL-SOURCE.md) - Navigating ExifTool's Perl code
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

See [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md). This is the #1 most important principle - trust ExifTool, not the spec. We translate ExifTool **verbatim**, including all its quirks and apparent inefficiencies.

### 2. Only `perl` can parse `perl`

WE CANNOT INTERPRET PERL CODE IN RUST. Only perl is competent at parsing perl.
There are too many gotchas and surprising perlisms--any rust parser we make will
be brittle and haunt us in the future.

### 3. Scope: Mainstream Tags Only

To maintain a manageable scope:

- We only implement tags with >80% frequency or marked `mainstream: true` in TagMetadata.json
- This reduces scope from ExifTool's 15,000+ tags to approximately 500-1000
- See [TagMetadata.json](third-party/exiftool/doc/TagMetadata.json) for tag popularity data

### 4. When a task is complete

1. Verify and validate! No task is complete until both `make fix` and `make test`
   pass. Many tasks will require adding new integration tests.

2. Concisely update any impacted and related docs, including reference
   documentation, todo lists, milestone planning, and architectural design.

### 5. The user is a rust newbie...

...so explaining things as we go would be wonderful. We want to make this
project be as idiomatic rust as possible, so please web search and examine the
rust language documentation to validate structures, setup, naming conventions,
module interactions, and any other aspects that the rust community has adopted
as a best practice, and explain those aspects to the user as we embrace them.

## Development guidance

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
