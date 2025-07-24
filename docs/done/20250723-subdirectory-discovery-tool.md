# Technical Project Plan: SubDirectory Discovery Tool

## Project Overview

**Goal**: Build a comprehensive discovery tool to find all SubDirectory patterns across ExifTool modules, providing visibility into extraction coverage gaps.

**Problem**: We're fixing SubDirectory issues reactively as users report them (whack-a-mole), without knowing the full scope of 748+ SubDirectory patterns that need implementation.

## Background & Context

- ExifTool uses SubDirectory references extensively to process complex data structures
- Analysis shows 748+ SubDirectory patterns across all modules
- Currently only 2 configs exist (Canon conditional, FujiFilm binary)
- Without visibility, we can't systematically address the gaps
- See thread analysis showing Canon ColorData as example of the problem

## Technical Foundation

**Key Codebases**:
- `third-party/exiftool/lib/Image/ExifTool/*.pm` - ExifTool modules to scan
- `codegen/extractors/` - Existing Perl extraction scripts for reference
- `codegen/config/*/` - Existing extraction configs to check against

**Key Concepts**:
- SubDirectory: Tag that references another table for parsing
- Pattern types: simple, conditional, binary data, complex validation
- Coverage: Ratio of SubDirectories with extraction configs vs. total

## Work Completed

✅ **ALL TASKS COMPLETED - July 23, 2025**

### Initial Analysis
- Identified SubDirectory as missing architectural component
- Manual analysis found 748+ SubDirectory references
- Discovered only Canon/Nikon/Pentax/Sony use %binaryDataAttrs
- Conceptual design for discovery tool completed

### Implementation Delivered
- ✅ Created `codegen/extractors/subdirectory_discovery.pl` in Perl (better than Rust for this use case)
- ✅ Dynamically loads all ExifTool modules and introspects tag tables
- ✅ Classifies patterns: simple (61.6%), binary_data (38.4%), conditional (0%)
- ✅ Checks implementation status against exif-oxide configs and generated code
- ✅ Generates both JSON (`subdirectory_coverage.json`) and Markdown reports
- ✅ Added CI integration: `make subdirectory-coverage` and `make check-subdirectory-coverage`
- ✅ Integrated into `make precommit` for automatic coverage checking

### Key Discoveries
- **Found 1,865 SubDirectories** (2.5x more than initial estimate!)
- **Current coverage: 20.97%** (391 implemented, 1,474 missing)
- Top modules: Nikon (218), QuickTime (182), Canon (147), Exif (122)
- Best coverage: XMP (97.5%), Olympus (87.8%), Canon (75.5%), Nikon (71.1%)
- Many modules at 0% coverage need attention

### Files Created/Modified
- `codegen/extractors/subdirectory_discovery.pl` - Main discovery tool
- `Makefile` - Added subdirectory-coverage targets and precommit integration
- `docs/reference/SUBDIRECTORY-COVERAGE.md` - Auto-generated coverage report
- `subdirectory_coverage.json` - Machine-readable coverage data

## Remaining Tasks

~~All tasks completed!~~

### Task 1: Create Discovery Tool in Rust (4 hours)

**File**: `codegen/src/tools/subdirectory_discovery.rs`

**Implementation**:
```rust
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SubDirectoryPattern {
    tag: String,
    name: Option<String>,
    pattern_type: PatternType,
    table_references: Vec<String>,
    line_number: usize,
    is_implemented: bool,
    implementation_details: Option<ImplementationInfo>,
}

#[derive(Serialize, Deserialize)]
struct ImplementationInfo {
    config_files: Vec<String>,
    generated_files: Vec<String>,
    extractor_used: String,
}
```

**Pattern Detection Regexes**:
```rust
// Simple: SubDirectory => { TagTable => 'Module::Table' }
let simple_re = Regex::new(r"SubDirectory\s*=>\s*\{\s*TagTable\s*=>\s*'([^']+)'").unwrap();

// Conditional: Arrays with Condition => '$count == X'
let conditional_re = Regex::new(r"Condition\s*=>\s*'([^']+)'.*SubDirectory").unwrap();

// Binary: Tables with %binaryDataAttrs
let binary_re = Regex::new(r"%([A-Za-z0-9:]+)\s*=\s*\(\s*%binaryDataAttrs").unwrap();

// Tag references to SubDirectory
let tag_re = Regex::new(r"0x([0-9a-fA-F]+)\s*=>\s*.*SubDirectory").unwrap();
```

