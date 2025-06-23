//! Debug tool to extract and display raw XMP content

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
            if metadata.xmp.is_empty() {
                println!("No XMP segments found");
                return;
            }

            for (i, xmp) in metadata.xmp.iter().enumerate() {
                println!("=== XMP Segment {} ===", i);
                println!("Data length: {} bytes", xmp.data.len());

                // The data includes the signature, let's skip it
                let xmp_signature = b"http://ns.adobe.com/xap/1.0/\0";
                let ext_signature = b"http://ns.adobe.com/xmp/extension/\0";

                let content_start = if xmp.data.starts_with(xmp_signature) {
                    xmp_signature.len()
                } else if xmp.data.starts_with(ext_signature) {
                    ext_signature.len()
                } else {
                    0
                };

                if content_start < xmp.data.len() {
                    let content = &xmp.data[content_start..];

                    // Try UTF-8 first
                    if let Ok(s) = std::str::from_utf8(content) {
                        println!("{}", s);
                    } else {
                        // Try UTF-16 BE (the pattern 00 XX suggests big-endian)
                        if content.len() % 2 == 0 {
                            let mut utf16_chars = Vec::new();
                            for chunk in content.chunks_exact(2) {
                                let code_unit = u16::from_be_bytes([chunk[0], chunk[1]]);
                                utf16_chars.push(code_unit);
                            }

                            match String::from_utf16(&utf16_chars) {
                                Ok(s) => println!("{}", s),
                                Err(_) => {
                                    println!("(Not valid UTF-8 or UTF-16)");
                                    // Try to print first 200 bytes as hex
                                    let preview_len = content.len().min(200);
                                    println!("First {} bytes as hex:", preview_len);
                                    for (i, byte) in content[..preview_len].iter().enumerate() {
                                        if i % 16 == 0 {
                                            print!("\n{:04x}: ", i);
                                        }
                                        print!("{:02x} ", byte);
                                    }
                                    println!();
                                }
                            }
                        } else {
                            println!("(Odd number of bytes, cannot decode as UTF-16)");
                        }
                    }
                } else {
                    println!("(No content after signature)");
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
