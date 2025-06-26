#!/usr/bin/env perl

=head1 NAME

extract_exiftool_table.pl - Extract complete tag tables from ExifTool modules

=head1 SYNOPSIS

    scripts/extract_exiftool_table.pl <ExifTool::Module>

    # Extract Canon tag tables
    scripts/extract_exiftool_table.pl Image::ExifTool::Canon

    # Extract main EXIF tag definitions
    scripts/extract_exiftool_table.pl Image::ExifTool::Exif

=head1 DESCRIPTION

This script extracts complete tag table information from ExifTool modules,
including all tag attributes, PrintConv mappings, and metadata. It handles
circular references and provides comprehensive statistics about the module.

Unlike extract_printconv.pl which focuses on PrintConv data, this script
extracts ALL tag information for analysis and debugging purposes.

=head1 OUTPUT FORMAT

The script outputs JSON with:
- metadata: Module info, extraction time, ExifTool version
- statistics: Comprehensive stats including longest PrintConv strings
- tables: All tag tables found in the module with complete definitions

=head1 FEATURES

- Handles circular references safely
- Converts non-serializable data (CODE refs, regexes)
- Calculates PrintConv statistics (string lengths, entry counts)
- Identifies all tag tables in a module

=head1 AUTHOR

Part of exif-oxide project - Rust implementation of ExifTool

=cut

use strict;
use warnings;
use JSON;
use Data::Dumper;
use File::Basename;
use lib 'third-party/exiftool/lib';

# Check command line arguments
if (@ARGV != 1) {
    die "Usage: $0 <ExifTool::Module>\n" .
        "Example: $0 Image::ExifTool::Canon\n" .
        "         $0 Image::ExifTool::Exif\n";
}

my $module_name = $ARGV[0];

# Load the module
eval "require $module_name";
if ($@) {
    die "Failed to load module $module_name: $@\n";
}

# Track seen references to prevent circular reference infinite loops
my %seen_refs;

# Function to convert code references and other non-serializable data
sub convert_value {
    my ($val, $depth) = @_;
    $depth ||= 0;
    
    # Prevent deep recursion
    return "DEEP_RECURSION" if $depth > 10;
    
    my $ref_type = ref($val);
    
    if (!$ref_type) {
        # Scalar value
        return $val;
    }
    
    # Check if we've seen this reference before (circular reference)
    my $ref_addr = "$val";
    if ($seen_refs{$ref_addr}) {
        return "CIRCULAR_REF";
    }
    $seen_refs{$ref_addr} = 1;
    
    my $result;
    
    if ($ref_type eq 'HASH') {
        # Recursively convert hash
        my %converted;
        foreach my $key (keys %$val) {
            $converted{$key} = convert_value($val->{$key}, $depth + 1);
        }
        $result = \%converted;
    } elsif ($ref_type eq 'ARRAY') {
        # Recursively convert array
        my @converted = map { convert_value($_, $depth + 1) } @$val;
        $result = \@converted;
    } elsif ($ref_type eq 'CODE') {
        # Convert code reference to string representation
        $result = "CODE_REF";
    } elsif ($ref_type eq 'Regexp') {
        # Convert regex to string
        $result = "$val";
    } else {
        # Other reference types
        $result = "$ref_type";
    }
    
    # Remove from seen refs when done
    delete $seen_refs{$ref_addr};
    
    return $result;
}

