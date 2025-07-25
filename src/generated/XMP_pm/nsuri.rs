//! XMP namespace prefix to URI mappings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (96 entries)
static NAMESPACE_URIS_DATA: &[(&'static str, &'static str)] = &[
    ("DICOM", "http://ns.adobe.com/DICOM/"),
    ("GAudio", "http://ns.google.com/photos/1.0/audio/"),
    ("GCamera", "http://ns.google.com/photos/1.0/camera/"),
    ("GContainer", "http://ns.google.com/photos/1.0/container/"),
    ("GCreations", "http://ns.google.com/photos/1.0/creations/"),
    ("GDepth", "http://ns.google.com/photos/1.0/depthmap/"),
    ("GFocus", "http://ns.google.com/photos/1.0/focus/"),
    ("GImage", "http://ns.google.com/photos/1.0/image/"),
    ("GPano", "http://ns.google.com/photos/1.0/panorama/"),
    ("GSpherical", "http://ns.google.com/videos/1.0/spherical/"),
    ("GettyImagesGIFT", "http://xmp.gettyimages.com/gift/1.0/"),
    ("HDRGainMap", "http://ns.apple.com/HDRGainMap/1.0/"),
    (
        "Iptc4xmpCore",
        "http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/",
    ),
    ("Iptc4xmpExt", "http://iptc.org/std/Iptc4xmpExt/2008-02-29/"),
    ("LImage", "http://ns.leiainc.com/photos/1.0/image/"),
    ("MP", "http://ns.microsoft.com/photo/1.2/"),
    ("MP1", "http://ns.microsoft.com/photo/1.1"),
    ("MPRI", "http://ns.microsoft.com/photo/1.2/t/RegionInfo#"),
    ("MPReg", "http://ns.microsoft.com/photo/1.2/t/Region#"),
    ("MicrosoftPhoto", "http://ns.microsoft.com/photo/1.0"),
    ("Profile", "http://ns.google.com/photos/dd/1.0/profile/"),
    ("aas", "http://ns.apple.com/adjustment-settings/1.0/"),
    ("acdsee", "http://ns.acdsee.com/iptc/1.0/"),
    ("acdsee-rs", "http://ns.acdsee.com/regions/"),
    ("album", "http://ns.adobe.com/album/1.0/"),
    ("apdi", "http://ns.apple.com/pixeldatainfo/1.0/"),
    ("apple-fi", "http://ns.apple.com/faceinfo/1.0/"),
    ("ast", "http://ns.nikon.com/asteroid/1.0/"),
    ("aux", "http://ns.adobe.com/exif/1.0/aux/"),
    ("cc", "http://creativecommons.org/ns#"),
    ("cell", "http://developer.sonyericsson.com/cell/1.0/"),
    ("crd", "http://ns.adobe.com/camera-raw-defaults/1.0/"),
    ("creatorAtom", "http://ns.adobe.com/creatorAtom/1.0/"),
    (
        "crlcp",
        "http://ns.adobe.com/camera-raw-embedded-lens-profile/1.0/",
    ),
    ("crs", "http://ns.adobe.com/camera-raw-settings/1.0/"),
    ("crss", "http://ns.adobe.com/camera-raw-saved-settings/1.0/"),
    ("dc", "http://purl.org/dc/elements/1.1/"),
    ("dex", "http://ns.optimasc.com/dex/1.0/"),
    ("digiKam", "http://www.digikam.org/ns/1.0/"),
    ("drone-dji", "http://www.dji.com/drone-dji/1.0/"),
    ("dwc", "http://rs.tdwg.org/dwc/index.htm"),
    ("et", "http://ns.exiftool.org/1.0/"),
    ("exif", "http://ns.adobe.com/exif/1.0/"),
    ("exifEX", "http://cipa.jp/exif/1.0/"),
    (
        "expressionmedia",
        "http://ns.microsoft.com/expressionmedia/1.0/",
    ),
    ("extensis", "http://ns.extensis.com/extensis/1.0/"),
    ("fpv", "http://ns.fastpictureviewer.com/fpv/1.0/"),
    ("hdr_metadata", "http://ns.adobe.com/hdr-metadata/1.0/"),
    ("hdrgm", "http://ns.adobe.com/hdr-gain-map/1.0/"),
    ("iX", "http://ns.adobe.com/iX/1.0/"),
    ("ics", "http://ns.idimager.com/ics/1.0/"),
    ("lr", "http://ns.adobe.com/lightroom/1.0/"),
    ("mediapro", "http://ns.iview-multimedia.com/mediapro/1.0/"),
    (
        "mwg-coll",
        "http://www.metadataworkinggroup.com/schemas/collections/",
    ),
    (
        "mwg-kw",
        "http://www.metadataworkinggroup.com/schemas/keywords/",
    ),
    (
        "mwg-rs",
        "http://www.metadataworkinggroup.com/schemas/regions/",
    ),
    ("nine", "http://ns.nikon.com/nine/1.0/"),
    (
        "panorama",
        "http://ns.adobe.com/photoshop/1.0/panorama-profile",
    ),
    ("pdf", "http://ns.adobe.com/pdf/1.3/"),
    ("pdfx", "http://ns.adobe.com/pdfx/1.3/"),
    ("photoshop", "http://ns.adobe.com/photoshop/1.0/"),
    ("plus", "http://ns.useplus.org/ldf/xmp/1.0/"),
    ("pmi", "http://prismstandard.org/namespaces/pmi/2.2/"),
    ("prism", "http://prismstandard.org/namespaces/basic/2.0/"),
    ("prl", "http://prismstandard.org/namespaces/prl/2.1/"),
    ("prm", "http://prismstandard.org/namespaces/prm/3.0/"),
    (
        "pur",
        "http://prismstandard.org/namespaces/prismusagerights/2.1/",
    ),
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ("sdc", "http://ns.nikon.com/sdc/1.0/"),
    ("seal", "http://ns.seal/2024/1.0/"),
    ("stArea", "http://ns.adobe.com/xmp/sType/Area#"),
    (
        "stCamera",
        "http://ns.adobe.com/photoshop/1.0/camera-profile",
    ),
    ("stDim", "http://ns.adobe.com/xap/1.0/sType/Dimensions#"),
    ("stEvt", "http://ns.adobe.com/xap/1.0/sType/ResourceEvent#"),
    ("stFnt", "http://ns.adobe.com/xap/1.0/sType/Font#"),
    ("stJob", "http://ns.adobe.com/xap/1.0/sType/Job#"),
    ("stMfs", "http://ns.adobe.com/xap/1.0/sType/ManifestItem#"),
    ("stRef", "http://ns.adobe.com/xap/1.0/sType/ResourceRef#"),
    ("stVer", "http://ns.adobe.com/xap/1.0/sType/Version#"),
    ("svg", "http://www.w3.org/2000/svg"),
    ("swf", "http://ns.adobe.com/swf/1.0/"),
    ("tiff", "http://ns.adobe.com/tiff/1.0/"),
    ("x", "adobe:ns:meta/"),
    ("xmp", "http://ns.adobe.com/xap/1.0/"),
    ("xmpBJ", "http://ns.adobe.com/xap/1.0/bj/"),
    ("xmpDM", "http://ns.adobe.com/xmp/1.0/DynamicMedia/"),
    (
        "xmpDSA",
        "http://leica-camera.com/digital-shift-assistant/1.0/",
    ),
    ("xmpG", "http://ns.adobe.com/xap/1.0/g/"),
    ("xmpGImg", "http://ns.adobe.com/xap/1.0/g/img/"),
    ("xmpMM", "http://ns.adobe.com/xap/1.0/mm/"),
    ("xmpNote", "http://ns.adobe.com/xmp/note/"),
    ("xmpPLUS", "http://ns.adobe.com/xap/1.0/PLUS/"),
    ("xmpRights", "http://ns.adobe.com/xap/1.0/rights/"),
    ("xmpTPg", "http://ns.adobe.com/xap/1.0/t/pg/"),
    ("xmpidq", "http://ns.adobe.com/xmp/Identifier/qual/1.0/"),
];

/// Lookup table (lazy-initialized)
pub static NAMESPACE_URIS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| NAMESPACE_URIS_DATA.iter().copied().collect());

/// Look up value by key
pub fn lookup_namespace_uris(key: &str) -> Option<&'static str> {
    NAMESPACE_URIS.get(key).copied()
}
