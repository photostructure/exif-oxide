use exif_oxide::tables::{lookup_canon_tag, CANON_TAGS};

fn main() {
    println!("Testing Canon tag generation...");

    // Check if Canon tags were generated
    println!("Total Canon tags: {}", CANON_TAGS.len());

    if CANON_TAGS.is_empty() {
        println!("ERROR: No Canon tags found!");
        return;
    }

    // Test some known Canon tags
    let test_tags = [
        (0x0001, "CanonCameraSettings"),
        (0x0006, "CanonImageType"),
        (0x0007, "CanonFirmwareVersion"),
        (0x0008, "FileNumber"),
        (0x0009, "OwnerName"),
    ];

    println!("\nTesting Canon tag lookup:");
    for (tag_id, expected_name) in test_tags {
        match lookup_canon_tag(tag_id) {
            Some(tag_info) => {
                println!(
                    "  0x{:04x} -> {} (expected: {})",
                    tag_id, tag_info.name, expected_name
                );
                if tag_info.name == expected_name {
                    println!("    ✓ MATCH");
                } else {
                    println!("    ✗ MISMATCH");
                }
            }
            None => {
                println!(
                    "  0x{:04x} -> NOT FOUND (expected: {})",
                    tag_id, expected_name
                );
            }
        }
    }

    // Show first 10 Canon tags
    println!("\nFirst 10 Canon tags:");
    for (i, (tag_id, tag_info)) in CANON_TAGS.iter().take(10).enumerate() {
        println!("  {}: 0x{:04x} -> {}", i + 1, tag_id, tag_info.name);
    }

    println!("\nCanon tag generation test complete!");
}
