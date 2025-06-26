#!/usr/bin/env perl

=head1 NAME

extract_printconv.pl - Extract PrintConv data from ExifTool modules

=head1 SYNOPSIS

    scripts/extract_printconv.pl [MODULE_NAME] [TAG_NAME]

    # Extract all PrintConv from Canon module
    scripts/extract_printconv.pl Image::ExifTool::Canon

    # Extract specific tag's PrintConv
    scripts/extract_printconv.pl Image::ExifTool::Canon LensType

    # Extract all modules (future feature)
    scripts/extract_printconv.pl --all-modules

=head1 DESCRIPTION

This script uses Perl introspection to extract PrintConv (Print Conversion) data
from ExifTool modules. PrintConv transforms raw EXIF values into human-readable
strings (e.g., lens IDs to lens names, flash codes to descriptions).

The script handles all PrintConv types:
- Direct hash mappings ({ 0 => 'Off', 1 => 'On' })
- Hash references (\%canonLensTypes with 500+ lens definitions)
- Code references (\&PrintExposureTime)
- String expressions ('"$val mm"')

Output is clean JSON suitable for consumption by the Rust sync tools.

=head1 OUTPUT FORMAT

The script outputs JSON with:
- metadata: Module info, extraction date, ExifTool version
- statistics: Tag counts, PrintConv type breakdown
- tags: Array of tags with their PrintConv data
- shared_lookups: Extracted shared tables like canonLensTypes

=head1 AUTHOR

Part of exif-oxide project - Rust implementation of ExifTool

=cut

use strict;
use warnings;
use JSON;
use Data::Dumper;
use lib 'third-party/exiftool/lib';

# Command line processing
my $module_name = $ARGV[0] || 'Image::ExifTool::Canon';
my $specific_tag = $ARGV[1];  # Optional: extract specific tag only

# Load the module
eval "require $module_name";
if ($@) {
    die "Failed to load module $module_name: $@\n";
}

# Track shared lookup tables we've already extracted
my %extracted_lookups;

# Extract PrintConv information from a tag definition
sub extract_printconv {
    my ($module_name, $tag_id, $tag_def) = @_;
    
    return undef unless ref($tag_def) eq 'HASH';
    
    my $pc = $tag_def->{PrintConv};
    return undef unless defined $pc;
    
    my $result = {
        tag_id => $tag_id,
        tag_name => $tag_def->{Name} || $tag_id,
        module => $module_name,
    };
    
    if (ref($pc) eq 'HASH') {
        # Direct hash mapping
        $result->{printconv_type} = 'hash';
        
        # Clean the hash data to remove CODE references
        my %clean_hash;
        foreach my $key (keys %$pc) {
            my $val = $pc->{$key};
            if (!ref($val) || ref($val) eq 'ARRAY') {
                $clean_hash{$key} = $val;
            }
        }
        
        $result->{printconv_data} = \%clean_hash;
        $result->{entry_count} = scalar(keys %clean_hash);
        
        # Check if this might be a known shared table by comparing entries
        if ($clean_hash{1} && $clean_hash{1} =~ /Canon EF/ && $result->{entry_count} > 100) {
            $result->{likely_shared_table} = 'canonLensTypes';
        }
    } elsif (ref($pc) eq 'CODE') {
        # Subroutine reference
        $result->{printconv_type} = 'code_ref';
        
        # Try to identify known subroutines by comparing references
        no strict 'refs';
        if ($pc == \&Image::ExifTool::Exif::PrintExposureTime) {
            $result->{printconv_func} = 'PrintExposureTime';
        } elsif ($pc == \&Image::ExifTool::Exif::PrintFraction) {
            $result->{printconv_func} = 'PrintFraction';
        } elsif ($pc == \&Image::ExifTool::ConvertBinary) {
            $result->{printconv_func} = 'ConvertBinary';
        } else {
            # Try to find function name through symbol table
            $result->{printconv_func} = 'UNKNOWN_CODE_REF';
        }
    } elsif (!ref($pc)) {
        # String (inline code or reference)
        $result->{printconv_type} = 'string';
        $result->{printconv_source} = $pc;
        
        # Check for hash reference pattern like \%canonLensTypes
        if ($pc =~ /^\\%(\w+)$/) {
            my $hash_name = $1;
            no strict 'refs';
            my $full_name = "${module_name}::${hash_name}";
            
            # Check if this hash exists
            my $hash_ref = eval { \%{$full_name} };
            if ($hash_ref && %$hash_ref) {
                $result->{printconv_type} = 'hash_ref';
                $result->{printconv_ref} = $hash_name;
                $result->{printconv_data} = $hash_ref;
                $result->{entry_count} = scalar(keys %$hash_ref);
                
                # Mark this lookup table as extracted
                $extracted_lookups{$hash_name} = {
                    full_name => $full_name,
                    data => $hash_ref,
                    module => $module_name,
                };
            }
        } elsif ($pc =~ /^\\&(\S+)$/) {
            # Subroutine reference like \&PrintExposureTime
            $result->{printconv_type} = 'sub_ref';
            $result->{printconv_func} = $1;
        }
    } elsif (ref($pc) eq 'ARRAY') {
        # Array reference (used for bitfield decoding)
        $result->{printconv_type} = 'array';
        $result->{printconv_data} = $pc;
        $result->{entry_count} = scalar(@$pc);
    }
    
    return $result;
}

