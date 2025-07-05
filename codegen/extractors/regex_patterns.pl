#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         regex_patterns.pl
#
# Description:  Extract regex patterns (magic numbers) from ExifTool
#
# Usage:        perl regex_patterns.pl > ../generated/regex_patterns.json
#
# Notes:        This script extracts regex patterns used for file type
#               detection and validates them for Rust compatibility.
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
    validate_regex_for_rust
    format_json_output
);

# Read configuration for regex pattern tables
my $config = load_json_config("$Bin/../simple_tables.json");
my @regex_tables = grep {
    $_->{extraction_type} && $_->{extraction_type} eq 'regex_strings'
} @{$config->{tables}};

print STDERR "Processing " . scalar(@regex_tables) . " regex pattern tables...\n";

my @all_patterns;
my $rust_compatible_count = 0;
my $incompatible_count = 0;

# Process each regex table
for my $table_config (@regex_tables) {
    my $module_file = "$Bin/../../third-party/exiftool/lib/Image/ExifTool/$table_config->{module}";
    my $hash_name = $table_config->{hash_name};
    
    print STDERR "\nExtracting $hash_name from $table_config->{module}...\n";
    
    # Load module
    my $module_name = load_module_from_file($module_file);
    unless ($module_name) {
        warn "  SKIPPED: Failed to load module\n";
        next;
    }
    
    # Get package hash
    my $hash_ref = get_package_hash($module_name, $hash_name);
    unless ($hash_ref) {
        warn "  SKIPPED: Hash not found\n";
        next;
    }
    
    # Extract patterns
    my @patterns = extract_regex_patterns($hash_ref, $table_config);
    
    # Add table metadata to each pattern
    for my $pattern (@patterns) {
        $pattern->{source_table} = {
            module => $table_config->{module},
            hash_name => $hash_name,
            description => $table_config->{description},
        };
        
        # Count compatibility
        if ($pattern->{rust_compatible}) {
            $rust_compatible_count++;
        } else {
            $incompatible_count++;
        }
    }
    
    push @all_patterns, @patterns;
    print STDERR "  Extracted " . scalar(@patterns) . " patterns\n";
}

# Group patterns by purpose
my %patterns_by_purpose = (
    magic_numbers => [],
    file_extensions => [],
    mime_patterns => [],
    other => [],
);

# Categorize patterns
for my $pattern (@all_patterns) {
    if ($pattern->{source_table}->{hash_name} =~ /magic/i) {
        push @{$patterns_by_purpose{magic_numbers}}, $pattern;
    } elsif ($pattern->{source_table}->{hash_name} =~ /ext/i) {
        push @{$patterns_by_purpose{file_extensions}}, $pattern;
    } elsif ($pattern->{source_table}->{hash_name} =~ /mime/i) {
        push @{$patterns_by_purpose{mime_patterns}}, $pattern;
    } else {
        push @{$patterns_by_purpose{other}}, $pattern;
    }
}

# Output JSON
my $output = {
    extracted_at => scalar(gmtime()) . " GMT",
    patterns => \%patterns_by_purpose,
    stats => {
        total_patterns => scalar(@all_patterns),
        rust_compatible => $rust_compatible_count,
        incompatible => $incompatible_count,
        by_category => {
            magic_numbers => scalar(@{$patterns_by_purpose{magic_numbers}}),
            file_extensions => scalar(@{$patterns_by_purpose{file_extensions}}),
            mime_patterns => scalar(@{$patterns_by_purpose{mime_patterns}}),
            other => scalar(@{$patterns_by_purpose{other}}),
        },
    },
    compatibility_notes => "Patterns validated for Rust regex crate compatibility",
};

print format_json_output($output);

print STDERR "\nSummary:\n";
print STDERR "  Total patterns: " . scalar(@all_patterns) . "\n";
print STDERR "  Rust compatible: $rust_compatible_count\n";
print STDERR "  Incompatible: $incompatible_count\n";

#------------------------------------------------------------------------------
# Extract regex patterns from a hash
#------------------------------------------------------------------------------
sub extract_regex_patterns {
    my ($hash_ref, $config) = @_;
    my @patterns;
    
    for my $key (sort keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        next unless defined $value && !ref $value;
        
        # Validate Rust compatibility
        my $compat = validate_regex_for_rust($value);
        
        push @patterns, {
            key => $key,
            pattern => $value,
            rust_compatible => $compat->{compatible} ? 1 : 0,
            compatibility_notes => $compat->{reason},
        };
    }
    
    return @patterns;
}