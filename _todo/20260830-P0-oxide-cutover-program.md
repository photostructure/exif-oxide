# TPP: Oxide Cutover program (management)

The umbrella tracker for replacing PhotoStructure's Perl ExifTool read path
with exif-oxide. Written 2026-08-30 after a full replan (five research
reports + independent Opus and Codex adversarial critiques, all verified
against code). Detailed plan with evidence: the "Oxide Cutover" artifact,
https://claude.ai/code/artifact/ec6a2c97-b316-45f1-b397-606bf95b5c81
(Matthew's private artifact; the load-bearing content is restated here).
Successor context: `_todo/20260701-P0-strategic-review-program.md` remains
the tracker for the 2026-07/08 workstreams; this TPP owns the cutover.

## Architecture decisions (2026-08-30)

1. **Transport swap, not library swap.** Keep exiftool-vendored and all its
   TS post-processing (ExifDateTime, tz inference, GPS repair — pinned by
   PhotoStructure fixtures). exif-oxide becomes a `-stay_open True -@ -` /
   `-execute` / `{ready}` work-alike CLI behind `exiftoolPath` +
   `checkPerl:false` + `ignoreShebang:false`. napi binding stays parked.
2. **PhotoStructure switches to `-G` family-0 tag names** (Matthew,
   2026-08-30). This supersedes porting ExifTool's bare-name
   priority-suppression machinery (Priority/Avoid/found-order/`Tag (N)`)
   into exif-oxide — group-prefixed output is exif-oxide's native shape and
   what the 379-snapshot oracle measures. The wrapper gains a groupless
   lookup (`get(tags, "DateTimeOriginal")`) applying ExifTool's merging
   heuristics in TS, verified differentially against bare-name
   `exiftool -j` over the corpus. Class-vs-helper is an open sub-choice
   (intern suggestion, unvetted): a Tags class needs hydrate/dehydrate at
   PhotoStructure's worker-IPC boundary; plain data + helper does not.
3. **Recommend dropping `-use MWG`** as part of the group-names migration
   (PhotoStructure owns cross-standard precedence; non-MWG ExifTool still
   emits Composite:SubSecDateTimeOriginal). Undecided. If MWG stays,
   exif-oxide must implement strict-mode reads in four modules + the MWG
   composites (its generated MWG table is currently EMPTY — codegen bug
   class: empty expected tables should fail closed).
4. **Translation strategy: hybrid.** Codegen keeps tables (87.5% of
   PrintConv sites work; absorbs ~80% of each ExifTool release
   automatically). LLM-assisted porting is allowed for procedural Perl
   (multi-statement expressions, helper runtime, ProcessProc ports) ONLY
   once a real Perl-vs-Rust differential harness exists — the existing
   generate-expression-tests infra is fixture-based, NOT differential.
   Full LLM-army module rewrite: rejected (release-churn re-sync cost,
   verification instrument too weak). Runtime Perl interpreter: rejected.
5. **`-ver` should report the vendored ExifTool version the build
   translates (13.59)** — satisfies VersionTask's `\d+.\d+` regex.
   (Recommendation, unconfirmed.)

## Milestones (gates in the artifact; sequence by dependency)

- **M0 Land the tree** — 2026-08-30, IN PROGRESS, nearly done (see log).
- **M1a stay_open protocol conformance** — argfile filtering, `-execute[N]`,
  `{ready[N]}` with stderr flushed first, bare-numeric `-ver`, zero stray
  stdout/stderr (batch-cluster kills children on unexpected output; tracing
  must be silenced in stay_open mode), accept-and-ignore the injected
  `-api Filter`, emit `Error`/`Warning` keys (NOT the current lowercase
  `errors` field exiftool-vendored ignores), drop invented
  `System:*DetectionStatus` tags.
- **M1b group-names migration** (exiftool-vendored + PhotoStructure repos):
  `-G` mode, groupless lookup helper + differential test, consumer/tier-list
  migration, fixture regen. Verified against Perl engine BEFORE any swap.
- **M2 compatibility oracle v2** — per `_todo/20260808-P0-compatibility-
  oracle-v2.md`, plus a PhotoStructure-argv comparison mode and corpus
  widening (MP4/AVI/XMP sidecars exist locally but are skipped by
  tools/generate_exiftool_json.sh extension list).
- **M3 output-contract parity** — struct=1 (ResourceEvent/RegionInfo —
  History.ts loop-appends sidecar History without it), `-x` family-0
  semantics, error-string contract ("Format error in file" = ignorable vs
  "File format error" = corrupt-media: opposite-signed in ErrorTypes.ts),
  File: PrintConvs (FileSize/EncodingProcess/YCbCrSubSampling), UTF-8
  marker synthesis (`__etvInvalidUtf8V1`), MWG per decision 3.
- **M4 differential oracle, then runtime gaps** — build the Perl-subprocess
  harness; then helper runtime (ConvertDateTime/ConvertDuration/Decode/
  GPS::ToDMS…), the regex-substitution trio; all placeholder hits on 60
  real files trace to 7 distinct expressions.
- **M5 video + hashes** — QuickTime Tasks 4-5, MP4/AVI corpus,
  ImageDataHash for MOV/HEIC/WebP (currently silently absent; hash equality
  groups assets in the library DB — data-integrity gate, byte-verified
  per format, non-skipping tests).
- **M6 faithful-read-IR strangler** — per `_todo/20260808-P1-faithful-read-
  ir.md`; fixes the CR3 (10%) / RAF (16%) / HEIC (20%) observation cliff.
- **M7 PhotoStructure rollout** — ReadBackend router inside `__readRawTags`
  (worker-resident; sync-process wrapper is bypassed by delegation), typed
  `UnsupportedByOxide` signal (never error-prose matching), cache keys + DB
  engine stamp so rollback invalidates native-derived values, shadow mode
  first, per-format allowlist growth, combined packaged-app Windows/macOS/
  Linux acceptance. Writes and binary extraction stay on Perl for v1.

## Key numbers (2026-08-30 baseline)

- Per-observation supported-tag match vs pinned 13.59: JPEG ~80%, DNG 72%,
  CR2 77%, RW2 58%, HEIC 20%, RAF 16%, CR3 10%. Strict works-everywhere:
  25/191 → higher after today's QuickTime landing.
- Throughput ~2.4× pooled Perl, measured at 53% of ExifTool's tag output —
  set a perf target at parity before quoting numbers.
- Oracle observes ~1.7% of the generated tag surface; 35 of 49 generated
  module trees (72k LOC) are unwired shelf-ware.

## Progress log

### 2026-08-30 (this session)

Landed on main (all review-gated; PPI + glob work Codex-second-opinioned):

- `4742fb34` codegen extraction fail-closed (+ EXIFTOOL_BASE override)
- `fad216d4` staged-copy codegen; submodule never patched again (49-file
  residue restored first; trap-lifecycle design deleted per 08-08 decision)
- `d967232e` verify/preflight split; `a10c511e` release workflow repair
- `2759d3e5` MakeTagName port; `ca0a5235` PPI structural renderer (the
  in-flight 08-08 fix + six regen-exposed defect classes, 21 regression
  tests); `aefa13c0` regenerate (placeholders 272→254)
- `8a354cb3` QuickTime Task 3 (5 tags off the compat allowlist)
- `df5e2f18` volatile File:FileInodeChangeDate excluded from oracle
- `bc192d98` codegen test PERL5LIB; `a7c35700` numeric-glob fix (14/14
  ExifTool parity); crossbeam-epoch RUSTSEC bump
- Perl/XS env rebuilt for perl 5.40.1 (was blocking codegen + 14 tests)

Remaining in M0: `make verify` green (audit ✅, check-perl fix in flight),
move 4 completed P0 TPPs to `_done/`, Codex review of the full
`4ff8ef21..HEAD` diff, push. Deliberately NOT committed:
`.claude/settings.local.json` (destructive-git allowlist hunk — Matthew to
review), `docs/chats/unknown-tags.md` (stale transcript, distill-or-delete).

New follow-up TPP: `_todo/20260830-P2-cli-tag-request-parity.md` (5
confirmed CLI request divergences deferred from the glob fix).

## Open decisions awaiting Matthew

- Drop `-use MWG` in the group-names migration? (recommended)
- Tags class vs plain data + lookup helper in exiftool-vendored?
- Geolocation: accept initial absence (city/country auto-tags degrade;
  58/231 fixtures change tzSource label) until a Geolocation.dat reader is
  ported?
- `-ver` reports 13.59? Oracle corpus in CI (LFS vs nightly runner)?
