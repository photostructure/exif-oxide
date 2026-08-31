# TPP: Spurious "Processor EXIF::BinaryData failed" warning on healthy files

## Summary

Healthy files emit `ExifTool:Warning: "Processor EXIF::BinaryData
failed: ... fallback"` in JSON output. Before M1a this was hidden (the
invented `Warning:Xxx` keys were gated behind `--warnings`); the M1a
decision to always emit the first warning (matching ExifTool) made it
consumer-visible. Real ExifTool emits no warning on the same files, so
this is a divergence a PhotoStructure operator would see on ordinary
photos. The always-emit behavior is correct; the underlying spurious
processor warning is the defect.

## Current phase

- [x] Research & Planning (discovered during M1a, 2026-08-31)
- [ ] Write breaking tests
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Approach

1. Find the warn site (rg "BinaryData failed" src/) and the files that
   trip it (run the corpus, count occurrences; compare against vendored
   exiftool which warns on none of them).
2. Determine why the processor "fails" then falls back successfully on
   files ExifTool handles silently — the fallback path evidently
   produces correct output, so either the primary path's precondition is
   wrong or the failure is an expected control-flow case being reported
   as a warning.
3. Breaking test first (docs/TDD.md): a healthy corpus file must produce
   zero ExifTool:Warning keys where vendored exiftool produces none.

## Constraints

- Do NOT re-gate warnings to hide it — decision 4 in
  `_todo/20260830-P1-stay-open-m1a.md` (always emit) stands.
- Trust ExifTool: if the primary processor genuinely cannot handle the
  structure, silence for expected fallbacks must match how ExifTool
  treats the same structure (it processes without warning).
