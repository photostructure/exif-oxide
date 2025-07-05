package ExifToolExtract;

#------------------------------------------------------------------------------
# File:         ExifToolExtract.pm
#
# Description:  Common utilities for ExifTool extraction scripts
#
# Notes:        This module provides shared functionality for all codegen
#               extraction scripts, following the DRY principle.
#------------------------------------------------------------------------------

use strict;
use warnings;
use JSON qw(encode_json decode_json);
use Exporter 'import';

our @EXPORT_OK = qw(
    load_module_from_file
    get_package_hash
    load_json_config
    load_tag_metadata
    is_mainstream_tag
    is_mainstream_composite_tag
    validate_primitive_value
    validate_primitive_table
    extract_source_line_info
    format_json_output
    generate_conv_ref
    extract_format
    extract_groups
    validate_regex_for_rust
    clean_tag_name
);

#------------------------------------------------------------------------------
# Safely load a Perl module from a file path
#------------------------------------------------------------------------------
sub load_module_from_file {
    my ($module_file) = @_;
    
    # Validate the file exists and is readable
    unless (-r $module_file) {
        warn "Cannot read module file: $module_file\n";
        return;
    }
    
    # Extract module name from file path
    my ($module_name) = $module_file =~ /([^\/]+)\.pm$/;
    unless ($module_name) {
        warn "Cannot extract module name from: $module_file\n";
        return;
    }
    
    # Convert to full package name
    if ($module_name eq 'ExifTool') {
        # Special case: ExifTool.pm is in parent directory
        $module_file =~ s{/Image/ExifTool/ExifTool\.pm$}{/Image/ExifTool.pm};
        unless (-r $module_file) {
            warn "Cannot find ExifTool.pm at expected location: $module_file\n";
            return;
        }
        $module_name = "Image::ExifTool";  # Main ExifTool module
    } else {
        $module_name = "Image::ExifTool::$module_name";
    }
    
    # Load the module safely
    eval {
        require $module_file;
        1;
    } or do {
        warn "Failed to load module $module_file: $@\n";
        return;
    };
    
    return $module_name;
}

#------------------------------------------------------------------------------
# Get a reference to a package hash variable
#------------------------------------------------------------------------------
sub get_package_hash {
    my ($module_name, $hash_name) = @_;
    
    # Clean up hash name (remove % prefix if present)
    $hash_name =~ s/^%//;
    
    # Access the package hash using symbolic references
    no strict 'refs';
    my $hash_ref = \%{$module_name . "::" . $hash_name};
    
    # Check if the hash exists and has entries
    unless (%$hash_ref) {
        # Try without module prefix for some special cases
        $hash_ref = \%{"::" . $hash_name};
        unless (%$hash_ref) {
            warn "Hash $hash_name not found or empty in module $module_name\n";
            return;
        }
    }
    
    return $hash_ref;
}

#------------------------------------------------------------------------------
# Load JSON configuration file
#------------------------------------------------------------------------------
sub load_json_config {
    my ($config_file) = @_;
    
    open(my $fh, '<', $config_file) or die "Cannot open $config_file: $!";
    my $json_text = do { local $/; <$fh> };
    close($fh);
    
    return decode_json($json_text);
}

#------------------------------------------------------------------------------
# Load TagMetadata.json for frequency filtering
#------------------------------------------------------------------------------
sub load_tag_metadata {
    my ($file) = @_;
    
    return load_json_config($file);
}

#------------------------------------------------------------------------------
# Check if tag should be included based on frequency/mainstream criteria
#------------------------------------------------------------------------------
sub is_mainstream_tag {
    my ($tag_name, $metadata) = @_;
    
    return 0 unless defined $tag_name;
    
    # Check metadata
    if (exists $metadata->{$tag_name}) {
        my $meta = $metadata->{$tag_name};
        
        # Include tags with frequency > 0.8
        return 1 if ($meta->{frequency} && $meta->{frequency} > 0.8);
        
        # Include all mainstream tags
        return 1 if ($meta->{mainstream});
    }
    
    # Always include basic file information tags
    my @always_include = qw(
        ImageWidth ImageHeight Make Model Orientation
        ExifImageWidth ExifImageHeight DateTime
        ImageDescription Copyright
        Flash ColorSpace ExposureProgram MeteringMode
        ResolutionUnit YCbCrPositioning YCbCrSubSampling
        WhiteBalance ExposureTime FNumber FocalLength
        DateTimeOriginal CreateDate
        ExifOffset GPSInfo
    );
    
    return 1 if grep { $_ eq $tag_name } @always_include;
    
    return 0;
}

#------------------------------------------------------------------------------
# Check if composite tag should be included (more lenient criteria)
#------------------------------------------------------------------------------
sub is_mainstream_composite_tag {
    my ($tag_name, $metadata) = @_;
    
    return 0 unless defined $tag_name;
    
    # Check metadata with more lenient criteria for composite tags
    if (exists $metadata->{$tag_name}) {
        my $meta = $metadata->{$tag_name};
        
        # Include composite tags with frequency > 0.5 (more lenient)
        return 1 if ($meta->{frequency} && $meta->{frequency} > 0.5);
        
        # Include all mainstream tags
        return 1 if ($meta->{mainstream});
    }
    
    # Always include essential composite tags
    my @always_include = qw(
        Aperture
        ShutterSpeed
        ISO
        LensID
        LensSpec
        FocalLength35efl
        GPSPosition
        GPSLatitude
        GPSLongitude
        SubSecCreateDate
        SubSecDateTimeOriginal
        SubSecModifyDate
    );
    
    return 1 if grep { $_ eq $tag_name } @always_include;
    
    return 0;
}

