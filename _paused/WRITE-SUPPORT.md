# Paused: Write Support

**Dated**: 2026-07-01

## Decision

A strategic review on 2026-07-01 concluded exif-oxide is **read-only for the
foreseeable future**. Write support is deferred indefinitely, not merely
deprioritized.

## Rationale

- Writing is roughly half of ExifTool's own implementation (`WriteExif.pl`,
  format-specific writers, offset-fixup logic) - comparable in scope to
  everything exif-oxide has built so far for reading.
- The writer is the only component in this codebase's design space that can
  **corrupt a user's original file**. A reader bug produces a wrong tag
  value; a writer bug can destroy irreplaceable photos. That asymmetry
  changes the acceptable risk profile substantially.
- Real ExifTool already solves this correctly today. PhotoStructure delegates
  writes to real ExifTool via `exiftool-vendored` (see
  `$HOME/src/exiftool-vendored.js`) using a tiered fallback architecture.
  There is no product need blocking on exif-oxide gaining write support.

## What this means for this repo

- Do not start writer work without a new TPP and explicit sign-off.
- Do not let read-path refactors quietly grow write-shaped abstractions
  "for later" - YAGNI applies doubly here given the corruption risk.

## If this is ever revisited

Start with roundtrip-vs-real-ExifTool test infrastructure before writing a
single byte of writer code: read a file with exif-oxide, write it back
unchanged, and diff the result against what real ExifTool produces for the
same edit. That test harness is the prerequisite for trusting any writer
change, not an afterthought - build it first, then write the TPP for the
first real writer feature.
