#!/usr/bin/env perl

=head1 NAME

extract_shared_tables.pl - Extract module-level shared lookup tables from ExifTool

=head1 SYNOPSIS

    scripts/extract_shared_tables.pl [MODULE_NAME]
    scripts/extract_shared_tables.pl --all-modules

=head1 DESCRIPTION

Phase 1 of the new two-phase PrintConv extraction architecture. This script extracts
module-level shared lookup tables like %canonLensTypes, %nikonLensTypes that are
defined once and referenced by multiple tags.

This preserves ExifTool's DRY architecture by extracting shared data separately
from tag definitions.

=head1 OUTPUT

JSON with shared lookup tables:
{
  "metadata": { ... },
  "shared_tables": {
    "canonLensTypes": {
      "module": "Image::ExifTool::Canon",
      "entries": { "1": "Canon EF 50mm f/1.8", ... }
    }
  }
}

=cut

use strict;
use warnings;
use JSON;
use lib 'third-party/exiftool/lib';

# Find shared lookup tables in ExifTool modules
my %known_shared_tables = (
    'Image::ExifTool::Canon' => ['canonLensTypes', 'canonModelID'],
    'Image::ExifTool::Nikon' => ['nikonLensTypes', 'nikonLensIDs'],
    'Image::ExifTool::Sony' => ['sonyLensTypes', 'sonyLensTypes2'],
    'Image::ExifTool::Olympus' => ['olympusLensTypes'],
    'Image::ExifTool::Pentax' => ['pentaxLensTypes'],
    'Image::ExifTool::Sigma' => ['sigmaLensTypes'],
    'Image::ExifTool::Tamron' => ['tamronLensTypes'],
);

sub extract_shared_tables {
    my ($module_name) = @_;
    
    # Load the module
    eval "require $module_name";
    if ($@) {
        warn "Failed to load module $module_name: $@";
        return {};
    }
    
    my %shared_tables;
    my $tables_to_check = $known_shared_tables{$module_name} || [];
    
    foreach my $table_name (@$tables_to_check) {
        no strict 'refs';
        my $full_name = "${module_name}::${table_name}";
        
        # Check if the hash exists and has content
        if (%{$full_name}) {
            my %table_copy = %{$full_name};
            
            # Clean and validate the table
            my %clean_entries;
            foreach my $key (keys %table_copy) {
                my $val = $table_copy{$key};
                
                # Only include scalar string values
                if (defined $val && !ref($val) && length($val) > 0) {
                    $clean_entries{$key} = $val;
                }
            }
            
            if (keys %clean_entries > 0) {
                $shared_tables{$table_name} = {
                    module => $module_name,
                    table_name => $table_name,
                    entry_count => scalar(keys %clean_entries),
                    entries => \%clean_entries,
                };
                
                print STDERR "Found shared table: $table_name with " . 
                            scalar(keys %clean_entries) . " entries\n";
            }
        }
    }
    
    return \%shared_tables;
}

sub extract_all_modules {
    my %all_shared_tables;
    my $total_tables = 0;
    my $total_entries = 0;
    
    foreach my $module_name (keys %known_shared_tables) {
        print STDERR "Processing module: $module_name\n";
        
        my $tables = extract_shared_tables($module_name);
        foreach my $table_name (keys %$tables) {
            $all_shared_tables{$table_name} = $tables->{$table_name};
            $total_tables++;
            $total_entries += $tables->{$table_name}{entry_count};
        }
    }
    
    my $output = {
        metadata => {
            extraction_mode => 'shared_tables_all_modules',
            extraction_date => scalar(gmtime()) . ' UTC',
            exiftool_version => get_exiftool_version(),
            processed_modules => scalar(keys %known_shared_tables),
        },
        statistics => {
            total_shared_tables => $total_tables,
            total_entries => $total_entries,
        },
        shared_tables => \%all_shared_tables,
    };
    
    return $output;
}

sub extract_single_module {
    my ($module_name) = @_;
    
    my $tables = extract_shared_tables($module_name);
    my $total_entries = 0;
    foreach my $table (values %$tables) {
        $total_entries += $table->{entry_count};
    }
    
    my $output = {
        metadata => {
            extraction_mode => 'shared_tables_single_module',
            extraction_date => scalar(gmtime()) . ' UTC',
            exiftool_version => get_exiftool_version(),
            module => $module_name,
        },
        statistics => {
            total_shared_tables => scalar(keys %$tables),
            total_entries => $total_entries,
        },
        shared_tables => $tables,
    };
    
    return $output;
}

sub get_exiftool_version {
    eval { require Image::ExifTool };
    return $Image::ExifTool::VERSION || 'unknown';
}

# Main execution
sub main {
    my $arg = shift @ARGV;
    
    my $output;
    if (!$arg) {
        die "Usage: $0 [MODULE_NAME] or $0 --all-modules\n";
    } elsif ($arg eq '--all-modules') {
        $output = extract_all_modules();
    } else {
        $output = extract_single_module($arg);
    }
    
    # Output deterministic JSON
    my $json = JSON->new->canonical->pretty;
    print $json->encode($output);
}

main() unless caller;