// AUTO-GENERATED from ExifTool v12.65
// Source: lib/Image/ExifTool.pm (%magicNumber, %mimeType, %fileTypeLookup)
// Generated: 2025-06-23 by extract_magic_numbers
// DO NOT EDIT - Regenerate with `cargo run --bin extract_magic_numbers`

use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    // Image formats
    JPEG,
    PNG,
    TIFF,
    GIF,
    BMP,
    WEBP,
    HEIF,
    HEIC,
    AVIF,

    // Canon RAW formats
    CR2,
    CR3,
    CRW,

    // Nikon RAW formats
    NEF,
    NRW,

    // Sony RAW formats
    ARW,
    SR2,
    ARQ, // Sony Alpha RAW (Pixel Shift)
    SRF, // Sony RAW (DSLR)

    // Other manufacturer RAW formats
    RAF,     // Fujifilm
    ORF,     // Olympus
    PEF,     // Pentax
    RW2,     // Panasonic
    DNG,     // Adobe Digital Negative
    RAW,     // Panasonic RAW
    RWL,     // Leica RAW Light
    X3F,     // Sigma RAW
    ThreeFR, // Hasselblad 3FR
    FFF,     // Hasselblad FFF
    IIQ,     // Phase One RAW
    GPR,     // GoPro RAW
    ERF,     // Epson RAW
    DCR,     // Kodak DCR
    K25,     // Kodak K25
    KDC,     // Kodak KDC
    MEF,     // Mamiya RAW
    MRW,     // Minolta RAW
    SRW,     // Samsung RAW

    // Video formats
    MP4,
    MOV,
    AVI,
    CRM,       // Canon RAW Movie
    ThreeGPP,  // 3GPP (.3gp)
    ThreeGPP2, // 3GPP2 (.3g2)
    M4V,       // iTunes Video
    HEIFS,     // HEIF sequence (video)
    HEICS,     // HEIC sequence (video)

    // Other
    Unknown,
}

#[derive(Debug)]
pub struct MagicPattern {
    pub pattern: &'static [u8],
    pub regex: Option<&'static str>,
    pub offset: usize,
    pub weak: bool,
    pub test_len: usize,
}

