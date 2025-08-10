#!/bin/bash

# Format Perl files in parallel
# This script formats Perl files using perltidy with parallel processing

set -e

# Check for perltidy in common locations
if ! command -v perltidy >/dev/null 2>&1; then
  echo "⚠️  Warning: perltidy not found, skipping Perl formatting"
  exit 0
fi

find . -name "*.pl" \
  -not -path "./third-party/*" \
  -type f \
  -print0 | xargs -0 -P 12 -n 3 perltidy -b

find . -name "*.pl.bak" -delete
