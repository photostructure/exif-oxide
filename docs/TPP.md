# Technical Project Plan (TPP)

A TPP ensures new engineers succeed by providing critical context, not just task lists.

## Goals

- **Purpose**: Context prevents wrong implementations. Tasks without "why" = dangerous.
- **Length**: < 500 lines (ReadTool limit)
- **Style**: Bullet points > prose. Examples > abstractions.
- **Only include sections that help** - omit the rest

---

## Required Boilerplate

Add this verbatim after Project Overview:

```md
---

## ⚠️ CRITICAL REMINDERS

- **MANDATORY: READ THESE TWO DOCUMENTS**: [CLAUDE.md](../CLAUDE.md) | [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)
- **Concurrent edits**: If build errors aren't near your code → STOP, tell user
- **Ask questions**: Confused about approach? Debugging >1hr? ASK before continuing
- **Keep this document updated with progress!**: Use 🟢Done/🟡WIP/🔴Blocked status as you work.
- **Add discoveries and research**: Add context that will be helpful to future engineers completing this task, or for future relevant tasks. 
- **Don't oversell your progress**:  Do not use hyperbolic "DRAMATIC IMPROVEMENT!"/"GROUNDBREAKING PROGRESS" styled updates.

Key sections to always apply from CLAUDE.md:
- "Assume Concurrent Edits" - Critical safety rule
- "Trust ExifTool" - Core principle #1  
- "Only perl can parse perl" - Codegen constraints
- "Look for easy codegen wins" - Maintenance strategy

---
```

## Project Overview

- **Goal**: [What success looks like in 1-2 sentences]
- **Problem**: [What's broken and why]
- **Constraints**: [Non-negotiable requirements]

Example:

- **Goal**: Fix PrintConv to show `3.9` not `[39, 10]`
- **Problem**: Conversion pipeline broken for array values
- **Constraints**: Zero runtime overhead, no circular deps

## Context & Foundation

- **Why**: [Business/technical driver]
- **Docs**: [Design docs, ExifTool source links]
- **Start here**: [Specific files/functions to examine first]

## Work Completed

- ✅ [Feature] → chose [approach] over [alternative] because [reason]
- ✅ [Decision] → rejected [option] due to [constraint]

## Remaining Tasks

### Task: [Name]

**Success**: [Show actual correct output/behavior]

**Failures to avoid**:

- ❌ [Common mistake] → [bad consequence]
- ❌ [Another trap] → [why it fails]

**Approach**: [Strategy, not steps]

### RESEARCH: [Topic]

**Questions**: [What exactly to discover]
**Done when**: [Specific deliverable/answer obtained]

## Prerequisites

- [Dependency] → [TPP link] → verify with `[command/test]`

## Testing

- **Unit**: Test [specific functions/edge cases]
- **Integration**: Verify [end-to-end scenario]
- **Manual check**: Run `[command]` and confirm [expected output]

## Definition of Done

- [ ] `cargo t [test_name]` passes
- [ ] `make precommit` clean
- [ ] [Specific acceptance criteria]

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

Real examples:

- **src/generated/\* looks buggy** → It's generated → Fix codegen configs, not the output
- **ExifTool does weird thing** → 25 years of camera bugs → Trust and copy exactly
- **Where's this tag?** → Could be anywhere → Check: generated/, composite_tags/, implementations/
- **Composite tag implemented but not working** → Probably missing from generated definitions → Check if it's in module-specific `%ModuleName::Composite` table that codegen doesn't extract
- **Manual edits to generated files disappear** → Codegen overwrites them → Always fix extraction configs, never edit generated code directly

## Quick Debugging

Stuck? Try these:

1. `grep -r "TagName" src/` - Find all uses
2. `rg "tag_name" third-party/exiftool/` - Check ExifTool impl
3. `cargo t test_name -- --nocapture` - See debug prints
4. `git log -S "feature"` - Find when/why added

---

## File Naming

`docs/todo/PXX-short-kebab-description.md`

- PXX = 2-digit priority (P00 highest → P99 lowest)
- Suffixes (a,b,c) ONLY for dependencies: P10a must complete before P10b
- Done: Move to `docs/done/YYYYMMDD-PXX-description.md`
