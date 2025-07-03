# exif-oxide Implementation Milestones

This document tracks the incremental development milestones for exif-oxide.

## MANDATORY Steps Before Starting

1. YOU MUST READ [TRUST-EXIFTOOL](./TRUST-EXIFTOOL.md) BEFORE CONTINUING

2. [CLAUDE.md](../CLAUDE.md), [ARCHITECTURE.md](./ARCHITECTURE.md), [EXCLUDED-TAGS.md](./EXCLUDED-TAGS.md), and [ExifTool's CLAUDE.md](../third-party/exiftool/CLAUDE.md)

3. Read all relevant related documentation before starting any work. With this project, **everything** is more complicated than you'd expect.

4. Ask clarifying questions, like ../CLAUDE.md told you to do.

## Important Steps During Implementation

1. Be sure to follow the `Trust ExifTool` and `Code smell` sections of ./TRUST-EXIFTOOL.md

## Important Milestone Validation Steps

After you think you're done implementing a milestone:

1. **Update Supported Tags Configuration**: If your milestone adds working PrintConv implementations, update the `MILESTONE_COMPLETIONS` configuration in `codegen/src/main.rs` to include your new supported tags, then run `cargo run -p codegen` to regenerate the supported tags JSON.

2. **Compatibility Testing**: Re-run `make compat` and iterate until all tests pass. The regenerated supported tags list will automatically be used by the compatibility tests.

3. **Code Quality**: Run `make precommit` and fix linting, compilation, and test errors.

4. **DON'T DELETE YOUR STUBS**: if clippy is complaining about unused code, don't delete your stub! Instead add a TODO with the milestone that will replace the stub with a real implementation.

## Important Steps After Completing a Milestone

1. Remove the completed milestone section from this document, $REPO_ROOT/docs/MILESTONES.md
2. Concisely summarize the completed work and add it to $REPO_ROOT/docs/archive/DONE-MILESTONES.md.
3. If you worked from a separate MILESTONES-$desc.md file, move it to $REPO_ROOT/docs/archive and edit with completion status and any surprising gotchas or tribal knowledge that tripped you up in the implementation.

## Core Principles

1. **Always Working**: Every milestone produces runnable code with graceful fallbacks
2. **No Panics**: Missing implementations return raw values, never crash
3. **Demand-Driven**: Only implement what's needed for real test images
4. **Manual Excellence**: Complex logic is manually ported with ExifTool references
5. **Transparent Progress**: Runtime tracking shows exactly what's missing

## Development Strategy

### Always Shippable

- Every milestone runs and extracts data
- Missing features show raw values, not errors
- Coverage grows incrementally

### Demand-Driven Implementation

- Use `--show-missing` to guide development
- Only implement what real images need
- Track frequency to prioritize work
- Focus on mainstream tags (>80% frequency)

### Manual Excellence

- Each complex feature manually implemented
- Always reference ExifTool source
- Build expertise through careful porting

### Transparent Progress

- Runtime metrics show coverage
- Missing implementation logs guide work
- Users see exactly what's not supported

## Risk Mitigation

- **No Stub Explosion**: Runtime references prevent code bloat
- **No Panic Risk**: Fallback system ensures stability
- **Incremental Complexity**: Each milestone adds one hard thing
- **Real-World Focus**: Test images drive implementation priority
- **Clear Scope**: ~50 processors enumerable, not infinite
- **Mainstream Focus**: ~500-1000 tags instead of 15,000+

This milestone plan embraces the reality that we're building a complex system incrementally. By using runtime fallbacks and demand-driven development, we can ship useful functionality immediately while building toward complete ExifTool compatibility over time.

---

### Milestone 14: Second Manufacturer - Nikon (4 weeks)

**Goal**: Prove architecture with encrypted maker notes

**Summary**: Add support for Nikon cameras including format detection, offset schemes, embedded binary image extraction, and basic encryption handling.

**Key Deliverables**:

- Nikon MakerNote detection (multiple format versions)
- Nikon offset schemes (TIFF header at 0x0a)
- ProcessNikonEncrypted skeleton
- Nikon-specific PrintConv/ValueConv

**Success Criteria**: Basic Nikon data extraction, correct format version detection, encryption detected (if not decrypted).

---

## Milestones that need research and planning

### Milestone 15: XMP/XML Support

This is a major format addition

### Milestone 16: Mimetypes

- validate the contents of MIMETYPES.md and add formats and reprioritize as research indicates
- Add MIMEType detection for file types listed in MIMETYPES.md
- Note: the CLI and API for non-JPEG file types should only reliably extract the mimetype tag -- parsing support comes in future milestones.

### Milestone 17a: Canon raw support

- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 17b: Nikon raw support

- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 18a: Sony

- include raw image support
- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 18b: Olympus

- include raw image support
- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 18c: Fujifilm

- include raw image support
- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 18d: Panasonic

- include raw image support
- ensure the CLI can now read tags from raw images
- ensure the CLI can now extract binary embedded images from raw image types

### Milestone 19: Video file types

- QuickTime atoms (track as milestone 19a?)
- MP4 atoms (track as milestone 19b?)

(what other common formats? research and add sub-milestones)

### Milestone 20: Error Classification System (2 weeks)

**Goal**: Port ExifTool's sophisticated error handling. We don't need verbatim error message replication -- only _sematically similar_ messaging (as the perl and rust I/O errors, for example, will use quite different verbiage, and there's little value to replicating the entire perl error system)

**Summary**: Implement MINOR_ERRORS classification, manufacturer quirk handling, and comprehensive validation.

**Key Deliverables**:

- Error classification (Fatal/Minor/Warning)
- Manufacturer quirk handling (Samsung entry count fix)
- Comprehensive validation (offset bounds, entry count, format)
- Error context tracking with full path

**Success Criteria**: Process 1000 files without crashing, known problematic files handled gracefully, error messages semantically match ExifTool's.

---

### Milestone 21: Basic write support

(milestone research should split this into sub-milestones, include a port of the ExifTool pre-validation-before-writing algorithm )

basic tag updates: title, caption, orientation, rating, dateTimeOriginal, subSecDateTimeOriginal, and all other `-allDates` tags

(what other common tags do we need to support writes to? research and add sub-milestones)

### Milestone 22: Advanced write support

- MakerNote preservation

### Milestone 23: ImageDataHash

See <https://exiftool.org/ExifTool.html#ImageHashType>

---

## Future milestones

1. **MIE Support** - Major format addition
1. **Async Support** - AsyncRead/AsyncSeek wrappers
1. **Advanced Nikon Encryption** - Complete crypto port
1. **Complete Coverage** - Remaining mainstream conversions

---

### Milestone: Performance & Coverage Analysis (2 weeks)

**Goal**: Optimize and assess implementation coverage

**Summary**: Profile performance, analyze coverage metrics, and create priority report for future development.

**Key Deliverables**:

- Performance profiling vs ExifTool
- Coverage metrics by manufacturer
- Memory usage analysis
- Optimization pass
- Priority report for next phase

**Success Criteria**: Performance within 10x of ExifTool, clear roadmap for continued development, 60%+ coverage of common tags.
