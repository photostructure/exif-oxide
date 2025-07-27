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

# Supported file extensions for compatibility testing
# Add new extensions here as support is added
SUPPORTED_EXTENSIONS=("jpg" "jpeg" "orf" "raw" "mrw" "rw2" "cr2" "arw" "sr2" "srf" "png" "gif" "tif" "tiff" "avif" "dng" "heic" "heif" "webp")

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

# Create snapshots directory if it doesn't exist
mkdir -p "$SNAPSHOTS_DIR"

# Check if we should regenerate all files (optional --force flag)
FORCE_REGENERATE=false
if [ "${1:-}" = "--force" ]; then
    FORCE_REGENERATE=true
    echo "Force regeneration enabled - all snapshots will be recreated"
    rm -f "$SNAPSHOTS_DIR"/*.json
fi

# Create temporary file for all supported image data
TEMP_JSON=$(mktemp)
trap "rm -f '$TEMP_JSON'" EXIT

echo "Scanning for supported image files..."
echo "Supported extensions: ${SUPPORTED_EXTENSIONS[*]}"

# Build ExifTool filter condition dynamically from supported extensions
# This avoids hardcoding MIME types and makes it easy to add new formats
EXTENSION_CONDITIONS=()
for ext in "${SUPPORTED_EXTENSIONS[@]}"; do
    # Convert extension to ExifTool condition using exact match
    # Handle both lower and upper case extensions (jpg, JPG, orf, ORF, etc.)
    EXTENSION_CONDITIONS+=("\$FileTypeExtension eq \"${ext}\"")
    if [ "$ext" != "${ext^^}" ]; then
        EXTENSION_CONDITIONS+=("\$FileTypeExtension eq \"${ext^^}\"")
    fi
done

# Join conditions with ' or ' properly for ExifTool
FILTER_CONDITION=""
for i in "${!EXTENSION_CONDITIONS[@]}"; do
    if [ $i -eq 0 ]; then
        FILTER_CONDITION="${EXTENSION_CONDITIONS[$i]}"
    else
        FILTER_CONDITION="$FILTER_CONDITION or ${EXTENSION_CONDITIONS[$i]}"
    fi
done

# Debug: echo "ExifTool filter: $FILTER_CONDITION"

# Get all supported image files from both test directories
# Note: Using default ExifTool behavior (rational arrays) for Milestone 6
# Milestone 8c: Using -G flag to get group-prefixed tag names (e.g., "EXIF:Make", "GPS:GPSLatitude")

if ! exiftool -r -json -struct -G -GPSLatitude\# -GPSLongitude\# -GPSAltitude\# -FileSize\# -all -if "$FILTER_CONDITION" \
    "$PROJECT_ROOT/test-images" \
    "$PROJECT_ROOT/third-party/exiftool/t/images" \
    > "$TEMP_JSON" 2>/dev/null; then
    echo "Warning: ExifTool scan failed or no supported image files found"
    echo "Contents of temp file:"
    cat "$TEMP_JSON" || true
    exit 1
fi

# Check if we got any data
if [ ! -s "$TEMP_JSON" ] || [ "$(cat "$TEMP_JSON")" = "[]" ]; then
    echo "No supported image files found in test directories"
    echo "Checked directories:"
    echo "  - $PROJECT_ROOT/test-images"
    echo "  - $PROJECT_ROOT/third-party/exiftool/t/images"
    exit 1
fi

echo "Processing supported image files and generating snapshots..."

# Count files for progress
TOTAL_FILES=$(jq length "$TEMP_JSON")
echo "Found $TOTAL_FILES supported image files"

if [ "$TOTAL_FILES" -eq 0 ]; then
    echo "No supported image files to process"
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
    
    # Skip if file already exists and we're not forcing regeneration
    if [ "$FORCE_REGENERATE" = false ] && [ -f "$SNAPSHOT_FILE" ]; then
        # echo "Skipping existing: $SNAPSHOT_FILE"
        continue
    fi
    
    # Filter to only supported tags and normalize paths before saving as snapshot
    # supported_tags.json now contains full group:tag format (e.g., "EXIF:Make")
    # Match the full key directly against the supported list
    echo "$file_data" | jq --argjson tags "$SUPPORTED_TAGS" --arg project_root "$PROJECT_ROOT" \
        'with_entries(select(
            .key as $k | 
            if $k == "SourceFile" then true
            else ($tags | index($k))
            end
        )) |
        # Normalize SourceFile to relative path
        if .SourceFile then
            .SourceFile = (.SourceFile | 
                if startswith($project_root) then
                    .[$project_root | length + 1:]
                else
                    .
                end)
        else . end |
        # Normalize File:Directory to relative path  
        if .["File:Directory"] then
            .["File:Directory"] = (.["File:Directory"] |
                if startswith($project_root) then
                    .[$project_root | length + 1:]
                else
                    .
                end)
        else . end' \
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
echo "  - Total snapshots in directory: $SNAPSHOT_COUNT"
if [ "$FORCE_REGENERATE" = true ]; then
    echo "  - All snapshots were regenerated (--force mode)"
else
    echo "  - Only missing snapshots were generated"
    echo "  - To regenerate all snapshots, run: $0 --force"
fi
echo "  - Supported tags: $(echo "$SUPPORTED_TAGS" | jq -r '.[]' | tr '\n' ' ')"
echo ""
echo "To run compatibility tests:"
echo "  make compat-test"