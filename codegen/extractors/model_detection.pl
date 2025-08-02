#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         model_detection.pl
#
# Description:  Extract model detection patterns from ExifTool Main tables
#
# Usage:        perl model_detection.pl <module_path> <table_name>
#
# Example:      perl model_detection.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main
#
# Notes:        This script extracts model-specific conditions from tag definitions
#               including $$self{Model} regex patterns and model-dependent logic.
#               Handles patterns like:
#               - { Condition => '$$self{Model} =~ /EOS/', Name => 'TagName' }
#               - Model-specific conditional arrays for same tag IDs
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
if ( @ARGV < 2 ) {
    die "Usage: $0 <module_path> <table_name>\n"
      . "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main\n";
}

my ( $module_path, $table_name ) = @ARGV;

# Validate module path
unless ( -f $module_path ) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path for display
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{};    # Remove path

# Extract manufacturer name from module
my $manufacturer = $module_display_name;
$manufacturer =~ s/\.pm$//;

print STDERR
"Extracting model detection patterns from $table_name table in $module_display_name...\n";

# Load the module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Get the tag table
my $table_symbol = "${module_name}::${table_name}";
my $table_ref    = eval "\\%${table_symbol}";
if ( !$table_ref || !%$table_ref ) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract model detection patterns
my $patterns_data =
  extract_model_detection_patterns( $table_ref, $manufacturer, $table_name );

print STDERR "  Found "
  . scalar( @{ $patterns_data->{patterns} } )
  . " model detection patterns\n";
if ( $patterns_data->{conditional_tags} ) {
    print STDERR "  Found "
      . scalar( @{ $patterns_data->{conditional_tags} } )
      . " conditional tag arrays\n";
}

# Output JSON
my $output = {
    source => {
        module       => $module_display_name,
        table        => $table_name,
        extracted_at => scalar( gmtime() ) . " GMT",
    },
    manufacturer  => $manufacturer,
    patterns_data => $patterns_data,
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract model detection patterns from table
#------------------------------------------------------------------------------
sub extract_model_detection_patterns {
    my ( $table_ref, $manufacturer, $table_name ) = @_;

    my @patterns;
    my @conditional_tags;
    my %model_patterns_seen;
    my %tag_conditions_seen;

    # Process all table entries
    foreach my $key ( sort keys %$table_ref ) {

        # Skip non-tag entries (metadata)
        next if $key !~ /^0x[0-9a-fA-F]+$/ && $key !~ /^\d+$/;

        my $tag_info = $table_ref->{$key};
        next unless defined $tag_info;

        # Handle array of conditional tag definitions
        if ( ref $tag_info eq 'ARRAY' ) {
            my @conditions = extract_conditional_array( $key, $tag_info );
            if (@conditions) {
                push @conditional_tags,
                  {
                    tag_id     => $key,
                    conditions => \@conditions,
                  };

                # Also extract model patterns from conditions
                foreach my $condition (@conditions) {
                    if ( $condition->{condition} ) {
                        my @model_refs = extract_model_patterns_from_condition(
                            $condition->{condition} );
                        foreach my $pattern (@model_refs) {
                            my $pattern_key = $pattern->{pattern};
                            unless ( $model_patterns_seen{$pattern_key} ) {
                                push @patterns, $pattern;
                                $model_patterns_seen{$pattern_key} = 1;
                            }
                        }
                    }
                }
            }
        }

        # Handle single tag with condition
        elsif ( ref $tag_info eq 'HASH' && $tag_info->{Condition} ) {
            my @model_refs =
              extract_model_patterns_from_condition( $tag_info->{Condition} );
            foreach my $pattern (@model_refs) {
                my $pattern_key = $pattern->{pattern};
                unless ( $model_patterns_seen{$pattern_key} ) {
                    push @patterns, $pattern;
                    $model_patterns_seen{$pattern_key} = 1;
                }
            }

            # Also record this as a conditional tag entry
            my $condition_data = extract_single_condition($tag_info);
            if ($condition_data) {
                push @conditional_tags,
                  {
                    tag_id     => $key,
                    conditions => [$condition_data],
                  };
            }
        }
    }

    return {
        table_name       => $table_name,
        patterns         => \@patterns,
        conditional_tags => \@conditional_tags,
    };
}

#------------------------------------------------------------------------------
# Extract conditions from an array of tag definitions
#------------------------------------------------------------------------------
sub extract_conditional_array {
    my ( $tag_id, $array_ref ) = @_;
    my @conditions;

    foreach my $tag_def (@$array_ref) {
        next unless ref $tag_def eq 'HASH';

        my $condition_data = extract_single_condition($tag_def);
        if ($condition_data) {
            push @conditions, $condition_data;
        }
    }

    return @conditions;
}

#------------------------------------------------------------------------------
# Extract condition data from a single tag definition
#------------------------------------------------------------------------------
sub extract_single_condition {
    my $tag_def = shift;

    return unless $tag_def->{Condition};

    my $condition_data = {
        condition => $tag_def->{Condition},
        name      => $tag_def->{Name} || 'Unknown',
    };

    # Add other relevant fields
    $condition_data->{description} = $tag_def->{Description}
      if $tag_def->{Description};
    $condition_data->{format}   = $tag_def->{Format} if $tag_def->{Format};
    $condition_data->{writable} = JSON::true         if $tag_def->{Writable};
    $condition_data->{subdirectory} = JSON::true if $tag_def->{SubDirectory};

    return $condition_data;
}

#------------------------------------------------------------------------------
# Extract model patterns from a condition string
#------------------------------------------------------------------------------
sub extract_model_patterns_from_condition {
    my $condition = shift;
    my @patterns;

    # Look for $$self{Model} patterns
    while ( $condition =~ /\$\$self\{Model\}\s*([!~=]+)\s*([^)]+)/g ) {
        my $operator = $1;
        my $pattern  = $2;

        # Clean up the pattern
        $pattern =~ s/^\s+|\s+$//g;    # Trim whitespace

        # Handle different pattern types
        if ( $pattern =~ m{^/(.+)/$} || $pattern =~ m{^/(.+)/[gimsx]*$} ) {

            # Regex pattern like /EOS/ or /EOS/i
            my $regex = $1;
            push @patterns,
              {
                type              => 'regex',
                operator          => $operator,
                pattern           => $regex,
                condition_context => $condition,
              };
        }
        elsif ( $pattern =~ /^["'](.+)["']$/ ) {

            # Quoted string like "EOS D30"
            my $string = $1;
            push @patterns,
              {
                type              => 'string',
                operator          => $operator,
                pattern           => $string,
                condition_context => $condition,
              };
        }
        else {
            # Variable or other expression
            push @patterns,
              {
                type              => 'expression',
                operator          => $operator,
                pattern           => $pattern,
                condition_context => $condition,
              };
        }
    }

    return @patterns;
}
