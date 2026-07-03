# ExifTool Version Upgrade Runbook

How to bump the vendored ExifTool submodule (`third-party/exiftool`) to a
newer upstream release. Written from the 13.43 → 13.59 bump (2026-07-02),
which is used as the worked example throughout. Expect ~1 focused session
per bump if nothing procedural needs porting; the triage step tells you
early if that's not true.

## TL;DR checklist

```text
[ ] 1. Verify real current version ($VERSION grep — NOT git describe)
[ ] 2. Preconditions: clean submodule, perl deps present, baseline compat run
[ ] 3. Merge upstream tag into fork `docs` branch; resolve --theirs; verify
[ ] 4. make codegen — READ THE LOG, exit code lies
[ ] 5. cargo build — build breaks here are codegen bugs; fix codegen/src/
[ ] 6. Triage the .pm diff (table vs procedural; read-path only)
[ ] 7. Port procedural changes (if any) with TDD + file:line citations
[ ] 8. Discard codegen's submodule patches; `make compat-gen-force` (MANDATORY on every bump)
[ ] 9. make compat-test — version-skew guard + gate; compare against the pre-bump baseline
[ ] 10. Push fork, commit gitlink + generated churn + ports together
```

## 1. Version ground truth

`git describe` and `git submodule status` report garbage like
`11.18-265-g3a79a582` because 11.18 is the only *annotated* tag in the
repo; every 12.x/13.x tag is lightweight. **Never trust them.**

```bash
# The only ground truth:
grep -n '^\$VERSION' third-party/exiftool/lib/Image/ExifTool.pm

# Latest upstream releases:
git -C third-party/exiftool ls-remote --tags upstream \
  | grep -oE 'refs/tags/13\.[0-9]+$' | sed 's#refs/tags/##' | sort -t. -k2 -n | tail -5
```

Release dates and security-release flags: `third-party/exiftool/Changes`
(after fetching) or https://exiftool.org/rss.xml.

## 2. Preconditions

```bash
# a. Submodule working tree must be clean (a dirty tree is usually
#    leftover codegen patches — see §8 — but confirm before discarding):
git -C third-party/exiftool status --porcelain

# b. Perl deps for codegen (XS modules silently vanish when the system
#    perl gets upgraded — this happened before the 13.59 bump):
perl -I ~/perl5/lib/perl5 -MJSON::XS -MPPI -e1 && echo ok
# If missing: ~/perl5/bin/cpanm -l ~/perl5 --notest --installdeps .

# c. Baseline: record the compat pass rate BEFORE touching anything.
#    You cannot interpret post-bump diffs without this.
make compat-test   # note "Success rate: N% (X/Y)" + category counts
```

## 3. Bump the submodule

The fork (`git@github.com:photostructure/exiftool.git`) tracks upstream
via **merge commits on the `docs` branch** ("Update to X"), which carries
fork-only additions (`doc/`, `CLAUDE.md`). Keep that convention:

```bash
git -C third-party/exiftool fetch upstream --tags --no-recurse-submodules
git -C third-party/exiftool merge 13.59 -m "Update to 13.59"
```

Expect **massive conflicts**: older fork updates were squash-style, so the
merge-base is ancient. The fork never intentionally diverges in
upstream-owned files, so resolution is wholesale take-theirs:

```bash
git -C third-party/exiftool checkout --theirs -- .
git -C third-party/exiftool add -A
git -C third-party/exiftool commit -m "Update to 13.59"
```

**Verify the merge** — this is the step that catches a botched resolution:

```bash
# Must show ONLY fork additions (doc/, CLAUDE.md, .claude/):
git -C third-party/exiftool diff --stat 13.59 HEAD
# Must show the new version:
grep '^\$VERSION' third-party/exiftool/lib/Image/ExifTool.pm
```

Push the fork branch (`git -C third-party/exiftool push origin docs`)
before committing the parent gitlink, or fresh clones can't resolve it.

## 4. Regenerate: `make codegen`

```bash
./scripts/capture.sh make codegen   # capture: the log matters
```

**`make codegen` exits 0 even when every extractor fails.** After the
perl-upgrade incident it "succeeded" while all modules died on missing
JSON::XS and regenerated nothing. Always check:

- The log has no `Failed to extract` lines. (A steady set of
  `Unsupported ExifTool namespace reference` / `Unknown ExifTool function`
  WARNs is normal — those are known-untranslatable expressions that fall
  back at runtime.)
- `git status src/generated/` shows real churn. A 16-release bump touched
  114 files (+2.2k/−1.4k). Zero churn after a version bump means codegen
  didn't actually run.

## 5. Build — where new-Perl-construct bugs surface

`cargo build`. Two real examples from 13.59, both fixed in `codegen/src/`
(NEVER hand-edit `src/generated/`):

- **Rust-keyword collision**: Exif.pm added a `%use` hash → codegen
  emitted `pub mod use;`. Fixed with a keyword guard in `to_snake_case`
  (`codegen/src/strategies/output_locations.rs`) → `use_`.
- **Strategy misrouting**: Nikon's new `MakerNotes0x56` tag table has
  fractional bit-field keys (`4.1`, `4.2`), which made
  `SimpleTableStrategy` claim a tag-definition table, breaking ValueConv
  function generation (dangling `ast_value_*` import, E0432). Fixed the
  `can_handle` claiming rule in `codegen/src/strategies/simple_table.rs`.

Pattern: a build failure after a bump is almost always the new ExifTool
release exercising a codegen path no previous release did. Fix the
strategy/generator with a unit test mirroring the offending table shape.

