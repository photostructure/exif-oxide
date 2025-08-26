#!/usr/bin/env perl

# Script to analyze PPI AST coverage from test JSON files
# Extracts expressions from JSON configs and outputs unique PPI classes

use strict;
use warnings;
use FindBin qw($Bin);
use JSON::XS;
use File::Find;

# Add our PPI library to path
use lib "$Bin";
use PPI::Simple;

my $config_dir = "$Bin/../tests/config";
my %unique_classes;
my %expressions_by_class;
my @all_expressions;

# Find all JSON test files
find(
    sub {
        return unless /\.json$/ && $_ ne 'schema.json';
        my $path = $File::Find::name;

        # Read JSON file
        open( my $fh, '<', $path ) or die "Cannot read $path: $!";
        my $json_text = do { local $/; <$fh> };
        close($fh);

        # Parse JSON
        my $config = eval { decode_json($json_text) };
        if ( $@ || !$config->{expression} ) {
            warn "Skipping $path: $@\n" if $@;
            return;
        }

        my $expression = $config->{expression};
        my $is_skip    = ( $File::Find::name =~ /SKIP_/ ) ? 1 : 0;

        push @all_expressions,
          {
            expr => $expression,
            file => $path,
            skip => $is_skip
          };

    },
    $config_dir
);

# Create PPI converter
my $ppi_converter = PPI::Simple->new(
    skip_whitespace   => 1,
    skip_comments     => 1,
    include_locations => 0,
    include_content   => 1,
);

# Process each expression
for my $entry (@all_expressions) {
    my $expr = $entry->{expr};
    my $file = $entry->{file};
    my $skip = $entry->{skip};

    # Parse the expression
    my $ast = $ppi_converter->parse_expression($expr);

    if ( !$ast ) {
        warn "Failed to parse expression from $file: $expr\n";
        next;
    }

    # Extract all unique classes recursively
    extract_classes( $ast, $expr, $skip );
}

sub extract_classes {
    my ( $node, $expr, $skip ) = @_;

    if ( ref($node) eq 'HASH' ) {
        if ( $node->{class} ) {
            my $class = $node->{class};
            $unique_classes{$class}{count}++;
            $unique_classes{$class}{skip}++    if $skip;
            $unique_classes{$class}{working}++ if !$skip;

            # Store example expression
            if ( !$expressions_by_class{$class}
                || length($expr) < length( $expressions_by_class{$class} ) )
            {
                $expressions_by_class{$class} = $expr;
            }
        }

        # Recurse through children
        if ( $node->{children} && ref( $node->{children} ) eq 'ARRAY' ) {
            for my $child ( @{ $node->{children} } ) {
                extract_classes( $child, $expr, $skip );
            }
        }
    }
    elsif ( ref($node) eq 'ARRAY' ) {
        for my $item (@$node) {
            extract_classes( $item, $expr, $skip );
        }
    }
}

# Output results
print "=== PPI AST Symbol Coverage Analysis ===\n\n";

print "Total expressions analyzed: " . scalar(@all_expressions) . "\n";
my $working = grep { !$_->{skip} } @all_expressions;
my $skipped = grep { $_->{skip} } @all_expressions;
print "Working: $working, Skipped: $skipped\n\n";

print "=== Unique PPI Classes Found ===\n\n";

# Sort by frequency
for my $class (
    sort { $unique_classes{$b}{count} <=> $unique_classes{$a}{count} }
    keys %unique_classes
  )
{
    my $info   = $unique_classes{$class};
    my $status = "";
    if ( $info->{working} && $info->{skip} ) {
        $status = "MIXED";
    }
    elsif ( $info->{working} ) {
        $status = "WORKING";
    }
    elsif ( $info->{skip} ) {
        $status = "SKIP";
    }

    printf "%-45s Count:%3d Status:%-7s Example: %s\n",
      $class,
      $info->{count},
      $status,
      substr( $expressions_by_class{$class} || "", 0, 40 );
}

print "\n=== Summary ===\n";
my $total_classes = scalar( keys %unique_classes );
my $working_classes =
  grep { $unique_classes{$_}{working} } keys %unique_classes;
my $skip_only_classes =
  grep { !$unique_classes{$_}{working} && $unique_classes{$_}{skip} }
  keys %unique_classes;

print "Total unique PPI classes: $total_classes\n";
print "Classes in working tests: $working_classes\n";
print "Classes only in SKIP tests: $skip_only_classes\n";
