#!/usr/bin/env perl

# Simple tool to parse a Perl expression and output its PPI AST as JSON
# Usage: ppi_ast.pl 'expression'
# Example: ppi_ast.pl 'sprintf("%.2f s", $val) . ($val > 254.5/60 ? " or longer" : "")'

use strict;
use warnings;
use FindBin qw($Bin);
use JSON::XS;

# Add our PPI library to path
use lib "$Bin";
use PPI::Simple;

if ( @ARGV < 1 ) {
    die "Usage: $0 'perl_expression'\n"
      . "  Parse a Perl expression and output its PPI AST as JSON\n"
      . "  Examples:\n"
      . "    $0 '\$val / 100'\n"
      . "    $0 'sprintf(\"%.3f x %.3f mm\", split(\" \",\$val))'\n"
      . "    $0 'length \$val'\n";
}

my $expression = join( ' ', @ARGV );

# Create PPI converter
my $ppi_converter = PPI::Simple->new(
    skip_whitespace   => 1,
    skip_comments     => 1,
    include_locations => 0,
    include_content   => 1,
);

# Parse the expression
my $ast = $ppi_converter->parse_expression($expression);

if ( !$ast ) {
    die "Failed to parse expression: $expression\n";
}

# Output as pretty JSON
my $json = JSON::XS->new->canonical(1)->pretty(1);
print $json->encode($ast);
