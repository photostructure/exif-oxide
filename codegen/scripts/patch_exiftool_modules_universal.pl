#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         patch_exiftool_modules_universal.pl
#
# Description:  Convert ALL top-level my-scoped hash/array variables to package variables
#
# Usage:        perl patch_exiftool_modules_universal.pl <module_path>
#
# Example:      perl patch_exiftool_modules_universal.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm
#
# Notes:        This script converts ALL 'my %variable' and 'my @variable'
#               declarations at package scope to 'our' declarations, making them
#               accessible for symbol table introspection. This enables automated
#               config generation tools to discover tag tables programmatically.
#------------------------------------------------------------------------------

use strict;
use warnings;

# Check arguments
if ( @ARGV != 1 ) {
    die "Usage: $0 <module_path>\n"
      . "Example: $0 third-party/exiftool/lib/Image/ExifTool/Canon.pm\n";
}

my $module_path = $ARGV[0];

# Validate module path
unless ( -f $module_path ) {
    die "Error: Module file not found: $module_path\n";
}

# Check if already converted (using a marker comment)
sub has_been_converted {
    my ($module_path) = @_;

    # Use grep to search for our conversion marker
    my $result =
`grep -q "# EXIF-OXIDE: converted ALL my variables to package variables" "$module_path" 2>/dev/null`;
    return $? == 0;    # grep returns 0 if found
}

# Convert ALL top-level my variables to package variables
sub convert_all_my_to_package_variables {
    my ($module_path) = @_;

    # Read the file
    open( my $fh, '<', $module_path ) or die "Cannot read $module_path: $!";
    my $content = do { local $/; <$fh> };
    close($fh);

    my $modified = 0;
    my @converted_vars;

    # Find all top-level 'my %var' or 'my @var' declarations first
    # (before we start modifying the content)
    while ( $content =~ /^(\s*)my\s+([%@])(\w+)\s*=/gm ) {
        my $sigil = $2;
        my $name = $3;
        push @converted_vars, [$sigil, $name];  # Store [sigil, name] as array ref
    }

    # Now perform the conversions
    if (@converted_vars) {
        # Convert all my to our
        $content =~ s/^(\s*)my(\s+[%@]\w+\s*=)/$1our$2/gm;
        $modified = 1;
        
        # Add symbol table aliases at the end of the package
        # Find where to insert (before the final "1;" or at end)
        my $insert_point = rindex($content, "\n1;");
        
        my $aliases = "\n# Ensure the field_extractor can see our exported fields:\n";
        foreach my $var (@converted_vars) {
            # Ensure we have an array ref
            if (ref($var) ne 'ARRAY') {
                die "Expected array ref but got: " . (ref($var) || "scalar value '$var'") . "\n";
            }
            my ($sigil, $name) = @$var;
            if ($sigil eq '%') {
                $aliases .= "*$name = \\%$name;\n";
            } elsif ($sigil eq '@') {
                $aliases .= "*$name = \\@$name;\n";
            }
        }
        
        if ($insert_point > -1) {
            # Insert before the "1;"
            substr($content, $insert_point, 0, $aliases);
        } else {
            # No "1;" found, append at end
            $content .= $aliases;
        }
    }

   # Add marker for AddCompositeTags calls
   # This helps our field_extractor identify which modules have composite tables
    if ( $content =~
s/^(Image::ExifTool::AddCompositeTags\('Image::ExifTool::\w+'\);)$/our \$__hasCompositeTags = 1; $1/gm
      )
    {
        $modified = 1;
    }

    if ($modified) {

        # Write back the modified content
        open( my $out_fh, '>', $module_path )
          or die "Cannot write to $module_path: $!";
        print $out_fh $content;
        close($out_fh);

        # Add a marker comment to track conversion
        open( my $append_fh, '>>', $module_path )
          or die "Cannot append to $module_path: $!";
        print $append_fh
          "\n# EXIF-OXIDE: converted ALL my variables to package variables\n";
        close($append_fh);
    }

    return $modified;
}

# Main logic
sub main {

    # Check if already converted
    if ( has_been_converted($module_path) ) {
        return;
    }

    # Convert the variables
    my $modified = convert_all_my_to_package_variables($module_path);
}

main();
