requires 'JSON', '4.00';
requires 'FindBin';
requires 'File::Basename';
requires 'File::Path';
requires 'File::Spec';
requires 'Getopt::Long';
requires 'Cwd';

on 'develop' => sub {
    requires 'Test::More', '0.96';
    requires 'Test::Deep';
    requires 'Data::Dumper';
};