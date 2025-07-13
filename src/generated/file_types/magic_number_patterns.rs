//! Magic number regex patterns generated from ExifTool's magicNumber hash
//!
//! Generated at: Fri Jul 11 04:48:48 2025 GMT
//! Total patterns: 110
//! Source: ExifTool.pm %magicNumber hash

use regex::bytes::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Compiled magic number regex patterns for file type detection
/// These patterns match at the beginning of files (anchored with ^)
static MAGIC_NUMBER_PATTERNS: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Each pattern is compiled once at initialization
    // Patterns use raw strings (r"...") to avoid double-escaping
    map.insert("AA", Regex::new(r"^.{4}\x57\x90\x75\x36").expect("Invalid regex for AA"));
    map.insert("AAC", Regex::new(r"^\xff[\xf0\xf1]").expect("Invalid regex for AAC"));
    map.insert("AIFF", Regex::new(r"^(FORM....AIF[FC]|AT&TFORM)").expect("Invalid regex for AIFF"));
    map.insert("ALIAS", Regex::new(r"^book\x00\x00\x00\x00mark\x00\x00\x00\x00").expect("Invalid regex for ALIAS"));
    map.insert("APE", Regex::new(r"^(MAC |APETAGEX|ID3)").expect("Invalid regex for APE"));
    map.insert("ASF", Regex::new(r"^\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c").expect("Invalid regex for ASF"));
    map.insert("AVC", Regex::new(r"^\+A\+V\+C\+").expect("Invalid regex for AVC"));
    map.insert("BMP", Regex::new(r"^BM").expect("Invalid regex for BMP"));
    map.insert("BPG", Regex::new(r"^BPG\xfb").expect("Invalid regex for BPG"));
    map.insert("BTF", Regex::new(r"^(II\x2b\x00|MM\x00\x2b)").expect("Invalid regex for BTF"));
    map.insert("BZ2", Regex::new(r"^BZh[1-9]\x31\x41\x59\x26\x53\x59").expect("Invalid regex for BZ2"));
    map.insert("CHM", Regex::new(r"^ITSF.{20}\x10\xfd\x01\x7c\xaa\x7b\xd0\x11\x9e\x0c\x00\xa0\xc9\x22\xe6\xec").expect("Invalid regex for CHM"));
    map.insert("CRW", Regex::new(r"^(II|MM).{4}HEAP(CCDR|JPGM)").expect("Invalid regex for CRW"));
    map.insert("CZI", Regex::new(r"^ZISRAWFILE\x00{6}").expect("Invalid regex for CZI"));
    map.insert("DCX", Regex::new(r"^\xb1\x68\xde\x3a").expect("Invalid regex for DCX"));
    map.insert("DEX", Regex::new(r"^dex\n035\x00").expect("Invalid regex for DEX"));
    map.insert("DICOM", Regex::new(r"^(.{128}DICM|\x00[\x02\x04\x06\x08]\x00[\x00-\x20]|[\x02\x04\x06\x08]\x00[\x00-\x20]\x00)").expect("Invalid regex for DICOM"));
    map.insert("DOCX", Regex::new(r"^PK\x03\x04").expect("Invalid regex for DOCX"));
    map.insert("DPX", Regex::new(r"^(SDPX|XPDS)").expect("Invalid regex for DPX"));
    map.insert("DR4", Regex::new(r"^IIII[\x04|\x05]\x00\x04\x00").expect("Invalid regex for DR4"));
    map.insert("DSS", Regex::new(r"^(\x02dss|\x03ds2)").expect("Invalid regex for DSS"));
    map.insert("DV", Regex::new(r"^\x1f\x07\x00[\x3f\xbf]").expect("Invalid regex for DV"));
    map.insert("DWF", Regex::new(r"^\(DWF V\d").expect("Invalid regex for DWF"));
    map.insert("DWG", Regex::new(r"^AC10\d{2}\x00").expect("Invalid regex for DWG"));
    map.insert("DXF", Regex::new(r"^\s*0\s+\x00?\s*SECTION\s+2\s+HEADER").expect("Invalid regex for DXF"));
    map.insert("EPS", Regex::new(r"^(%!PS|%!Ad|\xc5\xd0\xd3\xc6)").expect("Invalid regex for EPS"));
    map.insert("EXE", Regex::new(r"^(MZ|\xca\xfe\xba\xbe|\xfe\xed\xfa[\xce\xcf]|[\xce\xcf]\xfa\xed\xfe|Joy!peff|\x7fELF|#!\s*/\S*bin/|!<arch>\x0a)").expect("Invalid regex for EXE"));
    map.insert("EXIF", Regex::new(r"^(II\x2a\x00|MM\x00\x2a)").expect("Invalid regex for EXIF"));
    map.insert("EXR", Regex::new(r"^\x76\x2f\x31\x01").expect("Invalid regex for EXR"));
    map.insert("EXV", Regex::new(r"^\xff\x01Exiv2").expect("Invalid regex for EXV"));
    map.insert("FITS", Regex::new(r"^SIMPLE  = {20}T").expect("Invalid regex for FITS"));
    map.insert("FLAC", Regex::new(r"^(fLaC|ID3)").expect("Invalid regex for FLAC"));
    map.insert("FLIF", Regex::new(r"^FLIF[0-\x6f][0-2]").expect("Invalid regex for FLIF"));
    map.insert("FLIR", Regex::new(r"^[AF]FF\x00").expect("Invalid regex for FLIR"));
    map.insert("FLV", Regex::new(r"^FLV\x01").expect("Invalid regex for FLV"));
    map.insert("FPF", Regex::new(r"^FPF Public Image Format\x00").expect("Invalid regex for FPF"));
    map.insert("FPX", Regex::new(r"^\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1").expect("Invalid regex for FPX"));
    map.insert("Font", Regex::new(r"^((\x00\x01\x00\x00|OTTO|true|typ1)[\x00\x01]|ttcf\x00[\x01\x02]\x00\x00|\x00[\x01\x02]|(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])").expect("Invalid regex for Font"));
    map.insert("GIF", Regex::new(r"^GIF8[79]a").expect("Invalid regex for GIF"));
    map.insert("GZIP", Regex::new(r"^\x1f\x8b\x08").expect("Invalid regex for GZIP"));
    map.insert("HDR", Regex::new(r"^#\?(RADIANCE|RGBE)\x0a").expect("Invalid regex for HDR"));
    map.insert("HTML", Regex::new(r"^(\xef\xbb\xbf)?\s*(?i)<(!DOCTYPE\s+HTML|HTML|\?xml)").expect("Invalid regex for HTML"));
    map.insert("ICC", Regex::new(r"^.{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\x00{4}){2}").expect("Invalid regex for ICC"));
    map.insert("ICO", Regex::new(r"^\x00\x00[\x01\x02]\x00[^0]\x00").expect("Invalid regex for ICO"));
    map.insert("IND", Regex::new(r"^\x06\x06\xed\xf5\xd8\x1d\x46\xe5\xbd\x31\xef\xe7\xfe\x74\xb7\x1d").expect("Invalid regex for IND"));
    map.insert("ITC", Regex::new(r"^.{4}itch").expect("Invalid regex for ITC"));
    map.insert("JP2", Regex::new(r"^(\x00\x00\x00\x0cjP(  |\x1a\x1a)\x0d\x0a\x87\x0a|\xff\x4f\xff\x51\x00)").expect("Invalid regex for JP2"));
    map.insert("JPEG", Regex::new(r"^\xff\xd8\xff").expect("Invalid regex for JPEG"));
    map.insert("JSON", Regex::new(r#"^(\xef\xbb\xbf)?\s*(\[\s*)?\{\s*"[^"]*"\s*:"#).expect("Invalid regex for JSON"));
    map.insert("JUMBF", Regex::new(r"^.{4}jumb\x00.{3}jumd").expect("Invalid regex for JUMBF"));
    map.insert("JXL", Regex::new(r"^(\xff\x0a|\x00\x00\x00\x0cJXL \x0d\x0a......ftypjxl )").expect("Invalid regex for JXL"));
    map.insert("LFP", Regex::new(r"^\x89LFP\x0d\x0a\x1a\x0a").expect("Invalid regex for LFP"));
    map.insert("LIF", Regex::new(r"^\x70\x00{3}.{4}\x2a.{4}<\x00").expect("Invalid regex for LIF"));
    map.insert("LNK", Regex::new(r"^.{4}\x01\x14\x02\x00{5}\xc0\x00{6}\x46").expect("Invalid regex for LNK"));
    map.insert("LRI", Regex::new(r"^LELR \x00").expect("Invalid regex for LRI"));
    map.insert("M2TS", Regex::new(r"^(....)?\x47").expect("Invalid regex for M2TS"));
    map.insert("MIE", Regex::new(r"^~[\x10\x18]\x04.0MIE").expect("Invalid regex for MIE"));
    map.insert("MIFF", Regex::new(r"^id=ImageMagick").expect("Invalid regex for MIFF"));
    map.insert("MKV", Regex::new(r"^\x1a\x45\xdf\xa3").expect("Invalid regex for MKV"));
    map.insert("MOI", Regex::new(r"^V6").expect("Invalid regex for MOI"));
    map.insert("MOV", Regex::new(r"^.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)").expect("Invalid regex for MOV"));
    map.insert("MPC", Regex::new(r"^(MP\+|ID3)").expect("Invalid regex for MPC"));
    map.insert("MPEG", Regex::new(r"^\x00\x00\x01[\xb0-\xbf]").expect("Invalid regex for MPEG"));
    map.insert("MRC", Regex::new(r"^.{64}[\x01\x02\x03]\x00\x00\x00[\x01\x02\x03]\x00\x00\x00[\x01\x02\x03]\x00\x00\x00.{132}MAP[\x00 ](\x44\x44|\x44\x41|\x11\x11)\x00\x00").expect("Invalid regex for MRC"));
    map.insert("MRW", Regex::new(r"^\x00MR[MI]").expect("Invalid regex for MRW"));
    map.insert("MXF", Regex::new(r"^\x06\x0e\x2b\x34\x02\x05\x01\x01\x0d\x01\x02").expect("Invalid regex for MXF"));
    map.insert("MacOS", Regex::new(r"^\x00\x05\x16\x07\x00.\x00\x00Mac OS X        ").expect("Invalid regex for MacOS"));
    map.insert("NKA", Regex::new(r"^NIKONADJ").expect("Invalid regex for NKA"));
    map.insert("OGG", Regex::new(r"^(OggS|ID3)").expect("Invalid regex for OGG"));
    map.insert("ORF", Regex::new(r"^(II|MM)").expect("Invalid regex for ORF"));
    map.insert("PCAP", Regex::new(r"^\xa1\xb2(\xc3\xd4|\x3c\x4d)\x00.\x00.|(\xd4\xc3|\x4d\x3c)\xb2\xa1.\x00.\x00|\x0a\x0d\x0d\x0a.{4}(\x1a\x2b\x3c\x4d|\x4d\x3c\x2b\x1a)|GMBU\x00\x02").expect("Invalid regex for PCAP"));
    map.insert("PCX", Regex::new(r"^\x0a[\x00-\x05]\x01[\x01\x02\x04\x08].{64}[\x00-\x02]").expect("Invalid regex for PCX"));
    map.insert("PDB", Regex::new(r"^.{60}(\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)").expect("Invalid regex for PDB"));
    map.insert("PDF", Regex::new(r"^\s*%PDF-\d+\.\d+").expect("Invalid regex for PDF"));
    map.insert("PFM", Regex::new(r"^P[Ff]\x0a\d+ \d+\x0a[-+0-9.]+\x0a").expect("Invalid regex for PFM"));
    map.insert("PGF", Regex::new(r"^PGF").expect("Invalid regex for PGF"));
    map.insert("PHP", Regex::new(r"^<\?php\s").expect("Invalid regex for PHP"));
    map.insert("PICT", Regex::new(r"^(.{10}|.{522})(\x11\x01|\x00\x11)").expect("Invalid regex for PICT"));
    map.insert("PLIST", Regex::new(r"^(bplist0|\s*<|\xfe\xff\x00)").expect("Invalid regex for PLIST"));
    map.insert("PMP", Regex::new(r"^.{8}\x00{3}\x7c.{112}\xff\xd8\xff\xdb").expect("Invalid regex for PMP"));
    map.insert("PNG", Regex::new(r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n").expect("Invalid regex for PNG"));
    map.insert("PPM", Regex::new(r"^P[1-6]\s+").expect("Invalid regex for PPM"));
    map.insert("PS", Regex::new(r"^(%!PS|%!Ad|\xc5\xd0\xd3\xc6)").expect("Invalid regex for PS"));
    map.insert("PSD", Regex::new(r"^8BPS\x00[\x01\x02]").expect("Invalid regex for PSD"));
    map.insert("PSP", Regex::new(r"^Paint Shop Pro Image File\x0a\x1a\x00{5}").expect("Invalid regex for PSP"));
    map.insert("QTIF", Regex::new(r"^.{4}(idsc|idat|iicc)").expect("Invalid regex for QTIF"));
    map.insert("R3D", Regex::new(r"^\x00\x00..RED(1|2)").expect("Invalid regex for R3D"));
    map.insert("RAF", Regex::new(r"^FUJIFILM").expect("Invalid regex for RAF"));
    map.insert("RAR", Regex::new(r"^Rar!\x1a\x07\x01?\x00").expect("Invalid regex for RAR"));
    map.insert("RAW", Regex::new(r"^(.{25}ARECOYK|II|MM)").expect("Invalid regex for RAW"));
    map.insert("RIFF", Regex::new(r"^(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)").expect("Invalid regex for RIFF"));
    map.insert("RSRC", Regex::new(r"^(....)?\x00\x00\x01\x00").expect("Invalid regex for RSRC"));
    map.insert("RTF", Regex::new(r"^[\n\r]*\{[\n\r]*\\rtf").expect("Invalid regex for RTF"));
    map.insert("RWZ", Regex::new(r"^rawzor").expect("Invalid regex for RWZ"));
    map.insert("Real", Regex::new(r"^(\.RMF|\.ra\xfd|pnm://|rtsp://|http://)").expect("Invalid regex for Real"));
    map.insert("SWF", Regex::new(r"^[FC]WS[^\x00]").expect("Invalid regex for SWF"));
    map.insert("TAR", Regex::new(r"^.{257}ustar(  )?\x00").expect("Invalid regex for TAR"));
    map.insert("TIFF", Regex::new(r"^(II|MM)").expect("Invalid regex for TIFF"));
    map.insert("TXT", Regex::new(r"^(\xff\xfe|(\x00\x00)?\xfe\xff|(\xef\xbb\xbf)?[\x07-\x0d\x20-\x7e\x80-\xfe]*$)").expect("Invalid regex for TXT"));
    map.insert("Torrent", Regex::new(r"^d\d+:\w+").expect("Invalid regex for Torrent"));
    map.insert("VCard", Regex::new(r"^(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\r\n").expect("Invalid regex for VCard"));
    map.insert("VRD", Regex::new(r"^CANON OPTIONAL DATA\x00").expect("Invalid regex for VRD"));
    map.insert("WMF", Regex::new(r"^(\xd7\xcd\xc6\x9a\x00\x00|\x01\x00\x09\x00\x00\x03)").expect("Invalid regex for WMF"));
    map.insert("WPG", Regex::new(r"^\xff\x57\x50\x43").expect("Invalid regex for WPG"));
    map.insert("WTV", Regex::new(r"^\xb7\xd8\x00\x20\x37\x49\xda\x11\xa6\x4e\x00\x07\xe9\x5e\xad\x8d").expect("Invalid regex for WTV"));
    map.insert("X3F", Regex::new(r"^FOVb").expect("Invalid regex for X3F"));
    map.insert("XCF", Regex::new(r"^gimp xcf ").expect("Invalid regex for XCF"));
    map.insert("XISF", Regex::new(r"^XISF0100").expect("Invalid regex for XISF"));
    map.insert("XMP", Regex::new(r"^\x00{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\x00{0,3}\s*<").expect("Invalid regex for XMP"));
    map.insert("ZIP", Regex::new(r"^PK\x03\x04").expect("Invalid regex for ZIP"));
    
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
