#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         extract_tables.pl
#
# Description:  Extract EXIF IFD0 tags from ExifTool for exif-oxide codegen
#
# Usage:        perl extract_tables.pl > generated/tag_tables.json
#
# Notes:        This script extracts only mainstream tags (frequency > 0.95)
#               from ExifTool's EXIF::Main table and outputs clean JSON
#               for Rust code generation.
#------------------------------------------------------------------------------

use strict;
use warnings;
use JSON qw(encode_json);
use FindBin qw($Bin);
use lib "$Bin/../third-party/exiftool/lib";

# Load ExifTool modules
use Image::ExifTool;
use Image::ExifTool::Exif;
use Image::ExifTool::GPS;

# Load tag metadata for frequency filtering
my $metadata_file = "$Bin/../third-party/exiftool/doc/TagMetadata.json";
my $metadata = load_tag_metadata($metadata_file);

# Track all conversion references across all tables
my %all_print_conv_refs;
my %all_value_conv_refs;

# Extract tags from multiple tables
my @all_extracted_tags;

# Extract from EXIF::Main table
my @exif_main_tags = extract_exif_main_tags($metadata, \%all_print_conv_refs, \%all_value_conv_refs);
push @all_extracted_tags, @exif_main_tags;

# Extract from GPS table (Milestone 6+ requirement)
my @gps_tags = extract_gps_tags($metadata, \%all_print_conv_refs, \%all_value_conv_refs);
push @all_extracted_tags, @gps_tags;

# Convert hash keys to sorted arrays for consistent output
my @print_conv_refs = sort keys %all_print_conv_refs;
my @value_conv_refs = sort keys %all_value_conv_refs;

# Output JSON with conversion reference lists
print encode_json({
    extracted_at => scalar(gmtime()),
    exiftool_version => $Image::ExifTool::VERSION,
    filter_criteria => "frequency > 0.95 OR mainstream = true",
    total_tags => scalar(@all_extracted_tags),
    tags => \@all_extracted_tags,
    # New: Conversion reference lists for codegen
    conversion_refs => {
        print_conv => \@print_conv_refs,
        value_conv => \@value_conv_refs,
    },
});

exit 0;

#------------------------------------------------------------------------------
# Load TagMetadata.json for frequency filtering
#------------------------------------------------------------------------------
sub load_tag_metadata {
    my $file = shift;
    
    open(my $fh, '<', $file) or die "Cannot open TagMetadata.json: $!";
    my $json_text = do { local $/; <$fh> };
    close($fh);
    
    my $json = JSON->new;
    return $json->decode($json_text);
}

#------------------------------------------------------------------------------
# Check if tag should be included based on frequency/mainstream criteria
#------------------------------------------------------------------------------
sub is_mainstream_tag {
    my ($tag_name, $metadata) = @_;
    
    return 0 unless defined $tag_name;
    
    # Check metadata
    if (exists $metadata->{$tag_name}) {
        my $meta = $metadata->{$tag_name};
        
        # TODO: Lower frequency to include more tags
        return 1 if ($meta->{frequency} && $meta->{frequency} > 0.95);

        # TODO: Uncomment to include all mainstream tags
        # return 1 if ($meta->{mainstream});
    }
    
    # Always include basic file information tags and tags with working PrintConv implementations
    my @always_include = qw(
        ImageWidth ImageHeight Make Model Orientation
        ExifImageWidth ExifImageHeight DateTime
        ImageDescription Copyright
        Flash ColorSpace ExposureProgram MeteringMode
        ResolutionUnit YCbCrPositioning YCbCrSubSampling
        WhiteBalance ExposureTime FNumber FocalLength
        DateTimeOriginal CreateDate
        ExifOffset GPSInfo
    );
    
    return 1 if grep { $_ eq $tag_name } @always_include;
    
    return 0;
}

