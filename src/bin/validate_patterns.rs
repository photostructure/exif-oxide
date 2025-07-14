use regex::bytes::Regex;

fn main() {
    // Test the exact PNG pattern from claude-regex.md
    let claude_pattern = r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n";

    println!("Testing pattern from claude-regex.md: {}", claude_pattern);

    match Regex::new(claude_pattern) {
        Ok(regex) => {
            println!("✓ Pattern compiles successfully!");

            // Test with real PNG data
            let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
            println!("\nTesting with PNG data: {:02x?}", png_data);
            println!("Pattern matches: {}", regex.is_match(&png_data));

            // Test each alternative
            let alt1_data = vec![0x89, 0x50]; // \x89P
            let alt2_data = vec![0x8a, 0x4d]; // \x8aM
            let alt3_data = vec![0x8b, 0x4a]; // \x8bJ

            println!("\nTesting alternatives:");
            println!(
                "\\x89P matches first 2 bytes: {}",
                regex.is_match(&alt1_data)
            );
            println!(
                "\\x8aM matches first 2 bytes: {}",
                regex.is_match(&alt2_data)
            );
            println!(
                "\\x8bJ matches first 2 bytes: {}",
                regex.is_match(&alt3_data)
            );

            // Test without anchor
            let no_anchor_pattern = r"(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n";
            let no_anchor_regex = Regex::new(no_anchor_pattern).unwrap();
            println!("\nWithout ^ anchor:");
            println!("Pattern matches: {}", no_anchor_regex.is_match(&png_data));
        }
        Err(e) => {
            println!("✗ Pattern FAILED to compile: {}", e);
        }
    }

    // Also test a simpler pattern
    println!("\n--- Testing simpler patterns ---");

    let simple_pattern = r"^\x89PNG";
    match Regex::new(simple_pattern) {
        Ok(regex) => {
            println!("Simple pattern {} compiles!", simple_pattern);
            let data = vec![0x89, 0x50, 0x4e, 0x47]; // \x89PNG
            println!("Matches {:02x?}: {}", data, regex.is_match(&data));
        }
        Err(e) => {
            println!("Simple pattern FAILED: {}", e);
        }
    }
}
