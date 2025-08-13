#!/usr/bin/env perl

# P08: PPI AST Foundation - Simple field extractor with clean PPI AST support
#
# This script extracts symbols from ExifTool modules and adds simple,
# clean PPI AST parsing for expressions. No complex visitor patterns!

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use JSON::XS;
use Sub::Util    qw(subname);
use Scalar::Util qw(blessed reftype refaddr);

# Add our PPI library to path
use lib "$Bin";
use PPI::Simple;

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

# JSON serializer
my $json =
  JSON::XS->new->canonical(1)
  ->allow_blessed(1)
  ->convert_blessed(1)
  ->allow_nonref(1);

# PPI converter instance
my $ppi_converter = PPI::Simple->new(
    skip_whitespace   => 1,
    skip_comments     => 1,
    include_locations => 0,
    include_content   => 1,
);

if ( @ARGV < 1 ) {
    die "Usage: $0 <module_path> [field1] [field2] ...\n"
      . "  Extract all hash symbols from ExifTool module with clean PPI AST analysis\n"
      . "  Examples:\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool.pm\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool/Canon.pm\n";
}

my $module_path   = shift @ARGV;
my @target_fields = @ARGV;

# Extract module name from path
my $module_name = basename($module_path);
$module_name =~ s/\.pm$//;

# CAREFUL! The rust code **actually looks for this magic string**!
print STDERR "Field extraction with AST starting for $module_name:\n";

# Load the module - handle special case for main ExifTool.pm
my $package_name;
if ( $module_name eq 'ExifTool' ) {
    $package_name = "Image::ExifTool";
}
else {
    $package_name = "Image::ExifTool::$module_name";
}

eval "require $package_name";
if ($@) {
    die "Failed to load module $package_name: $@\n";
}

# Check if this module has composite tags (set by our patcher)
my $has_composite_tags = 0;
{
    no strict 'refs';
    $has_composite_tags = 1 if ${"${package_name}::__hasCompositeTags"};
}

# Extract symbols from symbol table
extract_symbols( $package_name, $module_name, \@target_fields,
    $has_composite_tags );

# Extract lexical arrays from source code
extract_lexical_arrays( $module_path, $module_name, \@target_fields );

print STDERR "Field extraction with AST complete for $module_name\n";

sub extract_symbols {
    my ( $package_name, $module_name, $target_fields, $has_composite_tags ) =
      @_;

    # Get module's symbol table
    no strict 'refs';
    my $symbol_table = *{"${package_name}::"};

    # Create a filter set if target fields are specified
    my %field_filter;
    if (@$target_fields) {
        %field_filter = map { $_ => 1 } @$target_fields;
    }

    # Examine each symbol in the package
    foreach my $symbol_name ( sort keys %$symbol_table ) {

        # Skip symbols not in our filter list (if filtering is enabled)
        next if ( @$target_fields && !exists $field_filter{$symbol_name} );

        my $glob = $symbol_table->{$symbol_name};

        # Try to extract hash symbols (most important for ExifTool)
        if ( my $hash_ref = *$glob{HASH} ) {
            if (%$hash_ref) {
                extract_hash_symbol(
                    $symbol_name, $hash_ref,
                    $module_name, $has_composite_tags
                );
            }
        }

        # Also try to extract array symbols
        elsif ( my $array_ref = *$glob{ARRAY} ) {
            if (@$array_ref) {
                extract_array_symbol( $symbol_name, $array_ref, $module_name );
            }
        }
    }
}

sub extract_array_symbol {
    my ( $symbol_name, $array_ref, $module_name ) = @_;

    # Filter out function references
    my $filtered_data = filter_code_refs($array_ref);

    # Add inline AST to any hash elements that have expressions
    add_inline_ast_to_data($filtered_data);

    # Package the data
    my $extracted = {
        name     => $symbol_name,
        module   => $module_name,
        type     => 'array',
        data     => $filtered_data,
        metadata => {
            size               => scalar(@$filtered_data),
            is_composite_table => 0,
        }
    };

    # Output the extracted array as JSON
    eval {
        my $json_data = $json->encode($extracted);
        print "$json_data\n";
    };
    if ($@) {
        print STDERR "Warning: Failed to serialize array $symbol_name: $@\n";
    }
}

