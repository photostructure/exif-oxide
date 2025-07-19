#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         tag_table_structure.pl
#
# Description:  Extract tag table structure from ExifTool modules for enum generation
#
# Usage:        perl tag_table_structure.pl <module_path> <table_name>
#
# Example:      perl tag_table_structure.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main
#
# Notes:        This script extracts the complete structure of manufacturer Main tables
#               to generate Rust enums with all metadata (tag_id, name, subdirectory, groups).
#               Works universally for all manufacturers (Canon, Nikon, Olympus, etc.)
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
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <table_name>\n" .
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/Canon.pm Main\n";
}

my ($module_path, $table_name) = @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

# Extract module name from path for display
my $module_display_name = $module_path;
$module_display_name =~ s{.*/}{}; # Remove path

# Extract manufacturer name from module
my $manufacturer = $module_display_name;
$manufacturer =~ s/\.pm$//;

print STDERR "Extracting tag table structure from $module_display_name table $table_name...\n";

# Load the module
my $module_name = load_module_from_file($module_path);
unless ($module_name) {
    die "Error: Failed to load module from $module_path\n";
}

# Get the tag table
my $table_symbol = "${module_name}::${table_name}";
my $table_ref = eval "\\%${table_symbol}";
if (!$table_ref || !%$table_ref) {
    die "Error: Table %$table_name not found in $module_display_name\n";
}

# Extract tag structure
my @tags = extract_tag_structure($table_ref, $manufacturer);

print STDERR "  Found " . scalar(@tags) . " tag definitions\n";

# Check for special processor types
my $has_process_binary_data = has_process_binary_data_tags($table_ref);
my $has_conditional_tags = has_conditional_tags($table_ref);

# Output JSON
my $output = {
    source => {
        module => $module_display_name,
        table => $table_name,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    manufacturer => $manufacturer,
    metadata => {
        total_tags => scalar(@tags),
        has_process_binary_data => $has_process_binary_data ? \1 : \0,
        has_conditional_tags => $has_conditional_tags ? \1 : \0,
    },
    tags => \@tags,
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract tag structure from a Main table
#------------------------------------------------------------------------------
sub extract_tag_structure {
    my ($table_ref, $manufacturer) = @_;
    my @tags;
    
    foreach my $tag_id (sort { $a <=> $b } keys %$table_ref) {
        # Skip special keys
        next if $tag_id =~ /^[A-Z]/;
        
        my $tag_info = $table_ref->{$tag_id};
        
        # Initialize tag data
        my $tag_data = {
            tag_id => sprintf("0x%04x", $tag_id),
            tag_id_decimal => $tag_id + 0,  # Force numeric
        };
        
        # Handle different tag info structures
        if (ref $tag_info eq 'ARRAY') {
            # Conditional tag definitions (array of variants)
            $tag_data->{conditional} = \1;
            $tag_data->{variants} = [];
            
            for my $variant (@$tag_info) {
                if (ref $variant eq 'HASH' && exists $variant->{Name}) {
                    my $variant_data = extract_single_tag($variant, $manufacturer);
                    if ($variant->{Condition}) {
                        $variant_data->{condition} = $variant->{Condition};
                    }
                    push @{$tag_data->{variants}}, $variant_data;
                }
            }
            
            # Use first variant's name as primary name
            if (@{$tag_data->{variants}}) {
                $tag_data->{name} = $tag_data->{variants}[0]->{name};
            }
        } elsif (ref $tag_info eq 'HASH') {
            # Standard tag definition
            my $single_tag = extract_single_tag($tag_info, $manufacturer);
            # Merge single tag data into main tag data
            @$tag_data{keys %$single_tag} = values %$single_tag;
        } else {
            # Simple value tag (rare in Main tables)
            next;
        }
        
        # Skip if we didn't get a name
        next unless exists $tag_data->{name};
        
        push @tags, $tag_data;
    }
    
    return @tags;
}

#------------------------------------------------------------------------------
# Extract data from a single tag definition
#------------------------------------------------------------------------------
sub extract_single_tag {
    my ($tag_info, $manufacturer) = @_;
    my $data = {};
    
    # Essential fields
    $data->{name} = $tag_info->{Name} if $tag_info->{Name};
    
    # Check for subdirectory
    if ($tag_info->{SubDirectory}) {
        $data->{has_subdirectory} = \1;
        
        # Extract subdirectory table name if available
        if (ref $tag_info->{SubDirectory} eq 'HASH') {
            if ($tag_info->{SubDirectory}->{TagTable}) {
                my $table = $tag_info->{SubDirectory}->{TagTable};
                $table =~ s/^Image::ExifTool:://;
                $table =~ s/^${manufacturer}:://;
                $data->{subdirectory_table} = $table;
            }
            
            # Check for ProcessBinaryData
            if ($tag_info->{SubDirectory}->{ProcessProc} && 
                $tag_info->{SubDirectory}->{ProcessProc} =~ /ProcessBinaryData/) {
                $data->{process_binary_data} = \1;
            }
        }
    }
    
    # Format information
    $data->{format} = $tag_info->{Format} || $tag_info->{Writable} if $tag_info->{Format} || $tag_info->{Writable};
    
    # Groups
    if ($tag_info->{Groups}) {
        if (ref $tag_info->{Groups} eq 'HASH') {
            my @groups;
            push @groups, $tag_info->{Groups}->{0} if $tag_info->{Groups}->{0};
            push @groups, $tag_info->{Groups}->{1} if $tag_info->{Groups}->{1};
            push @groups, $tag_info->{Groups}->{2} if $tag_info->{Groups}->{2};
            $data->{groups} = \@groups if @groups;
        }
    }
    
    # Flags
    $data->{writable} = \1 if $tag_info->{Writable};
    $data->{unknown} = \1 if $tag_info->{Unknown};
    $data->{binary} = \1 if $tag_info->{Binary};
    
    # Description
    $data->{description} = $tag_info->{Description} if $tag_info->{Description};
    
    return $data;
}

#------------------------------------------------------------------------------
# Check if table has ProcessBinaryData tags
#------------------------------------------------------------------------------
sub has_process_binary_data_tags {
    my $table_ref = shift;
    
    foreach my $tag_id (keys %$table_ref) {
        next if $tag_id =~ /^[A-Z]/;
        
        my $tag_info = $table_ref->{$tag_id};
        
        if (ref $tag_info eq 'HASH' && $tag_info->{SubDirectory}) {
            if (ref $tag_info->{SubDirectory} eq 'HASH' && 
                $tag_info->{SubDirectory}->{ProcessProc} &&
                $tag_info->{SubDirectory}->{ProcessProc} =~ /ProcessBinaryData/) {
                return 1;
            }
        }
    }
    
    return 0;
}

#------------------------------------------------------------------------------
# Check if table has conditional tags (array definitions)
#------------------------------------------------------------------------------
sub has_conditional_tags {
    my $table_ref = shift;
    
    foreach my $tag_id (keys %$table_ref) {
        next if $tag_id =~ /^[A-Z]/;
        
        my $tag_info = $table_ref->{$tag_id};
        
        if (ref $tag_info eq 'ARRAY') {
            return 1;
        }
    }
    
    return 0;
}