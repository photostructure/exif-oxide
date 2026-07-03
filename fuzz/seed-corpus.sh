#!/bin/bash
# Seed the libFuzzer corpora from the project's real test-image corpora.
#
# Corpora are COPIED (never symlinked) because libFuzzer mutates corpus files
# in place — a symlink would corrupt the source test images. fuzz/corpus/ is
# gitignored; run this after a fresh checkout to (re)populate seeds. Re-running
# is safe: it overwrites the seeds it manages and leaves fuzzer-discovered
# inputs untouched.
#
# Usage: fuzz/seed-corpus.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
CORPUS_ROOT="$SCRIPT_DIR/corpus"

# Seed sources. exiftool/t/images is 25 years of deliberately-malformed
# regression samples — an excellent adversarial seed set.
SRC_TEST_IMAGES="$REPO_ROOT/test-images"
SRC_EXIFTOOL="$REPO_ROOT/third-party/exiftool/t/images"

# Skip pathologically large seeds: libFuzzer is far more effective with many
# small seeds than a few multi-megabyte ones (slow per-iteration mutation).
MAX_SEED_BYTES=$((5 * 1024 * 1024))

# copy_seeds <target> <case-insensitive-name-glob...>
# Flattens matches from both source trees into fuzz/corpus/<target>/, mangling
# path separators into the filename so files from different dirs never collide.
copy_seeds() {
  local target="$1"
  shift
  local dest="$CORPUS_ROOT/$target"
  mkdir -p "$dest"

  local find_args=()
  local first=1
  local pat
  for pat in "$@"; do
    if [[ $first -eq 1 ]]; then
      first=0
    else
      find_args+=(-o)
    fi
    find_args+=(-iname "$pat")
  done

  local count=0
  local src f rel mangled
  for src in "$SRC_TEST_IMAGES" "$SRC_EXIFTOOL"; do
    [[ -d "$src" ]] || continue
    while IFS= read -r -d '' f; do
      rel="${f#"$REPO_ROOT"/}"
      mangled="$(printf '%s' "$rel" | tr '/ ' '__')"
      cp -f "$f" "$dest/$mangled"
      count=$((count + 1))
    done < <(find "$src" -type f \( "${find_args[@]}" \) -size -"${MAX_SEED_BYTES}"c -print0)
  done
  printf '  %-16s %4d seeds -> %s\n' "$target" "$count" "corpus/$target/"
}

echo "Seeding fuzz corpora (copying, max ${MAX_SEED_BYTES} bytes/file)..."

# JPEG container: whole .jpg/.jpeg readers.
copy_seeds fuzz_jpeg '*.jpg' '*.jpeg'

# TIFF-based files are valid input to BOTH the TIFF extractors and the raw
# EXIF/IFD walker (they open with a TIFF header), so they seed both targets.
TIFF_GLOBS=('*.tif' '*.tiff' '*.dng' '*.nef' '*.cr2' '*.arw' '*.rw2' '*.orf' '*.3fr' '*.pef' '*.sr2' '*.srf')
copy_seeds fuzz_tiff "${TIFF_GLOBS[@]}"
copy_seeds fuzz_exif_ifd "${TIFF_GLOBS[@]}"

# PNG IHDR.
copy_seeds fuzz_png '*.png'

# ISO-BMFF (AVIF/HEIC).
copy_seeds fuzz_avif '*.avif' '*.heic' '*.heif'

# GIF screen descriptor.
copy_seeds fuzz_gif '*.gif'

# IPTC lives in JPEG APP13; seed from JPEGs so mutation can reach the 8BIM path.
copy_seeds fuzz_iptc '*.jpg' '*.jpeg'

# XMP: RDF/XML sidecars and loose XML.
copy_seeds fuzz_xmp '*.xmp' '*.xml'

# Whole-file dispatch: a broad, format-agnostic sample of everything small.
copy_seeds fuzz_whole_file \
  '*.jpg' '*.jpeg' '*.tif' '*.tiff' '*.png' '*.gif' '*.avif' '*.heic' \
  '*.webp' '*.jxl' '*.xmp' '*.rw2' '*.orf' '*.raf' '*.x3f' '*.mrw' '*.pef'

echo "Done."
