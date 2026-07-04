# TPP: Strategic Review Program — Read-Only Re-Scope & Reliability Foundations

## Summary

Umbrella tracker for the 2026-07-01 strategic review. This is a **program
TPP**: it records the review's findings and decisions, and tracks the child
TPPs that implement them. Implementation detail lives in the child TPPs, not
here. A future session should read this file first, then `/tpp` the next
unstarted child.

**Problem**: exif-oxide's goal ("full read+write ExifTool port") was
unbounded, the planning docs had drifted from reality, and several
reliability gaps (never-failing compat test, no fuzzing, 16-release
submodule lag) were invisible.
**Solution**: re-scope to read-only with a permanent real-ExifTool fallback
tier, and land the reliability foundations as discrete TPPs.
**Success**: all child TPPs in the table below reach `_done/`.

## Current phase

- [x] Research & Planning (five-agent review, 2026-07-01)
- [x] Task breakdown (child TPPs authored)
- [ ] Implementation (child TPPs; see Program Status)
- [ ] Final Integration (all children done, MILESTONES.md re-triaged)

## Required reading

- [docs/MILESTONES.md](../docs/MILESTONES.md) — the scope tiers and priority
  order this program implements
- [_paused/WRITE-SUPPORT.md](../_paused/WRITE-SUPPORT.md) — the write-support
  deferral decision and its revival prerequisites

## Decisions made (2026-07-01, with the user)

1. **Read-only indefinitely.** Writing is ~half of ExifTool's scope and the
   only component that can corrupt user files. All writes delegate to real
   ExifTool (PhotoStructure already ships exiftool-vendored.js).
2. **Tiered architecture is permanent, not a bridge.** exif-oxide is the
   native fast path for covered tags/formats; real ExifTool (vendored Perl
   today, possibly a Perl-in-WASM build like the zeroperl-based ExifTool
   ports later) serves the long tail of reads and all writes. exif-oxide
   never has to be complete before it's useful.
3. **Keep this repo — do not restart.** The assets (codegen table
   extraction, 379-snapshot compat harness, 328+193-image corpus,
   manufacturer offset logic) outweigh the fragility, which is concentrated
   in one subsystem (see next).
4. **PPI transpiler: maintenance mode, not frozen** (revised 2026-07-02
   after Matthew pushed back on "freeze" — the transpiler is pivotal for
   absorbing future ExifTool updates, since every version bump pushes new
   Perl expressions through `make codegen`). The Perl→Rust expression
   transpiler (normalizer ~4.7k LOC + visitor ~2k LOC) is where every
   recorded emergency recovery originated, so it stays change-averse:
   extend it **reactively and narrowly** when a bump or required tag hits
   an expression it can't handle (with expression-level tests per change);
   no proactive rewrites/refactors. The ExifTool-fallback tier absorbs
   untranslatable expressions, so a transpiler gap degrades to "falls
   back", not "blocks the bump". **Re-evaluate a runtime expression
   interpreter** only if (a) we chase coverage far beyond required-tags,
   or (b) the transpiler causes another emergency.

## Findings worth preserving (verified, with corrections)

- **Version facts**: pinned submodule is now **v13.59** (since item #1,
  `f2bdb304`; was v13.43 at review time). Ground truth is `$VERSION` in
  `third-party/exiftool/lib/Image/ExifTool.pm` — never `git describe`,
  which reports a misleading `11.18-265` because only the 11.18 tag is
  annotated.
- **Release churn**: ~80% of a typical ExifTool release is table data that
  `make codegen` absorbs with zero human work; ~20% is procedural logic
  (Geotag interpolation, QuickTime parsing, Canon extender regex) needing
  manual ports. Monthly bumps are automatable with agent triage of the
  logic diffs — the child catch-up TPP builds the runbook.
- **The compat test never fails.** `test_exiftool_compatibility` prints a
  report and returns Ok unconditionally; its known-failures mechanism is
  dead code (`if 1 > 2`). Every mismatch currently "passes."
