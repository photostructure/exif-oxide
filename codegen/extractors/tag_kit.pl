#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         tag_kit.pl
#
# Description:  Extract complete tag definitions including PrintConv from ExifTool
#
# Usage:        perl tag_kit.pl <module_path> <table_name>
#
# Example:      perl tag_kit.pl ../third-party/exiftool/lib/Image/ExifTool/Exif.pm Main
#
# Notes:        This script extracts tag definitions WITH their PrintConv
#               implementations, creating a "tag kit" with everything needed
#               to process a tag.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_module_from_file
    get_package_hash
    validate_primitive_value
    format_json_output
);
use JSON;

# Check arguments
if (@ARGV != 2) {
    die "Usage: $0 <module_path> <table_name>\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Exif.pm Main\n";
}

my ($module_path, $table_name) = @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{}; # Remove path

# Load the module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Get the tag table
my $table_ref = get_package_hash($module_name, $table_name);
unless ($table_ref) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract tag kits
my @tag_kits;
my $extracted_count = 0;
my $skipped_complex = 0;

for my $tag_id (sort keys %$table_ref) {
    # Skip special ExifTool keys (all uppercase or special values)
    next if $tag_id =~ /^[A-Z_]+$/;
    next if $tag_id eq 'Notes';
    
    my $tag_value = $table_ref->{$tag_id};
    
    # Handle conditional arrays (multiple tag definitions with conditions)
    if (ref $tag_value eq 'ARRAY') {
        # Process each conditional variant
        for my $i (0 .. $#$tag_value) {
            my $tag_info = $tag_value->[$i];
            next unless ref $tag_info eq 'HASH';
            
            # Extract conditional tag with variant suffix
            my $variant_id = "${tag_id}_variant_$i";
            process_tag_info($tag_info, $variant_id, $tag_id, \@tag_kits, \$extracted_count, \$skipped_complex);
        }
        next;
    }
    
    # Skip if not a hash reference
    next unless ref $tag_value eq 'HASH';
    
    # Process single tag definition
    process_tag_info($tag_value, $tag_id, $tag_id, \@tag_kits, \$extracted_count, \$skipped_complex);
}

