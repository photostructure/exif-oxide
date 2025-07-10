#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         file_type_lookup.pl
#
# Description:  Extract file type lookup structures from ExifTool
#
# Usage:        perl file_type_lookup.pl <module_path> <hash_name>
#
# Notes:        This script extracts the complex file type lookup
#               discriminated unions used for file type detection.
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
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool.pm %fileTypeLookup\n";
}

my $module_path = shift @ARGV;
my $hash_name = shift @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

print STDERR "Extracting $hash_name from $module_path...\n";

my @all_lookups;

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

# Create a minimal config object for extract_file_type_lookups
my $table_config = {
    module => $module_path,
    hash_name => $hash_name,
};

# Extract file type lookups
my @lookups = extract_file_type_lookups($hash_ref, $table_config);

push @all_lookups, @lookups;
print STDERR "  Extracted " . scalar(@lookups) . " file type lookups\n";

# Categorize lookups
my %lookups_by_type = (
    extensions => [],
    mime_types => [],
    descriptions => [],
    magic_lookups => [],
);

for my $lookup (@all_lookups) {
    if ($lookup->{lookup_type} eq 'extension') {
        push @{$lookups_by_type{extensions}}, $lookup;
    } elsif ($lookup->{lookup_type} eq 'mime') {
        push @{$lookups_by_type{mime_types}}, $lookup;
    } elsif ($lookup->{lookup_type} eq 'description') {
        push @{$lookups_by_type{descriptions}}, $lookup;
    } elsif ($lookup->{lookup_type} eq 'magic') {
        push @{$lookups_by_type{magic_lookups}}, $lookup;
    }
}

# Output JSON
my $output = {
    extracted_at => scalar(gmtime()) . " GMT",
    file_type_lookups => \%lookups_by_type,
    stats => {
        total_lookups => scalar(@all_lookups),
        by_type => {
            extensions => scalar(@{$lookups_by_type{extensions}}),
            mime_types => scalar(@{$lookups_by_type{mime_types}}),
            descriptions => scalar(@{$lookups_by_type{descriptions}}),
            magic_lookups => scalar(@{$lookups_by_type{magic_lookups}}),
        },
    },
};

print format_json_output($output);

print STDERR "\nSummary:\n";
print STDERR "  Total lookups: " . scalar(@all_lookups) . "\n";
print STDERR "  Extensions: " . scalar(@{$lookups_by_type{extensions}}) . "\n";
print STDERR "  MIME types: " . scalar(@{$lookups_by_type{mime_types}}) . "\n";
print STDERR "  Descriptions: " . scalar(@{$lookups_by_type{descriptions}}) . "\n";

#------------------------------------------------------------------------------
# Extract file type lookups from a hash
#------------------------------------------------------------------------------
sub extract_file_type_lookups {
    my ($hash_ref, $config) = @_;
    my @lookups;
    
    for my $key (sort keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        
        # Determine lookup type and structure
        my $lookup_type = determine_lookup_type($key, $value);
        
        if ($lookup_type) {
            push @lookups, {
                key => $key,
                value => extract_lookup_value($value),
                lookup_type => $lookup_type,
                source => {
                    module => $config->{module},
                    hash => $config->{hash_name},
                },
            };
        }
    }
    
    return @lookups;
}

#------------------------------------------------------------------------------
# Determine the type of file lookup entry
#------------------------------------------------------------------------------
sub determine_lookup_type {
    my ($key, $value) = @_;
    
    # Simple heuristics to determine lookup type
    if (!ref $value) {
        # Simple string value
        if ($value =~ m{/}) {
            return 'mime';  # Looks like MIME type
        } elsif (length($value) <= 10 && $value =~ /^[A-Z0-9]+$/i) {
            return 'description';  # Short uppercase string (alias to another type)
        } else {
            return 'description';  # Longer descriptive string
        }
    } elsif (ref $value eq 'ARRAY') {
        # Array values in %fileTypeLookup typically mean [format, description]
        # These are file extensions that map to a format type
        if (ref $value->[0] eq 'ARRAY' || (defined $value->[0] && defined $value->[1])) {
            return 'extension';  # File extension with format mapping
        }
        return 'mime';
    } elsif (ref $value eq 'HASH') {
        # Complex structure (magic number lookup)
        return 'magic';
    }
    
    return undef;  # Unknown type
}

#------------------------------------------------------------------------------
# Extract the value based on its type
#------------------------------------------------------------------------------
sub extract_lookup_value {
    my $value = shift;
    
    if (!ref $value) {
        # Simple scalar
        return $value;
    } elsif (ref $value eq 'ARRAY') {
        # Array of values
        return $value;
    } elsif (ref $value eq 'HASH') {
        # Complex hash - convert to simpler structure
        my %simplified;
        for my $k (keys %$value) {
            if (!ref $value->{$k}) {
                $simplified{$k} = $value->{$k};
            }
        }
        return \%simplified;
    }
    
    return undef;
}