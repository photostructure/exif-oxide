#!/usr/bin/env perl

=head1 NAME

boolean_set.pl - Extract boolean sets from ExifTool modules

=head1 SYNOPSIS

    perl boolean_set.pl SOURCE_FILE HASH_NAME

=head1 DESCRIPTION

Extracts boolean sets (hashes with keys mapping to 1) from ExifTool modules.
These are used for fast membership testing like "if ($isDatChunk{$chunk})".

The script outputs JSON with entries for each key that maps to a truthy value.

=cut

use strict;
use warnings;
use JSON::PP;
use Cwd qw(abs_path);
use File::Basename qw(dirname);

# Add ExifTool lib to path
BEGIN {
    my $script_dir = dirname(abs_path($0));
    # When called from generated/extract directory: go up to codegen, then to repo root
    unshift @INC, "$script_dir/../../third-party/exiftool/lib";
    unshift @INC, "$script_dir/../lib";  # For our extraction utilities
}

if (@ARGV != 2) {
    die "Usage: $0 SOURCE_FILE HASH_NAME\n";
}

my ($source_file, $hash_name) = @ARGV;

# Load the module to access the hash
my $full_path = abs_path($source_file);
unless (-f $full_path) {
    die "Source file not found: $source_file\n";
}

# Extract module name from path for require
my $module_name;
if ($source_file =~ m{/Image/ExifTool/([^/]+)\.pm$}) {
    $module_name = "Image::ExifTool::$1";
} elsif ($source_file =~ m{/Image/ExifTool\.pm$}) {
    $module_name = "Image::ExifTool";
} else {
    die "Cannot determine module name from path: $source_file\n";
}


# Load the module
eval "require $module_name" or die "Cannot load module $module_name: $@\n";

# Get the hash reference
my $hash_ref;
{
    no strict 'refs';
    my $full_hash_name = $hash_name;
    $full_hash_name =~ s/^%//;  # Remove % prefix if present
    $full_hash_name = "${module_name}::${full_hash_name}";
    $hash_ref = \%{$full_hash_name};
}

unless (defined $hash_ref && %$hash_ref) {
    die "Hash $hash_name not found or empty in module $module_name\n";
}

# Build the output structure
my @entries;
for my $key (sort keys %$hash_ref) {
    my $value = $hash_ref->{$key};
    # Only include keys with truthy values (typically 1)
    if ($value) {
        push @entries, {
            key => $key,
            value => $value
        };
    }
}

my $output = {
    source => {
        module => $module_name,
        file => $source_file,
        hash_name => $hash_name,
        extracted_at => scalar(localtime()),
    },
    metadata => {
        description => "Boolean set from $hash_name",
        key_type => "String",
        constant_name => "BOOLEAN_SET",  # Will be overridden by config
        # DO NOT ADD entry_count HERE!
        # Rust calculates this from entries.len() - adding it creates inconsistencies
    },
    entries => \@entries
};

# Output JSON
my $json = JSON::PP->new->pretty->canonical;
print $json->encode($output);