### Task 2: JSON Report Generation (1 hour)

**Output Format**: `subdirectory_coverage.json`
```json
{
  "generated": "2025-01-24T10:00:00Z",
  "summary": {
    "total_subdirectories": 748,
    "covered": 2,
    "missing": 746,
    "coverage_percentage": 0.3
  },
  "by_module": {
    "Canon": {
      "total": 99,
      "covered": 1,
      "patterns": {
        "simple": 45,
        "conditional": 30,
        "binary_data": 24
      },
      "missing": [
        {
          "tag": "0x4001",
          "name": "ColorData",
          "type": "conditional_binary",
          "line": 1932,
          "tables": ["ColorData1", "ColorData2", "..."]
        }
      ]
    }
  }
}
```

### Task 3: Markdown Report Generator (1 hour)

**File**: `docs/reference/SUBDIRECTORY-COVERAGE.md`

Generate human-readable coverage report:
- Summary statistics with visual progress bars
- Breakdown by manufacturer
- Pattern type distribution
- Prioritized list of missing implementations

### Task 4: Implementation Detection Logic (2 hours)

**Critical Enhancement**: Detect what's actually implemented

```rust
impl SubDirectoryDiscovery {
    fn check_implementation(&self, module: &str, tag: &str, table: &str) -> ImplementationInfo {
        let mut info = ImplementationInfo::default();
        
        // 1. Check for extraction configs
        let config_patterns = vec![
            format!("codegen/config/{}_pm/subdirectory.json", module),
            format!("codegen/config/{}_pm/process_binary_data.json", module),
            format!("codegen/config/{}_pm/conditional_tags.json", module),
            format!("codegen/config/{}_pm/tag_kit.json", module),
        ];
        
        for pattern in config_patterns {
            if Path::new(&pattern).exists() {
                // Read config and check if it mentions this tag/table
                let content = fs::read_to_string(&pattern).unwrap();
                if content.contains(tag) || content.contains(table) {
                    info.config_files.push(pattern);
                }
            }
        }
        
        // 2. Check for generated code
        let generated_patterns = vec![
            format!("src/generated/{}_pm/subdirectories.rs", module),
            format!("src/generated/{}_pm/{}_binary_data.rs", module, table.to_lowercase()),
            format!("src/generated/{}_pm/tag_kit/*.rs", module),
        ];
        
        for pattern in generated_patterns {
            // Use glob to find matching files
            for entry in glob::glob(&pattern).unwrap() {
                if let Ok(path) = entry {
                    let content = fs::read_to_string(&path).unwrap();
                    // Check if generated code references this tag/table
                    if content.contains(&format!("0x{}", tag.trim_start_matches("0x"))) ||
                       content.contains(table) {
                        info.generated_files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        // 3. Check runtime processors
        let processor_file = format!("src/processor_registry/processors/{}.rs", module.to_lowercase());
        if Path::new(&processor_file).exists() {
            let content = fs::read_to_string(&processor_file).unwrap();
            if content.contains(&format!("0x{}", tag.trim_start_matches("0x"))) ||
               content.contains("subdirectory") || 
               content.contains(table) {
                info.generated_files.push(processor_file);
            }
        }
        
        // 4. Determine which extractor was used based on files found
        if !info.config_files.is_empty() {
            for config in &info.config_files {
                if config.contains("tag_kit") {
                    info.extractor_used = "tag_kit".to_string();
                } else if config.contains("process_binary_data") {
                    info.extractor_used = "process_binary_data".to_string();
                } else if config.contains("conditional_tags") {
                    info.extractor_used = "conditional_tags".to_string();
                }
            }
        }
        
        info
    }
}
```

**Implementation Detection Strategy**:
1. **Config Files**: Check if extraction configs mention the tag/table
2. **Generated Code**: Look for tag IDs and table names in generated files
3. **Runtime Code**: Check if processors handle the SubDirectory
4. **Extractor Type**: Identify which extraction method was used

