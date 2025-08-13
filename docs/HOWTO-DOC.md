**# How to Write Good Documentation**

## Core Principles

### Start with Purpose

- Write a 1-2 sentence summary explaining what the document covers and who should read it
- Use descriptive, concise titles (e.g., "API Authentication Guide" not "auth_stuff_v2")
- State prerequisites upfront

### Ensure that you have comprehensive, in-depth understanding of the subject matter

- Your job is to write as a Subject Matter Expert.
- Engineers reading your docs need to trust that you had a full and complete understanding of the described material
- The research that you do for this document will have a compounding beneficial effect, saving engineers time and effort!
- We have a large time budget for producing this document -- be sure to fully read the source code, relevant upstream and downstream code, and understand both the mechanics _but also the context_ that the code works within.

### Respect Reader Time

- Keep documents under 300-500 lines; split longer content into focused sub-documents
- Use clear, precise language. Avoid unnecessary prose.
- Define jargon when necessary
- Front-load critical information

### Capture Tribal Knowledge

- Document "everyone knows" assumptions that aren't written anywhere
- Include common pitfalls and their solutions
- Explain _why_, not just _how_
- Record edge cases and workarounds

### Actionable Content

- If relevant, provide working, copy-pasteable examples
- Include expected outputs
- Add troubleshooting sections
- Specify version numbers

### Enable Discovery

- Link to source code, APIs, and related docs
- Use searchable keywords and descriptive link text
- Date your documentation and mark deprecated sections
- Include contact info for subject matter experts

## Quality Checklist

- [ ] A newcomer could follow without help
- [ ] All links work
- [ ] Code examples are tested
- [ ] Answers what, why, when, and how

Remember: Good documentation is a living artifact. Plan for updates from the start.
