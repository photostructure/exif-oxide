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

Below is a recipe for translating your Perl magic‐number regexes into Rust using the [regex](https://docs.rs/regex) crate. In summary:

* **Use the Rust `regex` crate**, which supports a Perl-like syntax (including `\xNN` escapes) but *does not* support look-around or back-references. ([Docs.rs][1], [dtantsur.github.io][2])
* **Embed patterns as raw string literals** (`r"…"` or `r#"…"#`) to minimize backslash‐escaping pain. ([Docs.rs][1], [Wikipedia][3])
* **Prepare a map of `&str` → `Regex`** (e.g. via `once_cell`/`lazy_static` or at runtime) and compile each pattern with `Regex::new(...)`. ([Docs.rs][1])
* **Adjust any Perl-only constructs** (e.g. change `\0{6}` to Rust’s supported `{6}`, remove lookarounds) or split complex tests into multiple patterns as needed.

---

## 1. Add the `regex` crate

In your `Cargo.toml`:

```toml
[dependencies]
regex = "1"
once_cell = "1"      # or lazy_static = "1.4"
```

([Docs.rs][1])

Then in code:

```rust
use regex::Regex;
use once_cell::sync::Lazy;
// or: use lazy_static::lazy_static;
```

---

## 2. Represent each pattern as a raw string

Rust raw strings (`r"…"` or with `#` delimiters) prevent Rust from interpreting backslashes, so you can write:

```rust
// Perl: '.{4}\x57\x90\x75\x36'
// Rust raw string:
r".{4}\x57\x90\x75\x36"
```

Advantages of raw strings:

* No need to double-escape backslashes (`\\x57` vs. `\x57`). ([Docs.rs][1])
* You can include quotes by bumping the delimiter level: `r#"...""#`. ([Wikipedia][3])
* Raw strings don’t process any `\` escapes at all, so what you type is what the regex engine sees. ([Stack Overflow][4])

---

## 3. Compile into a lookup map

Here’s a skeleton showing how to build a static map of magic‐number tests:

```rust
use regex::Regex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static MAGIC_REGEX: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut m = HashMap::new();
    let patterns = [
        ("AA",   r".{4}\x57\x90\x75\x36"),
        ("AAC",  r"\xff[\xf0\xf1]"),
        ("AIFF", r"(FORM....AIF[FC]|AT&TFORM)"),
        ("ALIAS",r"book\0\0\0\0mark\0\0\0\0"),
        ("APE",  r"(MAC |APETAGEX|ID3)"),
        // … add the rest …
    ];
    for &(key, pat) in &patterns {
        // Regex::new uses the default (UTF-8, no multi-line by default)
        m.insert(key, Regex::new(pat).expect("invalid regex"));
    }
    m
});
```

* We use [`once_cell::sync::Lazy`](https://docs.rs/once_cell) for on-first-use initialization.
* You could also use the `lazy_static!` macro the same way.
* Each call to `Regex::new(pat)` compiles the pattern into a thread-safe `Regex`. ([Docs.rs][1])

---

## 4. Running a magic-number test

To test the first N bytes of a file:

```rust
fn match_magic(name: &str, data: &[u8]) -> bool {
    if let Some(re) = MAGIC_REGEX.get(name) {
        // Only look at the prefix; convert bytes to str lossily if needed:
        let prefix = std::str::from_utf8(&data[..std::cmp::min(data.len(), 64)])
            .unwrap_or_default();
        re.is_match(prefix)
    } else {
        false
    }
}
```

* This treats the file header as UTF-8; if you need true binary matching, see [`Regex::new_bytes`](https://docs.rs/regex/latest/regex/struct.Regex.html) in `regex-automata`. ([dtantsur.github.io][2])

---

## 5. Watch out for unsupported constructs

Rust’s `regex` crate **does not** implement:

* Look-around assertions (`(?<=…)`, `(?!)`, etc.)
* Back-references (`\1`, `\k<name>`)
* Some exotic Perl extensions (e.g. code‐execution, conditionals)

Any Perl patterns using these will need reworking (e.g. pre-filter in Rust code, or split into multiple regexes). ([Docs.rs][1])

---

### Further reading

* **Regex crate overview & syntax** – docs.rs ([Docs.rs][1])
* **Raw string literals** – Rust reference, examples ([Docs.rs][1], [Wikipedia][3])
* **Binary regex support** – `regex-automata` crate for byte‐oriented matching ([dtantsur.github.io][2])

With these steps, you can systematically convert each of your Perl magic-number tests into efficient, compile-checked Rust regexes.

[1]: https://docs.rs/regex/latest/regex/?utm_source=chatgpt.com "Crate regex - Rust - Docs.rs"
[2]: https://dtantsur.github.io/rust-openstack/regex/index.html?utm_source=chatgpt.com "Crate regex - Rust"
[3]: https://en.wikipedia.org/wiki/Leaning_toothpick_syndrome?utm_source=chatgpt.com "Leaning toothpick syndrome"
[4]: https://stackoverflow.com/questions/54912970/how-to-escape-escaped-regex-characters-when-using-rusts-regex-crate?utm_source=chatgpt.com "How to escape escaped regex characters when using Rust's regex ..."
