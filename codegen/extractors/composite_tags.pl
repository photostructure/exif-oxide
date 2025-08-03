#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         composite_tags.pl
#
# Description:  Extract composite tag definitions from ExifTool (config-driven)
#
# Usage:        perl composite_tags.pl <module_path> <table_name> [--frequency-threshold <value>] [--include-mainstream]
#
# Example:      perl composite_tags.pl ../third-party/exiftool/lib/Image/ExifTool.pm Composite --frequency-threshold 0.5 --include-mainstream
#
# Notes:        This script extracts composite tag definitions from a specific module and table
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
  is_mainstream_composite_tag
  generate_conv_ref
  format_json_output
  clean_tag_name
);

# Parse command line arguments
my $frequency_threshold = 0;
my $include_mainstream  = 0;

GetOptions(
    "frequency-threshold=f" => \$frequency_threshold,
    "include-mainstream"    => \$include_mainstream,
) or die "Error parsing command line options\n";

# Check required arguments
if ( @ARGV < 2 ) {
    die
"Usage: $0 <module_path> <table_name> [--frequency-threshold <value>] [--include-mainstream]\n"
      . "Example: $0 ../third-party/exiftool/lib/Image/ExifTool.pm Composite --frequency-threshold 0.5 --include-mainstream\n";
}

my ( $module_path, $table_name ) = @ARGV;

# Validate module path
unless ( -f $module_path ) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path for display
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{};    # Remove path

print STDERR
  "Extracting composite tags from $module_display_name table $table_name...\n";
print STDERR "  Frequency threshold: $frequency_threshold\n"
  if $frequency_threshold > 0;
print STDERR "  Include mainstream: "
  . ( $include_mainstream ? "yes" : "no" ) . "\n";

# Load the module dynamically
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Load tag metadata for frequency filtering
my $metadata_file = "$Bin/../../docs/tag-metadata.json";
my $metadata      = load_tag_metadata($metadata_file);

# Track conversion references
my %print_conv_refs;
my %value_conv_refs;

# Get the composite table
my $table_symbol = "${module_name}::${table_name}";
my $table_ref    = eval "\\%${table_symbol}";
if ( !$table_ref || !%$table_ref ) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract composite tags from the table
my @composite_tags =
  extract_composite_from_table( $table_ref, $table_name, $metadata,
    \%print_conv_refs, \%value_conv_refs );

print STDERR "  Found "
  . scalar(@composite_tags)
  . " composite tags matching criteria\n";

# Convert references to sorted arrays
my @print_conv_refs = sort keys %print_conv_refs;
my @value_conv_refs = sort keys %value_conv_refs;

# Output JSON
my $output = {
    source => {
        module       => $module_display_name,
        table        => $table_name,
        extracted_at => scalar( gmtime() ) . " GMT",
    },
    filters => {
        frequency_threshold => $frequency_threshold,
        include_mainstream  => $include_mainstream ? 1 : 0,
    },
    metadata => {
        total_composite_tags => scalar(@composite_tags),
    },
    composite_tags  => \@composite_tags,
    conversion_refs => {
        print_conv => \@print_conv_refs,
        value_conv => \@value_conv_refs,
    },
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract composite tags from a specific table
#------------------------------------------------------------------------------
sub extract_composite_from_table {
    my ( $table_ref, $table_name, $metadata, $print_conv_refs,
        $value_conv_refs )
      = @_;
    my @composite_tags;

    foreach my $tag_name ( sort keys %$table_ref ) {

        # Skip special table keys
        next if $tag_name =~ /^[A-Z_]+$/;
        next if $tag_name eq 'GROUPS';

        my $tag_info = $table_ref->{$tag_name};
        next unless ref $tag_info eq 'HASH';

        # Clean tag name
        my $clean_tag_name = clean_tag_name($tag_name);

        # Apply filtering
        next unless passes_filters( $clean_tag_name, $metadata );

        # Build composite data
        my $composite_data = {
            name      => $clean_tag_name,
            table     => $table_name,
            full_name => $tag_name,
        };

        # Extract dependencies
        if ( $tag_info->{Require} ) {
            $composite_data->{require} =
              extract_dependencies( $tag_info->{Require} );
        }

        if ( $tag_info->{Desire} ) {
            $composite_data->{desire} =
              extract_dependencies( $tag_info->{Desire} );
        }

        # Add conversion references
        if ( $tag_info->{PrintConv} ) {
            my $ref = generate_conv_ref( $clean_tag_name, 'print_conv',
                $tag_info->{PrintConv} );
            $composite_data->{print_conv_ref} = $ref;
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }

        if ( $tag_info->{ValueConv} ) {
            my $ref = generate_conv_ref( $clean_tag_name, 'value_conv',
                $tag_info->{ValueConv} );
            $composite_data->{value_conv_ref} = $ref;
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }

        # Add description if available
        $composite_data->{description} = $tag_info->{Description}
          if $tag_info->{Description};

        # Add writable flag
        $composite_data->{writable} = $tag_info->{Writable} ? 1 : 0;

        # Add metadata if available
        if ( exists $metadata->{$clean_tag_name} ) {
            my $meta = $metadata->{$clean_tag_name};
            $composite_data->{frequency} = $meta->{frequency}
              if $meta->{frequency};
            $composite_data->{mainstream} = $meta->{mainstream} ? 1 : 0
              if $meta->{mainstream};
        }

        push @composite_tags, $composite_data;
    }

    return @composite_tags;
}

#------------------------------------------------------------------------------
# Extract dependency information from Require/Desire fields
#------------------------------------------------------------------------------
sub extract_dependencies {
    my $deps = shift;

    if ( ref $deps eq 'HASH' ) {

        # Hash format with numbered keys
        my @dep_list;
        foreach my $key ( sort { $a <=> $b } keys %$deps ) {
            push @dep_list, $deps->{$key} if $key =~ /^\d+$/;
        }
        return \@dep_list;
    }
    elsif ( ref $deps eq 'ARRAY' ) {

        # Already an array
        return $deps;
    }
    elsif ( !ref $deps ) {

        # Single dependency as string
        return [$deps];
    }

    return [];
}

#------------------------------------------------------------------------------
# Check if a composite tag passes the configured filters
#------------------------------------------------------------------------------
sub passes_filters {
    my ( $tag_name, $metadata ) = @_;

    # Check mainstream filter
    if ( $include_mainstream
        && is_mainstream_composite_tag( $tag_name, $metadata ) )
    {
        return 1;
    }

    # Check frequency threshold
    if ( $frequency_threshold > 0 ) {
        if ( exists $metadata->{$tag_name}
            && $metadata->{$tag_name}{frequency} )
        {
            return 1
              if $metadata->{$tag_name}{frequency} >= $frequency_threshold;
        }
    }

    # If no filters are active or frequency threshold is 0, include all tags
    return 1 if $frequency_threshold == 0 && !$include_mainstream;

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
    $module_name =~ s{.*/lib/}{};    # Remove everything up to /lib/
    $module_name =~ s{\.pm$}{};      # Remove .pm extension
    $module_name =~ s{/}{::}g;       # Convert slashes to ::

    # Load the module
    eval "require $module_name";
    if ($@) {
        warn "Failed to load module $module_name: $@\n";
        return undef;
    }

    return $module_name;
}
