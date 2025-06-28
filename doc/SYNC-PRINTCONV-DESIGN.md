# PrintConv Synchronization Design v2.0

## Overview

ExifTool PrintConv ("Print Conversion") transforms raw EXIF values into human-readable strings. ExifTool has ~50,000 lines of PrintConv code across all manufacturers. This design enables automatic synchronization using Perl introspection to extract the actual PrintConv logic, rather than parsing Perl syntax with regex.

## Problem Statement

1. **Scale**: 11,000+ EXIF tags, each potentially having PrintConv logic
2. **Complexity**: PrintConv includes hash lookups, subroutines, shared lookup tables, and complex expressions
3. **Shared Data**: Lens databases (`%canonLensTypes`, `%nikonLensTypes`) used by multiple tags
4. **Updates**: Monthly ExifTool releases add new tags requiring PrintConv
5. **Architecture Mismatch**: Current sync duplicates shared lookup data instead of preserving DRY references
6. **Data Duplication**: Canon lens table (524 entries) extracted 25 times instead of once

## Tribal Knowledge Discovered During Implementation

### 1. ExifTool Module Discovery

**Key Finding**: ExifTool does NOT dynamically scan for modules. Instead, it uses a static list `@loadAllTables` defined in `lib/Image/ExifTool.pm` (lines 144-161). This list contains ~200 module names that are loaded via `LoadAllTables()`.

**Implication**: The `--all-modules` feature should use this same list rather than filesystem scanning.

### 2. Existing Infrastructure

The exif-oxide codebase already has significant PrintConv infrastructure:

- `printconv_analyzer.rs` - Analyzes patterns across manufacturers
- `printconv_generator.rs` - Generates Rust code from patterns
- `printconv_tables.rs` - Extracts complete tag tables
- `analyze_printconv_safety.rs` - Analyzes PrintConv safety across contexts (IN PROGRESS)

### 3. emit_sync_issue() Integration

The sync system now includes `emit_sync_issue()` functionality that tracks manual work needed. This creates a feedback loop where:

1. Sync runs and generates what it can
2. Unmapped patterns are tracked in `sync-todos.jsonl`
3. Engineers can prioritize based on frequency/importance
4. Manual implementations feed back into pattern tables

### 4. PrintConv Type Safety Analysis

A separate binary (`analyze_printconv_safety`) uses Perl introspection to analyze PrintConv safety across different contexts (EXIF/MakerNote/XMP). This identifies:

- Safe universal patterns (same implementation everywhere)
- Collision risks (same tag name, different implementations)
- Context-specific patterns requiring unique handling

## Architectural Fix Required

**Current Flaw**: Sync extracts expanded data from each tag, creating separate tables instead of preserving ExifTool's DRY shared lookup architecture.

**ExifTool Architecture**:
- Shared tables defined once: `%canonLensTypes = ( 1 => 'Canon EF 50mm', ... )`  
- Tag definitions reference shared tables: `PrintConv => \%canonLensTypes`
- DRY principle maintained

**New Architecture Required**:
1. **Extract module-level shared tables first** (`%canonLensTypes`, `%nikonLensTypes`)
2. **Generate single shared lookup files** (`canon_lens_types.rs`)
3. **Extract tag references to shared tables** (`PrintConvId::CanonLensTypes`)
4. **Runtime uses shared table references** (no data duplication)

## Solution Architecture

### 0. Prioritization can use exiftool-vendored.js

`$REPO_ROOT/third-party/exiftool-vendored.js/data/TagMetadata.json` is a list of tag names:
```
  "Aperture": {
    "frequency": 0.85,
    "mainstream": true,
    "groups": ["APP", "Composite", "MakerNotes"]
  },
```

Missing PrintConv for tag names that are "mainstream: true" or frequency is > 80% is a high priority task.

Missing PrintConv for tag names whose frequency > 25% is medium.

All else are low.

This is codified in `exiftool_sync::tag_metadata::get_priority()`:

```rust
use crate::extractors::tag_metadata::get_priority;

// Get priority for a tag based on mainstream status and frequency
let priority = get_priority("ExposureTime");  // Returns Priority::High

// Or use the full TagMetadata API:
use crate::extractors::tag_metadata::TagMetadata;

let metadata = TagMetadata::load()?;
let priority = metadata.get_priority("Aperture");  // Returns Priority::High
let high_priority_tags = metadata.get_tags_by_priority(Priority::High);
```

### 1. Two-Phase Perl Extraction

**Phase 1 - Extract Module-Level Shared Tables**:
```perl
# New function: extract_shared_tables($module)
# Finds: %canonLensTypes, %nikonLensTypes, etc.
# Output: shared_tables.json with module-level lookup data
```

