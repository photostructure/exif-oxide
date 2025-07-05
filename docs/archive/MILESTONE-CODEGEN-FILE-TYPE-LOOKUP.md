# ✅ COMPLETED: Milestone: File Type Lookup Codegen

**Duration**: 2-3 weeks  
**Goal**: Automate extraction and generation of ExifTool's core file type detection infrastructure

## Executive Summary

ExifTool's `fileTypeLookup` hash in `lib/Image/ExifTool.pm` (lines 225-580) contains **~344 entries** that define file extension to format mappings - the backbone of ExifTool's file type detection system. This milestone implements automated extraction and Rust code generation for this critical infrastructure, eliminating manual maintenance of hundreds of file type definitions.

### Current Manual Burden

- 344+ file type entries requiring manual transcription
- Complex discriminated union pattern (aliases vs definitions)
- Monthly ExifTool updates adding new file types
- Error-prone manual sync with format evolution

### Proposed Solution

Extend the simple table extraction framework with a specialized `file_type_lookup` extraction type that understands the unique structure and generates type-safe Rust code with helper functions for alias resolution.

## Technical Analysis

### Data Structure Pattern

The `fileTypeLookup` hash uses a **discriminated union pattern**:

```perl
%fileTypeLookup = (
    # Simple alias (string value)
    '3GP2' => '3G2',
    'AIFC' => 'AIFF',

    # Definition (array value)
    '360' => ['MOV', 'GoPro 360 video'],
    'AIFF' => ['AIFF', 'Audio Interchange File Format'],

    # Multiple format support
    'AI' => [['PDF','PS'], 'Adobe Illustrator'],
    'DOCX' => [['ZIP','FPX'], 'Office Open XML Document'],
);
```

**Value Types**:

1. **String**: Extension alias requiring follow-up lookup
2. **Array**: `[format_info, description]` where `format_info` can be:
   - String: Single format type
   - Array: Multiple possible formats (priority order)

### Complexity Assessment

- **High-value infrastructure**: Core to ExifTool's file detection
- **Well-defined semantics**: Clear patterns despite complexity
- **Stable interface**: Pattern unchanged for years
- **344 entries**: Substantial maintenance burden if manual
- **Monthly updates**: New file types added regularly

## Implementation Strategy

### Phase 1: Extraction Framework Enhancement (Week 1)

**Extend Configuration Schema**

```json
{
  "module": "ExifTool.pm",
  "hash_name": "%fileTypeLookup",
  "output_file": "core/file_types.rs",
  "rust_enum_name": "FileTypeEntry",
  "helper_fn_name": "resolve_file_type",
  "extraction_type": "file_type_lookup",
  "description": "Core file type detection lookup table"
}
```

**Enhance extract_simple_tables.pl**

- New `extract_file_type_lookup_entries()` function
- Parse string values as extension aliases
- Parse array values as `[format_info, description]` tuples
- Handle `format_info` as string OR array of strings
- Validate structure and detect malformed entries

```perl
sub extract_file_type_lookup_entries {
    my ($hash_ref) = @_;

    my @entries;
    for my $extension (keys %$hash_ref) {
        my $value = $hash_ref->{$extension};

        if (!ref $value) {
            # String alias
            push @entries, {
                extension => $extension,
                entry_type => 'alias',
                target => $value,
            };
        } elsif (ref $value eq 'ARRAY' && @$value == 2) {
            # [format_info, description] definition
            my ($format_info, $description) = @$value;
            my $formats = ref $format_info eq 'ARRAY' ? $format_info : [$format_info];

            push @entries, {
                extension => $extension,
                entry_type => 'definition',
                formats => $formats,
                description => $description,
            };
        } else {
            warn "Skipping malformed entry: $extension\n";
        }
    }

    return @entries;
}
```

### Phase 2: Rust Code Generation (Week 2)

**Generated Enum Structure**

