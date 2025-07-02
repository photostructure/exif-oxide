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
- Raise errors or warnings whenever ExifTool does -- but our errors and warnings only need to be semantically similar -- they don't have to match ExifTool errors verbatim.

### 2. Critical Examples

**Never attempt to "improve" or "simplify" ExifTool's parsing logic:**

- If ExifTool checks for `0x41` before `0x42`, do it in that order
- If ExifTool has a weird offset calculation, copy it exactly  
- If ExifTool special-cases "NIKON CORPORATION" vs "NIKON", there's a reason
- No Camera Follows The Spec. Trust The ExifTool Code.

### 3. Real-World Example

```perl
# ExifTool Canon.pm:1234
if ($val == 0x41) {
    $result = "Mode A";
} elsif ($val == 0x42) {
    $result = "Mode B";
} elsif ($val == 0x41) {  # Yes, 0x41 again!
    $result = "Mode A Alt";
}
```

Your instinct: "This is wrong! The second 0x41 check will never execute!"

**WRONG!** There's a Canon camera somewhere that depends on this exact behavior. Maybe it's a timing issue, maybe it's a side effect of the first check, maybe Phil discovered this through painful debugging.

Your implementation MUST be:

```rust
// Canon.pm:1234 - DO NOT CHANGE ORDER OR LOGIC
if val == 0x41 {
    result = "Mode A".to_string();
} else if val == 0x42 {
    result = "Mode B".to_string();
} else if val == 0x41 {  // Yes, checking 0x41 again - ExifTool does this
    result = "Mode A Alt".to_string();
}
```

### 4. Common Violations to Avoid

#### String Comparisons

```perl
# ExifTool
if ($make eq 'NIKON CORPORATION' or $make eq 'NIKON') { ... }
```

**WRONG**: `if make.starts_with("NIKON") { ... }`

**RIGHT**: `if make == "NIKON CORPORATION" || make == "NIKON" { ... }`

Why? Some camera writes "NIKONSCAN" and must NOT match.

#### Magic Numbers

```perl
# ExifTool
$offset += 0x1a;  # No comment explaining why
```

**WRONG**: `offset += 0x1a; // This seems arbitrary, using 0x20 for alignment`

**RIGHT**: `offset += 0x1a; // ExifTool Canon.pm:567`

Why? Phil reverse-engineered this offset from actual camera files.

#### Optimization Attempts

```perl
# ExifTool - processes same data twice
ProcessThing($data);
ProcessThing($data);  # Yes, again
```

**WRONG**: `process_thing(&data); // Only need to process once`

**RIGHT**:

```rust
process_thing(&data);  // ExifTool Foo.pm:123
process_thing(&data);  // ExifTool Foo.pm:124 - processes twice
```

Why? Some cameras have stateful firmware that requires double processing.

### 5. The Only Exception: Perl vs Rust Idioms

The ONLY changes allowed are those required for Rust syntax:

- Perl string concatenation → Rust string formatting
- Perl regex → Rust regex crate
- Perl hash → Rust HashMap
- Perl's implicit type conversion → Explicit Rust conversions

But the LOGIC must remain identical.

### 6. When You Think ExifTool is Wrong

It's not.

- Maybe there's a camera you haven't seen
- Maybe there's a firmware version with that bug
- Maybe it's handling corrupted files from a specific model
- Maybe it's working around a manufacturer's misunderstanding of the spec

Remember: ExifTool is battle-tested on millions of real files from tens of thousands of camera models across 25 years.

### 7. How to Document Your Compliance

Always include ExifTool source references:

```rust
// Implement Canon's bizarre WB coefficient calculation
// ExifTool Canon.pm:8934-8967
// Note: This seems to multiply by 2 then divide by 2, but DO NOT OPTIMIZE
fn canon_wb_coeff(val: i32) -> f64 {
    let doubled = val * 2;          // Canon.pm:8935
    let shifted = doubled >> 1;     // Canon.pm:8936
    shifted as f64                  // Canon.pm:8937
}
```

## The Bottom Line

If you find yourself thinking:

- "This could be cleaner"
- "This is inefficient"
- "This violates DRY"
- "This could be more Rusty"
- "Surely no camera actually needs this"

**STOP.** Re-read this document. Then implement it exactly as ExifTool does.

## Corollary: Chesterton's Fence

Before you remove a seemingly useless piece of code, you must first understand why it was put there. In ExifTool's case, you probably CAN'T understand why (it might be handling a camera model you've never heard of, discontinued 15 years ago, but still used by someone). Therefore, you MUST NOT remove it.

## Remember

- **No camera follows the spec**
- **The spec is a lie**
- **ExifTool is truth**
- **When in doubt, copy ExifTool**
- **When not in doubt, still copy ExifTool**

This is not a suggestion. This is not a guideline. This is the **fundamental law** of the exif-oxide project.

## The Short Version

**Trust ExifTool, Not the Spec.**
