# TPP: M1a — stay_open protocol conformance

Implementation plan produced 2026-08-30 by a read-only planning agent;
key citations spot-verified (main.rs:26-29/:51-59, exiftool:4896/:4925/
:4950, exiftool-vendored checkout present as sibling repo
`../exiftool-vendored.js`). Parent tracker:
`_todo/20260830-P0-oxide-cutover-program.md` (M1a milestone).

## Current phase

- [x] Research & Planning
- [ ] Write breaking tests
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Verified consumer contract (cited from real code, not assumptions)

- Spawn: `<exiftoolPath> -stay_open True -@ -`, no shell
  (`../exiftool-vendored.js/src/DefaultExiftoolArgs.ts:2`,
  `ExifTool.ts:284-291`). Child env replaced with `{LANG:"C"}`
  (`ExifTool.ts:268-277`) so `RUST_LOG` can't leak in — but we must be
  silent unconditionally anyway.
- Framing: `ExifToolTask.renderCommand` joins args with `\n`, appends
  `-ignoreMinorErrors` and bare `-execute\n` (`ExifToolTask.ts:30-33`).
  Startup/health task is `-ver\n-execute\n`. Exit is
  `-stay_open\nFalse\n` then stdin end (batch-cluster
  `ProcessTerminator.ts:133-146`).
- Ready token: literal `{ready}` matched as a substring of accumulated
  stdout (batch-cluster `Task.ts:86-101`). exiftool-vendored never sends
  numbered `-execute`; numbered `{ready[N]}` is protocol completeness.
- Zero stray output: non-blank stdout/stderr with NO task pending kills
  the child (batch-cluster `StreamHandler.ts:75-81`, `:95-100`). During
  the startup `-ver` task, ANY non-blank stderr is fatal
  (`Parser.ts:29-36`). During tasks, stderr lines matching
  `/error|warning/i` become task errors/warnings.
- stderr flushed BEFORE `{ready$id}\n` (exiftool:429-442, esp.
  :435-439); exiftool-vendored sets `streamFlushMillis: 1` citing those
  lines.
- `-ver` must match `/^\d{1,3}\.\d{1,3}(?:\.\d{1,3})?$/`
  (`VersionTask.ts:7,17-22`).
- Default ReadTask payload (one arg per line): `-json`, `-fast`,
  `-api` / `Filter=<one-line UTF-8 repair perl>` (`Utf8JsonFilter.ts:13-53`),
  `-api` / `struct=1`, `-use` / `MWG`, `-api` / `keepUTCTime`, seven
  numeric tag requests (`-*Duration*#` `-GPSAltitude#` `-GPSLatitude#`
  `-GPSLongitude#` `-GPSPosition#` `-GeolocationPosition#`
  `-Orientation#`), `-all`, absolute file path, `-ignoreMinorErrors`
  (AFTER the path), `-execute`. PhotoStructure adds `-x Group:Tag` pairs
  and `-api requesttags=imagedatahash` + `-api imagehashtype=MD5`.
- Error surfacing: consumer reads top-level `tags.Error`/`tags.Warning`
  (`ErrorsAndWarnings.ts:22-29`); ReadTask OVERWRITES `tags.errors` with
  its own array (`ReadTask.ts:111,257-259`) — our lowercase `errors` key
  is invisible to it. ReadTask throws on unexpected `SourceFile`
  (`ReadTask.ts:192-197`).
- ExifTool reference (third-party/exiftool/exiftool 13.59, read-only):
  `-execute(\d+)?` detection :629-631; `{ready$id}` emission :429-442;
  `ReadStayOpen` :4925-4987 (option-value lines never terminate a
  command, :4950-4963, via `%optArgs` :260-300; stdin EOF spins forever
  :4975-4979 — we deliberately diverge: EOF → clean exit 0);
  `FilterArgfileLine` :4896-4918 (`#` comments, `#[CSTR]` unescape,
  strip whitespace/CRLF); `-stay_open` values :1268-1293 (False → exit,
  NO ready token); file-not-found → `Error: File not found - $file\n` on
  stderr, no JSON entry (:2312-2318); parse errors → `Error` tag inside
  the JSON entry (:2403-2419).

## Current exif-oxide breakage (all verified; live probes recorded)

