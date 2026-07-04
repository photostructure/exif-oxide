# TPP: QuickTime/MOV Video Read Support

## Summary

**Problem**: Reading any QuickTime container video yields ONLY `File:` group
tags (`cargo run -- test-images/apple/IMG_3755.MOV` → 10 File: tags plus
`"MOV-based format MOV not yet supported"`, verified 2026-07-03). All 17
supported `QuickTime:*` tags plus the video-driven Composites (Rotation,
GPSAltitudeRef, GPSLatitude/Longitude/Altitude/Position, ImageSize,
Megapixels) are blocked because no atom walker exists for video containers.
**Why it matters**: program item #5; PhotoStructure needs video metadata
(Tier 2 in docs/MILESTONES.md). Bonus: 20 CR3 + 1 AVIF snapshots also expect
`QuickTime:*` tags, so the walker unlocks follow-on wins beyond the 5 MOVs.
**Solution**: streaming atom walker in `src/formats/quicktime.rs` consuming
the (already-generated, partly empty — see below) `QuickTime_pm` tables.
**Success test**: `cargo run --bin compare-with-exiftool -- test-images/apple/IMG_3755.MOV`
shows no `QuickTime:*` or `Composite:*` diffs; same for the other 4 MOVs.
**Key constraint**: Trust ExifTool — the metadata slice of QuickTime.pm
(10,771 lines) only. No A/V decoding, no ExtractEmbedded timed metadata.

## Current phase

- [x] Research & Planning (2026-07-03, all file:line verified against v13.59)
- [x] Write breaking tests (Task 0 DONE 2026-07-03: `tests/quicktime_video_tests.rs`;
      the 3 image-level tests are committed `#[ignore]`d — they were validated to
      fail on missing tags, not setup; **un-ignore them as Tasks 2-4 land**)
- [x] Task breakdown review
- [ ] Implementation (Tasks 1-2 DONE + review-gated + pushed — see logs below;
      **Task 3 IN FLIGHT** as of 2026-07-03 ~18:30: an opus subagent is
      implementing Keys/ilst (Make/Model/Software/CreationDate) + stsd
      CompressorName + hdlr HandlerDescription + Task-4 state capture
      (raw GPSCoordinates string, MatrixStructure, LensModel), including
      investigating how canon eos_500d.mov stores Model (`exiftool -v3`,
      not Apple mdta). If its report was lost to compaction: `git status`
      shows its edits; review-gate per the workflow note below, then commit.
      Tasks 4-5 not started)
- [ ] Review & Refinement
- [ ] Final Integration

**Workflow (per program-TPP orchestration guidance)**: one opus
implementation subagent per task → double-review gate (now formalized as
the `/coding:double-review` skill in `~/src/claude-code-skills`: codex
`codex exec --sandbox read-only "/review …" </dev/null &` + a Claude review
subagent, identically scoped and blind; vet every finding empirically
against the vendored exiftool/Perl before accept/veto; record verdicts
here) → one Conventional Commit per task, push without asking (this repo's
exception). When tags start passing, `make compat-test`'s stale-ratchet
demands removing exactly those tags from `config/compat_known_gaps.json` —
but QuickTime tags stay allowlisted while CR3 snapshots still miss them.

## Session log (2026-07-03, Task 2)

- **Walker landed**: `src/formats/quicktime.rs` — hand-written seek-based atom
  walker (Option A). Seeks past `mdat` (verified on the 37 MB eos_500d.mov,
  never `read_to_end`); handles 8-byte / size==1 64-bit / size==0-to-EOF
  headers; container routing moov→mvhd/trak, trak→tkhd/mdia, mdia→mdhd/hdlr/minf,
  minf→stbl→stsd; clearly-marked TODO arms for Task 3 (meta/keys/ilst, udta,
  stsd/CompressorName) and Task 4 (tkhd MatrixStructure + hdlr HandlerType, the
  latter already captured into walker state). Defensive: malformed/truncated
  atoms stop the walk, never panic (3 walker unit tests).
