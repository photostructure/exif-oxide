#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         regex_patterns.pl
#
# Description:  Extract regex patterns (magic numbers) from ExifTool
#
# Usage:        perl regex_patterns.pl <module_path> <hash_name>
#
# Notes:        This script extracts regex patterns used for file type
#               detection from the %magicNumber hash.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_module_from_file
    get_package_hash
    format_json_output
);

# Check arguments - take explicit module path and hash name
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <hash_name>\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool.pm %magicNumber\n";
}

my $module_path = shift @ARGV;
my $hash_name = shift @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

print STDERR "Extracting $hash_name from $module_path...\n";

# Load module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Get package hash
my $hash_ref = get_package_hash($module_name, $hash_name);
unless ($hash_ref) {
    die "Error: Hash $hash_name not found in module\n";
}

# Extract magic number patterns
my @patterns;
my $total_count = 0;

for my $file_type (sort keys %$hash_ref) {
    my $pattern = $hash_ref->{$file_type};
    $total_count++;
    
    # Create pattern entry
    push @patterns, {
        file_type => $file_type,
        pattern => $pattern,
        source => {
            module => $module_path,
            hash => $hash_name,
        },
    };
}

print STDERR "  Extracted $total_count magic number patterns\n";

# Output JSON
my $output = {
    extracted_at => scalar(gmtime()) . " GMT",
    magic_patterns => \@patterns,
    stats => {
        total_patterns => $total_count,
    },
};

print format_json_output($output);

print STDERR "\nExtraction complete: $total_count patterns\n";