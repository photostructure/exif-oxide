//! Example of reading nested XMP metadata (Phase 2)

use exif_oxide::xmp::{parse_xmp, read_xmp_from_jpeg, XmpArray, XmpValue};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <jpeg-file>", args[0]);
        return;
    }

    match read_xmp_from_jpeg(&args[1]) {
        Ok(Some(packet)) => {
            println!("=== XMP Packet ===");

            // Parse the XMP
            match parse_xmp(&packet.standard) {
                Ok(metadata) => {
                    // Print namespaces
                    println!("\nNamespaces:");
                    for (prefix, uri) in &metadata.namespaces {
                        println!("  {}: {}", prefix, uri);
                    }

                    // Print properties by namespace
                    for (namespace, properties) in &metadata.properties {
                        println!("\n{}:", namespace);

                        for (name, value) in properties {
                            print_value(&format!("  {}", name), value, 2);
                        }
                    }
                }
                Err(e) => eprintln!("Failed to parse XMP: {}", e),
            }
        }
        Ok(None) => {
            println!("No XMP metadata found");
        }
        Err(e) => eprintln!("Error reading XMP: {}", e),
    }
}

fn print_value(prefix: &str, value: &XmpValue, indent: usize) {
    match value {
        XmpValue::Simple(text) => {
            println!("{}: {}", prefix, text);
        }
        XmpValue::Array(array) => match array {
            XmpArray::Ordered(values) => {
                println!("{}: [Ordered array with {} items]", prefix, values.len());
                for (i, v) in values.iter().enumerate() {
                    print_value(&format!("{}  [{}]", " ".repeat(indent), i), v, indent + 2);
                }
            }
            XmpArray::Unordered(values) => {
                println!("{}: [Unordered array with {} items]", prefix, values.len());
                for v in values.iter() {
                    print_value(&format!("{}  -", " ".repeat(indent)), v, indent + 2);
                }
            }
            XmpArray::Alternative(alts) => {
                println!(
                    "{}: [Alternative array with {} languages]",
                    prefix,
                    alts.len()
                );
                for alt in alts {
                    print_value(
                        &format!("{}  [{}]", " ".repeat(indent), alt.lang),
                        &alt.value,
                        indent + 2,
                    );
                }
            }
        },
        XmpValue::Struct(map) => {
            println!("{}: [Struct with {} fields]", prefix, map.len());
            for (field, value) in map {
                print_value(
                    &format!("{}  {}", " ".repeat(indent), field),
                    value,
                    indent + 2,
                );
            }
        }
    }
}
