#!/bin/bash

# Regenerate all tag_kit.json configs using auto_config_gen.pl
# This script extracts tag table configurations from ExifTool modules

set -e

# Get the directory containing this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEGEN_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔄 Regenerating tag_kit.json configs for prioritized modules..."

# Move to codegen directory to ensure relative paths work correctly
cd "$CODEGEN_DIR"

echo "📦 Running universal patching first..."
./scripts/patch_all_modules.sh

echo "🔍 Processing curated ExifTool modules..."

count=0
success=0

CORE_MODULES="$(jq -r '.modules.core[]' ../config/exiftool_modules.json | tr '\n' ' ')"
MANUFACTURER_MODULES="$(jq -r '.modules.manufacturer[]' ../config/exiftool_modules.json | tr '\n' ' ')"
FORMAT_MODULES="$(jq -r '.modules.format[]' ../config/exiftool_modules.json | tr '\n' ' ')"

for module_group in "Core: $CORE_MODULES" "Manufacturer: $MANUFACTURER_MODULES" "Format: $FORMAT_MODULES"; do
  group_name=$(echo "$module_group" | cut -d: -f1)
  modules=$(echo "$module_group" | cut -d: -f2-)
  echo "📁 Processing $group_name modules..."

  for module in $modules; do
    count=$((count + 1))
    module_name=$(basename "$module" .pm)
    module_path="../third-party/exiftool/lib/Image/ExifTool/$module"

    if [ ! -f "$module_path" ]; then
      printf "  %-20s... ⚠️  (not found)\n" "$module_name"
      continue
    fi

    printf "  %-20s... " "$module_name"
    if perl scripts/auto_config_gen.pl "$module_name" >/dev/null 2>&1; then
      success=$((success + 1))
      echo "✅"
    else
      echo "⏭️  (no tag tables found)"
    fi
  done
done

echo "🧹 Cleaning up ExifTool patches..."
../scripts/undo-exiftool-patches.sh

echo "✅ Regenerated $success/$count tag_kit.json configs"
echo "💡 Use 'git diff' to review generated changes"