# Find all tag tables in the module
my @tables_to_process;
{
    no strict 'refs';
    my $symbol_table = \%{"${module_name}::"};
    
    foreach my $symbol (keys %$symbol_table) {
        my $glob = $symbol_table->{$symbol};
        if (*{$glob}{HASH}) {
            my $hash_ref = \%{"${module_name}::${symbol}"};
            # Check if this looks like a tag table
            if (exists $hash_ref->{GROUPS} || exists $hash_ref->{NOTES} || 
                exists $hash_ref->{0} || exists $hash_ref->{0x01} ||
                $symbol =~ /Table$/ || $symbol eq 'Main') {
                push @tables_to_process, {
                    name => $symbol,
                    ref => $hash_ref,
                };
            }
        }
    }
}

# Extract PrintConv from all tables
my @extracted_tags;
my %stats = (
    total_tags => 0,
    tags_with_printconv => 0,
    printconv_types => {},
);

foreach my $table_info (@tables_to_process) {
    my $table = $table_info->{ref};
    my $table_name = $table_info->{name};
    
    foreach my $key (keys %$table) {
        # Skip special keys
        next if $key =~ /^(GROUPS|NOTES|NAMESPACE|PRIORITY|WRITE_PROC|PROCESS_PROC|CHECK_PROC|VARS|TABLE_NAME|SHORT_NAME)$/;
        
        # Handle specific tag filter if provided
        if ($specific_tag) {
            my $tag_def = $table->{$key};
            if (ref($tag_def) eq 'HASH' && $tag_def->{Name}) {
                next unless $tag_def->{Name} eq $specific_tag;
            }
        }
        
        $stats{total_tags}++;
        
        my $printconv_info = extract_printconv($module_name, $key, $table->{$key});
        if ($printconv_info) {
            $printconv_info->{table_name} = $table_name;
            push @extracted_tags, $printconv_info;
            $stats{tags_with_printconv}++;
            $stats{printconv_types}{$printconv_info->{printconv_type}}++;
        }
    }
}

# Clean up shared lookups to remove CODE references
my %clean_lookups;
foreach my $name (keys %extracted_lookups) {
    my $lookup = $extracted_lookups{$name};
    my %clean_data;
    
    if (ref($lookup->{data}) eq 'HASH') {
        foreach my $key (keys %{$lookup->{data}}) {
            my $val = $lookup->{data}->{$key};
            # Skip CODE references and other non-serializable values
            if (!ref($val) || ref($val) eq 'ARRAY') {
                $clean_data{$key} = $val;
            }
        }
    }
    
    $clean_lookups{$name} = {
        %$lookup,
        data => \%clean_data,
        entry_count => scalar(keys %clean_data),
    };
}

# Create output structure
my $output = {
    metadata => {
        module => $module_name,
        extraction_date => scalar(localtime),
        exiftool_version => $Image::ExifTool::VERSION || 'unknown',
        specific_tag => $specific_tag,
    },
    statistics => \%stats,
    tags => \@extracted_tags,
    shared_lookups => \%clean_lookups,
};

# Output as JSON
my $json = JSON->new->pretty->canonical->allow_nonref;
print $json->encode($output);