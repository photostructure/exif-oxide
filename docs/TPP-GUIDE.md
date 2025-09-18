# Technical Project Plan (TPP) Guide

## What Makes a Great TPP

A great TPP is like having the original engineer sitting next to you, sharing:

- What problem we're solving and why it matters
- The approach that failed and cost 3 days of debugging
- The test file that reveals the edge case
- How to adapt when the architecture changes

**The golden rule**: Transfer expertise, not just instructions.

## Foundation: Required Reading

Before writing any TPP, YOU MUST READ AND INCORPORATE THE TEACHINGS OF THESE DOCUMENTS:

- **[SIMPLE-DESIGN.md](./SIMPLE-DESIGN.md)**: Kent Beck's Four Rules guide all design decisions
- **[TDD.md](./TDD.md)**: Bug fixes MUST start with a failing test that reproduces the issue. Refactoring efforts against well-tested modules may not need additional tests.
- **[ANTI-PATTERNS.md](./ANTI-PATTERNS.md)**: Many different ways that have wasted work. Let's make _different_ mistakes.

These documents ARE NOT OPTIONAL! They are pivotal to success.

## TPP Structure: Three Essential Parts

### Part 1: Define Success (5 minutes)

Write ONE clear sentence for each:

```markdown
Problem: Users see "153" instead of "Canon EF 50mm f/1.8"
Why it matters: Photographers can't identify which lens took which photo
Solution: Implement PrintConv to show human-readable lens names
Success test: `cargo run photo.jpg | grep "Canon EF 50mm"`
Key constraint: Must match ExifTool's Canon.pm:2847 logic exactly
```

This becomes your North Star - even if implementation details change, the user need remains constant.

**For bug fixes** (per [TDD.md](TDD.md)):

```markdown
Bug: GPS coordinates return None near equator
Reproducing test: `cargo t test_gps_equator_parsing` (currently fails)
Root cause: Integer underflow in latitude calculation
Fix approach: Match ExifTool's GPS.pm:234 handling of edge cases
```

### Part 2: Share Your Expertise (30 minutes)

This section should only attempt to avoid surprises -- don't document easy-to-follow code. It's fine to skip this section for pedestrian modules.

#### A. Find the Patterns

**If pertinent**, show what already works similarly:

```bash
# Find existing patterns to follow
rg "impl.*PrintConv" --type rust
rg "human.*readable|display.*format" src/

# Check if this is generated code (don't edit manually!)
ls src/generated/*.rs | xargs grep -l "lens"
```

Document what you find:

- "Copy pattern from `src/processors/nikon.rs:234` - handles similar lookup"
- "NEVER edit `src/generated/canon_tables.rs` - regenerated weekly from ExifTool"

#### B. Document the Landmines

**If pertinent**, share anything surprising what will break and why:

```bash
# Find what depends on current implementation
rg "trait.*Value" src/
cargo t 2>&1 | grep -i "lens"  # Tests that will catch mistakes
```

Document the dangers:

- "The trait at `src/value.rs:23` is used by 5 processors - changing it breaks all of them"
- "Test `test_canon_lens` will fail if lookup logic is wrong - it tests 200+ real lens IDs"

**Apply SIMPLE-DESIGN.md Rule 2 (Reveals Intention)**:

- Don't just say "this will break" - explain WHY it was designed this way
- "The trait enforces type safety across all camera manufacturers"

#### C. Plan for Change

If the architecture changes, how should the implementer adapt?

```markdown
If PrintConv no longer exists:

1. The user need hasn't changed (readable lens names)
2. Search for new pattern: `rg "human.*readable|display" src/`
3. Core goal remains: "153" → "Canon EF 50mm f/1.8"
```

### Part 3: Define Clear Tasks

Each task needs:

- **What success looks like** (with proof command)
- **How to implement** (with specific locations)
- **How to adapt** (if architecture changed since the TPP was written)

```markdown
### Task: Make Canon lens IDs human-readable

**Success**: `cargo run canon.jpg | grep "LensModel"` shows "Canon EF 50mm" not "153"

**Implementation**:

1. Add PrintConv to `src/canon/lens.rs:45`
2. Copy pattern from `Canon.pm:2847`
3. Wire into `src/processors/canon.rs:process_lens`

**If architecture changed**:

- No PrintConv? Find new display system: `rg "format.*display" src/`
- No lens.rs? Find lens handling: `rg "lens" --type rust`
- Goal unchanged: Binary ID → readable name

**Proof of completion** (follows [SIMPLE-DESIGN.md](SIMPLE-DESIGN.md) Rule 1 - must pass tests):

- [ ] Test passes: `cargo t test_canon_lens_printconv`
- [ ] Integration shown: `rg "lens_printconv" src/` finds usage
- [ ] Old code removed: `rg "raw_lens_id" src/` returns empty (Rule 4 - fewest elements)
```

## Common Anti-Patterns to Avoid

### ❌ The "It Works" Trap

Saying "I tested it and it works" without providing the exact test command.

### ❌ Shelf-ware Code

Beautiful implementation that nothing actually calls in production.

### ❌ The 95% Done Delusion

"Just needs cleanup" usually means 50% more work remains.

## Quality Checklist

Before marking your TPP complete:

- [ ] Problem and success criteria fit in one paragraph
- [ ] Included actual commands that find relevant code
- [ ] Documented at least one "learned the hard way" lesson
- [ ] Each task has a verifiable success command
- [ ] Explained how to adapt if architecture changed
- [ ] **Bug fixes start with failing test** ([TDD.md](TDD.md) requirement)
- [ ] **Code follows Four Rules** ([SIMPLE-DESIGN.md](SIMPLE-DESIGN.md))

## The Ultimate Test

Hand this TPP to someone unfamiliar with the codebase. If they can implement the solution without asking you questions - even if the code was refactored since you wrote it - you've written an excellent TPP.

## Emergency Recovery

Always include break-glass procedures:

```bash
# If something breaks
git diff HEAD~ > my_changes.patch
git apply -R my_changes.patch  # Revert just your changes

# Validate before declaring success
cargo t test_name && make precommit
```
