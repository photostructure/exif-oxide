# Architectural Anti-Patterns: Critical Mistakes That Cause PR Rejections

**🚨 CRITICAL: Read this before touching PPI/expression code or any ExifTool integration work.**

## 🚨 EMERGENCY: THESE PATTERNS CAUSE IMMEDIATE PR REJECTION 🚨

**WE'VE HAD 5+ EMERGENCY RECOVERIES** from engineers who ignored these warnings. **Most recent**: 546 lines of critical ExifTool pattern recognition were deleted, breaking Canon/Nikon/Sony support and requiring 3 weeks of emergency recovery work.

**IF YOU COMMIT ANY OF THESE PATTERNS, YOUR PR WILL BE REJECTED IMMEDIATELY:**

### ❌ INSTANT REJECTION PATTERNS

```rust
// ❌ AST STRING PARSING (Destroys type safety, breaks complex expressions)
args[1].split_whitespace()
parts.contains(&"unpack") 
node.to_string().starts_with("sprintf")

// ❌ DELETING EXIFTOOL PATTERNS (Breaks real camera files)
// Removing any pattern recognition code without understanding purpose
// Lines like: extract_pack_map_pattern, safe_reciprocal, sprintf handling

// ❌ DISABLING INFRASTRUCTURE (Creates technical debt)
// let normalized_ast = normalize()  // DISABLED
// Commenting out working systems instead of fixing integration

// ❌ MANUAL EXIFTOOL DATA (Silent bugs, transcription errors)
match wb_value { 0 => "Auto", 1 => "Daylight" }  // Hand-copied from ExifTool
static XLAT: [u8; 256] = [193, 191, 109, ...]    // Manually transcribed arrays
```

**REAL RECOVERY COSTS:**
- **546-line deletion**: 3 weeks of emergency recovery work
- **AST string parsing**: Complete rewrite of functions.rs required  
- **Manual transcription**: 100+ bug reports from silent failures
- **Disabled infrastructure**: Months of technical debt accumulation

This document covers the specific architectural mistakes that have caused multiple PR rejections and significant technical debt. These are not style issues - they are fundamental violations of the system architecture that break functionality.

## The Problem: Architectural Vandalism

New engineers consistently make the same architectural mistakes because they don't understand the core principles. This leads to:
- **Rejected PRs** after significant work
- **Broken functionality** for real camera files  
- **Technical debt** that requires emergency recovery
- **Lost development time** for entire team

## 🚨 NEVER Fight the AST - Use Visitor Pattern

**BANNED APPROACH**: Taking structured AST nodes and converting to strings for re-parsing

```rust
// ❌ ARCHITECTURAL VANDALISM - WILL BE REJECTED
let unpack_parts: Vec<&str> = args[1].split_whitespace().collect();
if unpack_parts.len() >= 2 && unpack_parts[0] == "unpack" {
    // String parsing of already-parsed AST - FORBIDDEN
}

// ❌ MORE VANDALISM
let parts: Vec<&str> = node.to_string().split(' ').collect();
if parts[0] == "sprintf" && parts[1].starts_with('"') {
    // Re-parsing structured data as strings - FORBIDDEN
}
```

**WHY THIS IS VANDALISM**: 
- PPI provides structured AST data with type safety
- Re-parsing destroys type information and creates brittle code
- Violates the visitor pattern that the entire system is built on
- Creates bugs when expressions contain spaces, quotes, or complex syntax

**REQUIRED APPROACH**: Use AST traversal

```rust
// ✅ CORRECT - Use structured AST data
if let Some(unpack_node) = node.children.iter().find(|c| c.is_function_call("unpack")) {
    let format = unpack_node.children[0].string_value.as_ref()?;
    // Process using AST structure, not string parsing
}

// ✅ CORRECT - Traverse AST children
for child in &node.children {
    match child.node_type {
        PpiNodeType::Token => self.process_token(child),
        PpiNodeType::Statement => self.visit_node(child),
        // Handle each node type properly
    }
}
```

