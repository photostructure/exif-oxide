#!/usr/bin/env perl
use strict;
use warnings;
use JSON;
use File::Basename;
use File::Spec;
use Getopt::Long;

# Script to validate that our generated Rust arrays exactly match ExifTool's perl arrays
# Usage: perl validate_arrays.pl [--verbose] [--array xlat[0]] config/Nikon_pm/simple_array.json

my $verbose        = 0;
my $specific_array = '';
GetOptions(
    'verbose' => \$verbose,
    'array=s' => \$specific_array,
) or die "Usage: $0 [--verbose] [--array ARRAY_NAME] CONFIG_FILE\n";

my $config_file = $ARGV[0]
  or die "Usage: $0 [--verbose] [--array ARRAY_NAME] CONFIG_FILE\n";

# Read the simple_array config
open my $fh, '<', $config_file or die "Cannot read $config_file: $!";
my $config_json = do { local $/; <$fh> };
close $fh;

my $config      = decode_json($config_json);
my $source_file = $config->{source};
my $arrays      = $config->{arrays};

print "🔍 Validating arrays from $source_file\n";
print "📋 Found " . scalar(@$arrays) . " array(s) to validate\n\n";

# Add ExifTool to the library path
my $exiftool_lib_dir =
  File::Spec->catdir( dirname(__FILE__), "..", "..", "third-party", "exiftool",
    "lib" );
unshift @INC, $exiftool_lib_dir;

# Load ExifTool and patch the Nikon module to access the arrays
my $exiftool_path =
  File::Spec->catfile( $exiftool_lib_dir, "Image", "ExifTool", "Nikon.pm" );
unless ( -f $exiftool_path ) {
    die "❌ ExifTool source not found at: $exiftool_path\n";
}

# Read and patch the Nikon module to make the arrays accessible
my $patched_code = patch_nikon_module($exiftool_path);

# Evaluate the patched code to load the arrays
eval $patched_code;
die "❌ Failed to load patched Nikon module: $@" if $@;

my $total_arrays  = 0;
my $passed_arrays = 0;
my $failed_arrays = 0;

foreach my $array_config (@$arrays) {
    my $array_name    = $array_config->{array_name};
    my $constant_name = $array_config->{constant_name};
    my $expected_size = $array_config->{size};

    # Skip if user specified a specific array and this isn't it
    if ( $specific_array && $array_name ne $specific_array ) {
        next;
    }

    $total_arrays++;

    print "🔧 Validating $array_name -> $constant_name\n";

    # Get the array from the loaded module
    my @perl_array = get_perl_array($array_name);

    if ( !@perl_array ) {
        print "❌ Failed to extract perl array $array_name\n";
        $failed_arrays++;
        next;
    }

    print "   📊 Extracted " . scalar(@perl_array) . " elements from perl\n"
      if $verbose;

    # Load the corresponding Rust file
    my $rust_file = find_rust_file($constant_name);
    if ( !$rust_file ) {
        print "❌ Could not find Rust file for $constant_name\n";
        $failed_arrays++;
        next;
    }

    print "   📂 Reading Rust file: $rust_file\n" if $verbose;

    # Parse the Rust array
    my @rust_array = parse_rust_array( $rust_file, $constant_name );
    if ( !@rust_array ) {
        print "❌ Failed to parse Rust array from $rust_file\n";
        $failed_arrays++;
        next;
    }

    print "   📊 Parsed " . scalar(@rust_array) . " elements from Rust\n"
      if $verbose;

    # Validate size
    if ( scalar(@perl_array) != scalar(@rust_array) ) {
        print "❌ Size mismatch: perl="
          . scalar(@perl_array)
          . " rust="
          . scalar(@rust_array) . "\n";
        $failed_arrays++;
        next;
    }

    if ( $expected_size && scalar(@perl_array) != $expected_size ) {
        print "⚠️  Size mismatch with config: expected=$expected_size actual="
          . scalar(@perl_array) . "\n";
    }

    # Validate each element
    my $element_errors = 0;
    for my $i ( 0 .. $#perl_array ) {

        # Convert hex values to decimal for comparison
        my $perl_val = $perl_array[$i];
        my $rust_val = $rust_array[$i];

        # Handle hex strings from perl (0xc1 -> 193)
        if ( $perl_val =~ /^0x/i ) {
            $perl_val = hex($perl_val);
        }

        if ( $perl_val != $rust_val ) {
            print "❌ Element mismatch at index $i: perl=$perl_val (0x"
              . sprintf( "%02x", $perl_val )
              . ") rust=$rust_val\n";
            $element_errors++;

            # Limit error output
            if ( $element_errors >= 10 ) {
                print "   ... (truncated after 10 errors)\n";
                last;
            }
        }
    }

    if ( $element_errors > 0 ) {
        print
          "❌ $array_name failed validation ($element_errors element errors)\n";
        $failed_arrays++;
    }
    else {
        print "✅ $array_name passed validation (all "
          . scalar(@perl_array)
          . " elements match)\n";
        $passed_arrays++;
    }

    print "\n";
}

