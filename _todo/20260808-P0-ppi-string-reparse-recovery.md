# TPP: PPI rendered-string reparse recovery

## Summary

**Problem**: Two visitor fallbacks render structured PPI children to Rust-like
strings, detect operators in those strings, join operands, and recursively
`split_whitespace()` them. This loses literal/token boundaries and independently
reimplements Perl precedence.
**Why it matters**: Valid ExifTool expressions can silently change meaning:
`$val eq "a or b"` treats the literal's `or` as an operator, concatenated
`" or longer"` is split as syntax, and `2 ** 3 ** 2` becomes left-associative.
**Solution**: Make `ExpressionPrecedenceNormalizer` the only parser for infix
operators and make the visitor render typed `BinaryOperation` nodes only.
**Success test**: Focused AST and public-generator-boundary regressions preserve
operator words and spaces inside string literals and render exponentiation
right-associatively; the forbidden-pattern scan is empty.
**Key constraint**: Preserve ExifTool/Perl semantics exactly. Do not edit
`src/generated/`, delete unrelated recognition, or reparse rendered AST by a
different string technique.

## Current phase

- [x] Research & Planning (2026-08-08)
- [x] Write breaking tests (initial 3/3, alias 2/2, type-safety 3/3 failed as expected)
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation (2026-08-08)
- [x] Review & Refinement (2026-08-08)
- [x] Final Integration (scoped gates pass; external Perl integration blocker recorded)

## Required reading completed

- `AGENTS.md`, `CLAUDE.md`
- `docs/TPP-GUIDE.md`, `docs/TRUST-EXIFTOOL.md`
- `docs/ANTI-PATTERNS.md`, `docs/TDD.md`, `docs/SIMPLE-DESIGN.md`
- `docs/ARCHITECTURE.md`, `docs/CODEGEN.md`
- `coding:tpp` skill instructions

## Scope and assumptions

- Change only handwritten PPI codegen/tests plus this TPP.
- The root crate contains unrelated incomplete QuickTime Task 3 changes and may
  not compile. Do not edit, diagnose, stage, commit, or push them.
- `third-party/exiftool` is already dirty from leaked codegen patches. Do not
  run `make codegen`, any cleanup command, or any Git command in that submodule.
- `cargo test -p codegen` is the broad feasible gate because it isolates the
  package. If shared-worktree dependency edits prevent it, record the exact
  failure without investigating QuickTime.
- Unsupported flat sequences must use the existing `UnsupportedStructure`
  error/function-registry fallback, not speculative output.

## Research evidence

### Proven corrupting flow

1. `codegen/src/ppi/rust_generator/generator.rs:562-570` maps each structured
   child through `visit_node` into `Vec<String>`, then calls
   `try_binary_operation_pattern` on rendered output.
2. `codegen/src/ppi/rust_generator/expressions/mod.rs:118-120` is the second
   rendered-parts caller after other legacy patterns.
3. `codegen/src/ppi/rust_generator/expressions/binary_ops.rs:132-188` scans
   strings as tokens, joins operand slices with spaces, then recursively calls
   `split_whitespace()`. A rendered literal containing `or`, `and`, `eq`, etc.
   is indistinguishable from syntax after the AST boundary is discarded.
4. The duplicate precedence table at `binary_ops.rs:8-29` says every equal
   precedence operator is left-associative (`<=` at line 146), contradicting
   Perl exponentiation. Thus `2 ** 3 ** 2` can render as `(2 ** 3) ** 2`.

### Canonical architecture already present

- `codegen/src/ppi/normalizer/multi_pass.rs:107-147` installs
  `ExpressionPrecedenceNormalizer` first in the standard three-pass pipeline.
- `expression_precedence.rs:171-200` classifies infix expressions and routes
  them to precedence climbing; lines 718-747 explicitly preserve equal-
  precedence right associativity and create typed `BinaryOperation` nodes.
- `visitor.rs:64-103` dispatches those typed nodes; lines 1769 onward render
  their two structural operands without reparsing them.
- `shared_pipeline.rs:45-64` passes PPI output through canonical normalization
  before `RustGenerator`; this call graph was audited but not executed here.

### Perl word-operator ground truth (review follow-up)

- Installed PPI 1.283 `PPI/Token/Operator.pm:17-31` lists `and`, `or`,
  `xor`, `not`, `eq`, and `ne` as operators and states that even superficially
  word-like operators are `PPI::Token::Operator`, not `PPI::Token::Word`.
