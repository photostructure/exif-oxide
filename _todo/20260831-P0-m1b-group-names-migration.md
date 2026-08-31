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

## Phase A plan (2026-08-31, planner verified vs live Perl; key
## citations re-verified: ReadTask.ts:109/:170-174/:235-237/:243-251)

Verified ground truth: under `-json -G` every ExifTool key is
`Group:Name` (exiftool:2952) EXCEPT `SourceFile` (bare, :2682);
Error/Warning/ExifToolVersion/Geolocation* are group `ExifTool`
(ExifTool.pm:1226-1227); `-json` forces Duplicates=1 (exiftool:952) so
the same bare name appears under several groups; bare-mode winners come
from the internal Priority merge and are NOT recoverable from -G order
(verified: oly.jpg MeteringMode→MakerNotes but ExposureMode→EXIF, same
output order). Wrapper today: degroup only on exact `-G`
(ReadTask.ts:109); library-synthesized keys stay bare (SourceFile,
errors/warnings, zone/tz/tzSource, parsed GPS quartet,
invalidUtf8Bytes); tz/video heuristics read a last-wins degrouped view
(:216-224) so they WORK under -G (approximation either way);
errorsAndWarnings() misses ExifTool:Error (ErrorsAndWarnings.ts:21-29);
NEW BUG: version-preservation regex (ReadTask.ts:170-174) can't match
`"ExifTool:ExifToolVersion"` so 13.30→13.3 under -G.

The contract: prefixed keys = ExifTool's output verbatim (except the
GPS quartet's prefixed keys carry the library's validated sign-corrected
values; invalid GPS omits all GPS keys); bare keys = the library's own
namespace (SourceFile, errors, warnings, zone, tz, tzSource,
invalidUtf8Bytes, parsed GPS quartet). errors/warnings aggregate
stderr + JSON Error/Warning in EITHER shape. Heuristics documented as
approximations for multi-group names.

Tasks (ordered, one implementation agent): T1 errorsAndWarnings reads
both shapes (+ ReadRawTask.parse cast; WriteTask untouched); T2 version
regex `/"(?:ExifTool:)?ExifToolVersion".../`; T3 first-class
`groupNames: boolean` option (ExifToolOptions near :343-381, default
false, ReadTaskOptionFields, `-G` injection in ReadTask args, pin
back-compat: readArgs ["-G"] still degroups, `-G1` still does NOT);
T4 GPS/synthetic contract locked by parse()-harness tests (minimal code;
keep :243 guard); T5 typing — Layer B `tag(t, name)` helper (exact key
then degrouped fallback; two-arg overload if template-literal inference
fails) + Layer A `GroupedTags` via mapped types over the EXISTING
per-group interfaces (~50 lines in the mktags footer, NOT a second
22k-line Tags.ts; APP-group caveat) + `read()` overload for
`groupNames: true`; T6 docs (README contract section; fix the
`["-g"]` jsdoc example at ExifTool.ts:350 — lowercase -g produces
nested objects ReadTask mangles). Sizing: T1/T2 <1h each, T3+T4 half
day, T5 the long pole, T6 1h.

Back-compat proof: with groupNames unset and no -G, every change is
unreachable or provably identical (bare keys never contain ':'); the
existing 1,975-line ReadTask.spec + live ExifTool.spec runs are the
gate (`npm run preflight` zero snapshot edits). Disclosed delta for
EXISTING -G users: JSON-embedded errors start populating errors[]
(bugfix; changelog). Live differential spec: read 4 test files bare vs
groupNames, assert bare-key namespace, equal errors/warnings/zone,
degrouped superset, string ExifToolVersion.

Defaults taken (flagged, overridable): keep last-wins degroup collision
order (neither order reproduces the Priority merge); fix the
never-populated `zoneSource` (ExifToolVendoredTags.ts:39-46) in
passing; `readArgs` suffices for readRaw (no groupNames injection
there). Awaiting Matthew: semver, bare-GPS-quartet confirmation,
deprecation stance (see Open decisions).

## Progress log

### 2026-08-31

M1b started. Two read-only planning agents launched: Phase A plan
(exiftool-vendored.js -G coherence, additive) and Phase B plan
(PhotoStructure migration order, helper design, trap-by-trap fixes,
fixture-regen strategy). Phase A plan landed and is recorded above;
Phase B planner still running.
