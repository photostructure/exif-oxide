# Release Process

## How to Release

1. **Edit CHANGELOG.md** with your release notes
2. **Go to Actions → Build & Release → Run workflow**
3. **Select version bump** (patch, minor, or major)
4. **Click "Run workflow"**

The workflow will:

- Run format, clippy, and tests on Linux
- Run tests on macOS and Windows
- Bump version in all Cargo.toml files
- Commit and create signed git tag
- Push to main
- Publish to crates.io
- The tag triggers `build-binaries.yml` which builds binaries for 5 platforms and creates the GitHub Release

## Workflows

| Workflow             | Trigger                               | Purpose                                      |
| -------------------- | ------------------------------------- | -------------------------------------------- |
| `build.yml`          | Push, PR, or manual workflow_dispatch | CI checks + release on manual trigger        |
| `build-binaries.yml` | Git tag (`v*.*.*`)                    | Cross-platform binary builds, GitHub release |

## Build Targets

Binaries are built natively on all platforms (no cross-compilation):

| Target                      | OS           | Architecture | Runner            |
| --------------------------- | ------------ | ------------ | ----------------- |
| `x86_64-unknown-linux-gnu`  | Linux        | x86_64       | `ubuntu-latest`   |
| `aarch64-unknown-linux-gnu` | Linux        | ARM64        | `ubuntu-24.04-arm`|
| `x86_64-apple-darwin`       | macOS        | Intel        | `macos-15-intel`  |
| `aarch64-apple-darwin`      | macOS        | Apple Silicon| `macos-15`        |
| `x86_64-pc-windows-msvc`    | Windows      | x86_64       | `windows-latest`  |
| `aarch64-pc-windows-msvc`   | Windows      | ARM64        | `windows-11-arm`  |

## Security

The binary release workflow:

- **No curl-to-sh scripts** - No piping remote scripts to shell
- **All native runners** - No cross-compilation, builds run on native architecture
- **SHA256 checksums** - Generated for every release artifact
- **Pinned action versions** - All GitHub Actions pinned to specific commit SHAs
- **Draft releases** - Artifacts upload to draft, then published after all builds succeed

## Installation Options

After release, users can install via:

```bash
# From source (requires Rust)
cargo install exif-oxide

# Pre-built binary (no Rust needed)
cargo binstall exif-oxide

# Or download directly from GitHub Releases
```

## Configuration Files

- `.github/workflows/build.yml` - CI and release workflow
- `.github/workflows/build-binaries.yml` - Binary builds (ripgrep-style)

## Why Not cargo-dist or release-plz?

We evaluated both tools and opted for a simpler, ripgrep-style approach instead.

### cargo-dist

- **Security concern**: Installs via `curl ... | sh` - piping remote scripts to shell
- **Complexity**: ~300 line generated workflow with JSON manifests and multi-phase orchestration
- **Opacity**: Hard to audit what the tool actually does
- **Dependency**: Requires installing cargo-dist binary in CI

### release-plz

- **PR-based workflow**: Creates release PRs rather than direct releases, adding process overhead
- **Automatic changelogs**: git-cliff integration produces mechanical changelogs; we prefer hand-written release notes
- **Configuration complexity**: Multiple config files (release-plz.toml, cliff.toml)

### Our approach

Following [ripgrep](https://github.com/BurntSushi/ripgrep), [bat](https://github.com/sharkdp/bat), and [fd](https://github.com/sharkdp/fd):

- **~170 lines** of straightforward YAML
- **Native runners only** - no cross-compilation tools to install
- **Fully auditable** - every step is visible in the workflow
- **No external dependencies** - just cargo, gh CLI, and standard Unix tools
