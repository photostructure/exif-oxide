//! Benchmarks for multi-format parsing performance
//!
//! Tests parsing speed across different file formats to ensure
//! multi-format support doesn't regress JPEG performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use exif_oxide::core::find_metadata_segment;
use std::path::Path;

/// Benchmark parsing different format files
fn bench_format_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_parsing");

    // Define test files for each format
    let test_files = vec![
        ("JPEG", "exiftool/t/images/Canon.jpg"),
        ("TIFF", "exiftool/t/images/ExifTool.tif"),
        ("PNG", "exiftool/t/images/PNG.png"),
        ("HEIF", "exiftool/t/images/HEIC.heic"),
        ("CR2", "exiftool/t/images/CanonRaw.cr2"),
        ("NEF", "exiftool/t/images/Nikon.nef"),
        ("WebP", "exiftool/t/images/WebP.webp"),
        ("MP4", "exiftool/t/images/MP4.mp4"),
    ];

    for (format_name, file_path) in test_files {
        if Path::new(file_path).exists() {
            group.bench_with_input(
                BenchmarkId::new("parse", format_name),
                &file_path,
                |b, path| {
                    b.iter(|| {
                        let _ = black_box(find_metadata_segment(path));
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark JPEG parsing specifically to ensure no regression
fn bench_jpeg_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("jpeg_parsing");

    // Test different JPEG sizes
    let jpeg_files = vec![
        ("small", "exiftool/t/images/ExifTool.jpg"),
        ("medium", "exiftool/t/images/Canon.jpg"),
        ("large", "exiftool/t/images/Nikon.jpg"),
    ];

    for (size, file_path) in jpeg_files {
        if Path::new(file_path).exists() {
            group.bench_with_input(BenchmarkId::new("jpeg", size), &file_path, |b, path| {
                b.iter(|| {
                    let _ = black_box(find_metadata_segment(path));
                });
            });
        }
    }

    group.finish();
}

/// Benchmark memory efficiency for large files
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10); // Fewer samples for memory benchmarks

    // Test with large RAW files if available
    let large_files = vec![
        ("CR2", "exiftool/t/images/CanonRaw.cr2"),
        ("NEF", "exiftool/t/images/Nikon.nef"),
        ("TIFF", "test-images/large.tif"),
    ];

    for (format, file_path) in large_files {
        if Path::new(file_path).exists() {
            group.bench_with_input(
                BenchmarkId::new("large_file", format),
                &file_path,
                |b, path| {
                    b.iter(|| {
                        let _ = black_box(find_metadata_segment(path));
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark format detection overhead
fn bench_format_detection(c: &mut Criterion) {
    use exif_oxide::detection::detect_file_type;
    use std::fs::File;
    use std::io::Read;

    let mut group = c.benchmark_group("format_detection");

    let test_files = vec![
        ("JPEG", "exiftool/t/images/Canon.jpg"),
        ("PNG", "exiftool/t/images/PNG.png"),
        ("WebP", "exiftool/t/images/WebP.webp"),
        ("MP4", "exiftool/t/images/MP4.mp4"),
    ];

    for (format, file_path) in test_files {
        if Path::new(file_path).exists() {
            // Pre-read first 1KB for detection
            let mut buffer = vec![0u8; 1024];
            if let Ok(mut file) = File::open(file_path) {
                let _ = file.read(&mut buffer);

                group.bench_with_input(BenchmarkId::new("detect", format), &buffer, |b, buf| {
                    b.iter(|| {
                        let _ = black_box(detect_file_type(buf));
                    });
                });
            }
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_parsing,
    bench_jpeg_parsing,
    bench_memory_usage,
    bench_format_detection
);
criterion_main!(benches);
