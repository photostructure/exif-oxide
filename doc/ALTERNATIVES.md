# Alternatives Considered

This document outlines the various approaches we evaluated before settling on the current exif-oxide design.

## 1. Use Existing Rust EXIF Crates

### Option A: exif-rs (kamadak-exif)

**Pros:**
- Mature, well-tested (5+ years)
- Pure Rust implementation
- Good format support (JPEG, TIFF, PNG, WebP, HEIF)
- Active maintenance

**Cons:**
- **No write support** - read-only
- **No embedded image extraction API** - only provides offsets
- Limited to ~150 predefined tags
- No timezone inference or datetime intelligence
- Would require significant forking to add our features

**Verdict:** Good for basic reading, but missing critical features.

### Option B: little_exif

**Pros:**
- Only Rust crate with read AND write support
- Simple API
- Supports 6 major formats
- Active development

**Cons:**
- **Loads entire files into memory** - poor for large RAWs
- Limited tag support (~92 tags)
- Less mature, more bugs
- No embedded image extraction
- No datetime intelligence

**Verdict:** Write support is nice, but memory usage and limited features are dealbreakers.

### Option C: nom-exif

**Pros:**
- Modern parser combinator approach
- Zero-copy design
- Good performance characteristics
- Unified API for images and video

**Cons:**
- **Very new** (<1 year, 80 stars)
- **No write support**
- **No embedded image extraction**
- Limited real-world testing
- Complex to extend (requires nom knowledge)

**Verdict:** Promising architecture but too immature and missing features.

### Option D: Fork and Extend

We considered forking nom-exif and adding:
- Embedded image extraction
- Write support
- ExifTool tag compatibility
- DateTime heuristics

**Problems:**
- Diverging from upstream makes maintenance hard
- Parser combinator approach doesn't match ExifTool's structure
- Would essentially be a rewrite anyway

## 2. FFI Wrapper Around ExifTool

### Approach: Rust wrapper calling Perl ExifTool

**Pros:**
- 100% compatibility
- No reimplementation needed
- Automatic updates with new cameras

**Cons:**
- **Still slow** - Perl runtime overhead remains
- Complex deployment (needs Perl)
- FFI overhead
- Can't optimize hot paths
- Memory safety concerns at boundary

**Verdict:** Doesn't solve the performance problem.

## 3. Full Rewrite from Scratch

### Approach: Design new metadata library without ExifTool compatibility

**Pros:**
- Clean, modern API design
- Optimal performance from day one
- Rust-idiomatic throughout

**Cons:**
- **Loses 25 years of camera quirks knowledge**
- Incompatible with existing ExifTool workflows
- Massive implementation effort
- Would need years to reach ExifTool's coverage
- Community adoption barriers

**Verdict:** Too much valuable knowledge would be lost.

## 4. Transpile Perl to Rust

### Approach: Automated conversion of ExifTool's Perl code

**Pros:**
- Preserves all logic exactly
- Could be fully automated

**Cons:**
- **Unidiomatic Rust** - would be terrible to maintain
- Perl's dynamic nature doesn't map well
- Would lose performance benefits
- Debugging nightmare

**Verdict:** Technically interesting but practically unusable.

## 5. Our Chosen Approach: Hybrid Table Generation

### Why This Works:

1. **Preserves ExifTool's Knowledge**
   - Tag definitions are mostly declarative data
   - Easy to convert tables to Rust structures
   - Maintains compatibility

2. **Allows Optimization**
   - Reimplement performance-critical paths in Rust
   - Use memory mapping, SIMD, parallelism
   - Zero-copy parsing where possible

3. **Enables Our Enhancements**
   - Add embedded image extraction
   - Integrate datetime heuristics
   - Future: Add write support

4. **Sustainable Maintenance**
   - Automated updates from ExifTool releases
   - Clear separation of generated vs hand-written code
   - Can track upstream changes easily

## Why Not Go?

We also considered Go instead of Rust:

**Go Pros:**
- Simpler language, faster development
- Good standard library
- Easy deployment (single binary)

**Go Cons:**
- **Garbage collection pauses** - hurts P99 latency
- Less control over memory layout
- No zero-copy string handling
- Weaker ecosystem for binary parsing
- Less suitable for WASM target

**Verdict:** Rust's zero-cost abstractions and memory control are better for this domain.

## Validation of Choice

Our approach is validated by:

1. **Similar Success Stories**
   - ripgrep (grep replacement) uses similar strategy
   - fd (find replacement) proves Rust performance gains
   - oxipng shows Rust can improve image tools

2. **Specific Needs**
   - You need embedded image extraction (missing everywhere)
   - You have proven datetime heuristics to integrate
   - ExifTool compatibility reduces adoption friction

3. **Performance Requirements**
   - 10-20x speedup is realistic with Rust
   - Memory safety for untrusted files
   - Parallelism for batch processing

## Conclusion

The hybrid approach of generating tag tables from ExifTool while reimplementing the parser in Rust gives us:
- ExifTool's camera knowledge
- Rust's performance and safety
- Room for our enhancements
- Sustainable maintenance path

This is the only approach that satisfies all our requirements without compromising on compatibility or performance.