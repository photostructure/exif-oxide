# TPP: Stop codegen from leaving the ExifTool submodule patched

## Summary

Bug: `codegen/Makefile` patched vendored ExifTool modules before running the
Rust generator but never undid those patches when the generator completed or
failed — undo only ran from `clean`. An interrupted 2026-08-08 run left 48
`.pm` files instrumented in `third-party/exiftool`.

Resolution (2026-08-30): codegen no longer touches the submodule at all. It
stages `third-party/exiftool/lib` into a `mktemp -d` copy, patches the copy,
and points the generator at it via `EXIFTOOL_BASE`. The copy is removed by an
`EXIT` trap, so interrupted runs leave no residue and there is nothing to
undo.

## Current phase

- [x] Research & Planning
- [x] Write breaking tests
- [x] Design alternatives
- [x] Task breakdown
- [x] Implementation
- [x] Review & Refinement
- [x] Final Integration

## Design history

Two designs were built in sequence:

### Rejected: trap-based patch lifecycle around the submodule (2026-08-08)

A lifecycle runner (`codegen-patch-lifecycle.sh`) wrapped
precondition/patch/generator/cleanup commands with `EXIT`/`INT`/`TERM` traps
and documented status precedence, plus a fail-closed clean-state precondition
shared between `codegen` and `clean`. It passed a seven-case hermetic shell
test, but review left two unresolved P1s and a P2:

1. Signal forwarding covered only the generator PID, so a hung patcher or a
   `cargo run` descendant escaped cleanup.
2. TOCTOU: the precondition proves the target set clean once, then a long
   codegen runs, then a broad `git checkout` can discard edits made in
   between.
3. Running the precondition before `clean` meant a dirty submodule (the exact
   state undo exists to fix) blocked its own recovery path.

The 2026-08-08 session concluded: "The simpler foundation is to run ExifTool
instrumentation in a temporary copy and point codegen at that copy, so the
real submodule is never mutated during normal generation" — and kept the
lifecycle work uncommitted. The runner, precondition, target-list helper, and
hermetic test were deleted when the staged-copy design landed.

### Adopted: stage a patched copy, never patch the submodule (2026-08-30)

- `codegen/scripts/codegen-with-staged-exiftool.sh` — copies
  `third-party/exiftool/lib` to `mktemp -d`, runs `exiftool-patcher.sh`
  against the copy, runs `cargo run` with `EXIFTOOL_BASE` set, removes the
  copy via an `EXIT`/`INT`/`TERM` trap. Activates `local::lib` so PPI and
  JSON::XS resolve without ambient `PERL5LIB`.
- `codegen/src/main.rs` — honors `EXIFTOOL_BASE` when resolving module paths
  (default unchanged: `../third-party/exiftool`).
- `codegen/scripts/field_extractor.pl` — honors `EXIFTOOL_BASE` for its
  `use lib` ExifTool include path.
- `codegen/scripts/exiftool-patcher.sh` — accepts the directory to patch as an
  optional argument (default unchanged for the ad-hoc analysis scripts in
  `scripts/uniq-*.sh` and `scripts/composite-dependencies.sh`, which still
  patch the submodule in place and can be cleaned up manually with
  `codegen/scripts/exiftool-patcher-undo.sh`).
- `codegen/Makefile` — `codegen-flocked` calls the staging script; `clean` no
  longer runs undo (there is nothing to undo).

This eliminates the whole failure class instead of managing it: no traps
around the submodule, no precondition, no TOCTOU window, no undo ownership.

## Validation

- The 48-file residue from the interrupted 2026-08-08 run was restored via
  `exiftool-patcher-undo.sh` on 2026-08-30 with explicit user approval;
  `git -C third-party/exiftool status --porcelain` is empty.
- `make codegen` runs green against the staged copy and leaves the submodule
  pristine afterwards (verified with `git status` before/after).