sub extract_hash_symbol {
    my ( $symbol_name, $hash_ref, $module_name, $has_composite_tags ) = @_;

    # Detect composite tables
    my $is_composite_table =
      ( $symbol_name eq 'Composite' && $has_composite_tags );

    # Filter out function references
    my $filtered_data = filter_code_refs($hash_ref);
    my $size          = scalar( keys %$filtered_data );

    # Skip if no data after filtering
    return unless $size > 0;

    # For non-composite tables, apply size limit to prevent huge output
    if ( !$is_composite_table && $size > 1000 ) {
        print STDERR "Skipping large symbol: $symbol_name (size: $size)\n";
        return;
    }

    # Special processing for magic number patterns
    my $processed_data = $filtered_data;
    if ( $symbol_name eq 'magicNumber' ) {
        $processed_data = convert_magic_number_patterns($filtered_data);
    }

    # Add inline AST to expressions
    add_inline_ast_to_data($processed_data);

    my $symbol_data = {
        type     => 'hash',
        name     => $symbol_name,
        data     => $processed_data,
        module   => $module_name,
        metadata => {
            size               => $size,
            is_composite_table => $is_composite_table ? 1 : 0,
        }
    };

    eval { print $json->encode($symbol_data) . "\n"; };
    if ($@) {
        print STDERR "Warning: Failed to serialize $symbol_name: $@\n";
    }
}

# Recursively find and add AST to expressions
sub add_inline_ast_to_data {
    my ($data) = @_;

    if ( ref $data eq 'HASH' ) {

        # Check if this is a tag hash with expression fields
        for my $expr_type (qw(PrintConv ValueConv RawConv Condition)) {
            if ( exists $data->{$expr_type} ) {
                my $expr_value = $data->{$expr_type};

                if ( ref $expr_value eq ''
                    && is_potential_expression($expr_value) )
                {
                    my $ast = $ppi_converter->parse_expression($expr_value);
                    if ($ast) {
                        $data->{"${expr_type}_ast"} = $ast;
                    }
                }
                elsif ( ref $expr_value eq 'HASH'
                    && exists $expr_value->{OTHER} )
                {
                    $data->{"${expr_type}_note"} =
"Contains OTHER function reference - requires manual implementation";
                }
            }
        }

        # Recurse into nested structures
        for my $value ( values %$data ) {
            add_inline_ast_to_data($value);
        }
    }
    elsif ( ref $data eq 'ARRAY' ) {

        # Recurse into array elements
        for my $element (@$data) {
            add_inline_ast_to_data($element);
        }
    }
}

