//! Sony offset calculation patterns
//! ExifTool: Sony.pm
//! Generated: Sun Jul 20 15:28:04 2025

use std::collections::HashMap;
use std::sync::LazyLock;

/// Sony model condition patterns
pub static SONY_MODEL_CONDITIONS: LazyLock<Vec<(&str, &str, &str)>> = LazyLock::new(|| {
    vec![
        ("=~", "^DSLR-A(850|900", "regex"),
        ("=~", "^DSLR-A(230|290|330|380|390", "regex"),
        ("=~", "^NEX-5N$/',", "regex"),
        ("=~", "^(SLT-A(65|77", "regex"),
        ("=~", "^(SLT-A(37|57", "regex"),
        ("=~", "^(DSC-(HX10V|HX20V|HX30V|HX200V|TX66|TX200V|TX300V|WX50|WX70|WX100|WX150", "regex"),
        ("=~", "^(SLT-A99V?|HV|SLT-A58|ILCE-(3000|3500", "regex"),
        ("=~", "^(DSC-(HX300|HX50|HX50V|TX30|WX60|WX80|WX200|WX300", "regex"),
        ("=~", "^(DSC-(RX100M2|QX10|QX100", "regex"),
        ("=~", "^(DSC-(QX30|RX10|RX100M3|HX60V|HX350|HX400V|WX220|WX350", "regex"),
        ("=~", "^(DSC-(RX0|RX1RM2|RX10M2|RX10M3|RX100M4|RX100M5|HX80|HX90V?|WX500", "regex"),
        ("=~", "^(ILCE-(6100|6400|6600|7C|7M3|7RM3A?|7RM4A?|9|9M2", "regex"),
        ("!~", "^DSC-", "regex"),
        ("=~", "^DSC-(RX10M4|RX100M6|RX100M7|RX100M5A|HX95|HX99|RX0M2", "regex"),
        ("=~", "^(SLT-|HV", "regex"),
        ("=~", "^(NEX-|ILCE-|ILME-|ZV-|DSC-(RX10M4|RX100M6|RX100M7|RX100M5A|HX95|HX99|RX0M2", "regex"),
        ("=~", "^ILCA-/',", "regex"),
        ("=~", "^(ILCE-|ILME-", "regex"),
        ("=~", "^ILCA-(68|77M2", "regex"),
        ("=~", "^ILCA-99M2/ and defined $$self{AFAreaILCA} and $$self{AFAreaILCA} != 8", "regex"),
        ("=~", "^ILCA-/ and defined $$self{AFAreaILCA} and $$self{AFAreaILCA} == 8", "regex"),
        ("=~", "^(NEX-|ILCE-|ILME-|ZV-|DSC-RX", "regex"),
        ("!~", "^(ILCA-|DSC-|ZV-", "regex"),
        ("=~", "^(ILCE-(5100|6000|7M2", "regex"),
        ("=~", "^ILCE-7RM2/',", "regex"),
        ("!~", "^(DSC-|Stellar|ILCE-(1|6100|6300|6400|6500|6600|6700|7C|7M3|7M4|7RM2|7RM3A?|7RM4A?|7RM5|7SM2|7SM3|9|9M2", "regex"),
        ("=~", "^(ILCE-(6100|6300|6400|6500|6600|7C|7M3|7RM2|7RM3A?|7RM4A?|7SM2|9|9M2", "regex"),
        ("=~", "^(ILCE-(1\\b|7M4|7RM5|7SM3", "regex"),
        ("=~", "^(ILCE-(6700|7CM2|7CR", "regex"),
        ("=~", "^(ILCE-1M2", "regex"),
        ("!~", "^(SLT-|HV|ILCA-", "regex"),
        ("=~", "^(NEX-|ILCE-|ILME-|Lunar|ZV-E10|ZV-E10M2|ZV-E1", "regex"),
        ("=~", "^(SLT-|HV|ILCA-", "regex"),
        ("=~", "^(NEX-|ILCE-|Lunar", "regex"),
        ("=~", "DSLR-A100\\b", "regex"),
        ("=~", "^(DSC-|Stellar", "regex"),
        ("!~", "^NEX-5C/',", "regex"),
        ("!~", "^DSLR-(A450|A500|A550", "regex"),
        ("=~", "^(DSLR-A(450|500|550", "regex"),
        ("=~", "^DSLR-A(450|500|550", "regex"),
        ("=~", "^(SLT-|DSLR-A(560|580", "regex"),
        ("=~", "^DSLR-A(700|850|900", "regex"),
        ("!~", "^DSLR-A(700|850|900", "regex"),
        ("=~", "^DSLR-A(230|290|330|380|390|850|900", "regex"),
        ("=~", "^DSLR-A(200|230|290|300|330|350|380|390|700|850|900", "regex"),
        ("=~", "^DSLR-(A450|A500|A550", "regex"),
        ("!~", "^(NEX-|DSLR-(A450|A500|A550", "regex"),
        ("=~", "^(DSLR-(A450|A500|A550", "regex"),
        ("!~", "^DSLR-A(450|500|550", "regex"),
        ("!~", "^NEX-(3|5", "regex"),
        ("=~", "^SLT-A(33|55V", "regex"),
        ("=~", "^DSLR-A(560|580", "regex"),
        ("=~", "^(SLT-A35|NEX-C3", "regex"),
        ("!~", "^NEX-(3|5|5C", "regex"),
        ("=~", "^NEX-(3|5|5C", "regex"),
        ("=~", "^NEX-", "regex"),
        ("!~", "^(NEX-(3|5|5C|C3|VG10|VG10E", "regex"),
        ("=~", "^SLT-/',", "regex"),
        ("=~", "^DSLR-/',", "regex"),
        ("=~", "^(NEX-(3|5|5C|C3|VG10|VG10E", "regex"),
        ("=~", "^(SLT-(A99|A99V", "regex"),
        ("=~", "^(DSC-(RX1|RX1R", "regex"),
        ("=~", "^(SLT-A58|ILCE-(3000|3500", "regex"),
        ("!~", "^(DSC-|Stellar", "regex"),
        ("!~", "^DSC-/',", "regex"),
        ("!~", "^(DSC-RX100|Stellar", "regex"),
        ("=~", "^(DSC-RX100|Stellar", "regex"),
        ("=~", "^DSC-/ ? undef : $val',", "regex"),
        ("!~", "^(ILCA-99M2|ILCE-6500|DSC-(RX0|RX100M5", "regex"),
        ("=~", "^(ILCA-99M2|ILCE-6500|DSC-(RX0|RX100M5", "regex"),
        ("!~", "^(NEX-|Lunar|ILCE-", "regex"),
        ("=~", "^(ILCE-(7(R|S|M2", "regex"),
        ("!~", "^(SLT-A(65|77", "regex"),
        ("!~", "^(Lunar|NEX-(5N|7|VG20E", "regex"),
        ("=~", "^(SLT-A99|HV|ILCE-7", "regex"),
        ("=~", "^(SLT-A(37|57|65|77", "regex"),
        ("!~", "^(SLT-A(37|57|65|77", "regex"),
        ("=~", "^(SLT-A(58|99V?", "regex"),
        ("=~", "^(ILCE-(5100|QX1", "regex"),
        ("=~", "^(ILCE-(5100|7S|7M2|QX1", "regex"),
        ("=~", "^(NEX-(5R|5T|6|VG30E|VG900", "regex"),
        ("=~", "^(ILCE-(3000|3500|5000|6000|7|7R", "regex"),
        ("=~", "^(ILCE-(7S|7M2|5100|QX1", "regex"),
        ("=~", "^(Lunar|NEX-(F3|5N|7|VG20E", "regex"),
        ("=~", "^(DSC-RX100M3|ILCA-(68|77M2", "regex"),
        ("=~", "^(DSC-RX10|SLT-A(58|99V?", "regex"),
        ("=~", "^(ILCA-", "regex"),
        ("!~", "^(ILCE-6400", "regex"),
        ("=~", "^(ILCE-(6100|6400|6600|7C|7RM4A?|9M2", "regex"),
        ("=~", "^(ILCE-(7M3|7RM3A?", "regex"),
        ("!~", "^(ILCA-99M2|ILCE-(6100|6400|6600|7C|7M3|7RM3A?|7RM4A?|9M2", "regex"),
        ("!~", "^(ILCA-99M2|ILCE-(6100|6400|6600|7C|7M3|7RM3A?|7RM4A?|9|9M2", "regex"),
        ("!~", "^(ILCE-(6100|6400|6600|7C|7M3|7RM3A?|7RM4A?|9M2", "regex"),
        ("=~", "^(ILCE-7|ILCE-9|ILCA-99", "regex"),
        ("=~", "^(ILCE-(7RM2|7SM2", "regex"),
        ("=~", "^(ILCE-(6300|6500", "regex"),
        ("=~", "^ILCE-(7RM4A?|7C|9M2", "regex"),
        ("=~", "^(ILCE-(7M3|7RM3A?|9", "regex"),
        ("=~", "^(ILCE-(6100|6400|6600|7M3|7RM3A?|9", "regex"),
        ("=~", "^(ILCA-99M2", "regex"),
        ("=~", "^(ILCE-(7M4|7RM5|7SM3", "regex"),
        ("=~", "^(ILCE-1", "regex"),
        ("=~", "^(ILCE-(1M2|6700|7CM2|7CR", "regex"),
        ("!~", "^(ZV-E10M2", "regex"),
        ("!~", "^(SLT-(A65|A77", "regex"),
        ("=~", "^(SLT-|HV|NEX-|Lunar|DSC-RX|Stellar", "regex"),
        ("=~", "^(ILCA-(68|77M2|99M2", "regex"),
        ("!~", "^(ILCE-(1|1M2|6700|7CM2|7CR|7M4|7RM5|7SM3|9M3", "regex"),
        ("!~", "^(ILCE-(1|6700|7CM2|7CR|7M4|7RM5|7SM3|9M3", "regex"),
        ("=~", "^(DSC-(HX350|HX400V|HX60V|HX80|HX90|HX90V|QX30|RX10|RX10M2|RX10M3|RX100M3|RX100M4", "regex"),
        ("=~", "^(DSC-(HX95|HX99|RX0|RX0M2|RX10M4|RX100M5|RX100M5A|RX100M6", "regex"),
        ("=~", "^(DSC-RX100M7|ZV-(1|1F|1M2", "regex"),
        ("!~", "^ZV-1M2", "regex"),
        ("=~", "^ZV-1M2/', Format => 'int8u[5]', SubDirectory => { TagTable => 'Image::ExifTool::Sony::ISOInfo' } },", "regex"),
        ("=~", "DSC/ ? 100 : 10", "regex"),
        ("!~", "^SLT-/',", "regex"),
        ("!~", "^(DSC-|ZV-", "regex"),
        ("!~", "^(ILCA-|ILCE-(7RM2|7M3|7RM3A?|7RM4A?|7SM2|6100|6300|6400|6500|6600|7C|9|9M2", "regex"),
        ("=~", "^(ILCA-(68|77M2", "regex"),
        ("=~", "^(DSC-(RX100M5|RX100M5A|RX100M6|RX100M7|RX10M4|HX99", "regex"),
        ("=~", "^(ILCE-7M2", "regex"),
        ("=~", "^(ILCA-99M2|ILCE-(6100|6400|6500|6600|7C|7M3|7RM3A?|7RM4A?|9|9M2", "regex"),
        ("=~", "^(ILCE-(6300|7RM2|7SM2", "regex"),
        ("=~", "^(ILCA-99M2|ILCE-6500", "regex"),
        ("!~", "^(ILCE-(7|7R", "regex"),
        ("!~", "^ILCA-/',", "regex"),
        ("=~", "^(ILCE-(6300|6500|7M3|7RM2|7RM3A?|7SM2|9", "regex"),
        ("!~", "^(DSC-", "regex"),
        ("=~", "^(DSC-", "regex"),
        ("=~", "^(ILCE-(1|7SM3", "regex"),
        ("=~", "^(ILCE-7M4", "regex"),
        ("=~", "^(ILCE-(1M2|6700|7CM2|7CR|7RM5", "regex"),
        ("=~", "^(ILCE-(1M2|7CM2|7CR|7RM5", "regex"),
    ]
});

