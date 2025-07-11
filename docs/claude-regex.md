how do I encode these perl regexes in rust?

```pl
# quick "magic number" file test used to avoid loading module unnecessarily:
# - regular expression evaluated on first $testLen bytes of file
# - must match beginning at first byte in file
# - this test must not be more stringent than module logic
%magicNumber = (
    AA   => '.{4}\x57\x90\x75\x36',
    AAC  => '\xff[\xf0\xf1]',
    AIFF => '(FORM....AIF[FC]|AT&TFORM)',
    ALIAS=> "book\0\0\0\0mark\0\0\0\0",
    APE  => '(MAC |APETAGEX|ID3)',
    ASF  => '\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c',
    AVC  => '\+A\+V\+C\+',
    Torrent => 'd\d+:\w+',
    BMP  => 'BM',
    BPG  => "BPG\xfb",
    BTF  => '(II\x2b\0|MM\0\x2b)',
    BZ2  => 'BZh[1-9]\x31\x41\x59\x26\x53\x59',
    CHM  => 'ITSF.{20}\x10\xfd\x01\x7c\xaa\x7b\xd0\x11\x9e\x0c\0\xa0\xc9\x22\xe6\xec',
    CRW  => '(II|MM).{4}HEAP(CCDR|JPGM)',
    CZI  => 'ZISRAWFILE\0{6}',
    DCX  => '\xb1\x68\xde\x3a',
    DEX  => "dex\n035\0",
    DICOM=> '(.{128}DICM|\0[\x02\x04\x06\x08]\0[\0-\x20]|[\x02\x04\x06\x08]\0[\0-\x20]\0)',
    DOCX => 'PK\x03\x04',
    DPX  => '(SDPX|XPDS)',
    DR4  => 'IIII[\x04|\x05]\0\x04\0',
    DSS  => '(\x02dss|\x03ds2)',
    DV   => '\x1f\x07\0[\x3f\xbf]', # (not tested if extension recognized)
    DWF  => '\(DWF V\d',
    DWG  => 'AC10\d{2}\0',
    DXF  => '\s0\s+\0?\sSECTION\s+2\s+HEADER',
    EPS  => '(%!PS|%!Ad|\xc5\xd0\xd3\xc6)',
    EXE  => '(MZ|\xca\xfe\xba\xbe|\xfe\xed\xfa[\xce\xcf]|[\xce\xcf]\xfa\xed\xfe|Joy!peff|\x7fELF|#!\s/\Sbin/|!<arch>\x0a)',
    EXIF => '(II\x2a\0|MM\0\x2a)',
    EXR  => '\x76\x2f\x31\x01',
    EXV  => '\xff\x01Exiv2',
    FITS => 'SIMPLE  = {20}T',
    FLAC => '(fLaC|ID3)',
    FLIF => 'FLIF[0-\x6f][0-2]',
    FLIR => '[AF]FF\0',
    FLV  => 'FLV\x01',
    Font => '((\0\x01\0\0|OTTO|true|typ1)[\0\x01]|ttcf\0[\x01\x02]\0\0|\0[\x01\x02]|' .
            '(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])',
    FPF  => 'FPF Public Image Format\0',
    FPX  => '\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1',
    GIF  => 'GIF8[79]a',
    GZIP => '\x1f\x8b\x08',
    HDR  => '#\?(RADIANCE|RGBE)\x0a',
    HTML => '(\xef\xbb\xbf)?\s(?i)<(!DOCTYPE\s+HTML|HTML|\?xml)', # (case insensitive)
    ICC  => '.{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\0{4}){2}',
    ICO  => '\0\0[\x01\x02]\0[^0]\0', # (reasonably assume that the file contains less than 256 images)
    IND  => '\x06\x06\xed\xf5\xd8\x1d\x46\xe5\xbd\x31\xef\xe7\xfe\x74\xb7\x1d',
  # ISO  =>  signature is at byte 32768
    ITC  => '.{4}itch',
    JP2  => '(\0\0\0\x0cjP(  |\x1a\x1a)\x0d\x0a\x87\x0a|\xff\x4f\xff\x51\0)',
    JPEG => '\xff\xd8\xff',
    JSON => '(\xef\xbb\xbf)?\s(\[\s)?\{\s"[^"]"\s:',
    JUMBF=> '.{4}jumb\0.{3}jumd',
    JXL  => '(\xff\x0a|\0\0\0\x0cJXL \x0d\x0a......ftypjxl )',
    LFP  => '\x89LFP\x0d\x0a\x1a\x0a',
    LIF  => '\x70\0{3}.{4}\x2a.{4}<\0',
    LNK  => '.{4}\x01\x14\x02\0{5}\xc0\0{6}\x46',
    LRI  => 'LELR \0',
    M2TS => '(....)?\x47',
    MacOS=> '\0\x05\x16\x07\0.\0\0Mac OS X        ',
    MIE  => '~[\x10\x18]\x04.0MIE',
    MIFF => 'id=ImageMagick',
    MKV  => '\x1a\x45\xdf\xa3',
    MOV  => '.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)', # (duplicated in WriteQuickTime.pl !!)
  # MP3  =>  difficult to rule out
    MPC  => '(MP\+|ID3)',
    MOI  => 'V6',
    MPEG => '\0\0\x01[\xb0-\xbf]',
    MRC  => '.{64}[\x01\x02\x03]\0\0\0[\x01\x02\x03]\0\0\0[\x01\x02\x03]\0\0\0.{132}MAP[\0 ](\x44\x44|\x44\x41|\x11\x11)\0\0',
    MRW  => '\0MR[MI]',
    MXF  => '\x06\x0e\x2b\x34\x02\x05\x01\x01\x0d\x01\x02', # (not tested if extension recognized)
    NKA  => 'NIKONADJ',
    OGG  => '(OggS|ID3)',
    ORF  => '(II|MM)',
    PCAP => '\xa1\xb2(\xc3\xd4|\x3c\x4d)\0.\0.|(\xd4\xc3|\x4d\x3c)\xb2\xa1.\0.\0|\x0a\x0d\x0d\x0a.{4}(\x1a\x2b\x3c\x4d|\x4d\x3c\x2b\x1a)|GMBU\0\x02',
  # PCD  =>  signature is at byte 2048
    PCX  => '\x0a[\0-\x05]\x01[\x01\x02\x04\x08].{64}[\0-\x02]',
    PDB  => '.{60}(\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)',
    PDF  => '\s%PDF-\d+\.\d+',
    PFM  => 'P[Ff]\x0a\d+ \d+\x0a[-+0-9.]+\x0a',
    PGF  => 'PGF',
    PHP  => '<\?php\s',
    PICT => '(.{10}|.{522})(\x11\x01|\x00\x11)',
    PLIST=> '(bplist0|\s<|\xfe\xff\x00)',
    PMP  => '.{8}\0{3}\x7c.{112}\xff\xd8\xff\xdb',
    PNG  => '(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n',
    PPM  => 'P[1-6]\s+',
    PS   => '(%!PS|%!Ad|\xc5\xd0\xd3\xc6)',
    PSD  => '8BPS\0[\x01\x02]',
    PSP  => 'Paint Shop Pro Image File\x0a\x1a\0{5}',
    QTIF => '.{4}(idsc|idat|iicc)',
    R3D  => '\0\0..RED(1|2)',
    RAF  => 'FUJIFILM',
    RAR  => 'Rar!\x1a\x07\x01?\0',
    RAW  => '(.{25}ARECOYK|II|MM)',
    Real => '(\.RMF|\.ra\xfd|pnm://|rtsp://|http://)',
    RIFF => '(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)', # RIFF plus other variants
    RSRC => '(....)?\0\0\x01\0',
    RTF  => '[\n\r]\\{[\n\r]\\\\rtf',
    RWZ  => 'rawzor',
    SWF  => '[FC]WS[^\0]',
    TAR  => '.{257}ustar(  )?\0', # (this doesn't catch old-style tar files)
    TXT  => '(\xff\xfe|(\0\0)?\xfe\xff|(\xef\xbb\xbf)?[\x07-\x0d\x20-\x7e\x80-\xfe]$)',
    TIFF => '(II|MM)', # don't test magic number (some raw formats are different)
    VCard=> '(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\r\n',
    VRD  => 'CANON OPTIONAL DATA\0',
    WMF  => '(\xd7\xcd\xc6\x9a\0\0|\x01\0\x09\0\0\x03)',
    WPG  => '\xff\x57\x50\x43',
    WTV  => '\xb7\xd8\x00\x20\x37\x49\xda\x11\xa6\x4e\x00\x07\xe9\x5e\xad\x8d',
    X3F  => 'FOVb',
    XCF  => 'gimp xcf ',
    XISF => 'XISF0100',
    XMP  => '\0{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\0{0,3}\s<',
    ZIP  => 'PK\x03\x04',
);
```

