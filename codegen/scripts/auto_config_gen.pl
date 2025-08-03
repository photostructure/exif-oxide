#!/usr/bin/env perl

#------------------------------------------------------------------------------
# File:         auto_config_gen_fixed.pl
#
# Description:  Generate simple tag_kit.json configs using symbol table introspection
#
# Usage:        perl auto_config_gen_fixed.pl <module_path> [--output=config_dir]
#
# Example:      perl auto_config_gen_fixed.pl third-party/exiftool/lib/Image/ExifTool/DNG.pm
#
# Notes:        This script uses Perl's symbol table to discover tag tables
#               and generates SIMPLE configs (10-20 lines) that specify what
#               to extract, NOT the extracted data itself.
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use File::Path qw(make_path);
use File::Spec;
use JSON;
use Getopt::Long;

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

my $output_dir;
my $help = 0;

GetOptions(
    'output=s' => \$output_dir,
    'help'     => \$help,
) or die "Error in command line arguments\n";

if ( $help || @ARGV != 1 ) {
    print_help();
    exit( $help ? 0 : 1 );
}

my $module_path = $ARGV[0];

# Resolve module path - if just module name given, construct full path
unless ( -f $module_path ) {
    my $constructed_path;

    # Special case for main ExifTool module
    if ( $module_path eq 'ExifTool' ) {
        $constructed_path =
          "$Bin/../../third-party/exiftool/lib/Image/ExifTool.pm";
    }
    else {
# Regular module (e.g., "DNG" -> "third-party/exiftool/lib/Image/ExifTool/DNG.pm")
        $constructed_path =
          "$Bin/../../third-party/exiftool/lib/Image/ExifTool/$module_path.pm";
    }

    if ( -f $constructed_path ) {
        $module_path = $constructed_path;
    }
    else {
        die "Error: Module file not found: $module_path\n";
    }
}

# Extract module info
my ( $module_name, $module_dir, $module_ext ) =
  fileparse( $module_path, qr/\.[^.]*/ );
my $config_dir = $output_dir || "$Bin/../config/${module_name}_pm";

print "Analyzing $module_path...\n";

# Load the module and discover tag tables
my $tables = discover_tag_tables( $module_path, $module_name );

if ( !@$tables ) {
    die "No tag tables found in $module_name\n";
}

print "Found " . scalar(@$tables) . " tag tables:\n";
foreach my $table (@$tables) {
    print "  - $table->{table_name}\n";
}

# Generate simple config
my $config = {
    '$schema'   => "../../schemas/tag_kit.json",
    source      => "third-party/exiftool/lib/Image/ExifTool/${module_name}.pm",
    description => "${module_name} format tag definitions",
    tables      => $tables
};

# Write config file
make_path($config_dir) unless -d $config_dir;
my $config_file = "$config_dir/tag_kit.json";

open my $fh, '>', $config_file or die "Cannot write $config_file: $!";
print $fh JSON->new->pretty->encode($config);
close $fh;

print "Generated simple config: $config_file\n";

sub discover_tag_tables {
    my ( $module_path, $module_name ) = @_;

    # First, ensure the module variables are accessible
    patch_module_if_needed($module_path);

    # Load the module
    eval {
        require $module_path;
        1;
    } or do {
        die "Failed to load module $module_path: $@\n";
    };

    # Get module's symbol table
    my $package_name = "Image::ExifTool::${module_name}";
    no strict 'refs';
    my $symbol_table = *{"${package_name}::"};

    my @tables;

    # Examine each symbol in the package
    foreach my $symbol_name ( sort keys %$symbol_table ) {
        my $glob = $symbol_table->{$symbol_name};
        next unless *$glob{HASH};    # Only hash variables

        my $hash_ref = *$glob{HASH};
        next unless %$hash_ref;      # Skip empty hashes

        # Check if it looks like a tag table
        if ( is_tag_table($hash_ref) ) {
            push @tables,
              {
                table_name  => $symbol_name,
                description =>
                  generate_table_description( $symbol_name, $hash_ref )
              };
        }
    }

    # Sort tables alphabetically by table_name for deterministic output
    @tables = sort { $a->{table_name} cmp $b->{table_name} } @tables;

    return \@tables;
}

sub is_tag_table {
    my ($hash) = @_;

    # Tag tables have specific characteristics:

    # 1. Have GROUPS key (group definitions)
    return 1 if exists $hash->{GROUPS};

    # 2. Have PROCESS_PROC (processing function)
    return 1 if exists $hash->{PROCESS_PROC};

    # 3. Have NOTES (documentation)
    return 1 if exists $hash->{NOTES};

    # 4. Have many keys that look like tag IDs
    my @tag_keys = grep {
        /^(0x[\da-f]+|\d+|[A-Z]\w*)$/i    # Hex, numeric, or CapitalCase
    } keys %$hash;

    # Require at least 5 tag-like keys to consider it a tag table
    return scalar(@tag_keys) >= 5;
}

sub count_tag_entries {
    my ($hash) = @_;

    # Count keys that look like actual tags (not metadata keys)
    my @tag_keys = grep {
        !/^(GROUPS|PROCESS_PROC|WRITE_PROC|CHECK_PROC|NOTES|FORMAT|FIRST_ENTRY|TAG_PREFIX|PRIORITY|WRITABLE|TABLE_DESC|NAMESPACE|PREFERRED|AVOID|LANG_INFO|STRUCT_NAME|SUBDIRECTORY|DEFAULT|DATAMEMBER|EXTRACT_UNKNOWN)$/
    } keys %$hash;

    return scalar(@tag_keys);
}

sub generate_table_description {
    my ( $table_name, $hash ) = @_;

    # Try NOTES (first sentence) then TABLE_DESC
    for my $source ( ( $hash->{NOTES} // '' ) =~ /^([^.]+\.)/ ? $1 : (),
        $hash->{TABLE_DESC} // '' )
    {
        next unless $source;    # Skip empty sources

# Normalize whitespace: trim leading/trailing, collapse all whitespace including newlines
        my $normalized = $source;
        $normalized =~ s/^\s+|\s+$//g;    # trim leading/trailing whitespace
        $normalized =~ s/\s+/ /g
          ;    # collapse all whitespace (including newlines) to single spaces

        return $normalized if $normalized;
    }

    return "$table_name tag definitions";
}

sub patch_module_if_needed {
    my ($module_path) = @_;

    # Check if already patched
    my $result =
`grep -q "# EXIF-OXIDE: converted ALL my variables to package variables" "$module_path" 2>/dev/null`;
    return if $? == 0;    # Already patched

    print "  Patching $module_path to make variables accessible...\n";

    # Run the universal patcher
    my $patcher = "$Bin/patch_exiftool_modules_universal.pl";
    system( "perl", $patcher, $module_path ) == 0
      or die "Failed to patch $module_path\n";
}

sub print_help {
    print <<EOF;
Usage: $0 <module_path> [options]

Options:
    --output=DIR        Output directory for config (default: ../config/ModuleName_pm)
    --help              Show this help message

Examples:
    $0 third-party/exiftool/lib/Image/ExifTool/DNG.pm
    $0 third-party/exiftool/lib/Image/ExifTool/RIFF.pm --output=custom_config_dir

The script generates simple tag_kit.json configurations by:
1. Patching the module to make variables accessible (if needed)
2. Using symbol table introspection to discover tag tables
3. Writing a clean, simple config (10-20 lines) that specifies WHAT to extract
EOF
}
