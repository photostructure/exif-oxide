#!/usr/bin/env perl
use strict;
use warnings;
use PPI;

# Parse the specific failing expression
my $code     = 'sprintf("%19d %4d %6d" . " %3d %4d %6d" x 8, split(" ",$val))';
my $document = PPI::Document->new( \$code );

# Print the AST structure
sub dump_node {
    my ( $node, $indent ) = @_;
    $indent //= 0;
    my $spaces = "  " x $indent;

    if ( $node->isa('PPI::Node') ) {
        print $spaces . ref($node) . "\n";
        for my $child ( $node->children ) {
            dump_node( $child, $indent + 1 );
        }
    }
    else {
        my $content      = $node->content // '';
        my $string_value = '';
        if ( $node->can('string') && $node->string ) {
            $string_value = ' string_value="' . $node->string . '"';
        }
        print $spaces
          . ref($node)
          . ' content="'
          . $content . '"'
          . $string_value . "\n";
    }
}

print "AST structure for: $code\n";
print "=" x 60 . "\n";
dump_node($document);
