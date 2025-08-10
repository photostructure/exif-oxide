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
use File::Basename;

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

    # Find and convert ONLY package-level 'my %var' or 'my @var' declarations
    # (not ones inside subroutines)
    my @vars_to_export;

    # Split content into lines for analysis
    my @lines = split /\n/, $content;

    for ( my $i = 0 ; $i < @lines ; $i++ ) {
        my $line = $lines[$i];

        # Only collect variables declared at package level
        # Must start at beginning of line with no indentation (^my, not ^\s+my)
        if ( $line =~ /^my\s+([%@])(\w+)\s*=/ ) {
            print STDERR
              "DEBUG: Found package-level variable at line $i: $1$2\n"
              if $ENV{DEBUG};
            push @vars_to_export,
              {
                sigil    => $1,
                name     => $2,
                line_num => $i
              };
        }
    }

    if (@vars_to_export) {
        print STDERR "Converting "
          . scalar(@vars_to_export)
          . " package-level variables from 'my' to 'our'\n"
          if $ENV{DEBUG};

        # Convert only package-level my declarations to our
        # Build a pattern that matches only our specific variables
        my $var_pattern =
          join( '|', map { quotemeta( $_->{name} ) } @vars_to_export );

        # Only match lines with no indentation (^my not ^\s*my)
        $content =~ s/^my(\s+[%@](?:$var_pattern)\s*=)/our$1/gm;
        $modified = 1;

        # Build symbol table export statements
        my $exports =
          "\n# Ensure the field_extractor can see our exported fields:\n";
        foreach my $var (@vars_to_export) {
            my $sigil = $var->{sigil};
            my $name  = $var->{name};
            $exports .= "*$name = \\${sigil}$name;\n";
        }

        # Find insertion point - at the very end of Perl code
        # Either before __END__ if it exists, or at the end of the file
        my $end_marker_pos = index( $content, "\n__END__" );

        my $insert_point;
        if ( $end_marker_pos > -1 ) {

            # Insert before __END__
            $insert_point = $end_marker_pos;
        }
        else {
            # No __END__, append at the very end
            $insert_point = length($content);
        }

        substr( $content, $insert_point, 0, $exports );
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
        close($append_fh);
    }

    return $modified;
}

# Main logic
sub main {

    # Check if already converted
    if ( has_been_converted($module_path) ) {
        print STDERR "DEBUG: Skipping $module_path (already converted)\n"
          if $ENV{DEBUG};
        return;
    }

    # Format with perltidy first (before patching)
    format_file_with_perltidy($module_path);

    # Convert the variables
    my $modified = convert_all_my_to_package_variables($module_path);
}

main();