- **Conversions**: `src/implementations/quicktime.rs` — exact ports of
  `%timeInfo` RawConv 1970-epoch patch (QuickTime.pm:257) + ConvertUnixTime
  gmtime (ExifTool.pm:6784), `%durationInfo` + ConvertDuration (ExifTool.pm:6877
  — threshold is `< 30`, not 60), FixWrongFormat (QuickTime.pm:8872). Version-1
  mvhd/tkhd/mdhd int64u date/duration shift handled per the ExifTool Hook.
- **Codegen de-stub**: registered the date ValueConv + the two duration
  PrintConvs in `codegen/src/impl_registry/{valueconv,printconv}_registry.rs`;
  `make codegen` rewrote only `hash_e8`/`hash_52`/`hash_9b` bodies from
  placeholders to real calls (table files unchanged). No dangling
  `calc_sample_rate`/`unpack_lang` (dormant — not reached by any expression).
- **Priority**: TrackHeader = first-track-wins, MediaHeader/mvhd = last-wins,
  resolved inside the walker (one TagEntry per name). Proven on IMG_3755:
  TrackDuration=2.96 s (video) vs MediaDuration=0.00 s (metadata track);
  ImageWidth=1920 from the video track (audio 0×0 → FixWrongFormat undef → no tag).
- **Results**: all 5 MOV snapshots match 13/13 core tags (11 QuickTime binary
  tags + Composite:ImageSize/Megapixels, which fall out for free). Core tests
  `canon_eos500d_mov_core_tags_match_exiftool` /
  `exiftool_quicktime_mov_core_tags_match_exiftool` un-ignored + green; unit
  tests `timeinfo_1970_epoch_patch` / `convert_duration_seconds` implemented +
  green. apple test stays ignored (needs Task 3 Keys + Task 4 GPS/Rotation).
- **Compat gate green, 0 removals**: the QuickTime tags are also allowlisted for
  the 20 CR3 + HEIC snapshots (walker not wired for CR3 — out of scope), so they
  remain valid gaps there and the stale-ratchet correctly flags nothing. Working
  count stays 23/191 until CR3/HEIC route through the walker (follow-up).
- Dispatch wired at `src/formats/mod.rs` for file types MOV + MP4 only.
- **Post-review verdicts (double-review gate: codex + Claude subagent,
  2026-07-03)**: no blockers. Claude reviewer fuzzed 10 hand-built malicious
  MOVs (0 panics/hangs; recursion is structurally cycle-proof — the container
  state machine never re-enters a parent kind) and verified v1 layouts against
  vendored exiftool on a hand-built v1 file. ACCEPTED: (a) v1 boxes had zero
  committed coverage (all 5 corpus MOVs are v0) → in-memory v1 fixture test
  added in `src/formats/quicktime.rs`; (b) codex: the Perl RawConv also
  patches when the `QuickTimeUTC` option is set — option not exposed in
  exif-oxide, divergence documented in `patch_time_zero` docs instead of dead
  code. VETOED: codex's "generated table path passes ctx=None so durations
  can't convert" (that path has no runtime consumer — the walker calls core
  helpers directly, by documented design; Claude reviewer independently
  confirmed no double-conversion); codex's "u64::MAX dates render zero date
  vs Perl's `1900:01:00 00:00:00`" (Perl's output there is platform-dependent
  gmtime overflow on garbage input; we render the zero date, never panic —
  documented in `patch_time_zero` docs). Benign note: nested size-0 atoms are
  dispatched-then-terminated vs ExifTool's pure-terminator handling — verified
  no spurious tags on the Canon files that use them.

## Session log (2026-07-03, Tasks 0-1)

