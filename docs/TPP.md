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
- **DO NOT DIRECTLY EDIT ANYTHING IN `src/generated/**/*.rs`** (Read [CODEGEN.md](CODEGEN.md) -- fix the generator or strategy in codegen/src instead!)
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

## TDD Foundation Requirement

### Task 0: Integration Test (conditional)

**Required for**:
- Feature development (new capabilities, functionality)
- Bug fixes (behavior corrections)
- System behavior changes (different outputs/processing)

**Optional/Skip for**:
- **Pure research**: Analyzing ExifTool algorithms, studying manufacturer formats
- **Documentation**: Writing guides, updating docs, creating reference materials  
- **Architecture/Design**: Planning module structure, designing interfaces
- **Pure refactoring**: Code reorganization with identical behavior (existing tests should suffice)
- **Infrastructure/Tooling**: CI setup, build improvements, development tools

### When Required: Write Failing Integration Test

**Purpose**: Ensure the TPP solves a real, measurable problem with verifiable success criteria.

**Success Criteria**:
- [ ] **Test exists**: `tests/integration_p[XX]_[goal_description].rs:test_function_name`
- [ ] **Test fails**: `cargo t test_name` fails with specific error demonstrating the problem
- [ ] **Integration focus**: Test validates end-to-end behavior change, not just unit functionality
- [ ] **TPP reference**: Test includes comment `// P[XX]: [Goal] - see docs/todo/P[XX]-description.md`
- [ ] **Measurable outcome**: Test clearly shows what "success" looks like when implementation completes

**Requirements**:
- Must test the overall goal described in Project Overview
- Should fail for the exact reason this TPP was created  
- Must demonstrate the problem is solved when all tasks complete
- Include error message linking back to this TPP: `"// Fails until P[XX] complete - requires [specific_capability]"`

**Quality Check**: Can you run the test, see it fail, and understand exactly what needs to be implemented to make it pass?

### When Skipping: Define Success Criteria

**If Task 0 doesn't apply**, clearly state why and define measurable success criteria:

**Example for Research TPP**:
```md
**Task 0**: Not applicable - pure research with no behavior changes
**Success Criteria**: Research document `docs/research/gps-analysis.md` exists with function signatures and implementation recommendations
```

**Example for Refactoring TPP**:
```md  
**Task 0**: Not applicable - refactoring with identical behavior
**Success Criteria**: All existing tests continue passing, module structure improved, no functionality changes
```

---

## Remaining Tasks

**REQUIRED**: Each task must have a unique alphabetic ID (A, B, C, etc.), be actionable, and include success criteria with specific proof requirements.

**Task Naming Convention**: Use `### Task A:`, `### Task B:`, etc. for unique identification and easy cross-referencing.

### Task A: [Specific, actionable name]

**Success Criteria**:
- [ ] **Implementation**: [Technical detail] → `src/path/file.rs:123-145` implements feature
- [ ] **Integration**: [How wired into production] → `src/main.rs:67` calls new function  
- [ ] **Task 0 passes**: `cargo t test_integration_p[XX]_[goal]` now succeeds (if Task 0 exists)
- [ ] **Unit tests**: `cargo t test_specific_feature` or `tests/unit_test.rs:test_name`
- [ ] **Manual validation**: `cargo run -- test_case` produces expected output change
- [ ] **Cleanup**: [Obsolete code removed] → commit `abc123f` or `grep -r "old_pattern" src/` returns empty
- [ ] **Documentation**: [Updated docs] → `docs/file.md:section` or N/A if none needed

**Implementation Details**: [Strategy and key technical steps]
**Integration Strategy**: [How to wire into production execution paths]  
**Validation Plan**: [Commands/tests that prove end-to-end functionality]
**Dependencies**: [What must be done first - reference other tasks by ID like "Task B complete"]

**Success Patterns**:
- ✅ [What good implementation looks like with specific proof]
- ✅ [Key indicator of correctness with measurement command]
- ✅ [How to verify integration works end-to-end]

### Task B: RESEARCH - [Specific question to answer]

**Objective**: [What exactly to discover]
**Success Criteria**: [Specific deliverable/answer to obtain]
**Done When**: [Clear completion signal]

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Task Completion Standards