- `PPI/Token/Word.pm:140-154` confirms the tokenizer reclasses exact entries
  from PPI's operator table as `Operator` tokens.
- Vendored `docs/reference/perlop.txt:123-155` orders `eq` above `and`, and
  `and` above `or`; both word logical operators are left-associative.
- Real ExifTool conditions use these shapes: `BMP.pm:201` has
  `eq "LINK" or ... eq "MBED"`; `Sigma.pm:555` has `>= 3 and ... eq "string"`.
- The precedence normalizer already produced the correct typed trees. Only the
  visitor rejected `BinaryOperation("and"|"or")`; no parser change was needed.

### History

- Commit `1bdcd0c6` (2025-12-05) added the recursive rendered-string split and
  duplicate precedence table as a fallback.
- The first rendered-parts call predates that fallback (`git blame` attributes
  `generator.rs:562-570` to `4a750ec5`, 2025-08-18), but the recursive reparse
  made its type loss and associativity errors substantially broader.
- Current mandatory scan has one match:
  `binary_ops.rs:171: expr.split_whitespace()`.

### Concrete regressions to prove

- Flat AST `[Symbol($val), Operator(eq), Quote("a or b")]` must keep the whole
  literal as the right operand and never emit logical `||` from its contents.
- The documented sprintf/concat case's literal-sensitive core must pass a
  parsed PPI JSON `Document` through public `generate_function`, keeping
  `" or longer"` intact. This test does not claim to execute the Perl parser or
  the complete compound expression through `shared_pipeline`.
- `2 ** 3 ** 2` must normalize/render as `power(2, power(3, 2))`, matching
  Perl's right-associative exponentiation, never `power(power(2, 3), 2)`.

## Solutions considered

### Option A: canonical AST-only infix path (preferred)

Delete `process_expression_recursively`, the duplicate string-level precedence
table/parser/generator, and both calls that detect binary operators after
rendering children. Add AST/public-generator regressions. Let typed
`BinaryOperation` nodes reach `visit_normalized_binary_operation`; let an
unnormalized flat infix sequence fall through to the established unsupported
error/fallback.

Pros: restores type boundaries, one precedence source, correct associativity,
fewest elements, and matches documented architecture. Cons: legacy direct-AST
tests/callers that skipped normalization may now fail explicitly; they should
be routed through normalization, not patched with another parser.

### Option B: pass original PPI nodes into legacy binary detection

Rewrite the visitor fallback to scan `PpiNode` operator children and recurse on
node slices without strings.

Pros: retains legacy behavior without literal confusion. Cons: still creates a
second precedence parser beside `ExpressionPrecedenceNormalizer`, violates the
single canonical architecture, and risks semantic drift. Rejected.

### Option C: quote-aware/token-aware parsing of rendered strings

Keep rendered strings but replace whitespace splitting with a lexer.

Pros: superficially small caller change. Cons: still reparses AST renderings,
must understand generated Rust wrappers as well as Perl, and merely disguises
the banned anti-pattern. Rejected.

## Tasks

### Task 1: Add breaking AST/public-generator tests

- Add a focused integration test file under `codegen/tests/` using
  direct flat PPI nodes for the forbidden visitor path and a parser-backed PPI
  JSON fixture through public `RustGenerator::generate_function`.
- Cover operator words inside a comparison literal, the `" or longer"`
  concat core, and right-associative chained exponentiation.
- Assert positive structure and explicit absence of corrupt forms.
- Run only that test target and record the exact expected failures below.

Proof:

- [x] Focused tests fail on the confirmed corruption before implementation.
- [x] Failures arise from generated output, not parser/setup errors.

### Task 2: Remove rendered-string binary parsing

- Remove the pre-render/detect block at `generator.rs:562-570`.
- Remove binary detection at `expressions/mod.rs:118-120`.
- Delete `process_expression_recursively`, duplicate precedence parsing, and
  string-level binary generation from `expressions/binary_ops.rs`, retaining
  only helpers still used by typed visitor rendering.
- If a caller exposes a genuinely flat infix AST, route it through existing
  normalization at the AST boundary or allow `UnsupportedStructure` to trigger
  established fallback. Never infer operators from rendered strings.

Proof:

- [x] `rg "split_whitespace|\.join.*split" codegen/src/ppi/` is empty.
- [x] `rg "try_binary_operation_pattern|process_expression_recursively" codegen/src/ppi/`
      is empty unless a remaining symbol is proven structural (none expected).
