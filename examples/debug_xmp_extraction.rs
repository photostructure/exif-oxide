//! Debug XMP extraction

use exif_oxide::core::jpeg::find_metadata_segments;
use std::env;
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <jpeg-file>", args[0]);
        return;
    }

    let mut file = File::open(&args[1]).expect("Failed to open file");

    match find_metadata_segments(&mut file) {
        Ok(metadata) => {
            println!("EXIF found: {}", metadata.exif.is_some());
            println!("XMP segments found: {}", metadata.xmp.len());

            for (i, xmp) in metadata.xmp.iter().enumerate() {
                println!("\nXMP segment {}:", i);
                println!("  Offset: {:#x}", xmp.offset);
                println!("  Is extended: {}", xmp.is_extended);
                println!("  Data length: {}", xmp.data.len());

                // Try to parse as string and show first 100 chars
                if let Ok(s) = std::str::from_utf8(&xmp.data) {
                    let preview = if s.len() > 100 { &s[..100] } else { s };
                    println!("  Preview: {}", preview);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
