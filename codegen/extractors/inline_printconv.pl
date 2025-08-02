#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         inline_printconv.pl
#
# Description:  Extract inline PrintConv tables from ExifTool tag definitions
#
# ⚠️ DEPRECATED: Use tag_kit.pl instead for complete tag extraction with PrintConvs
#
# This extractor is being replaced by the tag kit system which provides complete
# tag definitions including PrintConv data in a single unified structure.
# See EXTRACTOR-GUIDE.md for more information.
#
# Usage:        perl inline_printconv.pl <module_path> <table_name>
#
# Example:      perl inline_printconv.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm CameraSettings
#
# Notes:        This script extracts inline PrintConv hash definitions from
#               tag tables. Uses the Perl interpreter to properly parse the
#               tag structures - no regex parsing of Perl code!
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
  load_module_from_file
  get_package_hash
  validate_primitive_value
  format_json_output
);

# Check arguments
if ( @ARGV != 2 ) {
    die "Usage: $0 <module_path> <table_name>\n"
      . "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm CameraSettings\n";
}

my ( $module_path, $table_name ) = @ARGV;

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

# Get the tag table
my $table_ref = get_package_hash( $module_name, $table_name );
unless ($table_ref) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract inline PrintConv definitions
my @inline_printconvs;
my $extracted_count = 0;

for my $tag_id ( sort keys %$table_ref ) {

    # Skip special ExifTool keys (all uppercase or special values)
    next if $tag_id =~ /^[A-Z_]+$/;
    next if $tag_id eq 'Notes';

    my $tag_info = $table_ref->{$tag_id};

    # Skip if not a hash reference
    next unless ref $tag_info eq 'HASH';

    # Get tag name
    my $tag_name = $tag_info->{Name} || "Tag$tag_id";

    # Check for PrintConv
    if ( exists $tag_info->{PrintConv} ) {
        my $print_conv = $tag_info->{PrintConv};

        # We're looking for inline hash definitions
        if ( ref $print_conv eq 'HASH' ) {

            # Check if this is a simple hash (not containing special structures)
            if ( is_simple_printconv_hash($print_conv) ) {

                # Extract the key-value pairs
                my %entries;
                my $has_valid_entries = 0;

                for my $key ( sort keys %$print_conv ) {

                    # Skip special keys
                    next if $key eq 'OTHER';
                    next if $key eq 'Notes';
                    next if $key eq 'BITMASK';    # Skip BITMASK tables for now

                    my $value = $print_conv->{$key};

                    # Only extract primitive values
                    if ( validate_primitive_value($value) ) {
                        $entries{$key} = $value;
                        $has_valid_entries = 1;
                    }
                }

                if ($has_valid_entries) {

                    # Determine key type based on the keys
                    my $key_type = determine_key_type( keys %entries );

                    my $inline_printconv = {
                        tag_id   => $tag_id,
                        tag_name => $tag_name,
                        key_type => $key_type,
                        entries  => \%entries,
                    };

                    push @inline_printconvs, $inline_printconv;

                    $extracted_count++;
                    print STDERR
                      "  Found inline PrintConv for $tag_name (tag $tag_id): "
                      . scalar( keys %entries )
                      . " entries\n";
                }
            }
        }
    }
}

# Output results
if (@inline_printconvs) {
    my $output = {
        source => {
            module       => $module_display_name,
            table        => $table_name,
            extracted_at => scalar( gmtime() ) . " GMT",
        },
        metadata => {
            total_tags_scanned      => scalar( keys %$table_ref ),
            inline_printconvs_found => $extracted_count,
        },
        inline_printconvs => \@inline_printconvs,
    };

    # Write to file
    my $filename = "inline_printconv_${table_name}.json";
    $filename =~ s/([A-Z])/_\L$1/g;    # Convert camelCase to snake_case
    $filename =~ s/^_//;               # Remove leading underscore
    $filename = lc($filename);

    open( my $fh, '>', $filename ) or die "Cannot write to $filename: $!";
    print $fh format_json_output($output);
    close($fh);

    print STDERR "Created $filename\n";
    print STDERR
"Extracted $extracted_count inline PrintConv definitions from $table_name\n";
}
else {
    print STDERR "No inline PrintConv definitions found in $table_name\n";

    # Exit successfully - it's not an error if a table has no inline PrintConv
    exit 0;
}

#------------------------------------------------------------------------------
# Check if a PrintConv hash is simple (no complex structures)
#------------------------------------------------------------------------------
sub is_simple_printconv_hash {
    my ($print_conv) = @_;

    for my $key ( keys %$print_conv ) {
        my $value = $print_conv->{$key};

# Skip if value is a reference (except for BITMASK which we'll handle specially)
        if ( ref $value && $key ne 'BITMASK' ) {
            return 0;
        }

        # Skip if value contains Perl code
        if ( !ref $value && $value =~ /[\$\@\%]/ ) {
            return 0;
        }

        # Skip if value looks like code
        if ( !ref $value && $value =~ /\bsub\s*\{/ ) {
            return 0;
        }
    }

    return 1;
}

#------------------------------------------------------------------------------
# Determine the key type based on the keys in the hash
#------------------------------------------------------------------------------
sub determine_key_type {
    my @keys = @_;

    my $has_negative       = 0;
    my $has_large          = 0;
    my $has_very_large     = 0;
    my $has_string         = 0;
    my $has_negative_large = 0;

    for my $key (@keys) {
        if ( $key !~ /^-?\d+$/ ) {

            # Non-numeric key
            $has_string = 1;
        }
        elsif ( $key < 0 ) {
            $has_negative = 1;

            # Check if negative value exceeds i16 range
            if ( $key < -32768 ) {
                $has_negative_large = 1;
            }
        }
        elsif ( $key > 2147483647 ) {

            # Exceeds i32 max
            $has_very_large = 1;
        }
        elsif ( $key > 32767 ) {
            $has_large = 1;
        }
    }

    # Return appropriate type
    if ($has_string) {
        return "String";
    }
    elsif ($has_very_large) {
        return "u32";    # Unsigned 32-bit for very large values
    }
    elsif ( $has_negative_large || $has_large ) {
        return "i32";    # Signed 32-bit for safety
    }
    elsif ($has_negative) {
        return "i16";    # Signed 16-bit (fits in range)
    }
    elsif ( grep { $_ > 255 } @keys ) {
        return "u16";    # Unsigned 16-bit
    }
    else {
        return "u8";     # Unsigned 8-bit
    }
}
