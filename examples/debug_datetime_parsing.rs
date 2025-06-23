use chrono::NaiveDateTime;

fn main() {
    let test_string = "Thu Mar 15 14:30:00 2024";
    let format = "%a %b %d %H:%M:%S %Y";

    println!("Testing datetime parsing:");
    println!("  Input: {}", test_string);
    println!("  Format: {}", format);

    match NaiveDateTime::parse_from_str(test_string, format) {
        Ok(dt) => {
            println!("  Success: {}", dt);
        }
        Err(e) => {
            println!("  Failed: {}", e);

            // Try alternative formats
            let alt_formats = [
                "%a %b %e %H:%M:%S %Y", // %e allows single-digit day with space padding
                "%a %b %d %H:%M:%S %Y", // Original
                "%a %b%d %H:%M:%S %Y",  // No space before day
            ];

            for (i, alt_format) in alt_formats.iter().enumerate() {
                match NaiveDateTime::parse_from_str(test_string, alt_format) {
                    Ok(dt) => {
                        println!(
                            "  Alternative {}: Success with '{}': {}",
                            i + 1,
                            alt_format,
                            dt
                        );
                        break;
                    }
                    Err(e) => {
                        println!(
                            "  Alternative {}: Failed with '{}': {}",
                            i + 1,
                            alt_format,
                            e
                        );
                    }
                }
            }
        }
    }

    // Test the first format too
    let test_string2 = "Mar 15 2024 14:30:00";
    let format2 = "%b %d %Y %H:%M:%S";

    println!("\nTesting second format:");
    println!("  Input: {}", test_string2);
    println!("  Format: {}", format2);

    match NaiveDateTime::parse_from_str(test_string2, format2) {
        Ok(dt) => {
            println!("  Success: {}", dt);
        }
        Err(e) => {
            println!("  Failed: {}", e);
        }
    }
}
