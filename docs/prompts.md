# Make a new TPP

Excellent research. Please write a technical project plan, using our style guide, @docs/TPP.md, to docs/todo/... so we can hand this work off to another engineer. As the style guide states, include a description of the issue being addressed, code and docs they should study, relevant research you conducted, issues you've already addressed, success criteria, and any other tribal knowledge that you can pass down to help them to help them succeed. Ultrathink.

# Update TPP

Well done. Please update your technical project plan (TPP) with your progress. Use our style guide, @docs/TPP.md. Please add any novel, additional context that could help the next engineer complete this work, or, if all the work is complete, so an Engineer of the Future will have more context to grok what was considered, designed, and implemented for this TPP, and include attempted (but unsuccessful) strategies so the next engineer can make _new_ mistakes.

After updating the TPP, please continue work on the remaining incomplete tasks.

# Handoff (before compaction)

Well done. We need to hand this work to another engineer now. Please update your technical project plan (TPP) using our style guide, @docs/TPP.md. Your goal is to ensure the next engineer succeeds in picking up where you left off. Include: issue description, relevant code/docs to study, new task breakdowns, completed tasks, success criteria, and context needed to complete remaining work. DO NOT overstate task completion status! Correct any inaccuracies you find. Include refactoring ideas as future work. If you don't remember which TPP you're working on, **please ask**.

# When the initial plan is hand-wavy

That's really great analysis! Since this touches core architecture, it needs thorough research. For anything unclear or uncertain, verify against source code, docs, generated code, and ExifTool source. Ultrathink and re-analyze. Consider alternative approaches, weighing pros and cons to improve our plan. Don't expand scope without asking if it's relevant. Review TPPs in docs/todo and docs/done for context and coordination opportunities.

# Refining an unsaved TPP

That sounds great. This is critical infrastructure for this project, so let's do another iteration of research, analysis, and planning. Re-analyze and re-plan the TPP using the @docs/TPP.md style guide. Read and study **all** referenced documentation and source code before making any changes. Anything that we can clarify and discover at this point, especially if it is currently hand-wavy or nebulous, will save us time and effort in the future.  As @CLAUDE.md states, ask clarifying questions for anything odd, confusing, nebulous, or to help decide between alternative strategies. Ultrathink.

# Ultraplan

1. You have been asked to design a plan. You are going to **ultraplan**! 
2. This is critical infrastructure for this project, so let's do another iteration of research, analysis, and planning. 
3. Re-analyze and re-plan using our style guide, @docs/TPP.md to drive your Technical Project Plan, which we will hand over to another team to implement.
4. As you design your plan, carefully consider the requirements, constraints, and existing solutions in this project. If there are any clarifying questions for the user, or there are any assumptive statements that you are making, state them and ask the user for clarification.
5. Once you have your initial plan, generate a series of thorough critiques of your plan. These critiques should be applied to ensure the project balances simplicity with good software engineering practices, is maintainable, testable, DRY, scalable, and meets user requirements. Ensure any assumptions are validated by examining all elements of all code paths involved, and if any development is novel, validate APIs and libraries with web searches. For novel libraries and APIs, write HOWTO guides and documentation to $PROJECT_ROOT/docs
6. While performing critiques, scrutinize for nebulous or "hand-wavy" aspects. Do not assume how things work -- study the code, and if indicated, web search to see if someone has solved a similar problem. Ideally, perform research necessary to reduce uncertainty, but minimally, note these aspects as risks in your plan.
7. Brainstorm alternatives to your plan based on these critiques. Overarching goals should be to simplify the plan, reduce complexity, reduce the risk of the plan failing, and to improve overall code quality, maintainability, and reduce technical debt. See @docs/SIMPLE-DESIGN.md
8. Select the most promising alternative and develop it into a full plan.
9. Critique these alternatives in the same way as the original plan.
10. Assemble the best features from the original plan and the alternatives into a final plan.
11. Repeat steps 2-8 until you have a final plan that is robust, efficient, and meets all requirements. You must repeat this process at least thrice. You should ask the user for feedback and ask clarifying questions as needed for each iteration.

# Refining an existing TPP

