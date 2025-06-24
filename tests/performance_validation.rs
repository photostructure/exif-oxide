//! Performance validation integration tests
//!
//! Ensures that the multi-format support maintains the performance
//! targets established during Phase 1 development.

use exif_oxide::core::find_metadata_segment;
use exif_oxide::read_basic_exif;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

/// Target performance thresholds
const JPEG_TARGET_US: u128 = 10_000; // 10ms
const TIFF_TARGET_US: u128 = 15_000; // 15ms
const PNG_TARGET_US: u128 = 10_000; // 10ms
const RAW_TARGET_US: u128 = 20_000; // 20ms

/// Run a performance test and return microseconds
fn time_operation<F: FnOnce() -> R, R>(f: F) -> (R, u128) {
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed().as_micros();
    (result, elapsed)
}

#[test]
fn test_jpeg_performance_maintained() {
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        eprintln!("Skipping test - file not found: {}", test_file);
        return;
    }

    // Warm up
    let _ = find_metadata_segment(test_file);

    // Time metadata extraction
    let (result, elapsed) = time_operation(|| find_metadata_segment(test_file));

    assert!(result.is_ok(), "JPEG parsing should succeed");
    assert!(result.unwrap().is_some(), "Should find JPEG metadata");

    println!("JPEG metadata extraction: {}μs", elapsed);
    assert!(
        elapsed < JPEG_TARGET_US,
        "JPEG performance regression: {}μs > {}μs target",
        elapsed,
        JPEG_TARGET_US
    );
}

#[test]
fn test_basic_exif_performance() {
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        return;
    }

    // Warm up
    let _ = read_basic_exif(test_file);

    // Time full EXIF extraction
    let (result, elapsed) = time_operation(|| read_basic_exif(test_file));

    assert!(result.is_ok(), "Basic EXIF reading should succeed");

    println!("Basic EXIF extraction: {}μs", elapsed);
    assert!(
        elapsed < JPEG_TARGET_US * 2,
        "Basic EXIF performance issue: {}μs",
        elapsed
    );
}

#[test]
fn test_png_performance() {
    let test_file = "exiftool/t/images/PNG.png";
    if !Path::new(test_file).exists() {
        return;
    }

    let (result, elapsed) = time_operation(|| find_metadata_segment(test_file));

    assert!(result.is_ok(), "PNG parsing should not error");

    println!("PNG metadata extraction: {}μs", elapsed);
    assert!(
        elapsed < PNG_TARGET_US,
        "PNG performance issue: {}μs > {}μs target",
        elapsed,
        PNG_TARGET_US
    );
}

#[test]
fn test_tiff_performance() {
    let test_file = "exiftool/t/images/ExifTool.tif";
    if !Path::new(test_file).exists() {
        return;
    }

    let (result, elapsed) = time_operation(|| find_metadata_segment(test_file));

    assert!(result.is_ok(), "TIFF parsing should succeed");

    println!("TIFF metadata extraction: {}μs", elapsed);
    assert!(
        elapsed < TIFF_TARGET_US,
        "TIFF performance issue: {}μs > {}μs target",
        elapsed,
        TIFF_TARGET_US
    );
}

#[test]
fn test_raw_format_performance() {
    let test_cases = vec![
        ("exiftool/t/images/CanonRaw.cr2", "CR2"),
        ("exiftool/t/images/Nikon.nef", "NEF"),
        ("exiftool/t/images/Sony.arw", "ARW"),
    ];

    for (file_path, format_name) in test_cases {
        if !Path::new(file_path).exists() {
            continue;
        }

        let (result, elapsed) = time_operation(|| find_metadata_segment(file_path));

        assert!(result.is_ok(), "{} parsing should succeed", format_name);

        println!("{} metadata extraction: {}μs", format_name, elapsed);
        assert!(
            elapsed < RAW_TARGET_US,
            "{} performance issue: {}μs > {}μs target",
            format_name,
            elapsed,
            RAW_TARGET_US
        );
    }
}

