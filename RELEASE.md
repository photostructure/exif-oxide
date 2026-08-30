# Release Process

Releases are started manually with the `Build & Release` GitHub Actions
workflow. No automated release PR is created.

## Prepare the release

1. Run `make preflight` locally.
2. Review and commit every dependency, generated-code, snapshot, and
   automatic-fix change produced by preflight.
3. Update `CHANGELOG.md`, review it, and commit it.
4. Push the clean, reviewed preparation commits to `main` and wait for CI.

`make preflight` is intentionally mutating. Do not treat its output as an
automatic release commit.

## Run the workflow

1. Open **Actions → Build & Release → Run workflow**.
2. Choose `patch`, `minor`, or `major`.
3. Run the workflow from `main`.

The publish job runs only after the Linux checks and cross-platform test matrix
pass. It then:

1. installs the pinned cargo-edit release;
2. bumps only the publishable `exif-oxide` package and its lockfile entry;
3. rejects any version-bump diff outside root `Cargo.toml` and `Cargo.lock`;
4. creates a local release commit, but does not tag or push it yet;
5. reruns locked format, Clippy, and test validation on that commit;
6. lists and builds the locked crate package, then performs a locked crates.io
   publish dry-run;
7. creates and pushes the version tag only after every check succeeds; and
8. authenticates with crates.io trusted publishing and publishes only
   `exif-oxide`.

No dependency upgrade runs in the release workflow. Dependency changes must be
prepared, tested, reviewed, and committed separately before dispatch.

The pushed `v*.*.*` tag starts `build-binaries.yml`, which builds native
binaries for Linux x86-64/ARM64, macOS Intel/Apple Silicon, and Windows
x86-64/ARM64. It uploads checksums to a draft GitHub Release and publishes the
release only after every binary build succeeds.

## Failure boundaries

- A failure before the push step leaves no remote release commit or tag. Fix
  the problem on `main` and dispatch again.
- A push failure leaves crates.io untouched. Resolve the branch/tag conflict
  before retrying; do not force-push a release tag.
- A failure after the tag push may have started the binary workflow. Inspect
  both workflows before retrying or changing any tag.
- The crates.io publish is irreversible for that version. Never publish
  manually unless the package and dry-run commands below pass on the exact
  tagged commit.

## Local package rehearsal

Run these commands from a clean checkout of the release candidate:

```bash
cargo package --list --locked -p exif-oxide
cargo package --locked -p exif-oxide
cargo publish --dry-run --locked -p exif-oxide
```

The package must contain `src/generated/` and `config/*.json`; both are needed
by the published crate. Tests, plans, scripts, the ExifTool submodule, and
repository administration files are intentionally excluded.

## Workflows

| Workflow                               | Trigger                   | Purpose                              |
| -------------------------------------- | ------------------------- | ------------------------------------ |
| `.github/workflows/build.yml`          | Push, PR, manual dispatch | CI and the guarded crates.io release |
| `.github/workflows/build-binaries.yml` | Version tag               | Native binaries and GitHub Release   |
