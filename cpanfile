requires 'JSON', '4.00';
requires 'FindBin';

# codegen/scripts/field_extractor.pl and ppi_ast.pl
requires 'JSON::XS';
requires 'PPI';

on 'develop' => sub {
    requires 'Test::More', '0.96';
    requires 'Test::Deep';
    requires 'Data::Dumper';
};
