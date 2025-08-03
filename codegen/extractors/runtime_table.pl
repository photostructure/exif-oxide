#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         runtime_table.pl
#
# Description:  Extract ProcessBinaryData tables for runtime HashMap generation
#
# Usage:        perl runtime_table.pl <module_path> <table_name>
#
# Example:      perl runtime_table.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm CameraSettings
#
# Notes:        This script extracts ProcessBinaryData tables with focus on:
#               - Model-conditional processing (Condition fields)
#               - Complex PrintConv structures requiring runtime evaluation
#               - Format specifications with variable arrays
#               - DataMember dependencies for runtime table construction
#               Designed for generating runtime HashMap creation functions
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
  load_module_from_file
  format_json_output
);

# Check arguments
if ( @ARGV < 2 ) {
    die "Usage: $0 <module_path> <table_name>\n"
      . "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm CameraSettings\n";
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
  "Extracting runtime table $table_name from $module_display_name...\n";

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

# Verify this is a ProcessBinaryData table
unless ( has_process_binary_data($table_ref) ) {
    die "Error: Table %$table_name is not a ProcessBinaryData table\n";
}

# Extract runtime table structure
my $runtime_table_data = extract_runtime_table( $table_ref, $table_name );

my $tag_count = scalar keys %{ $runtime_table_data->{tag_definitions} };
print STDERR "  Found $tag_count runtime tag definitions\n";
print STDERR "  Model conditions: "
  . ( $runtime_table_data->{metadata}->{has_model_conditions} ? "Yes" : "No" )
  . "\n";
print STDERR "  Complex PrintConv: "
  . ( $runtime_table_data->{metadata}->{has_complex_printconv} ? "Yes" : "No" )
  . "\n";

# Output JSON (sanitize data before serializing)
my $sanitized_data = sanitize_for_json($runtime_table_data);

my $output = {
    source => {
        module       => $module_display_name,
        table        => $table_name,
        extracted_at => scalar( gmtime() ) . " GMT",
    },
    extracted_at      => scalar( gmtime() ) . " GMT",
    extraction_config => "runtime_table",
    tables            => {
        $table_name => $sanitized_data
    },
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Check if table has ProcessBinaryData
#------------------------------------------------------------------------------
sub has_process_binary_data {
    my $table_ref = shift;

    return 0 unless exists $table_ref->{PROCESS_PROC};

    my $process_proc = $table_ref->{PROCESS_PROC};
    if ( ref $process_proc eq 'CODE' ) {

        # For function references, we need to check the symbol table
        return 1;    # Assume it's ProcessBinaryData if it's a code ref
    }
    elsif ( $process_proc && $process_proc =~ /ProcessBinaryData/ ) {
        return 1;
    }

    return 0;
}

#------------------------------------------------------------------------------
# Extract runtime table structure
#------------------------------------------------------------------------------
sub extract_runtime_table {
    my ( $table_ref, $table_name ) = @_;

    # Extract table structure and metadata
    my $table_structure = extract_table_structure($table_ref);
    my $tag_definitions = extract_runtime_tag_definitions($table_ref);
    my $metadata = analyze_runtime_requirements( $table_ref, $tag_definitions );

    return {
        metadata        => $metadata,
        table_structure => $table_structure,
        tag_definitions => $tag_definitions,
    };
}

#------------------------------------------------------------------------------
# Extract ProcessBinaryData table structure
#------------------------------------------------------------------------------
sub extract_table_structure {
    my $table_ref = shift;

    my $structure = {};

    # Extract header attributes
    $structure->{format}      = $table_ref->{FORMAT} if $table_ref->{FORMAT};
    $structure->{first_entry} = $table_ref->{FIRST_ENTRY}
      if defined $table_ref->{FIRST_ENTRY};
    $structure->{writable} = $table_ref->{WRITABLE} ? 1 : 0;

    # Extract GROUPS structure (avoid CODE references)
    if ( $table_ref->{GROUPS} && ref $table_ref->{GROUPS} eq 'HASH' ) {
        $structure->{groups} = $table_ref->{GROUPS};
    }
    elsif ( $table_ref->{GROUPS} ) {

        # Handle non-hash GROUPS (like code references)
        $structure->{groups} = { note => "NON_HASH_GROUPS_PLACEHOLDER" };
    }

    # Extract DATAMEMBER if present
    if ( $table_ref->{DATAMEMBER} ) {
        $structure->{data_member} = $table_ref->{DATAMEMBER};
    }

    return $structure;
}

#------------------------------------------------------------------------------
# Extract runtime tag definitions
#------------------------------------------------------------------------------
sub extract_runtime_tag_definitions {
    my $table_ref = shift;
    my %tag_definitions;

    # Process only numeric keys (offsets) and some special cases
    my @keys = grep { /^(\d+(\.\d+)?|0x[0-9a-fA-F]+)$/ } keys %$table_ref;

    foreach my $offset_key (@keys) {
        my $tag_info = $table_ref->{$offset_key};

        # Skip if no tag info
        next unless $tag_info;

        my $tag_def = extract_tag_definition( $offset_key, $tag_info );
        next unless $tag_def && $tag_def->{name};

        $tag_definitions{$offset_key} = $tag_def;
    }

    return \%tag_definitions;
}

#------------------------------------------------------------------------------
# Extract individual tag definition
#------------------------------------------------------------------------------
sub extract_tag_definition {
    my ( $offset, $tag_info ) = @_;

    my $tag_def = { offset => $offset, };

    # Handle different tag info structures
    if ( ref $tag_info eq 'HASH' ) {

        # Complex tag definition
        $tag_def->{name} = $tag_info->{Name} if $tag_info->{Name};

        # Extract format specification
        if ( $tag_info->{Format} ) {
            $tag_def->{format} = extract_format_spec( $tag_info->{Format} );
        }

        # Extract condition specification
        if ( $tag_info->{Condition} ) {
            $tag_def->{condition} =
              extract_condition_spec( $tag_info->{Condition} );
        }

        # Extract PrintConv specification
        if ( $tag_info->{PrintConv} ) {
            $tag_def->{print_conv} =
              extract_print_conv_spec( $tag_info->{PrintConv} );
        }

        # Extract ValueConv specification
        if ( $tag_info->{ValueConv} ) {
            $tag_def->{value_conv} =
              extract_value_conv_spec( $tag_info->{ValueConv} );
        }

        # Extract groups (handle CODE references)
        if ( $tag_info->{Groups} && ref $tag_info->{Groups} eq 'HASH' ) {
            $tag_def->{groups} = $tag_info->{Groups};
        }
        elsif ( $tag_info->{Groups} && !ref $tag_info->{Groups} ) {

            # Simple scalar groups
            $tag_def->{groups} = { 1 => $tag_info->{Groups} };
        }

        # Extract notes (handle CODE references)
        if ( $tag_info->{Notes} && !ref $tag_info->{Notes} ) {
            $tag_def->{notes} = $tag_info->{Notes};
        }
        elsif ( $tag_info->{Notes} ) {
            $tag_def->{notes} = "COMPLEX_NOTES_PLACEHOLDER";
        }

    }
    else {
        # Simple tag name
        $tag_def->{name} = $tag_info;
    }

    return $tag_def;
}

#------------------------------------------------------------------------------
# Extract format specification
#------------------------------------------------------------------------------
sub extract_format_spec {
    my $format = shift;
    return undef unless $format;

    my $spec = {};

    # Check for array specifications like "int16u[4]" or "string[$val{2}]"
    if ( $format =~ /^(\w+)(?:\[([^\]]+)\])?$/ ) {
        $spec->{base_type} = $1;

        if ($2) {
            $spec->{array_size}  = $2;
            $spec->{is_variable} = ( $2 =~ /\$/ ) ? 1 : 0;
        }
        else {
            $spec->{is_variable} = 0;
        }
    }
    else {
        $spec->{base_type}   = $format;
        $spec->{is_variable} = 0;
    }

    return $spec;
}

#------------------------------------------------------------------------------
# Extract condition specification
#------------------------------------------------------------------------------
sub extract_condition_spec {
    my $condition = shift;
    return undef unless $condition;

    my $spec = { expression => $condition, };

    # Analyze condition type
    if ( $condition =~ /\$\$self\{Model\}\s*=~/ ) {
        $spec->{condition_type} = 'model_regex';
    }
    elsif ( $condition =~ /\$\$self\{Model\}\s*(eq|ne)/ ) {
        $spec->{condition_type} = 'model_exact';
    }
    elsif ( $condition =~ /\$val\s*[<>=!]/ ) {
        $spec->{condition_type} = 'value_comparison';
    }
    else {
        $spec->{condition_type} = 'expression';
    }

    return $spec;
}

#------------------------------------------------------------------------------
# Extract PrintConv specification
#------------------------------------------------------------------------------
sub extract_print_conv_spec {
    my $print_conv = shift;
    return undef unless $print_conv;

    my $spec = {};

    if ( ref $print_conv eq 'HASH' ) {

        # Simple hash lookup table
        $spec->{conversion_type} = 'simple_hash';
        $spec->{data}            = $print_conv;
    }
    elsif ( ref $print_conv eq 'CODE' ) {

   # Function reference - we can't serialize this, so record it as a placeholder
        $spec->{conversion_type} = 'function_ref';
        $spec->{data}            = 'CODE_REFERENCE_PLACEHOLDER';
    }
    elsif ( $print_conv && $print_conv =~ /^q\{/ ) {

        # Perl expression block
        $spec->{conversion_type} = 'perl_expression';
        $spec->{data}            = $print_conv;
    }
    elsif ( $print_conv =~ /[&\$]/ ) {

        # Complex expression or bitwise operation
        $spec->{conversion_type} = 'bitwise_operation';
        $spec->{data}            = $print_conv;
    }
    else {
        # Simple string
        $spec->{conversion_type} = 'simple_hash';
        $spec->{data}            = { '0' => $print_conv };
    }

    return $spec;
}

#------------------------------------------------------------------------------
# Extract ValueConv specification
#------------------------------------------------------------------------------
sub extract_value_conv_spec {
    my $value_conv = shift;
    return undef unless $value_conv;

    my $spec = { expression => $value_conv, };

    # Analyze conversion type
    if ( $value_conv =~ /exp\s*\(.*log/ ) {
        $spec->{conversion_type} = 'mathematical';
    }
    elsif ( $value_conv =~ /\/.*\|\|/ ) {
        $spec->{conversion_type} = 'division';
    }
    elsif ( $value_conv =~ /Image::ExifTool::\w+::\w+/ ) {
        $spec->{conversion_type} = 'function_call';
    }
    elsif ( $value_conv =~ /\?\s*undef\s*:/ ) {
        $spec->{conversion_type} = 'conditional';
    }
    else {
        $spec->{conversion_type} = 'mathematical';
    }

    return $spec;
}

#------------------------------------------------------------------------------
# Analyze runtime requirements
#------------------------------------------------------------------------------
sub analyze_runtime_requirements {
    my ( $table_ref, $tag_definitions ) = @_;

    my $metadata = {
        function_name   => 'create_table',        # Will be overridden by config
        table_name      => '%unknown',            # Will be overridden by config
        processing_mode => 'runtime_conditions',
        format_handling => 'dynamic',
        has_model_conditions  => 0,
        has_data_member_deps  => 0,
        has_complex_printconv => 0,
        description           => 'Runtime-generated ProcessBinaryData table',
    };

    # Check for model conditions
    foreach my $tag_def ( values %$tag_definitions ) {
        if (   $tag_def->{condition}
            && $tag_def->{condition}->{condition_type} =~ /model/ )
        {
            $metadata->{has_model_conditions} = 1;
            last;
        }
    }

    # Check for DataMember dependencies
    if ( $table_ref->{DATAMEMBER} ) {
        $metadata->{has_data_member_deps} = 1;
    }

    # Check for complex PrintConv
    foreach my $tag_def ( values %$tag_definitions ) {
        if (   $tag_def->{print_conv}
            && $tag_def->{print_conv}->{conversion_type} ne 'simple_hash' )
        {
            $metadata->{has_complex_printconv} = 1;
            last;
        }
    }

    return $metadata;
}

#------------------------------------------------------------------------------
# Sanitize data structure for JSON serialization
#------------------------------------------------------------------------------
sub sanitize_for_json {
    my $data = shift;

    if ( ref $data eq 'HASH' ) {
        my $sanitized = {};
        for my $key ( keys %$data ) {
            $sanitized->{$key} = sanitize_for_json( $data->{$key} );
        }
        return $sanitized;
    }
    elsif ( ref $data eq 'ARRAY' ) {
        my @sanitized = map { sanitize_for_json($_) } @$data;
        return \@sanitized;
    }
    elsif ( ref $data eq 'CODE' ) {
        return 'CODE_REFERENCE_SANITIZED';
    }
    elsif ( ref $data ) {

        # Other references (SCALAR, REF, etc.)
        return 'REFERENCE_SANITIZED';
    }
    else {
        # Simple scalar value
        return $data;
    }
}
