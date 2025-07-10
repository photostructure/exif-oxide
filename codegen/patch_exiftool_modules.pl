#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         patch_exiftool_modules.pl
#
# Description:  Convert my-scoped variables to package variables in ExifTool modules
#
# Usage:        perl patch_exiftool_modules.pl <module_path> <variable1> [<variable2> ...]
#
# Example:      perl patch_exiftool_modules.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm canonModelID canonWhiteBalance
#
# Notes:        This script converts specific 'my %variable' declarations to
#               'our %variable' declarations, making them accessible as package
#               variables for the simple table extraction framework.
#------------------------------------------------------------------------------

use strict;
use warnings;

# Check arguments
if (@ARGV < 2) {
    die "Usage: $0 <module_path> <variable1> [<variable2> ...]\n" .
        "Example: $0 third-party/exiftool/lib/Image/ExifTool/Canon.pm canonModelID canonWhiteBalance\n";
}

my $module_path = shift @ARGV;
my @variables_to_convert = @ARGV;

# Validate module path
unless (-f $module_path) {
    die "Error: Module file not found: $module_path\n";
}

# Check if already converted (using a marker comment)
sub has_been_converted {
    my ($module_path) = @_;
    
    # Use grep to search for our conversion marker
    my $result = `grep -q "# EXIF-OXIDE: converted my variables to package variables" "$module_path" 2>/dev/null`;
    return $? == 0;  # grep returns 0 if found
}

# Convert my variables to package variables for specific hashes
sub convert_my_to_package_variables {
    my ($module_path, @variables) = @_;
    
    return unless @variables;
    
    # Read the file
    open(my $fh, '<', $module_path) or die "Cannot read $module_path: $!";
    my $content = do { local $/; <$fh> };
    close($fh);
    
    my $modified = 0;
    for my $var (@variables) {
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
        
        # Add a marker comment to track conversion
        open(my $append_fh, '>>', $module_path) or die "Cannot append to $module_path: $!";
        print $append_fh "\n# EXIF-OXIDE: converted my variables to package variables\n";
        close($append_fh);
    }
    
    return $modified;
}

# Main logic
sub main {
    print "Checking $module_path...\n";
    
    # Check if already converted
    if (has_been_converted($module_path)) {
        print "  Already converted, skipping\n";
        return;
    }
    
    print "  Converting 'my' variables to package variables...\n";
    
    # Convert the variables
    my $modified = convert_my_to_package_variables($module_path, @variables_to_convert);
    
    if ($modified) {
        print "  Successfully converted $module_path\n";
    } else {
        print "  No variables to convert in $module_path\n";
    }
    
    print "Patching complete!\n";
}

main();