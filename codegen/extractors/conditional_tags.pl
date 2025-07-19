#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         conditional_tags.pl
#
# Description:  Extract complex conditional tag definitions from ExifTool modules
#
# Usage:        perl conditional_tags.pl <module_path> <table_name>
#
# Example:      perl conditional_tags.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main
#
# Notes:        This script extracts complex conditional tag arrays including:
#               - Count-based conditions ($count == 582)
#               - Binary pattern matching ($$valPt =~ /regex/)
#               - Format-dependent conditions ($format eq "undef")
#               - Cross-tag dependencies and state management
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";
use JSON;

use ExifToolExtract qw(
    load_module_from_file
    format_json_output
);

# Check arguments
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <table_name>\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main\n";
}

my ($module_path, $table_name) = @ARGV;

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

print STDERR "Extracting conditional tag definitions from $table_name table in $module_display_name...\n";

# Load the module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Get the tag table
my $table_symbol = "${module_name}::${table_name}";
my $table_ref = eval "\\%${table_symbol}";
if (!$table_ref || !%$table_ref) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract conditional tag definitions
my $conditional_data = extract_conditional_definitions($table_ref, $manufacturer, $table_name);

print STDERR "  Found " . scalar(@{$conditional_data->{conditional_arrays}}) . " conditional tag arrays\n";
print STDERR "  Found " . scalar(@{$conditional_data->{count_conditions}}) . " count-based conditions\n";
print STDERR "  Found " . scalar(@{$conditional_data->{binary_patterns}}) . " binary pattern conditions\n";

# Output JSON
my $output = {
    source => {
        module => $module_display_name,
        table => $table_name,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    manufacturer => $manufacturer,
    conditional_data => $conditional_data,
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract conditional tag definitions from table
#------------------------------------------------------------------------------
sub extract_conditional_definitions {
    my ($table_ref, $manufacturer, $table_name) = @_;
    
    my @conditional_arrays;
    my @count_conditions;
    my @binary_patterns;
    my @format_conditions;
    my @cross_tag_dependencies;
    
    # Process all table entries
    foreach my $key (sort keys %$table_ref) {
        # Skip non-tag entries (metadata)
        next if $key !~ /^0x[0-9a-fA-F]+$/ && $key !~ /^\d+$/;
        
        my $tag_info = $table_ref->{$key};
        next unless defined $tag_info;
        
        # Handle array of conditional tag definitions
        if (ref $tag_info eq 'ARRAY') {
            my $conditional_array = extract_conditional_array($key, $tag_info);
            if ($conditional_array && @{$conditional_array->{conditions}}) {
                push @conditional_arrays, $conditional_array;
                
                # Categorize the conditions within this array
                foreach my $condition (@{$conditional_array->{conditions}}) {
                    categorize_condition($condition, \@count_conditions, \@binary_patterns, 
                                       \@format_conditions, \@cross_tag_dependencies);
                }
            }
        }
        # Handle single conditional tag
        elsif (ref $tag_info eq 'HASH' && has_complex_condition($tag_info)) {
            my $condition_entry = extract_complex_condition($key, $tag_info);
            if ($condition_entry) {
                categorize_condition($condition_entry, \@count_conditions, \@binary_patterns,
                                   \@format_conditions, \@cross_tag_dependencies);
            }
        }
    }
    
    return {
        table_name => $table_name,
        conditional_arrays => \@conditional_arrays,
        count_conditions => \@count_conditions,
        binary_patterns => \@binary_patterns,
        format_conditions => \@format_conditions,
        cross_tag_dependencies => \@cross_tag_dependencies,
    };
}

#------------------------------------------------------------------------------
# Extract conditional array structure
#------------------------------------------------------------------------------
sub extract_conditional_array {
    my ($tag_id, $array_ref) = @_;
    my @conditions;
    
    foreach my $tag_def (@$array_ref) {
        next unless ref $tag_def eq 'HASH';
        next unless $tag_def->{Condition};
        
        my $condition_entry = {
            tag_id => $tag_id,
            condition => $tag_def->{Condition},
            name => $tag_def->{Name} || 'Unknown',
        };
        
        # Add additional fields
        $condition_entry->{description} = $tag_def->{Description} if $tag_def->{Description};
        $condition_entry->{format} = $tag_def->{Format} if $tag_def->{Format};
        $condition_entry->{writable} = JSON::true if $tag_def->{Writable};
        $condition_entry->{subdirectory} = JSON::true if $tag_def->{SubDirectory};
        $condition_entry->{data_member} = $tag_def->{DataMember} if $tag_def->{DataMember};
        $condition_entry->{raw_conv} = $tag_def->{RawConv} if $tag_def->{RawConv};
        $condition_entry->{value_conv} = $tag_def->{ValueConv} if $tag_def->{ValueConv};
        $condition_entry->{print_conv} = $tag_def->{PrintConv} if $tag_def->{PrintConv};
        
        push @conditions, $condition_entry;
    }
    
    return @conditions ? {
        tag_id => $tag_id,
        conditions => \@conditions,
    } : undef;
}

#------------------------------------------------------------------------------
# Check if tag has complex conditions worth extracting
#------------------------------------------------------------------------------
sub has_complex_condition {
    my $tag_info = shift;
    
    return 0 unless $tag_info->{Condition};
    
    my $condition = $tag_info->{Condition};
    
    # Look for complex conditions
    return 1 if $condition =~ /\$count\s*[=!]=\s*\d+/;  # Count conditions
    return 1 if $condition =~ /\$\$valPt\s*=~/;         # Binary pattern matching
    return 1 if $condition =~ /\$format\s*eq/;          # Format conditions
    return 1 if $condition =~ /\$\$self\{[^}]+\}/;      # Self reference conditions
    return 1 if $condition =~ /DataMember|RawConv|ValueConv/; # Cross-tag dependencies
    
    return 0;
}

#------------------------------------------------------------------------------
# Extract complex condition from single tag
#------------------------------------------------------------------------------
sub extract_complex_condition {
    my ($tag_id, $tag_info) = @_;
    
    return {
        tag_id => $tag_id,
        condition => $tag_info->{Condition},
        name => $tag_info->{Name} || 'Unknown',
        description => $tag_info->{Description},
        format => $tag_info->{Format},
        writable => $tag_info->{Writable} ? JSON::true : JSON::false,
        data_member => $tag_info->{DataMember},
        raw_conv => $tag_info->{RawConv},
        value_conv => $tag_info->{ValueConv},
        print_conv => $tag_info->{PrintConv},
    };
}

#------------------------------------------------------------------------------
# Categorize condition type for specialized handling
#------------------------------------------------------------------------------
sub categorize_condition {
    my ($condition_entry, $count_conditions, $binary_patterns, 
        $format_conditions, $cross_tag_dependencies) = @_;
    
    my $condition = $condition_entry->{condition};
    
    # Count-based conditions
    if ($condition =~ /\$count\s*[=!]=\s*\d+/) {
        push @$count_conditions, $condition_entry;
    }
    
    # Binary pattern matching
    if ($condition =~ /\$\$valPt\s*=~/) {
        push @$binary_patterns, $condition_entry;
    }
    
    # Format conditions
    if ($condition =~ /\$format\s*eq/) {
        push @$format_conditions, $condition_entry;
    }
    
    # Cross-tag dependencies
    if ($condition_entry->{data_member} || $condition_entry->{raw_conv} || 
        $condition =~ /\$\$self\{[^}]+\}/) {
        push @$cross_tag_dependencies, $condition_entry;
    }
}