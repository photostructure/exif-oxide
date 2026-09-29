# TPP: M1b — group-names (-G) migration

Umbrella: `_todo/20260830-P0-oxide-cutover-program.md` (M1b milestone,
scoped 2026-08-30 from the corpus collision survey + PhotoStructure
consumption inventory recorded there). Work happens in TWO OTHER REPOS:

- `../exiftool-vendored.js` (public npm library; Phase A shipped in
  v38.0.0 on 2026-09-06, latest 38.3.0)
- `../photostructure` — Phase B work happens ONLY in the worktree
  `~/src/worktrees/m1b-group-names` on branch `feat/m1b-group-names`
  (created off local main 2026-09-28)

**Authorization boundary** (Matthew, 2026-09-28, supersedes the
per-commit pause for Phase B): commit each vetted step on
`feat/m1b-group-names` after a `/second-opinion` pass, including the
staging those commits need; NEVER touch photostructure main or its main
working tree, NEVER push — Matthew reviews and merges the branch. Every
other git-mutating command in photostructure, and every git-mutating
command in exiftool-vendored.js, still needs explicit per-instance
approval. Read their CLAUDE.md / AGENTS.md before
any change.

## Goal and sequencing

PhotoStructure switches from bare tag names to `-G` group-prefixed
names, verified against the REAL Perl ExifTool before any engine swap
(the exif-oxide transport swap is M7). Order:

1. **Phase A — exiftool-vendored.js `-G` coherence** (shipped; below).
2. **Phase B — PhotoStructure migration**: PhotoStructure owns
   cross-group precedence (a groupless lookup helper with an explicit
   per-name precedence table), renames its tier lists and literal tag
   accesses, and regenerates its `assertEqlsPrior` fixtures with the git
   diff as the primary review artifact.

## Decisions

- RESOLVED: exif-oxide emulates only -G output; Error/Warning always
  `ExifTool:`-prefixed (M1a decision 3). panic=unwind approved
  (Matthew, 2026-08-31).
- RESOLVED (Matthew, 2026-09-29): drop `-use MWG` for READS ONLY;
  writes keep honoring the `useMWG` setting. Existing libraries get ONE
  full metadata re-read when the Phase B branch ships, not one per
  stage.
- DEFAULT TAKEN (not ruled by Matthew): plain data + helper rather than
  a Tags class — both Phase B planners concurred (a class needs
  hydrate/dehydrate at the worker-IPC boundary, breaks `{...t}` spreads
  and FileCache consumers).

## Phase A — shipped in exiftool-vendored v38.0.0 (2026-09-06)

`41945f3..2c76cf3`: errorsAndWarnings() reads both key shapes;
ExifToolVersion stays a string under -G; zoneSource is populated; a
first-class `groupNames` option with `GroupedTags` typing. The contract
— prefixed keys are ExifTool's output verbatim except the GPS quartet,
which carries the library's validated values; bare keys are the
library's namespace (SourceFile, errors, warnings, zone, tz, tzSource,
invalidUtf8Bytes, parsed GPS quartet) — is documented in the
exiftool-vendored README and the `ExifToolOptions.groupNames` TypeDoc.
Settled in review: `readArgs: ["-G"]` behaves like `groupNames: true`
at runtime but without the GroupedTags return type.

## Phase B — PhotoStructure branch `feat/m1b-group-names`

The working plan (Stages 0-4 and ship), evidence, decisions, and
fixture-regeneration tribal knowledge live in the branch's TPP,
`_active/20260929-m1b-group-names.md` (worktree
`~/src/worktrees/m1b-group-names`). The 2026-08-31 Phase B plan that
used to live here is superseded by it; the 2026-09-29 corpus survey
corrected several of that plan's precedence chains.

### Stage 0 re-measured 2026-09-29 (main `e93524b49`)

What changed vs the lost 2026-08-31 measurement:

- (A) capture-time subsecond loss confirmed; the cause is that
  `_capturedAtFromTags` candidates carry no precision or raw value, so
  every ranking key ties and `DateTimeOriginal` wins BY POSITION in
  `CapturedAtTagsPrimary` (not alphabetically). A committed spec pinned
  the defect. Visible harm: burst frames 0.945 s apart merge into one
  asset.
- (B) RW2 re-keying confirmed and measured: `cameraId`/`lensId` and
  pHash/rHash/wHash change on the 6 Panasonic RW2s (hashes because the
  loader now picks the embedded `ThumbnailImage`); `imageId`,
  `imageDataHash`, `sha` do not. An upgrade experiment updated rows in
  place (same ids, no regrouping, no tag changes) for RW2s with JPG
  siblings.
- (C) geo: MWG also bridged IPTC `Sub-location`, not just
  `Province-State`/`Country-PrimaryLocationName`.
- NEW: dropping MWG for WRITES breaks Apple Photos sidecar keywords
  (land only in `XMP-pdf:Keywords`) and drops `AllDates` offsets.
- The 2026-08-31 "writes AllDates zone attribution" failure does not
  reproduce on current main.

## Progress log

### 2026-09-29

