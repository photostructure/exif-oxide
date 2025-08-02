# Make a new TPP

Please output a technical project plan, using our style guide, @docs/TPP.md, to hand this work off to another engineer. As the style guide states, include a description of the issue being addressed, code and docs they should study, relevant research you conducted, issues you've already addressed, success criteria, and any other tribal knowledge that you can pass down to help them to help them succeed. Ultrathink.

# Update TPP

Well done. Please update your technical project plan (TPP) with your progress. Use our style guide, @docs/TPP.md. Please add any novel, additional context that could help the next engineer complete this work, or, if all the work is complete, so an Engineer of the Future will have more context to grok what was considered, designed, and implemented for this TPP.

# Handoff (before compaction)

We're stopping work now. Please update your technical project plan (TPP) using our style guide, @docs/TPP.md. Your goal is to ensure the next engineer succeeds in picking up where you left off. Include: issue description, relevant code/docs to study, completed tasks, success criteria, and context needed to complete remaining work. Correct any inaccuracies you find. Include refactoring ideas as future work. If you don't remember which TPP you're working on, **please ask**.

# When the initial plan is hand-wavy

That's really great analysis! Since this touches core architecture, it needs thorough research. For anything unclear or uncertain, verify against source code, docs, generated code, and ExifTool source. Ultrathink and re-analyze. Consider alternative approaches, weighing pros and cons to improve our plan. Don't expand scope without asking if it's relevant. Review TPPs in docs/todo and docs/done for context and coordination opportunities.

# Refining an existing TPP

Let's do more due diligence research, analysis, and planning for the work described in ✏️ . Re-analyze and re-plan the TPP using the @docs/TPP.md style guide. Read and study **all** referenced documentation and source code before making any changes. This is critical infrastructure for this project, so we have a large time budget for research, planning, analysis, and validation for this work. As @CLAUDE.md states, ask clarifying questions for anything odd, confusing, nebulous, or to help decide between alternative strategies. Ultrathink.

# Starting work on a TPP that needs validation

Let's work through the tasks in ✏️ -- this requires extensive preparation. Read every referenced source and doc, carefully validate and re-analyze the current situation, problem, and solution before starting. In this planning phase, run relevant tooling and tests to validate current code state. @CLAUDE.md and @docs/TRUST-EXIFTOOL.md provide invaluable project context and guidelines. Ultrathink.

# Starting work on a TPP that's solid

Let's work through the tasks in ✏️ -- this requires extensive preparation. Read every referenced source and doc, carefully validate the current situation, problem, and solution before starting. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work. @CLAUDE.md and @docs/TRUST-EXIFTOOL.md provide invaluable project context and guidelines. Ultrathink.

# Continuing work on a TPP

Let's continue the tasks in ✏️ -- this requires extensive preparation. Read every referenced source and doc, validate and re-analyze before starting. Prior engineers may have incorrectly stated that tasks were complete when they are not, or forgot to update as progress was made, so verify and validate the actual state by studying code and running tests. @CLAUDE.md and @docs/TRUST-EXIFTOOL.md provide invaluable project context and guidelines. Ultrathink.

# Validating and continuing work on a (probably complete) TPP

Validate the tasks listed in ✏️ -- this is critical infra, so we have a large time budget for research, planning, analysis, and validation for this work. Read **all** referenced documentation, and all relevant source code. Work step by step and show your work. Prior engineers may have incorrectly stated that tasks were complete when they are not, or forgot to update as progress was made, so verify everything carefully. Update and improve the TPP using this style guide: @docs/TPP.md. We want to complete the work in this TPP, so if, after your planning and research phase, there are incomplete tasks found, we want you to work on them. Ultrathink.

# Validating a completed TPP

Validate the tasks and their state of completion in ✏️ -- this is critical infra with a large time budget for research and validation. Read **all** referenced documentation, and all relevant source code. Work step by step and show your work. Prior engineers may have incorrectly stated that tasks were complete when they are not, so verify everything carefully. Update the TPP using @docs/TPP.md style guide. If all tasks are complete, update the TPP and move it to `docs/done/$YYYYMMDD-${tpp_basename}` (use current date in America/Los_Angeles timezone). Ultrathink.

# When the robots need a reminder

Remember: do not invent heuristics! @docs/TRUST-EXIFTOOL.md !

# Fix a test or bug

Fixing this bug will require in-depth understanding of both how our code and ExifTool ✏️. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work. Read **all** referenced documentation, and all relevant source code before planning your solution. Ultrathink.
