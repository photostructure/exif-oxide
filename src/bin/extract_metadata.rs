use exif_oxide::core::ifd::IfdParser;
use exif_oxide::core::jpeg;
use exif_oxide::detection::{detect_file_type, FileType};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image_file> [-j|--json]", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let json_output = args.len() > 2 && (args[2] == "-j" || args[2] == "--json");

    // Read file for detection
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Detect file type
    let file_info = detect_file_type(&buffer)?;

    if json_output {
        output_json(file_path, &file_info)?;
    } else {
        output_human(file_path, &file_info)?;
    }

    // If it's a JPEG, try to extract some EXIF data
    if file_info.file_type == FileType::JPEG {
        extract_jpeg_metadata(file_path, json_output)?;
    }

    Ok(())
}

fn output_json(
    file_path: &str,
    file_info: &exif_oxide::detection::FileInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = vec![json!({
        "SourceFile": file_path,
        "File:MIMEType": file_info.file_mime_type(),
        "FileType": format!("{:?}", file_info.file_type),
    })];

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_human(
    file_path: &str,
    file_info: &exif_oxide::detection::FileInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("File: {}", file_path);
    println!("File Type: {:?}", file_info.file_type);
    println!("File:MIMEType: {}", file_info.file_mime_type());
    Ok(())
}

fn extract_jpeg_metadata(
    file_path: &str,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;

    if let Ok(Some(exif_segment)) = jpeg::find_exif_segment(&mut file) {
        if let Ok(ifd) = IfdParser::parse(exif_segment.data) {
            if json_output {
                // Add basic EXIF data to JSON output
                let mut exif_data = HashMap::new();

                if let Ok(Some(make)) = ifd.get_string(0x10F) {
                    exif_data.insert("EXIF:Make".to_string(), make);
                }
                if let Ok(Some(model)) = ifd.get_string(0x110) {
                    exif_data.insert("EXIF:Model".to_string(), model);
                }

                if !exif_data.is_empty() {
                    println!("\nEXIF Data:");
                    for (key, value) in &exif_data {
                        println!("  {}: {}", key, value);
                    }
                }
            } else {
                // Human-readable output
                println!("\nEXIF Data Found:");
                if let Ok(Some(make)) = ifd.get_string(0x10F) {
                    println!("  Make: {}", make);
                }
                if let Ok(Some(model)) = ifd.get_string(0x110) {
                    println!("  Model: {}", model);
                }
            }
        }
    }

    Ok(())
}
