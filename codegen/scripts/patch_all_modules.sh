#!/bin/bash

#------------------------------------------------------------------------------
# File:         patch_all_modules.sh
#
# Description:  Apply universal global patching to all ExifTool modules
#
# Usage:        ./patch_all_modules.sh
#
# Notes:        Patches all modules to make variables accessible for symbol
#               table introspection. Safe to run multiple times.
#------------------------------------------------------------------------------

set -e

EXIFTOOL_LIB="../../third-party/exiftool/lib/Image/ExifTool"
PATCHER="./patch_exiftool_modules_universal.pl"
CONFIG_FILE="../../config/exiftool_modules.json"

if [ ! -f "$PATCHER" ]; then
  echo "Error: $PATCHER not found"
  exit 1
fi

if [ ! -d "$EXIFTOOL_LIB" ]; then
  echo "Error: ExifTool lib directory not found: $EXIFTOOL_LIB"
  exit 1
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: Module configuration not found: $CONFIG_FILE"
  exit 1
fi

# Check for jq
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required but not installed"
  exit 1
fi

echo "🔧 Starting universal patching of ExifTool modules..."
echo

# Load module lists from shared configuration
readarray -t CORE_MODULES < <(jq -r '.modules.core[]' "$CONFIG_FILE")
readarray -t MANUFACTURER_MODULES < <(jq -r '.modules.manufacturer[]' "$CONFIG_FILE")
readarray -t FORMAT_MODULES < <(jq -r '.modules.format[]' "$CONFIG_FILE")

# Function to patch a module
patch_module() {
  local module="$1"
  local module_path="$EXIFTOOL_LIB/$module"

  if [ ! -f "$module_path" ]; then
    echo "⚠️  Skipping $module (not found)"
    return
  fi

  echo "🔄 Patching $module..."
  perl "$PATCHER" "$module_path"
  echo
}

# Patch all modules
echo "📦 Patching core modules..."
for module in "${CORE_MODULES[@]}"; do
  patch_module "$module"
done

echo "📸 Patching manufacturer modules..."
for module in "${MANUFACTURER_MODULES[@]}"; do
  patch_module "$module"
done

echo "📁 Patching format modules..."
for module in "${FORMAT_MODULES[@]}"; do
  patch_module "$module"
done

echo "✅ Universal patching complete!"
echo
echo "Next steps:"
echo "  1. Run 'perl ./auto_config_gen.pl ../../third-party/exiftool/lib/Image/ExifTool/ModuleName.pm' to generate configs"
echo "  2. Use 'make codegen' to generate code from configs"
echo "  3. Test with representative files"
echo
