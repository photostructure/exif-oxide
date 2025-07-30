# Technical Project Plan (TPP)

A TPP ensures new engineers succeed by providing critical context AND clear, actionable tasks.

## Goals

**TL;DR:** a high quality technical project plan **provides the context necessary for an engineer to successfully accomplish a given task or set of tasks.** Everything else is secondary.

The remainder of this document provides some consistent structure, but omit irrelevant sections, and if there is content that doesn't fit into this structure, feel free to add that in a novel section.

- **Purpose**: Rich context + clear tasks prevent wrong implementations. Tasks without "why" = dangerous.
- **Length**: < 500 lines (ReadTool limit)
- **Style**: Bullet points > prose. Examples > abstractions.
- **Context First**: Document surprising/non-intuitive aspects that casual observers may miss

---

## Required Boilerplate

Add this verbatim after Project Overview:

```md
---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---
```

## Project Overview

- **Goal**: [What success looks like in 1-4 sentences]
- **Problem**: [What's broken and why]
- **Constraints**: [Non-negotiable requirements -- optional]

Example:

- **Goal**: Fix PrintConv to show `3.9` not `[39, 10]`
- **Problem**: Conversion pipeline broken for array values
- **Constraints**: Zero runtime overhead, no circular deps

## Context & Foundation

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **Components involved**: [2-3 sentence "cliff's notes" for each system this touches]
- **Key interactions**: [How these systems work together]

### Key Concepts & Domain Knowledge

- **Technical terms**: [Define domain-specific concepts]
- **Business logic**: [Why things work this way]

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Hidden dependencies**: [What relies on what in non-obvious ways]
- **Counterintuitive behaviors**: [Things that work differently than expected]
- **Historical quirks**: [Why code exists in seemingly odd states]
- **Gotchas**: [What will trip up future engineers]

### Foundation Documents

- **Design docs**: [Links to architectural decisions]
- **ExifTool source**: [Relevant perl code references]
- **Start here**: [Specific files/functions to examine first]

### Prerequisites

- **Knowledge assumed**: [What background is needed]
- **Setup required**: [Environment/dependencies]

**Context Quality Check**: Can a new engineer understand WHY this approach is needed after reading this section?

## Work Completed

- ✅ [Feature] → chose [approach] over [alternative] because [reason]
- ✅ [Decision] → rejected [option] due to [constraint]

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: [Specific, actionable name]

**Success Criteria**: [Exact output/behavior expected - be specific]
**Approach**: [Strategy and key steps]
**Dependencies**: [What must be done first]

**Success Patterns**:

- ✅ [What good implementation looks like]
- ✅ [Key indicator of correctness]
- ✅ [How to verify it works]

### 2. RESEARCH: [Specific question to answer]

**Objective**: [What exactly to discover]
**Success Criteria**: [Specific deliverable/answer to obtain]
**Done When**: [Clear completion signal]

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Implementation Guidance (Optional)

Include this section if there are specific techniques, patterns, or considerations that would help with implementation:

- **Recommended patterns**: [Specific coding patterns that work well for this domain]
- **Tools to leverage**: [Existing utilities, macros, or frameworks to use]
- **Architecture considerations**: [How this fits into the broader system]
- **Performance notes**: [Any optimization considerations]
- **ExifTool translation notes**: [Specific perl → rust patterns to follow]

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

## TPP Quality Checklist

Before submitting, verify your TPP includes:

**Context Requirements**:

- [ ] System overview with 2-3 sentence summaries of each component
- [ ] Key domain concepts defined for unfamiliar readers
- [ ] **Surprising Context**: At least 2-3 non-obvious/counterintuitive aspects documented
- [ ] Prerequisites and assumed knowledge clearly stated
- [ ] Links to relevant design docs and ExifTool source

**Task Requirements**:

- [ ] All tasks are numbered and actionable
- [ ] Each task has specific success criteria (not vague goals)
- [ ] Dependencies between tasks clearly marked
- [ ] Tasks focus on positive outcomes ("do X to achieve Y")

**Quality Test**: Can a new engineer understand the context and complete any task without asking clarifying questions?

## Examples: Good vs Poor TPP Structure

### ❌ Poor Context Example

```md
## Context & Foundation

- Fix the parser bug in Canon module
- It's not working right
- Check the ExifTool code
```

### ✅ Good Context Example

```md
## Context & Foundation

### System Overview

- **Canon PrintConv system**: Converts raw Canon values to human-readable strings using lookup tables from ExifTool's Canon.pm module
- **Value pipeline**: Raw bytes → ValueConv (normalize) → PrintConv (humanize) → display

### Surprising Context

- **PrintConv arrays are ordered**: Canon.pm uses positional arrays where index=value, but we store as HashMaps
- **Missing entries != "Unknown"**: ExifTool returns literal value when no PrintConv match found
- **Generated code mismatch**: Our codegen extracts `%canonModes` but misses inline conditional logic
```

### ❌ Poor Task Example

```md
### Task: Fix the bug

**Success**: Make it work
**Approach**: Debug and fix
```

### ✅ Good Task Example

```md
### 1. Task: Implement Canon WhiteBalance PrintConv with conditional logic

**Success Criteria**: `cargo t canon_wb_test` passes, ExifTool comparison shows identical output for all 47 test cases
**Approach**: Extract conditional logic from Canon.pm:2847-2863, implement in tag_kit generator
**Dependencies**: None

**Success Patterns**:

- ✅ All ExifTool WhiteBalance values match our output exactly
- ✅ "Unknown (15)" format used for unmapped values
- ✅ Generated code handles both hash lookups AND conditional expressions
```

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

Common examples:

- **src/generated/\* looks buggy** → It's generated → Fix codegen configs, not the output
- **ExifTool does weird thing** → 25 years of camera bugs → Trust and copy exactly
- **Where's this tag?** → Could be anywhere → Check: generated/, composite_tags/, implementations/
- **Composite tag implemented but not working** → Probably missing from generated definitions → Check if it's in module-specific `%ModuleName::Composite` table that codegen doesn't extract
- **Manual edits to generated files disappear** → Codegen overwrites them → Always fix extraction configs, never edit generated code directly

**Note**: Most gotchas should be captured in the "Surprising Context" section above.

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