- **Task 1 landed**: `codegen/src/strategies/tag_kit.rs` now emits parallel
  `*_TAGS_BY_NAME: HashMap<&'static [u8], TagInfo>` maps for non-numeric
  keys, **gated to module QuickTime** (`emits_by_name_map`) to keep churn
  confined — generalizing to JPEG/H264/RIFF later is a one-line change.
  Keys are byte-string literals (`b"\xa9ART"`, `b"mvhd"`,
  `b"location.ISO6709"`): atom IDs are raw FourCC bytes the walker will
  read from the file, and `\xa9` isn't valid UTF-8. The field extractor
  lossy-decodes Perl output, so U+FFFD is mapped back to 0xA9 — verified
  unambiguous: 0xA9 is the only high byte at key position in all of
  QuickTime.pm (grep), and zero U+FFFD leaked into generated output.
  ExifTool `%specialTags` (NOTES/GROUPS/…) are filtered from by-name maps.
  Counts: Keys=81, ItemList=105, Main=38, UserData=189 entries.
- **Latent bug fixed along the way**: generated QuickTime `length($val)`
  ValueConvs exposed that PPI emits `length_i32(&TagValue)` but
  `src/core/string/measurement.rs` declared owned args — made
  `length_i32`/`length_string` generic `<T: Into<TagValue>>` (the
  `core::math::abs` convention).
- The QuickTime coderef conversions (ConvertISO6709, date/duration, GPS
  PrintConvs) sit in the by-name maps as `Expression("[Function: …]")`
  placeholders — Tasks 2/4 de-stub them via `impl_registry` +
  `src/implementations/quicktime.rs` (the ConvertXMPDate pattern).

## Required reading

- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md), [TDD.md](../docs/TDD.md),
  [ANTI-PATTERNS.md](../docs/ANTI-PATTERNS.md), [CODEGEN.md](../docs/CODEGEN.md)
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:9932-10692` — ProcessMOV
- `src/formats/avif.rs` — existing ISO-BMFF box parsing (the in-repo pattern)
- `_done/20260703-P1-xmp-value-conversion.md` — recent example of
  "port conversion exactly + wire at parse site + snapshot-pinned tests"

## Blocked tags (verified against snapshots 2026-07-03)

17 `QuickTime:*` entries in `config/supported_tags.json:188-204`:
CompressorName, CreateDate, CreationDate, Duration, HandlerDescription,
ImageHeight, ImageWidth, Make, MediaCreateDate, MediaDuration,
MediaModifyDate, Model, ModifyDate, Software, TrackCreateDate,
TrackDuration, TrackModifyDate.

Video-driven Composites (all in supported_tags.json): `Rotation` (appears
only in QuickTime-container snapshots: 5 MOV + 20 CR3), `GPSAltitudeRef`
(only-video today), and on IMG_3755.MOV: GPSLatitude/Longitude/Altitude/
Position, ImageSize, Megapixels, LensID. The program TPP's "22 blocked
tags" ≈ 17 QuickTime + video composites (the review also counted 4
`RIFF:*` tags — excluded here, see Out of scope).

**Oracle corpus** (verified): 13 video files under `test-images/` +
`third-party/exiftool/t/images/`; **5 MOV snapshots** exist in
`generated/exiftool-json/` (apple IMG_3755, canon eos_500d/eos_60d,
fujifilm gfx100rf, t/images QuickTime). `.mp4` is NOT in the snapshot
generator's SUPPORTED_EXTENSIONS (`tools/generate_exiftool_json.sh:23`), so
pixel_7_pro.mp4 / gopro jump.mp4 have no snapshots yet.

## Current state (updated 2026-07-03 post-Task-2)

Pre-implementation gaps (empty string-keyed tables, no dispatch arm, missing
`implementations::quicktime`) are all resolved by Tasks 1-2 — see session
logs. Still true and needed by Tasks 4-5:

- **Composites generated but not computable**:
  `src/generated/composite_tags.rs:2775-2914` has the QuickTime GPS splits
  + Rotation, but Rotation's ValueConv is the expression
  `Image::ExifTool::QuickTime::CalcRotation($self)` (needs a
  `COMPOSITE_FALLBACKS` entry in `src/core/composite_fallbacks.rs`), and
  the name-keyed `COMPOSITE_TAGS` HashMap loses the GPS-vs-Sony-vs-QuickTime
  same-name defs (`src/composite_tags/orchestration.rs:200-202`; program
  TPP follow-up (b)).
- `function_registry.rs:193-210` maps CalcSampleRate/UnpackLang into
  `implementations::quicktime`; both remain unimplemented and dormant (no
  expression reaches them) — leave them out unless codegen wires a call site.
- The compat gate is tag-level: QuickTime tags stay allowlisted while the 20
  CR3 snapshots (walker not routed for CR3) still miss them. Per-file truth =
  `compare-with-exiftool`.

## ExifTool mechanism (all verified in QuickTime.pm v13.59)

Atom walk for the supported slice (`SubDirectory` chain, cite when porting):

| Path | Table (line) | Yields |
|------|-------------|--------|
| top level | Main:548 | moov:678, mdat pseudo-tags, meta (Start=>4!):552 |
| moov | Movie:1201 | mvhd:1205, trak:1209, udta:1214, meta (NO Start):1218 |
| moov/mvhd | MovieHeader:1343 (binary) | CreateDate, ModifyDate, TimeScale (state!), Duration |
| moov/trak/tkhd | TrackHeader:1493 (binary) | TrackCreateDate/ModifyDate/Duration, MatrixStructure (idx 10), ImageWidth/Height (idx 19/20) |
| moov/trak/mdia | Media:7218 | mdhd:7223, hdlr:7227, minf:7231 |
| .../mdhd | MediaHeader:7239 (binary) | MediaCreateDate/ModifyDate, MediaTimeScale (state), MediaDuration |
| .../hdlr | Handler:8391 (binary) | HandlerType (state, idx 8), HandlerDescription (idx 24) |
| .../minf/stbl/stsd | SampleTable:7365 → ProcessSampleDesc:9629 → VisualSampleDesc:7585 | CompressorName (idx 25) when HandlerType == 'vide' |
| moov/meta/keys | Keys:6651 via ProcessKeys:9779 | key-name list (mdta) |
| moov/meta/ilst | ItemList:3481 | Make/Model/Software/CreationDate/GPSCoordinates/LensModel via Keys indirection |
| moov/udta | UserData:1585 | XMP_ atom:1711; Canon CNTH:2044; Pentax PENT:2283; Fuji condition:1921 |

Key procedural logic:

- **ProcessMOV:9932**: 8-byte header `(u32 size, [u8;4] tag)`; `size==0` →
  atom runs to EOF (Canon also uses it to terminate CNTH, tolerated at
  10039); `size==1` → 8-byte extended size follows (10058). Seek past
  `mdat` (record `mdat-size`/`mdat-offset` pseudo-tags, Main:690-701, only
  needed for unsupported AvgBitrate — skip is fine).
- **Dates** (`%timeInfo`:243-293): epoch is 1904-01-01; ValueConv
  `ConvertUnixTime($val, QuickTimeUTC-option || FileType eq "CR3")`
  (ExifTool.pm:6784); RawConv patches "brain-dead software" writing
  1970-epoch values: subtract 2082844800 only when `$val >= offset` (else
  warn). Snapshots were made WITHOUT `-api QuickTimeUTC`
  (tools/generate_exiftool_json.sh:88), so MOV dates = `gmtime`, no TZ
  suffix. PrintConv `ConvertDateTime` is identity without `-d` (same
  conclusion as xmp TPP).
- **Durations** (`%durationInfo`:314-317): ValueConv divides by
  `$$self{TimeScale}` (set by mvhd idx 3); PrintConv `ConvertDuration`
  (ExifTool.pm:6877) → `"2.96 s"` under 60s. MediaDuration instead uses
  `$$self{MediaTS}` from the SAME mdhd (MediaHeader:7266-7274).
- **Keys indirection** (ProcessKeys:9779-9877): the `keys` atom lists mdta
  key names; strip `com.apple.quicktime.` prefix; ilst entries are keyed by
  **1-based index** into that list. Value atoms are `data` boxes: 16-byte
  header `(len, 'data', u32 format-flags, u16 ctry, u16 lang)` then payload;
  flags 0x1=UTF-8 etc. (10380-10416).
- **GPS**: Keys `location.ISO6709`:6701 → ValueConv ConvertISO6709:8884
  (3 regex forms: ±DD.DDD±DDD.DDD±ALT, ±DDMM.M..., ±DDMMSS...).
  Composites split it (8668-8696): lat=c[0], lon=c[1],
  GPSAltitude=abs(c[2]), GPSAltitudeRef = c[2]<0 ? 1 : 0 with
  Above/Below Sea Level PrintConv. Snapshots run numeric GPS mode
  (`-GPSLatitude# -GPSLongitude# -GPSAltitude# -GPSPosition#`), so expect
  `Composite:GPSLatitude = 37.5044` (number) but `GPSAltitudeRef =
  "Above Sea Level"` (string).