# Summary
print "📋 Validation Summary:\n";
print "   Total arrays: $total_arrays\n";
print "   ✅ Passed: $passed_arrays\n";
print "   ❌ Failed: $failed_arrays\n";

if ( $failed_arrays > 0 ) {
    print "\n❌ VALIDATION FAILED - some arrays have mismatches!\n";
    exit 1;
}
else {
    print "\n🎉 ALL ARRAYS VALIDATED SUCCESSFULLY!\n";
    exit 0;
}

sub patch_nikon_module {
    my ($nikon_path) = @_;

    open my $fh, '<', $nikon_path or die "Cannot read $nikon_path: $!";
    my $content = do { local $/; <$fh> };
    close $fh;

    # Convert 'my @xlat = (' to 'our @xlat = (' to make it accessible
    $content =~ s/\bmy\s+(\@\w+)\s*=/our $1 =/g;

    # Add package declaration if missing
    unless ( $content =~ /^\s*package\s+Image::ExifTool::Nikon/m ) {
        $content = "package Image::ExifTool::Nikon;\n" . $content;
    }

    return $content;
}

sub get_perl_array {
    my ($array_expr) = @_;

    # Handle different array expression formats
    if ( $array_expr eq 'xlat[0]' ) {

        # Access the first sub-array of @xlat
        no strict 'refs';
        my @array = @{"Image::ExifTool::Nikon::xlat"};
        return @{ $array[0] } if ref( $array[0] ) eq 'ARRAY';
    }
    elsif ( $array_expr eq 'xlat[1]' ) {

        # Access the second sub-array of @xlat
        no strict 'refs';
        my @array = @{"Image::ExifTool::Nikon::xlat"};
        return @{ $array[1] } if ref( $array[1] ) eq 'ARRAY';
    }

    # Add more array access patterns as needed
    return ();
}

sub find_rust_file {
    my ($constant_name) = @_;

    # Convert XLAT_0 to xlat_0.rs
    my $filename = lc($constant_name) . ".rs";

    # Look in the standard generated location
    my $rust_file =
      File::Spec->catfile( dirname(__FILE__), "..", "..", "src", "generated",
        "Nikon_pm", $filename );

    return -f $rust_file ? $rust_file : undef;
}

sub parse_rust_array {
    my ( $rust_file, $constant_name ) = @_;

    open my $fh, '<', $rust_file or die "Cannot read $rust_file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;

    # Find the array declaration: pub static XLAT_0: [u8; 256] = [
    my $pattern =
qr/pub\s+static\s+\Q$constant_name\E:\s*\[[^;]+;\s*\d+\]\s*=\s*\[(.*?)\];/s;

    if ( $content =~ /$pattern/ ) {
        my $array_content = $1;

        # Extract all numbers (handle multi-line formatting)
        my @numbers = $array_content =~ /(\d+)/g;

        return @numbers;
    }

    return ();
}
