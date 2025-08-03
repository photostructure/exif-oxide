#!/bin/bash

# Format Perl files in parallel
# This script formats Perl files using perltidy with parallel processing

set -e

# Check for perltidy in common locations
if command -v perltidy >/dev/null 2>&1; then
  PERLTIDY_DEFAULT=$(command -v perltidy)
elif [ -x "$HOME/perl5/bin/perltidy" ]; then
  PERLTIDY_DEFAULT="$HOME/perl5/bin/perltidy"
else
  echo "⚠️  Warning: perltidy not found, skipping Perl formatting"
  echo "   Install with: cpanm Perl::Tidy"
  exit 0
fi

# Function to format a single Perl file
format_perl_file() {
  local file="$1"

  # Check syntax first
  if ! perl -c "$file" >/dev/null 2>&1; then
    echo "❌ Error: $file has syntax errors"
    return 1
  fi

  # Set up local::lib environment and run perltidy
  local perltidy_path="${PERLTIDY_PATH:-$PERLTIDY_DEFAULT}"
  eval $(perl -I "$HOME/perl5/lib/perl5/" -Mlocal::lib) && "$perltidy_path" -st "$file" >"$file.tmp" && mv "$file.tmp" "$file"
}

# Export function and variables so xargs can use them
export -f format_perl_file
export PERLTIDY_DEFAULT

# Find Perl files and process them in parallel with up to 8 processes
find . -name "*.pl" \
  -not -path "./third-party/*" \
  -type f \
  -print0 | xargs -0 -P 8 -I {} bash -c 'format_perl_file "$@"' _ {}
