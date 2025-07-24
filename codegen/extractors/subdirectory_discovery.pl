#!/usr/bin/env perl
#------------------------------------------------------------------------------
# File:         subdirectory_discovery.pl
#
# Description:  Discovers all SubDirectory patterns across ExifTool modules
#               and checks implementation status in exif-oxide
#
# Usage:        perl subdirectory_discovery.pl [--json|--markdown]
#
# Notes:        Dynamically loads ExifTool modules to find SubDirectory refs
#------------------------------------------------------------------------------

use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../../third-party/exiftool/lib";
use File::Find;
use File::Basename;
use JSON;
use Data::Dumper;
use Getopt::Long;

# Command line options
my $output_json = 0;
my $output_markdown = 0;
my $verbose = 0;
GetOptions(
    'json'     => \$output_json,
    'markdown' => \$output_markdown,
    'verbose'  => \$verbose,
) or die "Usage: $0 [--json] [--markdown] [--verbose]\n";

# Default to JSON if nothing specified
$output_json = 1 unless $output_markdown;

# Global data structures
my %subdirectory_data = ();
my %module_stats = ();
my %pattern_stats = (
    simple => 0,
    conditional => 0,
    binary_data => 0,
    validated => 0,
    process_proc => 0,
);
my $total_subdirs = 0;
my $implemented_subdirs = 0;

# ExifTool library path
my $exiftool_lib = "$FindBin::Bin/../../third-party/exiftool/lib/Image/ExifTool";

# exif-oxide paths
my $codegen_config = "$FindBin::Bin/../config";
my $generated_src = "$FindBin::Bin/../../src/generated";

#------------------------------------------------------------------------------
# Find all ExifTool module files
#------------------------------------------------------------------------------
sub find_exiftool_modules {
    my @modules;
    
    find(sub {
        return unless -f && /\.pm$/;
        return if /^(BuildTagLookup|WriteCanonRaw|WritePostScript|WritePDF|Validate)\.pm$/;
        push @modules, $File::Find::name;
    }, $exiftool_lib);
    
    return @modules;
}

#------------------------------------------------------------------------------
# Load a module and get its symbol table
#------------------------------------------------------------------------------
sub load_module {
    my $module_file = shift;
    
    # Convert file path to module name
    my $module = $module_file;
    $module =~ s/.*?Image/Image/;
    $module =~ s/\.pm$//;
    $module =~ s/\//::/g;
    
    # Skip if already loaded
    return $module if $INC{$module_file};
    
    # Load the module
    eval "require $module";
    if ($@) {
        warn "Failed to load $module: $@\n" if $verbose;
        return;
    }
    
    return $module;
}

#------------------------------------------------------------------------------
# Check if a table uses binary data attributes
#------------------------------------------------------------------------------
sub is_binary_data_table {
    my ($table_ref) = @_;
    
    return 0 unless ref($table_ref) eq 'HASH';
    
    # Check for %binaryDataAttrs inheritance
    # In perl, this is indicated by presence of PROCESS_PROC => \&ProcessBinaryData
    # or by having FORMAT and FIRST_ENTRY keys
    return 1 if exists $table_ref->{PROCESS_PROC} && 
                ref($table_ref->{PROCESS_PROC}) eq 'CODE' &&
                $table_ref->{PROCESS_PROC} == \&Image::ExifTool::ProcessBinaryData;
    
    return 1 if exists $table_ref->{FORMAT} && exists $table_ref->{FIRST_ENTRY};
    
    return 0;
}

#------------------------------------------------------------------------------
# Classify SubDirectory pattern type
#------------------------------------------------------------------------------
sub classify_pattern {
    my ($tag_ref, $table_ref) = @_;
    
    # Check for validation
    if (ref($tag_ref) eq 'HASH' && exists $tag_ref->{SubDirectory}) {
        my $subdir = $tag_ref->{SubDirectory};
        return 'validated' if exists $subdir->{Validate};
        return 'process_proc' if exists $subdir->{ProcessProc};
    }
    
    # Check if target table is binary data
    if ($table_ref && is_binary_data_table($table_ref)) {
        return 'binary_data';
    }
    
    # Check for conditional (array of definitions)
    if (ref($tag_ref) eq 'ARRAY') {
        return 'conditional';
    }
    
    return 'simple';
}

