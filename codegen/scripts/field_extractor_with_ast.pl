#!/usr/bin/env perl

# P08: PPI AST Foundation - Enhanced field extractor with AST support
#
# This script extends the existing field_extractor.pl to include PPI AST parsing
# for ExifTool expressions, enabling AST-based Rust code generation.
#
# See docs/todo/P08-ppi-ast-foundation.md for complete details.

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use JSON::XS;
use Sub::Util    qw(subname);
use Scalar::Util qw(blessed reftype refaddr);

# P08: Add PPI support for AST parsing
use PPI;

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

# Global counters and debugging
my $total_symbols     = 0;
my $extracted_symbols = 0;
my $skipped_symbols   = 0;
my @skipped_list      = ();

# P08: AST processing counters
my $ast_processed_expressions = 0;
my $ast_failed_expressions    = 0;

# JSON serializer - let JSON::XS handle complex structures automatically
my $json =
  JSON::XS->new->canonical(1)
  ->allow_blessed(1)
  ->convert_blessed(1)
  ->allow_nonref(1);

if ( @ARGV < 1 ) {
    die "Usage: $0 <module_path> [field1] [field2] ...\n"
      . "  Extract all hash symbols from ExifTool module with PPI AST analysis\n"
      . "  Examples:\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool.pm\n"
      . "    $0 third-party/exiftool/lib/Image/ExifTool/Canon.pm\n";
}

my $module_path   = shift @ARGV;
my @target_fields = @ARGV;         # Optional list of specific fields to extract

# Extract module name from path
my $module_name = basename($module_path);
$module_name =~ s/\.pm$//;

# CAREFUL! The rust code **actually looks for this magic string**!
print STDERR "Field extraction with AST starting for $module_name:\n";

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

# Extract symbols from symbol table (with AST enhancement)
extract_symbols_with_ast( $package_name, $module_name, \@target_fields,
    $has_composite_tags );

# Extract lexical arrays from source code
extract_lexical_arrays( $module_path, $module_name, \@target_fields );

# Print summary including AST stats
print STDERR "Field extraction with AST complete for $module_name:\n";
print STDERR "  Total symbols examined: $total_symbols\n";
print STDERR "  Successfully extracted: $extracted_symbols\n";
print STDERR "  Skipped (non-serializable): $skipped_symbols\n";
print STDERR "  AST expressions processed: $ast_processed_expressions\n";
print STDERR "  AST parsing failures: $ast_failed_expressions\n";

# Print debug info about skipped symbols if requested
if ( $ENV{DEBUG} && @skipped_list ) {
    print STDERR "\nSkipped symbols:\n";
    for my $skipped (@skipped_list) {
        print STDERR "  - $skipped\n";
    }
}

# P08: Enhanced extract_symbols with AST support
sub extract_symbols_with_ast {
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

        print STDERR "  Processing symbol with AST: $symbol_name\n"
          if $ENV{DEBUG};

        my $glob = $symbol_table->{$symbol_name};

        # Try to extract hash symbols (most important for ExifTool)
        if ( my $hash_ref = *$glob{HASH} ) {
            if (%$hash_ref) {    # Skip empty hashes
                my $hash_size = scalar( keys %$hash_ref );
                print STDERR "    Hash found with $hash_size keys\n"
                  if $ENV{DEBUG};
                extract_hash_symbol_with_ast(
                    $symbol_name, $hash_ref,
                    $module_name, $has_composite_tags
                );
                print STDERR
                  "    Hash extraction with AST completed for $symbol_name\n"
                  if $ENV{DEBUG};
            }
        }

        # Also try to extract array symbols
        elsif ( my $array_ref = *$glob{ARRAY} ) {
            if (@$array_ref) {    # Skip empty arrays
                my $array_size = scalar(@$array_ref);
                print STDERR "    Array found with $array_size elements\n"
                  if $ENV{DEBUG};
                extract_array_symbol_with_ast( $symbol_name, $array_ref,
                    $module_name );
                print STDERR
                  "    Array extraction with AST completed for $symbol_name\n"
                  if $ENV{DEBUG};
            }
        }
        else {
            print STDERR "    No hash or array found for $symbol_name\n"
              if $ENV{DEBUG};
        }
    }
}

# P08: Enhanced array extraction with potential expression analysis
sub extract_array_symbol_with_ast {
    my ( $symbol_name, $array_ref, $module_name ) = @_;

    print STDERR "    Starting array extraction with AST for: $symbol_name\n"
      if $ENV{DEBUG};

 # Filter out function references if any (though arrays usually don't have them)
    my $filtered_data = filter_code_refs($array_ref);

 # P08: Check if array contains expressions that could benefit from AST analysis
    my $expression_analysis = analyze_array_for_expressions($filtered_data);

    # Package the data with metadata including AST analysis
    my $extracted = {
        name     => $symbol_name,
        module   => $module_name,
        type     => 'array',
        data     => $filtered_data,
        metadata => {
            size               => scalar(@$filtered_data),
            is_composite_table => 0,    # Arrays are never composite tables
            ast_analysis       => $expression_analysis,    # P08: AST metadata
        }
    };

    # Output the extracted array as JSON
    eval {
        my $json_data = encode_json($extracted);
        print "$json_data\n";
        print STDERR "    Successfully extracted array with AST: $symbol_name\n"
          if $ENV{DEBUG};
    };
    if ($@) {
        print STDERR "    Failed to encode array $symbol_name: $@\n";
    }
}