# Simple check if string looks like a Perl expression
sub is_potential_expression {
    my ($string) = @_;

    return 0 if length($string) < 3;

    # Look for Perl expression indicators
    return 1 if $string =~ /\$\w+/;               # Variables
    return 1 if $string =~ /\?\s*.*\s*:/;         # Ternary operator
    return 1 if $string =~ /sprintf\s*\(/;        # sprintf calls
    return 1 if $string =~ /[+\-*\/]\s*\$\w+/;    # Arithmetic with variables
    return 1 if $string =~ /\$\w+\s*[+\-*\/]/;    # Variables with arithmetic

    return 0;
}

sub filter_code_refs {
    my ( $data, $depth, $seen ) = @_;
    $depth //= 0;
    $seen  //= {};

    return "[MaxDepth]" if $depth > 10;

    if ( !ref($data) ) {
        return $data;
    }
    elsif ( reftype($data) eq 'CODE' ) {
        my $name = subname($data);
        return defined($name) ? "[Function: $name]" : "[Function: __ANON__]";
    }
    elsif ( reftype($data) eq 'HASH' ) {
        my $addr = refaddr($data);
        return "[Circular]" if $seen->{$addr};
        $seen->{$addr} = 1;

        my $filtered = {};
        for my $key ( keys %$data ) {
            if (   $key eq 'Table'
                && defined( reftype( $data->{$key} ) )
                && reftype( $data->{$key} ) eq 'HASH' )
            {
                if ( blessed( $data->{$key} ) ) {
                    $filtered->{$key} =
                      "[TableRef: " . blessed( $data->{$key} ) . "]";
                }
                else {
                    $filtered->{$key} = "[TableRef: HASH]";
                }
            }
            else {
                $filtered->{$key} =
                  filter_code_refs( $data->{$key}, $depth + 1, $seen );
            }
        }

        delete $seen->{$addr};
        return $filtered;
    }
    elsif ( reftype($data) eq 'ARRAY' ) {
        my $filtered = [];
        for my $item (@$data) {
            push @$filtered, filter_code_refs( $item, $depth + 1, $seen );
        }
        return $filtered;
    }
    elsif ( reftype($data) eq 'SCALAR' ) {
        return "[ScalarRef: " . $$data . "]";
    }
    elsif ( reftype($data) eq 'GLOB' ) {
        return "[Glob: " . ( $$data || 'UNKNOWN' ) . "]";
    }
    elsif ( blessed($data) ) {
        return "[Object: " . blessed($data) . "]";
    }
    else {
        my $ref_type = reftype($data) || ref($data) || 'UNKNOWN';
        return "[Ref: $ref_type]";
    }
}

sub extract_lexical_arrays {
    my ( $module_path, $module_name, $target_fields ) = @_;

    open my $fh, '<', $module_path or do {
        print STDERR
"Warning: Cannot open source file $module_path for array extraction: $!\n";
        return;
    };

    my $source_code = do { local $/; <$fh> };
    close $fh;

    # Create a filter set if target fields are specified
    my %field_filter;
    if (@$target_fields) {
        %field_filter = map { $_ => 1 } @$target_fields;
    }

    # Look for lexical array declarations: my @varname = (array_content);
    while ( $source_code =~ /^my\s+\@(\w+)\s*=\s*\(\s*(.*?)\s*\);/gms ) {
        my $array_name    = $1;
        my $array_content = $2;

        # Skip if filtering and not in target list
        next if ( @$target_fields && !exists $field_filter{$array_name} );

        # Parse the array content
        my $parsed_array = parse_perl_array($array_content);

        if ($parsed_array) {
            my $symbol_data = {
                type     => 'array',
                name     => $array_name,
                data     => $parsed_array,
                module   => $module_name,
                metadata => {
                    size               => scalar(@$parsed_array),
                    is_composite_table => 0,
                }
            };

            eval { print $json->encode($symbol_data) . "\n"; };
            if ($@) {
                print STDERR
                  "Warning: Failed to serialize array $array_name: $@\n";
            }
        }
    }
}

sub parse_perl_array {
    my ($array_content) = @_;

    $array_content =~ s/^\s+|\s+$//g;

    # Handle nested arrays like [ [...], [...] ]
    if ( $array_content =~ /^\s*\[\s*(.*?)\s*\]\s*,\s*\[\s*(.*?)\s*\]\s*$/s ) {
        my @result     = ();
        my @sub_arrays = split /\]\s*,\s*\[/, $array_content;

        for my $sub_array (@sub_arrays) {
            $sub_array =~ s/^\s*\[?\s*|\s*\]?\s*$//g;
            my $parsed_sub = parse_array_elements($sub_array);
            push @result, $parsed_sub if $parsed_sub;
        }

        return \@result;
    }
    else {
        return parse_array_elements($array_content);
    }
}

sub parse_array_elements {
    my ($elements_str) = @_;

    my @elements     = ();
    my @raw_elements = split /,/, $elements_str;

    for my $element (@raw_elements) {
        $element =~ s/^\s+|\s+$//g;
        next unless $element;

        if ( $element =~ /^0x([0-9a-fA-F]+)$/ ) {
            push @elements, hex($1);
        }
        elsif ( $element =~ /^-?\d+(\.\d+)?$/ ) {
            push @elements, $element + 0;
        }
        elsif ( $element =~ /^['\"](.*)['\"]$/ ) {
            push @elements, $1;
        }
        elsif ( $element =~ /^[a-zA-Z_]\w*$/ ) {
            push @elements, $element;
        }
    }

    return \@elements;
}

sub convert_magic_number_patterns {
    my ($patterns_hash) = @_;
    my %converted;

    for my $file_type ( keys %$patterns_hash ) {
        my $pattern = $patterns_hash->{$file_type};

        my @raw_bytes;
        for my $i ( 0 .. length($pattern) - 1 ) {
            push @raw_bytes, ord( substr( $pattern, $i, 1 ) );
        }

        $converted{$file_type} = { raw_bytes => \@raw_bytes };
    }

    return \%converted;
}