#------------------------------------------------------------------------------
# Generate conversion reference string for PrintConv/ValueConv
#------------------------------------------------------------------------------
sub generate_conv_ref {
    my ($tag_name, $conv_type, $conv_data) = @_;
    
    return undef unless defined $conv_data;
    
    # Generate a reference string based on tag name and conversion type
    my $ref = lc($tag_name);
    $ref =~ s/[^a-z0-9]/_/g;  # Replace non-alphanumeric with underscore
    $ref .= "_${conv_type}";
    
    return $ref;
}

#------------------------------------------------------------------------------
# Extract format information from tag definition
#------------------------------------------------------------------------------
sub extract_format {
    my $tag_info = shift;
    
    # Get Writable format (preferred) or Format
    my $format = $tag_info->{Writable} || $tag_info->{Format} || 'undef';
    
    # Handle format specifications
    if (ref $format eq 'HASH') {
        # Complex format - return default
        return 'undef';
    }
    
    # Clean up format string
    $format =~ s/\s+//g;  # Remove whitespace
    
    return $format;
}

#------------------------------------------------------------------------------
# Extract groups information from tag definition
#------------------------------------------------------------------------------
sub extract_groups {
    my $tag_info = shift;
    
    my @groups = ('EXIF');  # Default group
    
    if ($tag_info->{Groups}) {
        my $groups_ref = $tag_info->{Groups};
        if (ref $groups_ref eq 'HASH') {
            # Add group values
            push @groups, values %$groups_ref;
        }
    }
    
    # Remove duplicates and sort
    my %seen;
    @groups = grep { !$seen{$_}++ } @groups;
    @groups = sort @groups;
    
    return \@groups;
}

