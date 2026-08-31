# TPP: M1b — group-names (-G) migration

Umbrella: `_todo/20260830-P0-oxide-cutover-program.md` (M1b milestone,
scoped 2026-08-30 from the corpus collision survey + PhotoStructure
consumption inventory recorded there). Work happens in TWO OTHER REPOS:

- `../exiftool-vendored.js` (public npm library, v37.2.0, clean tree)
- `../photostructure` (branch main)

**Authorization boundary: unlike exif-oxide, NEITHER repo has standing
commit/push authorization — always ask Matthew before `git commit` or
`git push` there.** Read their CLAUDE.md / AGENTS.md before any change.

## Goal and sequencing

PhotoStructure switches from bare tag names to `-G` group-prefixed
names, verified against the REAL Perl ExifTool before any engine swap
(the exif-oxide transport swap is M7). Order:

1. **Phase A — exiftool-vendored.js `-G` coherence (foundation).**
   Today `-G` produces a mixed keyspace (raw tags prefixed; parsed GPS,
   `zone`/`tz`/`tzSource`, `SourceFile`, `errors` bare; both
   `GPSLatitude` and `EXIF:GPSLatitude` present via ReadTask.ts:240-285)
   and the flat `Tags` interface type-checks but reads `undefined`.
   errorsAndWarnings() reads only bare `t.Error`/`t.Warning`
   (ErrorsAndWarnings.ts:22-29) and must also read
   `ExifTool:Error`/`ExifTool:Warning` (real ExifTool emits prefixed
   under -G, exiftool:2949; exif-oxide always will, per M1a decision 3).
   MUST BE ADDITIVE: bare mode stays the default for the library's
   other consumers.
2. **Phase B — PhotoStructure migration.** Groupless lookup helper with
   an explicit per-field precedence table (survey: only 20 names ever
   collide with differing values; winners regular — EXIF for the
   exposure cluster, Composite for lens aggregates, container-type for
   CreateDate/ModifyDate); rename ~30 list constants + ~40 files of
   literal access; regen the 140 assertEqlsPrior fixtures with the git
   diff as the primary review artifact.

## The four traps (from the verified inventory; full citations in the
umbrella TPP's M1b entry)

- (a) Sidecar overlay merge (`ReadRawMergedTags.ts:189-209`): under -G,
  sidecar `XMP:Rating` stops colliding with file `EXIF:Rating` —
  sidecar-wins evaporates. The one place needing groupless resolution.
- (b) Persisted names: `capturedAtSrcDetail` + `cameraId`/`imageId`/
  `lensId` (ExifUid.ts:119-148; v1.1 decoder splits on ':'). Normalize
  to bare before hashing/storing to avoid a DB migration.
- (c) `StringArraySetting` splits on ':' (POSIX path.delimiter) for all
  tag-list settings except excludedExifTags → adopt
  splitStringArrayKeepingColons everywhere + user-config migration.
- (d) Heuristics: `isUtcTagName` startsWith("GPS")
  (ExifTags.ts:109-111), GeoTagger startsWith("Geolocation") (:49),
  History.ts isFirstCharAZ.

## Decisions

- RESOLVED: exif-oxide emulates only -G output; Error/Warning always
  `ExifTool:`-prefixed (M1a decision 3). panic=unwind approved
  (Matthew, 2026-08-31).
- OPEN: drop `-use MWG`? (recommended; Phase B decides which tags the
  tier lists name, so it lands there.)
- OPEN: Tags class vs plain data + helper. Recommended: plain data +
  helper (a class needs hydrate/dehydrate at the worker-IPC boundary).
  Planners should evaluate on the merits.

## Progress log

### 2026-08-31

M1b started. Two read-only planning agents launched: Phase A plan
(exiftool-vendored.js -G coherence, additive) and Phase B plan
(PhotoStructure migration order, helper design, trap-by-trap fixes,
fixture-regen strategy). Plans land here when vetted.
