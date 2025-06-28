#!/usr/bin/env perl

=head1 NAME

analyze_printconv_safety.pl - Analyze PrintConv safety across ExifTool modules

=head1 SYNOPSIS

    scripts/analyze_printconv_safety.pl [OPTIONS]

    # Analyze all modules (default)
    scripts/analyze_printconv_safety.pl

    # Verbose output
    scripts/analyze_printconv_safety.pl --verbose

    # Analyze specific modules only
    scripts/analyze_printconv_safety.pl --modules Canon,Nikon,Sony

=head1 DESCRIPTION

This script analyzes PrintConv (Print Conversion) implementations across all
ExifTool modules to identify:

1. Tags with same names in different contexts (EXIF, MakerNotes, XMP)
2. Whether those tags have identical or different PrintConv implementations
3. Safety levels for using universal PrintConv functions
4. Recommendations for handling name collisions

The script uses Perl introspection to extract actual PrintConv data rather than
attempting to parse Perl syntax, ensuring accurate analysis.

=head1 OUTPUT FORMAT

The script outputs JSON with:
- metadata: Analysis date, ExifTool version
- statistics: Summary of findings
- tags: Detailed analysis of each tag
- collisions: Groups of tags with same names
- recommendations: Suggested PrintConvId mappings

=head1 AUTHOR

Part of exif-oxide project - Rust implementation of ExifTool

=cut

use strict;
use warnings;
use JSON;
use Data::Dumper;
use Getopt::Long;
use File::Find;
use lib 'third-party/exiftool/lib';

# Command line options
my $verbose = 0;
my $specific_modules = '';
my $help = 0;

GetOptions(
    'verbose|v'   => \$verbose,
    'modules|m=s' => \$specific_modules,
    'help|h'      => \$help,
) or die "Error in command line arguments\n";

if ($help) {
    print "Usage: $0 [OPTIONS]\n";
    print "Options:\n";
    print "  --verbose, -v     Verbose output\n";
    print "  --modules, -m     Comma-separated list of modules to analyze\n";
    print "  --help, -h        Show this help\n";
    exit 0;
}

# Track all extracted tags
my @all_tags;
my %tags_by_name;
my %shared_lookups;
my %seen_modules;

# Function to normalize PrintConv for comparison
sub normalize_printconv {
    my ($pc) = @_;
    
    if (!defined $pc) {
        return { type => 'none', signature => 'NO_PRINTCONV' };
    }
    
    if (ref($pc) eq 'HASH') {
        # Direct hash - create sorted key-value signature
        my @pairs;
        foreach my $key (sort { 
            # Try numeric comparison first, fall back to string
            my $num_a = $a;
            my $num_b = $b;
            if ($num_a =~ /^\d+$/ && $num_b =~ /^\d+$/) {
                $num_a <=> $num_b;
            } else {
                $a cmp $b;
            }
        } keys %$pc) {
            my $val = $pc->{$key};
            if (!ref($val)) {
                push @pairs, "$key:$val";
            }
        }
        return {
            type => 'hash',
            signature => join(',', @pairs),
            entry_count => scalar(keys %$pc)
        };
    }
    
    if (ref($pc) eq 'CODE') {
        # Code reference - try to identify known functions
        no strict 'refs';
        
        # Check against known ExifTool functions
        my %known_funcs = (
            'Image::ExifTool::Exif::PrintExposureTime' => 'PrintExposureTime',
            'Image::ExifTool::Exif::PrintFraction' => 'PrintFraction',
            'Image::ExifTool::Exif::PrintFNumber' => 'PrintFNumber',
            'Image::ExifTool::GPS::PrintGPSCoordinate' => 'PrintGPSCoordinate',
            'Image::ExifTool::ConvertBinary' => 'ConvertBinary',
        );
        
        foreach my $full_name (keys %known_funcs) {
            if ($pc == \&{$full_name}) {
                return {
                    type => 'code_ref',
                    signature => "FUNC:$known_funcs{$full_name}",
                    function => $known_funcs{$full_name}
                };
            }
        }
        
        return {
            type => 'code_ref',
            signature => 'FUNC:UNKNOWN',
            function => 'UNKNOWN'
        };
    }
    
    if (ref($pc) eq 'ARRAY') {
        # Array (bitfield decoding)
        return {
            type => 'array',
            signature => 'ARRAY:' . scalar(@$pc),
            entry_count => scalar(@$pc)
        };
    }
    
    if (!ref($pc)) {
        # String expression or hash reference
        if ($pc =~ /^\\%(\w+)$/) {
            # Hash reference like \%canonLensTypes
            my $hash_name = $1;
            return {
                type => 'hash_ref',
                signature => "HASHREF:$hash_name",
                ref_name => $hash_name
            };
        } elsif ($pc =~ /^\\&(\S+)$/) {
            # Function reference like \&PrintExposureTime
            return {
                type => 'sub_ref',
                signature => "FUNCREF:$1",
                function => $1
            };
        } else {
            # String expression
            return {
                type => 'string',
                signature => "STRING:$pc",
                source => $pc
            };
        }
    }
    
    return {
        type => 'other',
        signature => 'UNKNOWN:' . ref($pc)
    };
}

