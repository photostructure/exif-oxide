#!/bin/bash

# Format code: Rust, Perl, and JSON files
# This script formats all code in the project according to style guidelines

set -e

# Run all formatting tasks in parallel
cargo fmt --all &
find . -name "*.sh" -not -path "./third-party/*" -type f -print0 | xargs -0 shfmt -w -i 2 &
./scripts/fmt-perl.sh &
./scripts/fmt-json.sh &

# Wait for all background jobs to complete
wait
