#!/usr/bin/env bash

# Run codegen against a patched temporary copy of ExifTool so that the
# third-party/exiftool submodule is never modified. Interrupted or failed runs
# leave no residue: the staged copy lives under mktemp and is removed on exit.

set -euo pipefail

# Make the local::lib Perl modules (PPI, JSON::XS) visible to the extractor,
# matching exiftool-patcher.sh.
eval "$(perl -I "$HOME/perl5/lib/perl5" -Mlocal::lib)"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
codegen_dir="$(cd "${script_dir}/.." && pwd)"
repo_root="$(cd "${codegen_dir}/.." && pwd)"

staged="$(mktemp -d -t exiftool-staged.XXXXXX)"
trap 'rm -rf "${staged}"' EXIT INT TERM

cp -a "${repo_root}/third-party/exiftool/lib" "${staged}/lib"
"${script_dir}/exiftool-patcher.sh" "${staged}"

cd "${codegen_dir}"
EXIFTOOL_BASE="${staged}" cargo run "$@"