- [x] Focused regression tests pass.

### Task 3: Regression and architecture gates

- Run `cargo test -p codegen --test ppi_ast_rendering_regressions` (final name
  may vary with the test file).
- Run `cargo test -p codegen` if feasible in the dirty shared worktree.
- Run focused normalizer/rust-generator unit tests if the broad package test
  exposes unrelated failures.
- Review the diff for unrelated pattern deletion and generated-file edits.
- Update this TPP with exact commands, counts, failures, assumptions, and any
  divergence. Keep it under 400 lines.

Proof:

- [x] Focused tests green.
- [x] Codegen package tests green, or exact unrelated blocker recorded.
- [x] No file under any `generated/` directory changed by this task.
- [x] No ExifTool submodule command or cleanup was run.
- [x] No staging, commit, or push was performed.

### Task 4: Accepted review follow-up

- Add red/green typed-node tests for exact Perl word operators `and` and `or`,
  including precedence-tree assertions and Rust short-circuit output.
- Add a public `parse_ppi_json` fixture → `generate_function` regression so
  coverage is not limited to direct `visit_statement` calls.
- Audit repeated normalization boundaries and prove representative idempotence.

Proof:

- [x] Review red run: 5 passed, only `and`/`or` failed as unsupported.
- [x] Review green run: all 7 focused regressions passed.
- [x] Full public PPI JSON fixture preserves the concat literal.
- [x] Repeated structural normalization produces identical serialized AST.

### Task 5: Context-aware logical value semantics

- Add red/green tests for `$val or $val` in Condition context, a mixed
  comparison/value Condition, and comparison pairs in ValueConv context.
- Type-check all three complete generated functions with `rustc` against a
  minimal `TagValue` contract, not only string-match their bodies.
- In Condition context, structurally identify boolean nodes and coerce value
  nodes with Perl `is_truthy()` semantics. In TagValue contexts, evaluate the
  left operand once and return one owned `TagValue` branch type, wrapping
  comparison results as `TagValue::Bool`; fail closed for unproven shapes.

Proof:

- [x] Type-safety red run: 7 passed, the three reviewer repros failed exactly.
- [x] Green run: all 10 focused tests passed and all three functions type-check.

## Success gates

- [x] String literals containing operator words remain opaque AST operands.
- [x] Concatenated `" or longer"` remains one literal value.
- [x] `2 ** 3 ** 2` is right-associative in normalized AST and generated Rust.
- [x] No PPI AST-to-rendered-string binary parsing remains.
- [x] Canonical normalizer/visitor is the only infix precedence path.
- [x] Unsupported constructs fail through the established fallback.
- [x] Required test and scan results are recorded here.
- [x] Exact PPI word operators `and`/`or` render via typed short-circuit paths.
- [x] Public generator-boundary coverage parses a PPI JSON fixture.

## Files expected to change

- `_todo/20260808-P0-ppi-string-reparse-recovery.md` (this plan/log)
- `codegen/tests/ppi_ast_rendering_regressions.rs` (new focused tests)
- `codegen/src/ppi/rust_generator/generator.rs`
- `codegen/src/ppi/rust_generator/expressions/mod.rs`
- `codegen/src/ppi/rust_generator/expressions/binary_ops.rs`
- `codegen/src/ppi/rust_generator/mod.rs` (remove obsolete trait wiring)
- `codegen/src/ppi/rust_generator/visitor.rs` (typed `and`/`or` rendering)

## Open questions and adaptation

- If later Perl-parser pipeline tests reveal `ExpressionPrecedenceNormalizer` does not
  normalize one flat sequence, first add a failing normalizer test and repair
  typed parsing there. Do not restore visitor parsing.
- If legacy handlers depend on `generate_binary_operation_from_parts`, move
  only rendering helpers to typed visitor code; do not preserve a string parser
  for compatibility.
- No user input is available during this delegated task. Record any material
  ambiguity here and choose the narrowest behavior-preserving path.

## Session log

### 2026-08-08 research

- Completed all required reading and safety checks from absolute repo paths.
- Baseline worktree is dirty with unrelated QuickTime/config/docs changes and a
  dirty ExifTool submodule; none are in this task's edit set.
- Baseline forbidden scan found exactly one `split_whitespace` occurrence in
  `binary_ops.rs:171`.