- **CreationDate ≠ CreateDate**: CreationDate is Keys `creationdate`:6683
  with `%iso8601Date`:295-312 = ConvertXMPDate + tz-colon fix — reuse the
  existing port in `src/xmp/value_conversion.rs`. iPhone writes it as
  local-with-TZ (`2025:06:24 15:24:45-07:00`) while CreateDate (binary,
  UTC seconds) renders `2025:06:24 22:24:45`.
- **Rotation**: Composite:8632, ValueConv CalcRotation:8797 — find first
  track whose HandlerType=='vide', take ITS MatrixStructure,
  `atan2(a[1],a[0])*180/3.14159`, +360 if negative, round to 3 decimals
  (GetRotationAngle:8782). MatrixStructure ValueConv divides cols 2,5,8 by
  0x4000 (TrackHeader:1561-1566).

## Tribal knowledge (gotchas verified 2026-07-03)

- **Duplicate-tag priority is load-bearing.** TrackHeader entries all carry
  `Priority => 0` → FIRST track wins (ExifTool.pm FoundTag:9536-9588:
  an existing 0-priority tag is promoted so a later 0-priority duplicate
  does NOT override). MediaHeader/Handler entries have default priority →
  LAST wins. Proof in the oracle: IMG_3755 `TrackDuration = 2.96 s` (first
  track, video) but `MediaDuration = 0.00 s` (LAST track, metadata) and
  `HandlerDescription = "Core Media Data Handler"` (last hdlr). Get this
  wrong and the audio track's zero width clobbers ImageWidth.
- **tkhd ImageWidth/Height are fixed 16.16** read as int32u then fixed by
  `FixWrongFormat`:8872 (`$val & 0xfff00000 ? unpack('n',pack('N',$val))
  : $val` — Pentax writes the wrong format). 1920 arrives as 1920<<16.
- **tkhd/mvhd/mdhd version 1 = 64-bit dates**: Hook bumps format to int64u
  and `$varSize += 4` (TrackHeader:1512-1532, MediaHeader:1343ff same).
- **`meta` is a FullBox only sometimes**: at file level `Start => 4` skips
  version/flags (Main:552-558); inside moov it's a bare container
  (Movie:1218-1221 has no Start). Apple MOV uses moov/meta.
- **CompressorName/HandlerDescription are "sometimes Pascal, sometimes C"
  strings** — RawConv strips a leading length byte if `ord < 0x20` and
  `< len` (VisualSampleDesc:7642-7647, Handler:8454-8460).
- **CR3 dates are host-TZ dependent**: `FileType eq "CR3"` forces the UTC
  flag → ConvertUnixTime converts to LOCAL time with TZ suffix. The 20 CR3
  snapshots bake in the generator machine's TZ (US Pacific:
  `2020:09:01 07:02:54-07:00`). MOV (flag off) is TZ-independent. Keep CR3
  out of this TPP's proofs or CI in another TZ will flake.
- ProcessMOV sets `PRIORITY_DIR = 'XMP'` for non-HEIC (10016) — only
  matters if a udta XMP tag name collides with an atom tag; none in our
  supported slice.
- Real videos are GBs — the walker MUST seek past mdat, never
  `read_to_end` like avif.rs does (Task 2 walker already complies).

