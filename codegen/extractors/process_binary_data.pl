#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         process_binary_data.pl
#
# Description:  Extract ProcessBinaryData table definitions from ExifTool modules
#
# Usage:        perl process_binary_data.pl <module_path> <table_name>
#
# Example:      perl process_binary_data.pl ../third-party/exiftool/lib/Image/ExifTool/FujiFilm.pm FFMV
#
# Notes:        This script extracts ProcessBinaryData table structure including:
#               - Table header attributes (FORMAT, FIRST_ENTRY, GROUPS, etc.)
#               - Tag definitions with offsets, names, formats, conditions
#               - Complex format specifications and conditional logic
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
        "Example: $0 ../third-party/exiftool/lib/Image/ExifTool/FujiFilm.pm FFMV\n";
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

print STDERR "Extracting ProcessBinaryData table $table_name from $module_display_name...\n";

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

# Verify this is a ProcessBinaryData table
unless (has_process_binary_data($table_ref)) {
    die "Error: Table %$table_name is not a ProcessBinaryData table\n";
}

# Extract table structure
my $table_data = extract_process_binary_data_table($table_ref, $manufacturer, $table_name);

print STDERR "  Found " . scalar(@{$table_data->{tags}}) . " tag definitions\n";
print STDERR "  Table format: " . ($table_data->{header}->{format} || 'default') . "\n";

# Output JSON
my $output = {
    source => {
        module => $module_display_name,
        table => $table_name,
        extracted_at => scalar(gmtime()) . " GMT",
    },
    manufacturer => $manufacturer,
    table_data => $table_data,
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Check if table has ProcessBinaryData
#------------------------------------------------------------------------------
sub has_process_binary_data {
    my $table_ref = shift;
    
    return 0 unless exists $table_ref->{PROCESS_PROC};
    
    my $process_proc = $table_ref->{PROCESS_PROC};
    if (ref $process_proc eq 'CODE') {
        # For function references, we need to check the symbol table
        # since we can't easily deparse code refs in all Perl versions
        return 1;  # Assume it's ProcessBinaryData if it's a code ref
    } elsif ($process_proc && $process_proc =~ /ProcessBinaryData/) {
        return 1;
    }
    
    return 0;
}

#------------------------------------------------------------------------------
# Extract ProcessBinaryData table structure
#------------------------------------------------------------------------------
sub extract_process_binary_data_table {
    my ($table_ref, $manufacturer, $table_name) = @_;
    
    my $table_data = {
        table_name => $table_name,
        header => extract_table_header($table_ref),
        tags => extract_binary_data_tags($table_ref),
    };
    
    return $table_data;
}

#------------------------------------------------------------------------------
# Extract table header attributes
#------------------------------------------------------------------------------
sub extract_table_header {
    my $table_ref = shift;
    my $header = {};
    
    # Extract special keys that affect the whole table
    $header->{format} = $table_ref->{FORMAT} if $table_ref->{FORMAT};
    $header->{first_entry} = $table_ref->{FIRST_ENTRY} if defined $table_ref->{FIRST_ENTRY};
    
    # Groups - convert hash to simple array
    if ($table_ref->{GROUPS}) {
        my @groups;
        if (ref $table_ref->{GROUPS} eq 'HASH') {
            for my $key (sort keys %{$table_ref->{GROUPS}}) {
                push @groups, $table_ref->{GROUPS}->{$key};
            }
        }
        $header->{groups} = \@groups if @groups;
    }
    
    # Other attributes
    $header->{writable} = 1 if $table_ref->{WRITABLE};
    $header->{notes} = $table_ref->{NOTES} if $table_ref->{NOTES};
    
    return $header;
}

#------------------------------------------------------------------------------
# Extract binary data tag definitions
#------------------------------------------------------------------------------
sub extract_binary_data_tags {
    my $table_ref = shift;
    my @tags;
    
    # Process only numeric keys (offsets) and sort them
    my @offsets = sort { $a <=> $b } grep { /^\d+(\.\d+)?$/ } keys %$table_ref;
    
    foreach my $offset_key (@offsets) {
        my $tag_info = $table_ref->{$offset_key};
        
        # Convert offset to hex for display
        my $offset_hex = sprintf("0x%02x", $offset_key);
        my $offset_decimal = $offset_key + 0;  # Force numeric
        
        my $tag_data = {
            offset => $offset_hex,
            offset_decimal => $offset_decimal,
        };
        
        # Handle different tag info structures
        if (ref $tag_info eq 'HASH') {
            # Complex tag definition
            extract_complex_tag_info($tag_data, $tag_info);
        } else {
            # Simple tag name
            $tag_data->{name} = $tag_info;
            $tag_data->{simple} = 1;
        }
        
        # Skip if we didn't get a name
        next unless $tag_data->{name};
        
        push @tags, $tag_data;
    }
    
    return \@tags;
}

#------------------------------------------------------------------------------
# Extract complex tag information
#------------------------------------------------------------------------------
sub extract_complex_tag_info {
    my ($tag_data, $tag_info) = @_;
    
    # Essential fields
    $tag_data->{name} = $tag_info->{Name} if $tag_info->{Name};
    
    # Format specification
    $tag_data->{format} = $tag_info->{Format} if $tag_info->{Format};
    
    # Conditional processing
    $tag_data->{condition} = $tag_info->{Condition} if $tag_info->{Condition};
    
    # Print conversion - simplified for initial implementation
    if ($tag_info->{PrintConv}) {
        if (ref $tag_info->{PrintConv} eq 'HASH') {
            # Convert hash to simpler structure
            my @print_conv_entries;
            for my $key (sort keys %{$tag_info->{PrintConv}}) {
                push @print_conv_entries, { key => $key, value => $tag_info->{PrintConv}->{$key} };
            }
            $tag_data->{print_conv} = \@print_conv_entries;
        } else {
            # String expression
            $tag_data->{print_conv_expr} = $tag_info->{PrintConv};
        }
    }
    
    # Flags and attributes
    $tag_data->{writable} = 1 if $tag_info->{Writable};
    $tag_data->{unknown} = 1 if $tag_info->{Unknown};
    $tag_data->{binary} = 1 if $tag_info->{Binary};
    $tag_data->{hidden} = 1 if $tag_info->{Hidden};
    
    # Description and notes
    $tag_data->{description} = $tag_info->{Description} if $tag_info->{Description};
    $tag_data->{notes} = $tag_info->{Notes} if $tag_info->{Notes};
    
    return;
}