- No code implementation or test edit preceded this TPP.

### 2026-08-08 red tests

- Added `codegen/tests/ppi_ast_rendering_regressions.rs` with three focused
  tests that call `RustGenerator::visit_statement` on the proven flat PPI-node
  sequence, bypassing only the outer normalization entry so the visitor's
  fallback is exercised directly.
- Command: `cargo test -p codegen --test ppi_ast_rendering_regressions`
  exited 101; 0 passed, 3 failed, all for the exact expected semantic damage:
  - `$val eq "a or b"` rendered `val.to_string() == "a || b"`.
  - `"5.00 s" . " or longer"` rendered concat with `" || longer"`.
  - `2 ** 3 ** 2` rendered `power(power(2, 3), 2)` (left-associative).
- There were no parser, fixture, dependency, or setup failures.

### 2026-08-08 implementation and green verification

- `generator.rs`: `visit_statement` and `visit_expression` now normalize their
  PPI container structurally before visiting it. Removed the earlier block that
  rendered every child to `String` and then searched those strings for infix
  operators.
- `expressions/mod.rs`: removed the second rendered-parts binary fallback;
  unrecognized legacy combinations now reach the existing
  `UnsupportedStructure`/function-registry fallback.
- `expressions/binary_ops.rs`: deleted the duplicate precedence table, recursive
  whitespace split, string-level operator parser/generator, and unused wrapper;
  the file now contains only operand-rendering helpers consumed by the typed
  visitor.
- `rust_generator/mod.rs`: removed the obsolete `BinaryOperationsHandler`
  trait export/implementation.
- Focused command after implementation:
  `cargo test -p codegen --test ppi_ast_rendering_regressions` exited 0;
  3 passed, 0 failed, 0 ignored.
- `cargo test -p codegen --lib` exited 0; 122 passed, 0 failed, 3 ignored.
- `cargo clippy -p codegen --lib -- -D warnings` exited 0.
- `cargo fmt --all -- --check` exited 0.
- `git diff --check -- <six task paths>` exited 0.
- Both architecture scans exited 1 with no output (the expected no-match code):
  - `rg "split_whitespace|\.join.*split" codegen/src/ppi/`
  - `rg "try_binary_operation_pattern|process_expression_recursively|generate_binary_operation_from_parts|get_operator_precedence" codegen/src/ppi/`

### Broad package test limitation

- `cargo test -p codegen` was attempted. Both duplicated unit-test binaries
  completed successfully (`src/lib.rs`: 122 passed/3 ignored; `src/main.rs`:
  121 passed/3 ignored), as did the zero-test tool binaries.
- The command then exited 101 in pre-existing integration test
  `function_import_sync_test` before parsing an expression because the host
  Perl executable and XS library are mismatched:
  `got first handshake key 0xf380080, needed 0xeb80080`.
- This is external setup, not a generated-output assertion. Per scope, it was
  recorded without diagnosing QuickTime, regenerating code, or touching the
  dirty ExifTool submodule.
- `make verify` was not run: the shared root has unrelated incomplete QuickTime
  work, and this task explicitly forbids codegen/submodule cleanup.

### 2026-08-08 accepted review follow-up

- Red command: `cargo test -p codegen --test ppi_ast_rendering_regressions`
  exited 101 with 5 passed/2 failed. Normalized-tree assertions passed; public
  generation failed only with `Unsupported binary operator: and` and
  `Unsupported binary operator: or`.
- `visitor.rs` now matches only exact typed aliases: `"&&" | "and"` and
  `"||" | "or"`. This first review implementation was subsequently tightened
  by Task 5 because its rendered-string type heuristic emitted invalid Rust.
  Precedence is not inferred here—it remains encoded by the `BinaryOperation` tree.
- Added a parser-backed public generator-boundary test:
  `parse_ppi_json(PPI Document/Statement/Quote/./Quote)` →
  `RustGenerator::generate_function`, proving `" or longer"` remains opaque.
- Normalization-call audit found intentional repetition, not exactly-once use:
  `shared_pipeline` normalizes for returned diagnostics/hash and then
  `generate_body` normalizes its supplied tree; expression-test generation
  normalizes before registry hashing and registry generation later calls
  `generate_function`; TagKit instead registers raw AST and normalizes during
  generation. Direct `visit_statement`/`visit_expression` normalize at their
  public boundary, and the legacy flat ternary fallback normalizes each
  extracted branch. Typed nodes are skipped by expression classification,
  making reapplication structural and idempotent; a focused serialized-AST
  equality regression now proves the representative `eq/or/eq` case.
