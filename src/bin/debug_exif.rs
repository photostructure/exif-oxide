//! Debug tool for EXIF parsing

use exif_oxide::core::ifd::TiffHeader;
use exif_oxide::core::jpeg::find_exif_segment;
use std::fs::File;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: debug_exif <jpeg-file>");
    let mut file = File::open(&path).expect("Failed to open file");

    let segment = find_exif_segment(&mut file)
        .expect("Failed to read JPEG")
        .expect("No EXIF data found");

    println!("EXIF segment found:");
    println!("  Offset: {}", segment.offset);
    println!("  Size: {} bytes", segment.data.len());

    // Parse TIFF header
    let header = TiffHeader::parse(&segment.data).expect("Failed to parse TIFF header");
    println!("\nTIFF Header:");
    println!("  Byte order: {:?}", header.byte_order);
    println!("  IFD0 offset: {}", header.ifd0_offset);

    // Parse IFD manually for debugging
    let ifd_offset = header.ifd0_offset as usize;
    if ifd_offset + 2 <= segment.data.len() {
        let entry_count = header
            .byte_order
            .read_u16(&segment.data[ifd_offset..ifd_offset + 2]);
        println!("\nIFD0 entries: {}", entry_count);

        let mut pos = ifd_offset + 2;
        for i in 0..entry_count.min(10) {
            if pos + 12 > segment.data.len() {
                break;
            }

            let entry_data = &segment.data[pos..pos + 12];
            let tag = header.byte_order.read_u16(&entry_data[0..2]);
            let format = header.byte_order.read_u16(&entry_data[2..4]);
            let count = header.byte_order.read_u32(&entry_data[4..8]);

            println!("\nEntry {}:", i);
            println!("  Tag: 0x{:04X}", tag);
            println!("  Format: {}", format);
            println!("  Count: {}", count);

            if tag == 0x010F || tag == 0x0110 {
                // Try to read string value
                if format == 2 && count > 0 {
                    let value_size = count as usize;
                    if value_size <= 4 {
                        // Value fits inline
                        let s = std::str::from_utf8(&entry_data[8..12]).unwrap_or("<invalid utf8>");
                        println!("  Value (inline): {:?}", s);
                    } else {
                        // Value at offset
                        let offset = header.byte_order.read_u32(&entry_data[8..12]) as usize;
                        if offset + value_size <= segment.data.len() {
                            let value_data = &segment.data[offset..offset + value_size];
                            let s = std::str::from_utf8(value_data).unwrap_or("<invalid utf8>");
                            println!("  Value (at offset {}): {:?}", offset, s);
                        }
                    }
                }
            }

            pos += 12;
        }

        // Check for next IFD offset (IFD1)
        let next_ifd_pos = pos;
        if next_ifd_pos + 4 <= segment.data.len() {
            let next_ifd_offset = header
                .byte_order
                .read_u32(&segment.data[next_ifd_pos..next_ifd_pos + 4]);
            println!("\nNext IFD offset: {}", next_ifd_offset);

            if next_ifd_offset > 0 && (next_ifd_offset as usize) < segment.data.len() {
                println!("\nIFD1 (Thumbnail):");
                let ifd1_offset = next_ifd_offset as usize;
                let entry_count = header
                    .byte_order
                    .read_u16(&segment.data[ifd1_offset..ifd1_offset + 2]);
                println!("  Entries: {}", entry_count);

                let mut pos = ifd1_offset + 2;
                for i in 0..entry_count.min(5) {
                    if pos + 12 > segment.data.len() {
                        break;
                    }

                    let entry_data = &segment.data[pos..pos + 12];
                    let tag = header.byte_order.read_u16(&entry_data[0..2]);
                    println!("  Entry {}: Tag 0x{:04X}", i, tag);
                    pos += 12;
                }
            }
        }
    }
}