# Output results
my $output = {
    source => {
        module => $module_display_name,
        table => $table_name,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    metadata => {
        total_tags_scanned => scalar(keys %$table_ref),
        tag_kits_extracted => $extracted_count,
        skipped_complex => $skipped_complex,
    },
    tag_kits => \@tag_kits,
};

# Write JSON to stdout (let the codegen system handle file output)
print format_json_output($output);

# Print summary to STDERR
print STDERR "Extracted $extracted_count tag kits from $table_name\n";
print STDERR "Skipped $skipped_complex complex PrintConvs requiring manual implementation\n" if $skipped_complex;

#------------------------------------------------------------------------------
# Extract PrintConv type and data
#------------------------------------------------------------------------------
sub extract_print_conv {
    my ($print_conv) = @_;
    
    # No PrintConv
    return ('None', undef) unless defined $print_conv;
    
    # Simple hash reference
    if (ref $print_conv eq 'HASH') {
        # Check if it's a simple hash (no complex structures)
        if (is_simple_printconv_hash($print_conv)) {
            my %entries;
            for my $key (sort keys %$print_conv) {
                next if $key eq 'OTHER';
                next if $key eq 'Notes';
                next if $key eq 'BITMASK';  # Skip BITMASK for now
                
                my $value = $print_conv->{$key};
                if (validate_primitive_value($value)) {
                    $entries{$key} = $value;
                }
            }
            return ('Simple', \%entries) if %entries;
        }
        # Complex hash - needs manual implementation
        return ('Manual', 'complex_hash_printconv');
    }
    
    # String expression
    if (!ref $print_conv && $print_conv !~ /^\\&/) {
        # Simple expression we might be able to handle
        if (is_simple_expression($print_conv)) {
            return ('Expression', $print_conv);
        }
        # Complex expression
        return ('Manual', 'complex_expression_printconv');
    }
    
    # Code reference
    if (ref $print_conv eq 'CODE' || (!ref $print_conv && $print_conv =~ /^\\&/)) {
        # Extract function name if possible
        if (!ref $print_conv && $print_conv =~ /^\\&(\w+)$/) {
            return ('Manual', $1);
        }
        return ('Manual', 'code_ref_printconv');
    }
    
    # Array reference (usually for bitwise operations)
    if (ref $print_conv eq 'ARRAY') {
        return ('Manual', 'array_printconv');
    }
    
    # Unknown type
    return ('Manual', 'unknown_printconv');
}

#------------------------------------------------------------------------------
# Check if a PrintConv hash is simple (no complex structures)
#------------------------------------------------------------------------------
sub is_simple_printconv_hash {
    my ($print_conv) = @_;
    
    for my $key (keys %$print_conv) {
        my $value = $print_conv->{$key};
        
        # Skip if value is a reference (except for BITMASK which we'll handle specially)
        if (ref $value && $key ne 'BITMASK') {
            return 0;
        }
        
        # Skip if value contains Perl variables
        if (!ref $value && $value =~ /[\$\@\%]/) {
            return 0;
        }
        
        # Skip if value looks like code
        if (!ref $value && $value =~ /\bsub\s*\{/) {
            return 0;
        }
    }
    
    return 1;
}

#------------------------------------------------------------------------------
# Check if an expression is simple enough to translate
#------------------------------------------------------------------------------
sub is_simple_expression {
    my ($expr) = @_;
    
    # Allow simple patterns we can translate:
    # - Ternary operators with simple conditions
    # - sprintf with basic formats
    # - Simple regex matches
    # - Basic arithmetic
    
    # Examples of simple expressions:
    # '$val =~ /^(inf|undef)$/ ? $val : "$val m"'
    # 'sprintf("%.1f", $val)'
    # '$val > 8 ? undef : $val'
    
    # For now, be conservative - only allow specific patterns
    return 1 if $expr =~ /^\$val\s*=~.*\?\s*.*:\s*.*$/;  # Regex with ternary
    return 1 if $expr =~ /^sprintf\s*\(/;                 # sprintf
    return 1 if $expr =~ /^\$val\s*[<>=]+\s*\d+\s*\?/;   # Comparison with ternary
    return 1 if $expr =~ /^".*\$val.*"$/;                 # Simple interpolation
    return 1 if $expr eq 'undef';                         # Literal undef
    
    # Otherwise it's complex
    return 0;
}

#------------------------------------------------------------------------------
# Process a single tag definition
#------------------------------------------------------------------------------
sub process_tag_info {
    my ($tag_info, $variant_id, $original_tag_id, $tag_kits_ref, $extracted_count_ref, $skipped_complex_ref) = @_;
    
    # Get basic tag information
    my $tag_name = $tag_info->{Name} || "Tag$original_tag_id";
    my $format = $tag_info->{Format} || $tag_info->{Writable} || 'unknown';
    # Ensure format is a string
    $format = "$format" if defined $format;
    my $notes = $tag_info->{Notes} || '';
    my $writable = $tag_info->{Writable};
    my $condition = $tag_info->{Condition};
    
    # Extract groups
    my $groups = {};
    if (exists $tag_info->{Groups}) {
        $groups = $tag_info->{Groups};
    }
    
    # Extract PrintConv
    my ($print_conv_type, $print_conv_data) = extract_print_conv($tag_info->{PrintConv});
    
    # Extract ValueConv if present
    my $value_conv = undef;
    if (exists $tag_info->{ValueConv} && !ref($tag_info->{ValueConv})) {
        $value_conv = $tag_info->{ValueConv};
    }
    
    # Extract SubDirectory if present
    my $subdirectory_info = undef;
    if (exists $tag_info->{SubDirectory}) {
        $subdirectory_info = extract_subdirectory($tag_info->{SubDirectory});
    }
    
    # Build tag kit
    my $tag_kit = {
        tag_id => $original_tag_id,
        variant_id => $variant_id,
        name => $tag_name,
        format => $format,
        groups => $groups,
    };
    
    # Add optional fields
    $tag_kit->{writable} = $writable if defined $writable;
    $tag_kit->{notes} = $notes if $notes;
    $tag_kit->{value_conv} = $value_conv if defined $value_conv;
    if (defined $condition) {
        # Normalize whitespace in conditions: trim and collapse multiple spaces
        $condition =~ s/^\s+|\s+$//g;  # trim leading/trailing whitespace
        $condition =~ s/\s+/ /g;        # collapse multiple spaces to single space
        $tag_kit->{condition} = $condition;
    }
    
    # Add SubDirectory info
    if ($subdirectory_info) {
        $tag_kit->{subdirectory} = $subdirectory_info;
    }
    
    # Add PrintConv info
    $tag_kit->{print_conv_type} = $print_conv_type;
    if ($print_conv_type ne 'None') {
        $tag_kit->{print_conv_data} = $print_conv_data;
        $$skipped_complex_ref++ if $print_conv_type eq 'Manual';
    }
    
    push @$tag_kits_ref, $tag_kit;
    $$extracted_count_ref++;
    
    if ($print_conv_type ne 'None') {
        # Comment out debug messages that interfere with stdout
        # print STDERR "  Found tag kit for $tag_name (tag $original_tag_id): PrintConv type = $print_conv_type\n";
    }
}

#------------------------------------------------------------------------------
# Extract SubDirectory information
#------------------------------------------------------------------------------
sub extract_subdirectory {
    my ($subdir) = @_;
    
    return undef unless $subdir;
    
    my $info = {
        tag_table => $subdir->{TagTable} || 'Unknown',
    };
    
    # Add optional fields (but skip CODE references)
    if (exists $subdir->{Validate}) {
        if (!ref($subdir->{Validate})) {
            $info->{validate} = $subdir->{Validate};
        } else {
            $info->{has_validate_code} = JSON::true;
        }
    }
    if (exists $subdir->{ProcessProc}) {
        if (!ref($subdir->{ProcessProc})) {
            $info->{process_proc} = $subdir->{ProcessProc};
        } else {
            $info->{has_process_proc_code} = JSON::true;
        }
    }
    $info->{base} = $subdir->{Base} if exists $subdir->{Base};
    $info->{byte_order} = $subdir->{ByteOrder} if exists $subdir->{ByteOrder};
    
    # Try to extract the referenced table
    if (defined $subdir->{TagTable} && $subdir->{TagTable} ne 'Unknown') {
        my $table_info = extract_subdirectory_table($subdir->{TagTable});
        if ($table_info) {
            $info->{extracted_table} = $table_info;
            $info->{is_binary_data} = $table_info->{is_binary_data} || JSON::false;
        }
    }
    
    return $info;
}

#------------------------------------------------------------------------------
# Extract a SubDirectory table
#------------------------------------------------------------------------------
sub extract_subdirectory_table {
    my ($table_name) = @_;
    
    # Get the table reference
    my $table_ref;
    {
        no strict 'refs';
        $table_ref = \%{$table_name};
        use strict 'refs';
    }
    
    # Skip if table doesn't exist or is empty
    return undef unless $table_ref && %$table_ref;
    
    my $table_info = {
        table_name => $table_name,
        is_binary_data => JSON::false,
        tags => [],
    };
    
    # Check if this is a binary data table
    if (exists $table_ref->{PROCESS_PROC}) {
        # Check if PROCESS_PROC points to ProcessBinaryData
        my $proc = $table_ref->{PROCESS_PROC};
        if (ref($proc) eq 'CODE') {
            # Mark as binary data but don't store the CODE ref
            $table_info->{is_binary_data} = JSON::true;
            $table_info->{has_process_proc} = JSON::true;
        }
    }
    
    # Extract format and first_entry for binary data tables
    if (exists $table_ref->{FORMAT}) {
        $table_info->{format} = $table_ref->{FORMAT};
        $table_info->{is_binary_data} = JSON::true;
    }
    if (exists $table_ref->{FIRST_ENTRY}) {
        $table_info->{first_entry} = $table_ref->{FIRST_ENTRY};
        $table_info->{is_binary_data} = JSON::true;
    }
    
    # Extract groups
    if (exists $table_ref->{GROUPS} && ref($table_ref->{GROUPS}) eq 'HASH') {
        $table_info->{groups} = $table_ref->{GROUPS};
    }
    
    # Extract tag definitions
    for my $tag_id (sort keys %$table_ref) {
        # Skip special keys
        next if $tag_id =~ /^[A-Z_]+$/;
        next if $tag_id eq 'Notes';
        
        my $tag_def = $table_ref->{$tag_id};
        
        # Skip non-hash entries
        next unless ref $tag_def eq 'HASH';
        
        # Extract basic tag info
        my $tag = {
            tag_id => $tag_id,
            name => $tag_def->{Name} || "Tag$tag_id",
        };
        
        # Add optional fields (skip any CODE references)
        if (exists $tag_def->{Format} && !ref($tag_def->{Format})) {
            $tag->{format} = $tag_def->{Format};
        }
        if (exists $tag_def->{Count}) {
            if (!ref($tag_def->{Count})) {
                # Convert to string to ensure JSON compatibility
                $tag->{count} = "$tag_def->{Count}";
            }
        }
        
        # Check for nested SubDirectory
        if (exists $tag_def->{SubDirectory}) {
            $tag->{has_subdirectory} = JSON::true;
            # Don't recurse too deep - just mark that it exists
        }
        
        push @{$table_info->{tags}}, $tag;
    }
    
    return $table_info;
}