**RULE**: No checkbox can be marked complete without specific proof.

### Required Evidence Types

- **Code references**: `file.rs:line_range` where implementation exists
- **Passing commands**: `cargo t test_name` or `make command` that succeeds  
- **Integration proof**: `grep -r "new_function" src/` shows production usage
- **Removal evidence**: Commit link or `grep` returning empty for removed code
- **Output changes**: Before/after examples showing behavior differences

### ❌ Common Incomplete Patterns

**Implementation without Integration**:
- "Module implemented but `main.rs` unchanged" → Missing integration proof
- "Feature works when called directly but no production usage" → Not wired into system
- "`grep -r new_feature src/` only shows test files" → No production consumption

**Testing without Validation**:
- "`cargo t` passes but new test was commented out" → Invalid test proof
- "Unit tests pass but integration test still fails" → Incomplete implementation
- "Test exists but doesn't validate end-to-end behavior" → Wrong test scope

**Cleanup Avoidance**:
- "Generated code updated but old workaround still active" → Incomplete cleanup
- "New feature works but legacy code path unchanged" → No obsolete code removal
- "Documentation says 'TODO: update this section'" → Invalid documentation proof

### ✅ Valid Completion Examples

- [ ] **Integration**: Parser uses new extraction method → `src/parser.rs:234` calls `extract_gps_coords()`
- [ ] **Testing**: Regression prevented → `cargo t test_gps_coordinate_parsing` passes  
- [ ] **Cleanup**: Dead code removed → `git show abc123f` removed `legacy_gps_parser()`
- [ ] **Documentation**: Guide updated → `docs/GPS-PARSING.md:45-67` documents new behavior
- [ ] **Validation**: Behavior changed → `cargo run image.jpg` now shows decimal GPS coordinates

**Accountability Principle**: Every checkbox creates a verifiable claim that another engineer can independently validate.

## Implementation Guidance (Optional)

Include this section if there are specific techniques, patterns, or considerations that would help with implementation:

- **Recommended patterns**: [Specific coding patterns that work well for this domain]
- **Tools to leverage**: [Existing utilities, macros, or frameworks to use]
- **Architecture considerations**: [How this fits into the broader system]
- **Performance notes**: [Any optimization considerations]
- **ExifTool translation notes**: [Specific perl → rust patterns to follow]

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

### Mandatory Integration Proof

Every feature must include specific evidence of integration:
- [ ] **Activation**: Feature is enabled/used by default → `src/main.rs:line` shows automatic usage
- [ ] **Consumption**: Existing code paths actively use new capability → `grep -r "new_function" src/` shows production calls
- [ ] **Measurement**: Can prove feature works via output changes → `cargo run test_case` shows different behavior
- [ ] **Cleanup**: Old approach deprecated/removed → commit link or `grep -r "old_function" src/` returns empty

### Integration Verification Commands

**Production Usage Proof**:
- `grep -r "new_feature" src/` → Should show non-test usage in main execution paths
- `git log --oneline -5` → Should show commits that wire new functionality into existing flows
- `cargo run representative_test_case` → Should demonstrate behavior change from previous baseline

**Integration vs Implementation Test**:
- ❌ **Implementation only**: "Feature works when I call `new_function()` directly"
- ✅ **Integrated**: "Feature works when I run normal workflow - `cargo run image.jpg` uses new logic automatically"

**Red Flag Check**: If a task seems like "build a tool/module but don't wire it anywhere," ask for clarity. We're not writing tools to sit on a shelf - everything must get us closer to "ExifTool in Rust for PhotoStructure."

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - something works differently/better than before
- ✅ **Default usage** - new capability is used automatically, not opt-in  
- ✅ **Old path removed** - previous workarounds/hacks are eliminated
- ❌ Code exists but isn't used *(example: "parser implemented but codegen still uses old logic")*
- ❌ Feature works "if you call it directly" *(example: "new API exists but nothing calls it")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

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

**Verification Requirements**:

