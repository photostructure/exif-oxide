# Technical Project Plan

Please create or update a Technical Project Plan that will allow a new engineer on our team to successfully continue your work.

These are also referred to as "technical design", "milestone", or "handoff" documents.

## File location

The doc should live in `${projectRoot}/docs/todo/${YYYYMMDD}-${short kebob-cased descriptive name}.md`

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
  
