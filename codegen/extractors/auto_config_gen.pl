#!/usr/bin/env perl
#
# Semi-Automated Config Generation Script
#
# This script analyzes ExifTool modules and automatically generates tag_kit.json
# configurations for high-priority modules identified by the coverage dashboard.
# It focuses on low-hanging fruit with simple conditions and high subdirectory counts.
#
# Usage: ./auto_config_gen.pl [--module=ModuleName] [--dry-run] [--force] [--priority=high|medium]
#

use strict;
use warnings;
use lib '../../third-party/exiftool/lib';
use File::Find;
use JSON;
use Data::Dumper;
use Getopt::Long;
use File::Basename;
use File::Path qw(make_path);

# Configure output
$Data::Dumper::Sortkeys = 1;
$Data::Dumper::Indent   = 1;

my $target_module   = '';
my $dry_run         = 0;
my $force           = 0;
my $priority_filter = 'high';    # Default to high priority modules
my $help            = 0;

GetOptions(
    'module=s'   => \$target_module,
    'dry-run'    => \$dry_run,
    'force'      => \$force,
    'priority=s' => \$priority_filter,
    'help'       => \$help,
) or die "Error in command line arguments\n";

if ($help) {
    print_help();
    exit 0;
}

# Load coverage data and required tags
print STDERR "Loading coverage data...\n";
my %coverage_data = load_coverage_data();

print STDERR "Loading required tag information...\n";
my %required_tags = load_required_tags();

# Get priority modules
my @target_modules;
if ($target_module) {
    @target_modules = ($target_module);
}
else {
    @target_modules = get_priority_modules( $priority_filter, \%coverage_data,
        \%required_tags );
}

print STDERR "Target modules for config generation: "
  . join( ", ", @target_modules ) . "\n";

# Process each target module
foreach my $module_name (@target_modules) {
    print STDERR "\n=== Processing $module_name ===\n";

    my $config_dir  = "../config/${module_name}_pm";
    my $config_file = "$config_dir/tag_kit.json";

    # Check if config already exists
    if ( -f $config_file && !$force ) {
        print STDERR
"Config already exists for $module_name, skipping (use --force to overwrite)\n";
        next;
    }

    # Generate config for this module
    my $config = generate_module_config( $module_name, \%coverage_data );

    if ( !$config ) {
        print STDERR "Failed to generate config for $module_name\n";
        next;
    }

    if ($dry_run) {
        print STDERR "DRY RUN: Would create $config_file\n";
        print encode_json($config);
        print "\n";
    }
    else {
        # Create directory and write config
        make_path($config_dir) unless -d $config_dir;

        open my $fh, '>', $config_file or die "Cannot write $config_file: $!";
        print $fh encode_json($config);
        close $fh;

        print STDERR "Generated $config_file\n";
    }
}

print STDERR "\nConfig generation complete!\n";

sub load_coverage_data {
    my %data;

    # Run coverage dashboard and parse output
    my $dashboard_output =
      `perl ./coverage_dashboard.pl --output=json 2>/dev/null`;

    if ($dashboard_output) {
        eval {
            my $json_data = decode_json($dashboard_output);
            %data = %{ $json_data->{modules} } if $json_data->{modules};
        };

        if ($@) {
            print STDERR
              "Warning: Could not parse coverage dashboard output: $@\n";
        }
    }

    return %data;
}

sub load_required_tags {
    my %required_tags;

    # Load tag metadata to find required tags
    eval {
        my $tag_metadata_file = '../../docs/tag-metadata.json';
        if ( -f $tag_metadata_file ) {
            open my $fh, '<', $tag_metadata_file or die $!;
            my $json_text = do { local $/; <$fh> };
            close $fh;

            my $metadata = decode_json($json_text);
            foreach my $tag_name ( keys %$metadata ) {
                my $tag_info = $metadata->{$tag_name};
                if ( $tag_info->{required} ) {

                    # Map tag to its groups (modules)
                    foreach my $group ( @{ $tag_info->{groups} } ) {
                        push @{ $required_tags{$group} }, $tag_name;
                    }
                }
            }
        }
    };

    if ($@) {
        print STDERR "Warning: Could not load required tags: $@\n";
    }

    return %required_tags;
}