Here's how you can encode these Perl regexes in Rust using the `regex` crate:Here are the key changes I made when converting from Perl to Rust:

## Major Changes:

1. **Anchoring**: Added `^` at the beginning of each pattern since the Perl comment mentions "must match beginning at first byte in file"

2. **Null bytes**: Changed `\0` to `\x00` (both work in Rust, but `\x00` is more explicit)

3. **Repetition syntax**: 
   - `\0{6}` became `\x00{6}` 
   - `\0{3}` became `\x00{3}`

4. **Character classes**: Fixed ranges like `[\0-\x20]` to `[\x00-\x20]`

5. **Case insensitive**: Used `(?i)` flag for HTML and VCard patterns

6. **Regex compilation**: Used `regex::bytes::Regex` for binary data matching

## To use this code:

1. Add to your `Cargo.toml`:
```toml
[dependencies]
regex = "1.0"
```

2. The `MagicNumberDetector` provides two main methods:
   - `detect_file_type()`: Returns all matching file types
   - `is_file_type()`: Tests against a specific file type

3. Use with file data:
```rust
let detector = MagicNumberDetector::new()?;
let file_data = std::fs::read("somefile.jpg")?;
let matches = detector.detect_file_type(&file_data[..1024.min(file_data.len())]);
```

The main advantage of this Rust version is type safety, memory safety, and the ability to compile all regexes once at initialization rather than on each use.

