# Trust ExifTool: The Prime Directive

## ⚠️ CRITICAL: This is the Most Important Document in the Project

This document explains the fundamental principle that governs ALL development on exif-oxide: **Trust ExifTool**.

## What This Means

### 1. We Translate, We Don't Innovate

ExifTool is the accumulation of 25 years of camera-specific quirks, edge cases, and tens of thousands of bugfixes. Every odd, confusing, seemingly "wrong" or "inefficient" piece of code exists for a reason - usually to handle a specific camera model's non-standard behavior.

**DO NOT**:

- "Improve" ExifTool's algorithms
- "Simplify" complex logic
- "Optimize" seemingly inefficient code
- Apply "best practices" that change behavior
- Make assumptions about what cameras "should" do
- Invent any parsing heuristics

**DO**:

- Copy ExifTool's logic exactly, even if it seems wrong
- Preserve the exact order of operations
- Keep all special cases and edge conditions
- Document ExifTool source references (file:line)
- Trust that Phil Harvey had a reason
- Use the same group names as ExifTool, **verbatim**
- Only stray from tag names when they include invalid Rust variable characters
- Only stray from PrintConv implementations when ExifTool itself is not consistent (like `FocalLength`)
- Some values, notably GPS, we only render decimal values (so we're always in `-GPSLatitude#` and ``-GPSLongitude#` and `-GPSAltitude#` "mode")
- Raise errors or warnings whenever ExifTool does -- but our errors and warnings only need to be semantically similar -- they don't have to match ExifTool errors verbatim.

### 2. Compatible `exiftool` output

We're trying to match ExifTool output with these options: `exiftool -j -struct -G`

### 3. Use codegen if applicable

- Use automated ExifTool heuristics extraction with our [CODEGEN](CODEGEN.md) infrastructure, when applicable. Any simple tabular data, especially if it's "big" (15+ elements) should be automatically generated for us and we use that instead of manually porting (which may quickly drift from correctness due to the frequency of ExifTool releases).

**🚨 CRITICAL**: Manual porting of ExifTool data is **BANNED**. We've had 100+ bugs from manual transcription errors. See [CODEGEN.md](CODEGEN.md#never-manual-port-exiftool-data) for why ALL ExifTool data must be extracted automatically.

### 4. Cite references

Always add a comment with the source filename, line number, and if applicable, the function or variable name that the code is derived from.

## Corollary: Chesterton's Fence

Before you remove a seemingly useless piece of code, you must first understand why it was put there. In ExifTool's case, you probably CAN'T understand why (it might be handling a camera model you've never heard of, discontinued 15 years ago, but still used by someone). Therefore, you MUST NOT remove it.

## Remember

- **No camera follows the spec**
- **The spec is a lie**
- **ExifTool is truth**
- **When in doubt, copy ExifTool**
- **When not in doubt, still copy ExifTool**

This is not a suggestion. This is not a guideline. This is the **fundamental law** of the exif-oxide project.
