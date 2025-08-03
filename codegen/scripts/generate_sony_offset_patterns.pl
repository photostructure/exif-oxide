#!/usr/bin/env perl
#------------------------------------------------------------------------------
# File:         generate_sony_offset_patterns.pl
#
# Description:  Generate Rust code for Sony offset patterns
#
# Usage:        perl generate_sony_offset_patterns.pl
#
# Note:         This script expects sony_offset_patterns.json to exist
#               and generates src/generated/Sony_pm/offset_patterns.rs
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use JSON;
use File::Path qw(make_path);

# Read the JSON data
my $json_file = "$Bin/sony_offset_patterns.json";
unless ( -f $json_file ) {
    die
      "Error: $json_file not found. Run the offset pattern extractor first.\n";
}

open my $fh, '<', $json_file or die "Cannot open $json_file: $!";
my $json_text = do { local $/; <$fh> };
close $fh;

my $data = decode_json($json_text);

# Generate Rust code
my $rust_code = generate_rust_code($data);

# Ensure output directory exists
my $output_dir = "$Bin/../src/generated/Sony_pm";
make_path($output_dir) unless -d $output_dir;

# Write the Rust file
my $output_file = "$output_dir/offset_patterns.rs";
open my $out_fh, '>', $output_file or die "Cannot write to $output_file: $!";
print $out_fh $rust_code;
close $out_fh;

print "Generated $output_file\n";

#------------------------------------------------------------------------------
# Generate Rust code from the extracted data
#------------------------------------------------------------------------------
sub generate_rust_code {
    my $data = shift;
    my $code = "";

    # Header
    $code .= "//! Sony offset calculation patterns\n";
    $code .= "//! ExifTool: " . $data->{source}->{module} . "\n";
    $code .= "//! Generated: " . scalar(localtime) . "\n\n";

    # Imports
    $code .= "use std::collections::HashMap;\n";
    $code .= "use std::sync::LazyLock;\n\n";

    # Model conditions
    my $patterns = $data->{offset_patterns};
    if ( @{ $patterns->{model_conditions} } ) {
        $code .= "/// Sony model condition patterns\n";
        $code .=
"pub static SONY_MODEL_CONDITIONS: LazyLock<Vec<(&str, &str, &str)>> = LazyLock::new(|| {\n";
        $code .= "    vec![\n";

        foreach my $cond ( @{ $patterns->{model_conditions} } ) {
            next unless $cond->{type} eq 'regex';
            my $pattern = escape_rust_string( $cond->{pattern} );
            $code .= sprintf(
                "        (\"%s\", \"%s\", \"%s\"),\n",
                escape_rust_string( $cond->{operator} ),
                $pattern, $cond->{type}
            );
        }

        $code .= "    ]\n";
        $code .= "});\n\n";
    }

    # Offset calculations summary
    if ( @{ $patterns->{offset_calculations} } ) {
        $code .= "/// Summary of offset calculation patterns found\n";
        $code .=
"pub static OFFSET_CALCULATION_TYPES: LazyLock<Vec<&str>> = LazyLock::new(|| {\n";
        $code .= "    vec![\n";

        my %seen_ops;
        foreach my $calc ( @{ $patterns->{offset_calculations} } ) {
            next if $seen_ops{ $calc->{operation} };
            next if $calc->{operation} eq 'complex_expression';
            $seen_ops{ $calc->{operation} } = 1;
            $code .= sprintf( "        \"%s\",  // e.g., %s\n",
                $calc->{operation},
                escape_rust_string( $calc->{raw_expression} ) );
        }

        $code .= "    ]\n";
        $code .= "});\n\n";
    }

    # Simple offset calculation examples
    $code .= "/// Example offset calculations extracted from Sony.pm\n";
    $code .= "/// These demonstrate the patterns that need to be implemented\n";
    $code .=
"pub static OFFSET_EXAMPLES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {\n";
    $code .= "    let mut map = HashMap::new();\n";

    my %example_added;
    foreach my $calc ( @{ $patterns->{offset_calculations} } ) {
        next if $calc->{operation} eq 'complex_expression';
        next if $example_added{ $calc->{operation} };
        $example_added{ $calc->{operation} } = 1;

        $code .= sprintf( "    map.insert(\"%s\", \"%s\");\n",
            $calc->{operation}, escape_rust_string( $calc->{raw_expression} ) );
    }

    $code .= "    map\n";
    $code .= "});\n";

    return $code;
}

#------------------------------------------------------------------------------
# Escape a string for Rust
#------------------------------------------------------------------------------
sub escape_rust_string {
    my $str = shift;
    $str =~ s/\\/\\\\/g;
    $str =~ s/"/\\"/g;
    $str =~ s/\n/\\n/g;
    $str =~ s/\r/\\r/g;
    $str =~ s/\t/\\t/g;
    return $str;
}
