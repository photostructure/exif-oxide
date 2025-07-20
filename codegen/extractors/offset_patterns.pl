#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         offset_patterns.pl
#
# Description:  Extract offset calculation patterns from ExifTool modules
#
# Usage:        perl offset_patterns.pl <module_path> [function_pattern]
#
# Example:      perl offset_patterns.pl ../third-party/exiftool/lib/Image/ExifTool/Sony.pm
#
# Notes:        This script extracts model-specific offset calculation patterns
#               including:
#               - Model condition patterns ($$self{Model} =~ /pattern/)
#               - Offset calculations (Get32u($dataPt, $entry + 4) + $makerNoteBase)
#               - Base types and offset adjustments
#               - IDC corruption patterns and recovery
#               Designed specifically for Sony.pm but works for any manufacturer
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_module_from_file
    format_json_output
);

# Check arguments
if (@ARGV < 1) {
    die "Usage: $0 <module_path> [function_pattern]\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Sony.pm\n";
}

my ($module_path, $function_pattern) = @ARGV;
$function_pattern ||= 'ProcessSony|Process.*Binary|SetARW';  # Default patterns for Sony

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path for display
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{}; # Remove path

# Extract manufacturer name from module
my $manufacturer = $module_display_name;
$manufacturer =~ s/\.pm$//;

print STDERR "Extracting offset patterns from $module_display_name...\n";
print STDERR "Function pattern: $function_pattern\n";

# Read and parse the source file directly for offset patterns
my $offset_patterns = extract_offset_patterns_from_source($module_path, $manufacturer);

print STDERR "  Found " . scalar(@{$offset_patterns->{model_conditions}}) . " model-specific conditions\n";
print STDERR "  Found " . scalar(@{$offset_patterns->{offset_calculations}}) . " offset calculation patterns\n";
print STDERR "  Found " . scalar(@{$offset_patterns->{base_types}}) . " base offset types\n";

