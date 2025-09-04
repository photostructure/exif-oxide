# Technical Project Plan (TPP) Guide

## TL;DR - What Makes a Great TPP

A great TPP is like having the original engineer sitting next to you, saying:
- "Here's what we're trying to fix and why it matters to users"
- "I tried X first but it failed because Y - do Z instead" 
- "When I was confused, this file helped: `src/examples/similar.rs`"
- "If the code changed, here's how to find the new version"
- "This weird edge case took me 3 days to find - test with `broken.jpg`"

**The golden rule**: Document the expertise, not just the task.

## Why TPPs Exist

You're about to hand work to someone who has never seen your code. They might be you in 3 months, having forgotten everything. They might be a new engineer who just joined. 

**A good TPP ensures they succeed without having to ask you questions.**

But more importantly: **A great TPP transfers your hard-won expertise so they don't repeat your mistakes.**

Every project has invisible knowledge:
- The approach that looks obvious but fails mysteriously 
- The test file that exposes the edge case
- The refactoring that happened last month that moved everything
- The 3-day debugging session that revealed the real problem

Your TPP captures this expertise before it evaporates.

## The TPP Mindset

Think of a TPP like GPS directions from a local expert who knows all the shortcuts, understands why roads were built where they are, and can adapt when things change.

**Useless GPS**: "Turn left in 500 feet"

**Basic GPS**: "Turn left at Oak Ave (Starbucks on corner)"

**Expert Local Guide**: "We need to reach the hospital before visiting hours end at 8pm. Turn left at Oak Ave - they built this bypass in 2019 to avoid downtown traffic. If there's construction (common in summer), take Elm St instead - it's 2 minutes longer but reliable. The hospital entrance moved last year to the north side, don't trust old signs."

Your TPP is that expert local guide. It provides:

- **Direction**: The exact steps to take today ("turn left at Oak Ave")
- **Context**: Why things are the way they are ("built in 2019 to avoid downtown") 
- **Adaptation**: How to handle changes ("if construction, take Elm St")
- **Expertise**: Hard-won knowledge that prevents failure ("entrance moved, don't trust old signs")

The difference between documentation and a TPP is the difference between a map and a guide who's made this journey before and knows where people get lost.

**Real Engineering Example**:
- **Useless**: "Implement lens name display"
- **Basic**: "Add PrintConv to show lens names in `src/canon/lens.rs`"
- **Expert Guide**: "We need readable lens names because photographers organize by gear. Add PrintConv to `src/canon/lens.rs:45` using the pattern from `nikon.rs:handleLens()`. Don't edit `src/generated/` - those files regenerate weekly. If PrintConv is gone, we likely moved to a new display system - check `src/formatters/`. The tricky part: Canon uses two-byte IDs that need byte-swapping on little-endian systems - I learned this after 3 days of debugging. Test with `canon-5d.raw` which has edge-case lens ID 0x00EF." 

## The Three-Part Structure

### Part 1: The Destination & Purpose (10 minutes)
*What are we building, why does it matter, and what's the deeper goal?*

Write exactly ONE sentence for each:
- **Problem**: What's broken for users right now
- **Root Cause**: WHY this problem exists (the underlying issue)
- **Solution**: What will work after we fix it  
- **Success Test**: The exact command that will prove it works
- **Constraints**: What absolutely cannot change
- **Core Intent**: What we're REALLY trying to achieve (beyond the immediate fix)

Example:
- Problem: Users see "153" instead of lens names in photo metadata
- Root Cause: We're returning raw binary values instead of human-readable strings
- Solution: Display "Canon EF 50mm f/1.8" by implementing PrintConv
- Success Test: `cargo run photo.jpg | grep "Canon EF 50mm"`
- Constraints: Must match ExifTool's Canon.pm:2847 logic exactly
- Core Intent: Make metadata useful for photographers, not just machines

