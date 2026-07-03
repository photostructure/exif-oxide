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

- **Version facts**: pinned submodule is **v13.43** (read `$VERSION` in
  `third-party/exiftool/lib/Image/ExifTool.pm`; `git describe` reports a
  misleading `11.18-265` because only the 11.18 tag is annotated). Upstream
  was 13.59 (2026-05-27): 16 releases behind, including 4 security releases.
  Earlier "v13.10 / 42 releases behind" figures were artifacts of the
  `git describe` gotcha.
- **Release churn**: ~80% of a typical ExifTool release is table data that
  `make codegen` absorbs with zero human work; ~20% is procedural logic
  (Geotag interpolation, QuickTime parsing, Canon extender regex) needing
  manual ports. Monthly bumps are automatable with agent triage of the
  logic diffs — the child catch-up TPP builds the runbook.
- **The compat test never fails.** `test_exiftool_compatibility` prints a
  report and returns Ok unconditionally; its known-failures mechanism is
  dead code (`if 1 > 2`). Every mismatch currently "passes."
- **Composite bugs**: Megapixels and ShutterSpeed are already fixed; only
  GPSPosition remains (longitude sign + precision,
  `src/core/composite_fallbacks.rs:391`, tracked in P03 backlog).
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
| 2 | Snapshot-oracle integrity (make compat test assert; allowlist; version-skew guard) | `_todo/20260701-P1-snapshot-oracle-integrity.md` | 🟨 IN PROGRESS — Task 1 (breaking test) done via the GPS fix (`tests/snapshot_oracle_tests.rs`, commit `141c4167`); Tasks 2-8 (assertive test, allowlist, version-skew guard) not started |
| 3 | cargo-fuzz infrastructure | `_done/20260701-P1-fuzzing-infrastructure.md` | ✅ DONE 2026-07-03 — 9 targets, nightly CI job, 5 real crash bugs found AND fixed (alloc-bomb, 3 overflow panics, makernote-recursion stack overflow); double-review (Claude 8-angle + codex) each caught a distinct real bug; reproducers committed under fuzz/artifacts/ |
| 4 | GPSPosition composite sign bug | `_todo/P03-implementation-backlog.md` (Next Steps) | ✅ DONE 2026-07-02, commit `141c4167` (byte-exact; review-gated; compat 84/191) |
| 5 | Video/QuickTime read support (22 blocked tags) | **no TPP yet — needs authoring** | ⬜ not started |
| 6 | napi-rs Node binding spike | `_todo/20260701-P3-napi-node-binding-spike.md` | ⬜ not started (its Task 1 is the licensing question below) |

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

## Session state at pause (2026-07-03) — resume here

- **Committed, NOT pushed** (push to main was blocked by the auto-mode
  classifier; Matthew should `git push origin main` or approve the
  prompt): `f2bdb304` (13.59 bump, item #1), `e19553f5` (tracker),
  `7fa2fd78` (lint rot), `141c4167` (GPSPosition, item #4). The
  submodule fork push DID land (`origin/docs` = "Update to 13.59").
- ~~Uncommitted working tree = item #3 (fuzzing) only~~ CLOSED 2026-07-03:
  item #3 committed and moved to `_done/` (two more fuzz-found crashes fixed
  during final validation; see that TPP's COMPLETION STATE). First nightly
  fuzz CI run happens after this lands on GitHub.
- **Next after #3 closes**: item #2 remaining tasks (assertive compat
  test + allowlist + version-skew guard — its TPP session log has the
  current 84/191 numbers), then #5 (author video TPP with a fable
  subagent), then #6 (napi spike; licensing answered above).
- Also uncommitted: `.claude/settings.local.json` (Matthew's, leave) and
  `docs/chats/` (Matthew's, leave).
- New composite-registry follow-up discovered during #4: name-keyed
  `COMPOSITE_TAGS` collision (GPS vs Sony vs QuickTime defs) suppresses
  `Composite:GPSLatitude/GPSLongitude/GPSAltitude/GPSDateTime` — worth
  its own small TPP (make the registry first-buildable-wins per
  ExifTool's rules; the GPSPosition special-case in
  `src/composite_tags/orchestration.rs` can then be simplified away).

## Follow-up candidates (small, non-blocking; found 2026-07-02)

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

- The submodule working tree accumulates codegen's mechanical `my` → `our`
  patches when a codegen run doesn't clean up. Verified safe to discard
  with `git -C third-party/exiftool checkout -- .` after confirming the
  diff is only `my`/`our` rewrites — but always sample the diff first.
- MILESTONES.md priority order is deliberate: the catch-up bump (#1)
  regenerates all snapshots, so doing oracle-integrity (#2) simultaneously
  or immediately after avoids triaging compat diffs twice.
- Child TPPs were fact-checked against source on 2026-07-01 (file:line
  citations, entry points, corpus counts). If much time has passed,
  re-verify their "verified" claims before trusting them.

## Tasks

- [ ] Task 1: Complete child TPP #1 (version catch-up). **Proof**: TPP in
      `_done/`, `docs/guides/EXIFTOOL-UPGRADE.md` exists.
- [ ] Task 2: Complete child TPP #2 (snapshot oracle). **Proof**: TPP in
      `_done/`, `make compat-test` exits non-zero on an undocumented diff.
- [x] Task 3: Complete child TPP #3 (fuzzing). **Proof**: TPP in `_done/`,
      CI fuzz job green. (2026-07-03: TPP in `_done/`; job is wired and
      validated locally — first nightly run pending on GitHub.)
- [ ] Task 4: Fix GPSPosition sign bug per P03 backlog. **Proof**:
      `compare-with-exiftool test-images/apple/IMG_3755.JPG` shows no
      GPSPosition diff.
- [ ] Task 5: Author the video/QuickTime read TPP (item #5 has no TPP).
      **Proof**: new TPP in `_todo/` per TPP-GUIDE, under 400 lines.
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
