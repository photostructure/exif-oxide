# TPP: Fail closed when codegen extraction is incomplete

## Summary

**Bug**: the Rust codegen binary logs field-extractor failures, accepts zero
symbols, prints `Code generation complete`, and exits successfully.
**Why it matters**: generated Rust can silently omit ExifTool data while CI and
release automation treat the result as authoritative.
**Solution**: reject an empty module result and aggregate every selected-module
failure before strategy processing; never generate from a partial extraction.
**Success test**: focused unit tests and an isolated `--modules QuickTime.pm`
preview return an error/nonzero status and never print the completion message.
**Key constraint**: do not generate files in `src/generated/`, touch the dirty
ExifTool submodule, or change the user's Perl installation.

## Current phase

- [x] Research & Planning
- [x] Write breaking tests
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation
- [x] Review & Refinement
- [ ] Final Integration

## Required reading

- [AGENTS.md](../AGENTS.md)
- [TPP-GUIDE.md](../docs/TPP-GUIDE.md)
- [TDD.md](../docs/TDD.md)
- [SIMPLE-DESIGN.md](../docs/SIMPLE-DESIGN.md)
- [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md)
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md)
- `codegen/src/main.rs`
- `codegen/src/field_extractor.rs`

## Ground truth and constraints

- `main.rs::run_universal_extraction` currently catches each
  `extract_module` error, emits a warning, and continues. It returns success
  even if all selected modules fail.
- The same function treats an empty aggregate as a warning-only condition and
  returns success. `main` therefore reaches `Code generation complete`.
- `FieldExtractor::extract_module` trusts a zero exit status even when stdout
  has no valid symbols. In the current environment, the Perl process reports
  an XS handshake mismatch but exits zero and supplies no symbols.
- A partial run is not faithful: the output directory does not encode which
  configured modules failed, and stale files may make it look complete.
  Therefore any selected-module failure must abort before strategy processing.
- The Perl XS mismatch is an environment blocker to successful real extraction,
  not permission to modify the user's Perl installation in this task.
- Other agents own current PPI, QuickTime, patch-lifecycle, and submodule work.
  This task changes only codegen entry/extractor code, focused tests, and this
  TPP. It must not stage, commit, run `make codegen`, or invoke submodule Git.

## Solutions

### Option A: aggregate and fail before dispatch (preferred)

Keep extracting all selected modules so one run reports every failed path, but
collect contextual errors. Reject any failure before `process_symbols`; also
reject zero-symbol module results at the extractor boundary.

Pros: complete diagnostics, no partial output, minimal control-flow change,
and `main`'s existing `?` prevents the completion message. Cons: one broken
module blocks all selected generation, which is intentional for authoritative
generated sources.

### Option B: fail on the first module error

Pros: smallest code. Cons: repeated fix/run cycles expose one module at a time
and obscure whether the failure is systemic. Rejected because aggregation is
equally safe and materially more useful.

### Option C: generate successful modules and return nonzero afterward

Rejected: it mutates output with a knowingly incomplete extraction, leaving
stale or partial generated files that can be mistaken for a complete port.

## Tasks

1. [x] Add a test seam around module extraction without changing production
   behavior, then add breaking tests proving all-failed, partial-failed, and
   zero-symbol selections cannot return success.
2. [x] Add a focused extractor test proving a zero-exit Perl helper with empty
   stdout is an error that retains module/stderr context. Also prove one valid
   JSONL record cannot mask a malformed record from the same module.
3. [x] Aggregate module failures and return one contextual error before any
   strategy/output processing. Preserve the existing main-level `?` so the
   completion log remains success-only.
4. [x] Run focused bin/library tests, `cargo fmt --check`, and focused Clippy.
   If safe, run the generator only against a temporary output directory with
   `--modules QuickTime.pm` and prove nonzero status/no completion message.

## Success gates

- [x] A selected module error makes universal extraction return `Err`.
- [x] One success plus one failure still returns `Err` before output dispatch.
- [x] Empty stdout/zero symbols cannot be successful extraction or codegen.
- [x] Malformed JSONL cannot be hidden by valid records from the same module.
- [x] Errors name each failed module and retain the underlying cause.
- [x] The CLI returns nonzero and never announces completion on this failure.
- [x] No generated file, submodule file, staged state, or Perl install changed.

## Expected files

- `_todo/20260808-P0-codegen-extraction-failclosed.md`
- `codegen/src/main.rs`
- `codegen/src/field_extractor.rs`

## Session log

### 2026-08-08 research

- Confirmed the per-module loop warns and continues on `Err`, then warns and
  succeeds when `all_symbols` is empty.
- Chose fail-any-module over partial generation because partial output cannot
  preserve the faithful-port contract and may be combined with stale files.
- The mandatory pre-edit scan initially found the already-owned PPI
  `split_whitespace` violation. This task does not modify PPI code.
- Added four breaking tests. Before the fix, each failed at `unwrap_err()`
  because the tested path returned `Ok`: selected failure, one-success/one-
  failure, zero symbols, and an empty zero-exit Perl helper with stderr.
- Added an intra-module partial-extraction test. Before the fix it failed at
  `unwrap_err()` because one valid JSONL record masked malformed line 2.

### 2026-08-08 implementation and validation

- `FieldExtractor` now rejects malformed JSONL records and zero symbols with
  module/line/stderr context. The universal loop collects every failed module,
  refuses to dispatch any successfully extracted subset, and has a final
  zero-aggregate invariant. `main` still logs completion only after this
  function returns success through its existing `?`.
- All five focused regressions pass after the fix. The broad codegen binary
  suite passes: 126 passed, 0 failed, 3 ignored. The library suite passes: 124
  passed, 0 failed, 3 ignored.
- `cargo fmt -p codegen -- --check` and scoped `git diff --check` pass.
  `cargo clippy -p codegen --bin codegen --locked --offline` succeeds with
  three pre-existing warnings at `main.rs:79`, `main.rs:89`, and `main.rs:364`.
  A strict all-test Clippy run also reports unrelated existing lints in other
  binaries/tests; this task did not widen scope to fix them.
- Isolated preview from `codegen/`:
  `cargo run -p codegen --locked --offline -- --output
  /tmp/exif-oxide-codegen-failclosed.1FmZKB --modules QuickTime.pm` exited 1,
  named `QuickTime.pm` and the underlying Perl XS handshake mismatch, omitted
  `Code generation complete`, and left the temporary output directory empty.
- The XS mismatch remains an environment blocker to successful extraction.
  No attempt was made to alter Perl. Root Git status still shows the already
  dirty ExifTool submodule; no submodule command or file write was performed.
- Full repository `make verify`/`cargo t` remains for Final Integration after
  the concurrent QuickTime/codegen worktree is assembled. Keep this TPP in
  `_todo/` until that shared gate passes.
