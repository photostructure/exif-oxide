# TPP: napi-rs Node Binding Spike

## Summary

exif-oxide has no Node.js binding today. This is a **spike**: prove the
shape of a napi-rs binding that exposes reads and binary extraction with
an API surface compatible with
[exiftool-vendored.js](https://github.com/photostructure/exiftool-vendored.js)
(the owner's existing Node library, which PhotoStructure ships today),
so PhotoStructure can eventually swap backends per-tag/per-format:
native exif-oxide where supported, falling back to a real ExifTool child
process (exactly what exiftool-vendored.js already does) for anything
exif-oxide doesn't cover yet, and for **all writes** (exif-oxide is
read-only — see `docs/MILESTONES.md`). Success is a smoke test reading a
real Canon JPEG through the binding with output matching
exiftool-vendored's `Tags` shape for the tags exif-oxide supports. This
is explicitly not production scope — no prebuilt-binary CI pipeline needs
to ship, just a design proven with working code.

## Current phase
- [x] Research & Planning
- [ ] Write breaking tests
- [ ] Design alternatives
- [ ] Task breakdown
- [ ] Implementation
- [ ] Review & Refinement
- [ ] Final Integration

## Required reading
- [TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md) — the binding must not
  reinterpret or "clean up" exif-oxide's output; pass it through
- [API-DESIGN.md](../docs/design/API-DESIGN.md) — existing Rust public
  API (`ExifReader`, `TagEntry`) this binding wraps
- `docs/MILESTONES.md` "Tiered Architecture" section — exif-oxide is
  read-only indefinitely; the fallback-to-real-ExifTool strategy is
  permanent, not a bridge to future parity
- `~/src/exiftool-vendored.js/README.md` and `src/ExifTool.ts`,
  `src/Tags.ts` — the API surface to match

## Description

**Goal**: a napi-rs crate (new workspace member, e.g. `napi/`) that:
1. Exposes a `read(filePath) -> Promise<object>` shaped like
   exiftool-vendored's `Tags` interface, for files/tags exif-oxide
   supports.
2. Exposes binary extraction (thumbnail/preview) similar to
   `extractThumbnail`/`extractPreview`/`extractBinaryTagToBuffer`.
3. Falls back to spawning real ExifTool (child process, same approach
   exiftool-vendored.js already uses via `batch-cluster`) for tags/
   formats exif-oxide doesn't support, and unconditionally for writes.
4. Ships prebuilt binaries per platform (design only for this spike —
   don't need a working CI matrix, just prove the local build works and
   sketch the distribution plan).

**Why it matters**: this is PhotoStructure's integration path to actually
use exif-oxide in production (`docs/MILESTONES.md` item 6). Nothing
downstream can happen until this shape is proven.

**Not in scope for this spike**: write support (permanently deferred),
a real prebuilt-binary CI pipeline, full `Tags` interface coverage,
performance benchmarking.

## Tribal knowledge

### The Rust surface to wrap already exists — don't reinvent it

`src/lib.rs` already exposes JSON-shaped entry points:
- `extract_metadata_json(file_path: &str) -> Result<Value, ExifError>`
  (`src/lib.rs:74`)
- `extract_metadata_json_with_filter(...)` (`src/lib.rs:146`)
- `extract_metadata_with_filter(...) -> Result<ExifData, ExifError>` (`src/lib.rs:195`)

`ExifData`/`TagEntry` (`src/types.rs`, see `API-DESIGN.md`) already carry
group-prefixed names (`"EXIF:Make"`), a `TagValue` (post-ValueConv), and
a `print` string (post-PrintConv) — matching ExifTool's `-j -G -struct`
output that `TRUST-EXIFTOOL.md` names as the compatibility target. The
napi layer's job is thin: call `extract_metadata_json` (or
`extract_metadata_with_filter` if you need PrintConv vs ValueConv
control), convert the `serde_json::Value` to a napi `JsObject`/`JsUnknown`,
and reshape group-prefixed flat keys (`"EXIF:Make"`) into whatever key
shape exiftool-vendored's `Tags` interface expects (check
`src/Tags.ts:22402` — it's a huge generated interface, flat fields like
`Make`, `Model`, not group-prefixed; you likely need a group-stripping
or `-G` vs no-`-G` mode toggle to match it exactly).

### The exiftool-vendored.js API surface to match (verified in its source)

Key `ExifTool` class methods (`~/src/exiftool-vendored.js/src/ExifTool.ts:220+`):
- `read<T extends Tags = Tags>(file, options?) -> Promise<T>` (line 318) —
  the primary read method, returns rich-typed `Tags` (dates as
  `ExifDateTime`, etc. — exif-oxide's ValueConv output is closer to
  ExifTool's raw values than this rich typing; decide in this spike
  whether the napi layer does date-string-to-`ExifDateTime` conversion
  itself or returns something intentionally simpler for v1).
- `readRaw<T extends Tags = Tags>(file, options?) -> Promise<T>` (line 394) —
  returns `string | number | string[]` values, no rich parsing. **This
  is the easier one to match first** — it's structurally close to what
  `extract_metadata_json` already produces.