**Phase 2 - Extract Tag PrintConv References**:
```perl  
# Enhanced: extract_printconv_references($module)
# Detects: PrintConv => \%canonLensTypes 
# Output: tag_printconv.json with references, not expanded data
```

Key architectural changes:
- **Shared tables extracted separately** from tag definitions
- **Tag definitions record references** (`PrintConvId::CanonLensTypes`) not expanded data  
- **No data duplication** in extraction output

### 2. Structured Data Output

The Perl script outputs JSON with complete PrintConv information:

```json
{
  "exiftool_version": "12.65",
  "extraction_date": "2024-06-26",
  "tags": [
    {
      "tag_id": "0x0095",
      "tag_name": "LensType",
      "module": "Image::ExifTool::Canon",
      "printconv_type": "hash_ref",
      "printconv_ref": "canonLensTypes",
      "printconv_data": {
        "0": "n/a",
        "1": "Canon EF 50mm f/1.8",
        "2": "Canon EF 28mm f/2.8",
        "3": "Canon EF 135mm f/2.8 Soft"
        // ... 300+ lens entries
      }
    },
    {
      "tag_id": "0x829a",
      "tag_name": "ExposureTime",
      "module": "Image::ExifTool::Exif",
      "printconv_type": "code_ref",
      "printconv_func": "PrintExposureTime"
    },
    {
      "tag_id": "0x0001",
      "tag_name": "FocalLength",
      "module": "Image::ExifTool::Canon",
      "printconv_type": "string",
      "printconv_source": "\"$val mm\""
    }
  ]
}
```

### 3. Rust Code Generation

Transform the extracted data into efficient Rust code:

```rust
// Generated file: src/tables/generated/canon_lens_types.rs
// EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm %canonLensTypes
// AUTO-GENERATED - DO NOT EDIT

use phf::phf_map;

pub static CANON_LENS_TYPES: phf::Map<u16, &'static str> = phf_map! {
    0u16 => "n/a",
    1u16 => "Canon EF 50mm f/1.8",
    2u16 => "Canon EF 28mm f/2.8",
    3u16 => "Canon EF 135mm f/2.8 Soft",
    // ... generated from printconv_data
};
```

### 4. PrintConv ID System with Pattern Matching

Map extracted PrintConv to reusable conversion functions:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrintConvId {
    None,

    // Simple lookups
    OnOff,              // { 0 => 'Off', 1 => 'On' }
    YesNo,              // { 0 => 'No', 1 => 'Yes' }

    // Shared data lookups
    CanonLensType,      // Uses CANON_LENS_TYPES table
    NikonLensType,      // Uses NIKON_LENS_TYPES table

    // Function-based
    ExposureTime,       // PrintExposureTime function
    FNumber,            // PrintFNumber function

    // String expressions
    Millimeters,        // "$val mm"
    Float1Decimal,      // sprintf("%.1f", $val)
    RoundToInt,         // sprintf("%.0f", $val)

    // Complex expressions
    GPSCoordinate,      // Complex DMS conversion
    Flash,              // Bitfield decoding
}

// Lookup tables for pattern matching
use phf::phf_map;

/// Maps Perl string expressions to PrintConvId
static PERL_STRING_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    "\"$val mm\"" => PrintConvId::Millimeters,
    "'$val mm'" => PrintConvId::Millimeters,
    "sprintf(\"%.1f\",$val)" => PrintConvId::Float1Decimal,
    "sprintf('%.1f',$val)" => PrintConvId::Float1Decimal,
    "sprintf(\"%.0f\",$val)" => PrintConvId::RoundToInt,
    "'$val'" => PrintConvId::None,  // Pass-through
};

/// Maps hash patterns to PrintConvId (normalized for comparison)
static HASH_PATTERNS: phf::Map<&str, PrintConvId> = phf_map! {
    "0:Off,1:On" => PrintConvId::OnOff,
    "0:No,1:Yes" => PrintConvId::YesNo,
    "0:Unknown,1:Macro,2:Close,3:Distant" => PrintConvId::SubjectDistanceRange,
};

