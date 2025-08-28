# Technical Project Plan (TPP) Template

**IMPORTANT**: This is a TEMPLATE. Copy the structure below but DELETE these instructions and write YOUR OWN content. Do not include the template's purpose statement or boilerplate text in your actual TPP.

**Purpose of TPPs**: Prevent architectural vandalism by ensuring engineers have sufficient context to make coherent solutions that integrate with existing systems.

---

## START YOUR TPP HERE - DELETE EVERYTHING ABOVE THIS LINE

# Technical Project Plan: [Your Specific Project Name]

## 🟦 TPP CREATION PHASE (Do This First)

**WHO**: TPP Author (you right now)  
**GOAL**: Gather sufficient context for implementer success  
**OUTPUT**: Research analysis written FOR the implementer  
**WORKFLOW**: Complete all research sections below, then move to Execution Phase

## 🟨 EXECUTION PHASE (Do This Later)

**WHO**: Implementer (could be you or someone else)  
**GOAL**: Complete tasks using research context  
**OUTPUT**: Working code with checkbox evidence  
**WORKFLOW**: Use research context to complete tasks with verifiable proof

## Goal Definition (MANDATORY FIRST SECTION)

**Template**: Complete this section first. Maximum 5 sentences covering what, why, constraints, and validation.

- **What Success Looks Like**: [1 sentence - specific, measurable outcome]
- **Core Problem**: [1 sentence - what's broken and why it matters]
- **Key Constraints**: [1 sentence - non-negotiable requirements]
- **ExifTool Alignment**: [1 sentence - how this relates to Trust ExifTool principle]
- **Success Validation**: [1 sentence - how you'll prove it works]

**Example**:

- **What Success Looks Like**: Canon lens metadata displays "Canon EF 50mm f/1.8" instead of raw numeric values
- **Core Problem**: PrintConv pipeline doesn't handle Canon's complex lens ID lookup tables
- **Key Constraints**: Must use generated lookup tables, zero manual transcription allowed
- **ExifTool Alignment**: Implement identical logic to Canon.pm:2847-2863 lens ID resolution
- **Success Validation**: All Canon test images show human-readable lens names matching ExifTool exactly

## Mandatory Context Research Phase

**CRITICAL**: Complete ALL sections before defining tasks. This prevents architectural vandalism and shallow solutions.

### Step 1: Project Foundation Review

**MANDATORY READING** (document your understanding - future implementers must read this):

**⚠️ CRITICAL**: Write this analysis for the IMPLEMENTER, not yourself. Use **IMPERATIVE LANGUAGE** ("You must...", "Never do...", "This will break if...") so implementers understand constraints.

**CLAUDE.md Analysis**: 
[Write 2-3 sentences explaining what the implementer must never do and why. Include specific file:line references for critical patterns. Explain what causes instant PR rejection.]

**TRUST-EXIFTOOL.md Analysis**:
[Document the exact ExifTool behavior the implementer must replicate. Include .pm file references and line numbers. Explain why deviation breaks compatibility.]

**SIMPLE-DESIGN.md Analysis**:
[Explain which of the 4 rules apply most to this work. Show how this project directly applies them with concrete examples.]

**TDD.md Analysis**:
[Specify what tests are needed to validate success. Document the testing framework and workflow the implementer must follow.]

**ARCHITECTURE.md Analysis**:
[Identify existing systems this integrates with. Explain what will break if integration points change. Provide specific file:line references.]

### Step 2: Precedent Analysis

**CRITICAL ARCHITECTURAL CONTEXT** (document constraints for implementers):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Explain what will BREAK if they don't follow existing patterns.

**Existing Patterns Analysis**:
[Document what similar work has been done. Specify exact patterns the implementer must follow with file:line citations. Explain consequences of deviating from these patterns.]

**Dependencies Analysis**:
[List what systems this change will affect. Trace actual data flow paths. Explain what breaks if these dependencies change.]

**Integration Points Analysis**: 
[Specify where this must connect to existing code with file:line references. Explain what happens if these integration points break.]

**Generated Code Analysis**:
[Document what lookup tables or codegen outputs are available. Explain why manual transcription is banned with specific examples of past disasters.]

### Step 3: ExifTool Research

**EXIFTOOL BEHAVIOR REQUIREMENTS** (document what implementers must replicate exactly):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Document the EXACT behavior they must replicate and WHY.

**Source Analysis**:
[Document what ExifTool does in this area with specific .pm files and line numbers. Explain the algorithms that must be replicated exactly.]

**Critical Edge Cases**:
[Document camera-specific quirks with concrete examples. Explain what will break if these aren't handled. Include manufacturer-specific behaviors.]

**Test Cases**:
[List sample files that demonstrate the behavior. Specify expressions that must work identically to ExifTool.]

**Output Format Requirements**:
[Specify what output must match ExifTool exactly. Document where deviation is allowed vs forbidden.]

### Step 4: Risk Assessment

**FAILURE MODES** (document specific ways implementers can break things):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Document SPECIFIC failure modes with EXAMPLES.

**What Could Go Wrong**:
[List specific ways this could break existing functionality. Provide concrete examples of what breaks and why.]

**Emergency Recovery Plan**:
[Explain how the implementer can quickly revert/fix if this breaks. Document the fallback plan with specific commands.]

**Validation Strategy**:
[Specify how the implementer will prove this works correctly. List exact commands that must pass.]

**Integration Testing Requirements**:
[Document end-to-end scenarios that must pass. Explain what indicates success vs failure.]

**Quality Gate**: Can another engineer understand the CONTEXT and MOTIVATION behind this approach?

## 🔍 TPP HANDOFF VALIDATION

**COMPLETE THIS BEFORE MARKING TPP READY FOR IMPLEMENTATION**

Before moving to Execution Phase, verify this TPP provides sufficient context for successful handoff:

- [ ] **Context Sufficiency**: Another engineer can understand WHY this approach is needed (not just what to build)
- [ ] **Implementation Clarity**: Tasks are specific enough to execute without clarifying questions  
- [ ] **Constraint Documentation**: All "gotchas" and failure modes are documented with concrete examples
- [ ] **Success Measurement**: Clear commands provided to prove each task is complete
- [ ] **ExifTool Alignment**: Specific .pm file references and behavior requirements documented
- [ ] **Integration Requirements**: Clear proof requirements showing production usage (not just test code)

**HANDOFF TEST**: Could you hand this TPP to another engineer and have them successfully complete the work without asking you clarifying questions?

---

## TDD Integration Test (Task 0)

**Required for**: Feature development, bug fixes, behavior changes  
**Skip for**: Pure research, documentation, architecture planning, refactoring

### When Required: Failing Integration Test

**Purpose**: Prove the TPP solves a real, measurable problem.

- [ ] **Test exists**: `tests/integration_p[XX]_[goal].rs:test_name`
- [ ] **Test fails**: `cargo t test_name` fails demonstrating the exact problem
- [ ] **End-to-end focus**: Tests the complete user-facing behavior, not internals
- [ ] **Success criteria clear**: Test shows what "working" looks like

### When Skipping: Alternative Success Criteria

**Research Example**: Success = `docs/research/topic.md` with implementation recommendations  
**Refactoring Example**: Success = all existing tests pass, cleaner module structure

---

## Task Definition

**Template**: Each task follows outcome-focused format with minimal bullets.

### Task A: [Specific, actionable name]

**What works after this task**: [1 sentence describing the capability gained]

**Implementation approach**: [2-3 sentences on strategy and key technical steps]

**Validation commands**: 
- `cargo t test_name` - [what this proves]
- `grep -r "pattern" src/` - [what this shows for integration]
- `cargo run test_case` - [what behavior change this demonstrates]

**Dependencies**: [Other tasks that must complete first, or "None"]

**Completion checklist** (mark during execution):
- [ ] **Code implemented** → [file:line where functionality exists]
- [ ] **Tests passing** → [specific test command that succeeds]  
- [ ] **Production integration** → [proof of actual usage in main flows]
- [ ] **Cleanup complete** → [evidence old code removed]

### Task B: RESEARCH - [Specific investigation]

**What is understood after this task**: [What will be documented/analyzed that enables implementation]

**Research approach**: [2-3 sentences on investigation strategy and key areas to analyze]

**Validation commands**: 
- `[analysis command]` - [what this reveals about the domain]
- `[comparison command]` - [what this shows about ExifTool behavior]

**Dependencies**: [Other tasks that must complete first, or "None"]

**Completion checklist** (mark during execution):
- [ ] **Analysis documented** → [docs/research/file.md with findings]
- [ ] **ExifTool behavior mapped** → [specific .pm files and line numbers cited]
- [ ] **Implementation strategy** → [clear path forward for implementation tasks]
- [ ] **Test cases identified** → [sample files and edge cases documented]

## Validation Requirements

**RULE**: Every checkbox must have verifiable proof that another engineer can independently validate.

**WARNING**: Do not mark checkboxes complete based on "it works when I test it manually." Provide specific commands and evidence.

### Required Evidence

- **Commands that pass**: `cargo t test_name`, `make precommit`, etc.
- **Code locations**: `src/file.rs:line_range` where implementation exists
- **Integration proof**: `grep` commands showing production usage
- **Behavior changes**: Before/after examples or comparison tool output

### Anti-Vandalism Validation

**CRITICAL**: "Working code" ≠ "Complete task". Most engineering failures come from declaring victory too early.

**Integration Requirements**: Every feature must prove it connects to existing systems.

- ✅ **Production Usage**: `grep -r "new_function" src/` shows actual usage (not just tests)
- ✅ **Behavior Change**: `cargo run test_case` produces different output than before
- ✅ **Cleanup Complete**: `grep -r "old_pattern" src/` returns empty. Obsolete/deadwood code has been deleted.
- ❌ **Shelf-ware**: Code exists but nothing calls it in production workflows
- ❌ **Half-integrated**: Works when called directly but not in normal usage

**Common Over-Selling Patterns** (DO NOT mark tasks complete if any apply):
- "Implementation works" but no integration proof
- "Tests pass" but only unit tests, no end-to-end validation
- "Feature complete" but old workaround code still active
- "Ready for review" but `make precommit` fails
- "99% done" but missing cleanup/documentation/integration

**Definition of Complete**: System behavior changes + old path removed + new capability used automatically + all validation commands pass.

## Quick Reference

### TPP Quality Checklist

- [ ] **Goal defined first**: 5 sentences covering what, why, constraints, ExifTool alignment, validation
- [ ] **Context research complete**: All 4 mandatory steps documented with citations
- [ ] **Tasks outcome-focused**: "At end of task X works that didn't before"
- [ ] **Integration required**: Every feature proves production usage
- [ ] **ExifTool references**: Cite specific .pm files and line numbers
- [ ] **No over-selling**: Tasks marked complete only when ALL checkboxes have evidence
- [ ] **End-to-end validation**: `make precommit` passes before declaring victory

### Common Project Gotchas

- **Generated code looks wrong** → Fix `codegen/`, never edit `src/generated/`
- **ExifTool does weird thing** → Trust it completely, 25 years of camera bug fixes
- **Manual edits disappear** → Use codegen extraction, ban manual transcription
- **Feature works but unused** → Integration proof required, no "shelf-ware"
- **"It works on my machine"** → Provide specific commands others can run to verify
- **"Just needs cleanup"** → Cleanup is PART of the task, not a future TODO
- **"95% complete"** → Either complete with evidence or incomplete, no percentages

### Essential Commands

- `cargo t test_name` - Run specific test
- `grep -r "pattern" src/` - Find usage in codebase
- `make precommit` - Final validation before completion
- `cargo run --bin compare-with-exiftool image.jpg` - Verify ExifTool alignment

### File Naming

`docs/todo/PXX-description.md` → `docs/done/YYYYMMDD-PXX-description.md`

PXX = priority (P00 highest, P99 lowest). Use suffixes (a,b,c) only for dependencies.
