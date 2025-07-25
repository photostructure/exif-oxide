#!/usr/bin/env perl
#
# Shared Tables Extractor
# 
# This script extracts tables that are referenced across module boundaries.
# It's designed to run as a pre-processing step before the main tag kit extraction.
#

use strict;
use warnings;
use lib '../../third-party/exiftool/lib';
use JSON;
use Data::Dumper;

# Configure output
$Data::Dumper::Sortkeys = 1;
$Data::Dumper::Indent = 1;

# Priority tables to extract (based on cross-module analysis)
my @priority_tables = (
    # XMP tables (most referenced)
    'Image::ExifTool::XMP::Main',
    'Image::ExifTool::XMP::dc',
    'Image::ExifTool::XMP::xmp',
    'Image::ExifTool::XMP::xmpMM',
    'Image::ExifTool::XMP::xmpBJ',
    'Image::ExifTool::XMP::xmpRights',
    'Image::ExifTool::XMP::xmpDM',
    'Image::ExifTool::XMP::pdf',
    'Image::ExifTool::XMP::photoshop',
    'Image::ExifTool::XMP::crs',
    
    # ICC_Profile tables
    'Image::ExifTool::ICC_Profile::Main',
    
    # Kodak tables
    'Image::ExifTool::Kodak::IFD',
    'Image::ExifTool::Kodak::SubIFD0',
    'Image::ExifTool::Kodak::SubIFD1',
    'Image::ExifTool::Kodak::SubIFD2',
    'Image::ExifTool::Kodak::SubIFD3',
    'Image::ExifTool::Kodak::SubIFD4',
    'Image::ExifTool::Kodak::SubIFD5',
    'Image::ExifTool::Kodak::SubIFD6',
    
    # NikonCustom tables
    'Image::ExifTool::NikonCustom::SettingsD40',
    'Image::ExifTool::NikonCustom::SettingsD80',
    'Image::ExifTool::NikonCustom::SettingsD90',
    'Image::ExifTool::NikonCustom::SettingsD200',
    'Image::ExifTool::NikonCustom::SettingsD300',
    'Image::ExifTool::NikonCustom::SettingsD3',
    'Image::ExifTool::NikonCustom::SettingsD700',
    'Image::ExifTool::NikonCustom::SettingsD7000',
    
    # Canon/CanonCustom tables
    'Image::ExifTool::Canon::AFInfo',
    'Image::ExifTool::Canon::CameraSettings',
    'Image::ExifTool::Canon::ShotInfo',
    'Image::ExifTool::CanonCustom::Functions1D',
    'Image::ExifTool::CanonCustom::Functions2',
    'Image::ExifTool::CanonCustom::Functions10D',
    'Image::ExifTool::CanonCustom::FunctionsD30',
    
    # PrintIM
    'Image::ExifTool::PrintIM::Main',
    
    # Other commonly referenced
    'Image::ExifTool::Exif::Main',
    'Image::ExifTool::GPS::Main',
    'Image::ExifTool::IPTC::Main',
    'Image::ExifTool::Photoshop::Main',
);

my %extracted_tables;
my $json = JSON->new->utf8->pretty->canonical(1);

# Load required modules
foreach my $table_path (@priority_tables) {
    if ($table_path =~ /^Image::ExifTool::(\w+)::/) {
        my $module = $1;
        eval "require Image::ExifTool::$module";
        if ($@) {
            warn "Failed to load module $module: $@\n";
        }
    }
}

# Extract table data
foreach my $table_path (@priority_tables) {
    extract_table($table_path);
}

# Output results
print $json->encode({
    metadata => {
        version => '1.0',
        purpose => 'Shared tables for cross-module subdirectory references',
        generated => scalar(localtime),
        table_count => scalar(keys %extracted_tables),
    },
    tables => \%extracted_tables,
});

sub extract_table {
    my ($table_path) = @_;
    
    # Get reference to the table
    my $table_ref;
    {
        no strict 'refs';
        $table_ref = \%{$table_path};
    }
    
    # Skip if table doesn't exist
    return unless %$table_ref;
    
    print STDERR "Extracting table: $table_path\n";
    
    # Extract basic structure
    my %table_data = (
        path => $table_path,
        type => detect_table_type($table_ref),
        tags => {},
    );
    
    # Extract tag information
    foreach my $tag (sort keys %$table_ref) {
        next if $tag =~ /^(GROUPS|TABLE_NAME|SHORT_NAME|PROCESS_PROC|WRITE_PROC|CHECK_PROC|VARS|DATAMEMBER|IS_OFFSET|IS_SUBDIR|FIRST_ENTRY|TAG_PREFIX|PRINT_CONV)$/;
        
        my $tag_info = $table_ref->{$tag};
        next unless ref $tag_info eq 'HASH';
        
        # Store essential tag information
        my %tag_data;
        
        # Copy basic fields
        foreach my $field (qw(Name Writable Format Binary)) {
            next unless exists $tag_info->{$field};
            my $value = $tag_info->{$field};
            # Convert CODE references to string
            if (ref $value eq 'CODE') {
                $tag_data{$field} = 'CODE';
            } elsif (ref $value eq 'HASH' || ref $value eq 'ARRAY') {
                # Skip complex references for now
                next;
            } else {
                $tag_data{$field} = $value;
            }
        }
        
        # Handle SubDirectory specially
        if (exists $tag_info->{SubDirectory}) {
            my $subdir = $tag_info->{SubDirectory};
            if (ref $subdir eq 'HASH') {
                $tag_data{SubDirectory} = {};
                
                # Handle each field carefully
                if (exists $subdir->{TagTable}) {
                    $tag_data{SubDirectory}{TagTable} = $subdir->{TagTable};
                }
                if (exists $subdir->{ProcessProc}) {
                    # Convert CODE ref to string representation
                    $tag_data{SubDirectory}{ProcessProc} = ref($subdir->{ProcessProc}) eq 'CODE' ? 'CODE' : $subdir->{ProcessProc};
                }
                if (exists $subdir->{Start}) {
                    $tag_data{SubDirectory}{Start} = $subdir->{Start};
                }
                if (exists $subdir->{ByteOrder}) {
                    $tag_data{SubDirectory}{ByteOrder} = $subdir->{ByteOrder};
                }
            }
        }
        
        $table_data{tags}{$tag} = \%tag_data;
    }
    
    # Store special table properties
    if (exists $table_ref->{PROCESS_PROC}) {
        $table_data{process_proc} = ref($table_ref->{PROCESS_PROC}) ? 'CODE' : $table_ref->{PROCESS_PROC};
    }
    
    $extracted_tables{$table_path} = \%table_data;
}

sub detect_table_type {
    my ($table_ref) = @_;
    
    # Check for binary data table indicators
    return 'binary_data' if exists $table_ref->{PROCESS_PROC};
    return 'binary_data' if exists $table_ref->{FIRST_ENTRY};
    
    # Check if it's primarily SubDirectory entries
    my $subdir_count = 0;
    my $total_tags = 0;
    
    foreach my $tag (keys %$table_ref) {
        next if $tag =~ /^[A-Z_]+$/;  # Skip special keys
        $total_tags++;
        my $tag_info = $table_ref->{$tag};
        $subdir_count++ if ref $tag_info eq 'HASH' && exists $tag_info->{SubDirectory};
    }
    
    return 'subdirectory_table' if $total_tags > 0 && $subdir_count / $total_tags > 0.5;
    
    return 'standard';
}