# Function to extract table data
sub extract_table {
    my ($table_ref, $table_name) = @_;
    my %table_data;
    
    # Handle both direct hash refs and package variables
    my %table;
    if (ref($table_ref) eq 'HASH') {
        %table = %$table_ref;
    } else {
        # It's a symbol table entry
        no strict 'refs';
        %table = %{"${module_name}::${table_name}"};
    }
    
    # Clear seen refs for each table
    %seen_refs = ();
    
    foreach my $key (keys %table) {
        # Skip special keys that aren't tag definitions
        next if $key =~ /^(GROUPS|NOTES|NAMESPACE|PRIORITY|WRITE_PROC|PROCESS_PROC|CHECK_PROC|VARS|TABLE_NAME|SHORT_NAME|AVOID|FIRST_ENTRY|TAG_PREFIX|PRINT_CONV|DID_TAG_ID)$/;
        
        my $val = $table{$key};
        my $tag_info = {};
        
        if (ref($val) eq 'HASH') {
            # This is a tag definition
            foreach my $attr (keys %$val) {
                if ($attr eq 'PrintConv') {
                    my $pc = $val->{$attr};
                    if (ref($pc) eq 'HASH') {
                        # PrintConv is a lookup table
                        $tag_info->{PrintConv} = convert_value($pc, 0);
                        
                        # Calculate stats for this PrintConv
                        my $max_length = 0;
                        my @long_values;
                        foreach my $k (keys %$pc) {
                            my $v = $pc->{$k};
                            if (!ref($v) && length($v) > $max_length) {
                                $max_length = length($v);
                            }
                            if (!ref($v) && length($v) > 20) {
                                push @long_values, { key => $k, value => $v, length => length($v) };
                            }
                        }
                        $tag_info->{PrintConvStats} = {
                            max_length => $max_length,
                            long_values => \@long_values,
                            total_entries => scalar(keys %$pc)
                        };
                    } elsif (ref($pc) eq 'ARRAY') {
                        # PrintConv is an array
                        $tag_info->{PrintConv} = convert_value($pc, 0);
                    } elsif (!ref($pc)) {
                        # PrintConv is a string (code)
                        $tag_info->{PrintConv} = $pc;
                        $tag_info->{PrintConvType} = 'code_string';
                    } else {
                        $tag_info->{PrintConv} = convert_value($pc, 0);
                    }
                } elsif ($attr eq 'PrintConvInv') {
                    # Skip PrintConvInv as it's usually a code ref
                    $tag_info->{$attr} = ref($val->{$attr}) || 'scalar';
                } elsif ($attr eq 'SubDirectory') {
                    # Handle SubDirectory specially
                    if (ref($val->{$attr}) eq 'HASH') {
                        $tag_info->{$attr} = {};
                        foreach my $k (keys %{$val->{$attr}}) {
                            $tag_info->{$attr}->{$k} = convert_value($val->{$attr}->{$k}, 0);
                        }
                    } else {
                        $tag_info->{$attr} = convert_value($val->{$attr}, 0);
                    }
                } else {
                    $tag_info->{$attr} = convert_value($val->{$attr}, 0);
                }
            }
        } else {
            # Simple tag definition
            $tag_info = { Value => convert_value($val, 0) };
        }
        
        $table_data{$key} = $tag_info;
    }
    
    return \%table_data;
}

# Find all tables in the module
my %all_tables;
{
    no strict 'refs';
    my $symbol_table = \%{"${module_name}::"};
    
    foreach my $symbol (keys %$symbol_table) {
        next if $symbol =~ /^[A-Z_]+$/; # Skip constants
        next if $symbol =~ /^(BEGIN|END|AUTOLOAD|DESTROY|import)$/; # Skip special subs
        
        my $glob = $symbol_table->{$symbol};
        if (*{$glob}{HASH}) {
            # This is a hash, might be a table
            my $hash_ref = \%{"${module_name}::${symbol}"};
            if (exists $hash_ref->{GROUPS} || exists $hash_ref->{NOTES} || 
                exists $hash_ref->{0} || exists $hash_ref->{1} ||
                (keys %$hash_ref && $symbol =~ /Table$/)) {
                # This looks like a tag table
                $all_tables{$symbol} = extract_table($hash_ref, $symbol);
            }
        }
    }
}

# Collect statistics
my %stats = (
    module => $module_name,
    tables => scalar(keys %all_tables),
    total_tags => 0,
    tags_with_printconv => 0,
    printconv_types => {
        hash => 0,
        array => 0,
        code_string => 0,
        other => 0
    },
    longest_printconv_strings => []
);

# Analyze all tables for statistics
foreach my $table_name (keys %all_tables) {
    my $table = $all_tables{$table_name};
    foreach my $tag (keys %$table) {
        $stats{total_tags}++;
        
        if (exists $table->{$tag}->{PrintConv}) {
            $stats{tags_with_printconv}++;
            
            my $pc = $table->{$tag}->{PrintConv};
            if (ref($pc) eq 'HASH') {
                $stats{printconv_types}->{hash}++;
                
                # Look for long strings
                if (exists $table->{$tag}->{PrintConvStats} && 
                    @{$table->{$tag}->{PrintConvStats}->{long_values}}) {
                    foreach my $long_val (@{$table->{$tag}->{PrintConvStats}->{long_values}}) {
                        push @{$stats{longest_printconv_strings}}, {
                            table => $table_name,
                            tag => $tag,
                            tag_name => $table->{$tag}->{Name} || $tag,
                            %$long_val
                        };
                    }
                }
            } elsif (ref($pc) eq 'ARRAY') {
                $stats{printconv_types}->{array}++;
            } elsif ($table->{$tag}->{PrintConvType} && $table->{$tag}->{PrintConvType} eq 'code_string') {
                $stats{printconv_types}->{code_string}++;
            } else {
                $stats{printconv_types}->{other}++;
            }
        }
    }
}

# Sort longest strings by length
@{$stats{longest_printconv_strings}} = 
    sort { $b->{length} <=> $a->{length} } 
    @{$stats{longest_printconv_strings}};

# Limit to top 50 longest strings
if (@{$stats{longest_printconv_strings}} > 50) {
    @{$stats{longest_printconv_strings}} = @{$stats{longest_printconv_strings}}[0..49];
}

# Create output structure
my $output = {
    metadata => {
        module => $module_name,
        extraction_time => scalar(localtime),
        exiftool_version => $Image::ExifTool::VERSION || 'unknown',
    },
    statistics => \%stats,
    tables => \%all_tables
};

# Output as JSON
my $json = JSON->new->pretty->canonical->allow_nonref;
print $json->encode($output);