**Why document the Root Cause and Core Intent?**
If the codebase changes and your solution no longer applies, the implementer needs to understand what problem you were REALLY solving so they can find a new path to the same destination.

### Part 2: The Map (30-60 minutes)
*What context does the implementer need?*

#### Find the Landmines (And Why They Exist)
Show the implementer where things will explode AND why these constraints exist:

```bash
# What already exists that we might break?
grep -r "similar_feature" src/
# What tests will tell us if we broke something?
cargo t 2>&1 | grep -i "related"  
# What documentation explains the current approach?
grep -r "YourFeature" docs/
```

Document what you find AND why it matters:
- "The PrintConv system at `src/tags/mod.rs:45` expects a specific trait - it's designed this way to handle 50+ different camera manufacturers uniformly"
- "If you change the trait at `src/value.rs:23`, these 5 processors break because they all assume values are pre-validated"
- "The integration test `test_canon_lens` will catch if you mess up the lookup - this test exists because Canon has 200+ lens IDs with special cases"

**If the landmine no longer exists:** 
Document what to check: "If `src/tags/mod.rs` no longer has PrintConv, check if it moved to `src/conversion/` or if we changed to a different human-readable output system. The goal is still the same: users need readable text, not binary codes."

#### Study the Prior Art (And Its Evolution)
Find patterns to copy, understand why they exist:

```bash
# How did we solve similar problems?
rg "impl.*Similar" --type rust
# Check git history to understand WHY this pattern emerged
git log -p --grep="similar feature" -- src/
# Is this already generated code we shouldn't touch?
ls src/generated/*.rs | xargs grep -l "YourFeature"
```

Document the patterns AND their rationale:
- "Copy the approach from `src/processors/nikon.rs:handleLens()` - it uses a two-stage lookup because Nikon embeds lens data across multiple tags"
- "Never edit `src/generated/canon_tables.rs` - it's regenerated weekly from ExifTool updates. Fix `codegen/src/canon.rs` instead"

**If the pattern changed:**
"If `handleLens()` no longer exists, look for how we process multi-tag data now. The core challenge remains: assembling complete information from fragments across multiple EXIF tags."

#### Understand the Source of Truth
For ExifTool-based projects specifically:

```bash
# Find what ExifTool does
grep -r "YourTag" third-party/exiftool/lib/Image/ExifTool/
# See it in action
exiftool -YourTag test-images/sample.jpg
```

Document the exact behavior to replicate:
- "ExifTool's algorithm at `Canon.pm:2847-2891` handles 3 special cases..."
- "The test file `test-images/canon-quirk.jpg` shows the edge case where..."

### Part 2B: Document Your Scars (The Painful Lessons)

Every complex task has "scar tissue" - the painful lessons you learned the hard way. Document these to save others the suffering:

**Example Scars to Document**:
```markdown
⚠️ SCAR: Don't use HashMap for lens IDs
- What I tried: HashMap<u16, String> for lens lookups
- Why it failed: Canon uses composite IDs across 3 tags, not simple u16
- What actually works: Two-stage lookup with fallback (see nikon.rs:234)
- Time wasted learning this: 2 days

⚠️ SCAR: The test file `canon-old.jpg` lies
- What seems true: It has standard EXIF structure  
- The gotcha: It's actually TIFF-wrapped EXIF from a 2003 firmware bug
- How I learned: 6 hours of debugger stepping
- Use instead: `canon-5d-modern.jpg` for standard structure
```

These scars are gold - they prevent days of wasted effort.

### Part 2C: When The Map No Longer Matches The Territory

**The Architecture Changed - Now What?**

If the implementer discovers the codebase has fundamentally changed, they need your "why chain" to trace back to first principles:

#### The Three-Why Chain (Your North Star)

