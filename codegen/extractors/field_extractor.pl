#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use JSON;
use Scalar::Util qw(blessed reftype);

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

# Global counters
my $total_symbols     = 0;
my $extracted_symbols = 0;
my $skipped_symbols   = 0;

if ( @ARGV != 1 ) {
    die "Usage: $0 <module_path>\n";
}

my $module_path = $ARGV[0];

# Extract module name from path
my $module_name = basename($module_path);
$module_name =~ s/\.pm$//;

print STDERR "Universal extraction starting for $module_name:\n";

# Load the module
my $package_name = "Image::ExifTool::$module_name";
eval "require $package_name";
if ($@) {
    die "Failed to load module $package_name: $@\n";
}

# Extract symbols
extract_symbols( $package_name, $module_name );

# Print summary
print STDERR "Universal extraction complete for $module_name:\n";
print STDERR "  Total symbols examined: $total_symbols\n";
print STDERR "  Successfully extracted: $extracted_symbols\n";
print STDERR "  Skipped (non-serializable): $skipped_symbols\n";

sub extract_symbols {
    my ( $package_name, $module_name ) = @_;

    # Get module's symbol table
    no strict 'refs';
    my $symbol_table = *{"${package_name}::"};

    # Examine each symbol in the package
    foreach my $symbol_name ( sort keys %$symbol_table ) {
        $total_symbols++;

        my $glob = $symbol_table->{$symbol_name};

        # Try to extract hash symbols (most important for ExifTool)
        if ( my $hash_ref = *$glob{HASH} ) {
            if (%$hash_ref) {    # Skip empty hashes
                extract_hash_symbol( $symbol_name, $hash_ref, $module_name );
            }
        }
    }
}

sub extract_hash_symbol {
    my ( $symbol_name, $hash_ref, $module_name ) = @_;

    # Simple check - skip if it looks complex
    my $clean_hash           = {};
    my $has_non_serializable = 0;
    my $size                 = 0;

    for my $key ( keys %$hash_ref ) {
        my $value = $hash_ref->{$key};
        $size++;

        # Only handle simple values for now
        if ( !ref($value) ) {

            # Simple scalar
            $clean_hash->{$key} = $value;
        }
        elsif ( ref($value) eq 'HASH' ) {

            # Simple nested hash - just count it
            $has_non_serializable = 1;
        }
        else {
            # Complex reference - skip
            $has_non_serializable = 1;
        }

        # Limit size to prevent huge output
        last if $size > 100;
    }

    # Only output if we have some simple data
    if ( keys %$clean_hash > 0 ) {
        my $symbol_data = {
            type     => 'hash',
            name     => $symbol_name,
            data     => $clean_hash,
            module   => $module_name,
            metadata => {
                size                 => scalar( keys %$clean_hash ),
                complexity           => 'simple',
                has_non_serializable => $has_non_serializable
            }
        };

        print JSON->new->canonical->encode($symbol_data) . "\n";
        $extracted_symbols++;
    }
    else {
        $skipped_symbols++;
    }
}
