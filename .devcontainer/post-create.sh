#!/bin/bash
set -e

echo "🚀 Setting up exif-oxide development environment..."

# Ensure we're in the workspace directory
cd /workspace

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

# Create a welcome message
echo "✅ Development environment setup complete!"
echo ""
echo "Welcome to exif-oxide development!"
echo ""
echo "Quick start commands:"
echo "  make precommit    - Run all pre-commit checks"
echo "  make test         - Run all tests"
echo "  make codegen      - Regenerate code from ExifTool"
echo "  make compat-test  - Run compatibility tests"
echo ""
echo "See docs/GETTING-STARTED.md for more information."
