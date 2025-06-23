//! Enhanced Phase 1 tests for XMP functionality

use exif_oxide::core::jpeg::find_metadata_segments;
use exif_oxide::xmp::extract_simple_properties;
use std::fs;
use std::io::Cursor;

#[test]
fn test_simple_attribute_parsing() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            dc:format="image/jpeg"
            dc:language="en-US"
            xmp:CreatorTool="ExifOxide 1.0"
            xmp:CreateDate="2024-01-15T10:30:00"
            xmp:ModifyDate="2024-01-16T14:45:00"
            xmp:MetadataDate="2024-01-16T14:45:00">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    assert_eq!(props.get("dc:format"), Some(&"image/jpeg".to_string()));
    assert_eq!(props.get("dc:language"), Some(&"en-US".to_string()));
    assert_eq!(
        props.get("xmp:CreatorTool"),
        Some(&"ExifOxide 1.0".to_string())
    );
    assert_eq!(
        props.get("xmp:CreateDate"),
        Some(&"2024-01-15T10:30:00".to_string())
    );
    assert_eq!(
        props.get("xmp:ModifyDate"),
        Some(&"2024-01-16T14:45:00".to_string())
    );
    assert_eq!(
        props.get("xmp:MetadataDate"),
        Some(&"2024-01-16T14:45:00".to_string())
    );
}

#[test]
fn test_photoshop_attributes() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
            photoshop:AuthorsPosition="Photographer"
            photoshop:CaptionWriter="John Doe"
            photoshop:Category="Nature"
            photoshop:City="San Francisco"
            photoshop:Country="USA"
            photoshop:Credit="John Doe Photography"
            photoshop:DateCreated="2024-01-15"
            photoshop:Headline="Beautiful Sunset"
            photoshop:Source="Original"
            photoshop:State="California"
            photoshop:Urgency="5">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    assert_eq!(
        props.get("photoshop:AuthorsPosition"),
        Some(&"Photographer".to_string())
    );
    assert_eq!(
        props.get("photoshop:CaptionWriter"),
        Some(&"John Doe".to_string())
    );
    assert_eq!(props.get("photoshop:Category"), Some(&"Nature".to_string()));
    assert_eq!(
        props.get("photoshop:City"),
        Some(&"San Francisco".to_string())
    );
    assert_eq!(props.get("photoshop:Country"), Some(&"USA".to_string()));
    assert_eq!(
        props.get("photoshop:Credit"),
        Some(&"John Doe Photography".to_string())
    );
    assert_eq!(
        props.get("photoshop:DateCreated"),
        Some(&"2024-01-15".to_string())
    );
    assert_eq!(
        props.get("photoshop:Headline"),
        Some(&"Beautiful Sunset".to_string())
    );
    assert_eq!(props.get("photoshop:Source"), Some(&"Original".to_string()));
    assert_eq!(
        props.get("photoshop:State"),
        Some(&"California".to_string())
    );
    assert_eq!(props.get("photoshop:Urgency"), Some(&"5".to_string()));
}

#[test]
fn test_tiff_exif_attributes() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
            xmlns:exif="http://ns.adobe.com/exif/1.0/"
            tiff:Make="Canon"
            tiff:Model="Canon EOS R5"
            tiff:Orientation="1"
            tiff:XResolution="300/1"
            tiff:YResolution="300/1"
            tiff:ResolutionUnit="2"
            exif:DateTimeOriginal="2024:01:15 10:30:00"
            exif:ExposureTime="1/200"
            exif:FNumber="8/1"
            exif:ISO="100"
            exif:FocalLength="85/1"
            exif:LensModel="EF 85mm f/1.8">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // TIFF properties
    assert_eq!(props.get("tiff:Make"), Some(&"Canon".to_string()));
    assert_eq!(props.get("tiff:Model"), Some(&"Canon EOS R5".to_string()));
    assert_eq!(props.get("tiff:Orientation"), Some(&"1".to_string()));
    assert_eq!(props.get("tiff:XResolution"), Some(&"300/1".to_string()));
    assert_eq!(props.get("tiff:YResolution"), Some(&"300/1".to_string()));
    assert_eq!(props.get("tiff:ResolutionUnit"), Some(&"2".to_string()));

    // EXIF properties
    assert_eq!(
        props.get("exif:DateTimeOriginal"),
        Some(&"2024:01:15 10:30:00".to_string())
    );
    assert_eq!(props.get("exif:ExposureTime"), Some(&"1/200".to_string()));
    assert_eq!(props.get("exif:FNumber"), Some(&"8/1".to_string()));
    assert_eq!(props.get("exif:ISO"), Some(&"100".to_string()));
    assert_eq!(props.get("exif:FocalLength"), Some(&"85/1".to_string()));
    assert_eq!(
        props.get("exif:LensModel"),
        Some(&"EF 85mm f/1.8".to_string())
    );
}

#[test]
fn test_xmp_rights_management() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/"
            xmpRights:Marked="True"
            xmpRights:WebStatement="https://example.com/copyright"
            xmpRights:UsageTerms="All rights reserved">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    assert_eq!(props.get("xmpRights:Marked"), Some(&"True".to_string()));
    assert_eq!(
        props.get("xmpRights:WebStatement"),
        Some(&"https://example.com/copyright".to_string())
    );
    assert_eq!(
        props.get("xmpRights:UsageTerms"),
        Some(&"All rights reserved".to_string())
    );
}

