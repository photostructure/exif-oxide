#!/bin/bash

# Undo ExifTool patches
# This script reverts all patches applied to ExifTool modules

set -e

if ! command -v git >/dev/null 2>&1; then
  echo "⚠️  Warning: git not found, skipping ExifTool patch undo" >&2
  echo "   Install with: sudo apt-get install git" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/../.. >/dev/null 2>&1 && pwd)"
cd "$ROOT"

# Note! We don't just revert all of third-party/exiftool because the
# exiftool-researcher may have written new docs in third-party/exiftool/docs
# that we don't want to lose.
git -C ./third-party/exiftool checkout --quiet -- lib/Image/ExifTool.pm lib/Image/ExifTool/*.pm lib/Image/ExifTool/*.pl