```rust
//! Generated file type lookup infrastructure
//!
//! Source: ExifTool lib/Image/ExifTool.pm %fileTypeLookup
//! Description: Core file type detection lookup table

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq)]
pub enum FileTypeEntry {
    /// Extension alias requiring follow-up lookup
    Alias(String),
    /// File type definition with formats and description
    Definition {
        formats: Vec<String>,
        description: String
    },
}

/// Core file type lookup table
/// Source: ExifTool %fileTypeLookup (344 entries)
pub static FILE_TYPE_LOOKUP: LazyLock<HashMap<&'static str, FileTypeEntry>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // Aliases
        map.insert("3GP2", FileTypeEntry::Alias("3G2".to_string()));
        map.insert("AIFC", FileTypeEntry::Alias("AIFF".to_string()));

        // Definitions
        map.insert("360", FileTypeEntry::Definition {
            formats: vec!["MOV".to_string()],
            description: "GoPro 360 video".to_string(),
        });
        map.insert("AIFF", FileTypeEntry::Definition {
            formats: vec!["AIFF".to_string()],
            description: "Audio Interchange File Format".to_string(),
        });
        map.insert("AI", FileTypeEntry::Definition {
            formats: vec!["PDF".to_string(), "PS".to_string()],
            description: "Adobe Illustrator".to_string(),
        });

        // ... 340+ more entries
        map
    });
```

**Helper Functions**

```rust
/// Resolve file type with alias following
/// Returns (formats, description) tuple if found
pub fn resolve_file_type(extension: &str) -> Option<(Vec<String>, String)> {
    const MAX_ALIAS_DEPTH: u8 = 10; // Prevent infinite loops

    let mut current_ext = extension;
    let mut depth = 0;

    while depth < MAX_ALIAS_DEPTH {
        match FILE_TYPE_LOOKUP.get(current_ext.to_uppercase().as_str()) {
            Some(FileTypeEntry::Alias(target)) => {
                current_ext = target;
                depth += 1;
            },
            Some(FileTypeEntry::Definition { formats, description }) => {
                return Some((formats.clone(), description.clone()));
            },
            None => return None,
        }
    }

    None // Alias chain too deep or circular
}

/// Get primary format for extension (first in formats list)
pub fn get_primary_format(extension: &str) -> Option<String> {
    resolve_file_type(extension)
        .and_then(|(formats, _)| formats.into_iter().next())
}

/// Check if extension supports a specific format
pub fn supports_format(extension: &str, format: &str) -> bool {
    resolve_file_type(extension)
        .map(|(formats, _)| formats.iter().any(|f| f == format))
        .unwrap_or(false)
}

/// Get all supported extensions for a format
pub fn extensions_for_format(format: &str) -> Vec<String> {
    FILE_TYPE_LOOKUP
        .iter()
        .filter_map(|(ext, entry)| {
            if let FileTypeEntry::Definition { formats, .. } = entry {
                if formats.contains(&format.to_string()) {
                    Some(ext.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
```

### Phase 3: Integration and Testing (Week 3)

**Integration with Main Codegen Pipeline**

```rust
// Add to main.rs generate_simple_tables function
fn generate_file_type_lookup_code(hash_name: &str, table_data: &ExtractedTable) -> Result<String> {
    // Generate enum + static HashMap + helper functions
    // Handle alias vs definition entries appropriately
    // Include comprehensive documentation
}
```

**Validation Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_resolution() {
        // Test: 3GP2 -> 3G2 -> [MOV, "3rd Gen..."]
        let result = resolve_file_type("3GP2");
        assert!(result.is_some());
        let (formats, desc) = result.unwrap();
        assert_eq!(formats[0], "MOV");
        assert!(desc.contains("3rd Gen"));
    }

    #[test]
    fn test_direct_definition() {
        let result = resolve_file_type("AIFF");
        assert!(result.is_some());
        let (formats, desc) = result.unwrap();
        assert_eq!(formats[0], "AIFF");
        assert_eq!(desc, "Audio Interchange File Format");
    }

    #[test]
    fn test_multiple_formats() {
        let result = resolve_file_type("AI");
        assert!(result.is_some());
        let (formats, _) = result.unwrap();
        assert_eq!(formats, vec!["PDF", "PS"]);
    }

    #[test]
    fn test_unknown_extension() {
        assert!(resolve_file_type("UNKNOWN").is_none());
    }

    #[test]
    fn test_circular_alias_protection() {
        // Would need to inject a circular reference for testing
        // Should return None rather than infinite loop
    }
}
```

**Performance Validation**

- Benchmark lookup performance vs manual HashMap
- Memory usage comparison
- Alias resolution overhead measurement

## Success Criteria

### Core Requirements

- [ ] **Complete Extraction**: All 344 fileTypeLookup entries extracted
- [ ] **Type Safety**: Rust enum prevents invalid access patterns
- [ ] **Alias Resolution**: Helper functions correctly follow alias chains
- [ ] **Format Support**: Multi-format entries properly handled
- [ ] **Error Handling**: Malformed entries detected and skipped
- [ ] **Performance**: Lookup speed comparable to manual implementation

### Generated Code Quality

- [ ] **Documentation**: Comprehensive rustdoc with examples
- [ ] **Testing**: 100% test coverage for helper functions
- [ ] **Integration**: Seamless codegen pipeline integration
- [ ] **Maintenance**: Zero manual updates required for ExifTool releases

### Validation Tests

- [ ] **Regression**: Existing simple tables continue working
- [ ] **Compatibility**: Generated lookups match ExifTool behavior
- [ ] **Edge Cases**: Circular aliases, missing targets handled gracefully
- [ ] **Integration**: Successfully replaces any manual file type code

## Dependencies and Prerequisites

### Required Reading

- [CODEGEN.md](../design/CODEGEN.md) - Simple table extraction framework
- [ExifTool.pm lines 225-580](../../third-party/exiftool/lib/Image/ExifTool.pm) - Source data structure
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Implementation fidelity principles

### Technical Dependencies

- Enhanced simple table extraction framework (already implemented)
- Perl JSON module for boolean handling
- Rust HashMap/LazyLock for static data
- serde support for serialization (optional)

### Development Environment

```bash
# Test current extraction
cd codegen && perl extract_simple_tables.pl