lazy_static! {
    pub static ref MAGIC_NUMBERS: HashMap<FileType, Vec<MagicPattern>> = {
        let mut map = HashMap::new();

        // JPEG - perl: '\xff\xd8\xff'
        map.insert(FileType::JPEG, vec![MagicPattern {
            pattern: &[0xff, 0xd8, 0xff],
            regex: Some(r"\xff\xd8\xff"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // PNG - perl: '(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n'
        map.insert(FileType::PNG, vec![MagicPattern {
            pattern: &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a],
            regex: Some(r"(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // TIFF - perl: '(II\x2a\0|MM\0\x2a)'
        // Note: Many RAW formats use TIFF structure but need additional checks
        map.insert(FileType::TIFF, vec![
            MagicPattern {
                pattern: &[0x49, 0x49, 0x2a, 0x00], // Little endian
                regex: Some(r"II\x2a\0"),
                offset: 0,
                weak: false,
                test_len: 1024,
            },
            MagicPattern {
                pattern: &[0x4d, 0x4d, 0x00, 0x2a], // Big endian
                regex: Some(r"MM\0\x2a"),
                offset: 0,
                weak: false,
                test_len: 1024,
            }
        ]);

        // HEIF/HEIC - part of ftyp box detection
        map.insert(FileType::HEIF, vec![MagicPattern {
            pattern: &[0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63],
            regex: Some(r"....ftypheic"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // AVIF - QuickTime container with "avif" brand
        map.insert(FileType::AVIF, vec![MagicPattern {
            pattern: &[0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66], // "ftypavif"
            regex: Some(r"....ftypavif"),
            offset: 4,
            weak: false,
            test_len: 1024,
        }]);

        // CR3 - Canon RAW v3
        map.insert(FileType::CR3, vec![MagicPattern {
            pattern: &[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x63, 0x72, 0x78, 0x20],
            regex: Some(r"....ftypcrx "),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // CR2 - Canon RAW v2 (TIFF-based with "CR" at offset 8)
        // Detected via TIFF magic + special handling in detect_raw_variant

        // NEF/NRW - Nikon (TIFF-based)
        // Detected via TIFF magic + special handling in detect_raw_variant

        // ARW - Sony Alpha RAW (TIFF-based)
        // Detected via TIFF magic + special handling in detect_raw_variant

        // GIF - perl: 'GIF8[79]a'
        map.insert(FileType::GIF, vec![
            MagicPattern {
                pattern: &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61], // "GIF87a"
                regex: Some(r"GIF87a"),
                offset: 0,
                weak: false,
                test_len: 1024,
            },
            MagicPattern {
                pattern: &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61], // "GIF89a"
                regex: Some(r"GIF89a"),
                offset: 0,
                weak: false,
                test_len: 1024,
            }
        ]);

        // BMP - perl: 'BM'
        map.insert(FileType::BMP, vec![MagicPattern {
            pattern: &[0x42, 0x4d], // "BM"
            regex: Some(r"BM"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // WebP - RIFF container with "WEBP" at offset 8
        map.insert(FileType::WEBP, vec![MagicPattern {
            pattern: &[0x52, 0x49, 0x46, 0x46], // "RIFF" (WebP uses RIFF container)
            regex: Some(r"RIFF....WEBP"),
            offset: 0,
            weak: true, // Needs additional check for "WEBP" at offset 8
            test_len: 1024,
        }]);

        // RAF - Fujifilm RAW: 'FUJIFILM'
        map.insert(FileType::RAF, vec![MagicPattern {
            pattern: &[0x46, 0x55, 0x4a, 0x49, 0x46, 0x49, 0x4c, 0x4d], // "FUJIFILM"
            regex: Some(r"FUJIFILM"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // CRW - Canon RAW (older format): 'HEAP'
        map.insert(FileType::CRW, vec![MagicPattern {
            pattern: &[0x48, 0x45, 0x41, 0x50], // "HEAP"
            regex: Some(r"(II|MM).{4}HEAP(CCDR|JPGM)"),
            offset: 6, // HEAP appears at offset 6 after II/MM + 4 bytes
            weak: true, // Needs additional validation
            test_len: 1024,
        }]);

        // MP4/MOV - QuickTime/MP4 container
        map.insert(FileType::MP4, vec![MagicPattern {
            pattern: &[0x66, 0x74, 0x79, 0x70], // "ftyp" at offset 4
            regex: Some(r".{4}ftyp"),
            offset: 4,
            weak: false,
            test_len: 1024,
        }]);

        // MOV - QuickTime (similar pattern but different brands)
        map.insert(FileType::MOV, vec![
            MagicPattern {
                pattern: &[0x6d, 0x6f, 0x6f, 0x76], // "moov"
                regex: Some(r".{4}moov"),
                offset: 4,
                weak: false,
                test_len: 1024,
            },
            MagicPattern {
                pattern: &[0x6d, 0x64, 0x61, 0x74], // "mdat"
                regex: Some(r".{4}mdat"),
                offset: 4,
                weak: false,
                test_len: 1024,
            }
        ]);

        // AVI - RIFF container with "AVI " at offset 8
        map.insert(FileType::AVI, vec![MagicPattern {
            pattern: &[0x52, 0x49, 0x46, 0x46], // "RIFF"
            regex: Some(r"RIFF....AVI "),
            offset: 0,
            weak: true, // Needs additional check for "AVI " at offset 8
            test_len: 1024,
        }]);

        // RW2 - Panasonic RAW (TIFF-based with special header)
        map.insert(FileType::RW2, vec![MagicPattern {
            pattern: &[0x49, 0x49, 0x55, 0x00], // "IIU\0"
            regex: Some(r"IIU\0"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // DNG - Adobe Digital Negative (TIFF-based)
        // Detected via TIFF magic + IFD analysis

        // X3F - Sigma RAW: 'FOVb'
        map.insert(FileType::X3F, vec![MagicPattern {
            pattern: &[0x46, 0x4f, 0x56, 0x62], // "FOVb"
            regex: Some(r"FOVb"),
            offset: 0,
            weak: false,
            test_len: 1024,
        }]);

        // TIFF-based RAW formats (SR2, ORF, PEF, ARQ, SRF, etc.) are detected via
        // TIFF magic + manufacturer-specific handling in detect_raw_variant

        map
    };

    pub static ref MIME_TYPES: HashMap<FileType, &'static str> = {
        let mut map = HashMap::new();

        // Standard image formats
        map.insert(FileType::JPEG, "image/jpeg");
        map.insert(FileType::PNG, "image/png");
        map.insert(FileType::TIFF, "image/tiff");
        map.insert(FileType::GIF, "image/gif");
        map.insert(FileType::BMP, "image/bmp");
        map.insert(FileType::WEBP, "image/webp");
        map.insert(FileType::HEIF, "image/heif");
        map.insert(FileType::HEIC, "image/heic");
        map.insert(FileType::AVIF, "image/avif");

        // Canon RAW formats
        map.insert(FileType::CR2, "image/x-canon-cr2");
        map.insert(FileType::CR3, "image/x-canon-cr3");
        map.insert(FileType::CRW, "image/x-canon-crw");

        // Nikon RAW formats
        map.insert(FileType::NEF, "image/x-nikon-nef");
        map.insert(FileType::NRW, "image/x-nikon-nrw");

        // Sony RAW formats
        map.insert(FileType::ARW, "image/x-sony-arw");
        map.insert(FileType::SR2, "image/x-sony-sr2");
        map.insert(FileType::ARQ, "image/x-sony-arq");
        map.insert(FileType::SRF, "image/x-sony-srf");

        // Other manufacturer RAW formats
        map.insert(FileType::RAF, "image/x-fujifilm-raf");
        map.insert(FileType::ORF, "image/x-olympus-orf");
        map.insert(FileType::PEF, "image/x-pentax-pef");
        map.insert(FileType::RW2, "image/x-panasonic-rw2");
        map.insert(FileType::DNG, "image/x-adobe-dng");
        map.insert(FileType::RAW, "image/x-panasonic-raw");
        map.insert(FileType::RWL, "image/x-leica-rwl");
        map.insert(FileType::X3F, "image/x-sigma-x3f");
        map.insert(FileType::ThreeFR, "image/x-hasselblad-3fr");
        map.insert(FileType::FFF, "image/x-hasselblad-fff");
        map.insert(FileType::IIQ, "image/x-phaseone-iiq");
        map.insert(FileType::GPR, "image/x-gopro-gpr");
        map.insert(FileType::ERF, "image/x-epson-erf");
        map.insert(FileType::DCR, "image/x-kodak-dcr");
        map.insert(FileType::K25, "image/x-kodak-k25");
        map.insert(FileType::KDC, "image/x-kodak-kdc");
        map.insert(FileType::MEF, "image/x-mamiya-mef");
        map.insert(FileType::MRW, "image/x-minolta-mrw");
        map.insert(FileType::SRW, "image/x-samsung-srw");

        // Video formats
        map.insert(FileType::MP4, "video/mp4");
        map.insert(FileType::MOV, "video/quicktime");
        map.insert(FileType::AVI, "video/x-msvideo");
        map.insert(FileType::CRM, "video/x-canon-crm");
        map.insert(FileType::ThreeGPP, "video/3gpp");
        map.insert(FileType::ThreeGPP2, "video/3gpp2");
        map.insert(FileType::M4V, "video/x-m4v");
        map.insert(FileType::HEIFS, "image/heif-sequence");
        map.insert(FileType::HEICS, "image/heic-sequence");

        map
    };

    pub static ref EXTENSION_LOOKUP: HashMap<&'static str, (FileType, Option<&'static str>)> = {
        let mut map = HashMap::new();

        // Common image extensions
        map.insert("JPG", (FileType::JPEG, Some("JPEG image")));
        map.insert("JPEG", (FileType::JPEG, Some("JPEG image")));
        map.insert("JPE", (FileType::JPEG, Some("JPEG image")));
        map.insert("PNG", (FileType::PNG, Some("PNG image")));
        map.insert("TIF", (FileType::TIFF, Some("TIFF image")));
        map.insert("TIFF", (FileType::TIFF, Some("TIFF image")));
        map.insert("GIF", (FileType::GIF, Some("GIF image")));
        map.insert("BMP", (FileType::BMP, Some("Bitmap image")));
        map.insert("WEBP", (FileType::WEBP, Some("WebP image")));
        map.insert("HEIF", (FileType::HEIF, Some("HEIF image")));
        map.insert("HEIC", (FileType::HEIC, Some("HEIC image")));
        map.insert("AVIF", (FileType::AVIF, Some("AVIF image")));

        // Canon RAW extensions
        map.insert("CR2", (FileType::CR2, Some("Canon RAW 2")));
        map.insert("CR3", (FileType::CR3, Some("Canon RAW 3")));
        map.insert("CRW", (FileType::CRW, Some("Canon RAW (legacy)")));

        // Nikon RAW extensions
        map.insert("NEF", (FileType::NEF, Some("Nikon Electronic Format")));
        map.insert("NRW", (FileType::NRW, Some("Nikon RAW 2")));

        // Sony RAW extensions
        map.insert("ARW", (FileType::ARW, Some("Sony Alpha RAW")));
        map.insert("SR2", (FileType::SR2, Some("Sony RAW 2")));
        map.insert("ARQ", (FileType::ARQ, Some("Sony Alpha RAW (Pixel Shift)")));
        map.insert("SRF", (FileType::SRF, Some("Sony RAW (DSLR)")));

        // Other manufacturer RAW extensions
        map.insert("RAF", (FileType::RAF, Some("Fujifilm RAW")));
        map.insert("ORF", (FileType::ORF, Some("Olympus RAW")));
        map.insert("PEF", (FileType::PEF, Some("Pentax RAW")));
        map.insert("RW2", (FileType::RW2, Some("Panasonic RAW 2")));
        map.insert("DNG", (FileType::DNG, Some("Digital Negative")));
        map.insert("RAW", (FileType::RAW, Some("Panasonic RAW")));
        map.insert("RWL", (FileType::RWL, Some("Leica RAW Light")));
        map.insert("X3F", (FileType::X3F, Some("Sigma RAW")));
        map.insert("3FR", (FileType::ThreeFR, Some("Hasselblad RAW")));
        map.insert("FFF", (FileType::FFF, Some("Hasselblad FFF")));
        map.insert("IIQ", (FileType::IIQ, Some("Phase One RAW")));
        map.insert("GPR", (FileType::GPR, Some("GoPro RAW")));
        map.insert("ERF", (FileType::ERF, Some("Epson RAW")));
        map.insert("DCR", (FileType::DCR, Some("Kodak DCR")));
        map.insert("K25", (FileType::K25, Some("Kodak K25")));
        map.insert("KDC", (FileType::KDC, Some("Kodak KDC")));
        map.insert("MEF", (FileType::MEF, Some("Mamiya RAW")));
        map.insert("MRW", (FileType::MRW, Some("Minolta RAW")));
        map.insert("SRW", (FileType::SRW, Some("Samsung RAW")));

        // Video extensions
        map.insert("MP4", (FileType::MP4, Some("MPEG-4 video")));
        map.insert("M4V", (FileType::M4V, Some("iTunes video")));
        map.insert("M4A", (FileType::MP4, Some("MPEG-4 audio")));
        map.insert("MOV", (FileType::MOV, Some("QuickTime movie")));
        map.insert("QT", (FileType::MOV, Some("QuickTime movie")));
        map.insert("AVI", (FileType::AVI, Some("Audio Video Interleave")));
        map.insert("CRM", (FileType::CRM, Some("Canon RAW Movie")));
        map.insert("3GP", (FileType::ThreeGPP, Some("3GPP multimedia")));
        map.insert("3G2", (FileType::ThreeGPP2, Some("3GPP2 multimedia")));
        map.insert("HEICS", (FileType::HEICS, Some("HEIC sequence")));
        map.insert("HEIFS", (FileType::HEIFS, Some("HEIF sequence")));

        map
    };
}
