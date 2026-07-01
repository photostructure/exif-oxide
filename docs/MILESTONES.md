# Milestones

Roadmap and scope for exif-oxide. This document exists because "what are we
actually building" needs one canonical answer — see [ARCHITECTURE.md](ARCHITECTURE.md)
for the *why* behind the technical decisions, and [TPP-GUIDE.md](TPP-GUIDE.md)
for how work gets planned and executed.

## Scope Decision: Read-Only, Indefinitely

**exif-oxide is read-only for the foreseeable future.** This was decided in a
strategic review on 2026-07-01, based on:

- ExifTool's writer is roughly half of ExifTool's codebase, and it is the
  only component that can corrupt a user's files. Porting it is a
  fundamentally different risk profile than porting readers, and it's
  deferred indefinitely.
- Write operations are delegated to real ExifTool. The project also
  maintains [exiftool-vendored.js](https://github.com/photostructure/exiftool-vendored.js),
  which PhotoStructure already ships and uses for writes today.
- exif-oxide never has to be *complete* before it's useful. Unported
  territory is served by falling back to real ExifTool, so adopting
  exif-oxide never regresses existing ExifTool functionality — it can only
  add speed where it has coverage.

## Tiered Architecture

exif-oxide is the native fast path for supported tags and formats. Real
ExifTool — today a vendored Perl process, potentially a Perl-in-WASM build
(e.g. the zeroperl-based ExifTool WASM ports) in the future — is
the permanent correctness fallback for the long tail of reads, and for
**all** writes. This isn't a temporary bridge to full parity; it's the
permanent shape of the system.

### Tier 1: PhotoStructure required tags (read)

The ~151 tags PhotoStructure needs (see `docs/required-tags.json`). Mostly
done. This is the tier that has to be right before exif-oxide can replace
the Perl ExifTool call in PhotoStructure's hot path.

### Tier 2: Full read support for PhotoStructure-relevant formats

Every format PhotoStructure encounters in the wild, including video
(QuickTime/MP4, RIFF/AVI). Broader than Tier 1's tag list — this is about
format coverage, not just the required-tags subset.

### Tier 3: Everything ExifTool reads

Aspirational. ExifTool reads 15,000+ tags across hundreds of formats;
exif-oxide will never need to match that exhaustively because Tier 3 gaps
are served by the ExifTool fallback in the meantime. Work here is
opportunistic, not scheduled.

### Tier 4: Write support

Deferred indefinitely. Served by real ExifTool. Revisit only if the
fallback strategy itself proves insufficient (e.g. performance or
deployment constraints that don't currently exist).

## Current Priorities

In order:

1. **ExifTool version catch-up (v13.43 → v13.59+) + upgrade runbook** —
   the vendored submodule is ~16 releases behind (verify via `$VERSION` in
   `third-party/exiftool/lib/Image/ExifTool.pm`, not `git describe` — see
   the catch-up TPP for why); codegen needs to be re-run against current
   ExifTool and a repeatable runbook written so future version bumps
   aren't a one-off archaeology project.
2. **Snapshot-oracle integrity fixes** — compatibility snapshots must only
   ever be generated from real ExifTool output, never hand-edited or
   generated from exif-oxide itself (that would make the oracle circular).
   Includes establishing a known-failures list for cases where exif-oxide
   intentionally diverges (see [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)
   allowed deviations).
3. **cargo-fuzz infrastructure** — fuzzing for the parser surface, given
   that exif-oxide reads untrusted, adversarial file formats.
4. **Composite tag bug fixes** — `Megapixels` sprintf formatting,
   `ShutterSpeed` fallback logic, `GPSPosition` sign handling.
5. **Video/QuickTime read support** — the Tier 2 format gap called out
   above (also tracked as "Milestone 18" in older docs/analysis files).
6. **napi-rs Node binding spike** — expose an
   `exiftool-vendored`-compatible API from exif-oxide, with per-tag
   fallback to real ExifTool for anything exif-oxide doesn't yet support.
   This is the integration path into PhotoStructure.

See `_todo/` for the TPPs covering items 1-3 and 6.

## Non-Goals

- **Write support of any kind** — see Tier 4 above.
- **Full ExifTool tag parity** — Tier 3 is aspirational, not scheduled.
- **Custom/user-defined tag configuration** — ExifTool's config system for
  user-defined tags is out of scope; see [ARCHITECTURE.md](ARCHITECTURE.md).
- **Novel parsing heuristics** — see [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md).
