#!/bin/bash

# Format JSON files in parallel
# This script formats JSON files using jq with parallel processing

set -e

if ! command -v jq >/dev/null 2>&1; then
  echo "⚠️  Warning: jq not found, skipping JSON formatting"
  echo "   Install with: sudo apt-get install jq"
  exit 0
fi

# Function to format a single JSON file
format_json_file() {
  local file="$1"
  if jq --indent 2 . "$file" >"$file.tmp" 2>/dev/null; then
    mv "$file.tmp" "$file"
  else
    echo "⚠️  Warning: Failed to format $file (invalid JSON?)"
    rm -f "$file.tmp"
  fi
}

# Export function so xargs can use it
export -f format_json_file

# Find JSON files and process them in parallel with up to 8 processes
find . -name "*.json" \
  -not -path "./third-party/*" \
  -not -path "./target/*" \
  -not -path "./*/target/*" \
  -not -path "./generated/*" \
  -not -path "./*/generated/*" \
  -type f \
  -print0 | xargs -0 -P 8 -I {} bash -c 'format_json_file "$@"' _ {}
