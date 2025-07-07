use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;
use std::collections::HashMap;
use std::str;

// Sample XMP data with various features we need to support
const SAMPLE_XMP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 5.5.0">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:xmp="http://ns.adobe.com/xap/1.0/"
        xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/"
        xmlns:Iptc4xmpCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/">
      
      <!-- Simple property -->
      <xmp:CreateDate>2024-01-15T10:30:00Z</xmp:CreateDate>
      
      <!-- Language alternatives (rdf:Alt) -->
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">Default Title</rdf:li>
          <rdf:li xml:lang="en-US">English Title</rdf:li>
          <rdf:li xml:lang="fr-FR">Titre Français</rdf:li>
        </rdf:Alt>
      </dc:title>
      
      <!-- Unordered list (rdf:Bag) -->
      <dc:subject>
        <rdf:Bag>
          <rdf:li>keyword1</rdf:li>
          <rdf:li>keyword2</rdf:li>
          <rdf:li>keyword3</rdf:li>
        </rdf:Bag>
      </dc:subject>
      
      <!-- Ordered list (rdf:Seq) -->
      <dc:creator>
        <rdf:Seq>
          <rdf:li>First Author</rdf:li>
          <rdf:li>Second Author</rdf:li>
        </rdf:Seq>
      </dc:creator>
      
      <!-- Nested structure -->
      <Iptc4xmpCore:CreatorContactInfo rdf:parseType="Resource">
        <Iptc4xmpCore:CiAdrCity>New York</Iptc4xmpCore:CiAdrCity>
        <Iptc4xmpCore:CiAdrCtry>USA</Iptc4xmpCore:CiAdrCtry>
        <Iptc4xmpCore:CiEmailWork>contact@example.com</Iptc4xmpCore:CiEmailWork>
      </Iptc4xmpCore:CreatorContactInfo>
      
      <!-- Rights with language alternatives -->
      <xmpRights:UsageTerms>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">© 2024 Example Corp</rdf:li>
          <rdf:li xml:lang="en">Copyright 2024 Example Corporation</rdf:li>
        </rdf:Alt>
      </xmpRights:UsageTerms>
      
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

// Minimal TagValue enum for testing
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum TagValue {
    String(String),
    Array(Vec<TagValue>),
    Object(HashMap<String, TagValue>),
}

fn main() -> Result<()> {
    println!("=== Testing quick-xml for XMP Requirements ===\n");

    // Test 1: Basic namespace resolution
    test_namespace_resolution()?;

    // Test 2: RDF container parsing
    test_rdf_containers()?;

    // Test 3: Nested structure handling
    test_nested_structures()?;

    // Test 4: Language alternatives
    test_language_alternatives()?;

    // Test 5: UTF encoding support
    test_utf_encoding()?;

    // Test 6: Performance characteristics
    test_performance()?;

    println!("\n✅ All tests passed! quick-xml appears suitable for XMP parsing.");

    Ok(())
}