Probe: `printf -- "-ver\n-execute\n" | exif-oxide -stay_open True -@ -`
→ `Unknown option -@`, exit 1.

1. `-@` rejected (main.rs:51-59); `-stay_open` becomes tag request,
   `True` becomes a file path (main.rs:37-71).
2. `-ver` prints `0.2.0-dev` (main.rs:26-29) — fails VersionTask regex.
3. Option values become phantom files: `-api Filter=…` yields a JSON
   entry with `SourceFile: "Filter=whatever"` → ReadTask throws.
   Only `-j`, `-struct`, `-G` are no-op'd (main.rs:30-36).
4. Parsing and errors call `process::exit` (main.rs:29,:59,:217,:233,
   :254-259,:277) — fatal inside a REPL.
5. Stray stderr with RUST_LOG unset: default `EnvFilter` emits
   ERROR-level events (`error!` at main.rs:275,:330; subscriber init
   :92-95). Unconditional library `eprintln!`:
   src/file_detection/magic_numbers.rs:30,:41,
   src/raw/formats/minolta.rs:465.
6. Lowercase `errors` field (src/types/metadata.rs:813-815) with
   invented `"0.1.0-oxide"` version (main.rs:333).
7. Invented `System:*DetectionStatus` tags — 16 insert sites in
   src/formats/mod.rs (:316,:376,:404,:413,:421,:447,:466,:627,:646,
   :674,:883,:936,:984,:1045,:1087,:1099), preserved by
   metadata.rs:880-884. No consumer (grepped).
8. Invented `Warning:Xxx` keys (~25 insert sites in formats/mod.rs)
   instead of ExifTool's single `Warning` key.

## Design

Two layers mirroring ExifTool's split, in a new lib module `src/cli/`
(argfile.rs, stay_open.rs, options.rs) so unit tests don't need the
binary; main.rs shrinks to mode dispatch.

- Argfile reader: owns stdin, implements FilterArgfileLine +
  ReadStayOpen chunking with an option-arg table, yields
  `Command { args, execute_id }`, handles `-stay_open False` and EOF.
- Command runner: `parse_exiftool_args` grows an ExifTool-style option
  table and RETURNS a `ParsedCommand` (no printing/exiting). Classic
  argv mode and stay_open mode share it; classic mode keeps clap for
  oxide-only flags.
- Stay_open detection from raw args BEFORE clap: `-stay_open true|1`
  (case-insensitive) + `-@ -`.

## Ordered tasks

### T1 — option table + non-exiting parser (~0.5d)
ParsedCommand return type; option table modeled on exiftool `%optArgs`
(:260-300) — value-taking: `-api -use -x/-exclude -charset(optional)
-@ -stay_open -echo -echo2 -w -d -c -if` (port the full table); no-op
flags: `-json -fast[0-9]* -q -ignoreMinorErrors -a/-duplicates -e`;
accept-and-ignore with value: `-api <anything>` (store pairs),
`-use MWG`, `-x PAIR`, `-charset filename=utf8`. Optional cheap win:
map `-api requesttags=imagedatahash`/`imagehashtype=X` onto existing
`FilterOptions.compute_image_hash` (main.rs:224-228). `-ver` → print
version constant (see decisions). Keep tag-request classification
byte-for-byte (parity regression suites: main.rs:516-825,
src/compat/filtering.rs tests).

### T2 — JSON output contract (~0.5d)
Keep `pub errors: Vec<String>` in-memory but `#[serde(skip)]`; emit
single `"Error"` key in prepare_for_serialization
(metadata.rs:872-931). Add `pub warnings: Vec<String>`; convert
`Warning:Xxx` inserts to `warnings.push`; serialize first as
`"Warning"`. File-not-found → stderr line + NO JSON entry
(exiftool:2312-2318); existing-but-unparseable → `{SourceFile, Error}`.
Delete the 16 DetectionStatus sites + the `System:` preservation
branch. Unify ExifToolVersion on the version constant. Update
tests/integration_tests.rs:119-121 and any compat fallout (run
`make compat` early — DetectionStatus removal should only improve
parity).

