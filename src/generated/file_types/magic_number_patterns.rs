//! Magic number regex patterns generated from ExifTool's magicNumber hash
//!
//! Generated at: Fri Jul 11 03:11:17 2025 GMT
//! Total patterns: 110
//! Source: ExifTool.pm %magicNumber hash

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Magic number regex patterns for file type detection
/// These patterns are validated to be compatible with the Rust regex crate
static MAGIC_NUMBER_PATTERNS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("AA", ".{4}\\x57\\x90\\x75\\x36");
    map.insert("AAC", "\\xff[\\xf0\\xf1]");
    map.insert("AIFF", "(FORM....AIF[FC]|AT&TFORM)");
    map.insert("ALIAS", "book\x00\x00\x00\x00mark\x00\x00\x00\x00");
    map.insert("APE", "(MAC |APETAGEX|ID3)");
    map.insert("ASF", "\\x30\\x26\\xb2\\x75\\x8e\\x66\\xcf\\x11\\xa6\\xd9\\x00\\xaa\\x00\\x62\\xce\\x6c");
    map.insert("AVC", "\\+A\\+V\\+C\\+");
    map.insert("BMP", "BM");
    map.insert("BPG", "BPG\\xfb");
    map.insert("BTF", "(II\\x2b\\x00|MM\\x00\\x2b)");
    map.insert("BZ2", "BZh[1-9]\\x31\\x41\\x59\\x26\\x53\\x59");
    map.insert("CHM", "ITSF.{20}\\x10\\xfd\\x01\\x7c\\xaa\\x7b\\xd0\\x11\\x9e\\x0c\\x00\\xa0\\xc9\\x22\\xe6\\xec");
    map.insert("CRW", "(II|MM).{4}HEAP(CCDR|JPGM)");
    map.insert("CZI", "ZISRAWFILE\\x00{6}");
    map.insert("DCX", "\\xb1\\x68\\xde\\x3a");
    map.insert("DEX", "dex\n035\x00");
    map.insert("DICOM", "(.{128}DICM|\\x00[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]|[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]\\x00)");
    map.insert("DOCX", "PK\\x03\\x04");
    map.insert("DPX", "(SDPX|XPDS)");
    map.insert("DR4", "IIII[\\x04|\\x05]\\x00\\x04\\x00");
    map.insert("DSS", "(\\x02dss|\\x03ds2)");
    map.insert("DV", "\\x1f\\x07\\x00[\\x3f\\xbf]");
    map.insert("DWF", "\\(DWF V\\d");
    map.insert("DWG", "AC10\\d{2}\\x00");
    map.insert("DXF", "\\s*0\\s+\\x00?\\s*SECTION\\s+2\\s+HEADER");
    map.insert("EPS", "(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)");
    map.insert("EXE", "(MZ|\\xca\\xfe\\xba\\xbe|\\xfe\\xed\\xfa[\\xce\\xcf]|[\\xce\\xcf]\\xfa\\xed\\xfe|Joy!peff|\\x7fELF|#!\\s*/\\S*bin/|!<arch>\\x0a)");
    map.insert("EXIF", "(II\\x2a\\x00|MM\\x00\\x2a)");
    map.insert("EXR", "\\x76\\x2f\\x31\\x01");
    map.insert("EXV", "\\xff\\x01Exiv2");
    map.insert("FITS", "SIMPLE  = {20}T");
    map.insert("FLAC", "(fLaC|ID3)");
    map.insert("FLIF", "FLIF[0-\\x6f][0-2]");
    map.insert("FLIR", "[AF]FF\\x00");
    map.insert("FLV", "FLV\\x01");
    map.insert("FPF", "FPF Public Image Format\\x00");
    map.insert("FPX", "\\xd0\\xcf\\x11\\xe0\\xa1\\xb1\\x1a\\xe1");
    map.insert("Font", "((\\x00\\x01\\x00\\x00|OTTO|true|typ1)[\\x00\\x01]|ttcf\\x00[\\x01\\x02]\\x00\\x00|\\x00[\\x01\\x02]|(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])");
    map.insert("GIF", "GIF8[79]a");
    map.insert("GZIP", "\\x1f\\x8b\\x08");
    map.insert("HDR", "#\\?(RADIANCE|RGBE)\\x0a");
    map.insert("HTML", "(\\xef\\xbb\\xbf)?\\s*(?i)<(!DOCTYPE\\s+HTML|HTML|\\?xml)");
    map.insert("ICC", ".{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\\x00{4}){2}");
    map.insert("ICO", "\\x00\\x00[\\x01\\x02]\\x00[^0]\\x00");
    map.insert("IND", "\\x06\\x06\\xed\\xf5\\xd8\\x1d\\x46\\xe5\\xbd\\x31\\xef\\xe7\\xfe\\x74\\xb7\\x1d");
    map.insert("ITC", ".{4}itch");
    map.insert("JP2", "(\\x00\\x00\\x00\\x0cjP(  |\\x1a\\x1a)\\x0d\\x0a\\x87\\x0a|\\xff\\x4f\\xff\\x51\\x00)");
    map.insert("JPEG", "\\xff\\xd8\\xff");
    map.insert("JSON", "(\\xef\\xbb\\xbf)?\\s*(\\[\\s*)?\\{\\s*\"[^\"]*\"\\s*:");
    map.insert("JUMBF", ".{4}jumb\\x00.{3}jumd");
    map.insert("JXL", "(\\xff\\x0a|\\x00\\x00\\x00\\x0cJXL \\x0d\\x0a......ftypjxl )");
    map.insert("LFP", "\\x89LFP\\x0d\\x0a\\x1a\\x0a");
    map.insert("LIF", "\\x70\\x00{3}.{4}\\x2a.{4}<\\x00");
    map.insert("LNK", ".{4}\\x01\\x14\\x02\\x00{5}\\xc0\\x00{6}\\x46");
    map.insert("LRI", "LELR \\x00");
    map.insert("M2TS", "(....)?\\x47");
    map.insert("MIE", "~[\\x10\\x18]\\x04.0MIE");
    map.insert("MIFF", "id=ImageMagick");
    map.insert("MKV", "\\x1a\\x45\\xdf\\xa3");
    map.insert("MOI", "V6");
    map.insert("MOV", ".{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)");
    map.insert("MPC", "(MP\\+|ID3)");
    map.insert("MPEG", "\\x00\\x00\\x01[\\xb0-\\xbf]");
    map.insert("MRC", ".{64}[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00.{132}MAP[\\x00 ](\\x44\\x44|\\x44\\x41|\\x11\\x11)\\x00\\x00");
    map.insert("MRW", "\\x00MR[MI]");
    map.insert("MXF", "\\x06\\x0e\\x2b\\x34\\x02\\x05\\x01\\x01\\x0d\\x01\\x02");
    map.insert("MacOS", "\\x00\\x05\\x16\\x07\\x00.\\x00\\x00Mac OS X        ");
    map.insert("NKA", "NIKONADJ");
    map.insert("OGG", "(OggS|ID3)");
    map.insert("ORF", "(II|MM)");
    map.insert("PCAP", "\\xa1\\xb2(\\xc3\\xd4|\\x3c\\x4d)\\x00.\\x00.|(\\xd4\\xc3|\\x4d\\x3c)\\xb2\\xa1.\\x00.\\x00|\\x0a\\x0d\\x0d\\x0a.{4}(\\x1a\\x2b\\x3c\\x4d|\\x4d\\x3c\\x2b\\x1a)|GMBU\\x00\\x02");
    map.insert("PCX", "\\x0a[\\x00-\\x05]\\x01[\\x01\\x02\\x04\\x08].{64}[\\x00-\\x02]");
    map.insert("PDB", ".{60}(\\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)");
    map.insert("PDF", "\\s*%PDF-\\d+\\.\\d+");
    map.insert("PFM", "P[Ff]\\x0a\\d+ \\d+\\x0a[-+0-9.]+\\x0a");
    map.insert("PGF", "PGF");
    map.insert("PHP", "<\\?php\\s");
    map.insert("PICT", "(.{10}|.{522})(\\x11\\x01|\\x00\\x11)");
    map.insert("PLIST", "(bplist0|\\s*<|\\xfe\\xff\\x00)");
    map.insert("PMP", ".{8}\\x00{3}\\x7c.{112}\\xff\\xd8\\xff\\xdb");
    map.insert("PNG", "(\\x89P|\\x8aM|\\x8bJ)NG\\x0d\\x0a\\x1a\\x0a");
    map.insert("PPM", "P[1-6]\\s+");
    map.insert("PS", "(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)");
    map.insert("PSD", "8BPS\\x00[\\x01\\x02]");
    map.insert("PSP", "Paint Shop Pro Image File\\x0a\\x1a\\x00{5}");
    map.insert("QTIF", ".{4}(idsc|idat|iicc)");
    map.insert("R3D", "\\x00\\x00..RED(1|2)");
    map.insert("RAF", "FUJIFILM");
    map.insert("RAR", "Rar!\\x1a\\x07\\x01?\\x00");
    map.insert("RAW", "(.{25}ARECOYK|II|MM)");
    map.insert("RIFF", "(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)");
    map.insert("RSRC", "(....)?\\x00\\x00\\x01\\x00");
    map.insert("RTF", "[\\x0a\\x0d]*\\{[\\x0a\\x0d]*\\\\x0dtf");
    map.insert("RWZ", "rawzor");
    map.insert("Real", "(\\.RMF|\\.ra\\xfd|pnm://|rtsp://|http://)");
    map.insert("SWF", "[FC]WS[^\\x00]");
    map.insert("TAR", ".{257}ustar(  )?\\x00");
    map.insert("TIFF", "(II|MM)");
    map.insert("TXT", "(\\xff\\xfe|(\\x00\\x00)?\\xfe\\xff|(\\xef\\xbb\\xbf)?[\\x07-\\x0d\\x20-\\x7e\\x80-\\xfe]*$)");
    map.insert("Torrent", "d\\d+:\\w+");
    map.insert("VCard", "(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\\x0d\\x0a");
    map.insert("VRD", "CANON OPTIONAL DATA\\x00");
    map.insert("WMF", "(\\xd7\\xcd\\xc6\\x9a\\x00\\x00|\\x01\\x00\\x09\\x00\\x00\\x03)");
    map.insert("WPG", "\\xff\\x57\\x50\\x43");
    map.insert("WTV", "\\xb7\\xd8\\x00\\x20\\x37\\x49\\xda\\x11\\xa6\\x4e\\x00\\x07\\xe9\\x5e\\xad\\x8d");
    map.insert("X3F", "FOVb");
    map.insert("XCF", "gimp xcf ");
    map.insert("XISF", "XISF0100");
    map.insert("XMP", "\\x00{0,3}(\\xfe\\xff|\\xff\\xfe|\\xef\\xbb\\xbf)?\\x00{0,3}\\s*<");
    map.insert("ZIP", "PK\\x03\\x04");
    map
});