# Extract tags from a single table
sub extract_tags_from_table {
    my ($module, $table_name, $table_ref) = @_;
    my @tags;
    
    foreach my $key (keys %$table_ref) {
        # Skip special keys
        next if $key =~ /^(GROUPS|NOTES|NAMESPACE|PRIORITY|WRITE_PROC|PROCESS_PROC|CHECK_PROC|VARS|TABLE_NAME|SHORT_NAME)$/;
        
        my $tag_def = $table_ref->{$key};
        next unless ref($tag_def) eq 'HASH';
        
        my $tag_name = $tag_def->{Name} || $key;
        my $printconv = $tag_def->{PrintConv};
        
        # Normalize the PrintConv
        my $pc_info = normalize_printconv($printconv);
        
        # Determine context and manufacturer
        my ($context, $manufacturer) = classify_module($module);
        
        # Create tag analysis entry
        my $tag_analysis = {
            tag_name => $tag_name,
            tag_id => $key,
            module => $module,
            table_name => $table_name,
            context => $context,
            manufacturer => $manufacturer,
            printconv_type => $pc_info->{type},
            printconv_signature => $pc_info->{signature},
            # Initialize required fields that will be set later
            group_id => '',
            safety_level => '',
            recommended_printconv_id => $tag_name, # Default to tag name
            collision_details => [],
        };
        
        # Add additional info based on type
        if ($pc_info->{type} eq 'hash_ref') {
            $tag_analysis->{printconv_ref} = $pc_info->{ref_name};
            
            # Try to resolve the hash reference
            no strict 'refs';
            my $full_name = "${module}::$pc_info->{ref_name}";
            my $hash_ref = eval { \%{$full_name} };
            if ($hash_ref && %$hash_ref) {
                my $resolved = normalize_printconv($hash_ref);
                $tag_analysis->{resolved_signature} = $resolved->{signature};
                $tag_analysis->{resolved_entry_count} = $resolved->{entry_count};
                
                # Track shared lookup
                $shared_lookups{$pc_info->{ref_name}} = {
                    module => $module,
                    entry_count => $resolved->{entry_count},
                    signature => $resolved->{signature}
                };
            }
        } elsif ($pc_info->{type} eq 'string') {
            $tag_analysis->{printconv_source} = $pc_info->{source};
        } elsif ($pc_info->{type} eq 'code_ref' || $pc_info->{type} eq 'sub_ref') {
            $tag_analysis->{printconv_function} = $pc_info->{function} || 'UNKNOWN';
        }
        
        push @tags, $tag_analysis;
    }
    
    return @tags;
}

