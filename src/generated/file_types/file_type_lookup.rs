//! File type lookup tables generated from ExifTool's fileTypeLookup hash
//!
//! Total lookups: 343

use std::collections::HashMap;
use std::sync::LazyLock;

/// Extension aliases - maps extensions to their canonical forms
static EXTENSION_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("3GP2", "3G2");
    map.insert("3GPP", "3GP");
    map.insert("AIF", "AIFF");
    map.insert("AIT", "AI");
    map.insert("AZW", "MOBI");
    map.insert("AZW3", "MOBI");
    map.insert("CAP", "PCAP");
    map.insert("DC3", "DICM");
    map.insert("DCM", "DICM");
    map.insert("DIC", "DICM");
    map.insert("DJV", "DJVU");
    map.insert("EPS2", "EPS");
    map.insert("EPS3", "EPS");
    map.insert("EPSF", "EPS");
    map.insert("FIT", "FITS");
    map.insert("GZ", "GZIP");
    map.insert("HIF", "HEIF");
    map.insert("HTM", "HTML");
    map.insert("ICAL", "ICS");
    map.insert("ICM", "ICC");
    map.insert("J2K", "J2C");
    map.insert("JPC", "J2C");
    map.insert("JPE", "JPEG");
    map.insert("JPF", "JP2");
    map.insert("JPG", "JPEG");
    map.insert("LFR", "LFP");
    map.insert("M2T", "M2TS");
    map.insert("MIF", "MIFF");
    map.insert("MPG", "MPEG");
    map.insert("MTS", "M2TS");
    map.insert("NEWER", "COS");
    map.insert("ORI", "ORF");
    map.insert("PCT", "PICT");
    map.insert("PHP3", "PHP");
    map.insert("PHP4", "PHP");
    map.insert("PHP5", "PHP");
    map.insert("PHPS", "PHP");
    map.insert("PHTML", "PHP");
    map.insert("PS2", "PS");
    map.insert("PS3", "PS");
    map.insert("PSPFRAME", "PSP");
    map.insert("PSPIMAGE", "PSP");
    map.insert("PSPSHAPE", "PSP");
    map.insert("PSPTUBE", "PSP");
    map.insert("QIF", "QTIF");
    map.insert("QT", "MOV");
    map.insert("QTI", "QTIF");
    map.insert("RIF", "RIFF");
    map.insert("TIF", "TIFF");
    map.insert("TS", "M2TS");
    map.insert("TUB", "PSP");
    map.insert("VCF", "VCARD");
    map
});

