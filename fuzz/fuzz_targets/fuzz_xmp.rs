#![no_main]
//! XMP (RDF/XML) processing.
//!
//! `process_xmp_data` and `process_xmp_data_individual` parse the XMP packet's
//! XML. Malformed/adversarial XML (deeply nested elements, bogus namespaces)
//! goes here. Seeds are the ExifTool `.xmp` sidecar corpus.
use exif_oxide::xmp::XmpProcessor;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut processor = XmpProcessor::new();
    let _ = processor.process_xmp_data(data);

    let mut processor = XmpProcessor::new();
    let _ = processor.process_xmp_data_individual(data);
});
