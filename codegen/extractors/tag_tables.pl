#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         tag_tables.pl
#
# Description:  Extract EXIF and GPS tag definitions from ExifTool
#
# Usage:        perl tag_tables.pl > ../generated/tag_tables.json
#
# Notes:        This script extracts only tag definitions (EXIF/GPS),
#               not composite tags.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_tag_metadata
    is_mainstream_tag
    generate_conv_ref
    extract_format
    extract_groups
    format_json_output
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

# Extract tags
my @all_tags;

# Extract EXIF tags
print STDERR "Extracting EXIF tags...\n";
my @exif_tags = extract_exif_tags($metadata, \%all_print_conv_refs, \%all_value_conv_refs);
push @all_tags, @exif_tags;
print STDERR "  Found " . scalar(@exif_tags) . " EXIF tags\n";

# Extract GPS tags
print STDERR "Extracting GPS tags...\n";
my @gps_tags = extract_gps_tags($metadata, \%all_print_conv_refs, \%all_value_conv_refs);
push @all_tags, @gps_tags;
print STDERR "  Found " . scalar(@gps_tags) . " GPS tags\n";

# Convert references to sorted arrays
my @print_conv_refs = sort keys %all_print_conv_refs;
my @value_conv_refs = sort keys %all_value_conv_refs;

# Output JSON
my $output = {
    extracted_at => scalar(gmtime()) . " GMT",
    exiftool_version => $Image::ExifTool::VERSION,
    filter_criteria => "frequency > 0.8 OR mainstream = true",
    tags => {
        exif => \@exif_tags,
        gps => \@gps_tags,
    },
    stats => {
        total_tags => scalar(@all_tags),
        exif_count => scalar(@exif_tags),
        gps_count => scalar(@gps_tags),
    },
    conversion_refs => {
        print_conv => \@print_conv_refs,
        value_conv => \@value_conv_refs,
    },
};

print format_json_output($output);

#------------------------------------------------------------------------------
# Extract tags from EXIF::Main table
#------------------------------------------------------------------------------
sub extract_exif_tags {
    my ($metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @tags;
    
    my %exif_main = %Image::ExifTool::Exif::Main;
    
    foreach my $tag_id (sort keys %exif_main) {
        next if $tag_id =~ /^[A-Z]/;  # Skip special keys
        
        my $tag_info = $exif_main{$tag_id};
        next unless ref $tag_info eq 'HASH';
        next unless exists $tag_info->{Name};
        
        my $tag_name = $tag_info->{Name};
        
        # Apply mainstream filtering
        next unless is_mainstream_tag($tag_name, $metadata);
        
        # Build tag data
        my $tag_data = {
            id => sprintf("0x%x", $tag_id),
            name => $tag_name,
            format => extract_format($tag_info),
            groups => extract_groups($tag_info),
            writable => $tag_info->{Writable} ? 1 : 0,
        };
        
        # Add optional fields
        $tag_data->{description} = $tag_info->{Description} if $tag_info->{Description};
        $tag_data->{notes} = $tag_info->{Notes} if $tag_info->{Notes};
        
        # Add conversion references
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($tag_name, 'print_conv', $tag_info->{PrintConv});
            $tag_data->{print_conv_ref} = $ref;
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($tag_name, 'value_conv', $tag_info->{ValueConv});
            $tag_data->{value_conv_ref} = $ref;
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add metadata
        if (exists $metadata->{$tag_name}) {
            my $meta = $metadata->{$tag_name};
            $tag_data->{frequency} = $meta->{frequency} if $meta->{frequency};
            $tag_data->{mainstream} = $meta->{mainstream} ? 1 : 0 if $meta->{mainstream};
        }
        
        push @tags, $tag_data;
    }
    
    return @tags;
}

#------------------------------------------------------------------------------
# Extract tags from GPS::Main table
#------------------------------------------------------------------------------
sub extract_gps_tags {
    my ($metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @tags;
    
    my %gps_main = %Image::ExifTool::GPS::Main;
    
    foreach my $tag_id (sort keys %gps_main) {
        next if $tag_id =~ /^[A-Z]/;  # Skip special keys
        
        my $tag_info = $gps_main{$tag_id};
        next unless ref $tag_info eq 'HASH';
        next unless exists $tag_info->{Name};
        
        my $tag_name = $tag_info->{Name};
        
        # GPS tags are always included (essential for Milestone 6+)
        
        # Build tag data
        my $tag_data = {
            id => sprintf("0x%x", $tag_id),
            name => $tag_name,
            format => extract_format($tag_info),
            groups => ['GPS'],
            writable => $tag_info->{Writable} ? 1 : 0,
        };
        
        # Add optional fields
        $tag_data->{description} = $tag_info->{Description} if $tag_info->{Description};
        $tag_data->{notes} = $tag_info->{Notes} if $tag_info->{Notes};
        
        # Add conversion references
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($tag_name, 'print_conv', $tag_info->{PrintConv});
            $tag_data->{print_conv_ref} = $ref;
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($tag_name, 'value_conv', $tag_info->{ValueConv});
            $tag_data->{value_conv_ref} = $ref;
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add metadata if available
        if (exists $metadata->{$tag_name}) {
            my $meta = $metadata->{$tag_name};
            $tag_data->{frequency} = $meta->{frequency} if $meta->{frequency};
            $tag_data->{mainstream} = $meta->{mainstream} ? 1 : 0 if $meta->{mainstream};
        }
        
        push @tags, $tag_data;
    }
    
    return @tags;
}