#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use JSON::XS;
use Sub::Util    qw(subname);
use Scalar::Util qw(blessed reftype refaddr);

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

# Global counters and debugging
my $total_symbols     = 0;
my $extracted_symbols = 0;
my $skipped_symbols   = 0;
my @skipped_list      = ();

# JSON serializer - let JSON::XS handle complex structures automatically
my $json =
  JSON::XS->new->canonical(1)
  ->allow_blessed(1)
  ->convert_blessed(1)
  ->allow_nonref(1);

if ( @ARGV < 1 ) {
    die "Usage: $0 <module_path> [field1] [field2] ...\n"
      . "  Extract all hash symbols from ExifTool module, optionally filtered to specific fields\n"
      . "  Examples:\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool.pm\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool.pm fileTypeLookup magicNumber mimeType\n";
}

my $module_path   = shift @ARGV;
my @target_fields = @ARGV;         # Optional list of specific fields to extract

# Extract module name from path
my $module_name = basename($module_path);
$module_name =~ s/\.pm$//;

# CAREFUL! The rust code **actually looks for this magic string**!
print STDERR "Field extraction starting for $module_name:\n";

# Load the module - handle special case for main ExifTool.pm
my $package_name;
if ( $module_name eq 'ExifTool' ) {

    # Main ExifTool.pm uses package "Image::ExifTool"
    $package_name = "Image::ExifTool";
}
else {
    # All other modules use "Image::ExifTool::ModuleName"
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

# Print summary
print STDERR "Field extraction complete for $module_name:\n";
print STDERR "  Total symbols examined: $total_symbols\n";
print STDERR "  Successfully extracted: $extracted_symbols\n";
print STDERR "  Skipped (non-serializable): $skipped_symbols\n";

# Print debug info about skipped symbols if requested
if ( $ENV{DEBUG} && @skipped_list ) {
    print STDERR "\nSkipped symbols:\n";
    for my $skipped (@skipped_list) {
        print STDERR "  - $skipped\n";
    }
}

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
        print STDERR "  Filtering for specific fields: "
          . join( ", ", @$target_fields ) . "\n";
    }

    # Examine each symbol in the package
    foreach my $symbol_name ( sort keys %$symbol_table ) {
        $total_symbols++;

        # Skip symbols not in our filter list (if filtering is enabled)
        if ( @$target_fields && !exists $field_filter{$symbol_name} ) {
            print STDERR "  Skipping symbol (not in filter): $symbol_name\n"
              if $ENV{DEBUG};
            next;
        }

        print STDERR "  Processing symbol: $symbol_name\n" if $ENV{DEBUG};

        my $glob = $symbol_table->{$symbol_name};

        # Try to extract hash symbols (most important for ExifTool)
        if ( my $hash_ref = *$glob{HASH} ) {
            if (%$hash_ref) {    # Skip empty hashes
                my $hash_size = scalar( keys %$hash_ref );
                print STDERR "    Hash found with $hash_size keys\n"
                  if $ENV{DEBUG};
                extract_hash_symbol(
                    $symbol_name, $hash_ref,
                    $module_name, $has_composite_tags
                );
                print STDERR "    Hash extraction completed for $symbol_name\n"
                  if $ENV{DEBUG};
            }
        }

        # Also try to extract array symbols
        elsif ( my $array_ref = *$glob{ARRAY} ) {
            if (@$array_ref) {    # Skip empty arrays
                my $array_size = scalar(@$array_ref);
                print STDERR "    Array found with $array_size elements\n"
                  if $ENV{DEBUG};
                extract_array_symbol( $symbol_name, $array_ref, $module_name );
                print STDERR "    Array extraction completed for $symbol_name\n"
                  if $ENV{DEBUG};
            }
        }
        else {
            print STDERR "    No hash or array found for $symbol_name\n"
              if $ENV{DEBUG};
        }
    }
}

sub extract_array_symbol {
    my ( $symbol_name, $array_ref, $module_name ) = @_;

    print STDERR "    Starting array extraction for: $symbol_name\n"
      if $ENV{DEBUG};

 # Filter out function references if any (though arrays usually don't have them)
    my $filtered_data = filter_code_refs($array_ref);

    # Package the data with metadata
    my $extracted = {
        name     => $symbol_name,
        module   => $module_name,
        type     => 'array',
        data     => $filtered_data,
        metadata => {
            size               => scalar(@$filtered_data),
            is_composite_table => 0,    # Arrays are never composite tables
        }
    };

    # Output the extracted array as JSON
    eval {
        my $json_data = encode_json($extracted);
        print "$json_data\n";
        print STDERR "    Successfully extracted array: $symbol_name\n"
          if $ENV{DEBUG};
    };
    if ($@) {
        print STDERR "    Failed to encode array $symbol_name: $@\n";
    }
}