```markdown
1. SYMPTOM (What we observed):
   "Users see '153' instead of 'Canon EF 50mm f/1.8'"

2. PROBLEM (Why that's bad):  
   "Photographers can't identify which lens took which photo"

3. NEED (Why they care):
   "Photographers organize portfolios by gear to show technical versatility"

4. APPROACH (Why we chose this solution):
   "PrintConv pattern because it's used uniformly across 50+ manufacturers"
```

**If everything changed**, work backwards:
- The NEED (portfolio organization) is eternal - photographers always care about gear
- The PROBLEM (identification) persists - binary data needs human labels
- The SYMPTOM might manifest differently - maybe now it shows "Unknown"
- The APPROACH can completely change - maybe PrintConv became DisplayFormat

**Document your chain for critical decisions**:
```markdown
Lens Name Resolution:
- Symptom: Shows "153" 
- Problem: Can't identify lens
- Need: Gear-based photo organization  
- Approach: PrintConv at src/canon/lens.rs:45

If src/canon/lens.rs doesn't exist:
- The need for gear organization hasn't changed
- Search: rg "lens.*name|lens.*model" --type rust
- Look for: The NEW pattern for human-readable output
- Test with: canon-5d.raw still needs to show "Canon EF 50mm"
```

The key: Even if every file moved and every pattern changed, the user's need remains constant.

### Part 3: The Journey (Task Breakdown)

#### Writing Good Tasks (With Adaptation Built In)

Bad task: "Implement PrintConv for Canon lenses"

Good task:
```markdown
### Task 1: Add human-readable output for Canon lens IDs

**Purpose** (the three whys):
1. Why: Users see "153" instead of lens names
2. Why it matters: Can't identify which lens took which photo
3. Why this approach: PrintConv is our standard pattern for readable output

**Success**: `cargo run canon.jpg` shows "Canon EF 50mm" not "153"

**Primary approach**: 
1. Add PrintConv field to LensInfo struct at `src/canon/lens.rs`
2. Copy lookup logic from `Canon.pm:2847` 
3. Wire into tag processor at `src/processors/canon.rs:process_lens`

**If the architecture changed**:
- If no PrintConv: Find the current human-readable output system
- If no LensInfo struct: Find where lens data is now stored
- If no processors/canon.rs: Find the new Canon metadata handler
- Core goal remains: Map lens ID 153 → "Canon EF 50mm f/1.8 STM"

**Verify**:
- [ ] Code: Added at `src/canon/lens.rs:45-67`
- [ ] Test: `cargo t test_canon_lens_printconv` passes
- [ ] Used: `grep -r "lens_printconv" src/` shows usage in main pipeline
- [ ] Clean: `grep -r "raw_lens_id" src/` returns nothing (old code removed)
```

#### Task Checklist Rules

Every checkbox needs a **command that proves completion**:
- ❌ "Implemented the feature" 
- ✅ "Code at `src/feature.rs:45-89` - run `grep -n "fn feature" src/feature.rs`"

- ❌ "Tests pass"
- ✅ "`cargo t test_feature_integration` exits 0"

- ❌ "Integrated with system"
- ✅ "`rg "call_feature" src/main.rs` shows usage at line 234"

## Common Pitfalls

### The "It Works" Trap
**Problem**: "I tested it manually and it works!"
**Why it fails**: The next engineer can't reproduce your manual test
**Solution**: Document the EXACT commands that prove it works

### The Shelf-ware Syndrome  
**Problem**: Beautiful code that nothing actually uses
**Why it fails**: Dead code is technical debt
**Solution**: Every feature must show `grep` proof of integration

### The 95% Done Delusion
**Problem**: "Just needs cleanup/docs/tests"
**Why it fails**: That last 5% is often 50% of the work
**Solution**: Tasks are binary - done with proof, or not done

## The Empathy Test

Before marking your TPP complete, imagine you're the implementer:

1. **Do I know WHY we're doing this?** (Part 1: Destination & Purpose)
2. **Do I understand the DEEPER PROBLEM?** (The three whys)
3. **Do I know what will BREAK if I'm not careful?** (Part 2: Landmines)  
4. **Do I have EXAMPLES to follow?** (Part 2: Prior Art)
5. **Can I ADAPT if the codebase changed?** (Part 2B: Architecture changes)
6. **Can I PROVE each task is complete?** (Part 3: Checkboxes)
7. **Will I SUCCEED without asking questions?** (The ultimate test)

## Emergency Recovery

Things will go wrong. Include the break-glass instructions:

```bash
# If you break something
git diff HEAD~ > before_i_broke_it.patch
git apply -R before_i_broke_it.patch  # Undo just your changes

# Quick smoke test after any change  
cargo t test_the_feature && echo "Still works!"
make precommit  # Final validation before declaring victory
```

## TPP Quality Gates

Your TPP is ready when:

- [ ] Another engineer can understand the problem in 30 seconds
- [ ] They can find all the context they need with your commands
- [ ] Each task has clear success criteria with proof commands
- [ ] The emergency recovery plan is tested and works
- [ ] You would confidently hand this to a stranger and take a vacation

## Examples of Good TPP Sections

### Good Problem Statement
"Users analyzing Canon photos see cryptic lens ID '153' instead of the human-readable 'Canon EF 50mm f/1.8 STM', making the metadata useless for photographers organizing their gear."

### Good Landmine Documentation
"The tag processor at `src/processors/mod.rs:dispatch()` uses a static registry. If you add a new processor without registering it there, your code will never run. The test `test_all_processors_registered` will catch this."

### Good Task Definition
```markdown
### Add lens name resolution to Canon processor

**Success**: Running `cargo run --bin exif-oxide canon_photo.jpg | grep "LensModel"` 
shows `LensModel: Canon EF 50mm f/1.8 STM` instead of `LensModel: 153`

**Implementation**: Extend the Canon processor at `src/processors/canon.rs:200` to:
1. Read the raw lens ID from tag 0x0095
2. Use the lookup table from `Canon.pm:2847` (already extracted to `src/generated/canon_lenses.rs`)
3. Return the PrintConv string instead of the raw value

**Proof commands**:
- `cargo t test_canon_lens_resolution` - passes
- `rg "lens_lookup\(" src/processors/canon.rs` - shows function called at line 245
- `./scripts/compare_with_exiftool.sh test-images/canon/*.jpg` - matches ExifTool output
```

## The Architectural Resilience Test

Ask yourself: "If someone refactored everything next week, could the implementer still succeed?"

Your TPP passes this test when:
- The "three whys" explain the unchanging user need
- Each constraint explains WHY it exists (not just that it does)
- Tasks describe the goal, not just the current implementation
- Fallback strategies exist for when code locations change

## Final Advice

**Write for the confused future you.** If you wouldn't be able to implement this TPP after forgetting everything about the project, it's not good enough.

**Document the WHY chain.** Every decision should have at least three levels of "why" - the immediate reason, the user impact, and the architectural rationale.

**Test your TPP.** Actually run the commands you're telling the implementer to run. Do they work? Do they give useful output?

**Be specific about the present, flexible about the future.** Give exact file:line references for TODAY'S code, but also explain the patterns and goals that will survive refactoring.

**Show, don't tell.** Instead of "integrate with the system", show exactly where: "Call from `src/main.rs:process_image()` at line 234, or wherever image processing dispatch happens if main.rs changed"

---

Remember: Every hour spent writing a clear TPP saves 10 hours of implementation confusion, failed attempts, and emergency fixes. But more importantly, a TPP that explains WHY can survive architectural changes that would obsolete a TPP that only explains WHAT.

**The ultimate test**: Could an engineer implement this even if the codebase was refactored between when you wrote the TPP and when they read it? If yes, you've written a great TPP.

**The expertise test**: Did you document the invisible knowledge - the scars, the gotchas, the "I wish someone had told me" moments? If an engineer reads your TPP and thinks "Thank god they warned me about that," you've written an excellent TPP.