#------------------------------------------------------------------------------
# Extract SubDirectory info from a tag definition
#------------------------------------------------------------------------------
sub extract_subdirectory_info {
    my ($tag_id, $tag_def, $module_name, $table_name) = @_;
    
    my @subdirs;
    
    # Handle array of conditional tags
    if (ref($tag_def) eq 'ARRAY') {
        foreach my $item (@$tag_def) {
            next unless ref($item) eq 'HASH' && exists $item->{SubDirectory};
            
            my $info = {
                tag_id => $tag_id,
                tag_name => $item->{Name} || 'Unknown',
                condition => $item->{Condition},
                subdirectory => $item->{SubDirectory},
                source_module => $module_name,
                source_table => $table_name,
            };
            
            push @subdirs, $info;
        }
    }
    # Handle single tag definition
    elsif (ref($tag_def) eq 'HASH' && exists $tag_def->{SubDirectory}) {
        my $info = {
            tag_id => $tag_id,
            tag_name => $tag_def->{Name} || 'Unknown',
            subdirectory => $tag_def->{SubDirectory},
            source_module => $module_name,
            source_table => $table_name,
        };
        
        push @subdirs, $info;
    }
    
    return @subdirs;
}

#------------------------------------------------------------------------------
# Process all tags in a table looking for SubDirectory references
#------------------------------------------------------------------------------
sub process_table {
    my ($table_ref, $table_name, $module_name) = @_;
    
    return unless ref($table_ref) eq 'HASH';
    
    my @found_subdirs;
    
    foreach my $tag_id (keys %$table_ref) {
        # Skip special keys
        next if $tag_id =~ /^(PROCESS_PROC|WRITE_PROC|CHECK_PROC|GROUPS|NOTES|FORMAT|FIRST_ENTRY|VARS|DATAMEMBER|NAMESPACE|PREFERRED|TABLE_NAME|SHORT_NAME|DID_TAG_ID|AVOID|PRIORITY)$/;
        
        my $tag_def = $table_ref->{$tag_id};
        
        # Extract SubDirectory info
        my @subdirs = extract_subdirectory_info($tag_id, $tag_def, $module_name, $table_name);
        push @found_subdirs, @subdirs if @subdirs;
    }
    
    return @found_subdirs;
}

#------------------------------------------------------------------------------
# Get all table references from a module
#------------------------------------------------------------------------------
sub get_module_tables {
    my $module_name = shift;
    
    my %tables;
    
    # Get the module's symbol table
    no strict 'refs';
    my $symtab = \%{$module_name . '::'};
    use strict 'refs';
    
    # Find all hash variables (potential tag tables)
    foreach my $symbol (keys %$symtab) {
        next unless $symbol =~ /^[A-Z]/;  # Tag tables usually start with uppercase
        
        no strict 'refs';
        my $glob = $symtab->{$symbol};
        my $hash_ref = *{$glob}{HASH};
        use strict 'refs';
        
        next unless $hash_ref && %$hash_ref;
        
        # Check if this looks like a tag table
        # (has numeric or hex keys, or typical tag table keys)
        my $looks_like_tag_table = 0;
        foreach my $key (keys %$hash_ref) {
            if ($key =~ /^(0x[0-9a-fA-F]+|\d+|GROUPS|PROCESS_PROC|NOTES|FORMAT)$/) {
                $looks_like_tag_table = 1;
                last;
            }
        }
        
        $tables{$symbol} = $hash_ref if $looks_like_tag_table;
    }
    
    return %tables;
}