sub extract_hash_symbol {
    my ( $symbol_name, $hash_ref, $module_name, $has_composite_tags ) = @_;

    print STDERR "    Starting extraction for: $symbol_name\n" if $ENV{DEBUG};

    # Detect composite tables by checking if this module called AddCompositeTags
    # AND the symbol is named exactly "Composite"
    my $is_composite_table = 0;
    if ( $symbol_name eq 'Composite' && $has_composite_tags ) {
        $is_composite_table = 1;
        print STDERR
          "    Composite table detected (has AddCompositeTags marker)\n"
          if $ENV{DEBUG};
    }

    # Filter out function references (JSON::XS can't handle them)
    print STDERR "    Filtering code references...\n" if $ENV{DEBUG};
    my $filtered_data = filter_code_refs($hash_ref);
    print STDERR "    Code reference filtering completed\n" if $ENV{DEBUG};
    my $size = scalar( keys %$filtered_data );

    # Skip if no data after filtering
    return unless $size > 0;

    # For non-composite tables, apply size limit to prevent huge output
    if ( !$is_composite_table && $size > 1000 ) {
        $skipped_symbols++;
        push @skipped_list, "$module_name:$symbol_name (size: $size)";
        print STDERR "  Skipping large symbol: $symbol_name (size: $size)\n"
          if $ENV{DEBUG};
        return;
    }

    # Special processing for magic number patterns
    my $processed_data = $filtered_data;
    if ( $symbol_name eq 'magicNumber' ) {
        print STDERR "    Processing magic number patterns...\n" if $ENV{DEBUG};
        $processed_data = convert_magic_number_patterns($filtered_data);
        print STDERR "    Magic number pattern processing completed\n"
          if $ENV{DEBUG};
    }

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

    eval {
        print $json->encode($symbol_data) . "\n";
        $extracted_symbols++;
        print STDERR "  Extracted: $symbol_name ("
          . ( $is_composite_table ? 'composite table' : 'regular hash' )
          . ", size: $size)\n"
          if $ENV{DEBUG};
    };
    if ($@) {
        $skipped_symbols++;
        push @skipped_list, "$module_name:$symbol_name (JSON error: $@)";
        print STDERR "  Warning: Failed to serialize $symbol_name: $@\n";
    }
}