impl PrintConvId {
    /// Determine PrintConvId from extracted JSON data
    pub fn from_extraction(data: &ExtractedPrintConv) -> Self {
        match &data.printconv_type {
            "none" => PrintConvId::None,

            "hash" => {
                // Normalize hash to pattern string for lookup
                if let Some(map) = &data.printconv_data {
                    let pattern = normalize_hash_pattern(map);
                    if let Some(&id) = HASH_PATTERNS.get(&pattern) {
                        return id;
                    }
                }
                PrintConvId::None
            }

            "hash_ref" => {
                match data.printconv_ref.as_deref() {
                    Some("canonLensTypes") => PrintConvId::CanonLensType,
                    Some("nikonLensTypes") => PrintConvId::NikonLensType,
                    _ => PrintConvId::None,
                }
            }

            "string" => {
                if let Some(source) = &data.printconv_source {
                    // Direct lookup of Perl expression
                    if let Some(&id) = PERL_STRING_PATTERNS.get(source.as_str()) {
                        return id;
                    }
                }
                PrintConvId::None
            }

            "code_ref" => {
                if let Some(func) = &data.printconv_func {
                    match func.as_str() {
                        "PrintExposureTime" | "Image::ExifTool::Exif::PrintExposureTime"
                            => PrintConvId::ExposureTime,
                        "PrintFNumber" | "Image::ExifTool::Exif::PrintFNumber"
                            => PrintConvId::FNumber,
                        _ => PrintConvId::None,
                    }
                } else {
                    PrintConvId::None
                }
            }

            _ => PrintConvId::None,
        }
    }
}

/// Normalize a hash map to a pattern string for matching
fn normalize_hash_pattern(map: &HashMap<String, String>) -> String {
    let mut pairs: Vec<_> = map.iter()
        .map(|(k, v)| format!("{}:{}", k, v))
        .collect();
    pairs.sort();
    pairs.join(",")
}
```

### 5. PrintConv Runtime Implementation

How the generated tables connect to runtime conversion:

```rust
// src/core/print_conv.rs

use crate::tables::generated::{
    canon_lens_types::CANON_LENS_TYPES,
    nikon_lens_types::NIKON_LENS_TYPES,
    // ... other generated tables
};

/// Apply PrintConv to transform raw value to human-readable string
pub fn apply_print_conv(value: &ExifValue, id: PrintConvId) -> String {
    match id {
        PrintConvId::None => format!("{}", value),

        // Simple lookups
        PrintConvId::OnOff => match value.as_u32() {
            Some(0) => "Off".to_string(),
            Some(1) => "On".to_string(),
            _ => format!("{}", value),
        },

        // Shared data lookups - THIS IS HOW CANON_LENS_TYPES GETS USED
        PrintConvId::CanonLensType => {
            if let Some(lens_id) = value.as_u16() {
                // Look up the lens ID in the generated static table
                if let Some(lens_name) = CANON_LENS_TYPES.get(&lens_id) {
                    lens_name.to_string()
                } else {
                    format!("Unknown ({})", lens_id)
                }
            } else {
                format!("{}", value)
            }
        },

        // String expressions
        PrintConvId::Millimeters => format!("{} mm", value),
        PrintConvId::Float1Decimal => {
            if let Some(f) = value.as_f64() {
                format!("{:.1}", f)
            } else {
                format!("{}", value)
            }
        },

        // Function-based conversions
        PrintConvId::ExposureTime => print_exposure_time(value),
        PrintConvId::FNumber => print_f_number(value),

        // ... other conversions
    }
}
```

### 6. Sync Tool Integration (Idempotent Design)

The sync tool orchestrates extraction and code generation with idempotent operations:

```rust
// src/bin/exiftool_sync/extractors/printconv_sync.rs

pub struct PrintConvSyncExtractor {
    perl_script: PathBuf,
    output_dir: PathBuf,
    cache_dir: PathBuf,
}

impl PrintConvSyncExtractor {
    pub fn sync(&self) -> Result<(), Box<dyn Error>> {
        // 1. Extract PrintConv data (always run to catch ExifTool updates)
        let extracted_data = self.extract_printconv_data()?;

        // 2. Check if anything changed using content hash
        let new_hash = calculate_hash(&extracted_data);
        let cached_hash = self.read_cached_hash()?;

        if new_hash == cached_hash {
            println!("PrintConv data unchanged, skipping regeneration");
            return Ok(());
        }

        // 3. Generate code only if data changed
        self.generate_all_code(&extracted_data)?;

        // 4. Save hash for next run
        self.write_cached_hash(&new_hash)?;

        // 5. Report results
        self.report_sync_results(&extracted_data)?;

        Ok(())
    }

    fn extract_printconv_data(&self) -> Result<ExtractedData, Box<dyn Error>> {
        // Run Perl script to extract all modules
        let output = Command::new("perl")
            .arg(&self.perl_script)
            .arg("--all-modules")  // TODO: implement in Perl script
            .output()?;

        // Parse JSON with deterministic ordering
        let mut extracted: ExtractedData = serde_json::from_slice(&output.stdout)?;
        extracted.sort_deterministic();  // Sort all data for consistency

        Ok(extracted)
    }

