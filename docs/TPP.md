# Technical Project Plan

Please create or update a Technical Project Plan that will allow a new engineer on our team to successfully continue your work.

These are also referred to as "technical design", "milestone", or "handoff" documents.

## Goals

- An excellent technical project plan provides the context necessary for a new engineer to be successful at the tasks described.
- Each section described below should only be included if you think it will help the engineer succeed. If the section doesn't apply, skip it.
- If there is content that you wanted to share with the implementation team and it doesn't fit into any of these sections, add a new section with an appropriate header.
- Be respectful of the next engineer's time: keep each section actionable and focused. Link to relevant external docs and source. Aim for bullet points and terse, clear sentences rather than lengthy prose. Avoid duplication between sections unless indicated for emphasis.
- Aim for less than 500 lines total -- if it takes more than that, consider splitting the task into separate TPPs.
- Communication is hard! Write the TPP for a new engineer to this project -- try to ensure they understand _why_ and _what_ we want to address in the TPP--don't just give a laundry list of tasks to do without context. Prior engineers have done surprisingly wrong work because they didn't sufficiently understand the context of the issue at hand.

## Comm

---

## Document structure

### Required section: How to TPP

Always include these sections, verbatim, after the project overview:

```md
## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md).

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!
```

### Project Overview

- **Goal**: [One or two sentences describing what success looks like]
- **Problem**: [What's broken and why it needs fixing]
- **Critical Constraints**: [Non-negotiable requirements - architecture, performance, compatibility]

Example:

- **Goal**: Fix PrintConv pipeline to show human-readable values instead of raw arrays
- **Problem**: Tags display as `[39, 10]` instead of `3.9` due to broken conversion pipeline
- **Critical Constraints**:
  - ⚡ Zero runtime overhead (all lookups at compile-time)
  - 🔧 No circular dependencies between generated and manual code
  - 📐 Must maintain compatibility with existing tag structures

### Background & Context

- Why this work is needed
- Links to related design docs

### Technical Foundation

- Key codebases
- Documentation
- APIs
- Systems to familiarize with

### Work Completed

- What has already been implemented
- Decisions made and rationale
- Issues resolved

### Remaining Tasks

For each high-confidence task, provide:

1. **Task description** with clear acceptance criteria
2. **Expected output** - concrete example of what correct looks like
3. **Common mistakes** - what incorrect implementations often do
4. **Implementation notes** - high-level approach, not step-by-step

Example format:

````md
#### Generate direct function calls from registry

**Acceptance Criteria**: Registry maps Perl expressions to Rust functions at codegen time

**✅ Correct Output:**

```rust
// In generated apply_print_conv function
match tag_id {
    6 => crate::implementations::print_conv::print_fraction(value),
    7 => crate::implementations::print_conv::fnumber_print_conv(value),
}
```

**❌ Common Mistake:**

```rust
// Runtime string matching - THIS IS WRONG
match expression {
    "PrintFraction($val)" => call_some_function(),
    _ => fallback(),
}
```

**Implementation**: Lookup expressions in conv_registry.rs during codegen...
````

For research tasks:

- Clearly mark as "RESEARCH NEEDED"
- List specific questions to answer
- Provide success criteria for research completion

### Prerequisites

- Any additions or changes needed before starting work. Ideally, point to an existing TPP.

### Testing Strategy

- Unit tests
- Integration tests
- Manual testing steps

### Success Criteria & Quality Gates

- How to know when work is complete
- Definition of done including required reviews

### Gotchas & Tribal Knowledge

- Known edge cases
- Technical debt
- Decision rationale
- Other insights to avoid pitfalls

---

## File location

Technical Project Plans live in `${projectRoot}/docs/todo/${PX}-${short kebob-cased descriptive name}.md`

Where `PXX` follows this 2-digit priority naming convention:

- `P00-P09` - Critical blockers that prevent other work
- `P10-P19` - Maximum required tag impact (JPEG + Video ecosystem, binary extraction)
  - Includes: EXIF, MakerNotes, Composite, XMP, HEIC/HEIF, Video metadata, Binary extraction
- `P20-P29` - Technical debt and efficiency improvements
- `P30-P39` - Architecture improvements
- `P40-P49` - Video format support (if not required tag related)
- `P50-P59` - RAW format support (low required tag impact but enables binary extraction)
- `P60+` - Long-term/speculative work (write support, advanced features)

Add letter suffixes (a, b, c) only for strong prerequisites:

- `P10a` - Must be done before P10b
- `P10b` - Depends on P10a
- `P13` - No dependencies within P10-P19 range

Examples:

- `P00-fix-olympus-compilation.md` (critical blocker)
- `P10a-exif-required-tags.md` (foundation for MakerNotes)
- `P10b-subdirectory-coverage-expansion.md` (depends on EXIF foundation)
- `P13-canon-required-tags.md` (manufacturer-specific, no dependencies)
- `P50-MILESTONE-17-RAW-Format-Support.md` (low required tag impact)

When moving completed work to `docs/done/`, add the completion date:

- `docs/done/YYYYMMDD-P10a-exif-required-tags.md`
