# Field Patching and Extraction Enhancement
**Date**: 2025-08-09  
**Engineers**: mrm, Claude

## Problem Statement

The codegen system was intermittently failing to generate `xlat_0.rs` and `xlat_1.rs` files from Nikon.pm's `@xlat` arrays. The issue manifested as:
- Files generated successfully when running `cargo run` directly in codegen/
- Files missing after `make clean-all precommit`
- Build failures due to unresolved imports for missing xlat files

## Root Causes Identified

### 1. Field Extractor Limited to Hashes
The `field_extractor.pl` script only extracted hash variables from ExifTool modules:
```perl
# Original code - arrays completely ignored
if ( my $hash_ref = *$glob{HASH} ) {
    extract_hash_symbol(...);
}
# No check for *$glob{ARRAY}
```

### 2. Patcher Overly Aggressive
The `patch_exiftool_modules_universal.pl` script was converting ALL `my` declarations to `our`, including:
- Variables inside subroutines (should remain `my`)
- Variables with indentation (likely inside blocks)

This caused compilation errors like:
```
Variable "%types" is not imported at ExifTool.pm line 10143
```

Because variables declared inside subroutines were being given symbol table aliases before they existed at package scope.

## Solution Implementation

### 1. Enhanced Field Extractor
Added array extraction support to `field_extractor.pl`:

```perl
# Check for arrays in addition to hashes
elsif ( my $array_ref = *$glob{ARRAY} ) {
    if (@$array_ref) {
        extract_array_symbol($symbol_name, $array_ref, $module_name);
    }
}

# New function to handle array extraction
sub extract_array_symbol {
    my ( $symbol_name, $array_ref, $module_name ) = @_;
    
    my $filtered_data = filter_code_refs($array_ref);
    my $extracted = {
        name   => $symbol_name,
        module => $module_name,
        type   => 'array',
        data   => $filtered_data
    };
    
    my $json_data = encode_json($extracted);
    print "$json_data\n";
}
```

### 2. Refined Patcher Logic
Fixed `patch_exiftool_modules_universal.pl` to be more selective:

```perl
# Only convert package-level declarations (no indentation)
if ( !$in_sub && $line =~ /^my\s+([%@])(\w+)\s*=/ ) {
    push @vars_to_export, { sigil => $1, name => $2 };
}

# Substitution also requires no indentation
$content =~ s/^my(\s+[%@](?:$var_pattern)\s*=)/our$1/gm;
```

### 3. Correct Symbol Table Export Placement
Symbol table aliases must go at the end of executable Perl code:

```perl
# Find insertion point - at the very end of Perl code
my $end_marker_pos = index( $content, "\n__END__" );

if ( $end_marker_pos > -1 ) {
    # Insert before __END__
    $insert_point = $end_marker_pos;
} else {
    # No __END__, append at the very end
    $insert_point = length($content);
}
```

## Key Insights

### Pattern Recognition Is Critical
The regex `^(\s*)my` was matching indented variables inside subroutines. Changing to `^my` ensures only package-level variables are converted.

### Symbol Table Timing Matters
Creating symbol table aliases (`*name = \@name;`) for variables before they're declared causes compilation errors. Aliases must be placed after all variable declarations.

### Arrays Were Completely Overlooked
The original field_extractor only checked for `*$glob{HASH}`, never `*$glob{ARRAY}`. This was a fundamental oversight that prevented any array extraction.

## Testing & Verification

After fixes:
```bash
# Patched Nikon.pm shows proper conversion
grep "^our @xlat" third-party/exiftool/lib/Image/ExifTool/Nikon.pm
# Output: our @xlat = (

# Symbol table alias at end of file
grep "*xlat = " third-party/exiftool/lib/Image/ExifTool/Nikon.pm  
# Output: *xlat = \@xlat;

# Field extractor now finds xlat
perl codegen/scripts/field_extractor.pl third-party/exiftool/lib/Image/ExifTool/Nikon.pm | grep xlat
# Output: {"name":"xlat","module":"Nikon","type":"array","data":[[193,191,109...]]}

# Codegen successfully generates files
make codegen && ls src/generated/nikon/xlat*.rs
# Output: xlat_0.rs xlat_1.rs
```

## Impact

This fix enables:
1. Reliable extraction of array variables from ExifTool modules
2. Proper Nikon decryption support via xlat arrays
3. Foundation for extracting other array-based data structures
4. More robust patching that doesn't break ExifTool module compilation

## Lessons for Future Development

1. **Test Both Data Types**: When building extractors, always test with both hashes AND arrays
2. **Scope Matters**: Package-level vs subroutine-level variables require different handling
3. **Symbol Table Mechanics**: Understanding Perl's symbol table and when variables become visible is crucial
4. **Incremental Testing**: Test each component (patcher, extractor, codegen) independently
5. **Debug Output**: The engineers' initial debugging was hampered by lack of visibility - adding DEBUG flags helps

## Files Modified

- `/codegen/scripts/patch_exiftool_modules_universal.pl` - Fixed to only patch package-level variables
- `/codegen/scripts/field_extractor.pl` - Enhanced to extract arrays in addition to hashes

## Related Documentation

- [CODEGEN.md](../CODEGEN.md) - Overall codegen architecture
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture including unified strategy pattern