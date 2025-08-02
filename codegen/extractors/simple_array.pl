#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         simple_array.pl
#
# Description:  Extract simple arrays from ExifTool modules
#
# Usage:        perl simple_array.pl <module_path> <array_expr> [<array_expr2> ...]
#
# Example:      perl simple_array.pl ../third-party/exiftool/lib/Image/ExifTool/Nikon.pm xlat[0] xlat[1]
#
# Notes:        This script extracts arrays with primitive element types.
#               Output is written to individual JSON files in current directory.
#               Supports complex expressions: xlat[0], afPoints231, obj->{prop}
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";
use Carp;

# Check required environment variables
unless ( $ENV{CODEGEN_DIR} && $ENV{REPO_ROOT} ) {
    croak
"Error: Required environment variables CODEGEN_DIR and REPO_ROOT must be set by the calling Rust code";
}

use ExifToolExtract qw(
  load_module_from_file
  get_package_array
  format_json_output
);

# Check arguments
if ( @ARGV < 2 ) {
    die "Usage: $0 <module_path> <array_expr> [<array_expr2> ...]\n"
      . "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Nikon.pm xlat[0] xlat[1]\n";
}

my $module_path       = shift @ARGV;
my @array_expressions = @ARGV;

# Validate module path
unless ( -f $module_path ) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{};    # Remove path

# Load the module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Extract each requested array
my @all_extractions;

for my $array_expr (@array_expressions) {

# Ensure array expression starts with @ if it's a simple name without complex syntax
    unless ( $array_expr =~ /[\[\]\-\>]/ || $array_expr =~ /^@/ ) {
        $array_expr = "@$array_expr";
    }

    # Get the array reference
    my $array_ref = get_package_array( $module_name, $array_expr );
    unless ($array_ref) {
        warn "Warning: Array $array_expr not found in $module_display_name\n";
        warn
"Note: Module should be patched by Rust orchestration before calling this script\n";
        next;
    }

    # Validate that it's actually an array reference
    unless ( ref($array_ref) eq 'ARRAY' ) {
        warn
"Warning: $array_expr is not an array reference in $module_display_name\n";
        next;
    }

    # Extract elements, validating they're primitive
    my @elements;
    for my $i ( 0 .. $#{$array_ref} ) {
        my $element = $array_ref->[$i];

        # Validate element is primitive (no complex Perl structures)
        if ( ref($element) ) {
            warn
"Warning: Non-primitive element at index $i in $array_expr (skipping)\n";
            next;
        }

        push @elements,
          {
            index => $i,
            value => $element,
          };
    }

    if (@elements) {
        push @all_extractions, {
            source => {
                module       => $module_display_name,
                array_expr   => $array_expr,
                extracted_at => scalar( gmtime() ) . " GMT",
            },
            metadata => {
                element_count => scalar(@elements),

                # Note: Size validation happens in Rust generator, not here
            },
            elements => \@elements,
        };

        print STDERR "Extracted "
          . scalar(@elements)
          . " elements from $array_expr\n";
    }
    else {
        warn "Warning: No primitive elements found in $array_expr\n";
    }
}

# Output individual JSON files for each array
if (@all_extractions) {
    for my $extraction (@all_extractions) {

        # Generate filename from array expression
        my $array_expr = $extraction->{source}{array_expr};

        # Convert array expression to safe filename
        # xlat[0] -> xlat_0, afPoints231 -> af_points231
        my $filename = $array_expr;
        $filename =~ s/^@//;            # Remove @ prefix
        $filename =~ s/\[(\d+)\]/_$1/g; # xlat[0] -> xlat_0
        $filename =~ s/([A-Z])/_\L$1/g; # Convert camelCase to snake_case
        $filename =~ s/^_//;            # Remove leading underscore
        $filename =~ s/[^a-z0-9_]/_/g;  # Replace non-safe chars with underscore
        $filename =
          lc($filename) . ".json";      # Ensure lowercase and add extension

        # Write to file
        open( my $fh, '>', $filename ) or die "Cannot write to $filename: $!";
        print $fh format_json_output($extraction);
        close($fh);

        print STDERR "Created $filename\n";
    }
}
else {
    die "Error: No arrays successfully extracted\n";
}

1;    # Return success
