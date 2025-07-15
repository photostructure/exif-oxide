//! Magic number regex patterns generated from ExifTool's magicNumber hash
//!
//! Generated at: Tue Jul 15 22:55:37 2025 GMT
//! Total patterns: 110
//! Source: ExifTool.pm %magicNumber hash
//!
//! IMPORTANT: These patterns use bytes::RegexBuilder with unicode(false)
//! to ensure hex escapes like \x89 match raw bytes, not Unicode codepoints.

use once_cell::sync::Lazy;
use regex::bytes::{Regex, RegexBuilder};
use std::collections::HashMap;

/// Compiled magic number regex patterns for file type detection
/// These patterns match at the beginning of files (anchored with ^)
/// Unicode mode is disabled to ensure hex escapes match raw bytes
static MAGIC_NUMBER_PATTERNS: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Each pattern is compiled once at initialization
    // Unicode mode is disabled so \x89 matches byte 0x89, not U+0089
    map.insert(
        "AA",
        RegexBuilder::new("^.{4}\\x57\\x90\\x75\\x36")
            .unicode(false)
            .build()
            .expect("Invalid regex for AA"),
    );
    map.insert(
        "AAC",
        RegexBuilder::new("^\\xff[\\xf0\\xf1]")
            .unicode(false)
            .build()
            .expect("Invalid regex for AAC"),
    );
    map.insert(
        "AIFF",
        RegexBuilder::new("^(FORM....AIF[FC]|AT&TFORM)")
            .unicode(false)
            .build()
            .expect("Invalid regex for AIFF"),
    );
    map.insert(
        "ALIAS",
        RegexBuilder::new("^book\\x00\\x00\\x00\\x00mark\\x00\\x00\\x00\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for ALIAS"),
    );
    map.insert(
        "APE",
        RegexBuilder::new("^(MAC |APETAGEX|ID3)")
            .unicode(false)
            .build()
            .expect("Invalid regex for APE"),
    );
    map.insert(
        "ASF",
        RegexBuilder::new(
            "^\\x30\\x26\\xb2\\x75\\x8e\\x66\\xcf\\x11\\xa6\\xd9\\x00\\xaa\\x00\\x62\\xce\\x6c",
        )
        .unicode(false)
        .build()
        .expect("Invalid regex for ASF"),
    );
    map.insert(
        "AVC",
        RegexBuilder::new("^\\+A\\+V\\+C\\+")
            .unicode(false)
            .build()
            .expect("Invalid regex for AVC"),
    );
    map.insert(
        "BMP",
        RegexBuilder::new("^BM")
            .unicode(false)
            .build()
            .expect("Invalid regex for BMP"),
    );
    map.insert(
        "BPG",
        RegexBuilder::new("^BPG\\xfb")
            .unicode(false)
            .build()
            .expect("Invalid regex for BPG"),
    );
    map.insert(
        "BTF",
        RegexBuilder::new("^(II\\x2b\\x00|MM\\x00\\x2b)")
            .unicode(false)
            .build()
            .expect("Invalid regex for BTF"),
    );
    map.insert(
        "BZ2",
        RegexBuilder::new("^BZh[1-9]\\x31\\x41\\x59\\x26\\x53\\x59")
            .unicode(false)
            .build()
            .expect("Invalid regex for BZ2"),
    );
    map.insert("CHM", RegexBuilder::new("^ITSF.{20}\\x10\\xfd\\x01\\x7c\\xaa\\x7b\\xd0\\x11\\x9e\\x0c\\x00\\xa0\\xc9\\x22\\xe6\\xec").unicode(false).build().expect("Invalid regex for CHM"));
    map.insert(
        "CRW",
        RegexBuilder::new("^(II|MM).{4}HEAP(CCDR|JPGM)")
            .unicode(false)
            .build()
            .expect("Invalid regex for CRW"),
    );
    map.insert(
        "CZI",
        RegexBuilder::new("^ZISRAWFILE\\x00{6}")
            .unicode(false)
            .build()
            .expect("Invalid regex for CZI"),
    );
    map.insert(
        "DCX",
        RegexBuilder::new("^\\xb1\\x68\\xde\\x3a")
            .unicode(false)
            .build()
            .expect("Invalid regex for DCX"),
    );
    map.insert(
        "DEX",
        RegexBuilder::new("^dex\\n035\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for DEX"),
    );
    map.insert("DICOM", RegexBuilder::new("^(.{128}DICM|\\x00[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]|[\\x02\\x04\\x06\\x08]\\x00[\\x00-\\x20]\\x00)").unicode(false).build().expect("Invalid regex for DICOM"));
    map.insert(
        "DOCX",
        RegexBuilder::new("^PK\\x03\\x04")
            .unicode(false)
            .build()
            .expect("Invalid regex for DOCX"),
    );
    map.insert(
        "DPX",
        RegexBuilder::new("^(SDPX|XPDS)")
            .unicode(false)
            .build()
            .expect("Invalid regex for DPX"),
    );
    map.insert(
        "DR4",
        RegexBuilder::new("^IIII[\\x04|\\x05]\\x00\\x04\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for DR4"),
    );
    map.insert(
        "DSS",
        RegexBuilder::new("^(\\x02dss|\\x03ds2)")
            .unicode(false)
            .build()
            .expect("Invalid regex for DSS"),
    );
    map.insert(
        "DV",
        RegexBuilder::new("^\\x1f\\x07\\x00[\\x3f\\xbf]")
            .unicode(false)
            .build()
            .expect("Invalid regex for DV"),
    );
    map.insert(
        "DWF",
        RegexBuilder::new("^\\(DWF V\\d")
            .unicode(false)
            .build()
            .expect("Invalid regex for DWF"),
    );
    map.insert(
        "DWG",
        RegexBuilder::new("^AC10\\d{2}\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for DWG"),
    );
    map.insert(
        "DXF",
        RegexBuilder::new("^\\s*0\\s+\\x00?\\s*SECTION\\s+2\\s+HEADER")
            .unicode(false)
            .build()
            .expect("Invalid regex for DXF"),
    );
    map.insert(
        "EPS",
        RegexBuilder::new("^(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)")
            .unicode(false)
            .build()
            .expect("Invalid regex for EPS"),
    );
    map.insert("EXE", RegexBuilder::new("^(MZ|\\xca\\xfe\\xba\\xbe|\\xfe\\xed\\xfa[\\xce\\xcf]|[\\xce\\xcf]\\xfa\\xed\\xfe|Joy!peff|\\x7fELF|#!\\s*/\\S*bin/|!<arch>\\x0a)").unicode(false).build().expect("Invalid regex for EXE"));
    map.insert(
        "EXIF",
        RegexBuilder::new("^(II\\x2a\\x00|MM\\x00\\x2a)")
            .unicode(false)
            .build()
            .expect("Invalid regex for EXIF"),
    );
    map.insert(
        "EXR",
        RegexBuilder::new("^\\x76\\x2f\\x31\\x01")
            .unicode(false)
            .build()
            .expect("Invalid regex for EXR"),
    );
    map.insert(
        "EXV",
        RegexBuilder::new("^\\xff\\x01Exiv2")
            .unicode(false)
            .build()
            .expect("Invalid regex for EXV"),
    );
    map.insert(
        "FITS",
        RegexBuilder::new("^SIMPLE  = {20}T")
            .unicode(false)
            .build()
            .expect("Invalid regex for FITS"),
    );
    map.insert(
        "FLAC",
        RegexBuilder::new("^(fLaC|ID3)")
            .unicode(false)
            .build()
            .expect("Invalid regex for FLAC"),
    );
    map.insert(
        "FLIF",
        RegexBuilder::new("^FLIF[0-\\x6f][0-2]")
            .unicode(false)
            .build()
            .expect("Invalid regex for FLIF"),
    );
    map.insert(
        "FLIR",
        RegexBuilder::new("^[AF]FF\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for FLIR"),
    );
    map.insert(
        "FLV",
        RegexBuilder::new("^FLV\\x01")
            .unicode(false)
            .build()
            .expect("Invalid regex for FLV"),
    );
    map.insert(
        "FPF",
        RegexBuilder::new("^FPF Public Image Format\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for FPF"),
    );
    map.insert(
        "FPX",
        RegexBuilder::new("^\\xd0\\xcf\\x11\\xe0\\xa1\\xb1\\x1a\\xe1")
            .unicode(false)
            .build()
            .expect("Invalid regex for FPX"),
    );
    map.insert("Font", RegexBuilder::new("^((\\x00\\x01\\x00\\x00|OTTO|true|typ1)[\\x00\\x01]|ttcf\\x00[\\x01\\x02]\\x00\\x00|\\x00[\\x01\\x02]|(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])").unicode(false).build().expect("Invalid regex for Font"));
    map.insert(
        "GIF",
        RegexBuilder::new("^GIF8[79]a")
            .unicode(false)
            .build()
            .expect("Invalid regex for GIF"),
    );
    map.insert(
        "GZIP",
        RegexBuilder::new("^\\x1f\\x8b\\x08")
            .unicode(false)
            .build()
            .expect("Invalid regex for GZIP"),
    );
    map.insert(
        "HDR",
        RegexBuilder::new("^#\\?(RADIANCE|RGBE)\\x0a")
            .unicode(false)
            .build()
            .expect("Invalid regex for HDR"),
    );
    map.insert(
        "HTML",
        RegexBuilder::new("^(\\xef\\xbb\\xbf)?\\s*(?i)<(!DOCTYPE\\s+HTML|HTML|\\?xml)")
            .unicode(false)
            .build()
            .expect("Invalid regex for HTML"),
    );
    map.insert("ICC", RegexBuilder::new("^.{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\\x00{4}){2}").unicode(false).build().expect("Invalid regex for ICC"));
    map.insert(
        "ICO",
        RegexBuilder::new("^\\x00\\x00[\\x01\\x02]\\x00[^0]\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for ICO"),
    );
    map.insert(
        "IND",
        RegexBuilder::new(
            "^\\x06\\x06\\xed\\xf5\\xd8\\x1d\\x46\\xe5\\xbd\\x31\\xef\\xe7\\xfe\\x74\\xb7\\x1d",
        )
        .unicode(false)
        .build()
        .expect("Invalid regex for IND"),
    );
    map.insert(
        "ITC",
        RegexBuilder::new("^.{4}itch")
            .unicode(false)
            .build()
            .expect("Invalid regex for ITC"),
    );
    map.insert("JP2", RegexBuilder::new("^(\\x00\\x00\\x00\\x0cjP(  |\\x1a\\x1a)\\x0d\\x0a\\x87\\x0a|\\xff\\x4f\\xff\\x51\\x00)").unicode(false).build().expect("Invalid regex for JP2"));
    map.insert(
        "JPEG",
        RegexBuilder::new("^\\xff\\xd8\\xff")
            .unicode(false)
            .build()
            .expect("Invalid regex for JPEG"),
    );
    map.insert(
        "JSON",
        RegexBuilder::new("^(\\xef\\xbb\\xbf)?\\s*(\\[\\s*)?\\{\\s*\"[^\"]*\"\\s*:")
            .unicode(false)
            .build()
            .expect("Invalid regex for JSON"),
    );
    map.insert(
        "JUMBF",
        RegexBuilder::new("^.{4}jumb\\x00.{3}jumd")
            .unicode(false)
            .build()
            .expect("Invalid regex for JUMBF"),
    );
    map.insert(
        "JXL",
        RegexBuilder::new("^(\\xff\\x0a|\\x00\\x00\\x00\\x0cJXL \\x0d\\x0a......ftypjxl )")
            .unicode(false)
            .build()
            .expect("Invalid regex for JXL"),
    );
    map.insert(
        "LFP",
        RegexBuilder::new("^\\x89LFP\\x0d\\x0a\\x1a\\x0a")
            .unicode(false)
            .build()
            .expect("Invalid regex for LFP"),
    );
    map.insert(
        "LIF",
        RegexBuilder::new("^\\x70\\x00{3}.{4}\\x2a.{4}<\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for LIF"),
    );
    map.insert(
        "LNK",
        RegexBuilder::new("^.{4}\\x01\\x14\\x02\\x00{5}\\xc0\\x00{6}\\x46")
            .unicode(false)
            .build()
            .expect("Invalid regex for LNK"),
    );
    map.insert(
        "LRI",
        RegexBuilder::new("^LELR \\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for LRI"),
    );
    map.insert(
        "M2TS",
        RegexBuilder::new("^(....)?\\x47")
            .unicode(false)
            .build()
            .expect("Invalid regex for M2TS"),
    );
    map.insert(
        "MIE",
        RegexBuilder::new("^~[\\x10\\x18]\\x04.0MIE")
            .unicode(false)
            .build()
            .expect("Invalid regex for MIE"),
    );
    map.insert(
        "MIFF",
        RegexBuilder::new("^id=ImageMagick")
            .unicode(false)
            .build()
            .expect("Invalid regex for MIFF"),
    );
    map.insert(
        "MKV",
        RegexBuilder::new("^\\x1a\\x45\\xdf\\xa3")
            .unicode(false)
            .build()
            .expect("Invalid regex for MKV"),
    );
    map.insert(
        "MOI",
        RegexBuilder::new("^V6")
            .unicode(false)
            .build()
            .expect("Invalid regex for MOI"),
    );
    map.insert(
        "MOV",
        RegexBuilder::new("^.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)")
            .unicode(false)
            .build()
            .expect("Invalid regex for MOV"),
    );
    map.insert(
        "MPC",
        RegexBuilder::new("^(MP\\+|ID3)")
            .unicode(false)
            .build()
            .expect("Invalid regex for MPC"),
    );
    map.insert(
        "MPEG",
        RegexBuilder::new("^\\x00\\x00\\x01[\\xb0-\\xbf]")
            .unicode(false)
            .build()
            .expect("Invalid regex for MPEG"),
    );
    map.insert("MRC", RegexBuilder::new("^.{64}[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00[\\x01\\x02\\x03]\\x00\\x00\\x00.{132}MAP[\\x00 ](\\x44\\x44|\\x44\\x41|\\x11\\x11)\\x00\\x00").unicode(false).build().expect("Invalid regex for MRC"));
    map.insert(
        "MRW",
        RegexBuilder::new("^\\x00MR[MI]")
            .unicode(false)
            .build()
            .expect("Invalid regex for MRW"),
    );
    map.insert(
        "MXF",
        RegexBuilder::new("^\\x06\\x0e\\x2b\\x34\\x02\\x05\\x01\\x01\\x0d\\x01\\x02")
            .unicode(false)
            .build()
            .expect("Invalid regex for MXF"),
    );
    map.insert(
        "MacOS",
        RegexBuilder::new("^\\x00\\x05\\x16\\x07\\x00.\\x00\\x00Mac OS X        ")
            .unicode(false)
            .build()
            .expect("Invalid regex for MacOS"),
    );
    map.insert(
        "NKA",
        RegexBuilder::new("^NIKONADJ")
            .unicode(false)
            .build()
            .expect("Invalid regex for NKA"),
    );
    map.insert(
        "OGG",
        RegexBuilder::new("^(OggS|ID3)")
            .unicode(false)
            .build()
            .expect("Invalid regex for OGG"),
    );
    map.insert(
        "ORF",
        RegexBuilder::new("^(II|MM)")
            .unicode(false)
            .build()
            .expect("Invalid regex for ORF"),
    );
    map.insert("PCAP", RegexBuilder::new("^\\xa1\\xb2(\\xc3\\xd4|\\x3c\\x4d)\\x00.\\x00.|(\\xd4\\xc3|\\x4d\\x3c)\\xb2\\xa1.\\x00.\\x00|\\x0a\\x0d\\x0d\\x0a.{4}(\\x1a\\x2b\\x3c\\x4d|\\x4d\\x3c\\x2b\\x1a)|GMBU\\x00\\x02").unicode(false).build().expect("Invalid regex for PCAP"));
    map.insert(
        "PCX",
        RegexBuilder::new("^\\x0a[\\x00-\\x05]\\x01[\\x01\\x02\\x04\\x08].{64}[\\x00-\\x02]")
            .unicode(false)
            .build()
            .expect("Invalid regex for PCX"),
    );
    map.insert("PDB", RegexBuilder::new("^.{60}(\\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)").unicode(false).build().expect("Invalid regex for PDB"));
    map.insert(
        "PDF",
        RegexBuilder::new("^\\s*%PDF-\\d+\\.\\d+")
            .unicode(false)
            .build()
            .expect("Invalid regex for PDF"),
    );
    map.insert(
        "PFM",
        RegexBuilder::new("^P[Ff]\\x0a\\d+ \\d+\\x0a[-+0-9.]+\\x0a")
            .unicode(false)
            .build()
            .expect("Invalid regex for PFM"),
    );
    map.insert(
        "PGF",
        RegexBuilder::new("^PGF")
            .unicode(false)
            .build()
            .expect("Invalid regex for PGF"),
    );
    map.insert(
        "PHP",
        RegexBuilder::new("^<\\?php\\s")
            .unicode(false)
            .build()
            .expect("Invalid regex for PHP"),
    );
    map.insert(
        "PICT",
        RegexBuilder::new("^(.{10}|.{522})(\\x11\\x01|\\x00\\x11)")
            .unicode(false)
            .build()
            .expect("Invalid regex for PICT"),
    );
    map.insert(
        "PLIST",
        RegexBuilder::new("^(bplist0|\\s*<|\\xfe\\xff\\x00)")
            .unicode(false)
            .build()
            .expect("Invalid regex for PLIST"),
    );
    map.insert(
        "PMP",
        RegexBuilder::new("^.{8}\\x00{3}\\x7c.{112}\\xff\\xd8\\xff\\xdb")
            .unicode(false)
            .build()
            .expect("Invalid regex for PMP"),
    );
    map.insert(
        "PNG",
        RegexBuilder::new("^(\\x89P|\\x8aM|\\x8bJ)NG\\r\\n\\x1a\\n")
            .unicode(false)
            .build()
            .expect("Invalid regex for PNG"),
    );
    map.insert(
        "PPM",
        RegexBuilder::new("^P[1-6]\\s+")
            .unicode(false)
            .build()
            .expect("Invalid regex for PPM"),
    );
    map.insert(
        "PS",
        RegexBuilder::new("^(%!PS|%!Ad|\\xc5\\xd0\\xd3\\xc6)")
            .unicode(false)
            .build()
            .expect("Invalid regex for PS"),
    );
    map.insert(
        "PSD",
        RegexBuilder::new("^8BPS\\x00[\\x01\\x02]")
            .unicode(false)
            .build()
            .expect("Invalid regex for PSD"),
    );
    map.insert(
        "PSP",
        RegexBuilder::new("^Paint Shop Pro Image File\\x0a\\x1a\\x00{5}")
            .unicode(false)
            .build()
            .expect("Invalid regex for PSP"),
    );
    map.insert(
        "QTIF",
        RegexBuilder::new("^.{4}(idsc|idat|iicc)")
            .unicode(false)
            .build()
            .expect("Invalid regex for QTIF"),
    );
    map.insert(
        "R3D",
        RegexBuilder::new("^\\x00\\x00..RED(1|2)")
            .unicode(false)
            .build()
            .expect("Invalid regex for R3D"),
    );
    map.insert(
        "RAF",
        RegexBuilder::new("^FUJIFILM")
            .unicode(false)
            .build()
            .expect("Invalid regex for RAF"),
    );
    map.insert(
        "RAR",
        RegexBuilder::new("^Rar!\\x1a\\x07\\x01?\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for RAR"),
    );
    map.insert(
        "RAW",
        RegexBuilder::new("^(.{25}ARECOYK|II|MM)")
            .unicode(false)
            .build()
            .expect("Invalid regex for RAW"),
    );
    map.insert(
        "RIFF",
        RegexBuilder::new("^(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)")
            .unicode(false)
            .build()
            .expect("Invalid regex for RIFF"),
    );
    map.insert(
        "RSRC",
        RegexBuilder::new("^(....)?\\x00\\x00\\x01\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for RSRC"),
    );
    map.insert(
        "RTF",
        RegexBuilder::new("^[\\n\\r]*\\{[\\n\\r]*\\\\rtf")
            .unicode(false)
            .build()
            .expect("Invalid regex for RTF"),
    );
    map.insert(
        "RWZ",
        RegexBuilder::new("^rawzor")
            .unicode(false)
            .build()
            .expect("Invalid regex for RWZ"),
    );
    map.insert(
        "Real",
        RegexBuilder::new("^(\\.RMF|\\.ra\\xfd|pnm://|rtsp://|http://)")
            .unicode(false)
            .build()
            .expect("Invalid regex for Real"),
    );
    map.insert(
        "SWF",
        RegexBuilder::new("^[FC]WS[^\\x00]")
            .unicode(false)
            .build()
            .expect("Invalid regex for SWF"),
    );
    map.insert(
        "TAR",
        RegexBuilder::new("^.{257}ustar(  )?\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for TAR"),
    );
    map.insert(
        "TIFF",
        RegexBuilder::new("^(II|MM)")
            .unicode(false)
            .build()
            .expect("Invalid regex for TIFF"),
    );
    map.insert("TXT", RegexBuilder::new("^(\\xff\\xfe|(\\x00\\x00)?\\xfe\\xff|(\\xef\\xbb\\xbf)?[\\x07-\\x0d\\x20-\\x7e\\x80-\\xfe]*$)").unicode(false).build().expect("Invalid regex for TXT"));
    map.insert(
        "Torrent",
        RegexBuilder::new("^d\\d+:\\w+")
            .unicode(false)
            .build()
            .expect("Invalid regex for Torrent"),
    );
    map.insert(
        "VCard",
        RegexBuilder::new("^(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\\r\\n")
            .unicode(false)
            .build()
            .expect("Invalid regex for VCard"),
    );
    map.insert(
        "VRD",
        RegexBuilder::new("^CANON OPTIONAL DATA\\x00")
            .unicode(false)
            .build()
            .expect("Invalid regex for VRD"),
    );
    map.insert(
        "WMF",
        RegexBuilder::new("^(\\xd7\\xcd\\xc6\\x9a\\x00\\x00|\\x01\\x00\\x09\\x00\\x00\\x03)")
            .unicode(false)
            .build()
            .expect("Invalid regex for WMF"),
    );
    map.insert(
        "WPG",
        RegexBuilder::new("^\\xff\\x57\\x50\\x43")
            .unicode(false)
            .build()
            .expect("Invalid regex for WPG"),
    );
    map.insert(
        "WTV",
        RegexBuilder::new(
            "^\\xb7\\xd8\\x00\\x20\\x37\\x49\\xda\\x11\\xa6\\x4e\\x00\\x07\\xe9\\x5e\\xad\\x8d",
        )
        .unicode(false)
        .build()
        .expect("Invalid regex for WTV"),
    );
    map.insert(
        "X3F",
        RegexBuilder::new("^FOVb")
            .unicode(false)
            .build()
            .expect("Invalid regex for X3F"),
    );
    map.insert(
        "XCF",
        RegexBuilder::new("^gimp xcf ")
            .unicode(false)
            .build()
            .expect("Invalid regex for XCF"),
    );
    map.insert(
        "XISF",
        RegexBuilder::new("^XISF0100")
            .unicode(false)
            .build()
            .expect("Invalid regex for XISF"),
    );
    map.insert(
        "XMP",
        RegexBuilder::new("^\\x00{0,3}(\\xfe\\xff|\\xff\\xfe|\\xef\\xbb\\xbf)?\\x00{0,3}\\s*<")
            .unicode(false)
            .build()
            .expect("Invalid regex for XMP"),
    );
    map.insert(
        "ZIP",
        RegexBuilder::new("^PK\\x03\\x04")
            .unicode(false)
            .build()
            .expect("Invalid regex for ZIP"),
    );

    map
});

/// Get compiled magic number regex for a file type
pub fn get_magic_number_pattern(file_type: &str) -> Option<&'static Regex> {
    MAGIC_NUMBER_PATTERNS.get(file_type)
}

/// Test if a byte buffer matches a file type's magic number pattern
pub fn matches_magic_number(file_type: &str, buffer: &[u8]) -> bool {
    if let Some(regex) = get_magic_number_pattern(file_type) {
        regex.is_match(buffer)
    } else {
        false
    }
}

/// Get all file types with magic number patterns
pub fn get_magic_file_types() -> Vec<&'static str> {
    MAGIC_NUMBER_PATTERNS.keys().copied().collect()
}
