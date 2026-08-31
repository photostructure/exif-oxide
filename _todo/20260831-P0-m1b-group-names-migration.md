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
there).

Phase A decisions (Matthew, 2026-08-31): **major bump — target
v38.0.0** (do not bump in the branch; release flow owns it);
**keep the bare GPS quartet**; **README recommends group-names mode as
the forward path** (bare mode documented but implicitly legacy);
implementation leaves ALL changes UNCOMMITTED in the working tree —
Matthew (2026-08-31, superseding the earlier branch-commits answer):
"don't make commits to exiftool-vendored without approval!" Every
commit there requires explicit per-instance approval; pushes likewise.

## Phase B plan (2026-08-31, planner verified anchors + probed real
## ExifTool; spot-checked: ReadRawTags.ts:126 choke point,
## ParsingSettings.ts:109 useMWG default true, MWG composites emit as
## Composite:* under -G family 0)

STAGED LANDING (each stage's fixture diff answers one question):
- Stage 0: drop `-use MWG` while still bare-name; fixture regen diff =
  the isolated MWG-dependency measurement Matthew ratifies.
- Stage 1: flip -G on + a ~50-line degroup shim at the ReadRawTags
  boundary (prefixed→bare via the precedence table); diff = pure
  merge-policy delta, zero key churn.
- Stage 2: dual-keyed object; consumers/tier lists migrate
  module-by-module, each green.
- Stage 3: persisted-name normalizers, settings colon handling,
  sidecar-overlay eviction.
- Stage 4: remove shim, flip Tags to the grouped type (tsc catches
  stragglers), final regen = pure key renames.
Single-flag-flip alternative rejected (un-reviewable combined diff).
Stage 0 can land against v37.2.0 now; Stage 1+ needs the Phase A
release (or npm link/tarball pin — flag for Matthew).

Helper (task 1): plain data + functions in new
src/core/tags/TagGroupPrecedence.ts — bareName(), pluckTag(tags,name)
(exact bare key → explicit chain → DefaultGroupOrder), omitByBareName().
~23-row table with per-row survey citations + spec pinned to a
checked-in survey fixture. Tags-CLASS REJECTED on merits: worker-IPC
JSON revivers (ShimDelegation.ts:86-97), FileCache consumers, {...t}
spread idioms (ReadTags.ts:157), DESIGN-PRINCIPLES rule 4. NOTE:
pluck/pluckDeep already exist in fe/Object.ts:397-433 — don't shadow.

Precedence-table highlights (planner re-ran survey on current corpus:
23 names, 221/381 files — pin fixture to a named corpus revision):
exposure cluster EXIF-first; lens Composite→XMP→MakerNotes; LensInfo
EXIF→XMP (keeps lensId "mli" stable); ImageWidth/Height
File→QuickTime→EXIF; dates handled by capturedAt tiers (helper chain
QuickTime→EXIF→XMP); GPS stays bare-synthetic; PreviewImage stays bare
for -b extraction (presence via suffix match); Rating XMP-first
(Matthew decision); Copyright cross-name chain (MWG-equivalent,
Matthew decision).

Traps: (a) sidecar overlay → BARE-NAME EVICTION in the merge loop
(delete same-bareName keys before assignFields, recording into
result.original) — reproduces today's semantics with one rule and
removes the sidecar dimension from pluckTag; IgnoredSidecarFields via
omitByBareName. (b) capturedAtSrcDetail stores bareName(key)
(CapturedAt.ts:662); UIDs keep bare synonym vocabulary + pluckTag picks
→ byte-stable IDs (only LensInfo intersects collisions; EXIF-first row
matches bare winner); v1.1 ':' decoder never meets a prefixed key; no
DB migration. (c) TagNameArraySetting subclass:
splitStringArrayKeepingColons + bare→canonical rename map in
toValidValues; TOML arrays parse natively; autoUpgradeSettings persists
normalized names; unknown custom names pass through (release note).
(d) heuristics via bareName(): isUtcTagName, GeoTagger
startsWith("Geolocation"), History (written ResourceEvent.Changed
vocabulary STAYS BARE — stable sidecar file format — translated on
read-back); PLUS TWO NEW: coerceTagTypes (ReadRawTags.ts:216-245,
TagMetadata.json is bare-keyed — every lookup misses under -G) and
srcHasIccProfile_ (HeifConvert.ts:243-261; key becomes
ICC_Profile:ICC_Profile — P3-desaturation regression, invisible to
fixtures, needs targeted test). Audit: AssetRevision.field vocabulary;
`as any` casts escape the type net — grep as checklist.

MWG evidence (probed): composites emit as Composite:* under -G;
XMP mwg-rs Regions extract WITHOUT -use MWG (WhoTagger unaffected);
consumed today: Rating (MWG's pure XMP mirror — implements the
documented "prefer XMP" intent), Keywords (already unioned; drop loses
only IPTC-truncation reconciliation), Copyright (needs replacement
chain EXIF:Copyright→XMP:Rights→IPTC:CopyrightNotice), geo names
(synonyms already enumerated). RECOMMEND DROP, ratified by the Stage-0
fixture diff; keep useMWG as a setting (Perl users), document exif-oxide
won't honor it.

Verification: 140 assertEqlsPrior fixtures; keystone
exif-tags-noinfer.json (231 entries) directly asserts
cameraId/imageId/lensId byte-stability. Workflow: test.sh --force-fix
(sources ~/.psenv) in core then library; review git diff examples/json/
with a keys-changed vs values-changed classifier script; runner
auto-re-runs rewritten specs (self-verifying regen). NO test-running CI
exists in PhotoStructure — the gate is local preflight; fixture-regen
commits must be reviewed, not rubber-stamped.

Phase A contract asks (from Phase B): first-class groupNames option
(also flips HeifConvert.ts:246); errorsAndWarnings reads prefixed;
pinned synthetic-key contract INCLUDING invalidUtf8Bytes inner keys =
emitted prefixed keys; a distinct grouped-keys Tags type (without it
Phase B loses its compile-time net — push back hard). Geolocation keys:
consumed as ExifTool:Geolocation* (consistent with the approved Phase A
contract; taken as default).

Defaults taken (flagged): plain-data helper (both planners concur);
includedPreviewTags strips groups before preview WRITES (status quo
write locations); Geolocation consumed prefixed.

Phase B decisions (Matthew, 2026-08-31): Stage 0 runs NOW as an
uncommitted experiment IN A DEDICATED WORKTREE (Matthew: "use a new
photostructure worktree please? don't play in ~/src/photostructure");
nothing committed anywhere in that repo — global ask-first rule
applies; Rating chain XMP → EXIF → MakerNotes;
Copyright adopts the full MWG-equivalent chain (EXIF:Copyright →
XMP:Rights → IPTC:CopyrightNotice) via a small extractCopyright();
MakerNotes date tags enter the capturedAt SECONDARY tier at Stage 2.
Still open: MWG-drop ratification (awaits the Stage-0 diff),
release-notes stance for custom bare-name settings.

Stage 0 design refinement: TWO regen passes for maximal evidence —
pass 1 flips useMWG only (diff = total MWG dependency), pass 2 adds the
Rating/Copyright replacement chains (diff = uncovered residue).

## Stage 0 results (2026-08-31, measured in worktree
## photostructure/.claude/worktrees/m1b-stage0, NOTHING committed;
## artifacts in the session scratchpad; Codex-vetted, 25 challenges)

Baseline: clean (one non-semantic trailing-newline caveat on 4
migrations-post fixtures). Pass 1 (useMWG=false only): 18/140 fixtures
change. MWG's TOTAL corpus dependency decomposes as:
- (A) DOMINANT: subsec/offset enrichment of bare date keys (120/207
  captured-at entries lose subseconds; zero zone/wall-clock changes).
  ROOT CAUSE IS OURS, NOT MWG's: capturedAtFromTags candidates never
  populate capturedAtPrecisionMs/Raw, so the precision tie-break
  compares undefined===undefined and DateTimeOriginal beats
  SubSecDateTimeOriginal alphabetically (CapturedAt.ts:627-651). MWG
  masked this defect; fixable in bare mode, recovers all 120 losses.
- (B) MWG strict mode suppressed "non-standard EXIF" in Panasonic RW2
  embedded JPEGs; dropping RESURRECTS maker notes on 6 RW2s (+1 DNG):
  serial numbers/lens fields appear → cameraId changes on 6, lensId on
  2+4, image hashes shift deterministically. New values are MORE
  correct (byte-identical to each RW2's JPG sibling) but re-key
  existing DB rows on rebuild — in-place migration behavior unassessed.
- (C) taumata.jpg loses Where State (IPTC Province-State was bridged
  only by MWG's composite) — one-line fix: add Province-State /
  Country-PrimaryLocationName to tagGeoSynonyms.
- (D) Rating/Keywords/Copyright/Description: ZERO corpus dependency
  (444-file dual-mode scan; Subject carries all Keywords values; no
  file has Rights/CopyrightNotice at all).
Pass 2 (extractCopyright chain wired + validated on synthetic files;
Rating chain is a NO-OP in bare mode — documented, defers to Stage 1/2):
residue delta vs pass 1 = ZERO across all 140 fixtures. imageId
byte-stable (231/231). 6 hard-coded spec assertions fail (subsec + one
read-side zone-attribution case on `writes AllDates` — written bytes
identical; decide attribution before editing the assertion).

RECOMMENDATION: ratify the drop CONDITIONALLY, bundled with: (1) the
capturedAt precision tie-break fix (own fixture pass), (2) the IPTC geo
synonyms, (3) a decision on Panasonic RW2 cameraId/lensId/hash
re-keying for existing libraries, (4) the zone-attribution decision.
Worktree kept in place for review; revert commands in the agent report.

## Progress log

### 2026-08-31

M1b started. Two read-only planning agents launched: Phase A plan
(exiftool-vendored.js -G coherence, additive) and Phase B plan
(PhotoStructure migration order, helper design, trap-by-trap fixes,
fixture-regen strategy). Phase A plan landed and is recorded above;
Phase B planner still running.