sub filter_code_refs {
    my ( $data, $depth, $seen ) = @_;
    $depth //= 0;
    $seen  //= {};

    # Prevent excessive recursion depth
    return "[MaxDepth]" if $depth > 10;

    if ( !ref($data) ) {
        return $data;
    }
    elsif ( reftype($data) eq 'CODE' ) {

        # Convert function reference to function name
        my $name = subname($data);
        return defined($name) ? "[Function: $name]" : "[Function: __ANON__]";
    }
    elsif ( reftype($data) eq 'HASH' ) {

        # Check for circular references using memory address
        my $addr = refaddr($data);
        return "[Circular]" if $seen->{$addr};
        $seen->{$addr} = 1;

        my $filtered = {};
        for my $key ( keys %$data ) {

            # Check if this is a Table reference that could cause circularity
            # Use reftype to check physical type, ignoring blessing
            if (   $key eq 'Table'
                && defined( reftype( $data->{$key} ) )
                && reftype( $data->{$key} ) eq 'HASH' )
            {
      # Replace Table references with string representation to break circularity
      # These are metadata pointers in ExifTool, not structural data
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

        # Remove from seen after processing to allow legitimate re-references
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
    elsif ( blessed($data) ) {
        return "[Object: " . blessed($data) . "]";
    }
    else {
        # Fallback for other reference types
        my $ref_type = reftype($data) || ref($data) || 'UNKNOWN';
        return "[Ref: $ref_type]";
    }
}

sub extract_lexical_arrays {
    my ( $module_path, $module_name, $target_fields ) = @_;

    # Read the source file
    open my $fh, '<', $module_path or do {
        print STDERR
"Warning: Cannot open source file $module_path for array extraction: $!\n";
        return;
    };

    my $source_code = do { local $/; <$fh> };
    close $fh;

    print STDERR "  Scanning source for lexical arrays...\n" if $ENV{DEBUG};

    # Create a filter set if target fields are specified
    my %field_filter;
    if (@$target_fields) {
        %field_filter = map { $_ => 1 } @$target_fields;
    }

    # Look for lexical array declarations
    # Pattern: my @varname = (array_content);
    while ( $source_code =~ /^my\s+\@(\w+)\s*=\s*\(\s*(.*?)\s*\);/gms ) {
        my $array_name    = $1;
        my $array_content = $2;

        $total_symbols++;

        # Skip if filtering and not in target list
        if ( @$target_fields && !exists $field_filter{$array_name} ) {
            print STDERR "  Skipping array (not in filter): $array_name\n"
              if $ENV{DEBUG};
            next;
        }

        print STDERR "  Found lexical array: \@$array_name\n" if $ENV{DEBUG};

        # Parse the array content
        my $parsed_array = parse_perl_array($array_content);

        if ($parsed_array) {

            # Create symbol data for the array
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

            eval {
                print $json->encode($symbol_data) . "\n";
                $extracted_symbols++;
                print STDERR "  Extracted array: $array_name (size: "
                  . scalar(@$parsed_array) . ")\n"
                  if $ENV{DEBUG};
            };
            if ($@) {
                $skipped_symbols++;
                push @skipped_list, "$module_name:$array_name (JSON error: $@)";
                print STDERR
                  "  Warning: Failed to serialize array $array_name: $@\n";
            }
        }
        else {
            print STDERR
              "  Warning: Failed to parse array content for $array_name\n"
              if $ENV{DEBUG};
            $skipped_symbols++;
            push @skipped_list, "$module_name:$array_name (parse error)";
        }
    }
}

sub parse_perl_array {
    my ($array_content) = @_;

    # Remove extra whitespace and normalize
    $array_content =~ s/^\s+|\s+$//g;

    # Handle nested arrays like [ [...], [...] ]
    if ( $array_content =~ /^\s*\[\s*(.*?)\s*\]\s*,\s*\[\s*(.*?)\s*\]\s*$/s ) {
        print STDERR "    Parsing nested array structure\n" if $ENV{DEBUG};
        my @result = ();

        # Split by '], [' to handle multiple sub-arrays
        my @sub_arrays = split /\]\s*,\s*\[/, $array_content;

        for my $sub_array (@sub_arrays) {

            # Clean up brackets that might remain
            $sub_array =~ s/^\s*\[?\s*|\s*\]?\s*$//g;

            my $parsed_sub = parse_array_elements($sub_array);
            push @result, $parsed_sub if $parsed_sub;
        }

        return \@result;
    }

    # Handle flat arrays
    else {
        return parse_array_elements($array_content);
    }
}

sub parse_array_elements {
    my ($elements_str) = @_;

    my @elements = ();

    # Split by comma, but be careful with nested structures
    my @raw_elements = split /,/, $elements_str;

    for my $element (@raw_elements) {
        $element =~ s/^\s+|\s+$//g;    # Trim whitespace
        next unless $element;          # Skip empty elements

        # Convert hex to decimal
        if ( $element =~ /^0x([0-9a-fA-F]+)$/ ) {
            push @elements, hex($1);
        }

        # Handle decimal numbers
        elsif ( $element =~ /^-?\d+(\.\d+)?$/ ) {
            push @elements, $element + 0;    # Force numeric context
        }

        # Handle quoted strings
        elsif ( $element =~ /^['"](.*)['"]$/ ) {
            push @elements, $1;
        }

        # Handle unquoted strings (barewords)
        elsif ( $element =~ /^[a-zA-Z_]\w*$/ ) {
            push @elements, $element;
        }
        else {
            print STDERR
              "    Warning: Unrecognized element format: '$element'\n"
              if $ENV{DEBUG};
        }
    }

    return \@elements;
}

#------------------------------------------------------------------------------
# Convert magic number patterns to raw byte arrays Input: Hash of file_type =>
# pattern Output: Hash of file_type => {raw_bytes => [byte, byte, ...]}
#
# CRITICAL: This MUST be done in Perl to avoid UTF-8 string escaping issues.
# ExifTool patterns contain raw bytes (0x00-0xFF) that become corrupted when
# passed through JSON->Rust string conversion. Instead of trying to parse
# patterns in Perl, we convert ALL patterns to raw byte arrays and let Rust
# handle the classification and optimization.
#
# _UNFORTUNATELY_: this means rust will see the raw byte arrays _including_ hex
# encoding and backslashes -- so rust has to un-do that nonsense on their side.
#------------------------------------------------------------------------------
sub convert_magic_number_patterns {
    my ($patterns_hash) = @_;
    my %converted;

    for my $file_type ( keys %$patterns_hash ) {
        my $pattern = $patterns_hash->{$file_type};

        # Convert pattern string to raw byte array
        my @raw_bytes;
        for my $i ( 0 .. length($pattern) - 1 ) {
            push @raw_bytes, ord( substr( $pattern, $i, 1 ) );
        }

        $converted{$file_type} = { raw_bytes => \@raw_bytes };
    }

    return \%converted;
}
