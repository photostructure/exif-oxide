# Kent Beck's Four Rules of Simple Design

These rules provide objective criteria for evaluating code design. They're in priority order—higher rules take precedence over lower ones when they conflict.

## Rule 1: Passes the Tests

**The code must work as intended.**

- All functionality proven through automated tests
- Nothing else matters if the system behaves incorrectly
- Tests provide confidence to refactor and improve design
- Avoid tests that only verify implementation details—tests should assert correct behavior

**Example**: Before optimizing tag parsing logic, ensure comprehensive tests prove correctness across all supported camera formats.

**Pitfall**: Don't skip tests for "simple" changes—camera metadata has countless edge cases.

## Rule 2: Reveals Intention

**Code should clearly express what it does and why.**

- Use descriptive names for variables, functions, and types
- Structure code to match the problem domain
- Prioritize readability for future maintainers

**Example**: 
```rust
// Poor intention
fn proc_tag(d: &[u8]) -> Result<Val, Err> { ... }

// Clear intention  
fn parse_canon_lens_model(raw_data: &[u8]) -> Result<LensModel, ParseError> { ... }
```

**Pitfall**: Don't sacrifice clarity for brevity—`parse_canon_lens_model` is better than `parse_clm`.

## Rule 3: No Duplication

**Eliminate repeated logic and knowledge.**

- Look for both obvious code duplication and hidden duplication
- Hidden duplication includes parallel class hierarchies and repeated concepts
- Use codegen to eliminate manual maintenance of lookup tables

**Example**: Instead of manually maintaining lens databases in multiple modules, extract them to `codegen/config/` and generate lookup functions automatically.

**Pitfall**: Don't create premature abstractions—sometimes temporary duplication is acceptable while understanding emerges.

## Rule 4: Fewest Elements

**Remove anything that doesn't serve the first three rules.**

- Avoid classes, methods, and abstractions added for speculative future needs
- Prefer simple solutions over architecturally complex ones
- Delete unused code ruthlessly

**Example**: Don't build a plugin system for tag processors until you have concrete evidence multiple implementations are needed.

**Pitfall**: Don't over-apply this rule—necessary complexity is still necessary.

## Rule 5: No bogus guardrails or defaults

When key assumptions that your code relies upon to work appear to be broken, fail early and visibly, rather than attempting to patch things up. In particular:

- Lean towards propagating errors up to callers, instead of silently "warning" about them inside of try/catch blocks.
- If you are fairly certain data should always exist, assume it does, rather than producing code with unnecessary guardrails or existence checks (esp. if such checks might mislead other programmers)
- Never use 'defaults' as a result of errors, either for users, or downstream callers.

## Priority and Conflicts

**When rules conflict, higher numbers win:**

- Working code (Rule 1) beats everything
- Clear names (Rule 2) and no duplication (Rule 3) often reinforce each other
- The "duplication vs clarity" debate misses the point—both improve together over time

**Common conflict**: During refactoring, you might temporarily duplicate code to pass tests, then eliminate duplication while improving names.

**Exception**: In test code, empathy for readers sometimes trumps technical purity.

## Quick Reference

Use this checklist during code review:

- ✅ **Tests pass**: All functionality verified
- ✅ **Clear intent**: Names and structure express purpose
- ✅ **No duplication**: Logic appears in exactly one place
- ✅ **Minimal elements**: No unused or speculative code

## Integration with exif-oxide

These rules align with our core principles:

- **Trust ExifTool**: Rule 1 ensures our translations work correctly
- **Codegen everything**: Rule 3 eliminates manual lookup table maintenance  
- **Clear naming**: Rule 2 helps future contributors understand camera-specific quirks
- **Incremental scope**: Rule 4 keeps us focused on mainstream tags

**Remember**: Design quality is predictably evaluable, not subjectively judged. These rules help sort out obvious problems before they become technical debt.