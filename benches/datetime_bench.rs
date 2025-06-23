//! Benchmark for datetime intelligence performance
//!
//! These benchmarks validate that datetime intelligence adds <5ms overhead
//! and identify performance bottlenecks for optimization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use exif_oxide::{extract_datetime_intelligence, read_basic_exif};
use std::path::Path;

fn bench_datetime_intelligence_canon(c: &mut Criterion) {
    if Path::new("exiftool/t/images/Canon.jpg").exists() {
        c.bench_function("datetime_intelligence_canon", |b| {
            b.iter(|| black_box(extract_datetime_intelligence("exiftool/t/images/Canon.jpg")))
        });
    }
}

fn bench_datetime_intelligence_nikon(c: &mut Criterion) {
    if Path::new("exiftool/t/images/Nikon.jpg").exists() {
        c.bench_function("datetime_intelligence_nikon", |b| {
            b.iter(|| black_box(extract_datetime_intelligence("exiftool/t/images/Nikon.jpg")))
        });
    }
}

fn bench_datetime_intelligence_sony(c: &mut Criterion) {
    if Path::new("exiftool/t/images/Sony.jpg").exists() {
        c.bench_function("datetime_intelligence_sony", |b| {
            b.iter(|| black_box(extract_datetime_intelligence("exiftool/t/images/Sony.jpg")))
        });
    }
}

fn bench_basic_exif_with_datetime(c: &mut Criterion) {
    if Path::new("exiftool/t/images/Canon.jpg").exists() {
        c.bench_function("basic_exif_with_datetime", |b| {
            b.iter(|| black_box(read_basic_exif("exiftool/t/images/Canon.jpg")))
        });
    }
}

fn bench_basic_exif_overhead_comparison(c: &mut Criterion) {
    if Path::new("exiftool/t/images/Canon.jpg").exists() {
        // Benchmark the core EXIF parsing (without datetime intelligence)
        c.bench_function("basic_exif_core_only", |b| {
            b.iter(|| {
                // Simulate basic EXIF reading without datetime intelligence
                use exif_oxide::core::{ifd, jpeg};
                use std::fs::File;

                let mut file = File::open("exiftool/t/images/Canon.jpg").unwrap();
                let exif_segment = jpeg::find_exif_segment(&mut file).unwrap().unwrap();
                let ifd = ifd::IfdParser::parse(exif_segment.data).unwrap();

                black_box((
                    ifd.get_string(0x10F).unwrap(), // Make
                    ifd.get_string(0x110).unwrap(), // Model
                    ifd.get_u16(0x112).unwrap(),    // Orientation
                ))
            })
        });

        // Benchmark with datetime intelligence for overhead comparison
        c.bench_function("basic_exif_with_datetime_full", |b| {
            b.iter(|| black_box(read_basic_exif("exiftool/t/images/Canon.jpg")))
        });
    }
}

fn bench_gps_timezone_lookup(c: &mut Criterion) {
    use exif_oxide::datetime::gps_timezone::GpsTimezoneInference;

    c.bench_function("gps_timezone_lookup", |b| {
        b.iter(|| {
            black_box(GpsTimezoneInference::infer_timezone(
                black_box(40.7128), // New York
                black_box(-74.0060),
            ))
        })
    });

    c.bench_function("gps_timezone_offset", |b| {
        b.iter(|| {
            black_box(GpsTimezoneInference::get_timezone_offset(
                black_box(40.7128),
                black_box(-74.0060),
                black_box(chrono::Utc::now()),
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_datetime_intelligence_canon,
    bench_datetime_intelligence_nikon,
    bench_datetime_intelligence_sony,
    bench_basic_exif_with_datetime,
    bench_basic_exif_overhead_comparison,
    bench_gps_timezone_lookup
);

criterion_main!(benches);
