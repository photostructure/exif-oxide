#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         exiftool-patcher.pl
#
# Description:  Driven by ./exiftool-patcher.sh -- not to be called directly
#
# Notes:        This script converts ALL 'my %variable' and 'my @variable'
#               declarations at package scope to 'our' declarations, making them
#               accessible for symbol table introspection. This enables automated
#               config generation tools to discover tag tables programmatically.
#------------------------------------------------------------------------------
use strict;
use warnings;

my $module_path = $ARGV[0];

# Validate module path
unless ( -f $module_path ) {
    die "Error: Module file not found: $module_path\n";
}

# Check if already converted (using a marker comment)
sub has_been_converted {
    my ($module_path) = @_;
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

    # Find ALL package-level 'my' variable declarations
    my @vars_to_export = ();
    while ( $content =~ /^my\s+([%@])(\w+)\s*=/gm ) {
        my $sigil = $1;
        my $name  = $2;
        push @vars_to_export, { sigil => $sigil, name => $name };
    }

    if (@vars_to_export) {

        # Convert only package-level my declarations to our
        my $var_pattern =
          join( '|', map { quotemeta( $_->{name} ) } @vars_to_export );
        $content =~ s/^my(\s+[%@](?:$var_pattern)\s*=)/our$1/gm;
        $modified = 1;

        # Build symbol table export statements
        my $exports =
          "\n# Ensure the field_extractor can see our exported fields:\n";
        foreach my $var (@vars_to_export) {
            my $sigil = $var->{sigil};
            my $name  = $var->{name};
            $exports .= "*$name = \\$sigil$name;\n";
        }

        # Add conversion marker and exports before the final __END__ or 1; line
        if ( $content =~ s/^(1;\s*#.*end.*|__END__)$/$exports\n\n$1/im ) {

            # Marker added before __END__ or "1; #end"
        }
        else {
            # Fallback: add at very end
            $content .= $exports;
        }
    }

    if ($modified) {

        # Write the modified content back
        open( my $out_fh, '>', $module_path )
          or die "Cannot write $module_path: $!";
        print $out_fh $content;
        close($out_fh);
        return 1;
    }

    return 0;
}

# Main logic
if ( has_been_converted($module_path) ) {
    exit 0;    # Already converted
}

# Convert the variables (perltidy already run)
convert_all_my_to_package_variables($module_path);