```rs
use regex::bytes::Regex;
use std::collections::HashMap;

pub struct MagicNumberDetector {
    patterns: HashMap<&'static str, Regex>,
}

impl MagicNumberDetector {
    pub fn new() -> Result<Self, regex::Error> {
        let mut patterns = HashMap::new();
        
        // Convert each Perl regex to Rust regex
        patterns.insert("AA", Regex::new(r"^.{4}\x57\x90\x75\x36")?);
        patterns.insert("AAC", Regex::new(r"^\xff[\xf0\xf1]")?);
        patterns.insert("AIFF", Regex::new(r"^(FORM....AIF[FC]|AT&TFORM)")?);
        patterns.insert("ALIAS", Regex::new(r"^book\x00\x00\x00\x00mark\x00\x00\x00\x00")?);
        patterns.insert("APE", Regex::new(r"^(MAC |APETAGEX|ID3)")?);
        patterns.insert("ASF", Regex::new(r"^\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c")?);
        patterns.insert("AVC", Regex::new(r"^\+A\+V\+C\+")?);
        patterns.insert("Torrent", Regex::new(r"^d\d+:\w+")?);
        patterns.insert("BMP", Regex::new(r"^BM")?);
        patterns.insert("BPG", Regex::new(r"^BPG\xfb")?);
        patterns.insert("BTF", Regex::new(r"^(II\x2b\x00|MM\x00\x2b)")?);
        patterns.insert("BZ2", Regex::new(r"^BZh[1-9]\x31\x41\x59\x26\x53\x59")?);
        patterns.insert("CHM", Regex::new(r"^ITSF.{20}\x10\xfd\x01\x7c\xaa\x7b\xd0\x11\x9e\x0c\x00\xa0\xc9\x22\xe6\xec")?);
        patterns.insert("CRW", Regex::new(r"^(II|MM).{4}HEAP(CCDR|JPGM)")?);
        patterns.insert("CZI", Regex::new(r"^ZISRAWFILE\x00{6}")?);
        patterns.insert("DCX", Regex::new(r"^\xb1\x68\xde\x3a")?);
        patterns.insert("DEX", Regex::new(r"^dex\n035\x00")?);
        patterns.insert("DICOM", Regex::new(r"^(.{128}DICM|\x00[\x02\x04\x06\x08]\x00[\x00-\x20]|[\x02\x04\x06\x08]\x00[\x00-\x20]\x00)")?);
        patterns.insert("DOCX", Regex::new(r"^PK\x03\x04")?);
        patterns.insert("DPX", Regex::new(r"^(SDPX|XPDS)")?);
        patterns.insert("DR4", Regex::new(r"^IIII[\x04\x05]\x00\x04\x00")?);
        patterns.insert("DSS", Regex::new(r"^(\x02dss|\x03ds2)")?);
        patterns.insert("DV", Regex::new(r"^\x1f\x07\x00[\x3f\xbf]")?);
        patterns.insert("DWF", Regex::new(r"^\(DWF V\d")?);
        patterns.insert("DWG", Regex::new(r"^AC10\d{2}\x00")?);
        patterns.insert("DXF", Regex::new(r"^\s*0\s+\x00?\s*SECTION\s+2\s+HEADER")?);
        patterns.insert("EPS", Regex::new(r"^(%!PS|%!Ad|\xc5\xd0\xd3\xc6)")?);
        patterns.insert("EXE", Regex::new(r"^(MZ|\xca\xfe\xba\xbe|\xfe\xed\xfa[\xce\xcf]|[\xce\xcf]\xfa\xed\xfe|Joy!peff|\x7fELF|#!\s*/\S*bin/|!<arch>\x0a)")?);
        patterns.insert("EXIF", Regex::new(r"^(II\x2a\x00|MM\x00\x2a)")?);
        patterns.insert("EXR", Regex::new(r"^\x76\x2f\x31\x01")?);
        patterns.insert("EXV", Regex::new(r"^\xff\x01Exiv2")?);
        patterns.insert("FITS", Regex::new(r"^SIMPLE  = {20}T")?);
        patterns.insert("FLAC", Regex::new(r"^(fLaC|ID3)")?);
        patterns.insert("FLIF", Regex::new(r"^FLIF[\x00-\x6f][\x00-\x02]")?);
        patterns.insert("FLIR", Regex::new(r"^[AF]FF\x00")?);
        patterns.insert("FLV", Regex::new(r"^FLV\x01")?);
        patterns.insert("Font", Regex::new(r"^((\x00\x01\x00\x00|OTTO|true|typ1)[\x00\x01]|ttcf\x00[\x01\x02]\x00\x00|\x00[\x01\x02]|(.{6})?%!(PS-(AdobeFont-|Bitstream )|FontType1-)|Start(Comp|Master)?FontMetrics|wOF[F2])")?);
        patterns.insert("FPF", Regex::new(r"^FPF Public Image Format\x00")?);
        patterns.insert("FPX", Regex::new(r"^\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")?);
        patterns.insert("GIF", Regex::new(r"^GIF8[79]a")?);
        patterns.insert("GZIP", Regex::new(r"^\x1f\x8b\x08")?);
        patterns.insert("HDR", Regex::new(r"^#\?(RADIANCE|RGBE)\x0a")?);
        patterns.insert("HTML", Regex::new(r"^(?i)(\xef\xbb\xbf)?\s*(<(!DOCTYPE\s+HTML|HTML|\?xml))")?);
        patterns.insert("ICC", Regex::new(r"^.{12}(scnr|mntr|prtr|link|spac|abst|nmcl|nkpf|cenc|mid |mlnk|mvis)(XYZ |Lab |Luv |YCbr|Yxy |RGB |GRAY|HSV |HLS |CMYK|CMY |[2-9A-F]CLR|nc..|\x00{4}){2}")?);
        patterns.insert("ICO", Regex::new(r"^\x00\x00[\x01\x02]\x00[^\x00]\x00")?);
        patterns.insert("IND", Regex::new(r"^\x06\x06\xed\xf5\xd8\x1d\x46\xe5\xbd\x31\xef\xe7\xfe\x74\xb7\x1d")?);
        patterns.insert("ITC", Regex::new(r"^.{4}itch")?);
        patterns.insert("JP2", Regex::new(r"^(\x00\x00\x00\x0cjP(  |\x1a\x1a)\x0d\x0a\x87\x0a|\xff\x4f\xff\x51\x00)")?);
        patterns.insert("JPEG", Regex::new(r"^\xff\xd8\xff")?);
        patterns.insert("JSON", Regex::new(r"^(\xef\xbb\xbf)?\s*(\[\s*)?\{\s*\"[^\"]*\"\s*:")?);
        patterns.insert("JUMBF", Regex::new(r"^.{4}jumb\x00.{3}jumd")?);
        patterns.insert("JXL", Regex::new(r"^(\xff\x0a|\x00\x00\x00\x0cJXL \x0d\x0a......ftypjxl )")?);
        patterns.insert("LFP", Regex::new(r"^\x89LFP\x0d\x0a\x1a\x0a")?);
        patterns.insert("LIF", Regex::new(r"^\x70\x00{3}.{4}\x2a.{4}<\x00")?);
        patterns.insert("LNK", Regex::new(r"^.{4}\x01\x14\x02\x00{5}\xc0\x00{6}\x46")?);
        patterns.insert("LRI", Regex::new(r"^LELR \x00")?);
        patterns.insert("M2TS", Regex::new(r"^(....)?\x47")?);
        patterns.insert("MacOS", Regex::new(r"^\x00\x05\x16\x07\x00.\x00\x00Mac OS X        ")?);
        patterns.insert("MIE", Regex::new(r"^~[\x10\x18]\x04.0MIE")?);
        patterns.insert("MIFF", Regex::new(r"^id=ImageMagick")?);
        patterns.insert("MKV", Regex::new(r"^\x1a\x45\xdf\xa3")?);
        patterns.insert("MOV", Regex::new(r"^.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)")?);
        patterns.insert("MPC", Regex::new(r"^(MP\+|ID3)")?);
        patterns.insert("MOI", Regex::new(r"^V6")?);
        patterns.insert("MPEG", Regex::new(r"^\x00\x00\x01[\xb0-\xbf]")?);
        patterns.insert("MRC", Regex::new(r"^.{64}[\x01\x02\x03]\x00\x00\x00[\x01\x02\x03]\x00\x00\x00[\x01\x02\x03]\x00\x00\x00.{132}MAP[\x00 ](\x44\x44|\x44\x41|\x11\x11)\x00\x00")?);
        patterns.insert("MRW", Regex::new(r"^\x00MR[MI]")?);
        patterns.insert("MXF", Regex::new(r"^\x06\x0e\x2b\x34\x02\x05\x01\x01\x0d\x01\x02")?);
        patterns.insert("NKA", Regex::new(r"^NIKONADJ")?);
        patterns.insert("OGG", Regex::new(r"^(OggS|ID3)")?);
        patterns.insert("ORF", Regex::new(r"^(II|MM)")?);
        patterns.insert("PCAP", Regex::new(r"^\xa1\xb2(\xc3\xd4|\x3c\x4d)\x00.\x00.|\xd4\xc3|\x4d\x3c)\xb2\xa1.\x00.\x00|\x0a\x0d\x0d\x0a.{4}(\x1a\x2b\x3c\x4d|\x4d\x3c\x2b\x1a)|GMBU\x00\x02")?);
        patterns.insert("PCX", Regex::new(r"^\x0a[\x00-\x05]\x01[\x01\x02\x04\x08].{64}[\x00-\x02]")?);
        patterns.insert("PDB", Regex::new(r"^.{60}(\.pdfADBE|TEXtREAd|BVokBDIC|DB99DBOS|PNRdPPrs|DataPPrs|vIMGView|PmDBPmDB|InfoINDB|ToGoToGo|SDocSilX|JbDbJBas|JfDbJFil|DATALSdb|Mdb1Mdb1|BOOKMOBI|DataPlkr|DataSprd|SM01SMem|TEXtTlDc|InfoTlIf|DataTlMl|DataTlPt|dataTDBP|TdatTide|ToRaTRPW|zTXTGPlm|BDOCWrdS)")?);
        patterns.insert("PDF", Regex::new(r"^\s*%PDF-\d+\.\d+")?);
        patterns.insert("PFM", Regex::new(r"^P[Ff]\x0a\d+ \d+\x0a[-+0-9.]+\x0a")?);
        patterns.insert("PGF", Regex::new(r"^PGF")?);
        patterns.insert("PHP", Regex::new(r"^<\?php\s")?);
        patterns.insert("PICT", Regex::new(r"^(.{10}|.{522})(\x11\x01|\x00\x11)")?);
        patterns.insert("PLIST", Regex::new(r"^(bplist0|\s*<|\xfe\xff\x00)")?);
        patterns.insert("PMP", Regex::new(r"^.{8}\x00{3}\x7c.{112}\xff\xd8\xff\xdb")?);
        patterns.insert("PNG", Regex::new(r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n")?);
        patterns.insert("PPM", Regex::new(r"^P[1-6]\s+")?);
        patterns.insert("PS", Regex::new(r"^(%!PS|%!Ad|\xc5\xd0\xd3\xc6)")?);
        patterns.insert("PSD", Regex::new(r"^8BPS\x00[\x01\x02]")?);
        patterns.insert("PSP", Regex::new(r"^Paint Shop Pro Image File\x0a\x1a\x00{5}")?);
        patterns.insert("QTIF", Regex::new(r"^.{4}(idsc|idat|iicc)")?);
        patterns.insert("R3D", Regex::new(r"^\x00\x00..RED[12]")?);
        patterns.insert("RAF", Regex::new(r"^FUJIFILM")?);
        patterns.insert("RAR", Regex::new(r"^Rar!\x1a\x07\x01?\x00")?);
        patterns.insert("RAW", Regex::new(r"^(.{25}ARECOYK|II|MM)")?);
        patterns.insert("Real", Regex::new(r"^(\.RMF|\.ra\xfd|pnm://|rtsp://|http://)")?);
        patterns.insert("RIFF", Regex::new(r"^(RIFF|LA0[234]|OFR |LPAC|wvpk|RF64)")?);
        patterns.insert("RSRC", Regex::new(r"^(....)?\x00\x00\x01\x00")?);
        patterns.insert("RTF", Regex::new(r"^[\n\r]*\\{[\n\r]*\\\\rtf")?);
        patterns.insert("RWZ", Regex::new(r"^rawzor")?);
        patterns.insert("SWF", Regex::new(r"^[FC]WS[^\x00]")?);
        patterns.insert("TAR", Regex::new(r"^.{257}ustar(  )?\x00")?);
        patterns.insert("TXT", Regex::new(r"^(\xff\xfe|(\x00\x00)?\xfe\xff|(\xef\xbb\xbf)?[\x07-\x0d\x20-\x7e\x80-\xfe]*$)")?);
        patterns.insert("TIFF", Regex::new(r"^(II|MM)")?);
        patterns.insert("VCard", Regex::new(r"^(?i)BEGIN:(VCARD|VCALENDAR|VNOTE)\r\n")?);
        patterns.insert("VRD", Regex::new(r"^CANON OPTIONAL DATA\x00")?);
        patterns.insert("WMF", Regex::new(r"^(\xd7\xcd\xc6\x9a\x00\x00|\x01\x00\x09\x00\x00\x03)")?);
        patterns.insert("WPG", Regex::new(r"^\xff\x57\x50\x43")?);
        patterns.insert("WTV", Regex::new(r"^\xb7\xd8\x00\x20\x37\x49\xda\x11\xa6\x4e\x00\x07\xe9\x5e\xad\x8d")?);
        patterns.insert("X3F", Regex::new(r"^FOVb")?);
        patterns.insert("XCF", Regex::new(r"^gimp xcf ")?);
        patterns.insert("XISF", Regex::new(r"^XISF0100")?);
        patterns.insert("XMP", Regex::new(r"^\x00{0,3}(\xfe\xff|\xff\xfe|\xef\xbb\xbf)?\x00{0,3}\s*<")?);
        patterns.insert("ZIP", Regex::new(r"^PK\x03\x04")?);

        Ok(MagicNumberDetector { patterns })
    }

    /// Test a file's magic number against all known patterns
    pub fn detect_file_type(&self, data: &[u8]) -> Vec<&'static str> {
        let mut matches = Vec::new();
        
        for (file_type, pattern) in &self.patterns {
            if pattern.is_match(data) {
                matches.push(*file_type);
            }
        }
        
        matches
    }

    /// Test a file's magic number against a specific pattern
    pub fn is_file_type(&self, data: &[u8], file_type: &str) -> bool {
        if let Some(pattern) = self.patterns.get(file_type) {
            pattern.is_match(data)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_numbers() {
        let detector = MagicNumberDetector::new().unwrap();
        
        // Test JPEG
        let jpeg_data = b"\xff\xd8\xff\xe0\x00\x10JFIF";
        assert!(detector.is_file_type(jpeg_data, "JPEG"));
        
        // Test PNG
        let png_data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert!(detector.is_file_type(png_data, "PNG"));
        
        // Test PDF
        let pdf_data = b"%PDF-1.4\n";
        assert!(detector.is_file_type(pdf_data, "PDF"));
        
        // Test ZIP
        let zip_data = b"PK\x03\x04\x14\x00\x00\x00";
        assert!(detector.is_file_type(zip_data, "ZIP"));
        
        // Test GIF
        let gif_data = b"GIF89a\x01\x00\x01\x00";
        assert!(detector.is_file_type(gif_data, "GIF"));
    }

    #[test]
    fn test_detect_multiple_types() {
        let detector = MagicNumberDetector::new().unwrap();
        
        // Some patterns might match multiple types
        let data = b"MMII\x2a\x00\x00\x00";
        let matches = detector.detect_file_type(data);
        
        // Should match both TIFF and potentially others
        assert!(!matches.is_empty());
    }
}

// Example usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = MagicNumberDetector::new()?;
    
    // Example: Read first 1024 bytes of a file and detect its type
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open("example.jpg")?;
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    
    let file_types = detector.detect_file_type(&buffer);
    
    if file_types.is_empty() {
        println!("Unknown file type");
    } else {
        println!("Detected file types: {:?}", file_types);
    }
    
    Ok(())
}
```