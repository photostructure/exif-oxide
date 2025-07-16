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

1. **Update Supported Tags Configuration**: If your milestone adds working PrintConv implementations, update the `MILESTONE_COMPLETIONS` configuration in `codegen/src/main.rs` to include your new supported tags, then run `make codegen` to regenerate the supported tags JSON.

2. **Compatibility Testing**: Re-run `make compat` and iterate until all tests pass. The regenerated supported tags list will automatically be used by the compatibility tests.

3. **Code Quality**: Run `make precommit` and fix linting, compilation, and test errors.

4. **DON'T DELETE YOUR STUBS**: if clippy is complaining about unused code, and the code is something that will be used in a near-future phase of work -- delete your stub! Instead add a TODO with the milestone that will replace the stub with a real implementation.

## Important Steps After Completing a Milestone 

1. If there is a direct reference in $REPO_ROOT/docs/MILESTONES.md to the milestone, remove the completed milestone section from this document.
2. Add a terse summary of the completed work to the end of $REPO_ROOT/docs/archive/DONE-MILESTONES.md.
3. If you worked from a separate MILESTONES-$desc.md or HANDOFF-$desc.md file, move it to $REPO_ROOT/docs/archive/DONE-$(YYYYMMDD)-$desc.md and edit with completion status and any surprising gotchas or tribal knowledge that tripped you up in the implementation. If you can, remove spurious or incorrect code or skipped tasks (due to in-flight strategic changes) to make sure the archived doc concisely preserves what was actually done (and not mislead the Engineers of Tomorrow if they ever refer to the doc)

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

## Planned Milestones

**Note**: Detailed planning documents exist in `docs/milestones/` for milestones with research completed.



### Milestone 17: RAW Image Format Support (Consolidated)
- Unified RAW processing foundation for all manufacturers
- Canon, Nikon, Sony, Olympus, Fujifilm, Panasonic support  
- TIFF-based formats with manufacturer-specific handlers
- **Planning**: See [MILESTONE-17-RAW-Format-Support.md](milestones/MILESTONE-17-RAW-Format-Support.md)

### Milestone 18: Video Format Support
- QuickTime/MP4 atom parsing infrastructure
- Smartphone and prosumer video metadata extraction
- HEIF/HEIC, MOV, AVI, MP4, MPEG-TS support
- **Planning**: See [MILESTONE-18-Video-Format-Support.md](milestones/MILESTONE-18-Video-Format-Support.md)

### Milestone 19: Binary Data Extraction (`-b` support)
- Extract embedded images, thumbnails, and binary data
- CLI support for `exiftool -b` equivalent functionality  
- Streaming API for large binary data extraction
- **Planning**: See [MILESTONE-19-Binary-Data-Extraction.md](milestones/MILESTONE-19-Binary-Data-Extraction.md)

### Milestone 20: Error Classification System
- MINOR_ERRORS classification and graceful degradation
- Manufacturer quirk handling and corruption recovery
- Comprehensive validation and error context tracking
- **Planning**: Research in progress

### Milestone 21: Basic Write Support
- Core tag writing: title, caption, orientation, rating, dates
- Pre-validation and safety checks
- Foundation for metadata modification workflows
- **Planning**: Research needed

### Milestone 22: Advanced Write Support  
- MakerNote preservation and complex metadata writing
- Advanced validation and backup strategies
- **Planning**: Research needed

### Milestone 23: ImageDataHash
- ExifTool ImageHashType functionality implementation
- **Planning**: Research needed

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