/// File type formats - maps file types to their format descriptions
static FILE_TYPE_FORMATS: LazyLock<HashMap<&'static str, (Vec<&'static str>, &'static str)>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("360", (vec!["MOV"], "GoPro 360 video"));
        map.insert("3FR", (vec!["TIFF"], "Hasselblad RAW format"));
        map.insert(
            "3G2",
            (vec!["MOV"], "3rd Gen. Partnership Project 2 audio/video"),
        );
        map.insert(
            "3GP",
            (vec!["MOV"], "3rd Gen. Partnership Project audio/video"),
        );
        map.insert("7Z", (vec!["7Z"], "7z archive"));
        map.insert("A", (vec!["EXE"], "Static library"));
        map.insert("AA", (vec!["AA"], "Audible Audiobook"));
        map.insert("AAC", (vec!["AAC"], "Advanced Audio Coding"));
        map.insert("AAE", (vec!["PLIST"], "Apple edit information"));
        map.insert("AAX", (vec!["MOV"], "Audible Enhanced Audiobook"));
        map.insert("ACFM", (vec!["Font"], "Adobe Composite Font Metrics"));
        map.insert(
            "ACR",
            (vec!["DICOM"], "American College of Radiology ACR-NEMA"),
        );
        map.insert("AFM", (vec!["Font"], "Adobe Font Metrics"));
        map.insert("AI", (vec!["PDF", "PS"], "Adobe Illustrator"));
        map.insert(
            "AIFC",
            (vec!["AIFF"], "Audio Interchange File Format Compressed"),
        );
        map.insert("AIFF", (vec!["AIFF"], "Audio Interchange File Format"));
        map.insert("ALIAS", (vec!["ALIAS"], "MacOS file alias"));
        map.insert("AMFM", (vec!["Font"], "Adobe Multiple Master Font Metrics"));
        map.insert("APE", (vec!["APE"], "Monkey's Audio format"));
        map.insert("APNG", (vec!["PNG"], "Animated Portable Network Graphics"));
        map.insert("ARQ", (vec!["TIFF"], "Sony Alpha Pixel-Shift RAW format"));
        map.insert("ARW", (vec!["TIFF"], "Sony Alpha RAW format"));
        map.insert("ASF", (vec!["ASF"], "Microsoft Advanced Systems Format"));
        map.insert("AVC", (vec!["AVC"], "Advanced Video Connection"));
        map.insert("AVI", (vec!["RIFF"], "Audio Video Interleaved"));
        map.insert("AVIF", (vec!["MOV"], "AV1 Image File Format"));
        map.insert("BMP", (vec!["BMP"], "Windows Bitmap"));
        map.insert("BPG", (vec!["BPG"], "Better Portable Graphics"));
        map.insert("BTF", (vec!["BTF"], "Big Tagged Image File Format"));
        map.insert("BZ2", (vec!["BZ2"], "BZIP2 archive"));
        map.insert(
            "C2PA",
            (
                vec!["JUMBF"],
                "Coalition for Content Provenance and Authenticity",
            ),
        );
        map.insert("CHM", (vec!["CHM"], "Microsoft Compiled HTML format"));
        map.insert("CIFF", (vec!["CRW"], "Camera Image File Format"));
        map.insert("COS", (vec!["COS"], "Capture One Settings"));
        map.insert("CR2", (vec!["TIFF"], "Canon RAW 2 format"));
        map.insert("CR3", (vec!["MOV"], "Canon RAW 3 format"));
        map.insert("CRM", (vec!["MOV"], "Canon RAW Movie"));
        map.insert("CRW", (vec!["CRW"], "Canon RAW format"));
        map.insert("CS1", (vec!["PSD"], "Sinar CaptureShop 1-Shot RAW"));
        map.insert("CSV", (vec!["TXT"], "Comma-Separated Values"));
        map.insert("CUR", (vec!["ICO"], "Windows Cursor"));
        map.insert("CZI", (vec!["CZI"], "Zeiss Integrated Software RAW"));
        map.insert("DCP", (vec!["TIFF"], "DNG Camera Profile"));
        map.insert("DCR", (vec!["TIFF"], "Kodak Digital Camera RAW"));
        map.insert("DCX", (vec!["DCX"], "Multi-page PC Paintbrush"));
        map.insert("DEX", (vec!["DEX"], "Dalvik Executable format"));
        map.insert("DFONT", (vec!["Font"], "Macintosh Data fork Font"));
        map.insert("DIB", (vec!["BMP"], "Device Independent Bitmap"));
        map.insert(
            "DICM",
            (
                vec!["DICOM"],
                "Digital Imaging and Communications in Medicine",
            ),
        );
        map.insert("DIR", (vec!["DIR"], "Directory"));
        map.insert("DIVX", (vec!["ASF"], "DivX media format"));
        map.insert("DJVU", (vec!["AIFF"], "DjVu image"));
        map.insert("DLL", (vec!["EXE"], "Windows Dynamic Link Library"));
        map.insert("DNG", (vec!["TIFF"], "Digital Negative"));
        map.insert("DOC", (vec!["FPX"], "Microsoft Word Document"));
        map.insert(
            "DOCM",
            (vec!["ZIP", "FPX"], "Office Open XML Document Macro-enabled"),
        );
        map.insert("DOCX", (vec!["ZIP", "FPX"], "Office Open XML Document"));
        map.insert("DOT", (vec!["FPX"], "Microsoft Word Template"));
        map.insert(
            "DOTM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Document Template Macro-enabled",
            ),
        );
        map.insert(
            "DOTX",
            (vec!["ZIP", "FPX"], "Office Open XML Document Template"),
        );
        map.insert("DPX", (vec!["DPX"], "Digital Picture Exchange"));
        map.insert("DR4", (vec!["DR4"], "Canon VRD version 4 Recipe"));
        map.insert("DS2", (vec!["DSS"], "Digital Speech Standard 2"));
        map.insert("DSS", (vec!["DSS"], "Digital Speech Standard"));
        map.insert("DV", (vec!["DV"], "Digital Video"));
        map.insert("DVB", (vec!["MOV"], "Digital Video Broadcasting"));
        map.insert("DVR-MS", (vec!["ASF"], "Microsoft Digital Video recording"));
        map.insert("DWF", (vec!["DWF"], "Autodesk drawing (Design Web Format)"));
        map.insert("DWG", (vec!["DWG"], "AutoCAD Drawing"));
        map.insert("DXF", (vec!["DXF"], "AutoCAD Drawing Exchange Format"));
        map.insert("DYLIB", (vec!["EXE"], "Mach-O Dynamic Link Library"));
        map.insert("EIP", (vec!["ZIP"], "Capture One Enhanced Image Package"));
        map.insert("EPS", (vec!["EPS"], "Encapsulated PostScript Format"));
        map.insert("EPUB", (vec!["ZIP"], "Electronic Publication"));
        map.insert("ERF", (vec!["TIFF"], "Epson Raw Format"));
        map.insert("EXE", (vec!["EXE"], "Windows executable file"));
        map.insert("EXIF", (vec!["EXIF"], "Exchangable Image File Metadata"));
        map.insert("EXR", (vec!["EXR"], "Open EXR"));
        map.insert("EXV", (vec!["EXV"], "Exiv2 metadata"));
        map.insert("F4A", (vec!["MOV"], "Adobe Flash Player 9+ Audio"));
        map.insert("F4B", (vec!["MOV"], "Adobe Flash Player 9+ audio Book"));
        map.insert("F4P", (vec!["MOV"], "Adobe Flash Player 9+ Protected"));
        map.insert("F4V", (vec!["MOV"], "Adobe Flash Player 9+ Video"));
        map.insert(
            "FFF",
            (vec!["TIFF", "FLIR"], "Hasselblad Flexible File Format"),
        );
        map.insert("FITS", (vec!["FITS"], "Flexible Image Transport System"));
        map.insert("FLA", (vec!["FPX"], "Macromedia/Adobe Flash project"));
        map.insert("FLAC", (vec!["FLAC"], "Free Lossless Audio Codec"));
        map.insert("FLIF", (vec!["FLIF"], "Free Lossless Image Format"));
        map.insert("FLIR", (vec!["FLIR"], "FLIR File Format"));
        map.insert("FLV", (vec!["FLV"], "Flash Video"));
        map.insert("FPF", (vec!["FPF"], "FLIR Public image Format"));
        map.insert("FPX", (vec!["FPX"], "FlashPix"));
        map.insert(
            "GIF",
            (vec!["GIF"], "Compuserve Graphics Interchange Format"),
        );
        map.insert("GLV", (vec!["MOV"], "Garmin Low-resolution Video"));
        map.insert("GPR", (vec!["TIFF"], "General Purpose RAW"));
        map.insert("GZIP", (vec!["GZIP"], "GNU ZIP compressed archive"));
        map.insert("HDP", (vec!["TIFF"], "Windows HD Photo"));
        map.insert("HDR", (vec!["HDR"], "Radiance RGBE High Dynamic Range"));
        map.insert(
            "HEIC",
            (vec!["MOV"], "High Efficiency Image Format still image"),
        );
        map.insert("HEIF", (vec!["MOV"], "High Efficiency Image Format"));
        map.insert("HTML", (vec!["HTML"], "HyperText Markup Language"));
        map.insert("ICC", (vec!["ICC"], "International Color Consortium"));
        map.insert("ICO", (vec!["ICO"], "Windows Icon"));
        map.insert("ICS", (vec!["VCard"], "iCalendar Schedule"));
        map.insert("IDML", (vec!["ZIP"], "Adobe InDesign Markup Language"));
        map.insert(
            "IIQ",
            (vec!["TIFF"], "Phase One Intelligent Image Quality RAW"),
        );
        map.insert("IND", (vec!["IND"], "Adobe InDesign"));
        map.insert("INDD", (vec!["IND"], "Adobe InDesign Document"));
        map.insert("INDT", (vec!["IND"], "Adobe InDesign Template"));
        map.insert("INSP", (vec!["JPEG"], "Insta360 Picture"));
        map.insert("INSV", (vec!["MOV"], "Insta360 Video"));
        map.insert("INX", (vec!["XMP"], "Adobe InDesign Interchange"));
        map.insert("ISO", (vec!["ISO"], "ISO 9660 disk image"));
        map.insert("ITC", (vec!["ITC"], "iTunes Cover Flow"));
        map.insert("J2C", (vec!["JP2"], "JPEG 2000 codestream"));
        map.insert("JNG", (vec!["PNG"], "JPG Network Graphics"));
        map.insert("JP2", (vec!["JP2"], "JPEG 2000 file"));
        map.insert("JPEG", (vec!["JPEG"], "Joint Photographic Experts Group"));
        map.insert("JPH", (vec!["JP2"], "High-throughput JPEG 2000"));
        map.insert("JPM", (vec!["JP2"], "JPEG 2000 compound image"));
        map.insert("JPS", (vec!["JPEG"], "JPEG Stereo image"));
        map.insert("JPX", (vec!["JP2"], "JPEG 2000 with extensions"));
        map.insert("JSON", (vec!["JSON"], "JavaScript Object Notation"));
        map.insert(
            "JUMBF",
            (vec!["JUMBF"], "JPEG Universal Metadata Box Format"),
        );
        map.insert("JXL", (vec!["JXL"], "JPEG XL"));
        map.insert("JXR", (vec!["TIFF"], "JPEG XR"));
        map.insert("K25", (vec!["TIFF"], "Kodak DC25 RAW"));
        map.insert("KDC", (vec!["TIFF"], "Kodak Digital Camera RAW"));
        map.insert("KEY", (vec!["ZIP"], "Apple Keynote presentation"));
        map.insert("KTH", (vec!["ZIP"], "Apple Keynote Theme"));
        map.insert("LA", (vec!["RIFF"], "Lossless Audio"));
        map.insert("LFP", (vec!["LFP"], "Lytro Light Field Picture"));
        map.insert("LIF", (vec!["LIF"], "Leica Image File"));
        map.insert("LNK", (vec!["LNK"], "Windows shortcut"));
        map.insert("LRI", (vec!["LRI"], "Light RAW"));
        map.insert("LRV", (vec!["MOV"], "Low-Resolution Video"));
        map.insert("M2TS", (vec!["M2TS"], "MPEG-2 Transport Stream"));
        map.insert("M2V", (vec!["MPEG"], "MPEG-2 Video"));
        map.insert("M4A", (vec!["MOV"], "MPEG-4 Audio"));
        map.insert("M4B", (vec!["MOV"], "MPEG-4 audio Book"));
        map.insert("M4P", (vec!["MOV"], "MPEG-4 Protected"));
        map.insert("M4V", (vec!["MOV"], "MPEG-4 Video"));
        map.insert("MACOS", (vec!["MacOS"], "MacOS ._ sidecar file"));
        map.insert("MAX", (vec!["FPX"], "3D Studio MAX"));
        map.insert("MEF", (vec!["TIFF"], "Mamiya (RAW) Electronic Format"));
        map.insert(
            "MIE",
            (vec!["MIE"], "Meta Information Encapsulation format"),
        );
        map.insert("MIFF", (vec!["MIFF"], "Magick Image File Format"));
        map.insert("MKA", (vec!["MKV"], "Matroska Audio"));
        map.insert("MKS", (vec!["MKV"], "Matroska Subtitle"));
        map.insert("MKV", (vec!["MKV"], "Matroska Video"));
        map.insert("MNG", (vec!["PNG"], "Multiple-image Network Graphics"));
        map.insert("MOBI", (vec!["PDB"], "Mobipocket electronic book"));
        map.insert("MODD", (vec!["PLIST"], "Sony Picture Motion metadata"));
        map.insert("MOI", (vec!["MOI"], "MOD Information file"));
        map.insert("MOS", (vec!["TIFF"], "Creo Leaf Mosaic"));
        map.insert("MOV", (vec!["MOV"], "Apple QuickTime movie"));
        map.insert("MP3", (vec!["MP3"], "MPEG-1 Layer 3 audio"));
        map.insert("MP4", (vec!["MOV"], "MPEG-4 video"));
        map.insert("MPC", (vec!["MPC"], "Musepack Audio"));
        map.insert("MPEG", (vec!["MPEG"], "MPEG-1 or MPEG-2 audio/video"));
        map.insert("MPO", (vec!["JPEG"], "Extended Multi-Picture format"));
        map.insert("MQV", (vec!["MOV"], "Sony Mobile Quicktime Video"));
        map.insert("MRC", (vec!["MRC"], "Medical Research Council image"));
        map.insert("MRW", (vec!["MRW"], "Minolta RAW format"));
        map.insert("MXF", (vec!["MXF"], "Material Exchange Format"));
        map.insert("NEF", (vec!["TIFF"], "Nikon (RAW) Electronic Format"));
        map.insert("NKA", (vec!["NKA"], "Nikon NX Studio Adjustments"));
        map.insert("NKSC", (vec!["XMP"], "Nikon Sidecar"));
        map.insert("NMBTEMPLATE", (vec!["ZIP"], "Apple Numbers Template"));
        map.insert("NRW", (vec!["TIFF"], "Nikon RAW (2)"));
        map.insert("NUMBERS", (vec!["ZIP"], "Apple Numbers spreadsheet"));
        map.insert("NXD", (vec!["XMP"], "Nikon NX-D Settings"));
        map.insert("O", (vec!["EXE"], "Relocatable Object"));
        map.insert("ODB", (vec!["ZIP"], "Open Document Database"));
        map.insert("ODC", (vec!["ZIP"], "Open Document Chart"));
        map.insert("ODF", (vec!["ZIP"], "Open Document Formula"));
        map.insert("ODG", (vec!["ZIP"], "Open Document Graphics"));
        map.insert("ODI", (vec!["ZIP"], "Open Document Image"));
        map.insert("ODP", (vec!["ZIP"], "Open Document Presentation"));
        map.insert("ODS", (vec!["ZIP"], "Open Document Spreadsheet"));
        map.insert("ODT", (vec!["ZIP"], "Open Document Text file"));
        map.insert("OFR", (vec!["RIFF"], "OptimFROG audio"));
        map.insert("OGG", (vec!["OGG"], "Ogg Vorbis audio file"));
        map.insert("OGV", (vec!["OGG"], "Ogg Video file"));
        map.insert("ONP", (vec!["JSON"], "ON1 Presets"));
        map.insert("OPUS", (vec!["OGG"], "Ogg Opus audio file"));
        map.insert("ORF", (vec!["ORF"], "Olympus RAW format"));
        map.insert("OTF", (vec!["Font"], "Open Type Font"));
        map.insert(
            "PAC",
            (vec!["RIFF"], "Lossless Predictive Audio Compression"),
        );
        map.insert("PAGES", (vec!["ZIP"], "Apple Pages document"));
        map.insert("PBM", (vec!["PPM"], "Portable BitMap"));
        map.insert("PCAP", (vec!["PCAP"], "Packet Capture"));
        map.insert("PCAPNG", (vec!["PCAP"], "Packet Capture Next Generation"));
        map.insert("PCD", (vec!["PCD"], "Kodak Photo CD Image Pac"));
        map.insert("PCX", (vec!["PCX"], "PC Paintbrush"));
        map.insert("PDB", (vec!["PDB"], "Palm Database"));
        map.insert("PDF", (vec!["PDF"], "Adobe Portable Document Format"));
        map.insert("PEF", (vec!["TIFF"], "Pentax (RAW) Electronic Format"));
        map.insert("PFA", (vec!["Font"], "PostScript Font ASCII"));
        map.insert("PFB", (vec!["Font"], "PostScript Font Binary"));
        map.insert("PFM", (vec!["Font", "PFM2"], "Printer Font Metrics"));
        map.insert("PGF", (vec!["PGF"], "Progressive Graphics File"));
        map.insert("PGM", (vec!["PPM"], "Portable Gray Map"));
        map.insert("PHP", (vec!["PHP"], "PHP Hypertext Preprocessor"));
        map.insert("PICT", (vec!["PICT"], "Apple PICTure"));
        map.insert("PLIST", (vec!["PLIST"], "Apple Property List"));
        map.insert("PMP", (vec!["PMP"], "Sony DSC-F1 Cyber-Shot PMP"));
        map.insert("PNG", (vec!["PNG"], "Portable Network Graphics"));
        map.insert("POT", (vec!["FPX"], "Microsoft PowerPoint Template"));
        map.insert(
            "POTM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Presentation Template Macro-enabled",
            ),
        );
        map.insert(
            "POTX",
            (vec!["ZIP", "FPX"], "Office Open XML Presentation Template"),
        );
        map.insert(
            "PPAM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Presentation Addin Macro-enabled",
            ),
        );
        map.insert(
            "PPAX",
            (vec!["ZIP", "FPX"], "Office Open XML Presentation Addin"),
        );
        map.insert("PPM", (vec!["PPM"], "Portable Pixel Map"));
        map.insert("PPS", (vec!["FPX"], "Microsoft PowerPoint Slideshow"));
        map.insert(
            "PPSM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Presentation Slideshow Macro-enabled",
            ),
        );
        map.insert(
            "PPSX",
            (vec!["ZIP", "FPX"], "Office Open XML Presentation Slideshow"),
        );
        map.insert("PPT", (vec!["FPX"], "Microsoft PowerPoint Presentation"));
        map.insert(
            "PPTM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Presentation Macro-enabled",
            ),
        );
        map.insert("PPTX", (vec!["ZIP", "FPX"], "Office Open XML Presentation"));
        map.insert("PRC", (vec!["PDB"], "Palm Database"));
        map.insert("PS", (vec!["PS"], "PostScript"));
        map.insert("PSB", (vec!["PSD"], "Photoshop Large Document"));
        map.insert("PSD", (vec!["PSD"], "Photoshop Document"));
        map.insert("PSDT", (vec!["PSD"], "Photoshop Document Template"));
        map.insert("PSP", (vec!["PSP"], "Paint Shop Pro"));
        map.insert("QTIF", (vec!["QTIF"], "QuickTime Image File"));
        map.insert("R3D", (vec!["R3D"], "Redcode RAW Video"));
        map.insert("RA", (vec!["Real"], "Real Audio"));
        map.insert("RAF", (vec!["RAF"], "FujiFilm RAW Format"));
        map.insert("RAM", (vec!["Real"], "Real Audio Metafile"));
        map.insert("RAR", (vec!["RAR"], "RAR Archive"));
        map.insert(
            "RAW",
            (
                vec!["RAW", "TIFF"],
                "Kyocera Contax N Digital RAW or Panasonic RAW",
            ),
        );
        map.insert("RIFF", (vec!["RIFF"], "Resource Interchange File Format"));
        map.insert("RM", (vec!["Real"], "Real Media"));
        map.insert("RMVB", (vec!["Real"], "Real Media Variable Bitrate"));
        map.insert("RPM", (vec!["Real"], "Real Media Plug-in Metafile"));
        map.insert("RSRC", (vec!["RSRC"], "Mac OS Resource"));
        map.insert("RTF", (vec!["RTF"], "Rich Text Format"));
        map.insert("RV", (vec!["Real"], "Real Video"));
        map.insert("RW2", (vec!["TIFF"], "Panasonic RAW 2"));
        map.insert("RWL", (vec!["TIFF"], "Leica RAW"));
        map.insert("RWZ", (vec!["RWZ"], "Rawzor compressed image"));
        map.insert("SEQ", (vec!["FLIR"], "FLIR image Sequence"));
        map.insert("SKETCH", (vec!["ZIP"], "Sketch design file"));
        map.insert("SO", (vec!["EXE"], "Shared Object file"));
        map.insert("SR2", (vec!["TIFF"], "Sony RAW Format 2"));
        map.insert("SRF", (vec!["TIFF"], "Sony RAW Format"));
        map.insert("SRW", (vec!["TIFF"], "Samsung RAW format"));
        map.insert("SVG", (vec!["XMP"], "Scalable Vector Graphics"));
        map.insert("SWF", (vec!["SWF"], "Shockwave Flash"));
        map.insert("TAR", (vec!["TAR"], "TAR archive"));
        map.insert("THM", (vec!["JPEG"], "Thumbnail"));
        map.insert("THMX", (vec!["ZIP", "FPX"], "Office Open XML Theme"));
        map.insert("TIFF", (vec!["TIFF"], "Tagged Image File Format"));
        map.insert("TORRENT", (vec!["Torrent"], "BitTorrent description file"));
        map.insert("TTC", (vec!["Font"], "True Type Font Collection"));
        map.insert("TTF", (vec!["Font"], "True Type Font"));
        map.insert("TXT", (vec!["TXT"], "Text file"));
        map.insert("VCARD", (vec!["VCard"], "Virtual Card"));
        map.insert(
            "VNT",
            (vec!["FPX", "VCard"], "Scene7 Vignette or V-Note text file"),
        );
        map.insert("VOB", (vec!["MPEG"], "Video Object"));
        map.insert("VRD", (vec!["VRD"], "Canon VRD Recipe Data"));
        map.insert("VSD", (vec!["FPX"], "Microsoft Visio Drawing"));
        map.insert("WAV", (vec!["RIFF"], "WAVeform (Windows digital audio)"));
        map.insert("WDP", (vec!["TIFF"], "Windows Media Photo"));
        map.insert("WEBM", (vec!["MKV"], "Google Web Movie"));
        map.insert("WEBP", (vec!["RIFF"], "Google Web Picture"));
        map.insert("WMA", (vec!["ASF"], "Windows Media Audio"));
        map.insert("WMF", (vec!["WMF"], "Windows Metafile Format"));
        map.insert("WMV", (vec!["ASF"], "Windows Media Video"));
        map.insert("WOFF", (vec!["Font"], "Web Open Font Format"));
        map.insert("WOFF2", (vec!["Font"], "Web Open Font Format 2"));
        map.insert("WPG", (vec!["WPG"], "WordPerfect Graphics"));
        map.insert("WTV", (vec!["WTV"], "Windows recorded TV show"));
        map.insert("WV", (vec!["RIFF"], "WavePack lossless audio"));
        map.insert("X3F", (vec!["X3F"], "Sigma RAW format"));
        map.insert("XCF", (vec!["XCF"], "GIMP native image format"));
        map.insert(
            "XHTML",
            (vec!["HTML"], "Extensible HyperText Markup Language"),
        );
        map.insert(
            "XISF",
            (vec!["XISF"], "Extensible Image Serialization Format"),
        );
        map.insert("XLA", (vec!["FPX"], "Microsoft Excel Add-in"));
        map.insert(
            "XLAM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Spreadsheet Add-in Macro-enabled",
            ),
        );
        map.insert("XLS", (vec!["FPX"], "Microsoft Excel Spreadsheet"));
        map.insert(
            "XLSB",
            (vec!["ZIP", "FPX"], "Office Open XML Spreadsheet Binary"),
        );
        map.insert(
            "XLSM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Spreadsheet Macro-enabled",
            ),
        );
        map.insert("XLSX", (vec!["ZIP", "FPX"], "Office Open XML Spreadsheet"));
        map.insert("XLT", (vec!["FPX"], "Microsoft Excel Template"));
        map.insert(
            "XLTM",
            (
                vec!["ZIP", "FPX"],
                "Office Open XML Spreadsheet Template Macro-enabled",
            ),
        );
        map.insert(
            "XLTX",
            (vec!["ZIP", "FPX"], "Office Open XML Spreadsheet Template"),
        );
        map.insert("XMP", (vec!["XMP"], "Extensible Metadata Platform"));
        map.insert("ZIP", (vec!["ZIP"], "ZIP archive"));
        map
    });

