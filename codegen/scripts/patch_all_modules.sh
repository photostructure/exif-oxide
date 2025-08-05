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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEGEN_DIR="$(dirname "$SCRIPT_DIR")"
cd "$CODEGEN_DIR"

EXIFTOOL_BASE="$(cd ../third-party/exiftool && pwd)"
PATCHER="./scripts/patch_exiftool_modules_universal.pl"
CONFIG_FILE="../config/exiftool_modules.json"

if [ ! -f "$PATCHER" ]; then
  echo "Error: $PATCHER not found"
  exit 1
fi

if [ ! -d "$EXIFTOOL_BASE" ]; then
  echo "Error: ExifTool base directory not found: $EXIFTOOL_BASE"
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

# Load module lists from shared configuration
readarray -t CORE_MODULES < <(jq -r '.modules.core[]' "$CONFIG_FILE")
readarray -t MANUFACTURER_MODULES < <(jq -r '.modules.manufacturer[]' "$CONFIG_FILE")
readarray -t FORMAT_MODULES < <(jq -r '.modules.format[]' "$CONFIG_FILE")

# Function to patch a module
patch_module() {
  local module_relative_path="$1"
  local module_path="${EXIFTOOL_BASE}/${module_relative_path}"

  if [ ! -f "$module_path" ]; then
    echo "⚠️  Skipping $module_relative_path (not found at $module_path)"
    return
  fi

  perl "$PATCHER" "$module_path"
}

# Patch all modules using relative paths from JSON config
for module in "${CORE_MODULES[@]}"; do
  patch_module "$module" &
done

for module in "${MANUFACTURER_MODULES[@]}"; do
  patch_module "$module" &
done

for module in "${FORMAT_MODULES[@]}"; do
  patch_module "$module" &
done

wait