    fn generate_all_code(&self, data: &ExtractedData) -> Result<(), Box<dyn Error>> {
        // Group by shared data
        let grouped = self.group_by_shared_data(data);

        // Generate lookup tables (only if changed)
        for (name, lookup_data) in grouped.shared_lookups {
            let output_path = self.output_dir
                .join("generated")
                .join(format!("{}.rs", name.to_lowercase()));

            // Skip if file exists and content matches
            if self.file_content_matches(&output_path, &lookup_data)? {
                continue;
            }

            self.generate_lookup_table(&output_path, &name, &lookup_data)?;
        }

        // Update tag tables with PrintConvId mappings
        self.update_tag_tables(data)?;

        Ok(())
    }

    fn file_content_matches(&self, path: &Path, data: &impl Serialize) -> Result<bool, Box<dyn Error>> {
        if !path.exists() {
            return Ok(false);
        }

        let existing = fs::read_to_string(path)?;
        let new_content = self.format_lookup_table(data)?;

        Ok(existing == new_content)
    }
}
```

### 7. Manual Override System

For complex PrintConv that can't be automatically mapped:

```toml
# printconv_overrides.toml
[overrides]
# GPS coordinates need custom formatting
"EXIF::GPSLatitude" = "GPSCoordinate"
"EXIF::GPSLongitude" = "GPSCoordinate"

# Complex bitfield decoding
"Canon::AFInfo" = "CanonAFInfo"

# Binary data parsing
"Nikon::ColorBalance" = "NikonColorBalance"

[unmapped_ok]
# These are OK to leave as None (rarely used or deprecated)
tags = ["OldSubfileType", "YCbCrCoefficients"]
```

## Implementation Workflow

### Build System Considerations

The implementation handles the "chicken and egg" problem of generated files not existing on first build through:

1. **Feature flags**: Use `#[cfg(feature = "generated-tables")]` to conditionally compile generated imports
2. **Stub files**: Create minimal stub files with empty tables for initial compilation
3. **Build script**: Detect presence of generated files and enable features accordingly

### Single Command Sync (Idempotent)

```bash
# Run sync - safe to run multiple times
make sync

# Or directly:
cargo run --bin exiftool_sync extract printconv-sync
```

This single command:

1. Extracts PrintConv data from all ExifTool modules using `--all-modules`
2. Compares with cached state to detect changes (idempotent)
3. Regenerates only changed files
4. Emits sync issues for unmapped patterns to `sync-todos.jsonl`
5. Reports statistics and unmapped pattern count

### For Engineers Adding New PrintConv Patterns

1. **Identify the Perl pattern** from unmapped report:

   ```bash
   cargo run --bin exiftool_sync report-unmapped
   ```

2. **Add pattern mapping**:

   - For string expressions: Add to `PERL_STRING_PATTERNS` in code above
   - For hash patterns: Add to `HASH_PATTERNS` in code above
   - For new shared tables: Add case to `from_extraction()`

3. **Implement the conversion** in `apply_print_conv()`:

   ```rust
   PrintConvId::YourNewPattern => {
       // Your conversion logic here
   }
   ```

4. **Re-run sync** to apply everywhere:
   ```bash
   make sync
   ```

### For ExifTool Updates

Simply run sync after updating ExifTool:

```bash
# Update ExifTool submodule
cd third-party/exiftool
git pull origin master
cd ../..

# Run sync to incorporate changes
make sync
```

The sync tool will:

- Detect new/changed PrintConv patterns
- Generate updated lookup tables
- Report any new unmapped patterns
- Preserve all manual mappings

## Key Design Decisions

1. **Perl for Extraction**: Only Perl can reliably parse Perl
2. **Lookup Tables for Patterns**: Fast O(1) pattern matching at build time
3. **Generated + Manual**: Generated tables for data, manual code for logic
4. **Content-Based Change Detection**: Only regenerate when data actually changes
5. **Deterministic Output**: Sorted, canonical JSON ensures consistent results

## Success Metrics

- Zero PrintConv data lost during sync
- 95%+ of PrintConv automatically mapped to correct PrintConvId
- All shared lookup tables (lenses, etc.) fully extracted
- Sub-second incremental sync (when no changes)
- Single command workflow
- Complete audit trail of unmapped PrintConv for review

## Additional Tools

### PrintConv Safety Analyzer

A separate tool analyzes PrintConv safety across all ExifTool contexts:

```bash
cargo run --bin exiftool_sync analyze printconv-safety
```

This tool:

- Uses Perl introspection via `scripts/analyze_printconv_safety.pl`
- Identifies safe universal patterns vs collision risks
- Generates CSV report with recommendations
- Emits sync issues for discovered problems
- Helps prioritize which PrintConv patterns to implement first

The safety analyzer is critical for avoiding bugs where the same tag name has different PrintConv implementations in different contexts (EXIF vs MakerNote vs XMP).