#------------------------------------------------------------------------------
# Check if a SubDirectory is implemented in exif-oxide
#------------------------------------------------------------------------------
sub check_implementation {
    my ($subdir_info) = @_;
    
    my $module = $subdir_info->{source_module};
    $module =~ s/.*:://;  # Get just the module name
    
    my @config_files;
    my @generated_files;
    my $is_implemented = 0;
    
    # Check for config files
    my @config_patterns = (
        "$codegen_config/${module}_pm/tag_kit.json",
        "$codegen_config/${module}_pm/subdirectory.json",
        "$codegen_config/${module}_pm/process_binary_data.json",
        "$codegen_config/${module}_pm/conditional_tags.json",
    );
    
    foreach my $config (@config_patterns) {
        if (-f $config) {
            # Read file and check if it mentions this tag or table
            open my $fh, '<', $config or next;
            my $content = do { local $/; <$fh> };
            close $fh;
            
            my $tag_hex = sprintf("0x%x", $subdir_info->{tag_id}) if $subdir_info->{tag_id} =~ /^\d+$/;
            my $tag_str = $subdir_info->{tag_id};
            
            if ($content =~ /$tag_str/i || ($tag_hex && $content =~ /$tag_hex/i)) {
                push @config_files, $config;
                $is_implemented = 1;
            }
        }
    }
    
    # Check for generated files
    if (-d "$generated_src/${module}_pm") {
        my @gen_files = glob("$generated_src/${module}_pm/*.rs");
        foreach my $gen_file (@gen_files) {
            open my $fh, '<', $gen_file or next;
            my $content = do { local $/; <$fh> };
            close $fh;
            
            if ($subdir_info->{tag_id} =~ /^\d+$/) {
                my $tag_hex = sprintf("0x%04x", $subdir_info->{tag_id});
                if ($content =~ /$tag_hex/i || $content =~ /$subdir_info->{tag_name}/i) {
                    push @generated_files, $gen_file;
                    $is_implemented = 1;
                }
            } elsif ($content =~ /$subdir_info->{tag_name}/i) {
                push @generated_files, $gen_file;
                $is_implemented = 1;
            }
        }
    }
    
    return {
        is_implemented => $is_implemented,
        config_files => \@config_files,
        generated_files => \@generated_files,
    };
}

#------------------------------------------------------------------------------
# Main processing
#------------------------------------------------------------------------------

print STDERR "Finding ExifTool modules...\n" if $verbose;
my @modules = find_exiftool_modules();
print STDERR "Found " . scalar(@modules) . " modules\n" if $verbose;

foreach my $module_file (@modules) {
    my $module_name = load_module($module_file);
    next unless $module_name;
    
    print STDERR "Processing $module_name...\n" if $verbose;
    
    # Get module short name for stats
    my $module_short = $module_name;
    $module_short =~ s/.*:://;
    
    # Get all tables from the module
    my %tables = get_module_tables($module_name);
    
    # Process each table
    foreach my $table_name (keys %tables) {
        my $table_ref = $tables{$table_name};
        my @subdirs = process_table($table_ref, $table_name, $module_name);
        
        foreach my $subdir (@subdirs) {
            $total_subdirs++;
            
            # Get referenced table name
            my $ref_table = $subdir->{subdirectory}{TagTable} || 'Unknown';
            
            # Try to load the referenced table
            my $target_table_ref;
            if ($ref_table && $ref_table ne 'Unknown') {
                no strict 'refs';
                $target_table_ref = \%{$ref_table};
                use strict 'refs';
            }
            
            # Classify the pattern
            my $pattern_type = classify_pattern($subdir, $target_table_ref);
            $pattern_stats{$pattern_type}++;
            
            # Check implementation status
            my $impl_status = check_implementation($subdir);
            $implemented_subdirs++ if $impl_status->{is_implemented};
            
            # Store the data
            $module_stats{$module_short}{total}++;
            $module_stats{$module_short}{implemented}++ if $impl_status->{is_implemented};
            
            push @{$subdirectory_data{$module_short}}, {
                tag_id => $subdir->{tag_id},
                tag_name => $subdir->{tag_name},
                pattern_type => $pattern_type,
                condition => $subdir->{condition},
                referenced_table => $ref_table,
                is_binary_data => is_binary_data_table($target_table_ref),
                implementation_status => $impl_status->{is_implemented} ? 'implemented' : 'missing',
                config_files => $impl_status->{config_files},
                generated_files => $impl_status->{generated_files},
            };
        }
    }
}

