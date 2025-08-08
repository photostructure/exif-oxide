#!/bin/bash

claude --append-system-prompt "$(cat << 'EOF'
## Study CLAUDE.md First

Every conversation begins by studying CLAUDE.md and following project rules:
- **Trust ExifTool** - Translate exactly, cite references, prefer codegen
- **Always Start By Reading** - Familiarize yourself with the codebase and context before making changes
- **Never edit `src/generated/**/*.rs`** - Fix generators in `codegen/src/` instead
- **Assume Concurrent Edits** - STOP if build errors aren't from your changes
- **Ask clarifying questions** - Maximize velocity, avoid spurious work
- **Keep docs current** - As you complete phases or tasks, update your TPP with progress and new context

## Compact Mode: Engineer Handoff

When compacting, your ONLY goal is next engineer success on incomplete tasks.

**Required elements:**
1. **TPP status** - Iff a TPP is being worked on, include current `docs/todo/PXX-*.md`, task progress, needed TPP updates
2. **Critical files** - Must-read paths with rationale
3. **Progress state** - What was tried, current status, remaining work
4. **Failed attempts** - What failed and why (prevent repetition)
5. **Key insights** - Important codebase/ExifTool discoveries
6. **Next steps** - Concrete actions with locations, TPP task references

Format as structured handoff enabling immediate progress continuation. Include 👍 in compact message to confirm compliance.

EOF
)" "$@"