# Output JSON
my $output = {
    source => {
        module => $module_display_name,
        function_pattern => $function_pattern,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    manufacturer => $manufacturer,
    offset_patterns => $offset_patterns,
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract offset patterns from source code
#------------------------------------------------------------------------------
sub extract_offset_patterns_from_source {
    my ($module_path, $manufacturer) = @_;
    
    # Read entire source file
    open my $fh, '<', $module_path or die "Cannot open $module_path: $!\n";
    my $source_code = do { local $/; <$fh> };
    close $fh;
    
    my @model_conditions;
    my @offset_calculations;
    my @base_types;
    my @idc_patterns;
    my %seen_conditions;
    my %seen_calculations;
    my %seen_bases;
    
    # Extract model condition patterns
    # Look for $$self{Model} patterns throughout the code
    while ($source_code =~ /\$\$self\{Model\}\s*([!~=]+)\s*([^);\n]+)/g) {
        my $operator = $1;
        my $pattern = $2;
        
        # Clean up the pattern
        $pattern =~ s/^\s+|\s+$//g;  # Trim whitespace
        $pattern =~ s/\s*\)\s*$//;   # Remove trailing )
        
        # Extract regex patterns
        if ($pattern =~ m{^/(.+?)/?(?:[gimsx]*)?$}) {
            my $regex = $1;
            my $condition_key = "$operator:$regex";
            
            unless ($seen_conditions{$condition_key}) {
                push @model_conditions, {
                    type => 'regex',
                    operator => $operator,
                    pattern => $regex,
                    raw_pattern => $pattern,
                };
                $seen_conditions{$condition_key} = 1;
            }
        } elsif ($pattern =~ /^["'](.+?)["']$/) {
            # Quoted string patterns
            my $string = $1;
            my $condition_key = "$operator:$string";
            
            unless ($seen_conditions{$condition_key}) {
                push @model_conditions, {
                    type => 'string', 
                    operator => $operator,
                    pattern => $string,
                    raw_pattern => $pattern,
                };
                $seen_conditions{$condition_key} = 1;
            }
        }
    }
    
    # Extract offset calculation patterns - look for actual Sony offset patterns
    
    # Look for DirStart/DirLen assignments (common in Sony.pm)
    while ($source_code =~ /DirStart\s*=>\s*([^,}]+)/g) {
        my $calculation = $1;
        $calculation =~ s/^\s+|\s+$//g;
        
        unless ($seen_calculations{$calculation}) {
            my $calc_data = parse_offset_calculation($calculation);
            if ($calc_data) {
                $calc_data->{context} = 'dir_start';
                push @offset_calculations, $calc_data;
                $seen_calculations{$calculation} = 1;
            }
        }
    }
    
    # Look for offset variable assignments
    while ($source_code =~ /\$(?:offset|addr|ptr|pos|start)\s*=\s*([^;]+);/g) {
        my $calculation = $1;
        $calculation =~ s/^\s+|\s+$//g;
        
        # Skip simple variable assignments
        next if $calculation =~ /^\$\w+$/;
        
        unless ($seen_calculations{$calculation}) {
            my $calc_data = parse_offset_calculation($calculation);
            if ($calc_data) {
                push @offset_calculations, $calc_data;
                $seen_calculations{$calculation} = 1;
            }
        }
    }
    
    # Look for Get32u/Get16u/Get8u patterns anywhere in expressions
    while ($source_code =~ /(Get(?:32u|16u|8u)\s*\([^)]+\)(?:\s*[\+\-\*]\s*[^;,\n)]+)?)/g) {
        my $calculation = $1;
        $calculation =~ s/^\s+|\s+$//g;
        
        unless ($seen_calculations{$calculation}) {
            my $calc_data = parse_offset_calculation($calculation);
            if ($calc_data) {
                push @offset_calculations, $calc_data;
                $seen_calculations{$calculation} = 1;
            }
        }
    }
    
    # Look for Set32u/Set16u patterns (offset writing)
    while ($source_code =~ /(Set(?:32u|16u)\s*\([^)]+,\s*[^)]+,\s*([^)]+)\))/g) {
        my $full_expr = $1;
        my $offset_expr = $2;
        $offset_expr =~ s/^\s+|\s+$//g;
        
        unless ($seen_calculations{$offset_expr}) {
            my $calc_data = parse_offset_calculation($offset_expr);
            if ($calc_data) {
                $calc_data->{context} = 'set_offset';
                push @offset_calculations, $calc_data;
                $seen_calculations{$offset_expr} = 1;
            }
        }
    }
    
    # Look for array offset calculations like $start + 4 + $i * 4
    while ($source_code =~ /(\$\w+\s*\+\s*\d+\s*\+\s*\$\w+\s*\*\s*\d+)/g) {
        my $calculation = $1;
        
        unless ($seen_calculations{$calculation}) {
            my $calc_data = parse_offset_calculation($calculation);
            if ($calc_data) {
                $calc_data->{context} = 'array_offset';
                push @offset_calculations, $calc_data;
                $seen_calculations{$calculation} = 1;
            }
        }
    }
    
    # Look for entry-based offsets like $entry{0xc634} + 8
    while ($source_code =~ /(\$entry\{0x[0-9a-fA-F]+\}\s*\+\s*\d+)/g) {
        my $calculation = $1;
        
        unless ($seen_calculations{$calculation}) {
            my $calc_data = parse_offset_calculation($calculation);
            if ($calc_data) {
                $calc_data->{context} = 'entry_offset';
                push @offset_calculations, $calc_data;
                $seen_calculations{$calculation} = 1;
            }
        }
    }
    
    # Extract base offset types
    # Look for common base patterns in offset calculations
    my @common_bases = (
        '$makerNoteBase', '$base', '$valuePtr', '$tiffBase', 
        '$ifdStart', '$fileStart', '\$entry', '\$dataPt'
    );
    
    foreach my $base_pattern (@common_bases) {
        if ($source_code =~ /\Q$base_pattern\E/) {
            my $base_name = $base_pattern;
            $base_name =~ s/^\$|\\\$//g;  # Remove $ prefix
            
            unless ($seen_bases{$base_name}) {
                push @base_types, {
                    name => $base_name,
                    pattern => $base_pattern,
                    usage_count => () = $source_code =~ /\Q$base_pattern\E/g,
                };
                $seen_bases{$base_name} = 1;
            }
        }
    }
    
    # Extract IDC corruption patterns (Sony-specific)
    if ($manufacturer eq 'Sony') {
        @idc_patterns = extract_idc_patterns($source_code);
    }
    
    return {
        model_conditions => \@model_conditions,
        offset_calculations => \@offset_calculations,
        base_types => \@base_types,
        idc_patterns => \@idc_patterns,
    };
}

