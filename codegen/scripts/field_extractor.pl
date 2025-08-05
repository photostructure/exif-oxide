#!/usr/bin/env perl

use strict;
use warnings;
use FindBin qw($Bin);
use File::Basename;
use JSON::XS;
use Sub::Util qw(subname);
use Scalar::Util qw(blessed reftype refaddr);

# Add ExifTool lib directory to @INC
use lib "$Bin/../../third-party/exiftool/lib";

# Global counters and debugging
my $total_symbols     = 0;
my $extracted_symbols = 0;
my $skipped_symbols   = 0;
my @skipped_list      = ();

# JSON serializer - let JSON::XS handle complex structures automatically
my $json = JSON::XS->new->canonical(1)->allow_blessed(1)->convert_blessed(1)->allow_nonref(1);

if ( @ARGV != 1 ) {
    die "Usage: $0 <module_path>\n";
}

my $module_path = $ARGV[0];

# Extract module name from path
my $module_name = basename($module_path);
$module_name =~ s/\.pm$//;

print STDERR "Universal extraction starting for $module_name:\n";

# Load the module - handle special case for main ExifTool.pm
my $package_name;
if ($module_name eq 'ExifTool') {
    # Main ExifTool.pm uses package "Image::ExifTool"
    $package_name = "Image::ExifTool";
} else {
    # All other modules use "Image::ExifTool::ModuleName"
    $package_name = "Image::ExifTool::$module_name";
}

eval "require $package_name";
if ($@) {
    die "Failed to load module $package_name: $@\n";
}

# Extract symbols
extract_symbols( $package_name, $module_name );

# Print summary
print STDERR "Universal extraction complete for $module_name:\n";
print STDERR "  Total symbols examined: $total_symbols\n";
print STDERR "  Successfully extracted: $extracted_symbols\n";
print STDERR "  Skipped (non-serializable): $skipped_symbols\n";

# Print debug info about skipped symbols if requested
if ($ENV{DEBUG} && @skipped_list) {
    print STDERR "\nSkipped symbols:\n";
    for my $skipped (@skipped_list) {
        print STDERR "  - $skipped\n";
    }
}

sub extract_symbols {
    my ( $package_name, $module_name ) = @_;

    # Get module's symbol table
    no strict 'refs';
    my $symbol_table = *{"${package_name}::"};

    # Examine each symbol in the package
    foreach my $symbol_name ( sort keys %$symbol_table ) {
        $total_symbols++;
        print STDERR "  Processing symbol: $symbol_name\n" if $ENV{DEBUG};

        my $glob = $symbol_table->{$symbol_name};

        # Try to extract hash symbols (most important for ExifTool)
        if ( my $hash_ref = *$glob{HASH} ) {
            if (%$hash_ref) {    # Skip empty hashes
                my $hash_size = scalar(keys %$hash_ref);
                print STDERR "    Hash found with $hash_size keys\n" if $ENV{DEBUG};
                extract_hash_symbol( $symbol_name, $hash_ref, $module_name );
                print STDERR "    Hash extraction completed for $symbol_name\n" if $ENV{DEBUG};
            }
        } else {
            print STDERR "    No hash found for $symbol_name\n" if $ENV{DEBUG};
        }
    }
}

sub extract_hash_symbol {
    my ( $symbol_name, $hash_ref, $module_name ) = @_;

    print STDERR "    Starting extraction for: $symbol_name\n" if $ENV{DEBUG};
    
    # Detect composite tables by name pattern
    my $is_composite = ($symbol_name eq 'Composite' || $symbol_name =~ /Composite$/);
    print STDERR "    Composite table: " . ($is_composite ? 'YES' : 'NO') . "\n" if $ENV{DEBUG};
    
    # Filter out function references (JSON::XS can't handle them)
    print STDERR "    Filtering code references...\n" if $ENV{DEBUG};
    my $filtered_data = filter_code_refs($hash_ref);
    print STDERR "    Code reference filtering completed\n" if $ENV{DEBUG};
    my $size = scalar(keys %$filtered_data);
    
    # Skip if no data after filtering
    return unless $size > 0;
    
    # For non-composite tables, apply size limit to prevent huge output
    if (!$is_composite && $size > 1000) {
        $skipped_symbols++;
        push @skipped_list, "$module_name:$symbol_name (size: $size)";
        print STDERR "  Skipping large symbol: $symbol_name (size: $size)\n" if $ENV{DEBUG};
        return;
    }

    my $symbol_data = {
        type     => $is_composite ? 'composite_hash' : 'hash',
        name     => $symbol_name,
        data     => $filtered_data,
        module   => $module_name,
        metadata => {
            size       => $size,
            complexity => $is_composite ? 'composite' : 'simple',
        }
    };

    eval {
        print $json->encode($symbol_data) . "\n";
        $extracted_symbols++;
        print STDERR "  Extracted: $symbol_name (type: " . ($is_composite ? 'composite' : 'simple') . ", size: $size)\n" if $ENV{DEBUG};
    };
    if ($@) {
        $skipped_symbols++;
        push @skipped_list, "$module_name:$symbol_name (JSON error: $@)";
        print STDERR "  Warning: Failed to serialize $symbol_name: $@\n";
    }
}

sub filter_code_refs {
    my ($data, $depth, $seen) = @_;
    $depth //= 0;
    $seen //= {};
    
    # Prevent excessive recursion depth
    return "[MaxDepth]" if $depth > 10;
    
    if (!ref($data)) {
        return $data;
    }
    elsif (reftype($data) eq 'CODE') {
        # Convert function reference to function name
        my $name = subname($data);
        return defined($name) ? "[Function: $name]" : "[Function: __ANON__]";
    }
    elsif (reftype($data) eq 'HASH') {
        # Check for circular references using memory address
        my $addr = refaddr($data);
        return "[Circular]" if $seen->{$addr};
        $seen->{$addr} = 1;
        
        my $filtered = {};
        for my $key (keys %$data) {
            # Check if this is a Table reference that could cause circularity
            # Use reftype to check physical type, ignoring blessing
            if ($key eq 'Table' && defined(reftype($data->{$key})) && reftype($data->{$key}) eq 'HASH') {
                # Replace Table references with string representation to break circularity
                # These are metadata pointers in ExifTool, not structural data
                if (blessed($data->{$key})) {
                    $filtered->{$key} = "[TableRef: " . blessed($data->{$key}) . "]";
                } else {
                    $filtered->{$key} = "[TableRef: HASH]";
                }
            } else {
                $filtered->{$key} = filter_code_refs($data->{$key}, $depth + 1, $seen);
            }
        }
        
        # Remove from seen after processing to allow legitimate re-references
        delete $seen->{$addr};
        return $filtered;
    }
    elsif (reftype($data) eq 'ARRAY') {
        my $filtered = [];
        for my $item (@$data) {
            push @$filtered, filter_code_refs($item, $depth + 1, $seen);
        }
        return $filtered;
    }
    elsif (reftype($data) eq 'SCALAR') {
        return "[ScalarRef: " . $$data . "]";
    }
    elsif (blessed($data)) {
        return "[Object: " . blessed($data) . "]";
    }
    else {
        # Fallback for other reference types
        my $ref_type = reftype($data) || ref($data) || 'UNKNOWN';
        return "[Ref: $ref_type]";
    }
}
