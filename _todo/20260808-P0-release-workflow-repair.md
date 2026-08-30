# TPP: Repair the manual release workflow

## Summary

The manual `Build & Release` workflow cannot publish: it stages and publishes
the nonexistent `exif-oxide-core` package, broadly updates dependencies after
CI has tested a different lockfile, and pushes a tag before checking the crate
that will be uploaded. The publishing guide instead describes deleted
release-plz automation.

Success means a manual dispatch bumps only the publishable `exif-oxide`
package, creates a local release commit, validates that exact locked tree and
crate before any remote mutation, then tags, pushes, and publishes only
`exif-oxide`. Documentation must describe those actual steps and failure
boundaries.

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
- [release process](../RELEASE.md)
- [publishing guide](../docs/guides/PUBLISH.md)

## Ground truth and constraints

- The workspace has two members: publishable `exif-oxide` and internal
  `codegen` (`publish = false`). There is no `exif-oxide-core` package.
- Before this repair, the workflow ran its matrix tests, bumped versions, ran
  `cargo update --workspace`, and immediately committed/tagged/pushed. The
  broad new dependency resolution was never tested.
- `cargo set-version --bump patch -p exif-oxide` was exercised in an isolated
  copy. It changed exactly root `Cargo.toml` and the `exif-oxide` package entry
  in `Cargo.lock`; it did not change `codegen/Cargo.toml` or dependency
  resolutions.
- cargo-edit 0.13.13 is the current upstream release (2026-07-15). Its
  manifest declares Rust 1.92; the workflow deliberately installs it with
  both exact `--version 0.13.13` and `--locked` under the current stable Rust
  toolchain.
- Package inspection found 1,659 files: 1,391 generated Rust sources are
  necessary, while tests, TPPs, scripts, and repository administration files
  are not. The packed crate was 1.5 MiB, so this is content hygiene rather
  than a crates.io size-limit emergency.
- The existing action SHA pins are a separate, already-reviewed change and
  must remain untouched.

## Regression checks written first

The pre-change workflow was checked for release invariants. The check failed
on all of these expected defects:

- references to `exif-oxide-core`;
- unpinned `cargo install cargo-edit`;
- `cargo update --workspace` after the tested tree;
- no package listing, package verification, or publish dry-run;
- tag and push occurring before package validation;
- stale release-plz and deleted workflow/configuration claims in the guide.

Because this repair is confined to workflow and documentation files, these
are repeatable read-only static checks rather than a new repository script.

## Design

1. Install exactly cargo-edit 0.13.13 with its published lockfile.
2. Bump only `exif-oxide`. Let cargo-edit make the corresponding lockfile
   package-version change, then fail unless the diff contains exactly root
   `Cargo.toml` and `Cargo.lock`.
3. Create the local release commit so Cargo's package commands can enforce a
   clean source tree. Do not tag or push yet.
4. Run locked format and workspace compilation, then test and Clippy the
   publishable package before package listing, verification, and publish
   dry-run. (`codegen` has unrelated Clippy debt and its integration tests
   require a working local Perl PPI/JSON::XS installation, so release CI
   compiles it without expanding this repair into a codegen/toolchain sweep.)
5. Only after every check passes, create the tag and push commit/tag. Use
   crates.io trusted publishing to publish only `exif-oxide` with `--locked`.
6. Narrow package contents to runtime/library sources, required compatibility
   configuration, and user-facing legal/package documents.
7. Permit the publish job only for a manual dispatch of `main`, and push the
   checked-out release commit explicitly as `HEAD:main` so detached-checkout
   behaviour cannot select a missing or unintended local branch.

## Alternatives considered

### Run `cargo update --workspace` and retest

Rejected. A release version bump must not silently become a dependency update.
Dependency upgrades belong in a separately reviewed commit before release.

### Tag before package validation

Rejected. A failed package or dry-run would leave an immutable remote release
coordinate pointing at an artifact that never published.

### Publish every workspace member

Rejected. `codegen` is an internal tool and explicitly has `publish = false`.

