#!/usr/bin/env perl
#
# Coverage Tracking Dashboard
#
# Enhanced subdirectory discovery tool that tracks implementation progress,
# identifies low-hanging fruit, and generates priority lists for Phase 4 expansion.
#
# Usage: ./coverage_dashboard.pl [--output=json|html|markdown] [--priority-only]
#

use strict;
use warnings;
use lib '../../third-party/exiftool/lib';
use File::Find;
use JSON;
use Data::Dumper;
use Getopt::Long;
use File::Basename;

# Configure output
$Data::Dumper::Sortkeys = 1;
$Data::Dumper::Indent   = 1;

my $output_format = 'markdown';
my $priority_only = 0;
my $help          = 0;

GetOptions(
    'output=s'      => \$output_format,
    'priority-only' => \$priority_only,
    'help'          => \$help,
) or die "Error in command line arguments\n";

if ($help) {
    print_help();
    exit 0;
}

# Data structures for tracking coverage
my %modules
  ; # module_name -> { tables => {}, subdirs => {}, conditions => {}, stats => {} }
my %implementation_status
  ;    # module_name -> { implemented => bool, coverage => float }
my %condition_complexity;    # condition_string -> complexity_score
my %tag_frequency
  ;    # tag_name -> frequency_score (from TagMetadata.json if available)

# Load existing implementation status
load_implementation_status();

# Load tag frequency data if available
load_tag_frequency_data();

print STDERR "Scanning ExifTool modules for subdirectory patterns...\n";

# Find all Perl modules
my @pm_files;
find(
    sub {
        push @pm_files, $File::Find::name
          if /\.pm$/ && !/Test/ && !/BuildTagLookup/;
    },
    '../../third-party/exiftool/lib/Image/ExifTool'
);

# Process each module
foreach my $file (@pm_files) {
    my $module_name = extract_module_name($file);
    next if $module_name eq 'ExifTool';    # Skip base module

    print STDERR "Processing $module_name...\n";
    $modules{$module_name} = analyze_module($file);
}

# Generate output based on format
if ( $output_format eq 'json' ) {
    generate_json_output();
}
elsif ( $output_format eq 'html' ) {
    generate_html_output();
}
else {
    generate_markdown_output();
}

