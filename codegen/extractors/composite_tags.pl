#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         composite_tags.pl
#
# Description:  Extract composite tag definitions from ExifTool
#
# Usage:        perl composite_tags.pl > ../generated/composite_tags.json
#
# Notes:        This script extracts only composite tag definitions.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_tag_metadata
    is_mainstream_composite_tag
    generate_conv_ref
    format_json_output
    clean_tag_name
);

use Image::ExifTool;
use Image::ExifTool::Exif;
use Image::ExifTool::GPS;

# Load tag metadata for frequency filtering
my $metadata_file = "$Bin/../../third-party/exiftool/doc/TagMetadata.json";
my $metadata = load_tag_metadata($metadata_file);

# Track conversion references
my %all_print_conv_refs;
my %all_value_conv_refs;

# Extract composite tags
my @all_composite_tags;

print STDERR "Extracting composite tags...\n";

# Extract from main composite table
print STDERR "  Checking main composite table...\n";
my @main_composites = extract_composite_from_table(
    \%Image::ExifTool::Composite,
    "Main",
    $metadata,
    \%all_print_conv_refs,
    \%all_value_conv_refs
);
push @all_composite_tags, @main_composites;
print STDERR "    Found " . scalar(@main_composites) . " from Main table\n";

# Extract from EXIF composite table
print STDERR "  Checking EXIF composite table...\n";
my @exif_composites = extract_composite_from_table(
    \%Image::ExifTool::Exif::Composite,
    "EXIF",
    $metadata,
    \%all_print_conv_refs,
    \%all_value_conv_refs
);
push @all_composite_tags, @exif_composites;
print STDERR "    Found " . scalar(@exif_composites) . " from EXIF table\n";

# Extract from GPS composite table
print STDERR "  Checking GPS composite table...\n";
my @gps_composites = extract_composite_from_table(
    \%Image::ExifTool::GPS::Composite,
    "GPS",
    $metadata,
    \%all_print_conv_refs,
    \%all_value_conv_refs
);
push @all_composite_tags, @gps_composites;
print STDERR "    Found " . scalar(@gps_composites) . " from GPS table\n";

print STDERR "Total composite tags extracted: " . scalar(@all_composite_tags) . "\n";

# Convert references to sorted arrays
my @print_conv_refs = sort keys %all_print_conv_refs;
my @value_conv_refs = sort keys %all_value_conv_refs;

# Output JSON
my $output = {
    extracted_at => scalar(gmtime()) . " GMT",
    exiftool_version => $Image::ExifTool::VERSION,
    filter_criteria => "frequency > 0.5 OR mainstream = true (lenient for composites)",
    composite_tags => \@all_composite_tags,
    stats => {
        total_composite_tags => scalar(@all_composite_tags),
        main_table => scalar(@main_composites),
        exif_table => scalar(@exif_composites),
        gps_table => scalar(@gps_composites),
    },
    conversion_refs => {
        print_conv => \@print_conv_refs,
        value_conv => \@value_conv_refs,
    },
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract composite tags from a specific table
#------------------------------------------------------------------------------
sub extract_composite_from_table {
    my ($table_ref, $table_name, $metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @composite_tags;
    
    foreach my $tag_name (sort keys %$table_ref) {
        # Skip special table keys
        next if $tag_name =~ /^[A-Z_]+$/;
        next if $tag_name eq 'GROUPS';
        
        my $tag_info = $table_ref->{$tag_name};
        next unless ref $tag_info eq 'HASH';
        
        # Clean tag name
        my $clean_tag_name = clean_tag_name($tag_name);
        
        # Apply mainstream filtering
        next unless is_mainstream_composite_tag($clean_tag_name, $metadata);
        
        # Build composite data
        my $composite_data = {
            name => $clean_tag_name,
            table => $table_name,
            full_name => $tag_name,
        };
        
        # Extract dependencies
        if ($tag_info->{Require}) {
            $composite_data->{require} = extract_dependencies($tag_info->{Require});
        }
        
        if ($tag_info->{Desire}) {
            $composite_data->{desire} = extract_dependencies($tag_info->{Desire});
        }
        
        # Add conversion references
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($clean_tag_name, 'print_conv', $tag_info->{PrintConv});
            $composite_data->{print_conv_ref} = $ref;
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($clean_tag_name, 'value_conv', $tag_info->{ValueConv});
            $composite_data->{value_conv_ref} = $ref;
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add description if available
        $composite_data->{description} = $tag_info->{Description} if $tag_info->{Description};
        
        # Add writable flag
        $composite_data->{writable} = $tag_info->{Writable} ? 1 : 0;
        
        # Add metadata if available
        if (exists $metadata->{$clean_tag_name}) {
            my $meta = $metadata->{$clean_tag_name};
            $composite_data->{frequency} = $meta->{frequency} if $meta->{frequency};
            $composite_data->{mainstream} = $meta->{mainstream} ? 1 : 0 if $meta->{mainstream};
        }
        
        push @composite_tags, $composite_data;
    }
    
    return @composite_tags;
}

#------------------------------------------------------------------------------
# Extract dependency information from Require/Desire fields
#------------------------------------------------------------------------------
sub extract_dependencies {
    my $deps = shift;
    
    if (ref $deps eq 'HASH') {
        # Hash format with numbered keys
        my @dep_list;
        foreach my $key (sort { $a <=> $b } keys %$deps) {
            push @dep_list, $deps->{$key} if $key =~ /^\d+$/;
        }
        return \@dep_list;
    } elsif (ref $deps eq 'ARRAY') {
        # Already an array
        return $deps;
    } elsif (!ref $deps) {
        # Single dependency as string
        return [$deps];
    }
    
    return [];
}