**ENFORCEMENT**: Any PR containing these patterns will be **REJECTED**:
- `split_whitespace()` on stringified AST nodes
- `starts_with()` matching on `node.to_string()` output
- `args[N]` array indexing from string splitting of AST data

## 🚨 NEVER Disable Working Infrastructure Without Integration Plan

**VANDALISM EXAMPLE**: Disabling AST normalizer without fixing integration

```rust
// ❌ DISABLED WORKING CODE - Found at rust_generator/mod.rs:102-105
// let normalized_ast = self.normalizer.normalize(ast)?;  // DISABLED
let normalized_ast = ast; // Fallback that breaks expectations
```

**WHY THIS IS WRONG**: 
- Disabling working systems without proper integration creates technical debt
- Creates broken expectations throughout the codebase
- Forces other parts of the system to work around disabled functionality

**REQUIRED APPROACH**: Fix integration, don't disable infrastructure

```rust
// ✅ CORRECT - Enable with proper error handling
let normalized_ast = match self.normalizer.normalize(ast) {
    Ok(normalized) => normalized,
    Err(e) => {
        warn!("Normalization failed: {}, using original AST", e);
        ast // Graceful fallback with logging
    }
};
```

## 🚨 NEVER Manual Port ExifTool Data

**Problem**: We've had **100+ bugs** from manual transcription of ExifTool data.

**Why Manual Porting Always Fails**:
- **Transcription errors**: Missing entries, typos in hex values, wrong magic numbers
- **Missing edge cases**: ExifTool handles special values (e.g. -1 = "n/a") that manual ports miss  
- **Version drift**: ExifTool releases monthly, manual ports become stale immediately
- **Silent failures**: Wrong lens IDs/white balance modes fail only on specific camera models

**Examples of Real Bugs**:

```perl
# ExifTool source (correct)
%canonWhiteBalance = (
    0 => 'Auto', 1 => 'Daylight', 2 => 'Cloudy', 
    3 => 'Tungsten', 4 => 'Fluorescent', 5 => 'Flash', 9 => 'Custom'
);
```

```rust
// ❌ BANNED - Manual port with missing entries
match wb_value {
    0 => "Auto", 1 => "Daylight", 2 => "Cloudy", 3 => "Tungsten",
    // Missing: 4 => "Fluorescent", 5 => "Flash", 9 => "Custom" 
    _ => "Unknown", // Silent failure for missing modes
}

// ❌ BANNED - Magic number typos  
0x0003 => "EF 35-80mm f/4-5.6",  // Should be 0x0004 - wrong lens name

// ❌ BANNED - Manually transcribed arrays with byte errors
static XLAT: [u8; 256] = [
    193, 191, 109, // Missing bytes, wrong values, transcription errors
    // ... 253 more values with potential errors
];
```

**REQUIRED APPROACH**: Use generated lookup tables

```rust
// ✅ REQUIRED - Use generated lookup tables
use crate::generated::Canon_pm::lookup_canon_white_balance;
if let Some(wb_name) = lookup_canon_white_balance(wb_value) {
    TagValue::string(wb_name)  // Zero transcription errors, auto-updates
}

// ✅ REQUIRED - Use generated arrays  
use crate::generated::Nikon_pm::XLAT_0;
let decrypted_byte = XLAT_0[input_byte as usize];  // Byte-perfect accuracy
```

**ENFORCEMENT**: All PRs containing manually transcribed ExifTool data will be **REJECTED**. Use codegen instead.

## 🚨 NEVER Add Extraction Timestamps

**Problem**: Generators must not include runtime timestamps in generated code comments.

**Why This is Prohibited**: 
- Timestamps change on every codegen run, even when the actual extracted data is unchanged
- Creates spurious git diffs that hide real changes to generated code  
- Makes it impossible to use `git diff` to track meaningful changes
- Wastes developer time reviewing meaningless timestamp-only diffs

