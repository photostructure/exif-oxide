#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         tag_definitions.pl
#
# Description:  Extract tag definitions from ExifTool tag tables (config-driven)
#
# Usage:        perl tag_definitions.pl <module_path> <table_name> [--frequency-threshold <value>] [--include-mainstream] [--groups <group1,group2>]
#
# Example:      perl tag_definitions.pl ../third-party/exiftool/lib/Image/ExifTool/Exif.pm Main --frequency-threshold 0.8 --include-mainstream --groups EXIF,ExifIFD
#
# Notes:        This script extracts tag definitions from a specific module and table
#               based on command-line configuration. Output is written to stdout.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";
use Getopt::Long;

use ExifToolExtract qw(
    load_tag_metadata
    is_mainstream_tag
    generate_conv_ref
    extract_format
    extract_groups
    format_json_output
);

# Parse command line arguments
my $frequency_threshold = 0;
my $include_mainstream = 0;
my $groups_str = "";

GetOptions(
    "frequency-threshold=f" => \$frequency_threshold,
    "include-mainstream" => \$include_mainstream,
    "groups=s" => \$groups_str,
) or die "Error parsing command line options\n";

# Check required arguments
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <table_name> [--frequency-threshold <value>] [--include-mainstream] [--groups <group1,group2>]\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Exif.pm Main --frequency-threshold 0.8 --include-mainstream\n";
}

my ($module_path, $table_name) = @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

# Parse groups filter
my @groups_filter = split(/,/, $groups_str) if $groups_str;

# Extract module name from path for display
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{}; # Remove path

print STDERR "Extracting tag definitions from $module_display_name table $table_name...\n";
print STDERR "  Frequency threshold: $frequency_threshold\n" if $frequency_threshold > 0;
print STDERR "  Include mainstream: " . ($include_mainstream ? "yes" : "no") . "\n";
print STDERR "  Groups filter: " . (@groups_filter ? join(", ", @groups_filter) : "none") . "\n";

# Load the module dynamically
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Load tag metadata for frequency filtering
my $metadata_file = "$Bin/../../third-party/exiftool/doc/TagMetadata.json";
my $metadata = load_tag_metadata($metadata_file);

# Track conversion references
my %print_conv_refs;
my %value_conv_refs;

# Get the tag table
my $table_symbol = "${module_name}::${table_name}";
my $table_ref = eval "\\%${table_symbol}";
if (!$table_ref || !%$table_ref) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract tags from the table
my @tags = extract_tags_from_table($table_ref, $metadata, \%print_conv_refs, \%value_conv_refs);

print STDERR "  Found " . scalar(@tags) . " tags matching criteria\n";

# Convert references to sorted arrays
my @print_conv_refs = sort keys %print_conv_refs;
my @value_conv_refs = sort keys %value_conv_refs;

# Output JSON
my $output = {
    source => {
        module => $module_display_name,
        table => $table_name,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    filters => {
        frequency_threshold => $frequency_threshold,
        include_mainstream => $include_mainstream ? 1 : 0,
        groups => \@groups_filter,
    },
    metadata => {
        total_tags => scalar(@tags),
    },
    tags => \@tags,
    conversion_refs => {
        print_conv => \@print_conv_refs,
        value_conv => \@value_conv_refs,
    },
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract tags from a tag table with filtering
#------------------------------------------------------------------------------
sub extract_tags_from_table {
    my ($table_ref, $metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @tags;
    
    foreach my $tag_id (sort keys %$table_ref) {
        next if $tag_id =~ /^[A-Z]/;  # Skip special keys
        
        my $tag_info = $table_ref->{$tag_id};
        next unless ref $tag_info eq 'HASH';
        next unless exists $tag_info->{Name};
        
        my $tag_name = $tag_info->{Name};
        
        # Apply filtering
        next unless passes_filters($tag_name, $tag_info, $metadata);
        
        # Build tag data
        my $tag_data = {
            id => sprintf("0x%x", $tag_id),
            name => $tag_name,
            format => extract_format($tag_info),
            groups => extract_groups($tag_info),
            writable => $tag_info->{Writable} ? 1 : 0,
        };
        
        # Add optional fields
        $tag_data->{description} = $tag_info->{Description} if $tag_info->{Description};
        $tag_data->{notes} = $tag_info->{Notes} if $tag_info->{Notes};
        
        # Add conversion references
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($tag_name, 'print_conv', $tag_info->{PrintConv});
            $tag_data->{print_conv_ref} = $ref;
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($tag_name, 'value_conv', $tag_info->{ValueConv});
            $tag_data->{value_conv_ref} = $ref;
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add metadata
        if (exists $metadata->{$tag_name}) {
            my $meta = $metadata->{$tag_name};
            $tag_data->{frequency} = $meta->{frequency} if $meta->{frequency};
            $tag_data->{mainstream} = $meta->{mainstream} ? 1 : 0 if $meta->{mainstream};
        }
        
        push @tags, $tag_data;
    }
    
    return @tags;
}

#------------------------------------------------------------------------------
# Check if a tag passes the configured filters
#------------------------------------------------------------------------------
sub passes_filters {
    my ($tag_name, $tag_info, $metadata) = @_;
    
    # Check mainstream filter
    if ($include_mainstream && is_mainstream_tag($tag_name, $metadata)) {
        return 1;
    }
    
    # Check frequency threshold
    if ($frequency_threshold > 0) {
        if (exists $metadata->{$tag_name} && $metadata->{$tag_name}{frequency}) {
            return 1 if $metadata->{$tag_name}{frequency} >= $frequency_threshold;
        }
    }
    
    # Check groups filter
    if (@groups_filter) {
        my @tag_groups = extract_groups($tag_info);
        foreach my $filter_group (@groups_filter) {
            foreach my $tag_group (@tag_groups) {
                return 1 if $tag_group eq $filter_group;
            }
        }
        return 0; # No matching groups found
    }
    
    # If no filters are active or frequency threshold is 0, include all tags
    return 1 if $frequency_threshold == 0 && !$include_mainstream && !@groups_filter;
    
    return 0;
}

#------------------------------------------------------------------------------
# Load module from file path
#------------------------------------------------------------------------------
sub load_module_from_file {
    my $file_path = shift;
    
    # Extract module name from file path
    # e.g., .../Image/ExifTool/Exif.pm -> Image::ExifTool::Exif
    my $module_name = $file_path;
    $module_name =~ s{.*/lib/}{};  # Remove everything up to /lib/
    $module_name =~ s{\.pm$}{};   # Remove .pm extension
    $module_name =~ s{/}{::}g;    # Convert slashes to ::
    
    # Load the module
    eval "require $module_name";
    if ($@) {
        warn "Failed to load module $module_name: $@\n";
        return undef;
    }
    
    return $module_name;
}