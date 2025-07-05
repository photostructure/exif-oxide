#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         extract_simple_tables.pl
#
# Description:  Extract simple lookup tables from ExifTool modules
#
# Usage:        perl extract_simple_tables.pl > generated/simple_tables.json
#
# Notes:        This script safely extracts primitive key-value tables from
#               ExifTool modules and outputs JSON for Rust code generation.
#               Only simple tables with no Perl logic are extracted.
#------------------------------------------------------------------------------

use strict;
use warnings;
use JSON qw(encode_json decode_json);
use FindBin qw($Bin);

# Read configuration file
sub load_table_config {
    my $config_file = "$Bin/simple_tables.json";
    
    open(my $fh, '<', $config_file) or die "Cannot open $config_file: $!";
    my $json_text = do { local $/; <$fh> };
    close($fh);
    
    my $config = decode_json($json_text);
    
    return @{$config->{tables}};
}

# Extract a single table from ExifTool module using dynamic loading
sub load_and_extract_table {
    my ($module_file, $hash_name) = @_;

    # Load the module dynamically
    my $module_name = load_module_from_file($module_file);
    return () unless $module_name;

    # Access the package hash using symbolic references
    my $hash_ref = get_package_hash($module_name, $hash_name);
    return () unless $hash_ref;

    # Extract primitive entries from the loaded hash
    return extract_primitive_entries($hash_ref);
}

# Safely load a Perl module from a file path
sub load_module_from_file {
    my ($module_file) = @_;
    
    # Validate the file exists and is readable
    unless (-r $module_file) {
        warn "Cannot read module file: $module_file\n";
        return;
    }
    
    # Extract module name from file path
    my ($module_name) = $module_file =~ /([^\/]+)\.pm$/;
    unless ($module_name) {
        warn "Cannot extract module name from: $module_file\n";
        return;
    }
    
    # Convert to full package name
    $module_name = "Image::ExifTool::$module_name";
    
    # Load the module safely
    eval {
        require $module_file;
        1;
    } or do {
        warn "Failed to load module $module_file: $@\n";
        return;
    };
    
    return $module_name;
}

# Get a reference to a package hash variable
sub get_package_hash {
    my ($module_name, $hash_name) = @_;
    
    # Clean up hash name (remove % prefix if present)
    $hash_name =~ s/^%//;
    
    # Access the package hash using symbolic references
    no strict 'refs';
    my $hash_ref = \%{$module_name . "::" . $hash_name};
    
    # Check if the hash exists and has entries
    unless (%$hash_ref) {
        warn "Hash $hash_name not found or empty in module $module_name\n";
        return;
    }
    
    return $hash_ref;
}

# Extract primitive entries from a loaded hash
sub extract_primitive_entries {
    my ($hash_ref) = @_;
    
    my @entries;
    
    for my $key (keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special ExifTool entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        
        # Only process primitive scalar values
        next unless defined $value && !ref $value;
        
        # Validate key and value are primitive (no variables or complex expressions)
        next if $key =~ /[\$\@\%]/;      # No Perl variables
        next if $value =~ /[\$\@\%]/;    # No variable interpolation
        next if ref $value;              # No references
        
        push @entries, {
            key => $key,
            value => $value,
            source_line => 0,  # Not available with dynamic loading
            raw_line => "$key => '$value'",  # Reconstructed for compatibility
        };
    }
    
    return @entries;
}

# Legacy function for backward compatibility - now calls the new implementation
sub extract_table {
    my ($module_file, $hash_name) = @_;
    return load_and_extract_table($module_file, $hash_name);
}

# Validate table is truly primitive (no Perl logic)
sub validate_primitive_table {
    my @entries = @_;

    for my $entry (@entries) {
        # Check for Perl expressions in keys or values
        return 0 if $entry->{key} =~ /[{}|\[\]\\]/;     # Complex structures
        return 0 if $entry->{value} =~ /\$|\@|%/;       # Variables
        return 0 if $entry->{raw_line} =~ /=>/&&$entry->{raw_line} =~ /[{}]/; # Nested structures
    }

    return 1;
}

# Main extraction logic
sub main {
    my @table_configs = load_table_config();
    my %extracted_data;

    for my $config (@table_configs) {
        my $module_file = "$Bin/../third-party/exiftool/lib/Image/ExifTool/$config->{module}";

        print STDERR "Extracting $config->{hash_name} from $config->{module}...\n";

        my @entries = extract_table($module_file, $config->{hash_name});

        if (!validate_primitive_table(@entries)) {
            warn "WARNING: $config->{hash_name} contains non-primitive data, skipping\n";
            next;
        }

        if (@entries == 0) {
            warn "WARNING: No entries found for $config->{hash_name}\n";
            next;
        }

        $extracted_data{$config->{hash_name}} = {
            config => $config,
            entries => \@entries,
            entry_count => scalar @entries,
        };

        print STDERR "  Extracted " . scalar(@entries) . " entries\n";
    }

    # Output unified JSON
    print encode_json({
        extracted_at => scalar(gmtime()),
        extraction_config => "simple_tables.json",
        total_tables => scalar(keys %extracted_data),
        tables => \%extracted_data,
    });
}

main();