Run `make lint` here too, not just at the end: clippy on generated code
is a real semantic oracle. In the 13.59 bump, `eq_op`/`identity_op`
(`1i32 / 1i32`, `1i32 * 3i32`) exposed the PPI transpiler dropping
parentheses (`1/(1+$val/32768)` → `1/1 + val/32768`). Never blanket-allow
those two lints for generated functions — degenerate literal arithmetic
in output is evidence of a transpiler precedence bug, not noise.

## 6. Triage the .pm diff

Only modules codegen processes matter (`config/exiftool_modules.json`),
and only *procedural read-path* changes need human work — table data is
absorbed by codegen, and write-path changes are out of scope entirely
(exif-oxide is read-only; see `docs/MILESTONES.md`).

```bash
# Changed files ∩ codegen module list, sorted by churn:
git -C third-party/exiftool diff --numstat 13.43..13.59 -- lib/ \
  | sort -rn | head -30   # then filter against exiftool_modules.json

# Fast tabular-vs-procedural first pass (~68% `=>` density ⇒ mostly tables):
git -C third-party/exiftool diff 13.43..13.59 -- lib/Image/ExifTool/Canon.pm \
  | grep -c '^+.*=>'
```

Classify each hunk:

- **(a) table data** — `key => value` lines, PrintConv hashes, new tags:
  codegen absorbs; no action.
- **(b) out of read scope** — writer code (`Writer.pl`, `Geotag.pm`
  track-log writing, `Set*` subs), options plumbing, docs: no action.
- **(c) procedural read-path** — new decode logic, dispatch conditions,
  regex heuristics: check whether `src/` hand-ports that area
  (`rg` for the sub name / tag name / ExifTool citation comments). If no
  port exists, the change falls to the ExifTool fallback tier —
  **PORT-OPTIONAL**, record and move on. Only changes landing on
  hand-ported logic are **PORT-REQUIRED**.

13.43→13.59 outcome (~28k added lines upstream): **zero PORT-REQUIRED**.
Big procedural diffs were all in unimplemented paths (Canon `PrintLensID`
teleconverter regex, Sony Tag94xx offset hooks, QuickTime/Matroska/DJI)
or neutralized by infrastructure (ExifTool's UCS2→UTF16 migration — we
already use `String::from_utf16`; XMP CDATA scanner fixes — we use
quick-xml). For a large gap, fan the per-module classification out to
subagents and spot-verify their claims.

## 7. Port required logic (when triage finds any)

Per `docs/TRUST-EXIFTOOL.md` and `docs/TDD.md`: translate exactly, cite
`ExifTool file:line`, write the test before the port, and verify with
`cargo run --bin compare-with-exiftool <affected image>`.

## 8. Clean the submodule, regenerate snapshots

Codegen patches the submodule working tree (`my`→`our` rewrites plus
glob-alias export blocks). They're mechanical and safe to discard, but
**only between codegen runs** — the next `make codegen` re-applies them:

```bash
git -C third-party/exiftool diff --stat        # sanity-check it's the usual patches
git -C third-party/exiftool checkout -- .
```

Snapshots must come from the **submodule's** exiftool, not whatever's on
`$PATH` (the system one is usually older). `tools/generate_exiftool_json.sh`
now hard-codes the vendored `third-party/exiftool/exiftool`, so no PATH
override is needed — just run it. **This step is mandatory on every submodule
bump**, even if you think nothing relevant changed: the committed
`generated/exiftool-json/.exiftool-version` marker (checked by the §9
version-skew guard) is only rewritten by a fully successful `--force` run,
and incremental runs (`make compat-gen`, and therefore `make compat`) abort
on a marker mismatch so a bump can't silently mix oracle versions.

```bash
make compat-gen-force
# Spot-check the recorded version marker:
cat generated/exiftool-json/.exiftool-version   # must match $VERSION (§1)
```

## 9. Compat test against baseline

```bash
make compat-test
```

Compare with the §2 baseline. The pass rate must be **≥ baseline**; the
interesting part is the *delta*, which mixes:

- our regressions (a port or codegen change broke something) — fix now;
- upstream behavior changes (tag renames, new tags, changed PrintConv
  strings) — legitimate; snapshot/oracle bookkeeping, not code bugs.
  13.59 examples: FujiFilm `RAFVersion`→`FirmwareVersion`, Pentax
  `AFInfo`→`AFInfoK3III`, composite `FocalLength35efl` description.

`make compat-test` now runs two hard gates (per the snapshot-oracle TPP):

- `test_snapshot_exiftool_version_matches_submodule` fails until you've run
  `make compat-gen-force` (§8) so the `.exiftool-version` marker matches the
  pinned submodule `$VERSION`. If you skipped §8, this is what catches it.
- `test_exiftool_compatibility` fails on any tag that diverges from the
  snapshot without an entry in `config/compat_known_gaps.json`, and also on
  any allowlisted tag that starts matching again (the ratchet). A legitimate
  upstream behavior change therefore means editing the allowlist (with a
  reason + reference) or removing a now-passing entry — not just eyeballing
  the report.

## 10. Land it

One coherent commit: submodule gitlink + `src/generated/` churn +
codegen fixes + ports + doc updates. Fork branch pushed first (§3).
`make codegen fmt lint t` must be clean.

## Future automation sketch

A nightly/weekly CI job could: `ls-remote --tags upstream` → compare to
`$VERSION` → on a new release, run §3–§5 and §8–§9 mechanically → open a
PR when green (with the compat delta in the description) or an issue with
the failing log when red. The human (or an agent) still does §6 triage on
the PR diff. Prerequisite: the snapshot oracle must actually fail on
regressions, else "green" is meaningless.