#[test]
fn test_iptc_attributes() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/"
            Iptc4xmpCore:CountryCode="US"
            Iptc4xmpCore:Location="Golden Gate Bridge"
            Iptc4xmpCore:IntellectualGenre="Landscape Photography">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    assert_eq!(
        props.get("Iptc4xmpCore:CountryCode"),
        Some(&"US".to_string())
    );
    assert_eq!(
        props.get("Iptc4xmpCore:Location"),
        Some(&"Golden Gate Bridge".to_string())
    );
    assert_eq!(
        props.get("Iptc4xmpCore:IntellectualGenre"),
        Some(&"Landscape Photography".to_string())
    );
}

#[test]
fn test_empty_xmp_packet() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about="">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();
    assert!(props.is_empty());
}

#[test]
fn test_multiple_description_blocks() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:format="image/jpeg">
        </rdf:Description>
        <rdf:Description rdf:about=""
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmp:CreatorTool="Test Tool">
        </rdf:Description>
        <rdf:Description rdf:about=""
            xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
            photoshop:City="New York">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // All properties from different description blocks should be parsed
    assert_eq!(props.get("dc:format"), Some(&"image/jpeg".to_string()));
    assert_eq!(props.get("xmp:CreatorTool"), Some(&"Test Tool".to_string()));
    assert_eq!(props.get("photoshop:City"), Some(&"New York".to_string()));
}

#[test]
fn test_xmp_packet_wrapper() {
    // Test XMP with packet wrapper
    let xmp = br#"<?xpacket begin='' id='W5M0MpCehiHzreSzNTczkc9d'?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:format="image/jpeg">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

    let props = extract_simple_properties(xmp).unwrap();
    assert_eq!(props.get("dc:format"), Some(&"image/jpeg".to_string()));
}

#[test]
fn test_xmp_with_xml_declaration() {
    let xmp = br#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 5.6">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmp:Rating="5">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();
    assert_eq!(props.get("xmp:Rating"), Some(&"5".to_string()));
}

#[test]
fn test_special_characters_in_attributes() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            dc:title="Title with &quot;quotes&quot; and &lt;brackets&gt;"
            dc:description="Ampersand &amp; apostrophe &apos; test">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // XML entities are preserved in quick-xml (not automatically decoded)
    assert_eq!(
        props.get("dc:title"),
        Some(&"Title with &quot;quotes&quot; and &lt;brackets&gt;".to_string())
    );
    assert_eq!(
        props.get("dc:description"),
        Some(&"Ampersand &amp; apostrophe &apos; test".to_string())
    );
}

#[test]
fn test_numeric_values() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmlns:exif="http://ns.adobe.com/exif/1.0/"
            xmp:Rating="5"
            exif:PixelXDimension="4000"
            exif:PixelYDimension="3000"
            exif:ISO="100"
            exif:ExposureTime="0.005">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // Numeric values are stored as strings in Phase 1
    assert_eq!(props.get("xmp:Rating"), Some(&"5".to_string()));
    assert_eq!(props.get("exif:PixelXDimension"), Some(&"4000".to_string()));
    assert_eq!(props.get("exif:PixelYDimension"), Some(&"3000".to_string()));
    assert_eq!(props.get("exif:ISO"), Some(&"100".to_string()));
    assert_eq!(props.get("exif:ExposureTime"), Some(&"0.005".to_string()));
}

#[test]
fn test_boolean_values() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/"
            xmlns:xmp="http://ns.adobe.com/xap/1.0/"
            xmpRights:Marked="True"
            xmp:Sidecar="False">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // Boolean values are stored as strings in Phase 1
    assert_eq!(props.get("xmpRights:Marked"), Some(&"True".to_string()));
    assert_eq!(props.get("xmp:Sidecar"), Some(&"False".to_string()));
}

#[test]
fn test_custom_namespace() {
    let xmp = br#"<?xml version="1.0"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about=""
            xmlns:myapp="http://example.com/myapp/1.0/"
            myapp:Version="1.2.3"
            myapp:CustomField="Custom Value"
            myapp:Timestamp="2024-01-15T10:30:00Z">
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>"#;

    let props = extract_simple_properties(xmp).unwrap();

    // Custom namespace properties should be parsed
    assert_eq!(props.get("myapp:Version"), Some(&"1.2.3".to_string()));
    assert_eq!(
        props.get("myapp:CustomField"),
        Some(&"Custom Value".to_string())
    );
    assert_eq!(
        props.get("myapp:Timestamp"),
        Some(&"2024-01-15T10:30:00Z".to_string())
    );
}

#[test]
fn test_real_world_xmp_from_jpeg() {
    // Test with the actual ExifTool test image
    let jpeg_data = fs::read("exiftool/t/images/XMP.jpg").expect("Failed to read test image");

    // Extract XMP using our JPEG parser
    let mut cursor = Cursor::new(&jpeg_data);
    let metadata = find_metadata_segments(&mut cursor).expect("Failed to parse JPEG");

    assert!(!metadata.xmp.is_empty(), "Should find XMP segment");

    // Parse simple properties
    let props = extract_simple_properties(&metadata.xmp[0].data).unwrap();

    // This test image has various photoshop properties
    assert!(props.contains_key("photoshop:City"));
    assert!(props.contains_key("photoshop:Country"));
    assert!(props.contains_key("photoshop:Credit"));
}
