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

**`make verify` GREEN (16m40s, full gate: audit, check, codegen-test, test,
compat, mime-compat, binary-compat, build)** after four env repairs:
crossbeam-epoch RUSTSEC bump, check-perl.sh local::lib self-activation,
codegen Makefile test env, and `use lib` fallback in ppi_ast.pl /
field_extractor.pl. The four completed P0 TPPs moved to `_done/`.

CLI parity (three Opus worktree agents on
`_todo/20260830-P2-cli-tag-request-parity.md`): **parity-matcher (items
2/4/5) LANDED as `74229584`** — family-0/1 group matching, `-all#`/`-*#`,
sterilization, SetFoundTags-ordered `request_matches_tag`, 40-form parity
sweep. Two agents still reworking in their worktrees after that commit
conflicted with their bases; both were told: commit locally on the worktree
branch, `git merge main`, resolve onto the new matcher API, re-probe/test,
then the orchestrator takes `git diff main...HEAD` and applies it to the
main checkout (validate with `cargo t` + clippy --all-features + fmt there —
worktrees lack test-images and a populated submodule, so full tests only
run in the main checkout):
- numeric-order (item 3, ordered numeric selectors): worktree
  `.claude/worktrees/agent-afda610e477376220`. Pinned semantics:
  **FIRST-match-wins** (SetFoundTags appends per request,
  ExifTool.pm:5433-5436; GetInfo 3266-3290; JSON writer noDups
  exiftool:2947-2953). Its pre-merge state: fix done, 770 tests, Codex
  findings resolved (2 accepted: `-EXIF:*` misrouted, `--GPS*` inclusion
  bug; 1 vetoed pre-existing).
- file-group-fastpath (item 1 + FilePermissions value/print split):
  worktree `.claude/worktrees/agent-ab5a6bb14d7f72eb1`. Pre-merge: done,
  759 tests, fast-path perf preserved (9ms vs 62ms). Asked to re-probe its
  deferred R875-B (`-File:FilePermissions#`) which 74229584 may have fixed.
- parity-matcher's worktree `.claude/worktrees/agent-aca0614b14692b8fd` is
  merged and can be pruned.

**Codex review of the landed range `4ff8ef21..HEAD` (17 commits): 6
findings, triaged:**
- R905-A ACCEPT, fix inline: default codegen only WARNS on configured
  modules missing from the staged tree (`codegen/src/main.rs:170`) — must
  hard-error before extraction, with a test.
- R905-B ACCEPT, fix inline: field_extractor.pl catches per-symbol JSON
  serialization errors with warn-and-continue (`:182`, `:232-235`) and
  exits 0 → partial module passes. Track failures, exit nonzero.
- R905-C ACCEPT partially, fix inline: `join_unpack_binary`
  (src/core/data/mod.rs:92-100) formats `C*` bytes as HEX; Perl gives
  decimal (probe: Panasonic FirmwareVersion `0 1 1 0` vs our
  `00 01 01 00`). Fix decimal-for-C. DEFERRED: regex-on-binary-scalar
  stringification corner (TagValue::Binary stringifies as placeholder).
- R905-D ACCEPT, subagent AFTER numeric-order lands (same-file conflicts):
  Perl `/` is always float division; TagValue ops keep Int/Int→Int
  (ops.rs:62), so Canon SelfTimer 15 → "1 s" instead of "1.5 s". Fix =
  float division + serialize whole-number floats without ".0" (this
  absorbs the known `4.0` vs `4` `-Tag#` issue). The compat oracle
  adjudicates fallout; expect snapshot churn.
- R905-E CLOSED: `-all#` was fixed by `74229584`; the review raced it
  (verified: 103 tags, numeric, at current HEAD).
- R905-F ACCEPT, fix inline: CI no longer runs codegen tests
  (build.yml:99 went `--workspace` → `-p exif-oxide`) — add a Linux step:
  perl-deps (cpanm PPI/JSON::XS) + `cargo test -p codegen --locked`.

R905-A/B/C/F fixed and committed (`4f26d6a7`, `83142220`, `5692b545`,
`c4e7dcf2`). The serialization fix uncovered real damage: symbols with
bare-typeglob values had been silently dropped on every regen, leaving 49
generated files stale across ExifTool versions (%isPC had 4 of 6 entries);
all refreshed against 13.59 and the full suite passes.

All three parity worktree agents are LANDED: `74229584` (matcher),
`0035b00b` (File fast path + FilePermissions split), `c7f0d725`
(first-match-wins ordered numeric selectors; both CLI parsers un-forked).
`_todo/20260830-P2-cli-tag-request-parity.md` records 13 deferred
follow-ups. Worktree symlink farms cleaned; the three agent worktrees under
`.claude/worktrees/` can be pruned.

R905-D LANDED (`e30572dc`): Div excluded from the arithmetic macro and
always numifies to f64 (Add/Sub/Mul keep integer variants — Perl leaves
those integral); `perl_number()` serializes whole-number floats as JSON
integers (exclusive 2^63 bound) across F64/F64Array/all rational forms;
PrintFNumber and XMP ApertureValue now return sprintf *strings* as
ExifTool does (they had relied on serde emitting "8.0" from bare F64).
The old integer Div arms panicked on divide-by-zero (PanasonicRaw
DistortionScale at raw -32768); now infinity. Full `make verify` green
(16m19s, agent-run on the exact committed tree).

Compat adjudication for R905-D: `config/compat_known_gaps.json`
UNCHANGED — the gate compares parsed JSON so it was blind to `4.0` vs
`4` in both directions. Sharper text-level oracle over all 379
snapshots: 1843 values changed rendering (`.0` dropped), zero changed
value; 2946 `-Tag#` numeric values probed live against 13.59: 0
int-vs-float mismatches.

Deferred findings from R905-D (Codex-vetted, all pre-existing):

- R014-A: numeric-looking Strings are parsed and re-rendered, so
  `"1.00"` → `1.0` (53 literals in snapshots, mostly EXIF:Software /
  Composite:Megapixels). Needs the pipeline to carry the literal token
  past `extract_metadata_json`'s serde_json::Value round-trip. Pinned as
  `test_numeric_strings_lose_redundant_precision`.
- R014-B: divide-by-zero yields infinity (JSON null) where ExifTool
  warns and drops the tag — needs fallible generated conversions.
- Perl `%.15g` + `RoundFloat($val,10)` (ExifTool.pm:6119): 740/2946
  numeric-mode values differ only in float precision (we emit
  `2.8284271247461903`, ExifTool `2.82842712474619`). The single largest
  remaining numeric-fidelity gap.
- `fnumber_print_conv` non-positive branch returns `Unknown (…)`;
  PrintFNumber returns `$val` unchanged when `!IsFloat($val) or $val<=0`.
- NONDETERMINISTIC OUTPUT (real defect, own TPP:
  `_todo/20260830-P1-nondeterministic-output.md`): identical runs flip
  Composite:FocalLength35efl 26↔5.7, MakerNotes:FileNumber
  130-0112↔6-5535, XMP:NativeDigest between source tags. Pre-existing
  (pre-fix binary flips identically). Poisons snapshot ratchets.

M0 checklist: (1) R905-D landed ✓; (2) `make verify` green ✓; (3) compat
recorded above ✓; (4) push everything — done this session.

Session gotchas for the next operator: worktree agents spawn at a stale
base — have them `git merge --ff-only main` before starting; long
multi-line `git commit -m` can be spuriously denied — use `-F <file>`;
`codex exec` needs `< /dev/null`; `./scripts/capture.sh` for anything
where stderr matters. Deliberately NOT
committed:
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
