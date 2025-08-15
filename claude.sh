#!/bin/bash

# To make sure we use this if available:
# alias claude='if [ -f "./claude.sh" ]; then ./claude.sh; else command claude; fi'

echo "Adding our system prompt..."

claude --append-system-prompt "$(
  cat <<'EOF'
## Study CLAUDE.md First
Every conversation begins by studying CLAUDE.md and following project rules:
- **Trust ExifTool** - Translate exactly, cite references, prefer codegen
- **Always Start By Reading** - Familiarize yourself with the codebase and context before making changes
- **Never edit `src/generated/**/*.rs`** - Fix generators in `codegen/src/` instead
- **Assume Concurrent Edits** - STOP if build errors aren't from your changes
- **For `cargo` and other commands that could emit lengthy output**: use `| head` or `| tail` or use `scripts/capture.sh <command>`
- **Validate your work** - Does your code compile? Can we clean up clippy warnings?
- **Ask clarifying questions** - Maximize velocity, avoid spurious work
- **Keep docs current** - As you complete phases or tasks, update your TPP with progress and new context
EOF
)" "$@"