sub get_priority_modules {
    my ( $priority, $coverage_ref, $required_tags_ref ) = @_;
    my @modules;

    # Calculate priority for each module and filter
    foreach my $module_name ( keys %$coverage_ref ) {
        my $module          = $coverage_ref->{$module_name};
        my $module_priority = calculate_module_priority( $module_name, $module,
            $required_tags_ref );
        my $priority_label = get_priority_label($module_priority);

        # Filter by requested priority
        if ( $priority eq 'high' && $priority_label =~ /High/ ) {
            push @modules, $module_name;
        }
        elsif ( $priority eq 'medium' && $priority_label =~ /Medium/ ) {
            push @modules, $module_name;
        }
        elsif ( $priority eq 'low' && $priority_label =~ /Low/ ) {
            push @modules, $module_name;
        }
    }

    # Sort by priority (highest first) and limit to top candidates
    @modules = sort {
        my $a_priority = calculate_module_priority( $a, $coverage_ref->{$a},
            $required_tags_ref );
        my $b_priority = calculate_module_priority( $b, $coverage_ref->{$b},
            $required_tags_ref );
        $b_priority <=> $a_priority;
    } @modules;

    # Limit to top 10 for manageable batch processing
    splice( @modules, 10 ) if @modules > 10;

    return @modules;
}

sub calculate_module_priority {
    my ( $module_name, $module, $required_tags_ref ) = @_;

    my $priority = 0;

    # HIGHEST PRIORITY: Modules with required tags get major bonus
    my $required_tag_count = 0;
    if ( $required_tags_ref && $required_tags_ref->{$module_name} ) {
        $required_tag_count = scalar @{ $required_tags_ref->{$module_name} };
        $priority += $required_tag_count * 50;    # 50 points per required tag
        print STDERR
          "Module $module_name has $required_tag_count required tags (+"
          . ( $required_tag_count * 50 )
          . " points)\n";
    }

    # High priority for modules with many direct subdirectories
    my $direct_subdirs = $module->{stats}{direct_subdirectories} || 0;
    $priority += $direct_subdirs * 1;

    # Bonus for modules with many subdirectory tables
    my $subdir_tables = $module->{stats}{subdirectory_tables} || 0;
    $priority += $subdir_tables * 2;

    # Manufacturer-specific bonuses (based on real-world usage)
    $priority += 10 if $module_name =~ /^(Canon|Nikon|Sony)$/;
    $priority += 5  if $module_name =~ /^(Olympus|Panasonic|Fuji)$/;

    # Format-specific bonuses for common file types
    $priority += 8 if $module_name =~ /^(JPEG|PNG|TIFF|PDF|XMP)$/;
    $priority += 5 if $module_name =~ /^(RIFF|Matroska|MIE)$/;

    # Core EXIF modules get extra priority
    $priority += 15 if $module_name =~ /^(EXIF|IFD0|IFD1|ExifIFD|GPS)$/i;

    return $priority;
}

sub get_priority_label {
    my ($priority) = @_;

    return "🔥 High" if $priority >= 50;    # Modules with required tags
    return "⚡ Medium"
      if $priority >= 15;    # Modules with good subdirectory coverage
    return "💤 Low";
}

sub generate_module_config {
    my ( $module_name, $coverage_ref ) = @_;

    # Find the module file
    my $module_file = find_module_file($module_name);

    if ( !$module_file ) {
        print STDERR "Could not find module file for $module_name\n";
        return undef;
    }

    print STDERR "Analyzing $module_file...\n";

    # Analyze the module to extract configuration
    my $analysis = analyze_module_for_config($module_file);

    if ( !$analysis ) {
        print STDERR "Could not analyze $module_name\n";
        return undef;
    }

    # Generate the tag_kit configuration
    my $config = create_tag_kit_config( $module_name, $analysis );

    return $config;
}

sub find_module_file {
    my ($module_name) = @_;

    my @candidates = (
        "../../third-party/exiftool/lib/Image/ExifTool/${module_name}.pm",
        "../../third-party/exiftool/lib/Image/ExifTool/${module_name}",
    );

    foreach my $candidate (@candidates) {
        return $candidate if -f $candidate;
    }

    return undef;
}

