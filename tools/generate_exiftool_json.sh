#!/bin/bash
# Generate reference snapshots from ExifTool for insta testing
#
# This script creates reference snapshots that serve as the authoritative
# source of truth for compatibility testing. These snapshots should never
# be auto-updated by our code - they represent ExifTool's expected output.

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SNAPSHOTS_DIR="$PROJECT_ROOT/generated/exiftool-json"

# Tags currently supported by exif-oxide (Milestone 7)  
# Conservative list - only tags that work perfectly with existing implementations
# Single source of truth now maintained in config/supported_tags.json (Milestone 8a)
# Milestone 8c: Now using group prefixes, need to specify allowed groups
SUPPORTED_TAGS=$(cat "$PROJECT_ROOT/config/supported_tags.json")
ALLOWED_GROUPS='["EXIF", "File", "System", "GPS"]'

echo "Generating ExifTool reference snapshots for exif-oxide compatibility testing"
echo "Project root: $PROJECT_ROOT"
echo "Snapshots directory: $SNAPSHOTS_DIR"

# Check if ExifTool is available
if ! command -v exiftool &> /dev/null; then
    echo "Error: ExifTool is not installed or not in PATH"
    echo "Please install ExifTool: https://exiftool.org/"
    exit 1
fi

echo "ExifTool version: $(exiftool -ver)"

# Clean and create snapshots directory
mkdir -p "$SNAPSHOTS_DIR"
rm -f "$SNAPSHOTS_DIR"/*.json

# Create temporary file for all JPEG data
TEMP_JSON=$(mktemp)
trap "rm -f '$TEMP_JSON'" EXIT

echo "Scanning for JPEG files..."

# Get all JPEG files from both test directories
# Note: Using default ExifTool behavior (rational arrays) for Milestone 6
# Decimal GPS conversion will be implemented in Milestone 8 (ValueConv), by using `exiftool -r -json -GPSLatitude\# -GPSLongitude\# -GPSAltitude\# ... -all ...`
# Milestone 8c: Using -G flag to get group-prefixed tag names (e.g., "EXIF:Make", "GPS:GPSLatitude")

if ! exiftool -r -json -G -all -if '$MIMEType eq "image/jpeg"' \
    "$PROJECT_ROOT/test-images" \
    "$PROJECT_ROOT/third-party/exiftool/t/images" \
    > "$TEMP_JSON" 2>/dev/null; then
    echo "Warning: ExifTool scan failed or no JPEG files found"
    echo "Contents of temp file:"
    cat "$TEMP_JSON" || true
    exit 1
fi

# Check if we got any data
if [ ! -s "$TEMP_JSON" ] || [ "$(cat "$TEMP_JSON")" = "[]" ]; then
    echo "No JPEG files found in test directories"
    echo "Checked directories:"
    echo "  - $PROJECT_ROOT/test-images"
    echo "  - $PROJECT_ROOT/third-party/exiftool/t/images"
    exit 1
fi

echo "Processing JPEG files and generating snapshots..."

# Count files for progress
TOTAL_FILES=$(jq length "$TEMP_JSON")
echo "Found $TOTAL_FILES JPEG files"

if [ "$TOTAL_FILES" -eq 0 ]; then
    echo "No JPEG files to process"
    exit 1
fi

# Process each file and create individual snapshots
jq -c '.[]' "$TEMP_JSON" | while IFS= read -r file_data; do
    # Extract source file path
    SOURCE_FILE=$(echo "$file_data" | jq -r '.SourceFile')
    
    if [ "$SOURCE_FILE" = "null" ] || [ -z "$SOURCE_FILE" ]; then
        echo "Warning: Skipping file with missing SourceFile"
        continue
    fi
    
    # Make path relative to project root
    RELATIVE_PATH=$(realpath --relative-to="$PROJECT_ROOT" "$SOURCE_FILE" 2>/dev/null || echo "$SOURCE_FILE")
    
    # Generate snapshot name from relative path
    # Replace any sequence of non-alphanumeric characters with single underscore
    SNAPSHOT_NAME=$(echo "$RELATIVE_PATH" | sed 's/[^a-zA-Z0-9]\+/_/g')
    SNAPSHOT_FILE="$SNAPSHOTS_DIR/${SNAPSHOT_NAME}.json"
    
    # Filter to only supported tags and save as snapshot
    # Handle group-prefixed tag names (e.g., "EXIF:Make" -> check if "Make" is supported)
    # Milestone 8c: Also check that the group is allowed (EXIF, File, System, GPS)
    echo "$file_data" | jq --argjson tags "$SUPPORTED_TAGS" --argjson groups "$ALLOWED_GROUPS" \
        'with_entries(select(
            .key as $k | 
            if $k == "SourceFile" then true
            elif ($k | contains(":")) then
                (($k | split(":")) as $parts |
                ($parts[0] as $group | $parts[1] as $tag_name |
                ($groups | index($group)) and ($tags | index($tag_name))))
            else false
            end
        ))' \
        > "$SNAPSHOT_FILE"
    
    # echo "Created: $SNAPSHOT_FILE (for $SOURCE_FILE)"
done

echo ""
echo "Snapshot generation complete!"
echo "Generated snapshots in: $SNAPSHOTS_DIR"
echo ""

# Show summary
SNAPSHOT_COUNT=$(find "$SNAPSHOTS_DIR" -name "*.json" | wc -l)
echo "Summary:"
echo "  - Total snapshots created: $SNAPSHOT_COUNT"
echo "  - Supported tags: $(echo "$SUPPORTED_TAGS" | jq -r '.[]' | tr '\n' ' ')"
echo ""
echo "To run compatibility tests:"
echo "  make compat-test"