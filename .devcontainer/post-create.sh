#!/bin/bash
set -e

echo "🚀 Setting up exif-oxide development environment..."

# Ensure we're in the workspace directory
cd /workspace

# Verify critical tools are available
echo "🔍 Verifying tools..."
check_tool() {
  if ! command -v "$1" &>/dev/null; then
    echo "❌ Missing: $1"
    return 1
  fi
  return 0
}

MISSING=0
for tool in git cargo rustc perl python3 jq rg sd shfmt yamllint exiftool; do
  if ! check_tool "$tool"; then
    MISSING=1
  fi
done

if [ $MISSING -eq 1 ]; then
  echo "⚠️  Some tools are missing. Container may need rebuild."
else
  echo "✅ All tools verified"
fi

# Fix any remaining permission issues
if [ -d "/usr/local/cargo/registry" ]; then
  sudo chown -R $(whoami):$(whoami) /usr/local/cargo/registry 2>/dev/null || true
fi

# Set up local Perl environment
echo "📦 Setting up Perl environment..."
eval "$(perl -I$HOME/perl5/lib/perl5 -Mlocal::lib)"

# Set up Claude if it exists in the mounted directory
if [ -d "$HOME/.claude/local/node_modules/.bin" ]; then
  echo "🤖 Setting up Claude..."
  # Create a wrapper script that uses the correct path
  mkdir -p $HOME/bin
  cat >$HOME/bin/claude <<'EOF'
#!/bin/bash
exec "$HOME/.claude/local/node_modules/.bin/claude" "$@"
EOF
  chmod +x $HOME/bin/claude
  echo 'export PATH="$HOME/bin:$PATH"' >>~/.bashrc
  echo 'export PATH="$HOME/bin:$PATH"' >>~/.zshrc
fi

# Run the project's Perl setup
echo "🔧 Installing Perl dependencies..."
make perl-setup
make perl-deps

# Install project dependencies
echo "📚 Installing Rust dependencies..."
cargo fetch

# Run initial code generation
echo "🏗️ Running code generation..."
make codegen

# Done
echo ""
echo "✅ exif-oxide devcontainer ready!"
echo ""
echo "Quick commands: make precommit | make test | make codegen"
echo "Full docs: docs/GETTING-STARTED.md"