# Classify module into context and manufacturer
sub classify_module {
    my ($module) = @_;
    
    # Strip Image::ExifTool:: prefix
    my $short = $module;
    $short =~ s/^Image::ExifTool:://;
    
    # Common contexts
    return ('EXIF', undef) if $short eq 'Exif';
    return ('XMP', undef) if $short eq 'XMP';
    return ('IPTC', undef) if $short eq 'IPTC';
    return ('GPS', undef) if $short eq 'GPS';
    return ('Composite', undef) if $short eq 'Composite';
    
    # Manufacturer modules
    my @manufacturers = qw(Canon Nikon Sony Pentax Olympus Fujifilm Panasonic 
                          Samsung Sigma Leica Ricoh Casio Minolta DJI Apple);
    
    foreach my $mfg (@manufacturers) {
        if ($short =~ /^$mfg/i) {
            return ('MakerNote', $mfg);
        }
    }
    
    # Other contexts
    return ('QuickTime', undef) if $short =~ /QuickTime/;
    return ('PDF', undef) if $short eq 'PDF';
    return ('PNG', undef) if $short eq 'PNG';
    
    return ('Other', $short);
}

# Load and analyze a module
sub analyze_module {
    my ($module_name) = @_;
    
    print STDERR "Analyzing $module_name...\n" if $verbose;
    
    # Load the module
    eval "require $module_name";
    if ($@) {
        warn "Failed to load $module_name: $@\n";
        return;
    }
    
    $seen_modules{$module_name} = 1;
    
    # Find all tag tables in the module
    no strict 'refs';
    my $symbol_table = \%{"${module_name}::"};
    
    foreach my $symbol (keys %$symbol_table) {
        my $glob = $symbol_table->{$symbol};
        if (*{$glob}{HASH}) {
            my $hash_ref = \%{"${module_name}::${symbol}"};
            
            # Check if this looks like a tag table
            if (exists $hash_ref->{GROUPS} || exists $hash_ref->{NOTES} || 
                exists $hash_ref->{0} || exists $hash_ref->{0x01} ||
                $symbol =~ /Table$/ || $symbol eq 'Main') {
                
                my @tags = extract_tags_from_table($module_name, $symbol, $hash_ref);
                push @all_tags, @tags;
                
                # Group by tag name
                foreach my $tag (@tags) {
                    push @{$tags_by_name{$tag->{tag_name}}}, $tag;
                }
            }
        }
    }
}

# Find all ExifTool modules
sub find_modules {
    my @modules;
    
    my $lib_dir = 'third-party/exiftool/lib/Image/ExifTool';
    
    find({
        wanted => sub {
            return unless /\.pm$/;
            return if /BuildTagLookup\.pm$/; # Skip this special module
            
            my $path = $File::Find::name;
            $path =~ s/^third-party\/exiftool\/lib\///;
            $path =~ s/\.pm$//;
            $path =~ s/\//\:\:/g;
            
            push @modules, $path;
        },
        no_chdir => 1
    }, $lib_dir);
    
    return @modules;
}

