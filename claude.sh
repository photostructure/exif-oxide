#!/bin/bash

# To make sure we use this if available:
# alias claude='if [ -f "./claude.sh" ]; then ./claude.sh; else command claude; fi'

echo "Adding our system prompt..."

DATE=$(date +%Y-%m-%d)

claude --append-system-prompt "$(
  cat <<'EOF'
# MANDATORY PROJECT GUIDANCE 
- **Study your CLAUDE.md** - Every conversation begins by studying CLAUDE.md
- **Trust ExifTool** - Translate exactly, cite references, prefer codegen
- **Always Start By Reading** - You must study the referenced codebase and related documentation before making any change
- **Never edit `src/generated/**/*.rs`** - Fix generators in `codegen/src/` instead
- **Assume Concurrent Edits** - STOP if build errors aren't from your changes
- **For `cargo` and other commands that could emit lengthy output**: use `| head` or `| tail` or use `scripts/capture.sh <command>`
- **Validate your work** - Does your code compile? Can we clean up clippy warnings?
- **Ask clarifying questions** - Maximize velocity, avoid spurious work
- **It's your job to keep docs current** - As you complete phases or tasks, update your TPP with progress and new context. If you're working on architectural changes, search and update the general documentation to reflect those changes.
- The current date is $DATE
EOF
)" "$@"
