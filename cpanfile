requires 'JSON', '4.00';
requires 'FindBin';

on 'develop' => sub {
    requires 'Test::More', '0.96';
    requires 'Test::Deep';
    requires 'Data::Dumper';
};