# P08: Enhanced hash extraction with AST support
sub extract_hash_symbol_with_ast {
    my ( $symbol_name, $hash_ref, $module_name, $has_composite_tags ) = @_;

    print STDERR "    Starting extraction with AST for: $symbol_name\n"
      if $ENV{DEBUG};

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

    # P08: Analyze expressions in the hash for AST opportunities
    print STDERR "    Analyzing expressions with PPI AST...\n" if $ENV{DEBUG};
    my $ast_analysis =
      analyze_expressions_with_ppi( $processed_data, $symbol_name,
        $module_name );
    print STDERR "    PPI AST analysis completed\n" if $ENV{DEBUG};

    my $symbol_data = {
        type     => 'hash',
        name     => $symbol_name,
        data     => $processed_data,
        module   => $module_name,
        metadata => {
            size               => $size,
            is_composite_table => $is_composite_table ? 1 : 0,
            ast_analysis       => $ast_analysis,    # P08: Include AST analysis
        }
    };

    eval {
        print $json->encode($symbol_data) . "\n";
        $extracted_symbols++;
        print STDERR "  Extracted with AST: $symbol_name ("
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

# P08: Analyze array elements for expressions
sub analyze_array_for_expressions {
    my ($array_data) = @_;

    my $analysis = {
        expressions_found => 0,
        ast_parseable     => 0,
        ppi_ast_data      => {},
        expression_types  => {
            condition  => 0,
            value_conv => 0,
            print_conv => 0,
            raw_conv   => 0,
        }
    };

    # Look for potential expressions in array elements
    for my $i ( 0 .. $#{$array_data} ) {
        my $element = $array_data->[$i];
        
        if ( ref $element eq '' ) {    # String element
            if ( is_potential_expression($element) ) {
                analyze_single_expression( $element, "[$i]", $analysis );
            }
        }
        elsif ( ref $element eq 'HASH' ) {    # Hash element - check for tag expressions
            analyze_tag_expressions( $element, "[$i]", $analysis );
        }
    }

    return $analysis;
}

# P08: Main PPI AST analysis function
sub analyze_expressions_with_ppi {
    my ( $hash_data, $symbol_name, $module_name ) = @_;

    my $ast_analysis = {
        expressions_found => 0,
        ast_parseable     => 0,
        ppi_ast_data      => {},
        expression_types  => {
            condition  => 0,
            value_conv => 0,
            print_conv => 0,
            raw_conv   => 0,
        }
    };

    # Examine each key-value pair for expressions
    for my $key ( keys %$hash_data ) {
        my $value = $hash_data->{$key};

        # Handle different value types
        if ( ref $value eq 'HASH' ) {

            # Nested hash - look for PrintConv, ValueConv, Condition
            analyze_tag_expressions( $value, $key, $ast_analysis );
        }
        elsif ( ref $value eq 'ARRAY' ) {

            # Array of objects - examine each element for expressions
            for my $i ( 0 .. $#{$value} ) {
                my $array_element = $value->[$i];
                if ( ref $array_element eq 'HASH' ) {
                    # Each array element might be a tag definition with Condition
                    analyze_tag_expressions( $array_element, "$key\[$i\]", $ast_analysis );
                }
            }
        }
        elsif ( ref $value eq '' ) {

            # String value - check if it's an expression
            if ( is_potential_expression($value) ) {
                analyze_single_expression( $value, "$symbol_name.$key",
                    $ast_analysis );
            }
        }
    }

    return $ast_analysis;
}

# P08: Analyze tag-specific expressions (PrintConv, ValueConv, Condition)
sub analyze_tag_expressions {
    my ( $tag_hash, $tag_id, $analysis ) = @_;

    # Note: *Inv fields (PrintConvInv, ValueConvInv, RawConvInv) are excluded
    # as they're for write support. Add them back when implementing P62-MILESTONE-21-Basic-Write-Support.md
    for my $expr_type (qw(PrintConv ValueConv RawConv Condition)) {
        if ( exists $tag_hash->{$expr_type} ) {
            my $expr_value = $tag_hash->{$expr_type};

            if ( ref $expr_value eq '' ) {    # String expression
                $analysis->{expressions_found}++;
                
                # Map ExifTool expression type to our normalized key
                my $expr_key = normalize_expr_type($expr_type);
                $analysis->{expression_types}->{$expr_key}++;

                # Try to parse with PPI
                if (
                    try_parse_expression_with_ppi(
                        $expr_value, "$tag_id.$expr_type", $analysis
                    )
                  )
                {
                    $analysis->{ast_parseable}++;
                }
            }
            elsif ( ref $expr_value eq 'HASH' ) {

         # Hash-based PrintConv (lookup table) - might contain OTHER expressions
                if ( exists $expr_value->{OTHER} ) {

                    # OTHER function reference - worth noting
                    $analysis->{ppi_ast_data}->{"$tag_id.$expr_type.OTHER"} = {
                        type      => 'function_ref',
                        parseable => 0,
                        note      =>
                          'Function reference - requires manual implementation'
                    };
                }
            }
        }
    }
}

# P08: Analyze a single expression string
sub analyze_single_expression {
    my ( $expr_string, $context, $analysis ) = @_;

    $analysis->{expressions_found}++;

    if ( try_parse_expression_with_ppi( $expr_string, $context, $analysis ) ) {
        $analysis->{ast_parseable}++;
    }
}

# P08: Try to parse expression with PPI and record results
sub try_parse_expression_with_ppi {
    my ( $expr_string, $context, $analysis ) = @_;

    # Skip very long expressions (likely multi-line code blocks)
    return 0 if length($expr_string) > 500;

    # Skip expressions with obviously unsupported constructs
    return 0 if $expr_string =~ /\b(eval|require|use)\b/;
    return 0 if $expr_string =~ /\$self->/;              # Method calls on $self

    eval {
        # Try to parse with PPI
        my $document = PPI::Document->new( \$expr_string );

        if ($document) {
            $ast_processed_expressions++;

            # Analyze the parsed AST
            my $ast_info = analyze_ppi_document( $document, $expr_string );

            # Store AST data
            $analysis->{ppi_ast_data}->{$context} = $ast_info;

            print STDERR "      AST parsed successfully: $context\n"
              if $ENV{DEBUG};
            return 1;
        }
        else {
            $ast_failed_expressions++;
            print STDERR "      AST parsing failed: $context\n" if $ENV{DEBUG};
            return 0;
        }
    };

    if ($@) {
        $ast_failed_expressions++;
        print STDERR "      AST parsing error for $context: $@\n"
          if $ENV{DEBUG};
        return 0;
    }
}

# P08: Analyze PPI document structure
sub analyze_ppi_document {
    my ( $document, $original_expr ) = @_;

    my $ast_info = {
        original      => $original_expr,
        parseable     => 1,
        complexity    => 'unknown',
        node_types    => [],
        has_variables => 0,
        has_self_refs => 0,
        has_functions => 0,
        has_operators => 0,
    };

    # Find different types of nodes
    my $statements = $document->find('PPI::Statement') || [];
    my $tokens     = $document->find('PPI::Token')     || [];

    # Analyze tokens for complexity
    for my $token (@$tokens) {
        my $token_type = ref($token);
        push @{ $ast_info->{node_types} }, $token_type;

        if ( $token_type eq 'PPI::Token::Symbol' ) {
            $ast_info->{has_variables} = 1;

            # Check for $$self references
            if ( $token->symbol =~ /^\$\$self/ ) {
                $ast_info->{has_self_refs} = 1;
            }
        }
        elsif ( $token_type eq 'PPI::Token::Word' ) {

            # Check for function calls
            my $word = $token->literal;
            if ( $word =~ /^(sprintf|printf|int|exp|log|abs|sqrt)$/ ) {
                $ast_info->{has_functions} = 1;
            }
        }
        elsif ( $token_type eq 'PPI::Token::Operator' ) {
            $ast_info->{has_operators} = 1;
        }
    }

# Note: complexity classification removed - using granular flags for smarter routing

    return $ast_info;
}

# P08: Removed complexity and feasibility assessment functions
# Now using granular boolean flags for intelligent routing decisions in Rust

# P08: Normalize ExifTool expression type names to our schema
sub normalize_expr_type {
    my ($expr_type) = @_;
    
    my %mapping = (
        'Condition' => 'condition',
        'PrintConv' => 'print_conv',
        'ValueConv' => 'value_conv',
        'RawConv'   => 'raw_conv',
    );
    
    return $mapping{$expr_type} // lc($expr_type);
}

# P08: Check if string looks like a Perl expression
sub is_potential_expression {
    my ($string) = @_;

    # Skip very short strings
    return 0 if length($string) < 3;

    # Look for Perl expression indicators
    return 1 if $string =~ /\$\w+/;               # Variables
    return 1 if $string =~ /\?\s*.*\s*:/;         # Ternary operator
    return 1 if $string =~ /sprintf\s*\(/;        # sprintf calls
    return 1 if $string =~ /[+\-*\/]\s*\$\w+/;    # Arithmetic with variables
    return 1 if $string =~ /\$\w+\s*[+\-*\/]/;    # Variables with arithmetic

    return 0;
}

# Re-use existing helper functions from field_extractor.pl
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
                # Replace Table references with string representation to break
                # circularity These are metadata pointers in ExifTool, not
                # structural data
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

# Re-use existing extract_lexical_arrays function
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

# Re-use existing parsing functions...
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

# Re-use existing magic number conversion
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
