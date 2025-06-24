//! Test MPF (Multi-Picture Format) extraction

use exif_oxide::binary::extract_mpf_preview;
use exif_oxide::core::{find_all_metadata_segments, mpf::ParsedMpf};
use std::fs;
use std::path::Path;

#[test]
fn test_mpf_extraction_from_canon_r50() {
    // Try to find a test image with MPF data
    let test_images = [
        "test-images/canon/canon_eos_r50v_01.jpg",
        "test-images/canon/Canon_T3i.JPG",
        "test-images/canon/Canon.jpg",
    ];

    for image_path in &test_images {
        if !Path::new(image_path).exists() {
            eprintln!("Test image not found: {}", image_path);
            continue;
        }

        println!("\nTesting MPF extraction from: {}", image_path);

        // Find all metadata segments
        let metadata = find_all_metadata_segments(image_path).unwrap();

        // Check if MPF segment exists
        if let Some(mpf_segment) = metadata.mpf {
            println!("Found MPF segment at offset: {}", mpf_segment.offset);

            // Parse MPF data
            let mpf = ParsedMpf::parse(mpf_segment.data).unwrap();

            // Print MPF information
            if let Some(num_images) = mpf.number_of_images() {
                println!("Number of images in MPF: {}", num_images);
            }

            println!("MPF images found: {}", mpf.images.len());
            for (i, image) in mpf.images.iter().enumerate() {
                println!(
                    "  Image {}: Type={:?}, Length={}, Offset={}",
                    i, image.image_type, image.length, image.offset
                );
            }

            // Try to extract the preview
            let file_data = fs::read(image_path).unwrap();
            if let Ok(Some(preview_data)) =
                extract_mpf_preview(&mpf, &file_data, mpf_segment.offset as usize)
            {
                println!(
                    "Successfully extracted MPF preview: {} bytes",
                    preview_data.len()
                );

                // Verify it's a valid JPEG
                if preview_data.len() > 2 && &preview_data[0..2] == b"\xFF\xD8" {
                    println!("Preview is a valid JPEG");
                }
            } else {
                println!("No preview found in MPF");
            }
        } else {
            println!("No MPF segment found");
        }
    }
}

#[test]
fn test_mpf_detection_in_jpeg() {
    use exif_oxide::core::jpeg;
    use std::io::Cursor;

    // Create a minimal JPEG with MPF APP2 segment
    let mut jpeg_data = vec![];
    jpeg_data.extend_from_slice(&[0xFF, 0xD8]); // SOI
    jpeg_data.extend_from_slice(&[0xFF, 0xE2]); // APP2 marker

    // Calculate exact length: MPF(4) + TIFF_header(4) + IFD_offset(4) + Entry_count(2) + Next_IFD(4) = 18 bytes
    // Length field includes itself (2 bytes), so total segment length = 18 + 2 = 20 bytes
    jpeg_data.extend_from_slice(&[0x00, 0x14]); // Length = 20 bytes
    jpeg_data.extend_from_slice(b"MPF\0"); // MPF signature (4 bytes)
    jpeg_data.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00]); // TIFF header (little-endian, 4 bytes)
    jpeg_data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset = 8 (4 bytes)
                                                            // Minimal IFD with 0 entries
    jpeg_data.extend_from_slice(&[0x00, 0x00]); // Entry count = 0 (2 bytes)
    jpeg_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Next IFD = None (4 bytes)

    jpeg_data.extend_from_slice(&[0xFF, 0xD9]); // EOI

    // Test MPF detection
    let mut cursor = Cursor::new(&jpeg_data);
    let metadata = jpeg::find_metadata_segments(&mut cursor).unwrap();

    assert!(metadata.mpf.is_some(), "MPF segment should be detected");
    let mpf = metadata.mpf.unwrap();
    assert_eq!(mpf.offset, 10); // After SOI(2) + APP2_marker(2) + Length(2) + MPF_signature(4) = 10
}
