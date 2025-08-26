#!/bin/bash

# Extract composite tag dependencies from ExifTool modules
# Output: composite-dependencies.json

set -e

# Make sure our working directory is the project root:
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Ensure ExifTool is patched
./codegen/scripts/exiftool-patcher.sh >&2

# Output file
OUTPUT="docs/analysis/expressions/composite-dependencies.json"

# Ensure output directory exists
mkdir -p docs/analysis/expressions

echo "Extracting composite tag dependencies..." >&2

# Use our Perl script to extract composite dependencies
perl scripts/extract-composite-deps.pl 2>/dev/null >"$OUTPUT"

# Validate the JSON
if ! jq empty "$OUTPUT" 2>/dev/null; then
  echo "Error: Invalid JSON generated" >&2
  exit 1
fi

# Summary
TOTAL=$(jq '._metadata.total_tags' "$OUTPUT")
echo "Extracted dependencies for $TOTAL composite tags" >&2
echo "Output written to: $OUTPUT" >&2

# Show top composite tags by dependency count
echo "" >&2
echo "Top 5 composite tags by dependency count:" >&2
jq -r '
  .tags | 
  to_entries | 
  map({
    name: .key, 
    require_count: (.value.require | length),
    desire_count: (.value.desire | length),
    total_deps: ((.value.require | length) + (.value.desire | length))
  }) |
  sort_by(.total_deps) | 
  reverse | 
  .[0:5] | 
  .[] | 
  "  \(.name): \(.require_count) required + \(.desire_count) desired = \(.total_deps) total"
' "$OUTPUT" >&2