## Tasks

1. [x] Capture the broken static invariants and isolated version-bump diff.
2. [x] Repair `.github/workflows/build.yml` ordering, pin cargo-edit, remove
       broad dependency updates and nonexistent package references, and add all
       locked validation/package gates before tag/push/publish.
3. [x] Narrow the Cargo package allowlist without excluding required generated
       sources or compatibility JSON.
4. [x] Rewrite `RELEASE.md` and `docs/guides/PUBLISH.md` to describe the actual
       manual workflow, trusted publishing, and failure boundaries.
5. [x] Run YAML lint/static invariant checks and exercise version bump,
       package-list, package, and publish-dry-run commands in an isolated copy.
6. [ ] Run repository `make verify` after concurrent parser/codegen work is
       integrated, then move this TPP to `_done/` with the release repair commit.

## Acceptance gates

- No workflow or publishing guide reference to `exif-oxide-core`,
  release-plz, or deleted release configuration.
- cargo-edit is installed at one exact, documented version with `--locked`.
- Version bump changes only root `Cargo.toml` and `Cargo.lock`; no dependency
  resolution command runs.
- Locked validation, `cargo package --list`, `cargo package --locked`, and
  `cargo publish --dry-run --locked` pass before tag or push.
- Only `exif-oxide` is published.
- The package contains required generated Rust and compatibility config, but
  excludes tests, plans, scripts, and repository-only files.
- Final `make verify` passes on the integrated worktree.

## Progress and validation

- The initial static regression check found every listed workflow defect.
  After the repair, `yamllint .github/workflows/build.yml`, `git diff
--check`, and negative searches for `exif-oxide-core`, the deleted release
  automation, and workflow `cargo update` all pass. Line-order inspection puts
  package list/build/dry-run before tag, push, authentication, and publish.
- Installed cargo-edit 0.13.13 with `--locked` into an isolated `/tmp` root on
  Rust/Cargo 1.96.1. Its `cargo set-version --bump patch -p exif-oxide` changed
  exactly root `Cargo.toml` (`0.2.0-dev` to `0.2.0`) and the matching
  `exif-oxide` entry in `Cargo.lock`; `codegen/Cargo.toml` and dependency
  resolutions were byte-identical.
- The package allowlist reduced the inspected package from 1,659 files / 13.1
  MiB unpacked / 1.5 MiB compressed to 1,555 files / 12.4 MiB unpacked / 1.3
  MiB compressed. It retains all `src/generated/` files and three `config`
  JSON files. `cargo package --locked -p exif-oxide --allow-dirty` and `cargo
publish --dry-run --locked -p exif-oxide --allow-dirty` both passed. The
  `--allow-dirty` applies only to this shared-worktree rehearsal; the workflow
  runs the required commands on its clean local release commit.
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets
--all-features --locked`, and `cargo clippy -p exif-oxide --all-targets
--all-features --locked -- -D warnings` pass on the shared worktree.
- The exact locked `cargo test -p exif-oxide --features
test-helpers,integration-tests` release command ran. Unit and preceding
  integration tests passed, but the compatibility gate correctly failed on
  concurrent QuickTime progress: three allowlisted QuickTime tags now match
  and must be removed, while volatile `File:FileInodeChangeDate` differed from
  its local snapshot. This task does not own that allowlist/oracle work, so
  Final Integration and `make verify` remain pending until those concurrent
  changes are integrated.
- A deliberately broader workspace Clippy rehearsal exposed pre-existing
  `codegen` warning debt, and workspace tests reached a local Perl
  PPI/JSON::XS ABI mismatch. The release workflow therefore compiles the
  entire workspace but applies Clippy/tests to the only published package;
  codegen's full gate remains `make preflight`/`make verify` during reviewed
  release preparation.

## Files

- `.github/workflows/build.yml`
- `Cargo.toml` (package content allowlist only)
- `RELEASE.md`
- `docs/guides/PUBLISH.md`
- `_todo/20260808-P0-release-workflow-repair.md`