sub analyze_module {
    my ($file) = @_;

    my %module_data = (
        tables     => {},
        subdirs    => {},
        conditions => {},
        stats      => {
            total_tables        => 0,
            subdirectory_tables => 0,
            simple_conditions   => 0,
            complex_conditions  => 0,
            cross_module_refs   => 0,
        }
    );

    open my $fh, '<', $file or die "Cannot open $file: $!";
    my $content = do { local $/; <$fh> };
    close $fh;

    # Extract table definitions
    while ( $content =~ /^\s*%(\w+)\s*=\s*\(/gm ) {
        my $table_name = $1;
        $module_data{tables}{$table_name} =
          analyze_table( $content, $table_name );
        $module_data{stats}{total_tables}++;

        if ( $module_data{tables}{$table_name}{has_subdirectories} ) {
            $module_data{stats}{subdirectory_tables}++;
        }
    }

    # Also look for direct SubDirectory patterns in the module
    my $subdirs_found = 0;
    while ( $content =~ /SubDirectory\s*=>\s*\{/g ) {
        $subdirs_found++;
    }

    # Update stats with direct SubDirectory count
    $module_data{stats}{direct_subdirectories} = $subdirs_found;
    if ( $subdirs_found > 0 && $module_data{stats}{subdirectory_tables} == 0 ) {

     # If we found subdirectories but no tables were marked, investigate further
        $module_data{stats}{needs_investigation} = 1;
    }

    return \%module_data;
}

sub analyze_table {
    my ( $content, $table_name ) = @_;

    my %table_data = (
        has_subdirectories => 0,
        subdirectories     => [],
        conditions         => [],
        complexity_score   => 0,
    );

    # Look for SubDirectory definitions in this table
    # Try multiple patterns to catch different table definition styles
    my @table_patterns = (
        qr/^\s*%${table_name}\s*=\s*\((.*?)\n^\s*\);/ms,    # Standard pattern
        qr/^\s*%${table_name}\s*=\s*\((.*?)\n\);/ms,        # Alternate ending
        qr/\$${table_name}\s*=\s*\{(.*?)\n\s*\};/ms,        # Hash ref pattern
    );

    my $table_content = '';
    foreach my $pattern (@table_patterns) {
        if ( $content =~ /$pattern/ ) {
            $table_content = $1;
            last;
        }
    }

    if ($table_content) {

        # Find SubDirectory entries with more flexible matching
        while ( $table_content =~
            /SubDirectory\s*=>\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}/gs )
        {
            my $subdir_def = $1;
            $table_data{has_subdirectories} = 1;

            my %subdir_info = parse_subdirectory_definition($subdir_def);
            push @{ $table_data{subdirectories} }, \%subdir_info;

            # Track conditions for complexity analysis
            if ( $subdir_info{condition} ) {
                push @{ $table_data{conditions} }, $subdir_info{condition};
                $table_data{complexity_score} +=
                  calculate_condition_complexity( $subdir_info{condition} );
            }
        }
    }

    return \%table_data;
}

sub parse_subdirectory_definition {
    my ($subdir_def) = @_;

    my %info = (
        condition    => undef,
        table_ref    => undef,
        process_proc => undef,
        complexity   => 'simple',
    );

    # Extract condition
    if ( $subdir_def =~ /Condition\s*=>\s*['"]([^'"]+)['"]/ ) {
        $info{condition}  = $1;
        $info{complexity} = classify_condition_complexity($1);
    }

    # Extract table reference
    if ( $subdir_def =~ /TagTable\s*=>\s*['"]?([^'",\s]+)['"]?/ ) {
        $info{table_ref} = $1;
    }

    # Extract ProcessProc
    if ( $subdir_def =~ /ProcessProc\s*=>\s*\\&(\w+)/ ) {
        $info{process_proc} = $1;
    }

    return %info;
}

sub calculate_condition_complexity {
    my ($condition) = @_;
    return 0 unless $condition;

    my $score = 0;

    # Simple patterns get low scores
    $score += 1 if $condition =~ /\$count\s*[><=]/;    # Count conditions
    $score += 1 if $condition =~ /\$format\s*eq/;      # Format conditions

    # Medium complexity patterns
    $score += 3 if $condition =~ /\$\$self\{/;    # Model/Make conditions
    $score += 3 if $condition =~ /\$\$valPt/;     # Binary pattern conditions

    # High complexity patterns
    $score += 5 if $condition =~ /&&|\|\|/;           # Logical operators
    $score += 5 if $condition =~ /\(.*\).*\(.*\)/;    # Multiple groupings
    $score += 7 if $condition =~ /eval\s*{/;          # Perl eval blocks

    return $score;
}

sub classify_condition_complexity {
    my ($condition) = @_;

    my $score = calculate_condition_complexity($condition);

    return 'simple' if $score <= 2;
    return 'medium' if $score <= 5;
    return 'complex';
}

sub load_implementation_status {

    # Check which modules have tag_kit configurations
    my $config_dir = '../config';
    return unless -d $config_dir;

    opendir my $dh, $config_dir or return;
    while ( my $dir = readdir $dh ) {
        next if $dir =~ /^\.\.?$/;
        next unless -d "$config_dir/$dir";

        if ( -f "$config_dir/$dir/tag_kit.json" ) {
            my $module_name = $dir;
            $module_name =~ s/_pm$//;    # Remove _pm suffix
            $implementation_status{$module_name} = { implemented => 1 };
        }
    }
    closedir $dh;
}

sub load_tag_frequency_data {
    my $metadata_file = '../../docs/tag-metadata.json';
    return unless -f $metadata_file;

    eval {
        my $json_text = do {
            open my $fh, '<', $metadata_file or die $!;
            local $/;
            <$fh>;
        };

        my $data = decode_json($json_text);
        foreach my $tag_name ( keys %$data ) {
            my $tag_info = $data->{$tag_name};
            if ( $tag_info->{frequency} ) {
                $tag_frequency{$tag_name} = $tag_info->{frequency};
            }
        }
    };

    if ($@) {
        print STDERR "Warning: Could not load tag frequency data: $@\n";
    }
}

sub extract_module_name {
    my ($file) = @_;
    my $base = basename( $file, '.pm' );
    return $base;
}

sub generate_markdown_output {
    print "# Subdirectory Coverage Dashboard\n\n";
    print "Generated: " . localtime() . "\n\n";

    # Summary statistics
    generate_summary_stats();

    # Implementation status
    generate_implementation_matrix();

    # Priority recommendations
    generate_priority_recommendations();

    # Detailed module analysis
    unless ($priority_only) {
        generate_detailed_analysis();
    }
}

sub generate_summary_stats {
    my $total_modules = scalar keys %modules;
    my $implemented_modules =
      scalar grep { $implementation_status{$_}{implemented} } keys %modules;
    my $implementation_percent =
      $total_modules > 0
      ? sprintf( "%.1f", 100 * $implemented_modules / $total_modules )
      : 0;

    my $total_subdirs   = 0;
    my $simple_subdirs  = 0;
    my $complex_subdirs = 0;

    foreach my $module_name ( keys %modules ) {
        my $module = $modules{$module_name};
        foreach my $table_name ( keys %{ $module->{tables} } ) {
            my $table = $module->{tables}{$table_name};
            next unless $table->{has_subdirectories};

            foreach my $subdir ( @{ $table->{subdirectories} } ) {
                $total_subdirs++;
                if ( $subdir->{complexity} eq 'simple' ) {
                    $simple_subdirs++;
                }
                elsif ( $subdir->{complexity} eq 'complex' ) {
                    $complex_subdirs++;
                }
            }
        }
    }

    print "## Summary\n\n";
    print "- **Total Modules**: $total_modules\n";
    print
      "- **Implemented**: $implemented_modules ($implementation_percent%)\n";
    print "- **Total Subdirectories**: $total_subdirs\n";
    print "- **Simple Conditions**: $simple_subdirs ("
      . sprintf( "%.1f", 100 * $simple_subdirs / ( $total_subdirs || 1 ) )
      . "%)\n";
    print "- **Complex Conditions**: $complex_subdirs ("
      . sprintf( "%.1f", 100 * $complex_subdirs / ( $total_subdirs || 1 ) )
      . "%)\n";
    print "\n";
}

sub generate_implementation_matrix {
    print "## Implementation Status Matrix\n\n";
    print
"| Module | Status | Tables | Subdirs | Direct | Simple | Complex | Priority |\n";
    print
"|--------|--------|---------|---------|--------|--------|---------|----------|\n";

    my @sorted_modules = sort {
        my $a_priority = calculate_module_priority($a);
        my $b_priority = calculate_module_priority($b);
        $b_priority <=> $a_priority;    # Descending priority
    } keys %modules;

    foreach my $module_name (@sorted_modules) {
        my $module = $modules{$module_name};
        my $status =
          $implementation_status{$module_name}{implemented} ? "✅" : "❌";

        my $total_tables   = $module->{stats}{total_tables};
        my $subdir_tables  = $module->{stats}{subdirectory_tables};
        my $direct_subdirs = $module->{stats}{direct_subdirectories} || 0;

        my ( $simple_count, $complex_count ) =
          count_condition_complexity($module);
        my $priority       = calculate_module_priority($module_name);
        my $priority_label = get_priority_label($priority);

        print
"| $module_name | $status | $total_tables | $subdir_tables | $direct_subdirs | $simple_count | $complex_count | $priority_label |\n";
    }
    print "\n";
}

sub generate_priority_recommendations {
    print "## Priority Recommendations\n\n";

    my @high_priority   = get_modules_by_priority('high');
    my @medium_priority = get_modules_by_priority('medium');

    print "### High Priority (Easy Wins)\n\n";
    foreach my $module_name (@high_priority) {
        next if $implementation_status{$module_name}{implemented};

        my $module = $modules{$module_name};
        my ( $simple_count, $complex_count ) =
          count_condition_complexity($module);
        my $reason = get_priority_reason($module_name);

        print
"- **$module_name**: $simple_count simple, $complex_count complex conditions. $reason\n";
    }

    print "\n### Medium Priority\n\n";
    foreach my $module_name (@medium_priority) {
        next if $implementation_status{$module_name}{implemented};

        my $module = $modules{$module_name};
        my ( $simple_count, $complex_count ) =
          count_condition_complexity($module);
        my $reason = get_priority_reason($module_name);

        print
"- **$module_name**: $simple_count simple, $complex_count complex conditions. $reason\n";
    }
    print "\n";
}

sub generate_detailed_analysis {
    print "## Detailed Module Analysis\n\n";

    foreach my $module_name ( sort keys %modules ) {
        my $module = $modules{$module_name};
        next unless $module->{stats}{subdirectory_tables} > 0;

        print "### $module_name\n\n";

        foreach my $table_name ( sort keys %{ $module->{tables} } ) {
            my $table = $module->{tables}{$table_name};
            next unless $table->{has_subdirectories};

            print "**Table: $table_name**\n\n";

            foreach my $subdir ( @{ $table->{subdirectories} } ) {
                my $condition  = $subdir->{condition} || 'unconditional';
                my $complexity = $subdir->{complexity};
                my $table_ref  = $subdir->{table_ref} || 'unknown';

                print "- Condition: `$condition` (complexity: $complexity)\n";
                print "  - References: $table_ref\n";
            }
            print "\n";
        }
    }
}

sub count_condition_complexity {
    my ($module) = @_;

    my $simple_count  = 0;
    my $complex_count = 0;

    foreach my $table_name ( keys %{ $module->{tables} } ) {
        my $table = $module->{tables}{$table_name};

        foreach my $subdir ( @{ $table->{subdirectories} } ) {
            if ( $subdir->{complexity} eq 'simple' ) {
                $simple_count++;
            }
            elsif ( $subdir->{complexity} eq 'complex' ) {
                $complex_count++;
            }
            else {
                $simple_count++;    # Treat medium as simple for now
            }
        }
    }

    return ( $simple_count, $complex_count );
}

sub calculate_module_priority {
    my ($module_name) = @_;

    return 0 if $implementation_status{$module_name}{implemented};

    my $module   = $modules{$module_name};
    my $priority = 0;

    # High priority for modules with many simple subdirectories
    my ( $simple_count, $complex_count ) = count_condition_complexity($module);
    $priority += $simple_count * 3;     # Simple conditions are easier
    $priority += $complex_count * 1;    # Complex conditions are harder

    # Bonus for modules with many subdirectory tables
    $priority += $module->{stats}{subdirectory_tables} * 2;

    # Additional bonus for modules with direct subdirectories
    $priority += ( $module->{stats}{direct_subdirectories} || 0 ) * 1;

    # Manufacturer-specific bonuses (based on real-world usage)
    $priority += 10 if $module_name =~ /^(Canon|Nikon|Sony)$/;
    $priority += 5  if $module_name =~ /^(Olympus|Panasonic|Fuji)$/;

    return $priority;
}

sub get_priority_label {
    my ($priority) = @_;

    return "🔥 High"   if $priority >= 15;
    return "⚡ Medium" if $priority >= 8;
    return "💤 Low";
}

sub get_modules_by_priority {
    my ($level) = @_;

    my @modules = grep {
        my $priority = calculate_module_priority($_);
        my $label    = get_priority_label($priority);
        $label =~ /\Q$level\E/i;
    } keys %modules;

    return
      sort { calculate_module_priority($b) <=> calculate_module_priority($a) }
      @modules;
}

sub get_priority_reason {
    my ($module_name) = @_;

    my $module = $modules{$module_name};
    my ( $simple_count, $complex_count ) = count_condition_complexity($module);

    if ( $simple_count > $complex_count * 2 ) {
        return "Mostly simple conditions, good for quick implementation.";
    }
    elsif ( $module->{stats}{subdirectory_tables} > 5 ) {
        return "Many subdirectory tables, high coverage impact.";
    }
    elsif ( $module_name =~ /^(Canon|Nikon|Sony)$/ ) {
        return "Major manufacturer, high real-world usage.";
    }
    else {
        return "Moderate complexity, suitable for systematic expansion.";
    }
}

sub generate_json_output {
    my %output = (
        summary => {
            total_modules       => scalar keys %modules,
            implemented_modules => scalar
              grep { $implementation_status{$_}{implemented} } keys %modules,
            generated_at => time(),
        },
        modules               => \%modules,
        implementation_status => \%implementation_status,
    );

    print encode_json( \%output );
}

sub generate_html_output {
    print
"<!DOCTYPE html>\n<html><head><title>Subdirectory Coverage Dashboard</title></head><body>\n";
    print "<h1>Subdirectory Coverage Dashboard</h1>\n";
    print "<p>Generated: " . localtime() . "</p>\n";

    # ... HTML generation logic ...
    print "</body></html>\n";
}

sub print_help {
    print <<EOF;
Usage: $0 [options]

Options:
    --output=FORMAT     Output format: json, html, markdown (default: markdown)
    --priority-only     Show only priority recommendations, skip detailed analysis
    --help              Show this help message

Examples:
    $0                              # Generate markdown dashboard
    $0 --output=json                # Generate JSON output for CI integration
    $0 --priority-only              # Quick priority summary
EOF
}
