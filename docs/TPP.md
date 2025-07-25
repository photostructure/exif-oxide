# Technical Project Plan

Please create or update a Technical Project Plan that will allow a new engineer on our team to successfully continue your work.

These are also referred to as "technical design", "milestone", or "handoff" documents.

## File location

The doc should live in `${projectRoot}/docs/todo/${PX}-${short kebob-cased descriptive name}.md`

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

## 🚨 CRITICAL: Keep This Document Updated! 🚨

**This TPP is a living document that MUST be updated throughout your work:**

1. **During Research**: Add discoveries, context, and findings to relevant sections
2. **When Making Decisions**: Document rationale in "Work Completed" or "Gotchas"
3. **As You Progress**: Move items from "Remaining Tasks" to "Work Completed"
4. **When Blocked**: Add blockers to "Prerequisites" or "Gotchas"
5. **Upon Completion**: 
   - Update all sections to reflect final state
   - Move to `docs/done/YYYYMMDD-PX-description.md`
   - Remove any obsolete or incorrect information
   - Add a "Completion Summary" section if helpful

**Remember**: The next engineer depends on this document being accurate and current!

## Goals

- An excellent technical project plan provides the context necessary for a new engineer to be successful at the tasks described.
- Each section described below should only be included if you think it will help the engineer succeed. If the section doesn't apply, skip it.
- If there is content that you wanted to share with the implementation team and it doesn't fit into any of these sections, add a new section with an appropriate header.
- Be respectful of the next engineer's time: keep each section actionable and focused. Link to relevant external docs and source. Aim for bullet points or short paragraphs rather than lengthy prose. Avoid duplication between sections. Aim for less than 500 lines total -- if it takes more than that, consider splitting the task into separate TPPs.

## Document structure

### Project Overview

- High-level goal
- Problem statement

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

- Tasks that have very high confidence can be listed directly with implementation instructions
- All other tasks should be clearly denoted as requiring additional research, analysis, and implementation design work. They may include possible implementation sketches if that could help illuminate the task better.

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

**⚠️ Final Reminder**: Update this document as you work! Don't wait until the end to document your discoveries, decisions, and progress. Your future self and teammates will thank you.
