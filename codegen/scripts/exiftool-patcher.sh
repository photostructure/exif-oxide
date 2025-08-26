#!/bin/bash

#------------------------------------------------------------------------------
# File:         exiftool_patcher.sh
#
# Description:  Apply universal global patching to all ExifTool modules
#
# Usage:        ./exiftool_patcher.sh
#
# Notes:        Patches all modules to make variables accessible for symbol
#               table introspection. Safe to run multiple times.
#------------------------------------------------------------------------------

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEGEN_DIR="$(dirname "$SCRIPT_DIR")"
cd "$CODEGEN_DIR"

MARKER="# EXIF-OXIDE PATCHED"

# Set up local::lib environment for perltidy access (inherited by all subshells)
eval $(perl -I "$HOME/perl5/lib/perl5/" -Mlocal::lib)

EXIFTOOL_BASE="$(cd ../third-party/exiftool && pwd)"
PATCHER="./scripts/exiftool-patcher.pl"
CONFIG_FILE="../config/exiftool_modules.json"

if [ ! -f "$PATCHER" ]; then
  echo "Error: $PATCHER not found" >&2
  exit 1
fi

if [ ! -d "$EXIFTOOL_BASE" ]; then
  echo "Error: ExifTool base directory not found: $EXIFTOOL_BASE" >&2
  exit 1
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: Module configuration not found: $CONFIG_FILE" >&2
  exit 1
fi

# Check for jq
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required but not installed" >&2
  exit 1
fi

# Load all modules from shared configuration in one go
readarray -t ALL_MODULES < <(jq -r '.modules | to_entries | map(.value[]) | .[]' "$CONFIG_FILE")

echo "Checking which of ${#ALL_MODULES[@]} modules need patching..." >&2

# First pass: quickly identify modules that need patching
MODULES_TO_PATCH=()
for module in "${ALL_MODULES[@]}"; do
  module_path="${EXIFTOOL_BASE}/${module}"
  if [ -f "$module_path" ] && ! grep -q "$MARKER" "$module_path" 2>/dev/null; then
    MODULES_TO_PATCH+=("$module_path")
  fi
done

if [ ${#MODULES_TO_PATCH[@]} -eq 0 ]; then
  echo "✅ All modules already patched and formatted" >&2
  exit 0
fi

echo "📝 Processing ${#MODULES_TO_PATCH[@]} modules that need conversion..." >&2

NPROC=12

# Second pass: run perltidy on files that need patching (disabled)
# if command -v perltidy >/dev/null 2>&1; then
#   echo "🎨 Running perltidy on ${#MODULES_TO_PATCH[@]} modules..."
#   printf '%s\n' "${MODULES_TO_PATCH[@]}" | xargs -P $NPROC -n 3 perltidy -b
#   # Clean up perltidy backup files
#   find "$EXIFTOOL_BASE" -name '*.bak' -delete
# else
#   echo "⚠️  perltidy not found, skipping formatting"
# fi

# Third pass: run the patcher on the formatted files (without perltidy)
echo "🔧 Applying variable conversion to ${#MODULES_TO_PATCH[@]} modules..." >&2

printf '%s\n' "${MODULES_TO_PATCH[@]}" | xargs -P $NPROC -n 1 $PATCHER
for module_path in "${MODULES_TO_PATCH[@]}"; do
  # Add the marker comment to indicate patching
  echo -e "\n$MARKER" >>"$module_path"
done

echo "✅ Completed processing ${#MODULES_TO_PATCH[@]} modules" >&2
