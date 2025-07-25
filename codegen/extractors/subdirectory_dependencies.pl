#!/usr/bin/env perl
#
# SubDirectory Dependency Scanner
# 
# This script scans ExifTool modules to build a dependency graph of subdirectory references.
# It identifies which tables reference which other tables across modules.
#

use strict;
use warnings;
use lib '../../third-party/exiftool/lib';
use File::Find;
use JSON;
use Data::Dumper;

# Configure output
$Data::Dumper::Sortkeys = 1;
$Data::Dumper::Indent = 1;

my %dependencies;  # module -> { table -> [referenced_tables] }
my %all_tables;    # track all discovered tables

# Find all Perl modules
my @pm_files;
find(
    sub {
        push @pm_files, $File::Find::name if /\.pm$/ && !/Test/ && !/BuildTagLookup/;
    },
    '../../third-party/exiftool/lib/Image/ExifTool'
);

# Scan each module
foreach my $pm_file (@pm_files) {
    scan_module($pm_file);
}

# Output dependency graph
my $json = JSON->new->utf8->pretty;
print $json->encode({
    dependencies => \%dependencies,
    all_tables => \%all_tables,
    stats => {
        total_modules => scalar(keys %dependencies),
        total_tables => scalar(keys %all_tables),
        cross_module_refs => count_cross_module_refs(),
    }
});

sub scan_module {
    my ($pm_file) = @_;
    
    # Extract module name from path
    my $module_name = $pm_file;
    $module_name =~ s{.*/Image/ExifTool/}{};
    $module_name =~ s/\.pm$//;
    $module_name =~ s{/}{::}g;
    
    open my $fh, '<', $pm_file or die "Can't open $pm_file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;
    
    # Find all SubDirectory references
    while ($content =~ /
        SubDirectory\s*=>\s*\{
        .*?
        TagTable\s*=>\s*['"]([^'"]+)['"]
        .*?
        \}
    /sgx) {
        my $referenced_table = $1;
        
        # Extract the current tag context (rough approximation)
        my $tag_context = "tag_in_$module_name";
        
        # Record dependency
        $dependencies{$module_name}{$tag_context} ||= [];
        push @{$dependencies{$module_name}{$tag_context}}, $referenced_table;
        
        # Track all tables
        $all_tables{$referenced_table} = 1;
        
        # Parse module from table reference
        if ($referenced_table =~ /^Image::ExifTool::(\w+)::/) {
            my $ref_module = $1;
            # Mark this as a referenced module
            $all_tables{"$ref_module (module)"} = 1;
        }
    }
}

sub count_cross_module_refs {
    my $count = 0;
    
    foreach my $module (keys %dependencies) {
        foreach my $tag (keys %{$dependencies{$module}}) {
            foreach my $ref_table (@{$dependencies{$module}{$tag}}) {
                # Check if reference is to a different module
                if ($ref_table =~ /^Image::ExifTool::(\w+)::/ && $1 ne $module) {
                    $count++;
                }
            }
        }
    }
    
    return $count;
}