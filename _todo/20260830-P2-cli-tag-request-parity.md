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

## Progress (2026-08-30)

Items 2, 4, 5 landed (`74229584`, worktree agent + Codex review). Items 1
and 3 are reworking onto that commit in their worktrees (see the management
TPP `20260830-P0-oxide-cutover-program.md` for merge mechanics).
Item 3 ground truth: **first-match-wins** (SetFoundTags appends per request
ExifTool.pm:5433-5436; JSON writer noDups exiftool:2947-2953).

## Deferred findings from the 2026-08-30 fixes (all probe-confirmed)

1. Families 2+, `id-`, `Copy0`, `FileN:` request forms unsupported —
   `TagEntry` models only families 0/1, so `-Image:FNumber` (family 2,
   matches in ExifTool) finds nothing.
2. Whole-number float serialization emits `4.0` where ExifTool prints `4`
   (every `-Tag#`) — being absorbed by the R905-D float-division fix
   tracked in the management TPP.
3. No `Invalid TAG name` / `Invalid group name` diagnostics channel.
4. `ExifTool:ExifToolVersion` suppressed for every filtered request
   (`src/formats/mod.rs` keys the P12 rule on `extract_all`); ExifTool
   includes it for `-*` / `-all#`.
5. `-System:FileName` cannot work: the File fast path filters before any
   `TagEntry` carrying `group1: "System"` exists.
6. Family-0-only filtering remains in the legacy-tag map
   (`src/formats/mod.rs`, keyed `Group0:Name`) and
   `src/compat/filtering.rs::apply_exiftool_filter` (harmless for the CLI
   today; `prepare_for_serialization` rebuilds from filtered entries).
7. `parse_exiftool_args` (src/main.rs) and `parse_exiftool_filters`
   (src/compat/filtering.rs) are near-duplicates that diverge on `-*` —
   DRY into one.
8. `-File:all` omits format-derived File tags (ImageWidth, EncodingProcess,
   YCbCrSubSampling, …) — inherent to the stat-only fast path; documented
   in tests so it is not mistaken for parity.
9. macOS: ExifTool emits FileInodeChangeDate everywhere but darwin
   (ExifTool.pm:2909, FileCreateDate rules :2953-2960); our full path
   diverges — untestable here, flagged in a code comment.
10. `EXIF:FileSource` renders `[3]` (undef-format array, no PrintConv)
    where ExifTool gives `"Digital Camera"` / `3`.
11. `-File*` misses MakerNotes tags whose NAMES match (Nikon
    FileInfoVersion, FileNumber).
12. R905-C corner: regexes in generated conversions run against
    `TagValue::Binary`'s display placeholder (`"[N bytes ...]"`), not the
    bytes — Perl regex-matches the raw scalar.

## Constraints

- Every fix verified against vendored `third-party/exiftool/exiftool`, not
  intuition; record the probe commands in the tests.
- `src/types/metadata.rs::request_matches_tag` is the single entry point
  added by the glob fix — extend it rather than adding parallel matchers.
