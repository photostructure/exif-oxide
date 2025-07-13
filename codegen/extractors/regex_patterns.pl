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
use MIME::Base64;

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
    # Encode the pattern as base64 to avoid character escaping issues
    # ExifTool patterns contain raw bytes (0x00-0xFF) that don't translate
    # well through JSON -> Rust string literals -> regex compilation.
    # Base64 encoding preserves the exact byte sequence without any
    # interpretation or escaping complications.
    
    # We need to convert Perl string escape sequences to actual bytes
    # before base64 encoding. This handles \xNN, \0, \r, \n etc.
    my $pattern_bytes = eval qq{"$pattern"};
    if ($@) {
        warn "Failed to eval pattern for $file_type: $@\n";
        $pattern_bytes = $pattern;  # Fall back to original
    }
    
    push @patterns, {
        file_type => $file_type,
        pattern => $pattern,
        pattern_base64 => encode_base64($pattern_bytes, ''),  # No newlines
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