- **Composite bugs from the review**: all fixed — Megapixels/ShutterSpeed
  pre-review, GPSPosition 2026-07-02 (item #4, `141c4167`). Remaining
  composite work is the `Condition` + registry-collision follow-ups below.
- **Landscape** (2026-07 web survey): no alternative meets full-fidelity +
  write + permissive-license + embeddable. Exiv2 is GPL and admits less
  coverage than ExifTool; little_exif write is toy-scale; kamadak-exif /
  nom-exif are read-only. exif-oxide's approach is genuinely differentiated.
- **No `unsafe` in src/** (excluding generated): fuzz crashes will be
  panics or allocation bombs, not memory unsafety.

## Program status

| # | Work item | Where | Status |
|---|-----------|-------|--------|
| 0 | Docs/planning re-scope (MILESTONES.md, scope language, TPP migration, TODO rewrite) | commits `9e72960e`, `dcc38b20` | ✅ DONE 2026-07-01 |
| 1 | ExifTool v13.43→13.59+ catch-up + `docs/guides/EXIFTOOL-UPGRADE.md` runbook | `_done/20260701-P0-exiftool-version-catchup.md`, commit `f2bdb304` | ✅ DONE 2026-07-02 (zero ports needed; 3 codegen bugs fixed; compat 42%→43%) |
| 1b | Pre-existing lint rot (75 clippy errors at HEAD from toolchain upgrade) | commit `7fa2fd78` | ✅ DONE 2026-07-02 (`make lint` gates again; deleted 6 dead legacy normalizer passes) |
| 2 | Snapshot-oracle integrity (make compat test assert; allowlist; version-skew guard) | `_done/20260701-P1-snapshot-oracle-integrity.md` | ✅ DONE 2026-07-03 — compat test is a hard gate (allowlist + stale-entry ratchet in `config/compat_known_gaps.json`, 168 triaged gaps in 14 root-cause groups); version-skew guard (committed `.exiftool-version` marker, vendored-exiftool-only generator, incremental regen aborts on skew). **Headline metric reframed: 94/191 under old first-seen-wins counting = 23/191 under honest works-in-every-file counting — not a regression.** Newly visible bugs: HEIC EXIF extraction (0/32 on IMG_9757.heic), Nikon Z-series NEF→NRW misdetection |
| 3 | cargo-fuzz infrastructure | `_done/20260701-P1-fuzzing-infrastructure.md` | ✅ DONE 2026-07-03 — 9 targets, nightly CI job, 5 real crash bugs found AND fixed (alloc-bomb, 3 overflow panics, makernote-recursion stack overflow); double-review (Claude 8-angle + codex) each caught a distinct real bug; reproducers committed under fuzz/artifacts/ |
| 4 | GPSPosition composite sign bug | `_todo/P03-implementation-backlog.md` (Next Steps) | ✅ DONE 2026-07-02, commit `141c4167` (byte-exact; review-gated; compat 84/191) |
| 5 | Video/QuickTime read support (22 blocked tags) | `_todo/20260703-P1-quicktime-video-read.md` | 🟨 TPP authored 2026-07-03 (fable agent, citations spot-verified); implementation not started. Key findings: generated QuickTime tables half-empty (tag_kit.rs parses keys as u16, dropping `'mvhd'`-style atom IDs); `function_registry.rs:193-210` maps to a nonexistent `implementations::quicktime` module (silent-stub trap); 20 CR3 snapshots also blocked on the same walker |
| 6 | napi-rs Node binding spike | `_todo/20260701-P3-napi-node-binding-spike.md` | ⬜ not started (its Task 1 is the licensing question below) |
| 7 | XMP value conversion (8 of 13 type mismatches, one root cause) | `_done/20260703-P1-xmp-value-conversion.md` | ✅ DONE 2026-07-03 — compat 86→94/191; review caught 2 missing PrintConv arms + a latent negative-EV `print_fraction` bug (shared EXIF path), all fixed |

## Orchestration guidance (from Matthew, 2026-07-02)

- Work this program via `/tpp-orchestrate`: delegate each child TPP to a
  subagent, review-gate the result, land one coherent commit per TPP.
- **Be token-efficient**: implementation subagents should run **opus** (or
  sonnet for tightly-pinned work); reserve fable for orchestration and for
  redesigning under-baked TPPs.
- **Treat child TPP task breakdowns as only directionally correct** — they
  were written by interns. If a TPP is not fully baked, spin a **fable**
  subagent to redesign/rebuild it before implementing.
- After each work item, run a `/review` subagent; weigh its findings like
  intern feedback (vet empirically, veto with evidence), apply accepted
  fixes until copacetic, then commit (use `/coding:stage` for coherent
  partial staging).
- **Ground truth** for disputed findings: real ExifTool — the vendored
  `third-party/exiftool/exiftool` script and
  `cargo run --bin compare-with-exiftool <image> [group]`.

## Session state (2026-07-03, third session) — resume here

- **QuickTime implementation is live and mid-flight**: Tasks 0-2 of
  `_todo/20260703-P1-quicktime-video-read.md` are pushed (`f26e1627`
  codegen string-keyed tables + breaking tests; `9d2e2390` the atom
  walker — all 5 MOV snapshots match 13/13 core tags, double-review-gated
  with verdicts recorded in that TPP). **A Task 3 opus subagent
  (Keys/ilst: Make/Model/Software/CreationDate + CompressorName +
  HandlerDescription) was in flight at handoff time** — if its report is
  gone, `git status` shows its edits; review-gate then commit per that
  TPP's workflow note.
- **The review gate is now a reusable skill**: `/coding:double-review`
  in `~/src/claude-code-skills` (extracted from tpp-orchestrate steps
  4-6, which now reference it; Matthew committed it). First live run on
  Task 2: codex + Claude reviewer, 1 accepted fix (v1 fixture test),
  1 documented divergence, 2 evidence-backed vetoes.
- **Items #2 and #5-authoring landed 2026-07-03 (third session)**: the
  compat gate (see table row #2 — headline is now an honest **23/191**
  with 168 triaged+allowlisted gaps) and the QuickTime TPP (`2d654d4d`).
  Orchestration pattern held: opus implementation subagent → adversarial
  opus review subagent → orchestrator vets findings empirically → fix →
  one commit. The reviewer confirmed the orchestrator-found HIGH hole
  (marker rewritten on incremental regen would defeat the skew guard)
  and added two real ones (marker not committed; stale-ratchet
  false-fire without the B2 corpus).
- Earlier 2026-07-03 (second session), pushed through `2551f6e5`:
  item #7 (XMP value conversion) took compat 86→94 under the old
  counting. That session ran item #7 end-to-end: researched the
  root cause inline, authored the child TPP, delegated implementation to
  an opus subagent, adversarially reviewed, fixed the review findings,
  committed `0116b472` + follow-up `2551f6e5` (de-stubbed
  `xmp_date_value_conv`, which the codegen impl_registry maps to
  ExifTool's ConvertXMPDate expression — it was a silent no-op waiting
  to be wired by a future codegen run, the same trap the IPTC
  ExifDate/ExifTime stubs sprang).
- **The review gate keeps earning its keep** (implement → adversarial
  review subagent → vet each finding empirically vs vendored exiftool →
  fix → commit): this round it caught 2 missing PrintConv arms
  (ExposureCompensation, SubjectDistance) and exposed that the
  pre-existing shared `print_fraction` printed every negative EV as a
  wrong whole number (`floor`+`abs` vs Perl's signed
  truncate-toward-zero `int`, Exif.pm:5524) — latent because no compat
  snapshot carries a negative EV.
- **Subagent latency gotcha**: two background research agents' final
  reports arrived 30-60+ min late (implementation/review agents were
  fine). Don't block the critical path on research subagents — do
  must-have research inline and treat late reports as cross-checks
  (both eventually confirmed the shipped work and surfaced the de-stub
  item).
- **Shelf-ware candidate (Matthew to decide)**:
  `examples/validate_quick_xml_xmp_v3.rs` is a 517-line quick-xml
  evaluation spike predating the shipped `src/xmp/processor.rs`. It has
  now broken on BOTH quick-xml bumps (`e6ab6393`, then the 0.41 bump —
  migrated to keep the build green). Suggest deleting it; the processor
  + tests + fuzz_xmp cover its purpose.
- **Prior-session highlights** (2026-07-03 morning, all pushed): fuzzing
  item #3 (`cb3506ca`, first nightly CI run was pending); IPTC ValueConv
  + composite string-interpolation fixes, compat 84→86 (`1438c0c4`);
  quick-xml 0.38→0.41 security bump (`f4e2a58d` — if `make audit` dies
  with "unsupported CVSS version: 4.0", install cargo-audit ≥0.22.2; a
  corrupted `~/.cargo/registry/src/.../rustversion-1.0.22` extraction
  blocks that install, `rm -rf` the dir); dead `schemas/tag_kit.rs`
  removed (`f010615e`).
- **Next**: implement the video TPP (`_todo/20260703-P1-quicktime-video-read.md`,
  item #5 — biggest single win: 32 allowlisted tags + 20 CR3 snapshots),
  or #6 (napi spike; licensing answered below), or the HEIC extraction
  bug (33 allowlisted tags, `test-images/apple/IMG_9757.heic` extracts 0
  EXIF — needs a TPP/backlog entry, untriaged).
- **Working the gate (post-#2)**: when a fix lands, the stale-entry
  ratchet fails `make compat-test` until the tag is removed from
  `config/compat_known_gaps.json` — that's the intended workflow, not a
  bug. `COMPAT_DUMP_GAPS=1` dumps the machine-readable gap list.
  Composite:LensID snapshots for nikon/d3500* churn randomly on
  `compat-gen-force` (ExifTool Perl hash-order for tied lens candidates —
  verified both orderings from the same binary); safe to discard or
  commit, the tag is an allowlisted gap either way.
- Local-only files, leave alone: `.claude/settings.local.json`,
  `docs/chats/`.

## Follow-up candidates (small, non-blocking)

- **Next compat moves (post-#7, 94/191)**:
  - (a) **composite `Condition` support** (~1-2 tags but correctness):
    codegen drops ExifTool's composite `Condition` (e.g. `not defined
    $$self{VALUE}{DateTimeOriginal}`, Exif.pm:4952) and orchestration
    never evaluates it, so `Composite:DateTimeOriginal` is emitted on
    files where ExifTool suppresses it. Needs a `condition` field on
    CompositeTagDef + runtime evaluation; clears 1 of the 5
    only-in-exif-oxide gaps.
  - (b) **name-keyed `COMPOSITE_TAGS` collision** (~4 GPS tags): GPS vs
    Sony vs QuickTime defs collide, suppressing
    `Composite:GPSLatitude/GPSLongitude/GPSAltitude/GPSDateTime`. Make
    the registry first-buildable-wins per ExifTool's rules; the
    GPSPosition special-case in `src/composite_tags/orchestration.rs`
    can then be simplified away. Worth its own small TPP.
  - (c) the 5 remaining type mismatches, each a distinct small root
    cause: IPTC:Keywords list accumulation, EXIF:XPKeywords UCS-2
    decode, EXIF:GPSProcessingMethod encoding-prefix strip,
    MakerNotes:Categories (Canon), XMP:Source apostrophe truncation
    (possible XML quote-parsing bug — worth a look, could corrupt other
    values).
  - (d) XMP GPSLatitude/GPSLongitude DMS→decimal (`ToDegrees`)
    conversion, noted during #7 on iphone_x.jpg.
  - Known documented micro-gaps from #7 (fine to ignore): XMPAutoConv
    for unknown tags; `inf`/`undef` reaching Layer-2 conversions;
    attribute-form RDF in standalone .xmp not extracted (pre-existing,
    see `_todo/P10-RDF-RESOURCE-ATTRIBUTES.md`).

- **codegen should prune stale `src/generated/` files** — staleness turned
  two 13.59 codegen bugs into build breaks (details in
  `_done/20260701-P0-exiftool-version-catchup.md`).
- **codegen-internal lint debt**: `make lint` only covers codegen's LIB;
  its bins/tests/examples carry their own toolchain-lint debt visible via
  `cd codegen && cargo clippy --all-targets`. Decide: widen make lint or
  accept the scope.
- **Orphan file**: `codegen/src/ppi/normalizer/passes/binary_operators_improved.rs`
  is not declared as a `mod` anywhere (never compiles). Delete or wire in.
- **tag_kit fractional-key limitation**: bit-field subtags (`4.1`) are
  dropped from all ProcessBinaryData tables; fine while out of supported
  scope (vetoed as a 13.59 regression — see catchup TPP post-review).

## Open questions (user decisions pending)

- **AGPL vs. napi linking — ANSWERED 2026-07-02**: Matthew confirmed the
  dual-license path ("we're the author — we can dual-license for
  ourselves"). Correct: AGPL binds licensees, not the copyright holder;
  PhotoStructure can use exif-oxide under whatever terms Matthew grants
  it. Two diligence items before productionizing (not blockers for the
  spike): (a) confirm no external contributors hold copyright on any
  retained code (`git shortlog -sne`), and (b) note `src/generated/**` is
  derived from ExifTool source, which is "same terms as Perl" (dual
  GPL-1+ / Artistic) — the Artistic option permits this, but record the
  attribution reasoning in the napi TPP.
- ~~**`third-party/exiftool/doc/concepts/IMAGE_DATA_HASH.md`** untracked~~
  **RESOLVED 2026-07-02**: Matthew committed it to the fork (`eb2279b9`).
  The parent gitlink still points at `a66d7bfe`; fold the gitlink update
  into the version-catchup commit. Note the fork carries doc-only commits
  on top of upstream 13.43 (`3a79a582`, `eb2279b9`) that CLAUDE.md links
  to — the bump must preserve them (rebase onto the new tag or relocate
  the docs).

## Tribal knowledge

- The submodule working tree accumulates codegen's mechanical patches
  (`my`→`our`, glob-alias exports, `# EXIF-OXIDE PATCHED` markers) when a
  codegen run doesn't clean up. The sanctioned cleanup is
  `bash codegen/scripts/exiftool-patcher-undo.sh` (used successfully
  2026-07-03) — still sample the diff first to confirm it's only patcher
  residue before discarding anything.
- MILESTONES.md priority order is deliberate: the catch-up bump (#1)
  regenerates all snapshots, so doing oracle-integrity (#2) simultaneously
  or immediately after avoids triaging compat diffs twice.
- Child TPPs were fact-checked against source on 2026-07-01 (file:line
  citations, entry points, corpus counts). If much time has passed,
  re-verify their "verified" claims before trusting them.

## Tasks

- [x] Task 1: Complete child TPP #1 (version catch-up). **Proof**: TPP in
      `_done/`, `docs/guides/EXIFTOOL-UPGRADE.md` exists. (DONE 2026-07-02,
      commit `f2bdb304`.)
- [x] Task 2: Complete child TPP #2 (snapshot oracle). **Proof**: TPP in
      `_done/`, `make compat-test` exits non-zero on an undocumented diff.
      (DONE 2026-07-03; proven by removing an allowlist entry → gate
      fails; version skew → generator and test both fail.)
- [x] Task 3: Complete child TPP #3 (fuzzing). **Proof**: TPP in `_done/`,
      CI fuzz job green. (2026-07-03: TPP in `_done/`; job is wired and
      validated locally — first nightly run pending on GitHub.)
- [x] Task 4: Fix GPSPosition sign bug per P03 backlog. **Proof**:
      `compare-with-exiftool test-images/apple/IMG_3755.JPG` shows no
      GPSPosition diff. (DONE 2026-07-02, commit `141c4167`.)
- [x] Task 5: Author the video/QuickTime read TPP (item #5 has no TPP).
      **Proof**: new TPP in `_todo/` per TPP-GUIDE, under 400 lines.
      (DONE 2026-07-03: `_todo/20260703-P1-quicktime-video-read.md`, 370
      lines; load-bearing claims verified by orchestrator — empty
      `QUICK_TIME_MAIN_TAGS`, u16 key-drop at tag_kit.rs:450, phantom
      `implementations::quicktime` mapping, 17 supported QuickTime tags.)
- [ ] Task 6: Complete child TPP #6 (napi spike), starting with the AGPL
      question. **Proof**: TPP in `_done/` with spike write-up.
- [ ] Task 7: Re-triage MILESTONES.md when the above are done; move this
      program TPP to `_done/`.

## Files referenced

- `docs/MILESTONES.md` — scope tiers + priorities (this program implements it)
- `_todo/20260701-*.md` — the four child TPPs
- `_todo/P03-implementation-backlog.md` — GPSPosition bug
- `_paused/WRITE-SUPPORT.md` — write deferral rationale
- `tests/exiftool_compatibility_tests.rs:441-458` — the never-failing test
- `third-party/exiftool/lib/Image/ExifTool.pm:32` — `$VERSION` ground truth
