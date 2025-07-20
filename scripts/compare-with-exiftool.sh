#!/bin/bash
set -euo pipefail

# Compare exif-oxide output with ExifTool's JSON output
# Usage: ./scripts/compare-with-exiftool.sh <image-file>

if [ $# -ne 1 ]; then
    echo "Usage: $0 <image-file>"
    echo "Compare exif-oxide output with ExifTool's JSON output"
    exit 1
fi

INPUT_FILE="$1"

if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: File '$INPUT_FILE' not found"
    exit 1
fi

# Check for required tools
for tool in exiftool jq diff cargo; do
    if ! command -v $tool &> /dev/null; then
        echo "Error: Required tool '$tool' is not installed"
        exit 1
    fi
done

# Create temp directory for outputs
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

EXIFTOOL_OUTPUT="$TEMP_DIR/exiftool.json"
EXIF_OXIDE_OUTPUT="$TEMP_DIR/exif-oxide.json"
EXIFTOOL_SORTED="$TEMP_DIR/exiftool-sorted.json"
EXIF_OXIDE_SORTED="$TEMP_DIR/exif-oxide-sorted.json"

echo "Processing: $INPUT_FILE"
echo "----------------------------------------"

# Run ExifTool with JSON output, structured output, and group names
echo "Running ExifTool..."
if ! exiftool -j -struct -G "$INPUT_FILE" > "$EXIFTOOL_OUTPUT" 2>&1; then
    echo "Error: ExifTool failed to process the file"
    cat "$EXIFTOOL_OUTPUT"
    exit 1
fi

# Run exif-oxide
echo "Running exif-oxide..."
# Note: exif-oxide doesn't support --json flag yet, so we get JSON by default
if ! cargo run --quiet -- "$INPUT_FILE" > "$EXIF_OXIDE_OUTPUT" 2>&1; then
    echo "Error: exif-oxide failed to process the file"
    cat "$EXIF_OXIDE_OUTPUT"
    exit 1
fi

# Sort both outputs using jq for consistent ordering
# This ensures the diff is minimal and focuses on actual differences
echo "Sorting outputs..."

# Sort ExifTool output - it returns an array with one object
jq '.[0] | to_entries | sort_by(.key) | from_entries' "$EXIFTOOL_OUTPUT" > "$EXIFTOOL_SORTED" 2>/dev/null || {
    echo "Error: Failed to parse ExifTool JSON output"
    exit 1
}

# Sort exif-oxide output - it also returns an array with one object
jq '.[0] | to_entries | sort_by(.key) | from_entries' "$EXIF_OXIDE_OUTPUT" > "$EXIF_OXIDE_SORTED" 2>/dev/null || {
    echo "Error: Failed to parse exif-oxide JSON output"
    exit 1
}

# Show the diff
echo "Differences between ExifTool and exif-oxide:"
echo "----------------------------------------"
echo "Legend: < = ExifTool only, > = exif-oxide only"
echo ""

# Use diff with unified format for better readability
if diff -u "$EXIFTOOL_SORTED" "$EXIF_OXIDE_SORTED"; then
    echo "✅ No differences found! Outputs match."
else
    echo ""
    echo "----------------------------------------"
    echo "Summary: Differences detected between outputs"
fi

# Optionally show tag counts
EXIFTOOL_COUNT=$(jq 'length' "$EXIFTOOL_SORTED")
EXIF_OXIDE_COUNT=$(jq 'length' "$EXIF_OXIDE_SORTED")

echo ""
echo "Tag counts:"
echo "  ExifTool:   $EXIFTOOL_COUNT tags"
echo "  exif-oxide: $EXIF_OXIDE_COUNT tags"

# Save raw outputs for debugging if needed
if [ "${DEBUG:-}" = "1" ]; then
    echo ""
    echo "Debug mode: Raw outputs saved to:"
    echo "  ExifTool:   $EXIFTOOL_OUTPUT"
    echo "  exif-oxide: $EXIF_OXIDE_OUTPUT"
    trap - EXIT  # Don't delete temp dir in debug mode
fi