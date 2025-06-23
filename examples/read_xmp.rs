//! Example of reading XMP metadata from JPEG files

use exif_oxide::extract_xmp_properties;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <jpeg-file>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    println!("Reading XMP from: {}", filename);
    println!("{}", "=".repeat(50));

    match extract_xmp_properties(filename) {
        Ok(properties) => {
            if properties.is_empty() {
                println!("No XMP metadata found in file.");
                println!(
                    "(Note: This might mean the XMP uses features not yet supported in Phase 1)"
                );
            } else {
                println!("Found {} XMP properties:", properties.len());
                println!();

                // Group by namespace
                let mut by_namespace: std::collections::HashMap<String, Vec<(String, String)>> =
                    std::collections::HashMap::new();

                for (key, value) in properties {
                    if let Some((ns, prop)) = key.split_once(':') {
                        by_namespace
                            .entry(ns.to_string())
                            .or_default()
                            .push((prop.to_string(), value));
                    }
                }

                // Display grouped by namespace
                for (ns, props) in by_namespace {
                    println!("{}:", ns);
                    for (prop, value) in props {
                        println!("  {}: {}", prop, value);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading XMP: {}", e);
            process::exit(1);
        }
    }
}