Stage 0 re-measured (Codex-vetted; summary above). Matthew decided:
MWG off for reads only; one full re-read at ship. The branch TPP
`_active/20260929-m1b-group-names.md` carries the Stage 0-4 plan and
Tribal Knowledge for fixture regeneration (the `--force-fix` pass skips
the `--small`/`--full` sync suites where the RW2 files live; library web
specs need the `public/app` bundle; generated settings docs regenerate
even in read mode and already drift on main — regenerate once at ship).

Stage 0 COMPLETE on the branch (each commit Codex second-opinion LAND):
`f3fd9936d` capture-time tie-break; `3adddfd26` IPTC geo synonyms and the
branch TPP; `aef56b8a4` Copyright resolved per file (so a sidecar's XMP
Rights still overrides the main file's EXIF Copyright, as MWG's per-file
composite did); `8e38fd870` + `365b2b790` separate reader/writer ExifTool
pools; `5630f1af6` `resolveKeywords()`; `4d76e0f6e` reads drop MWG
(breaking change for the `useMWG` setting's read semantics).

Discoveries that changed the plan:
- ExifTool never unloads MWG: after one `-use MWG` command, every later
  command on that stay_open process gets MWG composites and strict mode.
  A per-read `useMWG: false` on a shared pool is not MWG-free, so
  Matthew chose separate reader/writer pools — the same split exif-oxide
  needs (reads to exif-oxide, writes stay on Perl). exiftool-vendored's
  per-call `useMWG` has this gap for every consumer (flagged to Matthew as
  a separate task).
- Writes keep MWG and reads don't, so round trips must hold for "written
  with MWG, read without": MWG stores sidecar keywords joined in
  XMP-pdf:Keywords, which bare-mode reads return (fixed by
  `resolveKeywords`). No other PhotoStructure-written field changed.
- Must land before ship: a third-party file whose only capture date is
  XMP-photoshop:DateCreated loses it without MWG; needs group names.

NEXT: Stage 1 (`groupNames: true` + degroup shim) is in progress.

### 2026-09-28

State re-verified live after a three-week gap:

- exiftool-vendored: Phase A (`41945f3..2c76cf3`) shipped in v38.0.0
  (2026-09-06); releases continued to 38.3.0 (2026-09-28). Decision (a)
  "release v38" is closed.
- photostructure main pins `exiftool-vendored` 38.2.0, so `groupNames`
  is available; its only use in PhotoStructure source is one spec
  (`src/core/tags/ExifTool.spec.ts:723`), which writes capture time to
  MIE/EXV/JPEG files and reads them back with `groupNames: true` to
  check group-prefixed keys such as `MIE:DateTimeOriginal`.
- The 2026-08-31 Stage 0 worktree and diff are lost.
- None of the MWG ratification trio has landed, and main moved under
  two of them: `useMWG` still defaults true
  (`src/core/settings/ParsingSettings.ts:115`); `tagGeoSynonyms`
  (`src/core/settings/TaggingSettings.ts:333`) was rewritten around
  `applePhotos*` and `LocationShown/LocationCreated.*` synonyms and still
  lacks `Province-State` and `Country-PrimaryLocationName`; CapturedAt
  lives at `src/core/tags/CapturedAt.ts` and `517376f3a` (2026-09-18,
  "rank capture dates by provenance") reworked its ranking, so the
  Stage 0 tie-break diagnosis (cited as CapturedAt.ts:627-651) must be
  re-derived from current code.

Matthew (2026-09-28): "push this through", with `/second-opinion`
before every commit; branch-commit authorization recorded at the top of
this TPP. Worktree `~/src/worktrees/m1b-group-names` created on
`feat/m1b-group-names` off local main `e93524b49`.

### 2026-09-05

Phase A LANDED locally in ../exiftool-vendored.js (NOT pushed): five
reviewed commits, each approved by Matthew before committing —
`41945f3` fix(ErrorsAndWarnings) both key shapes; `b6dafe8`
fix(ReadTask) ExifToolVersion string under -G (+ regression test added
after Matthew's Codex finding R240-A); `4fa6506` fix(ReadTask) populate
zoneSource; `02e33ee` feat(GroupedTags) groupNames option + contract +
typing (full suite 1938 passing at commit time); `2c76cf3` docs.
Design point settled in review: `readArgs: ["-G"]` behaves like
`groupNames: true` at runtime (all parsing keys off the final args via
`degroup`; the option is consulted only for -G injection) but without
the GroupedTags return type; explicit `groupNames: false` does not
strip a caller-supplied -G — documented in the option TypeDoc.
NEXT: Matthew pushes/releases v38 when ready; Phase B Stage 1+ blocks
on that release (or an npm link/tarball pin). Stage 0 MWG ratification
decisions still open (tie-break fix, geo synonyms, RW2 re-keying).

### 2026-08-31

M1b started. Two read-only planning agents launched: Phase A plan
(exiftool-vendored.js -G coherence, additive) and Phase B plan
(PhotoStructure migration order, helper design, trap-by-trap fixes,
fixture-regen strategy). Phase A plan landed and is recorded above;
Phase B planner still running.