# Analyze safety for tag groups
sub analyze_safety {
    my %collision_groups;
    my %safe_groups;
    my %unique_tags;
    
    foreach my $tag_name (keys %tags_by_name) {
        my $tags = $tags_by_name{$tag_name};
        
        if (@$tags == 1) {
            # Unique tag name
            $unique_tags{$tag_name} = $tags->[0];
            $tags->[0]->{safety_level} = 'UniqueContext';
            $tags->[0]->{group_id} = "UNIQUE_" . sprintf("%03d", scalar(keys %unique_tags));
        } else {
            # Multiple tags with same name - check for collisions
            my %signatures;
            my $has_printconv = 0;
            
            foreach my $tag (@$tags) {
                my $sig = $tag->{resolved_signature} || $tag->{printconv_signature};
                push @{$signatures{$sig}}, $tag;
                $has_printconv = 1 if $sig ne 'NO_PRINTCONV';
            }
            
            if (scalar(keys %signatures) == 1 && $has_printconv) {
                # All have same PrintConv - SAFE
                my $group_id = "SAFE_" . sprintf("%03d", scalar(keys %safe_groups) + 1);
                $safe_groups{$tag_name} = {
                    tags => $tags,
                    signature => (keys %signatures)[0]
                };
                
                foreach my $tag (@$tags) {
                    $tag->{safety_level} = 'Safe';
                    $tag->{group_id} = $group_id;
                    $tag->{recommended_printconv_id} = $tag_name;
                }
            } elsif (scalar(keys %signatures) > 1) {
                # Different implementations - COLLISION
                my $group_id = "COLLISION_" . sprintf("%03d", scalar(keys %collision_groups) + 1);
                $collision_groups{$tag_name} = {
                    tags => $tags,
                    signatures => \%signatures
                };
                
                foreach my $tag (@$tags) {
                    $tag->{safety_level} = 'CollisionRisk';
                    $tag->{group_id} = $group_id;
                    
                    # Generate context-specific ID
                    my $suffix = $tag->{manufacturer} || $tag->{context};
                    $suffix =~ s/MakerNote//;
                    $tag->{recommended_printconv_id} = $tag_name . $suffix;
                    
                    # Add collision details
                    my @collision_details;
                    foreach my $other (@$tags) {
                        next if $other == $tag;
                        my $detail = sprintf("%s:%s has signature '%s'",
                            $other->{context},
                            $other->{manufacturer} || '',
                            $other->{printconv_signature}
                        );
                        push @collision_details, $detail;
                    }
                    $tag->{collision_details} = \@collision_details;
                }
            } else {
                # No PrintConv implementations
                my $group_id = "MANUAL_" . sprintf("%03d", scalar(keys %unique_tags) + scalar(keys %safe_groups) + scalar(keys %collision_groups) + 1);
                
                foreach my $tag (@$tags) {
                    $tag->{safety_level} = 'NoImplementation';
                    $tag->{group_id} = $group_id;
                    $tag->{recommended_printconv_id} = $tag_name;
                }
            }
        }
    }
    
    return {
        collision_groups => \%collision_groups,
        safe_groups => \%safe_groups,
        unique_tags => \%unique_tags
    };
}

# Main execution
print STDERR "Analyzing PrintConv safety across ExifTool modules...\n" if $verbose;

# Get list of modules to analyze
my @modules_to_analyze;

if ($specific_modules) {
    # Analyze specific modules
    my @requested = split(/,/, $specific_modules);
    foreach my $mod (@requested) {
        push @modules_to_analyze, "Image::ExifTool::$mod";
    }
} else {
    # Find all modules
    @modules_to_analyze = find_modules();
}

print STDERR "Found " . scalar(@modules_to_analyze) . " modules to analyze\n" if $verbose;

# Analyze each module
foreach my $module (@modules_to_analyze) {
    analyze_module($module);
}

# Analyze safety
my $safety_analysis = analyze_safety();

# Calculate statistics
my %stats = (
    total_modules => scalar(keys %seen_modules),
    total_tags => scalar(@all_tags),
    unique_tag_names => scalar(keys %tags_by_name),
    safe_universal => scalar(keys %{$safety_analysis->{safe_groups}}),
    collision_risks => scalar(keys %{$safety_analysis->{collision_groups}}),
    unique_context => scalar(keys %{$safety_analysis->{unique_tags}}),
    tags_with_printconv => scalar(grep { $_->{printconv_signature} ne 'NO_PRINTCONV' } @all_tags),
    shared_lookups => scalar(keys %shared_lookups),
);

# Add PrintConv type breakdown
my %pc_types;
foreach my $tag (@all_tags) {
    $pc_types{$tag->{printconv_type}}++;
}
$stats{printconv_types} = \%pc_types;

# Create output structure
my $output = {
    metadata => {
        extraction_date => scalar(localtime),
        exiftool_version => $Image::ExifTool::VERSION || 'unknown',
        modules_analyzed => scalar(keys %seen_modules),
    },
    statistics => \%stats,
    tags => \@all_tags,
    collision_groups => $safety_analysis->{collision_groups},
    safe_groups => $safety_analysis->{safe_groups},
    shared_lookups => \%shared_lookups,
};

# Output as JSON
my $json = JSON->new->pretty->canonical->allow_nonref;
print $json->encode($output);