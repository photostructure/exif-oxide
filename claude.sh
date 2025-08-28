#!/bin/bash

# To make sure we use this if available:
# alias claude='if [ -f "./claude.sh" ]; then ./claude.sh; else command claude; fi'

echo "Adding our system prompt..."

DATE=$(date +%Y-%m-%d)

claude --append-system-prompt "$(
  cat <<'EOF'
# MANDATORY GUIDELINES
- **Study your CLAUDE.md** - Every conversation begins by studying CLAUDE.md
- **Trust ExifTool** - Translate exactly, cite references, prefer codegen
- **Always Start By Reading** - You must study the referenced codebase and related documentation before making any change. NEVER assume APIs or implementation details.
- **Never edit `src/generated/**/*.rs`** - Fix generators in `codegen/src/` instead
- **Assume Concurrent Edits** - STOP if build errors aren't from your changes
- **For `cargo` and other commands that could emit lengthy output**: use `| head` or `| tail` or use `scripts/capture.sh <command>`
- **Validate your work** - Does your code compile? Can we clean up clippy warnings? Do the related tests pass?
- **Don't use git checkout to undo changes** - Instead, re-apply your diff in reverse. You have to assume that the git tree was not clean when you made edits.
- **Ask questions** - If anything is nebulous or unclear, it is IMPERATIVE that you ask clarifying questions to maximize velocity and avoid spurious work.
- **It's YOUR JOB to keep docs current** - As you complete phases or tasks, update your TPP with your progress and new context. DO NOT say that non-validated tasks are "complete". If you're working on architectural changes, search and update the general documentation to reflect those changes.
- **Do not delete files without asking** - If you need to delete a file, please ask for permission first, and provide a justification for why it should be deleted.
- The current date is $DATE
EOF
)" "$@"
