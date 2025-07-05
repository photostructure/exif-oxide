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
    my ($module_file, $hash_name, $extraction_type) = @_;

    # Load the module dynamically
    my $module_name = load_module_from_file($module_file);
    return () unless $module_name;

    # Try to access the package hash using symbolic references
    my $hash_ref = get_package_hash($module_name, $hash_name);
    
    if ($hash_ref) {
        # Extract entries based on extraction type
        if ($extraction_type && $extraction_type eq 'regex_strings') {
            return extract_regex_entries($hash_ref);
        } else {
            return extract_primitive_entries($hash_ref);
        }
    } else {
        # Cannot extract this hash - it might be a 'my' scoped variable
        warn "SKIPPED: $hash_name not found as package variable (might be 'my' scoped)\n";
        warn "Try running: make patch-exiftool to convert 'my' variables to package variables\n";
        return ();
    }
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
    if ($module_name eq 'ExifTool') {
        $module_name = "Image::ExifTool";  # Main ExifTool module
    } else {
        $module_name = "Image::ExifTool::$module_name";
    }
    
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


# Extract regex entries from a loaded hash (magic number patterns)
sub extract_regex_entries {
    my ($hash_ref) = @_;
    
    my @entries;
    
    for my $key (keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Only process scalar values (regex patterns)
        next unless defined $value && !ref $value;
        
        # Validate key is a reasonable file type identifier
        next if $key =~ /[\$\@\%]/;      # No Perl variables
        next if $key eq 'Notes';         # Skip ExifTool special entries
        next if $key eq 'OTHER';
        
        # Validate regex pattern syntax for Rust compatibility
        my $rust_compat = validate_regex_for_rust($value);
        if (!$rust_compat->{compatible}) {
            warn "WARNING: Regex pattern for $key may not be compatible with Rust: $rust_compat->{reason}\n";
            warn "  Pattern: $value\n";
            # Still include it but mark the issue
        }
        
        push @entries, {
            key => $key,
            value => $value,
            source_line => 0,  # Not available with dynamic loading
            raw_line => "$key => '$value'",  # Reconstructed for compatibility
            rust_compatible => $rust_compat->{compatible} ? JSON::true : JSON::false,
            compatibility_notes => $rust_compat->{reason},
        };
    }
    
    return @entries;
}

# Validate regex pattern for Rust regex crate compatibility
sub validate_regex_for_rust {
    my ($pattern) = @_;
    
    # Check for features not supported by Rust regex crate
    # Based on research: Rust regex doesn't support lookaround or backreferences
    
    # Check for lookaround assertions
    if ($pattern =~ /\(\?[=!]/) {
        return { compatible => 0, reason => "Contains positive/negative lookahead (?= or (?!)" };
    }
    if ($pattern =~ /\(\?<[=!]/) {
        return { compatible => 0, reason => "Contains positive/negative lookbehind (?<= or (?<!)" };
    }
    
    # Check for backreferences  
    if ($pattern =~ /\\[1-9]/) {
        return { compatible => 0, reason => "Contains numbered backreferences (\\1, \\2, etc.)" };
    }
    if ($pattern =~ /\\g\{?-?\d+\}?/) {
        return { compatible => 0, reason => "Contains named backreferences (\\g1, \\g{-1}, etc.)" };
    }
    
    # Check for other potentially problematic features
    if ($pattern =~ /\(\?[a-z]+:/) {
        # This catches various Perl regex features that might not be supported
        # We'll allow (?i: for case insensitive since that's supported
        if ($pattern !~ /\(\?i:/) {
            return { compatible => 0, reason => "Contains advanced Perl regex features" };
        }
    }
    
    # Pattern appears compatible
    return { compatible => 1, reason => "Pattern appears compatible with Rust regex" };
}



# Legacy function for backward compatibility - now calls the new implementation
sub extract_table {
    my ($module_file, $hash_name, $extraction_type) = @_;
    return load_and_extract_table($module_file, $hash_name, $extraction_type);
}

# Validate table is truly primitive (no Perl logic)
sub validate_primitive_table {
    my @entries = @_;

    for my $entry (@entries) {
        # Check for regex entries (magic number patterns)
        if (exists $entry->{rust_compatible}) {
            # Regex entries are valid if they passed extraction
            # Compatibility warnings are already issued during extraction
            next;
        }
        
        # Check for Perl expressions in keys or values (primitive entries)
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
        # Handle special case for main ExifTool.pm file
        my $module_file;
        if ($config->{module} eq 'ExifTool.pm') {
            $module_file = "$Bin/../third-party/exiftool/lib/Image/ExifTool.pm";
        } else {
            $module_file = "$Bin/../third-party/exiftool/lib/Image/ExifTool/$config->{module}";
        }
        my $extraction_type = $config->{extraction_type} || 'simple_table';

        print STDERR "Extracting $config->{hash_name} from $config->{module} (type: $extraction_type)...\n";

        my @entries = extract_table($module_file, $config->{hash_name}, $extraction_type);

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
            extraction_type => $extraction_type,
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