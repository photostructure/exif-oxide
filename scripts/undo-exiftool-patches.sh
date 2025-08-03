#!/bin/bash

# Undo ExifTool patches
# This script reverts all patches applied to ExifTool modules

set -e

if ! command -v git >/dev/null 2>&1; then
  echo "⚠️  Warning: git not found, skipping ExifTool patch undo"
  echo "   Install with: sudo apt-get install git"
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. >/dev/null 2>&1 && pwd)"
cd "$ROOT"

git -C ./third-party/exiftool checkout -- lib/Image/ExifTool/*.pm