## Design (Option A — SHIPPED in Task 2)

Hand-written streaming walker (`src/formats/quicktime.rs`) + state struct
mirroring ExifTool's `$$self{...}` data members; leaf conversions in
`src/implementations/quicktime.rs` registered via impl_registry (the
ConvertXMPDate de-stub pattern, `2551f6e5`). Rejected: extending avif.rs's
whole-buffer parser (loads entire file); a fully table-driven SubDirectory
engine (~700 lines of ProcessMOV machinery for ~10 needed arms — revisit if
atom coverage grows).

## Out of scope (deliberate)

- **RIFF/AVI/WebP** (4 supported `RIFF:*` tags; webp snapshots expect
  RIFF:ImageWidth/Height): different container (RIFF.pm), zero shared
  code with atom parsing. Needs its own small TPP; note it in the program
  TPP follow-ups when this lands.
- **CR3/HEIC QuickTime-group tags** (20 CR3 snapshots): same walker, but
  needs Canon uuid routing + the TZ trap above. Follow-up TPP; design the
  walker so `formats/mod.rs` can later route CR3/HEIC through it.
- **ExtractEmbedded / timed metadata / QuickTimeStream.pl**, audio tags,
  AvgBitrate, chapter lists, preview images: not in supported_tags.json.
- **Composite:LensID on video** (needs QuickTime:LensModel + the LensID
  composite chain; which same-name LensID def wins is tangled in the
  registry-collision follow-up): document as known gap if it doesn't fall
  out of Task 4 for free.
- Write support (repo-wide).

## Tasks

### Task 0: Breaking tests — ✅ DONE 2026-07-03 (commit `f26e1627`)

`tests/quicktime_video_tests.rs`, snapshot-pinned; apple image test still
`#[ignore]`d pending Task 4 (un-ignore it then). See Tasks 0-1 session log.

### Task 1: Codegen string-keyed tables — ✅ DONE 2026-07-03 (`f26e1627`)

`*_TAGS_BY_NAME: HashMap<&'static [u8], TagInfo>` maps, QuickTime-gated.
See Tasks 0-1 session log for the byte-key/U+FFFD design.

### Task 2: Atom walker + mvhd/tkhd/mdhd — ✅ DONE 2026-07-03 (`9d2e2390`)

All 5 MOVs match 13/13 core tags; double-review gate passed (verdicts in the
Task 2 session log). Walker: `src/formats/quicktime.rs`; conversions:
`src/implementations/quicktime.rs`; dispatch: MOV|MP4 in `formats/mod.rs`.

### Task 3: Keys/ItemList (Make, Model, Software, CreationDate)

**Success**: IMG_3755 Make/Model/Software/CreationDate match snapshot.
**Implementation**: port ProcessKeys:9779 (keys atom parse, prefix strip,
1-based index registry) + ilst `data` box decode (10380-10416, format
flags: at minimum 0x1 UTF-8; warn-and-skip others). moov/meta is a bare
container (Movie:1218); route values through the Task 1 string-keyed
Keys/ItemList tables for names + conversions. CreationDate reuses
`convert_xmp_date` (`src/xmp/value_conversion.rs`) per %iso8601Date:295.
Also capture LensModel + GPSCoordinates + MatrixStructure + HandlerType as
TagEntries (unsupported but harmless to emit; Task 4 depends on them).
**Proof**: `cargo run -- test-images/apple/IMG_3755.MOV | grep -E "Make|Model|Software|CreationDate"`
matches snapshot; `cargo t quicktime_video`.

### Task 4: GPS ISO6709 + Composite wiring (Rotation, GPS splits)

