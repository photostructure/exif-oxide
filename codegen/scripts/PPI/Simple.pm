package PPI::Simple;

# Simple PPI-to-JSON AST converter following PPI::Dumper pattern
# Much simpler than the intern's overly-complex visitor pattern

use strict;
use warnings;
use PPI;
use JSON::XS;
use Scalar::Util qw(blessed);

our $VERSION = '1.0';

# Create a new converter
sub new {
    my ($class, %options) = @_;
    
    my $self = bless {
        include_locations => $options{include_locations} // 0,
        include_content => $options{include_content} // 1,
        skip_whitespace => $options{skip_whitespace} // 1,
        skip_comments => $options{skip_comments} // 1,
        max_depth => $options{max_depth} // 20,
    }, $class;
    
    return $self;
}

# Convert PPI element to simple hash structure
sub convert_to_hash {
    my ($self, $element) = @_;
    
    return undef unless $element;
    return $self->_convert_element($element, 0);
}

# Convert PPI element to JSON string
sub convert_to_json {
    my ($self, $element) = @_;
    
    my $hash = $self->convert_to_hash($element);
    return undef unless $hash;
    
    my $json = JSON::XS->new->canonical(1)->pretty(0);
    return $json->encode($hash);
}

# Parse Perl expression and return AST
sub parse_expression {
    my ($self, $expr_string) = @_;
    
    return undef unless defined $expr_string && length($expr_string) > 0;
    
    # Skip very long expressions to avoid performance issues
    return undef if length($expr_string) > 1000;
    
    # Try to parse as a document fragment
    my $document;
    eval {
        $document = PPI::Document->new(\$expr_string);
    };
    
    if ($@ || !$document) {
        # Try as fragment for partial expressions
        eval {
            $document = PPI::Document::Fragment->new(\$expr_string);
        };
        return undef if ($@ || !$document);
    }
    
    return $self->convert_to_hash($document);
}

# Internal recursive converter - follows PPI::Dumper pattern exactly
sub _convert_element {
    my ($self, $element, $depth) = @_;
    
    return { type => 'max_depth_exceeded' } if $depth > $self->{max_depth};
    
    # Should we skip this element?
    my $should_skip = 0;
    if (blessed($element)) {
        if ($self->{skip_whitespace} && $element->isa('PPI::Token::Whitespace')) {
            $should_skip = 1;
        } elsif ($self->{skip_comments} && $element->isa('PPI::Token::Comment')) {
            $should_skip = 1;
        }
    }
    
    return undef if $should_skip;
    
    # Build base element structure
    my $result = {
        class => ref($element),
    };
    
    # Add location if requested and available
    if ($self->{include_locations} && blessed($element) && $element->isa('PPI::Token')) {
        my $location = $element->location;
        if ($location) {
            $result->{location} = {
                line => $location->[0],
                rowchar => $location->[1], 
                column => $location->[2],
            };
        }
    }
    
    # Handle content for tokens
    if (blessed($element) && $element->isa('PPI::Token') && $self->{include_content}) {
        my $content = $element->content;
        # Clean up content for JSON
        $content =~ s/\n/\\n/g;
        $content =~ s/\t/\\t/g;
        $content =~ s/\r/\\r/g;
        $result->{content} = $content;
        
        # For specific token types, add semantic info
        if ($element->isa('PPI::Token::Symbol')) {
            $result->{symbol_type} = $self->_classify_symbol($content);
        } elsif ($element->isa('PPI::Token::Number')) {
            $result->{numeric_value} = $element->literal;
        } elsif ($element->isa('PPI::Token::Quote')) {
            $result->{string_value} = $element->string;
        }
    }
    
    # Handle structure boundaries
    if (blessed($element) && $element->isa('PPI::Structure') && $self->{include_content}) {
        my $start = $element->start ? $element->start->content : '???';
        my $finish = $element->finish ? $element->finish->content : '???';
        $result->{structure_bounds} = "$start ... $finish";
    }
    
    # Recurse into children for nodes (exactly like PPI::Dumper)
    if (blessed($element) && $element->isa('PPI::Node')) {
        my @children = ();
        
        # Access children directly like PPI::Dumper does
        if ($element->{children}) {
            foreach my $child (@{$element->{children}}) {
                my $converted_child = $self->_convert_element($child, $depth + 1);
                push @children, $converted_child if defined $converted_child;
            }
        }
        
        $result->{children} = \@children if @children;
    }
    
    return $result;
}

# Classify symbol type for semantic understanding
sub _classify_symbol {
    my ($self, $symbol) = @_;
    
    return 'scalar' if $symbol =~ /^\$/;
    return 'array' if $symbol =~ /^\@/;
    return 'hash' if $symbol =~ /^\%/;
    return 'glob' if $symbol =~ /^\*/;
    return 'unknown';
}

# Extract semantic patterns from PPI tree (simplified)
sub extract_patterns {
    my ($self, $element) = @_;
    
    my $patterns = {
        has_variables => 0,
        has_self_refs => 0,
        has_conditionals => 0,
        has_operators => 0,
        has_function_calls => 0,
        variables => [],
        functions => [],
    };
    
    $self->_extract_patterns_recursive($element, $patterns);
    return $patterns;
}

sub _extract_patterns_recursive {
    my ($self, $element, $patterns) = @_;
    
    return unless $element;
    
    # Check current element
    if (blessed($element)) {
        if ($element->isa('PPI::Token::Symbol')) {
            $patterns->{has_variables} = 1;
            my $symbol = $element->content;
            push @{$patterns->{variables}}, $symbol;
            
            if ($symbol =~ /^\$\$self/) {
                $patterns->{has_self_refs} = 1;
            }
        } elsif ($element->isa('PPI::Token::Operator')) {
            my $op = $element->content;
            $patterns->{has_operators} = 1;
            
            if ($op =~ /^(\?|:)$/) {
                $patterns->{has_conditionals} = 1;
            }
        } elsif ($element->isa('PPI::Token::Word')) {
            my $word = $element->content;
            
            # Check if this looks like a function call (word followed by parentheses)
            my $next = $element->next_sibling;
            if ($next && blessed($next) && $next->isa('PPI::Structure::List')) {
                $patterns->{has_function_calls} = 1;
                push @{$patterns->{functions}}, $word;
            }
        }
    }
    
    # Recurse into children
    if (blessed($element) && $element->isa('PPI::Node')) {
        foreach my $child (@{$element->{children} || []}) {
            $self->_extract_patterns_recursive($child, $patterns);
        }
    }
}

1;

__END__

=head1 NAME

PPI::Simple - Simplified PPI-to-JSON converter following PPI::Dumper patterns

=head1 SYNOPSIS

    use PPI::Simple;
    
    my $converter = PPI::Simple->new(
        skip_whitespace => 1,
        include_locations => 0
    );
    
    my $ast = $converter->parse_expression('$$self{Make} eq "Canon"');
    my $json = $converter->convert_to_json($ast);

=head1 DESCRIPTION

This module provides a simple, clean way to convert PPI parse trees to JSON
format for consumption by Rust code generators. It follows the same recursive
pattern as PPI::Dumper but outputs structured data instead of debug strings.

The key principle is simplicity - no complex visitor patterns, no deep 
recursion, just a clean traverse-and-convert approach.

=cut