use exif_oxide::core::find_all_metadata_segments;
use exif_oxide::extract_gpmf_metadata;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_path = "test-images/gopro/GoPro.jpg";
    
    println!("Testing GPMF extraction for: {}", image_path);
    
    // Test 1: Check if metadata segments are found
    let metadata_collection = find_all_metadata_segments(image_path)?;
    println!("EXIF segment: {:?}", metadata_collection.exif.is_some());
    println!("GPMF segments found: {}", metadata_collection.gpmf.len());
    
    for (i, gpmf_segment) in metadata_collection.gpmf.iter().enumerate() {
        println!("GPMF segment {}: {} bytes at offset {}", 
                 i, gpmf_segment.data.len(), gpmf_segment.offset);
        
        // Show first 16 bytes to debug format
        let preview = if gpmf_segment.data.len() >= 16 {
            &gpmf_segment.data[0..16]
        } else {
            &gpmf_segment.data
        };
        println!("  First {} bytes: {:?}", preview.len(), preview);
        
        // Try to parse this segment directly
        let gpmf_parser = exif_oxide::gpmf::GpmfParser::new();
        match gpmf_parser.parse(&gpmf_segment.data) {
            Ok(gpmf_data) => {
                println!("  Parsed {} GPMF tags", gpmf_data.len());
                for (tag, value) in gpmf_data.iter().take(5) {
                    println!("    {} -> {:?}", tag, value);
                }
            }
            Err(e) => {
                println!("  GPMF parsing error: {}", e);
            }
        }
    }
    
    // Test 2: Try the high-level extraction function
    println!("\nTesting high-level GPMF extraction:");
    match extract_gpmf_metadata(image_path) {
        Ok(gpmf_data) => {
            println!("Successfully extracted {} GPMF tags", gpmf_data.len());
            for (tag, value) in gpmf_data.iter().take(10) {
                println!("  {} -> {:?}", tag, value);
            }
        }
        Err(e) => {
            println!("GPMF extraction error: {}", e);
        }
    }
    
    Ok(())
}