#------------------------------------------------------------------------------
# Validate that a value is primitive (no Perl code)
#------------------------------------------------------------------------------
sub validate_primitive_value {
    my ($value) = @_;
    
    return 0 unless defined $value;
    return 0 if ref $value;              # No references
    return 0 if $value =~ /[\$\@\%]/;    # No Perl variables
    return 0 if $value =~ /\bsub\s*\{/;  # No anonymous subs
    return 0 if $value =~ /\beval\s*\{/; # No eval blocks
    
    return 1;
}

#------------------------------------------------------------------------------
# Validate that an entire table contains only primitive values
#------------------------------------------------------------------------------
sub validate_primitive_table {
    my ($hash_ref) = @_;
    
    for my $key (keys %$hash_ref) {
        my $value = $hash_ref->{$key};
        
        # Skip special ExifTool entries
        next if $key eq 'Notes';
        next if $key eq 'OTHER';
        next if $key =~ /^[A-Z_]+$/;  # Skip all-caps keys
        
        # Validate key is primitive
        return 0 if $key =~ /[\$\@\%]/;
        
        # Validate value is primitive
        return 0 unless validate_primitive_value($value);
    }
    
    return 1;
}

#------------------------------------------------------------------------------
# Extract source line information (stub for now, can be enhanced later)
#------------------------------------------------------------------------------
sub extract_source_line_info {
    my ($module_file, $hash_name, $key) = @_;
    
    # This is a simplified version - could be enhanced to actually
    # parse the source file and find line numbers
    return {
        file => $module_file,
        hash => $hash_name,
        line => 0,  # Would need file parsing to get actual line
    };
}

#------------------------------------------------------------------------------
# Format JSON output consistently
#------------------------------------------------------------------------------
sub format_json_output {
    my ($data) = @_;
    
    # Use canonical ordering for consistent output
    my $json = JSON->new->canonical->pretty;
    return $json->encode($data);
}

#------------------------------------------------------------------------------
# Generate conversion reference string for PrintConv/ValueConv
#------------------------------------------------------------------------------
sub generate_conv_ref {
    my ($tag_name, $conv_type, $conv_data) = @_;
    
    return undef unless defined $conv_data;
    
    # Generate a reference string based on tag name and conversion type
    my $ref = lc($tag_name);
    $ref =~ s/[^a-z0-9]/_/g;  # Replace non-alphanumeric with underscore
    $ref .= "_${conv_type}";
    
    return $ref;
}

#------------------------------------------------------------------------------
# Extract format information from tag definition
#------------------------------------------------------------------------------
sub extract_format {
    my $tag_info = shift;
    
    # Get Writable format (preferred) or Format
    my $format = $tag_info->{Writable} || $tag_info->{Format} || 'undef';
    
    # Handle format specifications
    if (ref $format eq 'HASH') {
        # Complex format - return default
        return 'undef';
    }
    
    # Clean up format string
    $format =~ s/\s+//g;  # Remove whitespace
    
    return $format;
}

#------------------------------------------------------------------------------
# Extract groups information from tag definition
#------------------------------------------------------------------------------
sub extract_groups {
    my $tag_info = shift;
    
    my @groups = ('EXIF');  # Default group
    
    if ($tag_info->{Groups}) {
        my $groups_ref = $tag_info->{Groups};
        if (ref $groups_ref eq 'HASH') {
            # Add group values
            push @groups, values %$groups_ref;
        }
    }
    
    # Remove duplicates and sort
    my %seen;
    @groups = grep { !$seen{$_}++ } @groups;
    @groups = sort @groups;
    
    return \@groups;
}

#------------------------------------------------------------------------------
# Validate regex pattern for Rust regex crate compatibility
#------------------------------------------------------------------------------
sub validate_regex_for_rust {
    my ($pattern) = @_;
    
    # Check for features not supported by Rust regex crate
    
    # Check for lookaround assertions
    if ($pattern =~ /\(\?[=!]/) {
        return { compatible => 0, reason => "Contains positive/negative lookahead (?= or (?!)" };
    }
    if ($pattern =~ /\(\?<[=!]/) {
        return { compatible => 0, reason => "Contains positive/negative lookbehind (?<= or (?<!)" };
    }
    
    # Check for backreferences
    if ($pattern =~ /\\[1-9]/) {
        return { compatible => 0, reason => "Contains backreferences (\\1, \\2, etc.)" };
    }
    
    # Check for possessive quantifiers
    if ($pattern =~ /[*+?]\+/) {
        return { compatible => 0, reason => "Contains possessive quantifiers (*+, ++, ?+)" };
    }
    
    # Check for atomic groups
    if ($pattern =~ /\(\?>/) {
        return { compatible => 0, reason => "Contains atomic groups (?>...)" };
    }
    
    # Check for conditional patterns
    if ($pattern =~ /\(\?\(/) {
        return { compatible => 0, reason => "Contains conditional patterns (?(condition)yes|no)" };
    }
    
    # Pattern appears to be compatible
    return { compatible => 1, reason => "Compatible with Rust regex crate" };
}

#------------------------------------------------------------------------------
# Clean tag name by removing module prefixes
#------------------------------------------------------------------------------
sub clean_tag_name {
    my ($tag_name) = @_;
    
    # Remove module prefixes like "Exif-", "GPS-", etc.
    $tag_name =~ s/^[A-Za-z]+-//;
    
    return $tag_name;
}

1;  # Module must return true