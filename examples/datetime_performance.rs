//! Performance measurement for datetime intelligence
//!
//! This example measures the actual performance overhead of datetime intelligence
//! to validate we're meeting the <5ms target.

use exif_oxide::{extract_datetime_intelligence, read_basic_exif};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DateTime Intelligence Performance Test");
    println!("=====================================");

    let test_files = [
        "exiftool/t/images/Canon.jpg",
        "exiftool/t/images/Nikon.jpg",
        "exiftool/t/images/Sony.jpg",
        "exiftool/t/images/ExifTool.jpg",
    ];

    for test_file in &test_files {
        if Path::new(test_file).exists() {
            println!("\nTesting with: {}", test_file);

            // Warm up
            let _ = extract_datetime_intelligence(test_file)?;

            // Measure datetime intelligence only
            let mut total_dt_time = std::time::Duration::ZERO;
            let iterations = 100;

            for _ in 0..iterations {
                let start = Instant::now();
                let _result = extract_datetime_intelligence(test_file)?;
                total_dt_time += start.elapsed();
            }

            let avg_dt_time = total_dt_time / iterations;
            println!(
                "  Datetime intelligence: {:?} (avg over {} runs)",
                avg_dt_time, iterations
            );

            // Measure BasicExif with datetime (full API)
            let mut total_basic_time = std::time::Duration::ZERO;

            for _ in 0..iterations {
                let start = Instant::now();
                let _result = read_basic_exif(test_file)?;
                total_basic_time += start.elapsed();
            }

            let avg_basic_time = total_basic_time / iterations;
            println!(
                "  BasicExif with datetime: {:?} (avg over {} runs)",
                avg_basic_time, iterations
            );

            // Performance analysis
            println!("  Performance analysis:");
            if avg_dt_time.as_millis() < 5 {
                println!(
                    "    ✅ DateTime intelligence: {}ms (target: <5ms)",
                    avg_dt_time.as_millis()
                );
            } else {
                println!(
                    "    ❌ DateTime intelligence: {}ms (exceeds 5ms target)",
                    avg_dt_time.as_millis()
                );
            }

            if avg_basic_time.as_millis() < 10 {
                println!(
                    "    ✅ Full BasicExif: {}ms (reasonable)",
                    avg_basic_time.as_millis()
                );
            } else {
                println!(
                    "    ⚠️  Full BasicExif: {}ms (could be optimized)",
                    avg_basic_time.as_millis()
                );
            }
        } else {
            println!("Skipping {} (file not found)", test_file);
        }
    }

    // Test GPS timezone lookup performance specifically
    println!("\n\nGPS Timezone Performance");
    println!("========================");

    use exif_oxide::datetime::gps_timezone::GpsTimezoneInference;

    let gps_coords = [
        (40.7128, -74.0060, "New York"),
        (51.5074, -0.1278, "London"),
        (35.6762, 139.6503, "Tokyo"),
        (-33.8688, 151.2093, "Sydney"),
    ];

    for (lat, lng, name) in &gps_coords {
        // Warm up
        let _ = GpsTimezoneInference::infer_timezone(*lat, *lng);

        let mut total_time = std::time::Duration::ZERO;
        let iterations = 1000;

        for _ in 0..iterations {
            let start = Instant::now();
            let _result = GpsTimezoneInference::infer_timezone(*lat, *lng);
            total_time += start.elapsed();
        }

        let avg_time = total_time / iterations;
        println!(
            "  {} GPS lookup: {:?} (avg over {} runs)",
            name, avg_time, iterations
        );

        if avg_time.as_micros() < 5000 {
            // 5ms = 5000 microseconds
            println!("    ✅ Performance: {}μs (excellent)", avg_time.as_micros());
        } else {
            println!(
                "    ⚠️  Performance: {}μs (could be optimized)",
                avg_time.as_micros()
            );
        }
    }

    Ok(())
}