- [ ] Task 0: TDD Foundation exists with failing integration test
- [ ] Every task includes specific proof requirements for each checkbox
- [ ] Integration proof explicitly required (not just implementation proof)
- [ ] Cleanup verification includes specific commands or commit references
- [ ] Manual validation includes exact commands that demonstrate success
- [ ] All checkboxes require code references, passing commands, or verifiable evidence

**Quality Test**: Can a new engineer understand the context, complete any task, and verify completion without asking clarifying questions?

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
### Task A: Implement Canon WhiteBalance PrintConv with conditional logic

**Success Criteria**:
- [ ] **Implementation**: Conditional logic extracted → `src/generated/Canon_pm/white_balance.rs:23-67`
- [ ] **Integration**: PrintConv generator uses logic → `codegen/src/tag_kit.rs:145` calls `generate_conditional_printconv()`
- [ ] **Task 0 passes**: `cargo t test_integration_p10_canon_wb` now succeeds
- [ ] **Unit tests**: `cargo t test_canon_white_balance_printconv` passes
- [ ] **Manual validation**: `cargo run -- canon_image.jpg` shows "Daylight" not "1"
- [ ] **Cleanup**: Old hardcoded lookup removed → `git show abc123f` removed static WB array
- [ ] **Documentation**: N/A

**Implementation Details**: Extract conditional logic from Canon.pm:2847-2863, implement in tag_kit generator
**Integration Strategy**: Wire into PrintConv pipeline for all Canon WB tags
**Validation Plan**: Test with 47 ExifTool comparison cases
**Dependencies**: None

**Success Patterns**:
- ✅ All ExifTool WhiteBalance values match our output exactly
- ✅ "Unknown (15)" format used for unmapped values  
- ✅ Generated code handles both hash lookups AND conditional expressions
```

## Examples: Task Completion Patterns

### ❌ Poor Completion Examples

**Implementation without Integration**:
```md
- [x] **Implementation**: GPS parser written → `src/gps.rs:45-120` implements parsing
- [x] **Integration**: Added to main system → "It compiles when imported"
- [x] **Testing**: Unit tests pass → `cargo t test_gps_unit`
```
*Problem*: No proof that main system actually uses GPS parser, no integration test, vague integration claim.

**Testing without Validation**:
```md
- [x] **Testing**: All tests pass → `cargo t` succeeds
- [x] **Manual validation**: Tested manually → "I ran it and it worked"
```
*Problem*: Could be passing because test was commented out, no specific validation command.

**Cleanup Avoidance**:
```md
- [x] **Cleanup**: Old code removed → "Not needed anymore"
- [x] **Documentation**: Updated → "Fixed the docs"
```
*Problem*: No evidence of actual removal or specific documentation changes.

### ✅ Good Completion Examples

**Complete Implementation with Integration (Task C example)**:
```md
- [x] **Implementation**: GPS coordinate parser → `src/gps.rs:45-120` implements `parse_coordinates()`
- [x] **Integration**: Main parser uses GPS logic → `src/parser.rs:234` calls `parse_coordinates()` for GPS tags
- [x] **Task 0 passes**: `cargo t test_integration_p15_gps` now succeeds
- [x] **Unit tests**: `cargo t test_gps_coordinate_parsing` passes
- [x] **Manual validation**: `cargo run -- test-images/gps/sample.jpg` shows decimal coordinates
- [x] **Cleanup**: Removed string fallback → `git show abc123f` deleted `gps_string_fallback()`
- [x] **Documentation**: Updated GPS guide → `docs/GPS-PARSING.md:23-45` documents coordinate format
```
*Why good*: Every checkbox has specific, verifiable evidence that another engineer can independently check.

**Proper Research Documentation (Task A example)**:
```md
- [x] **Research complete**: Canon lens database analyzed → `docs/research/canon-lens-analysis.md` with 47 lens entries
- [x] **Implementation plan**: Strategy documented → Same file, lines 89-102 outline extraction approach  
- [x] **ExifTool verification**: Confirmed behavior → `./exiftool -LensModel test-images/canon/*.jpg` output matches analysis
```
*Why good*: Research creates concrete deliverables that inform later tasks (Task B, Task C, etc.).

**Accountability Principle**: Each checkbox creates a verifiable claim. If you can't provide specific evidence, the task isn't complete.

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
