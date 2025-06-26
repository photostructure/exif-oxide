# PrintConv Synchronization Design v2.0

## Overview

ExifTool PrintConv ("Print Conversion") transforms raw EXIF values into human-readable strings. ExifTool has ~50,000 lines of PrintConv code across all manufacturers. This design enables automatic synchronization using Perl introspection to extract the actual PrintConv logic, rather than parsing Perl syntax with regex.

## Problem Statement

1. **Scale**: 5,000+ EXIF tags, each potentially having PrintConv logic
2. **Complexity**: PrintConv includes hash lookups, subroutines, shared lookup tables, and complex expressions
3. **Shared Data**: Lens databases (`%canonLensTypes`, `%nikonLensTypes`) used by multiple tags
4. **Updates**: Monthly ExifTool releases add new tags requiring PrintConv
5. **Current Issue**: Regex-based Perl parsing cannot handle references like `\%canonLensTypes`

## Solution Architecture

### 1. Perl-Based Extraction Layer

Use Perl to introspect ExifTool modules and extract PrintConv data. The extraction script is implemented in [`scripts/extract_printconv.pl`](../scripts/extract_printconv.pl).

Key features of the extraction script:
- Uses Perl to directly load and introspect ExifTool modules
- Handles all PrintConv types: hash references (`\%canonLensTypes`), direct hashes, code references, and strings
- Extracts shared lookup tables (e.g., lens databases with 500+ entries)
- Outputs clean JSON for consumption by Rust sync tools
- Identifies known function references (PrintExposureTime, PrintFraction, etc.)

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
        "3": "Canon EF 135mm f/2.8 Soft",
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
      "tag_name": "InteropVersion",
      "module": "Image::ExifTool::Exif",
      "printconv_type": "string",
      "printconv_source": "'$val =~ s/\0+$//; $val'"
    }
  ]
}
```

### 3. Rust Code Generation

Transform the extracted data into efficient Rust code:

```rust
// Generated file: src/tables/canon_lens_types.rs
// EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm %canonLensTypes

use phf::phf_map;

pub static CANON_LENS_TYPES: phf::Map<u16, &'static str> = phf_map! {
    0u16 => "n/a",
    1u16 => "Canon EF 50mm f/1.8",
    2u16 => "Canon EF 28mm f/2.8",
    3u16 => "Canon EF 135mm f/2.8 Soft",
    // ... generated from printconv_data
};
```

### 4. PrintConv ID System

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
    
    // Complex expressions
    GPSCoordinate,      // Complex DMS conversion
    Flash,              // Bitfield decoding
}

impl PrintConvId {
    /// Determine PrintConvId from extracted JSON data
    pub fn from_extraction(data: &ExtractedPrintConv) -> Self {
        match &data.printconv_type {
            "none" => PrintConvId::None,
            
            "hash" => {
                // Analyze hash data to find known patterns
                if let Some(map) = &data.printconv_data {
                    if map.get("0") == Some("Off") && map.get("1") == Some("On") {
                        return PrintConvId::OnOff;
                    }
                    // ... check other patterns
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
            
            "code_ref" | "string" => {
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
```

### 5. Sync Tool Integration

The sync tool orchestrates the extraction and code generation:

```rust
// src/bin/exiftool_sync/extractors/printconv_sync.rs

pub struct PrintConvSyncExtractor {
    perl_script: PathBuf,
    output_dir: PathBuf,
}

impl PrintConvSyncExtractor {
    pub fn extract_all(&self) -> Result<(), Box<dyn Error>> {
        // 1. Run Perl extraction script
        let output = Command::new("perl")
            .arg(&self.perl_script)
            .arg("--all-modules")
            .output()?;
        
        // 2. Parse JSON output
        let extracted: ExtractedData = serde_json::from_slice(&output.stdout)?;
        
        // 3. Group by shared data (e.g., all tags using canonLensTypes)
        let grouped = self.group_by_shared_data(&extracted);
        
        // 4. Generate lookup tables for shared data
        for (name, data) in grouped.shared_lookups {
            self.generate_lookup_table(&name, &data)?;
        }
        
        // 5. Generate tag definitions with PrintConvId
        self.generate_tag_tables(&extracted)?;
        
        // 6. Report unmapped PrintConv for manual review
        self.report_unmapped(&extracted)?;
        
        Ok(())
    }
    
    fn generate_lookup_table(&self, name: &str, data: &HashMap<String, String>) -> Result<(), Box<dyn Error>> {
        // Generate Rust phf::Map for large lookup tables
        let mut file = File::create(self.output_dir.join(format!("{}.rs", name.to_lowercase())))?;
        
        writeln!(file, "// Generated from ExifTool {}", name)?;
        writeln!(file, "use phf::phf_map;")?;
        writeln!(file, "")?;
        writeln!(file, "pub static {}: phf::Map<u16, &'static str> = phf_map! {{", name.to_uppercase())?;
        
        for (key, value) in data {
            if let Ok(num) = key.parse::<u16>() {
                writeln!(file, "    {}u16 => {:?},", num, value)?;
            }
        }
        
        writeln!(file, "}};")?;
        Ok(())
    }
}
```

### 6. Manual Override System

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

### Initial Setup

1. **Run extraction using the existing script**:
   ```bash
   scripts/extract_printconv.pl --all-modules > printconv_data.json
   ```

3. **Generate Rust code**:
   ```bash
   cargo run --bin exiftool_sync extract-printconv
   ```

### For Engineers Adding PrintConv

1. **Check if pattern exists**:
   ```bash
   cargo run --bin exiftool_sync analyze-printconv --tag LensType
   ```

2. **If new pattern needed**:
   - Add to PrintConvId enum
   - Implement conversion logic
   - Update `from_extraction` mapping

3. **Regenerate**:
   ```bash
   make sync-printconv
   ```

### For ExifTool Updates

1. **Extract new version**:
   ```bash
   scripts/extract_printconv.pl --all-modules > printconv_data_new.json
   ```

2. **Diff changes**:
   ```bash
   cargo run --bin exiftool_sync diff-printconv printconv_data.json printconv_data_new.json
   ```

3. **Review unmapped**:
   ```bash
   cargo run --bin exiftool_sync report-unmapped
   ```

## Key Advantages

1. **100% Accurate**: Using Perl to understand Perl eliminates parsing errors
2. **Complete Data**: Extracts full lens databases and lookup tables
3. **Type Safe**: Generated Rust code with compile-time verification
4. **Maintainable**: Clear separation between extraction and code generation
5. **Automated**: New tags automatically get PrintConv if they match known patterns
6. **Transparent**: JSON intermediate format for debugging and analysis

## Success Metrics

- Zero PrintConv data lost during sync
- 95%+ of PrintConv automatically mapped to correct PrintConvId
- All shared lookup tables (lenses, etc.) fully extracted
- Sub-second incremental sync for ExifTool updates
- Complete audit trail of unmapped PrintConv for review