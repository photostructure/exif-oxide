#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         simple_table.pl
#
# Description:  Extract simple lookup tables from ExifTool modules
#
# Usage:        perl simple_table.pl <module_path> <hash_name> [<hash_name2> ...]
#
# Example:      perl simple_table.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm %canonWhiteBalance %pictureStyles
#
# Notes:        This script extracts only simple primitive lookup tables.
#               Output is written to individual JSON files in current directory.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";
use Carp;

# Check required environment variables
unless ($ENV{CODEGEN_DIR} && $ENV{REPO_ROOT}) {
    croak "Error: Required environment variables CODEGEN_DIR and REPO_ROOT must be set by the calling Rust code";
}

use ExifToolExtract qw(
    load_module_from_file
    get_package_hash
    validate_primitive_value
    format_json_output
);

# Check arguments
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <hash_name> [<hash_name2> ...]\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm %canonWhiteBalance\n";
}

my $module_path = shift @ARGV;
my @hash_names = @ARGV;

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

# Extract each requested hash
my @all_extractions;

for my $hash_name (@hash_names) {
    # Ensure hash name starts with %
    $hash_name = "%$hash_name" unless $hash_name =~ /^%/;
    
    # Get the hash
    my $hash_ref = get_package_hash($module_name, $hash_name);
    unless ($hash_ref) {
        warn "Warning: Hash $hash_name not found in $module_display_name\n";
        warn "Note: Module should be patched by Rust orchestration before calling this script\n";
        next;
    }
    
    # Extract primitive entries
    my @entries;
    for my $key (sort keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        
        # Only process primitive values
        next unless validate_primitive_value($value);
        
        push @entries, {
            key => $key,
            value => $value,
        };
    }
    
    if (@entries) {
        push @all_extractions, {
            source => {
                module => $module_display_name,
                hash_name => $hash_name,
                extracted_at => scalar(gmtime()) . " GMT",
            },
            metadata => {
                # DO NOT ADD entry_count HERE!
                # Rust calculates this from entries.len() - adding it creates inconsistencies
            },
            entries => \@entries,
        };
        
        print STDERR "Extracted " . scalar(@entries) . " entries from $hash_name\n";
    } else {
        warn "Warning: No primitive entries found in $hash_name\n";
    }
}

# Output individual JSON files
if (@all_extractions) {
    for my $extraction (@all_extractions) {
        # Generate filename from hash name
        my $hash_name = $extraction->{source}{hash_name};
        $hash_name =~ s/^%//;  # Remove % prefix
        $hash_name =~ s/([A-Z])/_\L$1/g;  # Convert camelCase to snake_case
        $hash_name =~ s/^_//;  # Remove leading underscore
        $hash_name = lc($hash_name);  # Ensure lowercase
        my $filename = "${hash_name}.json";
        
        # Write to file
        open(my $fh, '>', $filename) or die "Cannot write to $filename: $!";
        print $fh format_json_output($extraction);
        close($fh);
        
        print STDERR "Created $filename\n";
    }
} else {
    die "Error: No tables successfully extracted\n";
}