sub analyze_module_for_config {
    my ($module_file) = @_;

    open my $fh, '<', $module_file or return undef;
    my $content = do { local $/; <$fh> };
    close $fh;

    my %analysis = (
        tables           => {},
        main_table       => undef,
        subdirectories   => [],
        has_binary_data  => 0,
        complexity_level => 'simple',
    );

    # Find main table (largest table or first one with subdirectories)
    my @table_candidates;

 # Look for both simple %table and full %Image::ExifTool::Module::table patterns
    while ( $content =~ /^%((?:\w+::)*)?(\w+)\s*=\s*\(/gm ) {
        my $full_prefix = $1 || '';
        my $table_name  = $2;
        push @table_candidates, $table_name;

        # Check if this table has subdirectories
        my $table_content = extract_table_content( $content, $table_name );
        if ( $table_content && $table_content =~ /SubDirectory\s*=>/i ) {
            $analysis{main_table} = $table_name;
            last;    # Prefer tables with subdirectories
        }
    }

    # If no table with subdirectories found, use the first major table
    if ( !$analysis{main_table} && @table_candidates ) {
        $analysis{main_table} = $table_candidates[0];
    }

    if ( !$analysis{main_table} ) {
        print STDERR "No suitable main table found\n";
        return undef;
    }

    print STDERR "Using main table: $analysis{main_table}\n";

    # Analyze the main table
    my $table_content =
      extract_table_content( $content, $analysis{main_table} );
    if ($table_content) {
        $analysis{tables}{ $analysis{main_table} } =
          analyze_table_content($table_content);

        # Extract subdirectory information
        $analysis{subdirectories} = extract_subdirectory_info($table_content);

        # Determine complexity
        $analysis{complexity_level} = determine_complexity($table_content);

        # Check for binary data
        $analysis{has_binary_data} =
          ( $table_content =~ /PROCESS_PROC.*ProcessBinaryData/i );
    }

    return \%analysis;
}

sub extract_table_content {
    my ( $content, $table_name ) = @_;

    # Try multiple patterns to extract table content
    my @patterns = (

        # Standard %table = ( format
        qr/^%${table_name}\s*=\s*\((.*?)^\);/ms,
        qr/^%${table_name}\s*=\s*\((.*?)\n\);/ms,

        # Full module path %Image::ExifTool::Module::table = ( format
        qr/^%Image::ExifTool::\w+::${table_name}\s*=\s*\((.*?)^\);/ms,
        qr/^%Image::ExifTool::\w+::${table_name}\s*=\s*\((.*?)\n\);/ms,

        # Hash reference format
        qr/\$${table_name}\s*=\s*\{(.*?)\n\};/ms,
    );

    foreach my $pattern (@patterns) {
        if ( $content =~ /$pattern/ ) {
            return $1;
        }
    }

    return undef;
}

sub analyze_table_content {
    my ($table_content) = @_;

    my %table_info = (
        format         => 'mixed',
        has_printconv  => 0,
        has_conditions => 0,
        tag_count      => 0,
    );

    # Count tags (rough estimate)
    my @tag_matches = $table_content =~ /^\s*(\w+|0x[0-9a-fA-F]+)\s*=>/gm;
    $table_info{tag_count} = scalar @tag_matches;

    # Check for PrintConv
    $table_info{has_printconv} = ( $table_content =~ /PrintConv\s*=>/i );

    # Check for conditions
    $table_info{has_conditions} = ( $table_content =~ /Condition\s*=>/i );

    # Determine format
    if ( $table_content =~ /FORMAT\s*=>\s*'(int\d+[us]?)'/i ) {
        $table_info{format} = $1;
    }

    return \%table_info;
}

sub extract_subdirectory_info {
    my ($table_content) = @_;

    my @subdirs;

    # Find all SubDirectory entries
    while ( $table_content =~
        /SubDirectory\s*=>\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}/gs )
    {
        my $subdir_def = $1;

        my %subdir_info = (
            condition  => 'unconditional',
            table_ref  => 'unknown',
            complexity => 'simple',
        );

        # Extract condition
        if ( $subdir_def =~ /Condition\s*=>\s*['"]([^'"]+)['"]/i ) {
            $subdir_info{condition}  = $1;
            $subdir_info{complexity} = classify_condition_complexity($1);
        }

        # Extract table reference
        if ( $subdir_def =~ /TagTable\s*=>\s*['"]?([^'",\s]+)['"]?/i ) {
            $subdir_info{table_ref} = $1;
        }

        push @subdirs, \%subdir_info;
    }

    return \@subdirs;
}

