#[cfg(test)]
mod tests {
    use regex::bytes::Regex;

    #[test]
    fn test_png_pattern() {
        let buffer = [137u8, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];

        println!("Buffer: {:?}", &buffer[..8]);

        // Try different ways to represent byte 137 (\x89)
        let patterns = [
            r"^\x89PNG",                         // Direct hex escape
            r"^[\x89]PNG",                       // Character class
            "^\\x89PNG",                         // Double escaped
            r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n", // Original ExifTool pattern
        ];

        for (i, pattern) in patterns.iter().enumerate() {
            println!("Pattern {i}: {pattern}");
            match Regex::new(pattern) {
                Ok(re) => {
                    let matches = re.is_match(&buffer);
                    println!("  Match result: {matches}");
                }
                Err(e) => {
                    println!("  Regex error: {e}");
                }
            }
        }

        // Try matching directly against byte literals
        println!(
            "Direct byte comparison: buffer[0] = {}, target = 137",
            buffer[0]
        );
        println!(
            "Direct match first 4 bytes: {:?} vs PNG header",
            &buffer[..4]
        );
    }
}
