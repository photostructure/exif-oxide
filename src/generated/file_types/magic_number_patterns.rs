//! Magic number regex patterns generated from ExifTool's magicNumber hash
//!
//! Total patterns: 111
//! Source: ExifTool.pm %magicNumber hash
//!
//! IMPORTANT: These patterns use bytes::RegexBuilder with unicode(false)
//! to ensure hex escapes like \x89 match raw bytes, not Unicode codepoints.

use crate::file_types::lazy_regex::LazyRegexMap;
use regex::bytes::Regex;
use std::sync::LazyLock;

/// Magic number patterns from ExifTool's %magicNumber hash
static PATTERN_DATA: &[(&str, &str)] = &[
    ("AA", "^.{4}\\x57\\x90\\x75\\x36"),
    ("AAC", "^\\xff[\\xf0\\xf1]"),
    ("AIFF", "^(FORM....AIF[FC]|AT&TFORM)"),
    ("ALIAS", "^book\\x00\\x00\\x00\\x00mark\\x00\\x00\\x00\\x00"),
    ("APE", "^(MAC |APETAGEX|ID3)"),
    ("ASF", "^\\x30\\x26\\xb2\\x75\\x8e\\x66\\xcf\\x11\\xa6\\xd9\\x00\\xaa\\x00\\x62\\xce\\x6c"),
    ("AVC", "^\\+A\\+V\\+C\\+"),
    ("BMP", "^BM"),
    ("BPG", "^BPG�"),
    ("BTF", "^(II\\x2b\\x00|MM\\x00\\x2b)"),
    ("BZ2", "^BZh[1-9]\\x31\\x41\\x59\\x26\\x53\\x59"),
    ("CHM", "^ITSF.{20}\\x10\\xfd\\x01\\x7c\\xaa\\x7b\\xd0\\x11\\x9e\\x0c\\x00\\xa0\\xc9\\x22\\xe6\\xec"),
    ("CRW", "^(II|MM).{4}HEAP(CCDR|JPGM)"),
    ("CZI", "^ZISRAWFILE\\x00{6}"),
    ("DCX", "^\\xb1\\x68\\xde\\x3a"),
    ("DEX", "^dex\\n035\\x00"),
    ("DICOM", "^(.{128}DICM|\\x00[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]|[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]\\x00)"),
    ("DOCX", "^PK\\x03\\x04"),
    ("DPX", "^(SDPX|XPDS)"),
    ("DR4", "^IIII[\\x04|\\x05]\\x00\\x04\\x00"),
    ("DSS", "^(\\x02dss|\\x03ds2)"),
    ("DV", "^\\x1f\\x07\\x00[\\x3f\\xbf]"),
    ("DWF", "^\\(DWF V\\d"),
    ("DWG", "^AC10\\d{2}\\x00"),
    ("DXF", "^\\s*0\\s+\\x00?\\s*SECTION\\s+2\\s+HEADER"),
    ("EPS", "^(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)"),
    ("EXE", "^(MZ|\\xca\\xfe\\xba\\xbe|\\xfe\\xed\\xfa[\\xce\\xcf]|[\\xce\\xcf]\\xfa\\xed\\xfe|Joy!peff|\\x7fELF|#!\\s*/\\S*bin/|!<arch>\\x0a)"),
    ("EXIF", "^(II\\x2a\\x00|MM\\x00\\x2a)"),
    ("EXR", "^\\x76\\x2f\\x31\\x01"),
    ("EXV", "^\\xff\\x01Exiv2"),
    ("FITS", "^SIMPLE  = {20}T"),
    ("FLAC", "^(fLaC|ID3)"),
    ("FLIF", "^FLIF[0-\\x6f][0-2]"),
    ("FLIR", "^[AF]FF\\x00"),
    ("FLV", "^FLV\\x01"),
    ("FPF", "^FPF Public Image Format\\x00"),
    ("FPX", "^\\xd0\\xcf\\x11\\xe0\\xa1\\xb1\\x1a\\xe1"),
    ("Font", "^((\\x00\\x01\\x00\\x00|OTTO|true|typ1)[\\x00\\x01]|ttcf\\x00[\\x01\\x02]\\x00\\x00|\\x00[\\x01\\x02]|(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])"),
    ("GIF", "^GIF8[79]a"),
    ("GZIP", "^\\x1f\\x8b\\x08"),
    ("HDR", "^#\\?(RADIANCE|RGBE)\\x0a"),
    ("HTML", "^(\\xef\\xbb\\xbf)?\\s*(?i)<(!DOCTYPE\\s+HTML|HTML|\\?xml)"),
    ("ICC", "^.{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\\x00{4}){2}"),
    ("ICO", "^\\x00\\x00[\\x01\\x02]\\x00[^0]\\x00"),
    ("IND", "^\\x06\\x06\\xed\\xf5\\xd8\\x1d\\x46\\xe5\\xbd\\x31\\xef\\xe7\\xfe\\x74\\xb7\\x1d"),
    ("ITC", "^.{4}itch"),
    ("JP2", "^(\\x00\\x00\\x00\\x0cjP(  |\\x1a\\x1a)\\x0d\\x0a\\x87\\x0a|\\xff\\x4f\\xff\\x51\\x00)"),
    ("JPEG", "^\\xff\\xd8\\xff"),
    ("JSON", "^(\\xef\\xbb\\xbf)?\\s*(\\[\\s*)?\\{\\s*\"[^\"]*\"\\s*:"),
    ("JUMBF", "^.{4}jumb\\x00.{3}jumd"),
    ("JXL", "^(\\xff\\x0a|\\x00\\x00\\x00\\x0cJXL \\x0d\\x0a......ftypjxl )"),
    ("LFP", "^\\x89LFP\\x0d\\x0a\\x1a\\x0a"),
    ("LIF", "^\\x70\\x00{3}.{4}\\x2a.{4}<\\x00"),
    ("LNK", "^.{4}\\x01\\x14\\x02\\x00{5}\\xc0\\x00{6}\\x46"),
    ("LRI", "^LELR \\x00"),
    ("M2TS", "^(....)?\\x47"),
    ("MIE", "^~[\\x10\\x18]\\x04.0MIE"),
    ("MIFF", "^id=ImageMagick"),
    ("MKV", "^\\x1a\\x45\\xdf\\xa3"),
    ("MOI", "^V6"),
    ("MOV", "^.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)"),
    ("MPC", "^(MP\\+|ID3)"),
    ("MPEG", "^\\x00\\x00\\x01[\\xb0-\\xbf]"),
    ("MRC", "^.{64}[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00.{132}MAP[\\x00 ](\\x44\\x44|\\x44\\x41|\\x11\\x11)\\x00\\x00"),
    ("MRW", "^\\x00MR[MI]"),
    ("MXF", "^\\x06\\x0e\\x2b\\x34\\x02\\x05\\x01\\x01\\x0d\\x01\\x02"),
    ("MacOS", "^\\x00\\x05\\x16\\x07\\x00.\\x00\\x00Mac OS X        "),
    ("NKA", "^NIKONADJ"),
    ("OGG", "^(OggS|ID3)"),
    ("ORF", "^(II|MM)"),
    ("PCAP", "^\\xa1\\xb2(\\xc3\\xd4|\\x3c\\x4d)\\x00.\\x00.|(\\xd4\\xc3|\\x4d\\x3c)\\xb2\\xa1.\\x00.\\x00|\\x0a\\x0d\\x0d\\x0a.{4}(\\x1a\\x2b\\x3c\\x4d|\\x4d\\x3c\\x2b\\x1a)|GMBU\\x00\\x02"),
    ("PCX", "^\\x0a[\\x00-\\x05]\\x01[\\x01\\x02\\x04\\x08].{64}[\\x00-\\x02]"),
    ("PDB", "^.{60}(\\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)"),
    ("PDF", "^\\s*%PDF-\\d+\\.\\d+"),
    ("PFM", "^P[Ff]\\x0a\\d+ \\d+\\x0a[-+0-9.]+\\x0a"),
    ("PGF", "^PGF"),
    ("PHP", "^<\\?php\\s"),
    ("PICT", "^(.{10}|.{522})(\\x11\\x01|\\x00\\x11)"),
    ("PLIST", "^(bplist0|\\s*<|\\xfe\\xff\\x00)"),
    ("PMP", "^.{8}\\x00{3}\\x7c.{112}\\xff\\xd8\\xff\\xdb"),
    ("PNG", "^(\\x89P|\\x8aM|\\x8bJ)NG\\r\\n\\x1a\\n"),
    ("PPM", "^P[1-6]\\s+"),
    ("PS", "^(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)"),
    ("PSD", "^8BPS\\x00[\\x01\\x02]"),
    ("PSP", "^Paint Shop Pro Image File\\x0a\\x1a\\x00{5}"),
    ("QTIF", "^.{4}(idsc|idat|iicc)"),
    ("R3D", "^\\x00\\x00..RED(1|2)"),
    ("RAF", "^FUJIFILM"),
    ("RAR", "^Rar!\\x1a\\x07\\x01?\\x00"),
    ("RAW", "^(.{25}ARECOYK|II|MM)"),
    ("RIFF", "^(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)"),
    ("RSRC", "^(....)?\\x00\\x00\\x01\\x00"),
    ("RTF", "^[\\n\\r]*\\{[\\n\\r]*\\\\rtf"),
    ("RWZ", "^rawzor"),
    ("Real", "^(\\.RMF|\\.ra\\xfd|pnm://|rtsp://|http://)"),
    ("SWF", "^[FC]WS[^\\x00]"),
    ("TAR", "^.{257}ustar(  )?\\x00"),
    ("TIFF", "^(II|MM)"),
    ("TNEF", "^\\x78\\x9f\\x3e\\x22..\\x01\\x06\\x90\\x08\\x00"),
    ("TXT", "^(\\xff\\xfe|(\\x00\\x00)?\\xfe\\xff|(\\xef\\xbb\\xbf)?[\\x07-\\x0d\\x20-\\x7e\\x80-\\xfe]*$)"),
    ("Torrent", "^d\\d+:\\w+"),
    ("VCard", "^(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\\r\\n"),
    ("VRD", "^CANON OPTIONAL DATA\\x00"),
    ("WMF", "^(\\xd7\\xcd\\xc6\\x9a\\x00\\x00|\\x01\\x00\\x09\\x00\\x00\\x03)"),
    ("WPG", "^\\xff\\x57\\x50\\x43"),
    ("WTV", "^\\xb7\\xd8\\x00\\x20\\x37\\x49\\xda\\x11\\xa6\\x4e\\x00\\x07\\xe9\\x5e\\xad\\x8d"),
    ("X3F", "^FOVb"),
    ("XCF", "^gimp xcf "),
    ("XISF", "^XISF0100"),
    ("XMP", "^\\x00{0,3}(\\xfe\\xff|\\xff\\xfe|\\xef\\xbb\\xbf)?\\x00{0,3}\\s*<"),
    ("ZIP", "^PK\\x03\\x04"),
];

/// Lazy-compiled regex patterns for magic number detection
static MAGIC_PATTERNS: LazyLock<LazyRegexMap> = LazyLock::new(|| LazyRegexMap::new(PATTERN_DATA));

/// Test if a byte buffer matches a file type's magic number pattern
pub fn matches_magic_number(file_type: &str, buffer: &[u8]) -> bool {
    MAGIC_PATTERNS.matches(file_type, buffer)
}

/// Get all file types with magic number patterns
pub fn get_magic_file_types() -> Vec<&'static str> {
    MAGIC_PATTERNS.file_types()
}

/// Get compiled magic number regex for a file type
/// Uses the cached version if available, compiles and caches if not
pub fn get_magic_number_pattern(file_type: &str) -> Option<Regex> {
    MAGIC_PATTERNS.get_regex(file_type)
}
