# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with exif-oxide.

## Project Overview

As much as possible, exif-oxide is a _translation_ of [ExifTool](https://exiftool.org/) from perl to Rust.

The biggest "complexifier" for this project is that ExifTool has monthly
releases. New parsers, file types, and bugfixes accompany every new release.

If our codebase is manually ported over, examining thousands of lines of diff to
keep up to date with releases will become sisyphean and untenable.

The current hypothesis involves a balance of manually-written components that
are stitched together by a code generator that reads and parses ExifTool's
largely tabular codebase. This is discussed in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Essential Documentation

Before starting work on exif-oxide, familiarize yourself with:

### Our Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Overall system design and code generation strategy
- [MILESTONES.md](docs/MILESTONES.md) - Development roadmap with 12+ incremental milestones
- [ENGINEER-GUIDE.md](docs/ENGINEER-GUIDE.md) - Practical guide for new contributors
- [STATE-MANAGEMENT.md](docs/STATE-MANAGEMENT.md) - How we handle stateful processing
- [PROCESSOR-PROC-DISPATCH.md](docs/PROCESSOR-PROC-DISPATCH.md) - Processor dispatch strategy
- [OFFSET-BASE-MANAGEMENT.md](docs/OFFSET-BASE-MANAGEMENT.md) - Critical offset calculation patterns

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

### 1. ExifTool is Gospel

- ExifTool is the accumulation of 25 years of camera-specific quirks and edge
  cases, and tens of thousands of bugfixes.

- We must maintain exact tag name and structure compatibility

- **Do not invent any heuristics**. This project is a translation effort. Always
  defer to ExifTool's algorithms, and translate **verbatim**. Chesterton's Fence
  applies here in a big way -- assume that odd, confusing, or obscure `ExifTool`
  code **is that way for a reason**, and **we do not want to nor do we care why
  it is like that**--our only job is to **perfectly translate**.

- Always include a comment pointing back to the ExifTool code (using the
  filename, function or structure, and line numbers) so that Engineers of
  Tomorrow can trace back to where magic values and confusing heuristics
  originated.

**⚠️ CRITICAL**: Never attempt to "improve" or "simplify" ExifTool's logic:

- If ExifTool checks for `0x41` before `0x42`, do it in that order
- If ExifTool has a weird offset calculation, copy it exactly
- If ExifTool special-cases "NIKON CORPORATION" vs "NIKON", there's a reason
- No Camera Follows The Spec. Trust The ExifTool Code.

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