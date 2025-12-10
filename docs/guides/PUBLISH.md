# Publishing Guide for exif-oxide

This guide explains the automated release system set up for exif-oxide and how to publish new versions.

## Overview

The project uses a modern, automated release workflow based on:

- **release-plz**: Automated version bumps and release PRs
- **Conventional Commits**: Semantic versioning based on commit messages
- **GitHub Actions**: Automated CI/CD pipeline
- **Feature flags**: Conditional integration tests for published crates

## Initial Setup (Already Complete)

The following has been configured:

### 1. GitHub Actions CI Pipeline

- **File**: `.github/workflows/ci.yml`
- **Coverage**: ARM/Intel macOS, ARM/Intel Windows, ARM64/AMD64 Linux (glibc + musl)
- **Tests**: Clippy, format checks, unit tests, and integration tests

### 2. Automated Release System

- **File**: `.github/workflows/release-plz.yml`
- **Configuration**: `release-plz.toml`
- **Features**: Automatic version bumps, changelog generation, GitHub releases

### 3. Dependency Management

- **File**: `.github/dependabot.yml`
- **Updates**: Weekly automated dependency updates

### 4. Package Configuration

- **Optimized package size**: 312KB (down from 431MB)
- **Feature flags**: Integration tests require `--features integration-tests`
- **Exclusions**: Test assets and development files excluded from published crate

## Publishing Workflow

### Development Flow

1. **Make changes** following conventional commit format:

   ```bash
   git commit -m "feat: add new EXIF tag support"    # Minor version bump
   git commit -m "fix: handle malformed IFD data"    # Patch version bump
   git commit -m "chore: update dependencies"        # No version bump
   ```

2. **Push to main**:

   ```bash
   git push origin main
   ```

3. **release-plz automatically**:
   - Creates a release PR with version bump
   - Updates `CHANGELOG.md`
   - Runs full CI pipeline

### Release Flow

1. **Review release PR** created by release-plz
2. **Merge release PR** when ready
3. **Automatic actions**:
   - Version bump in `Cargo.toml`
   - Git tag created (e.g., `v0.2.0`)
   - Published to crates.io
   - GitHub release created with changelog

## Manual Publishing (First Time Only)

For the initial v0.1.0 release, manual publishing is required:

### 1. Login to crates.io

```bash
cargo login
```

This opens your browser for authentication.

### 2. Publish

```bash
cargo publish
```

### 3. Set Up Trusted Publishing (After First Release)

1. Go to https://crates.io/me
2. Find the "Publishing" section (appears after first publish)
3. Add `photostructure/exif-oxide` as a trusted publisher
4. Future releases will use secure OIDC authentication

## Testing

### Local Development

```bash
make unit-test              # Fast unit tests only
make test                   # All tests including integration tests
make precommit              # Full validation before commit
```

### Published Crate Testing

Users of the published crate get:

```bash
cargo test                  # Unit tests only (239 tests)
cargo test --features integration-tests  # All tests (requires test assets)
```

## Conventional Commit Format

Use these prefixes for automatic version bumps:

| Prefix   | Version Bump          | Example                                 |
| -------- | --------------------- | --------------------------------------- |
| `feat:`  | Minor (0.1.0 → 0.2.0) | `feat: add RAW format support`          |
| `fix:`   | Patch (0.1.0 → 0.1.1) | `fix: handle corrupted EXIF data`       |
| `perf:`  | Patch                 | `perf: optimize tag lookup performance` |
| `chore:` | None                  | `chore: update dependencies`            |
| `docs:`  | None                  | `docs: update API documentation`        |
| `test:`  | None                  | `test: add integration tests for Canon` |

### Breaking Changes

Add `!` after type or include `BREAKING CHANGE:` in footer:

```bash
git commit -m "feat!: change API to return Result<>"  # Major version bump
```

## Configuration Files

### release-plz.toml

Controls release behavior:

- Conventional commit types that trigger releases
- Changelog generation
- GitHub release creation
- PR template

## Troubleshooting

### Release PR Not Created

- Check commit messages follow conventional format
- Ensure commits include `feat:`, `fix:`, or `perf:` prefixes
- Verify GitHub Actions have proper permissions

### Publish Fails

- Check package size (should be <10MB)
- Verify all dependencies are valid
- Ensure tests pass: `make precommit`

### Integration Tests Fail in CI

- Verify test assets are available in repository
- Check that `integration-tests` feature is enabled in CI
- Ensure all integration test files have `#![cfg(feature = "integration-tests")]`

## Future Maintenance

The release system is fully automated. Your only responsibilities are:

1. **Write good commit messages** using conventional format
2. **Review and merge release PRs** created by release-plz
3. **Monitor CI status** and fix any issues

The system handles:

- ✅ Version bumping
- ✅ Changelog generation
- ✅ Git tagging
- ✅ crates.io publishing
- ✅ GitHub releases
- ✅ Dependency updates

## Emergency Manual Release

If automation fails, you can always release manually:

```bash
# Bump version in Cargo.toml manually
cargo publish
git tag v0.x.x
git push origin v0.x.x
```

Then create a GitHub release manually using the tag.