### T3 — argfile reader + REPL (~1d)
filter_argfile_line per exiftool:4896-4918 incl. `#[CSTR]`; blocking
read_until(b'\n') on locked stdin, lossy UTF-8 (documented limitation);
terminate command at `^-execute(\d+)?$` (case-insensitive) when not an
option value; `-stay_open False|0` → flush + exit 0 (no ready token);
redundant True → silently ignore (divergence: ExifTool warns to
stderr); EOF → exit 0 (documented divergence from :4975-4979 spin).
Per command: run runner, write JSON + stderr lines DURING the task,
flush stderr, then `{ready<ID>}\n` + flush stdout. Empty/-ver-only
command: no "No files specified" (main.rs:231-234 must not fire).
`-echo/-echo2` immediate (exiftool:1016-1028); skip -echo3/4.

### T4 — silencing + crash containment (~2h)
No tracing subscriber in stay_open mode (main.rs:92-95). Convert the
three live library eprintln! sites to tracing::warn!. Silent panic hook
+ catch_unwind per command: on panic, `Error: internal error: …` to
stderr while task pending (surfaces as task error, doesn't kill pool),
then `{ready}` — ExifTool "NEVER say die" (exiftool:348).

### T5 — tests (~1d)
- Unit (no assets, CI-covered): argfile filtering (comments, #[CSTR],
  CRLF, empty), option-value consumption (real Utf8JsonFilter string
  verbatim; value line literally `-execute` after `-if` is NOT a
  terminator), `-execute7` → id "7", stay_open values, non-exiting
  unknown option.
- `tests/stay_open_protocol.rs` (NO feature gate — asset-free, runs in
  CI): spawn `env!("CARGO_BIN_EXE_exif-oxide")` with piped stdio +
  reader threads: (1) `-ver` cycle → exactly `<VER>\n{ready}\n`, empty
  stderr; (2) `-execute123` → `{ready123}`; (3) missing file → no JSON
  entry, `{ready2}`, stderr `Error: File not found - …`;
  (4) `-stay_open False` → exit 0, no trailing output; (5) EOF → exit
  0; (6) whole-session: accumulated stdout EXACTLY equals expected
  concatenation — zero stray bytes.
- `tests/stay_open_readtask_replay.rs` `#![cfg(feature =
  "integration-tests")]`: replay the full default ReadTask payload
  verbatim ×3 files + one missing-file cycle; per cycle: JSON array of
  one object, SourceFile echoes path, no `errors`/`System:`/
  `Warning:*` keys, ExifToolVersion matches constant, no cross-task
  residue.
- Optional differential (framing only) vs vendored exiftool.

## Decisions (Matthew) — see also parent TPP open decisions

1. `-ver` value: recommend `13.59` named constant (codegen-sync note).
   Auto-derive from submodule at codegen time is a possible follow-up.
2. `ExifToolVersion` JSON key: recommend same constant.
3. `Error`/`Warning` keys: recommend BARE keys always (consumer reads
   bare; real ExifTool under `-G` emits `ExifTool:Error` which the
   consumer would MISS — record for M1b).
4. Warning gating: recommend always emit first `Warning` in JSON; defer
   stderr `[minor]` semantics to M3.
5. `-x` in M1a: accept-and-ignore (exclusion semantics are M3).
6. Classic-mode exit codes (ExifTool exits 1 on any failed file; we
   exit 0): defer — batch-cluster ignores exit codes.
7. Debug escape hatch: optional `EXIF_OXIDE_LOG=/path` file logging in
   stay_open mode (never a std stream).

## Risks

- Rust stdout LineWriter buffering when piped: mitigated by explicit
  flushes + the integration test.
- `-stay_open False` with trailing buffered args: ExifTool processes
  residue first (:1283-1288); consumer never does this; we exit
  immediately (documented divergence).
- formats/mod.rs is a merge hotspot (~25 Warning edits) — sequence with
  other agents.
- Compat-gate fallout from DetectionStatus removal expected zero, but
  `config/compat_known_gaps.json` not inspected — run `make compat`
  early.
- Non-UTF-8 argfile paths: lossy (consumer always writes UTF-8).
- readRaw args are a subset of ReadTask's; no dedicated test planned.

## Constraints

- Sequence AFTER the nondeterminism fix lands
  (`_todo/20260830-P1-nondeterministic-output.md`) — same tree.
- docs/TDD.md workflow; docs/TRUST-EXIFTOOL.md; never edit generated/.
- Gate: `cargo t`, clippy `-D warnings`, fmt, `make verify`.