/// Summary of offset calculation patterns found
pub static OFFSET_CALCULATION_TYPES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "variable_assignment",    // e.g., $offset
        "get16u_valpt",           // e.g., Get16u($valPt, 0)
        "get16u_variable",        // e.g., Get16u($dataPt, $start)
        "get16u_entry",           // e.g., Get16u($dataPt, $entry + 2)
        "get32u_valpt",           // e.g., Get32u($valPt, 0)
        "get32u_variable",        // e.g., Get32u($dataPt, $dirEnd)
        "entry_hash_offset",      // e.g., $entry{0xc634} + 8
        "variable_plus_constant", // e.g., $entry + 8
        "array_offset",           // e.g., $start + 4 + $i * 4
    ]
});

/// Example offset calculations extracted from Sony.pm
/// These demonstrate the patterns that need to be implemented
pub static OFFSET_EXAMPLES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("variable_assignment", "$offset");
    map.insert("get16u_valpt", "Get16u($valPt, 0)");
    map.insert("get16u_variable", "Get16u($dataPt, $start)");
    map.insert("get16u_entry", "Get16u($dataPt, $entry + 2)");
    map.insert("get32u_valpt", "Get32u($valPt, 0)");
    map.insert("get32u_variable", "Get32u($dataPt, $dirEnd)");
    map.insert("entry_hash_offset", "$entry{0xc634} + 8");
    map.insert("variable_plus_constant", "$entry + 8");
    map.insert("array_offset", "$start + 4 + $i * 4");
    map
});