**Examples of Banned Patterns**:

```rust
// ❌ BANNED - Creates spurious git diffs
//! Extracted at: Wed Jul 23 17:15:51 2025 GMT

// ❌ BANNED - Same problem with different format  
//! Generated on: 2025-07-23 17:15:51 UTC

// ❌ BANNED - Any volatile timestamp
code.push_str(&format!("//! Timestamp: {}", source.extracted_at));
```

**CORRECT APPROACH**:

```rust
// ✅ GOOD - Useful source information without volatile data
//! Generated from: Canon.pm table: canonWhiteBalance
//! 
//! DO NOT EDIT MANUALLY - changes will be overwritten.
```

## 🚨 MANDATORY Pre-Commit Checks: Avoid Immediate Rejection 🚨

**RUN ALL OF THESE BEFORE SUBMITTING YOUR PR**. If any fail, your PR will be rejected:

```bash
# 1. Check for AST string parsing anti-patterns (MUST return 0 matches)
rg "split_whitespace|\.join.*split|args\[.*\]\.starts_with" codegen/src/ppi/
echo "AST string parsing violations found: $?" # Must be 1 (no matches)

# 2. Check pattern recognition completeness (MUST be >400 lines)
find codegen/src/ppi -type f | xargs wc -l

# 3. Check for disabled infrastructure (MUST return 0 matches)
rg "DISABLED|TODO.*normalize|//.*normalize.*DISABLED" codegen/src/ppi/rust_generator/mod.rs
echo "Disabled infrastructure found: $?" # Must be 1 (no disabled code)

# 4. Check for manual ExifTool transcription (SCAN results carefully)
rg "match.*=>" src/implementations/ | grep -E "0x[0-9a-f]+ =>"
# Any hardcoded hex lookup tables are BANNED - use generated tables

# 5. Build must succeed (MUST pass)
cargo check -p codegen
echo "Codegen build status: $?" # Must be 0 (success)

# 6. Verify no string parsing anywhere in PPI system
find codegen/src/ppi -name "*.rs" -exec grep -l "split(" {} \;
# Should return empty - string splitting on AST data is BANNED
```

**ENFORCEMENT**: If ANY of these checks fail, fix the issues before submitting your PR. PRs with these violations will be rejected without review.

## Verification Commands

**Check for string parsing anti-patterns**:
```bash
# Should return empty - any matches are violations
rg "split_whitespace|\.join.*split" codegen/src/ppi/
rg "args\[.*\]\.starts_with" codegen/src/ppi/
```

**Verify AST normalizer is enabled**:
```bash
# Should show normalizer.normalize() being called
grep -A5 -B5 "normalizer.*normalize" codegen/src/ppi/rust_generator/mod.rs
```

**Verify no manual ExifTool data**:
```bash
# Look for hardcoded lookup tables that should be generated
rg "match.*=>" src/implementations/ | grep -E "0x[0-9a-f]+ =>"
```

## Emergency Recovery

If you find these anti-patterns in the codebase:

1. **String parsing of AST**: See `docs/todo/P07c-emergency-ppi-recovery.md`
2. **Deleted pattern recognition**: Restore from `expressions_original.rs.bak`
3. **Disabled infrastructure**: Re-enable with proper error handling
4. **Manual ExifTool data**: Replace with generated tables via codegen

## Enforcement

These are not suggestions - they are **architectural requirements**:

- **PRs containing these anti-patterns will be REJECTED**
- **No exceptions for "working code" - broken architecture is never acceptable**
- **Emergency recovery procedures exist for fixing these mistakes**

## Why This Document Exists

The patterns documented here have caused:
- **5+ rejected PRs** with significant rework required
- **Multiple emergency recovery efforts** to fix architectural damage
- **Broken camera support** for real-world files
- **Technical debt** that compounds over time

Following these guidelines prevents wasted effort and ensures your contributions will be accepted.