- Green results:
  - focused integration target: 7 passed, 0 failed, 0 ignored;
  - `cargo test -p codegen --lib`: 122 passed, 0 failed, 3 ignored;
  - `cargo clippy -p codegen --lib -- -D warnings`: passed;
  - `cargo clippy -p codegen --test ppi_ast_rendering_regressions -- -D warnings`: passed;
  - task-scoped `rustfmt --edition 2021 --check <six Rust files>`: passed;
  - forbidden parser scans: no matches (exit 1 as expected).
- The current `cargo fmt --all -- --check` passes.
- No codegen/regeneration, ExifTool-submodule command, staging, commit, or push
  occurred.

### 2026-08-08 accepted type-safety follow-up

- Red focused run exited 101: 7 passed/3 failed. `$val or $val` returned
  `TagValue` branches from a `-> bool` function; mixed `eq/and/$val` called
  `.is_truthy()` on `bool`; ValueConv comparison pairs emitted `Ok(bool)`.
- `visitor.rs` now makes result typing from PPI node classes/operators and
  `ExpressionType`, never rendered strings. Conditions emit short-circuit bool
  expressions with structural truthiness coercion. ValueConv/PrintConv logical
  expressions use a scoped `logical_left`, preserve operand return and
  short-circuit semantics, and unify comparison values with `TagValue::Bool`.
- Focused green: 10 passed/0 failed; each of the three new complete generated
  functions also passed an actual `rustc --crate-type=lib` type-check.
- `cargo test -p codegen --lib`: 124 passed/0 failed/3 ignored.
- `cargo test -p codegen`: both unit binaries passed (124/3 ignored and 126/3
  ignored), then exited 101 at `function_import_sync_test` due the unchanged
  Perl/XS handshake mismatch (`0xf380080` vs `0xeb80080`).
- `cargo clippy -p codegen --lib -- -D warnings` and full fmt check pass. The
  all-target Clippy command remains blocked by unrelated existing warnings in
  registry/tool/test files; no such warning is in the changed PPI library.
- Both forbidden-parser scans are empty (expected `rg` exit 1).

### 2026-08-30 first full regeneration: six defect classes fixed

The first `make codegen` since the change produced 18 compile errors. Each was
reproduced by a focused regression before the fix; all live in
`codegen/tests/ppi_ast_rendering_regressions.rs` (10 → 21 tests).

- Ternary conditions rendered the ValueConv/PrintConv `logical_left` block in
  `if` position. `visitor.rs` now renders a condition in boolean context
  (`render_condition_as_bool`), recursing through logical operators instead of
  sniffing the rendered string for `==`.
- `split_processed_expression_on_commas` returned multi-token arguments
  unnormalized, so `join`/`sprintf` never saw the typed `FunctionCall` for a
  nested `unpack`. `sprintf` lost Perl's argument splatting and `join` got a
  `TagValue::Array` where `&[TagValue]` was required.
- Computed `unpack` templates (`"H2" x 29`) render an owned `String`;
  `render_unpack_template` borrows them for the `&str` parameter.
- A ValueConv interpolated literal (`"Unknown ($val)"`) renders `format!`, a
  `String`. Ternary branches and `return` values now use the structural owned
  conversion, falling back to the string-shape wrapper only for unproven shapes.
- Precedence climbing silently dropped the operand of a leading `!` (it parses a
  left operand first). Unary preprocessing now emits a typed `LogicalNegation`.
- `visit_break` joined each child's rendering with spaces, leaking Perl's `.`
  into the Rust source; it now renders the post-`return` tokens as one
  expression. `defined` fails closed instead of emitting a bare identifier, and
  `>>` renders (there is no `Shl` impl, so `<<` still fails closed).

Results: `cargo test -p codegen` 290 passed/0 failed; `make codegen` clean;
`cargo check --all-targets --features test-helpers,integration-tests` and
`cargo clippy --all-targets --all-features -- -D warnings` clean; `cargo t` 744
passed with one unrelated failure (the compat ratchet wants
`QuickTime:CreationDate` and `QuickTime:Software` removed from
`config/compat_known_gaps.json` — they now match, from the in-flight QuickTime
work). Seven composite ValueConvs gained implementations and the tag-function
placeholder count fell from 272 to 254.