- `extractThumbnail`, `extractPreview`, `extractJpgFromRaw`,
  `extractBinaryTag`, `extractBinaryTagToBuffer` — binary extraction
  methods; exif-oxide's binary extraction is tracked separately in
  `_todo/P1-BINARY-EXTRACTION-ALL-FORMATS.md` and
  `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md` (JPEG thumbnails complete) —
  check their status before assuming full format coverage exists to wrap.
- `write(...)` — **do not implement**; always delegate to real ExifTool.

### Fallback delegation design question — decide this first, don't discover it mid-implementation

exiftool-vendored.js already solves "run real ExifTool as a
long-lived batch process" via `batch-cluster` (`~/src/batch-cluster.js`,
same owner). The fallback path for unsupported tags/formats should
almost certainly **reuse that exact mechanism** rather than
reimplementing process pooling in Rust — i.e., the simplest correct
design may be: the *Node* layer (not the Rust layer) decides per-call
whether to call the native binding or delegate to an already-running
`exiftool-vendored` instance, since that library already exists,
is battle-tested, and PhotoStructure already depends on it. Only
implement child-process spawning inside the Rust/napi layer if there's
a concrete reason the decision must be made below the JS layer (e.g.
per-tag fallback within a single file read, where JS would need the
partial native result before it knows which tags are missing). Resolve
this design question as part of Task 2, not by defaulting to "reimplement
in Rust because this is a Rust project."

### License mismatch with the existing precedent — flag, don't silently resolve

exif-oxide is `AGPL-3.0-or-later` (`Cargo.toml:11`). exiftool-vendored.js
is `MIT` (`~/src/exiftool-vendored.js/package.json:93`) — but
exiftool-vendored.js only *spawns* the GPL'd `exiftool` Perl script as a
separate process (a well-established boundary that doesn't trigger
GPL/AGPL copyleft on the caller). A napi binding **links exif-oxide's
compiled AGPL code directly into a Node native addon** that then gets
`require()`d into PhotoStructure — a materially different licensing
posture than subprocess delegation. This is a real, unresolved question,
not a spike blocker to solve technically, but **surface it explicitly to
the user before this spike is treated as a path to production** — don't
let "it built and worked" imply the licensing question was also
answered.

### napi-rs conventions (standard, not yet verified locally — no existing napi-rs crate in this repo or sibling repos to copy from)

No sibling repo (`~/src/sqlite-vec`, `~/src/node-sqlite`, `~/src/electrobun`)
has actual napi-rs source to crib from (checked — only transitive
`package-lock.json` mentions, not real usage). This spike is greenfield
here. Standard napi-rs project shape (from the `@napi-rs/cli` tool,
confirm current CLI usage since conventions drift):
- `napi new` scaffolds a crate with `#[napi]` macros over `cdylib`
  output, plus a generated `index.d.ts` and per-platform `.node` binary.
- Prebuilt binaries are typically distributed as per-platform npm
  packages under `optionalDependencies` (e.g. `@your-scope/pkg-darwin-arm64`),
  built via a GitHub Actions matrix. **This spike does not need to build
  that matrix** — just note the plan.

## Solutions

### Option A (preferred): New `napi/` workspace member crate, thin wrapper over existing `lib.rs` functions

Add `napi` to `Cargo.toml`'s `[workspace] members`. The napi crate
depends on the root `exif-oxide` crate as a path dependency and exposes
`#[napi]` functions that call `extract_metadata_with_filter`/
`extract_metadata_json`, converting `ExifData`/`serde_json::Value` to
JS values.

**Pros**: reuses all existing parsing/PrintConv/composite-tag logic
untouched; keeps the binding layer thin (SIMPLE-DESIGN.md Rule 4); the
root crate stays a normal Rust library, no napi-specific pollution.
**Cons**: two-crate build to keep in sync; `#[napi]` macro constraints
(e.g. supported return types) may force some data reshaping at the
boundary.

### Option B: Add napi bindings directly in the root crate behind a feature flag

`#[cfg(feature = "napi")]` annotations inside `src/`.

**Why Option A is better**: napi-rs's `#[napi]` macro and `cdylib` crate
type requirements would leak into the core library's `Cargo.toml`
(`crate-type`) for all consumers, including the CLI binary and future
fuzz targets from `_todo/20260701-P1-fuzzing-infrastructure.md`. A
separate crate keeps the core library's build graph clean — exactly the
kind of "fewest elements, no unnecessary coupling" tradeoff
`SIMPLE-DESIGN.md` argues for.

## Tasks

- [ ] **Task 1: Resolve the licensing question with the user** (see
      Tribal Knowledge above) before writing any distribution-facing
      code. This is a spike either way, but don't let "the spike worked"
      accidentally become "so we shipped it" without this being answered.
      **Proof**: decision recorded in this TPP's handoff notes.

