#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         simple_tables.pl
#
# Description:  Extract simple lookup tables from ExifTool modules
#
# Usage:        perl simple_tables.pl
#
# Notes:        This script extracts only simple primitive lookup tables.
#               Each table is saved as an individual JSON file.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_module_from_file
    get_package_hash
    load_json_config
    validate_primitive_value
    format_json_output
    extract_source_line_info
);

# Read configuration file
my $config = load_json_config("$Bin/../simple_tables.json");
my @tables = grep { 
    !$_->{extraction_type} || $_->{extraction_type} eq 'simple' || $_->{extraction_type} eq 'boolean_set'
} @{$config->{tables}};

print STDERR "Processing " . scalar(@tables) . " simple lookup tables...\n";

# Create output directory
my $output_dir = "$Bin/../generated/simple_tables";
mkdir $output_dir unless -d $output_dir;

# Process each table individually
my $success_count = 0;
my $skip_count = 0;

for my $table_config (@tables) {
    my $module_file;
    if ($table_config->{module} eq 'ExifTool.pm') {
        # Special case: ExifTool.pm is in parent directory
        $module_file = "$Bin/../../third-party/exiftool/lib/Image/ExifTool.pm";
    } else {
        $module_file = "$Bin/../../third-party/exiftool/lib/Image/ExifTool/$table_config->{module}";
    }
    my $hash_name = $table_config->{hash_name};
    
    print STDERR "Extracting $hash_name from $table_config->{module}...\n";
    
    # Load module and extract table
    my $module_name = load_module_from_file($module_file);
    unless ($module_name) {
        warn "  SKIPPED: Failed to load module\n";
        $skip_count++;
        next;
    }
    
    # Get package hash
    my $hash_ref = get_package_hash($module_name, $hash_name);
    unless ($hash_ref) {
        warn "  SKIPPED: Hash not found (may need patching)\n";
        $skip_count++;
        next;
    }
    
    # Extract primitive entries
    my @entries = extract_primitive_entries($hash_ref, $table_config);
    
    if (@entries) {
        # Create output structure
        my $output = {
            source => {
                module => $table_config->{module},
                hash_name => $hash_name,
                extracted_at => scalar(gmtime()) . " GMT",
            },
            metadata => {
                description => $table_config->{description} || "Lookup table from $table_config->{module}",
                constant_name => $table_config->{constant_name},
                key_type => $table_config->{key_type} || 'String',
                entry_count => scalar(@entries),
            },
            entries => \@entries,
        };
        
        # Generate output filename from constant name
        my $filename = lc($table_config->{constant_name});
        $filename =~ s/_table$//;  # Remove _table suffix if present
        $filename .= ".json";
        
        # Write individual JSON file
        my $output_path = "$output_dir/$filename";
        open(my $fh, '>', $output_path) or die "Cannot write to $output_path: $!";
        print $fh format_json_output($output);
        close($fh);
        
        print STDERR "  ✓ Extracted " . scalar(@entries) . " entries to $filename\n";
        $success_count++;
    } else {
        warn "  SKIPPED: No primitive entries found\n";
        $skip_count++;
    }
}

print STDERR "\nSummary:\n";
print STDERR "  Successfully extracted: $success_count tables\n";
print STDERR "  Skipped: $skip_count tables\n";

# Extract primitive entries from a hash
sub extract_primitive_entries {
    my ($hash_ref, $config) = @_;
    my @entries;
    
    for my $key (sort keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        next if $key =~ /^[A-Z_]+$/;
        
        # Only process primitive values
        next unless validate_primitive_value($value);
        
        push @entries, {
            key => $key,
            value => $value,
        };
    }
    
    return @entries;
}