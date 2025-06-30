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
largely tabular codebase. This is discussed in docs/ARCHITECTURE.md

## Critical Development Principles

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

### 3. When a task is complete

1. Verify and validate! No task is complete until both `make fix` and `make test`
   pass. Many tasks will require adding new integration tests.

2. Concisely update any impacted and related docs, including reference
   documentation, todo lists, milestone planning, and architectural design.