/// Resolve file type from extension, following aliases
pub fn resolve_file_type(extension: &str) -> Option<(Vec<&'static str>, &'static str)> {
    // Convert to uppercase for case-insensitive lookup
    let ext_upper = extension.to_uppercase();

    // First check for direct format lookup
    if let Some((formats, desc)) = FILE_TYPE_FORMATS.get(ext_upper.as_str()) {
        return Some((formats.clone(), *desc));
    }

    // Check for alias resolution
    if let Some(alias) = EXTENSION_ALIASES.get(ext_upper.as_str()) {
        return resolve_file_type(alias);
    }

    None
}

/// Get primary format for a file type
pub fn get_primary_format(file_type: &str) -> Option<String> {
    resolve_file_type(file_type).map(|(formats, _)| formats[0].to_string())
}

/// Check if a file type supports a specific format
pub fn supports_format(file_type: &str, format: &str) -> bool {
    resolve_file_type(file_type)
        .map(|(formats, _)| formats.contains(&format))
        .unwrap_or(false)
}

/// Get all extensions that support a specific format
pub fn extensions_for_format(target_format: &str) -> Vec<String> {
    let mut extensions = Vec::new();

    for (ext, (formats, _)) in FILE_TYPE_FORMATS.iter() {
        if formats.contains(&target_format) {
            extensions.push(ext.to_string());
        }
    }

    extensions
}

/// All known file type extensions
pub static FILE_TYPE_EXTENSIONS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut extensions = Vec::new();

    // Add all known extensions from FILE_TYPE_FORMATS
    for ext in FILE_TYPE_FORMATS.keys() {
        extensions.push(*ext);
    }

    // Add all extension aliases
    for ext in EXTENSION_ALIASES.keys() {
        extensions.push(*ext);
    }

    extensions.sort();
    extensions.dedup();
    extensions
});

/// Lookup file type by extension (wrapper around resolve_file_type)
/// Returns the first format for compatibility with existing code
pub fn lookup_file_type_by_extension(extension: &str) -> Option<String> {
    resolve_file_type(extension).map(|(formats, _)| formats[0].to_string())
}
