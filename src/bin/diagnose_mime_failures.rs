use exif_oxide::file_detection::FileTypeDetector;
use std::path::Path;

fn main() {
    let test_files = vec![
        "third-party/exiftool/t/images/JXL2.jxl",
        "third-party/exiftool/t/images/JXL.jxl",
        "test-images/nikon/nikon_z8_73.NEF",
    ];

    for file_path in test_files {
        println!("\n=== Diagnosing: {} ===", file_path);
        
        // Read file bytes
        let path = Path::new(file_path);
        if let Ok(buffer) = std::fs::read(path) {
            // Show first 32 bytes
            print!("File bytes: ");
            for (i, &byte) in buffer.iter().take(32).enumerate() {
                if i > 0 && i % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
            println!();
            
            // Try detection
            let detector = FileTypeDetector::new();
            let mut file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open file: {}", e);
                    continue;
                }
            };
            
            match detector.detect_file_type(path, &mut file) {
                Ok(detected) => {
                    println!("Detected type: {:?}", detected.file_type);
                    println!("MIME type: {:?}", detected.mime_type);
                    
                    // Show extension resolution
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        println!("\nExtension '{}' resolves to: {:?}", ext, 
                            exif_oxide::generated::file_types::file_type_lookup::resolve_file_type(ext));
                    }
                }
                Err(e) => {
                    println!("Detection failed: {:?}", e);
                }
            }
        } else {
            println!("Failed to read file");
        }
    }
}