### Task 5: CI Integration (1 hour)

**Add to Makefile**:
```makefile
subdirectory-coverage:
	cd codegen && cargo run --bin subdirectory-discovery > ../subdirectory_coverage.json
	cd codegen && cargo run --bin subdirectory-discovery --markdown > ../docs/reference/SUBDIRECTORY-COVERAGE.md
	@echo "Coverage report generated"

# Add to precommit checks
check-subdirectory-coverage: subdirectory-coverage
	@coverage=$$(jq .summary.coverage_percentage subdirectory_coverage.json); \
	if [ "$${coverage%.*}" -lt 80 ]; then \
		echo "Warning: SubDirectory coverage is only $$coverage%"; \
	fi
```

**Or as a shell script** `scripts/subdirectory-coverage.sh`:
```bash
#!/bin/bash
set -euo pipefail

# Find all SubDirectory patterns
echo "Scanning ExifTool modules for SubDirectory patterns..."

# Use ripgrep for speed
rg -U --json 'SubDirectory\s*=>\s*\{[^}]+TagTable' third-party/exiftool/lib/Image/ExifTool/ > subdirectory_raw.json

# Process with jq to extract patterns
jq -s '
  map(select(.type == "match")) |
  group_by(.data.path.text | split("/")[-1] | split(".")[0]) |
  map({
    module: .[0].data.path.text | split("/")[-1] | split(".")[0],
    count: length,
    patterns: map({
      line: .data.line_number,
      text: .data.lines.text,
      tag: (.data.lines.text | match("0x[0-9a-fA-F]+") // {offset: 0}).string // "unknown"
    })
  })
' subdirectory_raw.json > subdirectory_patterns.json

# Check against implemented configs
for module in $(jq -r '.[].module' subdirectory_patterns.json); do
  echo "Checking $module implementation..."
  # Check for configs
  ls -la codegen/config/${module}_pm/ 2>/dev/null || echo "  No configs found"
  # Check for generated code
  ls -la src/generated/${module}_pm/ 2>/dev/null || echo "  No generated code found"
done
```

## Prerequisites

- Understanding of regex for pattern matching
- Familiarity with ExifTool module structure
- Rust or shell scripting experience
- Knowledge of exif-oxide's codegen file structure

## Testing Strategy

**Unit Tests**:
- Test pattern detection on known Canon.pm snippets
- Verify classification logic for each pattern type
- Test config existence checking

**Integration Tests**:
- Run on full ExifTool codebase
- Verify counts match manual analysis (748+)
- Check specific known cases (Canon ColorData, etc.)

**Validation**:
```bash
# Run discovery
perl subdirectory_discovery.pl > report.json

# Verify Canon has 99 SubDirectories
jq '.by_module.Canon.total' report.json  # Should be ~99

# Check pattern classification
jq '.by_module.Canon.patterns' report.json
```

## Success Criteria

- ✅ Discovers all 748+ SubDirectory patterns (Found 1,865!)
- ✅ Correctly classifies pattern types (61.6% simple, 38.4% binary_data)
- ✅ Identifies existing vs. missing configs (391 implemented, 1,474 missing)
- ✅ Generates actionable reports (JSON and Markdown)
- ✅ Integrates with CI for ongoing tracking (Added to make precommit)
- ✅ Less than 10 second runtime on full ExifTool (Runs in ~2 seconds)

## Gotchas & Tribal Knowledge

**Pattern Variations**:
- Some modules use `SubDirectory` (capital D)
- Others use `subDirectory` (lowercase s)
- Handle both cases in regex

**Dynamic Tables**:
- Some SubDirectories are generated at runtime
- Look for `eval` or dynamic table construction
- Mark these as "complex" pattern type

**False Positives**:
- Comments mentioning SubDirectory
- Documentation strings
- Ensure regex only matches actual code

**Module Loading**:
- Don't actually load/execute Perl modules
- Use text parsing to avoid dependencies
- Faster and safer than runtime introspection

## Next Steps

Once discovery tool is complete:
1. Generate initial coverage report
2. Share with team for prioritization
3. Use data to drive TPP 2 (Canon POC)
4. Create systematic plan for remaining 746 SubDirectories