- [ ] **Task 2: Decide the fallback-delegation boundary** (Node-layer
      orchestration reusing `batch-cluster`/`exiftool-vendored` vs.
      Rust-layer child-process spawning). Write a short design note in
      this TPP recording the decision and why.
      **Proof**: design note committed to this TPP.

- [ ] **Task 3: Scaffold the napi crate.** `napi new` (or manual
      `Cargo.toml` + `#[napi]` setup) under `napi/`, added to workspace
      members. Wire one trivial `#[napi] fn version() -> String`
      end-to-end (Rust to a runnable `node -e "require(...).version()"`)
      before touching real parsing logic — this proves the toolchain
      works in isolation from exif-oxide's own complexity.
      **Proof**: `node -e "console.log(require('./napi').version())"`
      prints the crate version.

- [ ] **Task 4: Wire `readRaw`-equivalent first** (simpler shape than
      full `read`'s rich date typing — see tribal knowledge). Expose
      `#[napi] fn read_raw(file_path: String) -> Result<serde_json::Value>`
      (or a hand-built `napi::Object`) calling
      `exif_oxide::extract_metadata_json`.
      **Proof**: reading a real Canon JPEG
      (`third-party/exiftool/t/images/Canon.jpg`) through the binding
      returns a JS object; spot-check 5+ fields (Make, Model,
      ISO, DateTimeOriginal, ImageSize) match
      `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg`'s
      "working" output for the same file.

- [ ] **Task 5: Binary extraction smoke test.** Wire one binary
      extraction path (thumbnail, since `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md`
      is complete for JPEG, unlike the other RAW-format binary work) through
      to a `Buffer` return, matching `extractThumbnail`'s shape closely
      enough that a PhotoStructure-style caller could swap it in.
      **Proof**: extracted buffer's bytes match
      `exiftool -b -ThumbnailImage <file>` byte-for-byte on a test image
      that has a thumbnail.

- [ ] **Task 6: Fallback stub.** Implement (or clearly stub with a TODO
      and the Task 2 design note) the per-tag/per-format fallback path
      to real ExifTool for at least one deliberately-unsupported case
      (e.g. a tag known to be in `docs/EXCLUDED-TAGS.md` or the
      QuickTime/RIFF "Blocked on Milestone 18" list from
      `_todo/P03-implementation-backlog.md`).
      **Proof**: reading a file/tag combination exif-oxide doesn't
      support still returns a correct value via the fallback, end-to-end
      through the binding.

- [ ] **Task 7: Success smoke test.** A small Node script (not
      necessarily a full test suite — this is a spike) that reads a
      Canon JPEG through the binding and asserts the returned object's
      shape and supported-tag values match what
      `exiftool-vendored`'s `readRaw` would return for the same tags on
      the same file (run both side-by-side if `exiftool-vendored` is
      available as a dependency for comparison in this smoke test).
      **Proof**: script runs, prints a pass/fail diff, committed
      somewhere discoverable (e.g. `napi/examples/smoke.mjs`).

- [ ] **Task 8: Sketch prebuilt-binary distribution plan.** Written
      plan only (per-platform npm packages + `optionalDependencies` +
      GitHub Actions build matrix) — no working CI required for this
      spike. Note explicitly that this is future work, not done here.
      **Proof**: plan section added to this TPP or spun into a follow-up
      TPP if the spike concludes the approach is worth productionizing.

- [ ] **Task 9: Final spike write-up.** Summarize what was proven, what
      wasn't, and what a "production" TPP would need to cover next
      (this spike is explicitly not that TPP). `cargo build` for the
      whole workspace (including `napi/`) succeeds; `make lint` passes
      on the new crate too. Move to `_done/` once the write-up is
      complete — a spike concludes when the question is answered, not
      when everything is production-ready.

## Files referenced

- `src/lib.rs:74,146,195` — `extract_metadata_json`,
  `extract_metadata_json_with_filter`, `extract_metadata_with_filter`
- `src/types.rs` — `ExifData`, `TagEntry`, `TagValue`
- `docs/design/API-DESIGN.md` — existing public API design this wraps
- `docs/MILESTONES.md` — read-only scope decision, tiered architecture,
  fallback strategy as permanent (not temporary)
- `Cargo.toml:1-2` — workspace members (napi crate to be added here)
- `~/src/exiftool-vendored.js/src/ExifTool.ts:220-663` — `read`,
  `readRaw`, `write`, `extractThumbnail`, `extractPreview`,
  `extractJpgFromRaw`, `extractBinaryTag`, `extractBinaryTagToBuffer`
- `~/src/exiftool-vendored.js/src/Tags.ts:22402` — generated `Tags` interface
- `~/src/batch-cluster.js` — process-pooling mechanism exiftool-vendored.js
  already uses; candidate reuse target for fallback delegation (Task 2)
- `_done/P0-IFD1-THUMBNAIL-EXTRACTION.md`,
  `_todo/P1-BINARY-EXTRACTION-ALL-FORMATS.md` — binary extraction status
  to check before Task 5
- `docs/EXCLUDED-TAGS.md`, `_todo/P03-implementation-backlog.md` — known
  unsupported tags to use as the Task 6 fallback test case
