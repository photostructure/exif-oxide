use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::jpeg;
use exif_oxide::maker;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Canon maker note parsing with Canon1DmkIII.jpg...");

    let image_path = "exiftool/t/images/Canon1DmkIII.jpg";
    let mut file = File::open(image_path)?;

    // Extract EXIF segment
    let exif_segment = jpeg::find_exif_segment(&mut file)?.ok_or("No EXIF data found")?;

    println!("Found EXIF segment: {} bytes", exif_segment.data.len());

    // Parse IFD to get main EXIF data including Make and MakerNotes
    let ifd = IfdParser::parse(exif_segment.data)?;
    let entries = ifd.entries();

    // Check Make tag
    let make = match entries.get(&0x10f) {
        Some(exif_oxide::core::ExifValue::Ascii(s)) => s,
        _ => {
            println!("No Make tag found");
            return Ok(());
        }
    };

    println!("Camera Make: {}", make);

    // Check if MakerNotes tag exists
    let maker_notes_tag = 0x927c;
    match entries.get(&maker_notes_tag) {
        Some(exif_oxide::core::ExifValue::Undefined(data)) => {
            println!("Found MakerNotes: {} bytes", data.len());

            // Test our Canon parser directly
            let manufacturer = maker::Manufacturer::from_make(make);
            println!("Detected manufacturer: {:?}", manufacturer);

            if manufacturer == maker::Manufacturer::Canon {
                println!("Testing Canon maker note parser...");

                match maker::parse_maker_notes(data, make, exif_oxide::core::Endian::Little, 0) {
                    Ok(canon_entries) => {
                        println!("Successfully parsed Canon maker notes!");
                        println!("Found {} Canon-specific tags:", canon_entries.len());

                        for (tag_id, value) in canon_entries.iter().take(10) {
                            println!("  0x{:04x}: {:?}", tag_id, value);
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse Canon maker notes: {}", e);
                    }
                }
            }
        }
        Some(other) => {
            println!("MakerNotes found but not as raw bytes: {:?}", other);
        }
        None => {
            println!("No MakerNotes tag found");

            // Check if maker note data was parsed and stored with prefix
            let mut maker_note_count = 0;
            for (tag_id, value) in entries {
                if *tag_id >= 0x8000 {
                    maker_note_count += 1;
                    if maker_note_count <= 5 {
                        println!("  Maker note 0x{:04x}: {:?}", tag_id, value);
                    }
                }
            }

            if maker_note_count > 0 {
                println!(
                    "Found {} maker note entries with 0x8000+ prefix",
                    maker_note_count
                );
            }
        }
    }

    // Show all EXIF tags for debugging (first 20)
    println!("\nAll EXIF tags (first 20):");
    for (tag_id, value) in entries.iter().take(20) {
        println!("  0x{:04x}: {:?}", tag_id, value);
    }

    // Look specifically for maker notes tag
    println!("\nLooking for MakerNotes tag (0x927c)...");
    if let Some(value) = entries.get(&0x927c) {
        println!("Found MakerNotes: {:?}", value);
    } else {
        println!("MakerNotes tag (0x927c) not found in IFD0");
    }

    // Check if there's an ExifIFD pointer (tag 0x8769)
    if let Some(value) = entries.get(&0x8769) {
        println!("Found ExifIFD pointer: {:?}", value);
        println!("Maker notes might be in ExifIFD sub-directory");
    }

    // Show ALL tags to find maker notes
    println!("\nAll {} EXIF tags:", entries.len());
    for (tag_id, value) in entries.iter() {
        println!(
            "  0x{:04x}: {}",
            tag_id,
            match value {
                exif_oxide::core::ExifValue::Ascii(s) => format!("Ascii(\"{}\")", s),
                exif_oxide::core::ExifValue::U16(v) => format!("U16({})", v),
                exif_oxide::core::ExifValue::U32(v) => format!("U32({})", v),
                exif_oxide::core::ExifValue::Rational(n, d) => format!("Rational({}/{})", n, d),
                exif_oxide::core::ExifValue::Undefined(data) =>
                    format!("Undefined({} bytes)", data.len()),
                _ => format!("{:?}", value),
            }
        );
    }

    Ok(())
}
