# Publishing Guide

exif-oxide uses a manually dispatched GitHub Actions workflow. There is one
publishable crate, `exif-oxide`; the `codegen` workspace member is an internal
tool with `publish = false`.

The short operator checklist is in [RELEASE.md](../../RELEASE.md). This guide
documents setup, validation, and recovery details.

## Prerequisites

### GitHub

The `Build & Release` workflow needs:

- `contents: write` to push the release commit and tag;
- `id-token: write` for crates.io trusted publishing;
- `SSH_SIGNING_KEY`, `GIT_USER_NAME`, and `GIT_USER_EMAIL` repository secrets
  for the configured release commit/tag identity.

All third-party actions are pinned to full commit SHAs. Update those pins in a
separate reviewed dependency commit.

### crates.io trusted publisher

Configure the crates.io package to trust this repository's
`.github/workflows/build.yml` workflow. The workflow obtains a short-lived
token through `rust-lang/crates-io-auth-action`; it does not store a long-lived
crates.io API token in GitHub.

## Preparing `main`

From a clean checkout:

```bash
make preflight
git status --short
```

Preflight updates dependencies and GitHub Actions, regenerates source and
missing compatibility snapshots, applies automatic fixes, and finishes with
`make verify`. Review and commit those changes before release. It is not safe
to dispatch a release with unreviewed preflight output or an uncommitted
changelog.

Update `CHANGELOG.md` by hand, commit the final release preparation, push it to
`main`, and wait for the normal CI run.

## Manual workflow sequence

Open **Actions → Build & Release**, choose **Run workflow**, and select the
SemVer bump. The workflow performs these guarded stages:

1. Run the existing Linux checks and Linux/macOS/Windows tests.
2. Install exactly cargo-edit 0.13.13 from its published lockfile.
3. Run `cargo set-version` for `exif-oxide` only. cargo-edit updates root
   `Cargo.toml` and the corresponding root `Cargo.lock` package entry.
4. Fail if any other file changed, then create the local release commit.
5. On that bumped commit, run locked format, Clippy, and test validation.
6. Run, in order:

   ```bash
   cargo package --list --locked -p exif-oxide
   cargo package --locked -p exif-oxide
   cargo publish --dry-run --locked -p exif-oxide
   ```

7. Create the version tag and push the commit and tag.
8. Authenticate through crates.io trusted publishing and run a locked publish
   for `exif-oxide` only.

The workflow never runs `cargo update`. A release bump must not select new
dependency versions after the tested preparation commit.

## Package contents

The Cargo package allowlist contains:

- Rust sources, including generated translation modules;
- compatibility configuration JSON read by the library and comparison tool;
- Cargo metadata and lockfile; and
- README, changelog, and license.

Integration tests, TPPs, scripts, local test media, the vendored ExifTool
submodule, and CI/editor configuration are repository inputs, not crate
contents. Inspect every release with `cargo package --list`; do not infer the
package from the worktree.

## What happens after the tag

`build-binaries.yml` validates that the tag matches `Cargo.toml`, builds on six
native platform/architecture runners, produces SHA-256 checksums, and uploads
the archives to a draft GitHub Release. It publishes that release only when
all builds complete.

## Troubleshooting

### Version-bump scope check fails

Do not weaken the file check. A release bump should modify only root
`Cargo.toml` and `Cargo.lock`. Put dependency or workspace-member changes in a
separate reviewed commit, then rerun the workflow.

### Package or dry-run fails before push

No remote release state exists yet. Reproduce the three package commands in a
clean checkout, fix the source or manifest on `main`, and dispatch again.

### Push fails

The local runner commit/tag were not published to crates.io. Check whether
`main` advanced or the tag already exists. Never overwrite an existing release
tag; reconcile the repository state and make a new deliberate dispatch.

### Publish fails after the tag was pushed

Inspect the crates.io version and the tag-triggered binary workflow before any
retry. If crates.io already accepted the version, it cannot be overwritten.
If only binaries failed, repair that workflow without republishing the crate.

### Manual emergency publish

Use the exact tagged commit and repeat locked validation, package build, and
publish dry-run first. Manual publishing is a recovery procedure, not a way to
bypass a failed gate.
