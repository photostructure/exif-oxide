#!/usr/bin/env perl

# Extract composite tag dependencies directly from ExifTool modules
# This script handles the Composite hash specially since it contains code refs

use strict;
use warnings;
use lib 'third-party/exiftool/lib';
use JSON::XS;
use Scalar::Util qw(reftype);

my $json = JSON::XS->new->canonical(1)->pretty(1);

# Load modules that have composite tags
my @modules = (
    'Image::ExifTool',    # Main module with Composite hash
    'Image::ExifTool::GPS',
    'Image::ExifTool::Exif',
    'Image::ExifTool::Canon',
    'Image::ExifTool::Nikon',
    'Image::ExifTool::Sony',
);

my %all_composites;

foreach my $module (@modules) {
    eval "require $module";
    if ($@) {
        warn "Failed to load $module: $@\n";
        next;
    }

    # Look for Composite hash in this module
    no strict 'refs';
    my $composite_hash = \%{"${module}::Composite"};

    if (%$composite_hash) {
        print STDERR "Found composite tags in $module\n";

        foreach my $tag ( keys %$composite_hash ) {
            my $tag_def = $composite_hash->{$tag};

            # Skip if not a hash ref (shouldn't happen, but be safe)
            next unless ref($tag_def) eq 'HASH';

            my $entry = {
                source  => $module,
                require => extract_deps( $tag_def->{Require} ),
                desire  => extract_deps( $tag_def->{Desire} ),
                inhibit => extract_deps( $tag_def->{Inhibit} ),
            };

            # Extract expressions if they're strings
            if ( exists $tag_def->{ValueConv} && !ref( $tag_def->{ValueConv} ) )
            {
                $entry->{value_conv} = $tag_def->{ValueConv};
            }
            if ( exists $tag_def->{PrintConv} && !ref( $tag_def->{PrintConv} ) )
            {
                $entry->{print_conv} = $tag_def->{PrintConv};
            }

            # Only include if there are dependencies
            if (   @{ $entry->{require} }
                || @{ $entry->{desire} }
                || @{ $entry->{inhibit} } )
            {
                $all_composites{$tag} = $entry;
            }
        }
    }
}

# Output the results
my $output = {
    '_metadata' => {
        'description' => 'Composite tag dependencies extracted from ExifTool',
        'generated'   => scalar(localtime),
        'total_tags'  => scalar( keys %all_composites ),
    },
    'tags' => \%all_composites,
};

print $json->encode($output);

# Helper to extract dependency arrays
sub extract_deps {
    my ($deps) = @_;
    return [] unless defined $deps;

    if ( ref $deps eq 'ARRAY' ) {

        # Simple array of tag names
        return [ map { clean_tag_name($_) } @$deps ];
    }
    elsif ( ref $deps eq 'HASH' ) {

        # Hash with numeric keys (common in ExifTool)
        my @tags;
        foreach my $key ( sort { $a <=> $b } keys %$deps ) {
            push @tags, clean_tag_name( $deps->{$key} );
        }
        return \@tags;
    }
    elsif ( !ref $deps ) {

        # Single string dependency
        return [ clean_tag_name($deps) ];
    }
    else {
        return [];
    }
}

# Remove group prefix from tag names
sub clean_tag_name {
    my ($tag) = @_;
    return '' unless defined $tag;

    # Remove group prefix like "EXIF:" or "File:"
    $tag =~ s/^[A-Z][a-z]+://;
    return $tag;
}