#[test]
fn test_format_detection_overhead() {
    // Test that format detection adds minimal overhead
    let test_file = "exiftool/t/images/Canon.jpg";
    if !Path::new(test_file).exists() {
        return;
    }

    // Time just format detection
    let mut file = File::open(test_file).unwrap();
    let mut buffer = vec![0u8; 1024];

    let (_, detect_time) = time_operation(|| {
        file.read(&mut buffer).unwrap();
        exif_oxide::detection::detect_file_type(&buffer).unwrap()
    });

    println!("Format detection overhead: {}μs", detect_time);
    assert!(
        detect_time < 100,
        "Format detection too slow: {}μs",
        detect_time
    );
}

#[test]
fn test_memory_mode_performance_benefit() {
    use exif_oxide::core::tiff::{find_ifd_data_with_mode, TiffParseMode};

    let test_file = "exiftool/t/images/CanonRaw.cr2";
    if !Path::new(test_file).exists() {
        return;
    }

    // Time full file mode
    let mut file1 = File::open(test_file).unwrap();
    let (_, full_time) =
        time_operation(|| find_ifd_data_with_mode(&mut file1, TiffParseMode::FullFile));

    // Time metadata-only mode
    let mut file2 = File::open(test_file).unwrap();
    let (_, metadata_time) =
        time_operation(|| find_ifd_data_with_mode(&mut file2, TiffParseMode::MetadataOnly));

    println!("TIFF full mode: {}μs", full_time);
    println!("TIFF metadata mode: {}μs", metadata_time);
    println!(
        "Speed improvement: {:.1}x",
        full_time as f64 / metadata_time as f64
    );

    // For small files, the performance difference might not be significant
    // Check if there's any improvement or if both are fast enough
    if full_time > 1000 && metadata_time >= full_time {
        panic!("Metadata mode should be faster for large files");
    } else if full_time <= 1000 {
        println!("Both modes are fast enough for small files");
    }
}

#[test]
fn test_container_streaming_performance() {
    // Test that container parsers don't load entire files
    let test_file = "exiftool/t/images/MP4.mp4";
    if !Path::new(test_file).exists() {
        return;
    }

    let file_size = std::fs::metadata(test_file).unwrap().len();

    let (result, elapsed) = time_operation(|| find_metadata_segment(test_file));

    assert!(result.is_ok(), "MP4 parsing should not error");

    println!("MP4 ({} bytes) parsed in {}μs", file_size, elapsed);

    // Even large files should parse quickly due to streaming
    assert!(
        elapsed < 50_000,
        "Container parsing too slow: {}μs",
        elapsed
    );
}

#[test]
fn test_concurrent_format_parsing() {
    use std::thread;

    let formats = vec![
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/ExifTool.tif",
        "exiftool/t/images/PNG.png",
        "exiftool/t/images/CanonRaw.cr2",
    ];

    // Filter to existing files
    let existing_files: Vec<_> = formats
        .into_iter()
        .filter(|f| Path::new(f).exists())
        .collect();

    if existing_files.is_empty() {
        return;
    }

    let start = Instant::now();

    // Parse all formats concurrently
    let handles: Vec<_> = existing_files
        .iter()
        .map(|&file| thread::spawn(move || find_metadata_segment(file)))
        .collect();

    // Wait for all to complete
    for handle in handles {
        let _ = handle.join();
    }

    let total_elapsed = start.elapsed().as_micros();
    println!(
        "Concurrent parsing of {} formats: {}μs total",
        existing_files.len(),
        total_elapsed
    );

    // Should complete reasonably fast even with multiple formats
    assert!(
        total_elapsed < 100_000,
        "Concurrent parsing too slow: {}μs",
        total_elapsed
    );
}

#[test]
fn test_error_recovery_performance() {
    // Test that error handling doesn't significantly impact performance

    // Create corrupted JPEG
    let mut bad_jpeg = vec![0xFF, 0xD8]; // SOI
    bad_jpeg.extend_from_slice(&vec![0xFF; 1000]); // Invalid markers

    let mut cursor = std::io::Cursor::new(bad_jpeg);

    let (result, elapsed) =
        time_operation(|| exif_oxide::core::find_metadata_segment_from_reader(&mut cursor));

    match result {
        Ok(_) => println!("Error recovery succeeded"),
        Err(_) => println!("Error recovery returned error (acceptable)"),
    }

    println!("Error recovery time: {}μs", elapsed);
    assert!(elapsed < 1000, "Error handling too slow: {}μs", elapsed);
}