sub classify_condition_complexity {
    my ($condition) = @_;

    return 'simple'  if $condition =~ /^\s*\d+\s*$/;    # Numeric conditions
    return 'simple'  if $condition =~ /^\s*\$format\s+eq\s+/i;   # Format checks
    return 'medium'  if $condition =~ /\$\$self\{/;    # Model/Make conditions
    return 'complex' if $condition =~ /&&|\|\||eval\s*\{/;    # Complex logic

    return 'medium';                                          # Default
}

sub determine_complexity {
    my ($table_content) = @_;

    # Count complex patterns
    my $complexity_score = 0;

    $complexity_score += 1 if $table_content =~ /eval\s*\{/i;
    $complexity_score += 1 if $table_content =~ /\$\$valPt/;
    $complexity_score += 1 if $table_content =~ /ProcessProc/i;
    $complexity_score += 1 if $table_content =~ /WriteProc/i;

    return 'complex' if $complexity_score >= 3;
    return 'medium'  if $complexity_score >= 1;
    return 'simple';
}

sub create_tag_kit_config {
    my ( $module_name, $analysis ) = @_;

    my $config = {
        extractor => "tag_kit.pl",
        source => "third-party/exiftool/lib/Image/ExifTool/${module_name}.pm",
        description => "Auto-generated tag definitions for $module_name module",
        tables      => [
            {
                table_name  => $analysis->{main_table},
                description => "Main $module_name tag table with "
                  . scalar @{ $analysis->{subdirectories} }
                  . " subdirectories"
            }
        ],
        _auto_generated => {
            generated_by => "auto_config_gen.pl",
            generated_at => scalar( localtime() ),
            complexity   => $analysis->{complexity_level},
            config       => {
                include_subdirectories =>
                  ( scalar @{ $analysis->{subdirectories} } > 0 ) ? "true"
                : "false",
                include_printconv =>
                  $analysis->{tables}{ $analysis->{main_table} }{has_printconv}
                ? "true"
                : "false",
                subdirectory_strategy =>
                  determine_subdirectory_strategy($analysis),
                condition_handling => determine_condition_handling($analysis),
            },
            notes => [
                "Auto-generated configuration for Phase 4 expansion",
                "Module complexity: $analysis->{complexity_level}",
                "Subdirectories found: "
                  . scalar @{ $analysis->{subdirectories} },
                "Main table: $analysis->{main_table}",
            ]
        }
    };

    # Add subdirectory metadata to auto-generated section if needed
    if ( @{ $analysis->{subdirectories} } ) {
        $config->{_auto_generated}{subdirectories} = [];
        foreach my $subdir ( @{ $analysis->{subdirectories} } ) {
            push @{ $config->{_auto_generated}{subdirectories} },
              {
                condition            => $subdir->{condition},
                table_ref            => $subdir->{table_ref},
                complexity           => $subdir->{complexity},
                implementation_notes => generate_implementation_notes($subdir),
              };
        }
    }

    return $config;
}

sub determine_subdirectory_strategy {
    my ($analysis) = @_;

    my $subdir_count = scalar @{ $analysis->{subdirectories} };

    return "none"               if $subdir_count == 0;
    return "simple_dispatch"    if $subdir_count <= 3;
    return "runtime_evaluation" if $subdir_count > 10;
    return "condition_based";    # Default for moderate counts
}

sub determine_condition_handling {
    my ($analysis) = @_;

    my $complex_conditions =
      grep { $_->{complexity} eq 'complex' } @{ $analysis->{subdirectories} };

    return "runtime_required" if $complex_conditions > 0;
    return "static_dispatch"  if @{ $analysis->{subdirectories} } <= 5;
    return "hybrid";             # Mix of static and runtime
}

sub generate_implementation_notes {
    my ($subdir) = @_;

    my @notes;

    if ( $subdir->{complexity} eq 'simple' ) {
        push @notes, "Simple condition - suitable for static dispatch";
    }
    elsif ( $subdir->{complexity} eq 'complex' ) {
        push @notes, "Complex condition - requires runtime evaluation";
        push @notes, "Consider enhanced_processors.rs integration";
    }

    if ( $subdir->{table_ref} =~ /::/ ) {
        push @notes, "Cross-module reference - may need stub implementation";
    }

    if ( $subdir->{condition} eq 'unconditional' ) {
        push @notes, "Unconditional subdirectory - always processes";
    }

    return \@notes;
}

sub print_help {
    print <<EOF;
Usage: $0 [options]

Options:
    --module=NAME       Generate config for specific module only
    --dry-run           Show what would be generated without writing files
    --force             Overwrite existing configurations
    --priority=LEVEL    Filter by priority: high, medium, low (default: high)
    --help              Show this help message

Examples:
    $0                              # Generate configs for all high-priority modules
    $0 --module=JPEG --dry-run      # Preview config for JPEG module
    $0 --priority=medium --force    # Generate medium-priority configs, overwrite existing
    $0 --module=Canon               # Generate config for Canon module only

The script analyzes ExifTool modules and creates tag_kit.json configurations
for systematic subdirectory expansion. It focuses on modules with high
subdirectory counts and manageable complexity levels.
EOF
}