#------------------------------------------------------------------------------
# Generate output
#------------------------------------------------------------------------------

if ($output_json) {
    my $coverage_pct = $total_subdirs > 0 ? 
        sprintf("%.2f", ($implemented_subdirs / $total_subdirs) * 100) : 0;
    
    my $output = {
        scan_metadata => {
            timestamp => scalar(gmtime) . " UTC",
            modules_scanned => scalar(@modules),
        },
        summary => {
            total_subdirectories => $total_subdirs,
            implemented => $implemented_subdirs,
            missing => $total_subdirs - $implemented_subdirs,
            coverage_percentage => $coverage_pct,
        },
        pattern_statistics => \%pattern_stats,
        by_module => {},
    };
    
    # Add module data
    foreach my $module (sort keys %subdirectory_data) {
        $output->{by_module}{$module} = {
            total => $module_stats{$module}{total} || 0,
            implemented => $module_stats{$module}{implemented} || 0,
            subdirectories => $subdirectory_data{$module},
        };
    }
    
    print JSON->new->pretty->encode($output);
}
elsif ($output_markdown) {
    # Generate markdown report
    print "# ExifTool SubDirectory Coverage Report\n\n";
    print "Generated: " . scalar(gmtime) . " UTC\n\n";
    
    print "## Summary\n\n";
    printf "- **Total SubDirectories**: %d\n", $total_subdirs;
    printf "- **Implemented**: %d\n", $implemented_subdirs;
    printf "- **Missing**: %d\n", $total_subdirs - $implemented_subdirs;
    printf "- **Coverage**: %.2f%%\n\n", ($implemented_subdirs / $total_subdirs) * 100;
    
    print "## Pattern Distribution\n\n";
    print "| Pattern Type | Count | Percentage |\n";
    print "|--------------|-------|------------|\n";
    foreach my $type (sort keys %pattern_stats) {
        printf "| %-12s | %5d | %9.1f%% |\n", 
            ucfirst($type), 
            $pattern_stats{$type},
            ($pattern_stats{$type} / $total_subdirs) * 100;
    }
    
    print "\n## Module Breakdown\n\n";
    print "| Module | Total | Implemented | Coverage |\n";
    print "|--------|-------|-------------|----------|\n";
    
    foreach my $module (sort { $module_stats{$b}{total} <=> $module_stats{$a}{total} } keys %module_stats) {
        my $total = $module_stats{$module}{total} || 0;
        my $impl = $module_stats{$module}{implemented} || 0;
        my $pct = $total > 0 ? ($impl / $total) * 100 : 0;
        
        printf "| %-20s | %5d | %11d | %7.1f%% |\n", $module, $total, $impl, $pct;
    }
    
    print "\n## Missing High-Priority SubDirectories\n\n";
    print "Top 20 unimplemented SubDirectories by module:\n\n";
    
    my $count = 0;
    foreach my $module (sort { $module_stats{$b}{total} <=> $module_stats{$a}{total} } keys %subdirectory_data) {
        my @missing = grep { $_->{implementation_status} eq 'missing' } @{$subdirectory_data{$module}};
        next unless @missing;
        
        print "### $module\n\n";
        foreach my $subdir (@missing[0..4]) {  # First 5 per module
            printf "- **%s** (%s): %s pattern, references `%s`\n",
                $subdir->{tag_name},
                $subdir->{tag_id},
                $subdir->{pattern_type},
                $subdir->{referenced_table};
            
            last if ++$count >= 20;
        }
        print "\n";
        last if $count >= 20;
    }
}