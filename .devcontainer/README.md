# exif-oxide Development Container

Complete development environment for exif-oxide with Rust, Perl, and all required tools.

## What's Included

- **Rust 1.83**: rustfmt, clippy, rust-analyzer, cargo-audit, cargo-edit, sd
- **Perl**: cpanminus, Perl::LanguageServer, Perl::Critic, Perl::Tidy, ExifTool
- **Python 3**: yamllint, jsonschema, ruff
- **Tools**: ripgrep, jq, shfmt, lldb, gdb, git

## Files

- `devcontainer.json` - Container configuration
- `Dockerfile` - Image definition with all dependencies
- `post-create.sh` - Runs `make perl-deps`, `make codegen`, `cargo fetch`
- `init-firewall.sh` - Restricts network to approved domains (GitHub, crates.io, Anthropic, etc.)

## Volume Mounts

Persists across rebuilds:
- `~/.claude` → Claude config
- `~/.cargo/registry` → Crate cache
- `~/.cargo/git` → Git dependency cache

## Quick Start

### With VS Code
1. Open in VS Code: "Reopen in Container"
2. Wait for build and post-create (~5-10 min first time)
3. Verify: `make check-deps`

### Manual (without VS Code)
```bash
# Interactive (runs and removes on exit)
.devcontainer/run.sh run

# Or persistent (keeps running in background)
.devcontainer/run.sh start    # Start container
.devcontainer/run.sh exec     # Connect to it
.devcontainer/run.sh stop     # Stop and remove

# Other commands
.devcontainer/run.sh build    # Just build image
.devcontainer/run.sh clean    # Remove container and image
```

## Common Issues

**Permission errors:** `sudo chown -R vscode:vscode /usr/local/cargo`

**Firewall blocks domain:** Add to `domains` array in `init-firewall.sh`

**Codegen fails:** `git submodule update --init --recursive && make codegen`

## Adding Dependencies

- **System:** `Dockerfile` apt-get section
- **Rust:** cargo install section
- **Perl:** `cpanfile` in project root
- **Python:** pip3 install section
- **Network:** `init-firewall.sh` domains array

Order Dockerfile layers by change frequency for optimal caching.
