#!/bin/bash

# Check Perl files for syntax errors
# This script validates Perl files using perl -c with parallel processing

set -e

# Function to check a single Perl file
check_perl_file() {
  local file="$1"

  # Check syntax
  if ! perl -c "$file" >/dev/null 2>&1; then
    echo "❌ Error: $file has syntax errors"
    # Show the actual error
    perl -c "$file" 2>&1 | sed 's/^/   /'
    return 1
  fi
}

# Export function so xargs can use it
export -f check_perl_file

# Find Perl files and process them in parallel with up to 8 processes
find . -name "*.pl" \
  -not -path "./third-party/*" \
  -type f \
  -print0 | xargs -0 -P 8 -I {} bash -c 'check_perl_file "$@"' _ {}
