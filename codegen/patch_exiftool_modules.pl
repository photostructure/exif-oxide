#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         patch_exiftool_modules.pl
#
# Description:  Convert my-scoped variables to package variables in ExifTool modules
#
# Usage:        perl patch_exiftool_modules.pl [module1.pm module2.pm ...]
#               perl patch_exiftool_modules.pl  # patches all modules in config
#
# Notes:        This script converts specific 'my %variable' declarations to
#               'our %variable' declarations, making them accessible as package
#               variables for the simple table extraction framework.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use JSON qw(decode_json);

# Convert my variables to package variables for specific hashes
sub convert_my_to_package_variables {
    my ($module_path, $module_name) = @_;
    
    # Define which variables to convert based on our config
    my @variables_to_convert;
    if ($module_name eq 'Canon.pm') {
        @variables_to_convert = qw(canonQuality canonImageSize canonWhiteBalance pictureStyles);
    } elsif ($module_name eq 'Nikon.pm') {
        @variables_to_convert = qw(nikonLensIDs);
    }
    
    return unless @variables_to_convert;
    
    # Read the file
    open(my $fh, '<', $module_path) or die "Cannot read $module_path: $!";
    my $content = do { local $/; <$fh> };
    close($fh);
    
    my $modified = 0;
    for my $var (@variables_to_convert) {
        # Convert "my %varName =" to "our %varName ="
        if ($content =~ s/^(\s*)my(\s+%$var\s*=)/$1our$2/gm) {
            print STDERR "  Converted 'my %$var' to 'our %$var'\n";
            $modified = 1;
        }
    }
    
    if ($modified) {
        # Write back the modified content
        open(my $out_fh, '>', $module_path) or die "Cannot write to $module_path: $!";
        print $out_fh $content;
        close($out_fh);
    }
    
    return $modified;
}

# Read configuration to get list of modules to patch
sub load_table_config {
    my $config_file = "$Bin/simple_tables.json";
    
    open(my $fh, '<', $config_file) or die "Cannot open $config_file: $!";
    my $json_text = do { local $/; <$fh> };
    close($fh);
    
    my $config = decode_json($json_text);
    return @{$config->{tables}};
}

# Get unique list of modules that need patching
sub get_modules_to_patch {
    my @table_configs = load_table_config();
    my %modules_seen;
    
    for my $config (@table_configs) {
        $modules_seen{$config->{module}} = 1;
    }
    
    return keys %modules_seen;
}

# Check if a module has already been converted (using a marker comment)
sub has_been_converted {
    my ($module_path) = @_;
    
    # Use grep to search for our conversion marker
    my $result = `grep -q "# EXIF-OXIDE: converted my variables to package variables" "$module_path" 2>/dev/null`;
    return $? == 0;  # grep returns 0 if found
}


# Convert my variables to package variables in a module file
sub convert_module {
    my ($module_path, $module_name) = @_;
    
    print "Checking $module_path...\n";
    
    # Check if already converted
    if (has_been_converted($module_path)) {
        print "  Already converted, skipping\n";
        return;
    }
    
    print "  Converting 'my' variables to package variables...\n";
    
    # Convert the variables
    my $modified = convert_my_to_package_variables($module_path, $module_name);
    
    if ($modified) {
        # Add a marker comment to track conversion
        open(my $fh, '>>', $module_path) or die "Cannot append to $module_path: $!";
        print $fh "\n# EXIF-OXIDE: converted my variables to package variables\n";
        close($fh);
        
        print "  Successfully converted $module_path\n";
    } else {
        print "  No variables to convert in $module_path\n";
    }
}

# Main logic
sub main {
    my @modules_to_patch;
    
    if (@ARGV) {
        # Use command line arguments
        @modules_to_patch = @ARGV;
    } else {
        # Use modules from configuration
        @modules_to_patch = get_modules_to_patch();
    }
    
    my $exiftool_lib_path = "$Bin/../third-party/exiftool/lib/Image/ExifTool";
    
    for my $module_name (@modules_to_patch) {
        my $module_path = "$exiftool_lib_path/$module_name";
        
        unless (-f $module_path) {
            warn "Module not found: $module_path\n";
            next;
        }
        
        convert_module($module_path, $module_name);
    }
    
    print "Patching complete!\n";
}

main();