fn test_namespace_resolution() -> Result<()> {
    println!("Test 1: Namespace Resolution");
    println!("{}", "-".repeat(40));

    let mut reader = NsReader::from_str(SAMPLE_XMP);
    reader.config_mut().trim_text(true);

    let mut namespace_map = HashMap::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => {
                let (ns_result, local) = reader.resolve_element(e.name());

                if let ResolveResult::Bound(ns) = ns_result {
                    let namespace_str = str::from_utf8(ns.as_ref())?;
                    let local_name = str::from_utf8(local.as_ref())?;

                    // Track unique namespaces
                    if !namespace_str.is_empty() && !namespace_map.contains_key(namespace_str) {
                        namespace_map.insert(namespace_str.to_string(), local_name.to_string());
                        println!(
                            "  Found namespace: {namespace_str} (example element: {local_name})"
                        );
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    println!("  Total unique namespaces found: {}", namespace_map.len());
    assert!(
        namespace_map.len() >= 5,
        "Should find at least 5 namespaces"
    );
    assert!(
        namespace_map.contains_key("http://purl.org/dc/elements/1.1/"),
        "Should find Dublin Core namespace"
    );

    Ok(())
}

fn test_rdf_containers() -> Result<()> {
    println!("\nTest 2: RDF Container Parsing");
    println!("{}", "-".repeat(40));

    let mut reader = NsReader::from_str(SAMPLE_XMP);
    reader.config_mut().trim_text(true);

    let mut in_bag = false;
    let mut in_seq = false;
    let mut in_alt = false;
    let mut bag_items = Vec::new();
    let mut seq_items = Vec::new();
    let mut alt_items = HashMap::new();
    let mut current_lang = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                match name {
                    "Bag" => in_bag = true,
                    "Seq" => in_seq = true,
                    "Alt" => in_alt = true,
                    "li" if in_bag || in_seq || in_alt => {
                        // Check for xml:lang attribute in Alt containers
                        if in_alt {
                            for attr in e.attributes() {
                                let attr = attr?;
                                let (_, attr_local) = reader.resolve_attribute(attr.key);
                                let attr_name = str::from_utf8(attr_local.as_ref())?;
                                if attr_name == "lang" {
                                    let lang = str::from_utf8(&attr.value)?;
                                    current_lang = lang.to_string();
                                    alt_items.insert(lang.to_string(), String::new());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(e) => {
                let text = e.unescape()?.into_owned();
                if in_bag && !text.trim().is_empty() {
                    bag_items.push(text);
                } else if in_seq && !text.trim().is_empty() {
                    seq_items.push(text);
                } else if in_alt && !text.trim().is_empty() && !current_lang.is_empty() {
                    // Update the language entry
                    if let Some(value) = alt_items.get_mut(&current_lang) {
                        *value = text;
                    }
                }
            }
            Event::End(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                match name {
                    "Bag" => {
                        in_bag = false;
                        println!(
                            "  Found rdf:Bag with {} items: {:?}",
                            bag_items.len(),
                            bag_items
                        );
                        bag_items.clear();
                    }
                    "Seq" => {
                        in_seq = false;
                        println!(
                            "  Found rdf:Seq with {} items: {:?}",
                            seq_items.len(),
                            seq_items
                        );
                        seq_items.clear();
                    }
                    "Alt" => {
                        in_alt = false;
                        println!(
                            "  Found rdf:Alt with {} languages: {:?}",
                            alt_items.len(),
                            alt_items
                        );
                        alt_items.clear();
                    }
                    "li" => {
                        current_lang.clear();
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

fn test_nested_structures() -> Result<()> {
    println!("\nTest 3: Nested Structure Handling");
    println!("{}", "-".repeat(40));

    let mut reader = NsReader::from_str(SAMPLE_XMP);
    reader.config_mut().trim_text(true);

    let mut in_contact_info = false;
    let mut contact_info = HashMap::new();
    let mut current_field = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                if name == "CreatorContactInfo" {
                    in_contact_info = true;
                    // Check for rdf:parseType="Resource"
                    for attr in e.attributes() {
                        let attr = attr?;
                        let (_, attr_local) = reader.resolve_attribute(attr.key);
                        let attr_name = str::from_utf8(attr_local.as_ref())?;
                        if attr_name == "parseType" {
                            let value = str::from_utf8(&attr.value)?;
                            println!("  Found nested structure with rdf:parseType=\"{value}\"");
                        }
                    }
                } else if in_contact_info {
                    current_field = name.to_string();
                }
            }
            Event::Text(e) if in_contact_info => {
                let text = e.unescape()?.into_owned();
                if !current_field.is_empty() && !text.trim().is_empty() {
                    contact_info.insert(current_field.clone(), text);
                }
            }
            Event::End(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                if name == "CreatorContactInfo" {
                    in_contact_info = false;
                    println!("  Parsed ContactInfo structure: {contact_info:?}");
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    assert!(
        !contact_info.is_empty(),
        "Should parse nested ContactInfo structure"
    );

    Ok(())
}

fn test_language_alternatives() -> Result<()> {
    println!("\nTest 4: Language Alternative Handling");
    println!("{}", "-".repeat(40));

    let mut reader = NsReader::from_str(SAMPLE_XMP);
    reader.config_mut().trim_text(true);

    let mut in_title = false;
    let mut in_alt = false;
    let mut title_langs = HashMap::new();
    let mut current_lang = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                if name == "title" {
                    in_title = true;
                } else if name == "Alt" && in_title {
                    in_alt = true;
                } else if name == "li" && in_alt {
                    // Extract xml:lang attribute
                    for attr in e.attributes() {
                        let attr = attr?;
                        let (_, attr_local) = reader.resolve_attribute(attr.key);
                        let attr_name = str::from_utf8(attr_local.as_ref())?;
                        if attr_name == "lang" {
                            current_lang = str::from_utf8(&attr.value)?.to_string();
                        }
                    }
                }
            }
            Event::Text(e) if in_alt && !current_lang.is_empty() => {
                let text = e.unescape()?.into_owned();
                if !text.trim().is_empty() {
                    title_langs.insert(current_lang.clone(), text);
                }
            }
            Event::End(e) => {
                let (_, local) = reader.resolve_element(e.name());
                let name = str::from_utf8(local.as_ref())?;

                if name == "title" {
                    in_title = false;
                    println!("  Title language alternatives: {title_langs:?}");
                } else if name == "Alt" {
                    in_alt = false;
                } else if name == "li" {
                    current_lang.clear();
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    assert!(
        title_langs.contains_key("x-default"),
        "Should have x-default language"
    );
    assert!(
        title_langs.len() >= 2,
        "Should have multiple language alternatives"
    );

    Ok(())
}

fn test_utf_encoding() -> Result<()> {
    println!("\nTest 5: UTF Encoding Support");
    println!("{}", "-".repeat(40));

    // Test with UTF-8 content including various Unicode characters
    let utf8_xmp = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="ja">写真のタイトル 📸</rdf:li>
          <rdf:li xml:lang="zh">照片标题 🎯</rdf:li>
          <rdf:li xml:lang="ar">عنوان الصورة ✨</rdf:li>
        </rdf:Alt>
      </dc:title>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

    let mut reader = NsReader::from_str(utf8_xmp);
    reader.config_mut().trim_text(true);

    let mut unicode_texts = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Text(e) => {
                let text = e.unescape()?.into_owned();
                if !text.trim().is_empty() {
                    unicode_texts.push(text);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    println!("  Successfully parsed Unicode content:");
    for text in &unicode_texts {
        println!("    - {text}");
    }

    assert!(!unicode_texts.is_empty(), "Should parse Unicode content");

    // Test UTF-16 BOM detection would require actual file reading
    println!("  Note: UTF-16/32 BOM detection requires file-based testing");

    Ok(())
}

fn test_performance() -> Result<()> {
    println!("\nTest 6: Performance Characteristics");
    println!("{}", "-".repeat(40));

    use std::time::Instant;

    // Generate a larger XMP document for performance testing
    let mut large_xmp = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">"#,
    );

    // Add 1000 subject keywords
    large_xmp.push_str("<dc:subject><rdf:Bag>");
    for i in 0..1000 {
        large_xmp.push_str(&format!("<rdf:li>keyword{i}</rdf:li>"));
    }
    large_xmp.push_str("</rdf:Bag></dc:subject>");

    large_xmp.push_str("</rdf:Description></rdf:RDF></x:xmpmeta>");

    // Measure parsing time
    let start = Instant::now();
    let mut reader = NsReader::from_str(&large_xmp);
    reader.config_mut().trim_text(true);

    let mut event_count = 0;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            _ => event_count += 1,
        }
        buf.clear();
    }

    let duration = start.elapsed();

    println!(
        "  Parsed {} events from ~{}KB document in {:?}",
        event_count,
        large_xmp.len() / 1024,
        duration
    );
    println!(
        "  Events per second: {:.0}",
        event_count as f64 / duration.as_secs_f64()
    );

    // Memory usage observation
    println!("  Memory usage: Streaming parser (no DOM built)");

    // Test malformed XML handling
    println!("\n  Testing malformed XML handling:");
    let malformed_xml = r#"<x:xmpmeta><rdf:RDF><unclosed"#;
    let mut reader = NsReader::from_str(malformed_xml);
    let mut buf = Vec::new();

    let mut error_found = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("  ✓ Properly rejected malformed XML: {e}");
                error_found = true;
                break;
            }
            Ok(Event::Eof) => break,
            Ok(_) => continue,
        }
    }

    if !error_found {
        println!("  ⚠️  Accepted malformed XML without error");
    }

    Ok(())
}
