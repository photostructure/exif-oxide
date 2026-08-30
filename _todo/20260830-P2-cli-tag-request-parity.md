# TPP: CLI tag-request parity gaps (deferred from the glob fix)

## Summary

While fixing wildcard numeric-tag matching (2026-08-30, Codex-reviewed),
five real divergences from ExifTool's tag-request handling were confirmed
empirically against vendored 13.59 and deferred. Each needs a breaking test
first (docs/TDD.md), and ExifTool's `SetFoundTags`
(`third-party/exiftool/lib/Image/ExifTool.pm:5348-5401`) is the reference.

## Current phase

- [x] Research & Planning
- [ ] Write breaking tests
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Confirmed gaps

1. **Bare `File*` requests skip EXIF parsing.** `is_file_group_only()`
   treats a `File*` *name* pattern as a File-group-only request, so
   `exif-oxide "-File*#"` omits `EXIF:FileSource` that
   `exiftool "-File*#"` returns. Related: `src/formats/mod.rs` stores the
   PrintConv string in both `value` and `print` for `FilePermissions`, so
   its `#` form is wrong independently. Perf note: the fast path exists on
   purpose; the fix must key on the *group* portion, not the name.
2. **Family-1 group requests unsupported.** `-ExifIFD:FNum?er#` matches in
   ExifTool (group1) and returns nothing here — the matcher only sees
   family-0 groups.
3. **Numeric selectors lose request order.** ExifTool honors the later
   request: `-Duration "-*Duration*#"` keeps the PrintConv string,
   `"-*Duration*#" -Duration` yields the number. Ours is an unordered
   `HashSet` in `FilterOptions`; needs an ordered request list.
4. **`-all#` and `-*#` are broken** (`-all#` returns nothing; `-*#` hits a
   length guard in the CLI arg parser).
5. **No illegal-character sterilization.** ExifTool strips characters
   outside `[-\w*?:]` from requests (`-*Duration*.#` works there). Must
   happen after the group split, or it eats the `:` that `EXIF:*` needs.

## Constraints

- Every fix verified against vendored `third-party/exiftool/exiftool`, not
  intuition; record the probe commands in the tests.
- `src/types/metadata.rs::request_matches_tag` is the single entry point
  added by the glob fix — extend it rather than adding parallel matchers.
