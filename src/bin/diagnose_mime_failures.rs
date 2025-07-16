//! Diagnostic tool to analyze MIME type detection failures in exif-oxide
//!
//! This tool helps debug why certain file formats (particularly JXL) might fail
//! MIME type detection, and compares our implementation against ExifTool's logic.

use exif_oxide::file_detection::{FileDetectionError, FileTypeDetector};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test file paths to diagnose
    let test_files = vec![
        "third-party/exiftool/t/images/JXL2.jxl",
        "third-party/exiftool/t/images/JXL.jxl",
        "test-images/nikon/nikon_z8_73.NEF",
    ];

    println!("=== ExifTool File Detection Logic Analysis ===\n");
    println!("Based on ExifTool.pm:2913-2999, the detection flow is:");
    println!("1. Get extension-based candidates (GetFileType)");
    println!("2. Read test buffer (1024 bytes)");
    println!("3. For each candidate type:");
    println!("   a. If has magic pattern: test against buffer (line 2964)");
    println!("   b. If no magic pattern but has module: skip if extension-only (line 2967)");
    println!("   c. If weak magic: skip if recognizedExt exists (line 2970)");
    println!("4. If no match and recognizedExt: use extension type (line 2974)");
    println!("5. Last resort: scan for embedded JPEG/TIFF (line 2977)\n");

    println!("Key variables:");
    println!("- recognizedExt: Set when extension is known but has no magic pattern");
    println!("- weakMagic: Only MP3 is marked as weak");
    println!("- noMagic: MXF and DV skip magic tests\n");

    // Analyze JXL specifically
    println!("=== JXL Format Analysis ===");
    println!("From ExifTool source:");
    println!("- Magic pattern: '(\\xff\\x0a|\\0\\0\\0\\x0cJXL \\x0d\\x0a......ftypjxl )'");
    println!("- Module: 'Jpeg2000' (JXL uses JPEG 2000 processing)");
    println!("- MIME type: 'image/jxl'");
    println!("- Description: 'JPEG XL'\n");

    println!("JXL has two possible magic signatures:");
    println!("1. Bare codestream: starts with 0xFF 0x0A");
    println!(
        "2. ISO BMFF container: starts with '\\0\\0\\0\\x0cJXL \\x0d\\x0a' followed by 'ftypjxl '"
    );
    println!("   - This is 12 bytes: 00 00 00 0C 4A 58 4C 20 0D 0A XX XX XX XX 66 74 79 70 6A 78 6C 20\n");

    // Test actual files if provided
    if !test_files.is_empty() {
        println!("=== Testing Actual Files ===\n");

        for file_path in test_files {
            if Path::new(file_path).exists() {
                diagnose_file(file_path)?;
            } else {
                println!("File not found: {file_path}");
            }
        }
    }

    // Demonstrate the detection algorithm
    println!("\n=== Detection Algorithm Walkthrough ===");
    println!("For a .jxl file, the algorithm should:");
    println!("1. Normalize extension 'jxl' -> 'JXL'");
    println!("2. Get candidates: ['JXL'] (since JXL is in fileTypeLookup)");
    println!("3. Read buffer and test JXL magic pattern");
    println!("4. If magic matches -> detect as JXL");
    println!("5. If magic fails but extension is JXL:");
    println!("   - Since JXL has a magic pattern defined, it won't use extension-only fallback");
    println!("   - This could cause detection to fail!\n");

    println!("=== Potential Issues ===");
    println!("1. If JXL file doesn't match the magic pattern, detection fails");
    println!("2. Unlike formats without magic patterns, JXL can't fallback to extension");
    println!("3. The regex pattern might not match all valid JXL files");
    println!("4. Binary pattern matching in Rust might differ from Perl regex behavior\n");

    Ok(())
}

fn diagnose_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Diagnosing: {file_path}");

    let path = Path::new(file_path);
    let mut file = BufReader::new(File::open(path)?);

    // Read first 1024 bytes for analysis
    let mut buffer = vec![0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    println!("  Extension: {:?}", path.extension());
    println!("  File size: {bytes_read} bytes read");

    // Show first 32 bytes in hex
    print!("  First 32 bytes: ");
    for (i, &byte) in buffer.iter().take(32).enumerate() {
        if i > 0 && i % 4 == 0 {
            print!(" ");
        }
        print!("{byte:02X}");
    }
    println!();

    // Check for JXL signatures
    if buffer.len() >= 2 && buffer[0] == 0xFF && buffer[1] == 0x0A {
        println!("  ✓ Detected JXL bare codestream signature (FF 0A)");
    } else if buffer.len() >= 20 {
        let sig_start = &buffer[0..12];
        let ftyp_start = if buffer.len() >= 20 {
            &buffer[16..20]
        } else {
            &[]
        };

        if sig_start == b"\x00\x00\x00\x0CJXL \x0D\x0A" {
            println!("  ✓ Detected JXL ISO BMFF container signature");
            if ftyp_start == b"ftyp" {
                println!("  ✓ Found ftyp box at expected position");
                if buffer.len() >= 24 && &buffer[20..24] == b"jxl " {
                    println!("  ✓ Found 'jxl ' brand in ftyp box");
                }
            }
        }
    }

    // Try detection
    let detector = FileTypeDetector::new();
    let mut file = BufReader::new(File::open(path)?);

    match detector.detect_file_type(path, &mut file) {
        Ok(result) => {
            println!("\n  Detection succeeded:");
            println!("    File type: {}", result.file_type);
            println!("    Format: {}", result.format);
            println!("    MIME type: {}", result.mime_type);
            println!("    Description: {}", result.description);
        }
        Err(e) => {
            println!("\n  Detection failed: {e:?}");

            // Additional diagnostics for failure
            match e {
                FileDetectionError::UnknownFileType => {
                    println!("    → No magic pattern matched and no fallback available");
                }
                FileDetectionError::IoError(io_err) => {
                    println!("    → IO error: {io_err}");
                }
                FileDetectionError::InvalidPath => {
                    println!("    → Invalid file path");
                }
            }
        }
    }

    println!();
    Ok(())
}
