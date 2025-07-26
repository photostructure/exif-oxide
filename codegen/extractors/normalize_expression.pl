#!/usr/bin/env perl

=head1 NAME

normalize_expression.pl - Normalize Perl expressions for consistent lookup

=head1 SYNOPSIS

    echo 'sprintf( "%.1f mm" , $val )' | perl normalize_expression.pl
    # Output: sprintf("%.1f mm", $val)

=head1 DESCRIPTION

This script normalizes Perl expressions found in ExifTool's PrintConv and ValueConv
fields to ensure consistent whitespace and formatting for reliable registry lookups.

The script uses Perl::Tidy with specific formatting options to:
- Remove unnecessary whitespace around operators and punctuation
- Standardize spacing in function calls
- Split multi-statement expressions onto separate lines
- Apply consistent Perl formatting standards

=head2 Why This Approach?

Originally, we attempted to parse Perl expressions manually in Rust, which was:
- Error-prone (Perl syntax is complex with many edge cases)
- Difficult to maintain (hundreds of lines of parsing logic)
- Incomplete (couldn't handle all Perl constructs correctly)

Using Perl::Tidy leverages Perl's own proven parsing and formatting:
- Handles all Perl syntax correctly
- Produces consistent, readable output
- Only 4 lines of code vs 200+ lines of manual parsing
- Battle-tested by the Perl community

=head2 Perl::Tidy Options

The formatting options used are:
- C<-npro>: Don't read .perltidyrc configuration files
- C<-pt=2>: Tight parentheses (minimal spacing)
- C<-bt=2>: Tight square brackets
- C<-sbt=2>: Tight curly braces
- C<-ci=0>: No continuation indentation

=head1 DEPENDENCIES

Requires Perl::Tidy, which is installed via the project's cpanfile.

=head1 AUTHORS

exif-oxide team

=cut

use Perl::Tidy;
my $source = do { local $/; <STDIN> };
my $formatted;
Perl::Tidy::perltidy(source => \$source, destination => \$formatted, argv => '-npro -pt=2 -bt=2 -sbt=2 -ci=0');
print $formatted;