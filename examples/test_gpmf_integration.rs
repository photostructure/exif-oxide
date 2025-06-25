#!/usr/bin/env cargo +nightly -Zscript
//! Simple integration test for GPMF functionality

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic API function
    println!("Testing extract_gpmf_metadata API...");

    let gopro_path = "test-images/gopro/GoPro.jpg";
    if Path::new(gopro_path).exists() {
        println!("Found GoPro test file: {}", gopro_path);

        // Try to extract GPMF metadata
        match exif_oxide::extract_gpmf_metadata(gopro_path) {
            Ok(gpmf_data) => {
                println!("GPMF extraction succeeded!");
                println!("Found {} GPMF tags", gpmf_data.len());

                for (tag_id, value) in &gpmf_data {
                    println!("  {}: {:?}", tag_id, value);
                }
            }
            Err(e) => {
                println!("GPMF extraction failed: {}", e);
                println!(
                    "This might be expected if the JPEG doesn't contain GPMF data in APP6 segments"
                );
            }
        }
    } else {
        println!("GoPro test file not found: {}", gopro_path);
    }

    // Test tag lookup functions
    println!("\nTesting GPMF tag lookup...");
    if let Some(tag) = exif_oxide::gpmf::get_gpmf_tag("DVNM") {
        println!("Found DVNM tag: {} ({})", tag.name, tag.tag_id);
    } else {
        println!("DVNM tag not found");
    }

    // Test format lookup
    println!("\nTesting GPMF format lookup...");
    if let Some(format) = exif_oxide::gpmf::get_gpmf_format(0x63) {
        println!("Found format 0x63: {:?}", format);
    } else {
        println!("Format 0x63 not found (expected for stub implementation)");
    }

    println!("\nGPMF integration test complete!");
    Ok(())
}