#------------------------------------------------------------------------------
# Parse individual offset calculation
#------------------------------------------------------------------------------
sub parse_offset_calculation {
    my $calculation = shift;
    
    # Common patterns in Sony.pm offset calculations
    my $calc_data = {
        raw_expression => $calculation,
        components => [],
        base_type => 'unknown',
        operation => 'unknown',
    };
    
    # Pattern: Get32u/Get16u/Get8u($dataPt, $entry + N) + $base
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$dataPt\s*,\s*\$entry\s*\+\s*(\d+)\s*\)\s*\+\s*(\$\w+)/) {
        $calc_data->{operation} = lc("get$1_entry_offset");
        $calc_data->{entry_offset} = $2;
        $calc_data->{base_variable} = $3;
        $calc_data->{base_type} = 'entry_plus_base';
        return $calc_data;
    }
    
    # Pattern: Get32u/Get16u/Get8u($dataPt, $valuePtr) + $base
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$dataPt\s*,\s*\$valuePtr\s*\)\s*\+\s*(\$\w+)/) {
        $calc_data->{operation} = lc("get$1_valueptr");
        $calc_data->{base_variable} = $2;
        $calc_data->{base_type} = 'valueptr_plus_base';
        return $calc_data;
    }
    
    # Pattern: Get32u/Get16u/Get8u($valPt, N) - direct offset read
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$valPt\s*,\s*(\d+)\s*\)/) {
        $calc_data->{operation} = lc("get$1_valpt");
        $calc_data->{offset} = $2;
        $calc_data->{base_type} = 'direct_read';
        return $calc_data;
    }
    
    # Pattern: Get32u/Get16u/Get8u($dataPt, N) + constant
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$dataPt\s*,\s*(\d+)\s*\)\s*\+\s*(0x[0-9a-fA-F]+|\d+)/) {
        $calc_data->{operation} = lc("get$1_fixed_offset");
        $calc_data->{data_offset} = $2;
        $calc_data->{constant} = $3;
        $calc_data->{base_type} = 'fixed_offset';
        return $calc_data;
    }
    
    # Pattern: Get16u($dataPt, $entry + N) - without additional base
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$dataPt\s*,\s*\$entry\s*\+\s*(\d+)\s*\)/) {
        $calc_data->{operation} = lc("get$1_entry");
        $calc_data->{entry_offset} = $2;
        $calc_data->{base_type} = 'entry_direct';
        return $calc_data;
    }
    
    # Pattern: $start + 4 + $i * 4 (array offset)
    if ($calculation =~ /(\$\w+)\s*\+\s*(\d+)\s*\+\s*(\$\w+)\s*\*\s*(\d+)/) {
        $calc_data->{operation} = 'array_offset';
        $calc_data->{base_variable} = $1;
        $calc_data->{base_offset} = $2;
        $calc_data->{index_variable} = $3;
        $calc_data->{element_size} = $4;
        $calc_data->{base_type} = 'array_calculation';
        return $calc_data;
    }
    
    # Pattern: $entry{0xc634} + 8 (entry hash offset)
    if ($calculation =~ /\$entry\{(0x[0-9a-fA-F]+)\}\s*\+\s*(\d+)/) {
        $calc_data->{operation} = 'entry_hash_offset';
        $calc_data->{tag_id} = $1;
        $calc_data->{offset} = $2;
        $calc_data->{base_type} = 'entry_hash';
        return $calc_data;
    }
    
    # Pattern: $value + constant
    if ($calculation =~ /^(\$\w+)\s*\+\s*(0x[0-9a-fA-F]+|\d+)$/) {
        $calc_data->{operation} = 'variable_plus_constant';
        $calc_data->{base_variable} = $1;
        $calc_data->{constant} = $2;
        $calc_data->{base_type} = 'variable_offset';
        return $calc_data;
    }
    
    # Pattern: simple Get32u/Get16u/Get8u($dataPt, $offset)
    if ($calculation =~ /Get(32u|16u|8u)\s*\(\s*\$dataPt\s*,\s*(\$\w+)\s*\)/) {
        $calc_data->{operation} = lc("get$1_variable");
        $calc_data->{offset_variable} = $2;
        $calc_data->{base_type} = 'variable_read';
        return $calc_data;
    }
    
    # Simple variable assignment
    if ($calculation =~ /^(\$\w+)$/) {
        $calc_data->{operation} = 'variable_assignment';
        $calc_data->{base_variable} = $1;
        $calc_data->{base_type} = 'simple_variable';
        return $calc_data;
    }
    
    # Complex expression - store as-is for manual analysis
    $calc_data->{operation} = 'complex_expression';
    $calc_data->{base_type} = 'complex';
    
    return $calc_data;
}

#------------------------------------------------------------------------------
# Extract IDC corruption patterns (Sony-specific)
#------------------------------------------------------------------------------
sub extract_idc_patterns {
    my $source_code = shift;
    my @patterns;
    
    # Look for SetARW function which handles IDC corruption
    if ($source_code =~ /sub\s+SetARW\s*\{(.*?)\}/s) {
        my $setarw_code = $1;
        
        # Look for A100 specific handling
        if ($setarw_code =~ /A100.*?\{(.*?)\}/s) {
            push @patterns, {
                type => 'A100_IDC',
                description => 'A100 IDC corruption handling',
                pattern => 'Model contains A100',
                recovery => 'Tag 0x14a corruption fix',
            };
        }
        
        # Look for general IDC detection
        if ($setarw_code =~ /Sony\s+IDC/) {
            push @patterns, {
                type => 'General_IDC',
                description => 'General Sony IDC corruption detection',
                pattern => 'Software contains Sony IDC',
                recovery => 'Offset adjustment',
            };
        }
    }
    
    return @patterns;
}