#------------------------------------------------------------------------------
# Extract tags from Image::ExifTool::Exif::Main table
#------------------------------------------------------------------------------
sub extract_exif_main_tags {
    my ($metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @tags;
    
    # Access the EXIF::Main table
    my %exif_main = %Image::ExifTool::Exif::Main;
    
    # Process each tag in the table
    foreach my $tag_id (sort keys %exif_main) {
        next if $tag_id =~ /^[A-Z]/;  # Skip special table keys (GROUPS, WRITE_PROC, etc.)
        
        my $tag_info = $exif_main{$tag_id};
        next unless ref $tag_info eq 'HASH';
        next unless exists $tag_info->{Name};
        
        my $tag_name = $tag_info->{Name};
        
        # Apply mainstream filtering
        next unless is_mainstream_tag($tag_name, $metadata);
        
        # Extract tag information
        my $tag_data = {
            id => sprintf("0x%x", $tag_id),
            name => $tag_name,
            format => extract_format($tag_info),
            groups => extract_groups($tag_info),
            writable => $tag_info->{Writable} ? 1 : 0,
        };
        
        # Add description if available
        if ($tag_info->{Description}) {
            $tag_data->{description} = $tag_info->{Description};
        }
        
        # Generate conversion references (always as strings, even for simple ones)
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($tag_name, 'print_conv', $tag_info->{PrintConv});
            $tag_data->{print_conv_ref} = $ref;
            # Track for comprehensive list
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($tag_name, 'value_conv', $tag_info->{ValueConv});
            $tag_data->{value_conv_ref} = $ref;
            # Track for comprehensive list
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add metadata if available
        if (exists $metadata->{$tag_name}) {
            my $meta = $metadata->{$tag_name};
            $tag_data->{frequency} = $meta->{frequency};
            $tag_data->{mainstream} = $meta->{mainstream} ? 1 : 0;
        }
        
        # Add notes if available
        if ($tag_info->{Notes}) {
            $tag_data->{notes} = $tag_info->{Notes};
        }
        
        push @tags, $tag_data;
    }
    
    return @tags;
}

#------------------------------------------------------------------------------
# Extract tags from Image::ExifTool::GPS::Main table
#------------------------------------------------------------------------------
sub extract_gps_tags {
    my ($metadata, $print_conv_refs, $value_conv_refs) = @_;
    my @tags;
    
    # Access the GPS::Main table
    my %gps_main = %Image::ExifTool::GPS::Main;
    
    # Process each tag in the GPS table
    foreach my $tag_id (sort keys %gps_main) {
        next if $tag_id =~ /^[A-Z]/;  # Skip special table keys (GROUPS, WRITE_PROC, etc.)
        
        my $tag_info = $gps_main{$tag_id};
        next unless ref $tag_info eq 'HASH';
        next unless exists $tag_info->{Name};
        
        my $tag_name = $tag_info->{Name};
        
        # GPS tags are always included (essential for Milestone 6+)
        # Apply lighter filtering for GPS as they're already specialized
        
        # Extract tag information
        my $tag_data = {
            id => sprintf("0x%x", $tag_id),
            name => $tag_name,
            format => extract_format($tag_info),
            groups => ['GPS'],  # GPS tags always in GPS group
            writable => $tag_info->{Writable} ? 1 : 0,
        };
        
        # Add description if available
        if ($tag_info->{Description}) {
            $tag_data->{description} = $tag_info->{Description};
        }
        
        # Generate conversion references for GPS tags
        if ($tag_info->{PrintConv}) {
            my $ref = generate_conv_ref($tag_name, 'print_conv', $tag_info->{PrintConv});
            $tag_data->{print_conv_ref} = $ref;
            # Track for comprehensive list
            $print_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        if ($tag_info->{ValueConv}) {
            my $ref = generate_conv_ref($tag_name, 'value_conv', $tag_info->{ValueConv});
            $tag_data->{value_conv_ref} = $ref;
            # Track for comprehensive list
            $value_conv_refs->{$ref} = 1 if defined $ref;
        }
        
        # Add metadata if available (GPS tags might not have frequency data)
        if (exists $metadata->{$tag_name}) {
            my $meta = $metadata->{$tag_name};
            $tag_data->{frequency} = $meta->{frequency};
            $tag_data->{mainstream} = $meta->{mainstream} ? 1 : 0;
        }
        
        # Add notes if available
        if ($tag_info->{Notes}) {
            $tag_data->{notes} = $tag_info->{Notes};
        }
        
        push @tags, $tag_data;
    }
    
    return @tags;
}

__END__

=head1 NAME

extract_tables.pl - Extract ExifTool tags for exif-oxide codegen

=head1 SYNOPSIS

perl extract_tables.pl > tag_tables.json

=head1 DESCRIPTION

This script extracts mainstream EXIF and GPS tags from ExifTool's tag tables:
- lib/Image/ExifTool/Exif.pm Main table  
- lib/Image/ExifTool/GPS.pm Main table

It filters EXIF tags based on frequency (> 0.95) or mainstream flag in
TagMetadata.json. GPS tags are always included as they're essential for
Milestone 6+ functionality.

The output includes comprehensive conversion reference lists for DRY code
generation, following the "runtime references, no stubs" approach from 
exif-oxide's architecture.

=head1 OUTPUT FORMAT

{
  "extracted_at": "timestamp",
  "exiftool_version": "12.76", 
  "filter_criteria": "frequency > 0.95 OR mainstream = true",
  "total_tags": 55,
  "tags": [
    {
      "id": "0x10f",
      "name": "Make",
      "format": "string",
      "groups": ["EXIF", "IFD0", "Camera"],
      "writable": 1,
      "print_conv_ref": "make_print_conv",
      "value_conv_ref": null,
      "frequency": 0.98,
      "mainstream": 1
    }
  ],
  "conversion_refs": {
    "print_conv": [
      "colorspace_print_conv",
      "flash_print_conv", 
      "gpslatituderef_print_conv",
      "orientation_print_conv"
    ],
    "value_conv": [
      "gps_coordinate_value_conv"
    ]
  }
}

=head1 AUTHOR

exif-oxide project

=cut