✏️ is a technical project plan: but it needs more due diligence research, analysis, and planning. Re-analyze and re-plan the TPP using the @docs/TPP.md style guide. Read and study **all** referenced documentation and source code before making any changes. This is critical infrastructure for this project, so we have a large time budget for research, planning, analysis, and validation for this work. As @CLAUDE.md states, ask clarifying questions for anything odd, confusing, nebulous, or to help decide between alternative strategies. Ultrathink.

# Work on a TPP 

✏️ is a technical project plan: we're going to work on the remaining incomplete tasks. This represents critical work for our project, and requires comprehensive prerequisite research to be done by you before you start work. Read every referenced source and doc, and carefully validate the current situation, problem, and that the currently planned solution is still the best way forward. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work. @CLAUDE.md @docs/CODEGEN.md @docs/ARCHITECTURE.md and @docs/TRUST-EXIFTOOL.md provide invaluable project context and guidelines. Ultrathink.

# Validating a TPP

✏️ is a technical project plan: we're going to carefully validate every task to see if it is actually complete. Prior engineers may have incorrectly stated that tasks were complete when they are not, so verify everything carefully. This is critical work for our project, and requires extensive and exhaustive prerequisite research. Read every referenced source and doc. Run relevant tooling and tests and study existing source to validate current code state. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work. @CLAUDE.md and @docs/TRUST-EXIFTOOL.md provide invaluable project context and guidelines. If all tasks are complete, revise the TPP with updated status and move it to `docs/done/$(TZ=America/Los_Angeles date +%Y%m%d)-$(basename $TPP_FILE_NAME)`. Ultrathink.

# When the robots need a reminder

Remember: do not invent heuristics! @docs/TRUST-EXIFTOOL.md !

**Use rg|sd instead of the default MultiEdit tool**: This is extremely quick and efficient: `rg -l 'old-pattern' src/ | xargs sd 'old-pattern' 'new-pattern'`

Remember: do not edit, add, or delete files in @src/generated/** -- the `codegen` system completely overwrites all files in that directory. If you need any edits made, fix the generator.

# /compact

/compact and include the following context for the next engineer so they can successfully complete your unfinished tasks:

1. **TPP status** - Iff a TPP is being worked on, require the next engineer to read `docs/todo/PXX-*.md`, and include supplemental task progress, and any pending updates that should be made to the TPP 
2. **Critical files** - Must-read paths with rationale
3. **Progress state** - What was tried, current status, remaining work
4. **Failed attempts** - What failed and why (prevent repetition)
5. **Key insights** - Important codebase/ExifTool discoveries
6. **Next steps** - Concrete actions with locations, TPP task references

# Fix a test or bug

Fixing this bug will require in-depth understanding of both how our code and ExifTool. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work. Read **all** referenced documentation, and all relevant source code before planning your solution. Ultrathink.

# Let me rethink this...

Whenever you say "let me rethink this approach" it makes me think that either 1: the architecture is convoluted 2: the code isn't consistently following a coherent architecture or 3: the architecture wasn't really designed to gracefully/intuitively deal with this use case. Let's take a couple steps back here. Can you describe to me what the problem that we're trying to solve is here? What caused you to doubt your prior plan? Are you sure you have a deep understanding of both the problem we're solving and the different ways our current codebase could address it? Ultrathink.


# Validate git diffs

Your task is to review the staged changes in git and provide recommendations for
improvement, ensuring code quality and completeness. We have a large time budget for research, planning, analysis, and validation for this work. Take it step by step. Show your work.

Be concise with your review comments, but don't omit important details. Use bullet points for clarity.
Take it step by step, and for every file changed:

1. Summarize its modifications, excluding import changes.
2. Consider opportunities for API improvement (function, class, and variable names).
3. Validate test coverage is sufficient. See .github/instructions/tests.instructions.md for details.
4. Look for code redundancy, unnecessary complexity, and dead code. Pay special attention to:
   - Duplicate blocks of code with minor or no differences
   - If/else branches that do essentially the same thing
   - Special case handling that's identical to the default case
   - Functions or parameters that aren't used
5. Ask for clarification if any part of the diff is odd, inappropriate, confusing, or boring.

For each file, carefully analyze:

- Function implementations line by line to catch inconsistencies
- Conditional logic that might be simplified or contains redundancies
- Parameter handling and edge cases
- Potential performance issues or unintended side effects

If all changes look reasonable, compose a concise git commit message following Conventional Commit specs. For the "scope" of the Conventional Commit message, use the name of the file with the most important changes. Details should be short bullet points.
