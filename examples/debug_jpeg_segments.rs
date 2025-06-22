//! Debug tool to show all JPEG segments

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <jpeg-file>", args[0]);
        return;
    }

    let mut file = File::open(&args[1]).expect("Failed to open file");

    // Check SOI
    let mut marker = [0u8; 2];
    file.read_exact(&mut marker).expect("Failed to read SOI");
    if marker != [0xFF, 0xD8] {
        eprintln!("Not a JPEG file");
        return;
    }

    println!("JPEG segments found:");
    println!("Offset  Marker  Length  Description");
    println!("------  ------  ------  -----------");

    loop {
        let offset = file.seek(SeekFrom::Current(0)).unwrap();

        // Read marker
        let mut marker_bytes = [0u8; 2];
        if file.read_exact(&mut marker_bytes).is_err() {
            break;
        }

        if marker_bytes[0] != 0xFF {
            eprintln!("Invalid marker at offset {:#x}", offset);
            break;
        }

        let marker = marker_bytes[1];

        match marker {
            0xD8 => println!("{:#06x}  FFD8    -       Start of Image", offset),
            0xD9 => {
                println!("{:#06x}  FFD9    -       End of Image", offset);
                break;
            }
            0xDA => {
                println!("{:#06x}  FFDA    -       Start of Scan", offset);
                break;
            }
            0xD0..=0xD7 | 0x01 => {
                println!(
                    "{:#06x}  FF{:02X}    -       Restart marker",
                    offset, marker
                );
            }
            _ => {
                // Read segment length
                let mut len_bytes = [0u8; 2];
                file.read_exact(&mut len_bytes)
                    .expect("Failed to read length");
                let segment_len = u16::from_be_bytes(len_bytes) as usize;

                // Identify segment type
                let desc = match marker {
                    0xE0 => "APP0 (JFIF)",
                    0xE1 => {
                        // Check if EXIF or XMP
                        let mut sig = [0u8; 30];
                        let read_len = file.read(&mut sig).unwrap();
                        file.seek(SeekFrom::Current(-(read_len as i64))).unwrap();

                        if sig.starts_with(b"Exif\0\0") {
                            "APP1 (EXIF)"
                        } else if sig.starts_with(b"http://ns.adobe.com/xap/1.0/\0") {
                            "APP1 (XMP)"
                        } else if sig.starts_with(b"http://ns.adobe.com/xmp/extension/\0") {
                            "APP1 (Extended XMP)"
                        } else {
                            "APP1 (Unknown)"
                        }
                    }
                    0xE2 => "APP2",
                    0xE3 => "APP3",
                    0xE4 => "APP4",
                    0xE5 => "APP5",
                    0xE6 => "APP6",
                    0xE7 => "APP7",
                    0xE8 => "APP8",
                    0xE9 => "APP9",
                    0xEA => "APP10",
                    0xEB => "APP11",
                    0xEC => "APP12",
                    0xED => "APP13",
                    0xEE => "APP14",
                    0xEF => "APP15",
                    0xDB => "DQT (Quantization Table)",
                    0xC0 => "SOF0 (Baseline)",
                    0xC2 => "SOF2 (Progressive)",
                    0xC4 => "DHT (Huffman Table)",
                    0xDD => "DRI (Restart Interval)",
                    0xFE => "COM (Comment)",
                    _ => "Unknown",
                };

                println!(
                    "{:#06x}  FF{:02X}    {:#06x}  {}",
                    offset, marker, segment_len, desc
                );

                // Skip segment data (minus 2 bytes for length we already read)
                file.seek(SeekFrom::Current((segment_len - 2) as i64))
                    .unwrap();
            }
        }
    }
}