**Success**: IMG_3755 Composite:GPSLatitude=37.5044,
GPSLongitude=-122.4763, GPSAltitude=25.247, GPSAltitudeRef="Above Sea
Level", GPSPosition="37.5044 -122.4763", Rotation=90; Rotation=0 on the
other 4 MOVs; ImageSize/Megapixels correct.
**Implementation**: port ConvertISO6709:8884 exactly (three regex arms,
`+0` numification); Rotation via a `COMPOSITE_FALLBACKS` entry
(`src/core/composite_fallbacks.rs`) porting CalcRotation:8797 +
GetRotationAngle:8782 (needs HandlerType/MatrixStructure pairing per
track — mirror how `compute_gps_position` reads the full tag map). The
QuickTime GPS composite defs lose the name-keyed registry collision with
GPS/Sony defs (orchestration.rs:200); either land the program follow-up
(b) first-buildable-wins fix here (preferred — clears 4 tags) or extend
the GPSPosition-style special case. Confirm ImageSize/Megapixels resolve
from QuickTime ImageWidth/Height via existing composite deps.
**Proof**: `cargo run --bin compare-with-exiftool -- test-images/apple/IMG_3755.MOV`
→ zero Composite diffs; `cargo t` (no regression in GPS composite tests on
still images — `tests/gps_registry_fix_integration_test.rs`).

### Task 5: udta XMP + regression sweep

**Success**: XMP:GPSLatitude/GPSLongitude on IMG_3755 and
XMP:MetadataDate on QuickTime.mov match snapshots (they're supported tags
present in MOV snapshots via the UserData `XMP_` atom:1711).
**Implementation**: route udta `XMP_` payload into the existing XMP
processor (`src/xmp/processor.rs`); then full gate:
`make codegen fmt lint t`, add `fuzz/fuzz_targets/fuzz_quicktime.rs`
(clone fuzz_avif.rs shape), run compare-with-exiftool on all 5 MOVs and
record remaining diffs here. Update program TPP row #5 + move this TPP
when done. Known-remaining (acceptable, documents follow-ups): eos_60d /
gfx100rf / QuickTime.mov EXIF+MakerNotes tags need embedded-EXIF udta
routing (Canon CNTH:2044 → Canon::CNTH, Fuji:1921, Pentax:2283 → existing
TIFF pipeline) — file as follow-up TPP with CR3 unless trivially small.
**Proof**: `make verify` clean; diffs-per-file list recorded in this TPP.

## Files referenced

- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm` — Main:548,
  Movie:1201, MovieHeader:1343, Track:1424, TrackHeader:1493,
  UserData:1585 (XMP_:1711, CNTH:2044, PENT:2283), Meta:2810 (ilst:2814,
  keys:2877), ItemList:3481, Keys:6651 (creationdate:6683, ISO6709:6701),
  Media:7218, MediaHeader:7239, SampleTable:7365, VisualSampleDesc:7585,
  Handler:8391, Composite:8630, GetRotationAngle:8782, CalcRotation:8797,
  FixWrongFormat:8872, ConvertISO6709:8884, ProcessKeys:9779,
  ProcessMOV:9932, data-box flags:10387; %timeInfo:243, %iso8601Date:295,
  %durationInfo:314
- `third-party/exiftool/lib/Image/ExifTool.pm` — ConvertUnixTime:6784,
  ConvertDuration:6877, FoundTag priority:9448/9536-9588
- `src/formats/mod.rs:1131-1248` — dispatch site (gap at 1237)
- `src/formats/avif.rs:70` — box-header parsing reference
- `src/generated/QuickTime_pm/*` — tables (empty string-keyed ones listed
  above); `src/generated/composite_tags.rs:2775-2914` — QuickTime composites
- `codegen/src/strategies/tag_kit.rs:450-453,277` — u16 key assumption
- `codegen/src/impl_registry/function_registry.rs:193-210` — dangling
  `implementations::quicktime` mapping
- `src/core/composite_fallbacks.rs`, `src/composite_tags/orchestration.rs:49-110,200`
- `src/xmp/value_conversion.rs` — convert_xmp_date to reuse
- `generated/exiftool-json/test_images_apple_IMG_3755_MOV.json` (+4 more
  `*_mov.json`) — oracle snapshots
- `tools/generate_exiftool_json.sh:23,88` — snapshot extensions + flags