/// Compatibility status for each magic number pattern
static PATTERN_COMPATIBILITY: Lazy<HashMap<&'static str, bool>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("AA", true);
    map.insert("AAC", true);
    map.insert("AIFF", true);
    map.insert("ALIAS", true);
    map.insert("APE", true);
    map.insert("ASF", true);
    map.insert("AVC", true);
    map.insert("BMP", true);
    map.insert("BPG", true);
    map.insert("BTF", true);
    map.insert("BZ2", true);
    map.insert("CHM", true);
    map.insert("CRW", true);
    map.insert("CZI", true);
    map.insert("DCX", true);
    map.insert("DEX", true);
    map.insert("DICOM", true);
    map.insert("DOCX", true);
    map.insert("DPX", true);
    map.insert("DR4", true);
    map.insert("DSS", true);
    map.insert("DV", true);
    map.insert("DWF", true);
    map.insert("DWG", true);
    map.insert("DXF", true);
    map.insert("EPS", true);
    map.insert("EXE", true);
    map.insert("EXIF", true);
    map.insert("EXR", true);
    map.insert("EXV", true);
    map.insert("FITS", true);
    map.insert("FLAC", true);
    map.insert("FLIF", true);
    map.insert("FLIR", true);
    map.insert("FLV", true);
    map.insert("FPF", true);
    map.insert("FPX", true);
    map.insert("Font", true);
    map.insert("GIF", true);
    map.insert("GZIP", true);
    map.insert("HDR", true);
    map.insert("HTML", true);
    map.insert("ICC", true);
    map.insert("ICO", true);
    map.insert("IND", true);
    map.insert("ITC", true);
    map.insert("JP2", true);
    map.insert("JPEG", true);
    map.insert("JSON", true);
    map.insert("JUMBF", true);
    map.insert("JXL", true);
    map.insert("LFP", true);
    map.insert("LIF", true);
    map.insert("LNK", true);
    map.insert("LRI", true);
    map.insert("M2TS", true);
    map.insert("MIE", true);
    map.insert("MIFF", true);
    map.insert("MKV", true);
    map.insert("MOI", true);
    map.insert("MOV", true);
    map.insert("MPC", true);
    map.insert("MPEG", true);
    map.insert("MRC", true);
    map.insert("MRW", true);
    map.insert("MXF", true);
    map.insert("MacOS", true);
    map.insert("NKA", true);
    map.insert("OGG", true);
    map.insert("ORF", true);
    map.insert("PCAP", true);
    map.insert("PCX", true);
    map.insert("PDB", true);
    map.insert("PDF", true);
    map.insert("PFM", true);
    map.insert("PGF", true);
    map.insert("PHP", true);
    map.insert("PICT", true);
    map.insert("PLIST", true);
    map.insert("PMP", true);
    map.insert("PNG", true);
    map.insert("PPM", true);
    map.insert("PS", true);
    map.insert("PSD", true);
    map.insert("PSP", true);
    map.insert("QTIF", true);
    map.insert("R3D", true);
    map.insert("RAF", true);
    map.insert("RAR", true);
    map.insert("RAW", true);
    map.insert("RIFF", true);
    map.insert("RSRC", true);
    map.insert("RTF", true);
    map.insert("RWZ", true);
    map.insert("Real", true);
    map.insert("SWF", true);
    map.insert("TAR", true);
    map.insert("TIFF", true);
    map.insert("TXT", true);
    map.insert("Torrent", true);
    map.insert("VCard", true);
    map.insert("VRD", true);
    map.insert("WMF", true);
    map.insert("WPG", true);
    map.insert("WTV", true);
    map.insert("X3F", true);
    map.insert("XCF", true);
    map.insert("XISF", true);
    map.insert("XMP", true);
    map.insert("ZIP", true);
    map
});

/// Get magic number pattern for a file type
pub fn get_magic_number_pattern(file_type: &str) -> Option<&'static str> {
    MAGIC_NUMBER_PATTERNS.get(file_type).copied()
}

/// Check if a file type has a Rust-compatible magic number pattern
pub fn is_pattern_compatible(file_type: &str) -> bool {
    PATTERN_COMPATIBILITY.get(file_type).copied().unwrap_or(false)
}

/// Get all file types with magic number patterns
pub fn get_magic_file_types() -> Vec<&'static str> {
    MAGIC_NUMBER_PATTERNS.keys().copied().collect()
}

/// Get all file types with Rust-compatible patterns
pub fn get_compatible_file_types() -> Vec<&'static str> {
    PATTERN_COMPATIBILITY.iter()
        .filter(|(_, &compatible)| compatible)
        .map(|(&file_type, _)| file_type)
        .collect()
}