# Validate Rust generation
cargo run -- test_minimal.json --output-dir test_output

# Check ExifTool source
grep -A 10 "fileTypeLookup" third-party/exiftool/lib/Image/ExifTool.pm
```

## Risk Mitigation

### Data Structure Evolution Risk

- **Risk**: ExifTool changes fileTypeLookup structure
- **Mitigation**: Comprehensive validation during extraction
- **Monitoring**: Test against each ExifTool release

### Alias Chain Complexity Risk

- **Risk**: Circular references or deep alias chains
- **Mitigation**: Max depth protection and cycle detection
- **Testing**: Inject problematic cases in test suite

### Performance Impact Risk

- **Risk**: Generated code slower than manual implementation
- **Mitigation**: Benchmark against hand-coded HashMap
- **Optimization**: LazyLock for one-time initialization cost

### Integration Complexity Risk

- **Risk**: Breaks existing simple table extraction
- **Mitigation**: Comprehensive regression testing
- **Rollback**: Feature flag to disable file_type_lookup

## Future Enhancements

### Potential Extensions

- **Write support**: Generate file extension mapping for writing
- **Priority handling**: Implement format priority for multi-format files
- **Dynamic updates**: Runtime fileTypeLookup updates
- **Compression**: Optimize memory usage for embedded systems

### Related Opportunities

- Other complex ExifTool hashes with similar patterns
- Configuration-driven extraction for custom data structures
- Cross-language code generation (Python, C++, etc.)

### Recommended Refactoring (Future Work)

**Dedicated Extraction Scripts**: The current implementation extends the generic `extract_simple_tables.pl` to handle the file_type_lookup discriminated union pattern. A cleaner approach would be to create dedicated extraction scripts for specific patterns:

- `extract_file_type_lookup.pl` - Dedicated to fileTypeLookup pattern
- `extract_magic_numbers.pl` - Dedicated to magicNumber regex patterns
- `extract_simple_lookup.pl` - For basic key-value tables

This would:

- Simplify each script's logic and improve maintainability
- Allow pattern-specific optimizations
- Make it easier to add new extraction types
- Reduce the complexity of validation logic

The current single-script approach works but becomes unwieldy as more extraction types are added.

## Engineer Onboarding Guide

### Getting Started (Day 1)

1. **Read documentation**: CODEGEN.md, TRUST-EXIFTOOL.md
2. **Study source data**: ExifTool.pm lines 225-580
3. **Run existing pipeline**: Test simple table extraction
4. **Examine generated code**: Review existing XMP structures

### Week 1 Focus

- Understand discriminated union pattern in Perl
- Map data structure to Rust enum design
- Implement Perl extraction functions
- Validate against small subset

### Week 2 Focus

- Design Rust enum and helper functions
- Implement code generation logic
- Create comprehensive test suite
- Optimize for performance

### Week 3 Focus

- Integration with main pipeline
- Regression testing
- Documentation and examples
- Performance validation

### Key Success Metrics

- All 344 entries successfully extracted
- Zero manual intervention required
- Helper functions pass all test cases
- Performance within 10% of manual code
- Ready for production ExifTool updates

This milestone represents a critical infrastructure improvement that eliminates manual maintenance burden while providing type-safe, performant file type detection capabilities for the exif-oxide project.
