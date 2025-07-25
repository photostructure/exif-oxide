#!/usr/bin/env perl
#
# Cross-Module SubDirectory Reference Analyzer
# 
# This script analyzes subdirectory references to find cross-module dependencies
# and suggests solutions for the tag kit code generation.
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

my %cross_module_refs;  # source_module -> target_module -> [table_names]
my %missing_tables;     # tables referenced but not defined in their module
my %table_definitions;  # module -> table_name -> 1

# First pass: Find all table definitions
print STDERR "First pass: Finding table definitions...\n";
find_table_definitions();

# Second pass: Find cross-module references
print STDERR "Second pass: Finding cross-module references...\n";
find_cross_module_refs();

# Analyze and output results
analyze_results();

sub find_table_definitions {
    my @pm_files;
    find(
        sub {
            push @pm_files, $File::Find::name if /\.pm$/ && !/Test/ && !/BuildTagLookup/;
        },
        '../../third-party/exiftool/lib/Image/ExifTool'
    );
    
    foreach my $pm_file (@pm_files) {
        my $module_name = extract_module_name($pm_file);
        
        open my $fh, '<', $pm_file or die "Can't open $pm_file: $!";
        my $content = do { local $/; <$fh> };
        close $fh;
        
        # Find table definitions like %Image::ExifTool::Canon::ShotInfo
        while ($content =~ /
            %Image::ExifTool::(\w+)::(\w+)\s*=\s*\(
        /gx) {
            my $def_module = $1;
            my $table_name = $2;
            $table_definitions{$def_module}{$table_name} = 1;
        }
    }
}

sub find_cross_module_refs {
    my @pm_files;
    find(
        sub {
            push @pm_files, $File::Find::name if /\.pm$/ && !/Test/ && !/BuildTagLookup/;
        },
        '../../third-party/exiftool/lib/Image/ExifTool'
    );
    
    foreach my $pm_file (@pm_files) {
        scan_module_for_refs($pm_file);
    }
}

sub scan_module_for_refs {
    my ($pm_file) = @_;
    
    my $source_module = extract_module_name($pm_file);
    
    open my $fh, '<', $pm_file or die "Can't open $pm_file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;
    
    # Find all SubDirectory references
    while ($content =~ /
        SubDirectory\s*=>\s*\{
        (.*?)
        \}
    /sgx) {
        my $subdir_block = $1;
        
        # Extract TagTable reference
        if ($subdir_block =~ /TagTable\s*=>\s*['"]Image::ExifTool::(\w+)::(\w+)['"]/) {
            my $target_module = $1;
            my $table_name = $2;
            
            # Check if this is a cross-module reference
            if ($target_module ne $source_module) {
                push @{$cross_module_refs{$source_module}{$target_module}}, 
                     "Image::ExifTool::${target_module}::${table_name}";
                
                # Check if table is defined in target module
                unless (exists $table_definitions{$target_module}{$table_name}) {
                    $missing_tables{"Image::ExifTool::${target_module}::${table_name}"} = 1;
                }
            }
        }
    }
}

sub extract_module_name {
    my ($pm_file) = @_;
    my $module_name = $pm_file;
    $module_name =~ s{.*/Image/ExifTool/}{};
    $module_name =~ s/\.pm$//;
    $module_name =~ s{/}{::}g;
    return $module_name;
}

sub analyze_results {
    # Sort results for deterministic output
    my %sorted_refs;
    foreach my $source (sort keys %cross_module_refs) {
        foreach my $target (sort keys %{$cross_module_refs{$source}}) {
            my @unique_tables = do {
                my %seen;
                grep { !$seen{$_}++ } @{$cross_module_refs{$source}{$target}};
            };
            @unique_tables = sort @unique_tables;
            $sorted_refs{$source}{$target} = \@unique_tables;
        }
    }
    
    my @missing_sorted = sort keys %missing_tables;
    
    # Count statistics
    my $total_cross_refs = 0;
    foreach my $source (keys %sorted_refs) {
        foreach my $target (keys %{$sorted_refs{$source}}) {
            $total_cross_refs += @{$sorted_refs{$source}{$target}};
        }
    }
    
    # Output JSON report
    my $json = JSON->new->utf8->pretty->canonical(1);  # canonical for deterministic output
    print $json->encode({
        summary => {
            total_cross_module_refs => $total_cross_refs,
            source_modules => scalar(keys %sorted_refs),
            missing_tables => scalar(@missing_sorted),
        },
        cross_module_refs => \%sorted_refs,
        missing_tables => \@missing_sorted,
        recommendations => {
            extract_from_modules => [sort keys %{{ map { $_ => 1 } map { keys %$_ } values %sorted_refs }}],
            priority_modules => find_priority_modules(\%sorted_refs),
        }
    });
}

sub find_priority_modules {
    my ($refs) = @_;
    
    # Count how many times each module is referenced
    my %ref_count;
    foreach my $source (keys %$refs) {
        foreach my $target (keys %{$refs->{$source}}) {
            $ref_count{$target} += @{$refs->{$source}{$target}};
        }
    }
    
    # Sort by reference count
    my @priority = sort { $ref_count{$b} <=> $ref_count{$a} } keys %ref_count;
    
    # Return top 10 with counts
    my @result;
    foreach my $module (@priority[0..9]) {
        last unless defined $module;
        push @result, {
            module => $module,
            reference_count => $ref_count{$module},
        };
    }
    
    return \@result;
}