//! Inline PrintConv tables for CameraSettings3 table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: CameraSettings3)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static CAMERA_SETTINGS3_ASPECT_RATIO_DATA: &[(u8, &'static str)] = &[
    (4, "3:2"),
    (8, "16:9"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_ASPECT_RATIO: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_ASPECT_RATIO_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__aspect_ratio(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_ASPECT_RATIO.get(&key).copied()
}

/// Raw data (276 entries)
static CAMERA_SETTINGS3_LENS_TYPE2_DATA: &[(&'static str, &'static str)] = &[
    ("0", "Unknown E-mount lens or other lens"),
    ("0.1", "Sigma 19mm F2.8 [EX] DN"),
    ("0.10", "Zeiss Touit 50mm F2.8 Macro"),
    ("0.11", "Zeiss Loxia 50mm F2"),
    ("0.12", "Zeiss Loxia 35mm F2"),
    ("0.13", "Viltrox 85mm F1.8"),
    ("0.2", "Sigma 30mm F2.8 [EX] DN"),
    ("0.3", "Sigma 60mm F2.8 DN"),
    ("0.4", "Sony E 18-200mm F3.5-6.3 OSS LE"),
    ("0.5", "Tamron 18-200mm F3.5-6.3 Di III VC"),
    ("0.6", "Tokina FiRIN 20mm F2 FE AF"),
    ("0.7", "Tokina FiRIN 20mm F2 FE MF"),
    ("0.8", "Zeiss Touit 12mm F2.8"),
    ("0.9", "Zeiss Touit 32mm F1.8"),
    ("1", "Sony LA-EA1 or Sigma MC-11 Adapter"),
    ("13", "Samyang AF 35-150mm F2-2.8"),
    ("184", "Metabones Canon EF Speed Booster Ultra"),
    ("2", "Sony LA-EA2 Adapter"),
    ("20", "Samyang AF 35mm F1.4 P FE"),
    ("21", "Samyang AF 14-24mm F2.8"),
    ("234", "Metabones Canon EF Smart Adapter Mark IV"),
    ("239", "Metabones Canon EF Speed Booster"),
    ("24593", "LA-EA4r MonsterAdapter"),
    ("3", "Sony LA-EA3 Adapter"),
    ("32784", "Sony E 16mm F2.8"),
    ("32785", "Sony E 18-55mm F3.5-5.6 OSS"),
    ("32786", "Sony E 55-210mm F4.5-6.3 OSS"),
    ("32787", "Sony E 18-200mm F3.5-6.3 OSS"),
    ("32788", "Sony E 30mm F3.5 Macro"),
    ("32789", "Sony E 24mm F1.8 ZA or Samyang AF 50mm F1.4"),
    ("32789.1", "Samyang AF 50mm F1.4"),
    ("32790", "Sony E 50mm F1.8 OSS or Samyang AF 14mm F2.8"),
    ("32790.1", "Samyang AF 14mm F2.8"),
    ("32791", "Sony E 16-70mm F4 ZA OSS"),
    ("32792", "Sony E 10-18mm F4 OSS"),
    ("32793", "Sony E PZ 16-50mm F3.5-5.6 OSS"),
    ("32794", "Sony FE 35mm F2.8 ZA or Samyang Lens"),
    ("32794.1", "Samyang AF 24mm F2.8"),
    ("32794.2", "Samyang AF 35mm F2.8"),
    ("32795", "Sony FE 24-70mm F4 ZA OSS"),
    ("32796", "Sony FE 85mm F1.8 or Viltrox PFU RBMH 85mm F1.8"),
    ("32796.1", "Viltrox PFU RBMH 85mm F1.8"),
    ("32797", "Sony E 18-200mm F3.5-6.3 OSS LE"),
    ("32798", "Sony E 20mm F2.8"),
    ("32799", "Sony E 35mm F1.8 OSS"),
    ("32800", "Sony E PZ 18-105mm F4 G OSS"),
    ("32801", "Sony FE 12-24mm F4 G"),
    ("32802", "Sony FE 90mm F2.8 Macro G OSS"),
    ("32803", "Sony E 18-50mm F4-5.6"),
    ("32804", "Sony FE 24mm F1.4 GM"),
    ("32805", "Sony FE 24-105mm F4 G OSS"),
    ("32807", "Sony E PZ 18-200mm F3.5-6.3 OSS"),
    ("32808", "Sony FE 55mm F1.8 ZA"),
    ("32810", "Sony FE 70-200mm F4 G OSS"),
    ("32811", "Sony FE 16-35mm F4 ZA OSS"),
    ("32812", "Sony FE 50mm F2.8 Macro"),
    ("32813", "Sony FE 28-70mm F3.5-5.6 OSS"),
    ("32814", "Sony FE 35mm F1.4 ZA"),
    ("32815", "Sony FE 24-240mm F3.5-6.3 OSS"),
    ("32816", "Sony FE 28mm F2"),
    ("32817", "Sony FE PZ 28-135mm F4 G OSS"),
    ("32819", "Sony FE 100mm F2.8 STF GM OSS"),
    ("32820", "Sony E PZ 18-110mm F4 G OSS"),
    ("32821", "Sony FE 24-70mm F2.8 GM"),
    ("32822", "Sony FE 50mm F1.4 ZA"),
    ("32823", "Sony FE 85mm F1.4 GM or Samyang AF 85mm F1.4"),
    ("32823.1", "Samyang AF 85mm F1.4"),
    ("32824", "Sony FE 50mm F1.8"),
    ("32826", "Sony FE 21mm F2.8 (SEL28F20 + SEL075UWC)"),
    ("32827", "Sony FE 16mm F3.5 Fisheye (SEL28F20 + SEL057FEC)"),
    ("32828", "Sony FE 70-300mm F4.5-5.6 G OSS"),
    ("32829", "Sony FE 100-400mm F4.5-5.6 GM OSS"),
    ("32830", "Sony FE 70-200mm F2.8 GM OSS"),
    ("32831", "Sony FE 16-35mm F2.8 GM"),
    ("32848", "Sony FE 400mm F2.8 GM OSS"),
    ("32849", "Sony E 18-135mm F3.5-5.6 OSS"),
    ("32850", "Sony FE 135mm F1.8 GM"),
    ("32851", "Sony FE 200-600mm F5.6-6.3 G OSS"),
    ("32852", "Sony FE 600mm F4 GM OSS"),
    ("32853", "Sony E 16-55mm F2.8 G"),
    ("32854", "Sony E 70-350mm F4.5-6.3 G OSS"),
    ("32855", "Sony FE C 16-35mm T3.1 G"),
    ("32858", "Sony FE 35mm F1.8"),
    ("32859", "Sony FE 20mm F1.8 G"),
    ("32860", "Sony FE 12-24mm F2.8 GM"),
    ("32862", "Sony FE 50mm F1.2 GM"),
    ("32863", "Sony FE 14mm F1.8 GM"),
    ("32864", "Sony FE 28-60mm F4-5.6"),
    ("32865", "Sony FE 35mm F1.4 GM"),
    ("32866", "Sony FE 24mm F2.8 G"),
    ("32867", "Sony FE 40mm F2.5 G"),
    ("32868", "Sony FE 50mm F2.5 G"),
    ("32871", "Sony FE PZ 16-35mm F4 G"),
    ("32873", "Sony E PZ 10-20mm F4 G"),
    ("32874", "Sony FE 70-200mm F2.8 GM OSS II"),
    ("32875", "Sony FE 24-70mm F2.8 GM II"),
    ("32876", "Sony E 11mm F1.8"),
    ("32877", "Sony E 15mm F1.4 G"),
    ("32878", "Sony FE 20-70mm F4 G"),
    ("32879", "Sony FE 50mm F1.4 GM"),
    ("32880", "Sony FE 16mm F1.8 G"),
    ("32881", "Sony FE 24-50mm F2.8 G"),
    ("32882", "Sony FE 16-25mm F2.8 G"),
    ("32884", "Sony FE 70-200mm F4 Macro G OSS II"),
    ("32885", "Sony FE 16-35mm F2.8 GM II"),
    ("32886", "Sony FE 300mm F2.8 GM OSS"),
    ("32887", "Sony E PZ 16-50mm F3.5-5.6 OSS II"),
    ("32888", "Sony FE 85mm F1.4 GM II"),
    ("32889", "Sony FE 28-70mm F2 GM"),
    ("32890", "Sony FE 400-800mm F6.3-8 G OSS"),
    ("32891", "Sony FE 50-150mm F2 GM"),
    ("33072", "Sony FE 70-200mm F2.8 GM OSS + 1.4X Teleconverter"),
    ("33073", "Sony FE 70-200mm F2.8 GM OSS + 2X Teleconverter"),
    ("33076", "Sony FE 100mm F2.8 STF GM OSS (macro mode)"),
    ("33077", "Sony FE 100-400mm F4.5-5.6 GM OSS + 1.4X Teleconverter"),
    ("33078", "Sony FE 100-400mm F4.5-5.6 GM OSS + 2X Teleconverter"),
    ("33079", "Sony FE 400mm F2.8 GM OSS + 1.4X Teleconverter"),
    ("33080", "Sony FE 400mm F2.8 GM OSS + 2X Teleconverter"),
    ("33081", "Sony FE 200-600mm F5.6-6.3 G OSS + 1.4X Teleconverter"),
    ("33082", "Sony FE 200-600mm F5.6-6.3 G OSS + 2X Teleconverter"),
    ("33083", "Sony FE 600mm F4 GM OSS + 1.4X Teleconverter"),
    ("33084", "Sony FE 600mm F4 GM OSS + 2X Teleconverter"),
    ("33085", "Sony FE 70-200mm F2.8 GM OSS II + 1.4X Teleconverter"),
    ("33086", "Sony FE 70-200mm F2.8 GM OSS II + 2X Teleconverter"),
    ("33087", "Sony FE 70-200mm F4 Macro G OSS II + 1.4X Teleconverter"),
    ("33088", "Sony FE 70-200mm F4 Macro G OSS II + 2X Teleconverter"),
    ("33089", "Sony FE 300mm F2.8 GM OSS + 1.4X Teleconverter"),
    ("33090", "Sony FE 300mm F2.8 GM OSS + 2X Teleconverter"),
    ("33091", "Sony FE 400-800mm F6.3-8 G OSS + 1.4X Teleconverter"),
    ("33092", "Sony FE 400-800mm F6.3-8 G OSS + 2X Teleconverter"),
    ("44", "Metabones Canon EF Smart Adapter"),
    ("49201", "Zeiss Touit 12mm F2.8"),
    ("49202", "Zeiss Touit 32mm F1.8"),
    ("49203", "Zeiss Touit 50mm F2.8 Macro"),
    ("49216", "Zeiss Batis 25mm F2"),
    ("49217", "Zeiss Batis 85mm F1.8"),
    ("49218", "Zeiss Batis 18mm F2.8"),
    ("49219", "Zeiss Batis 135mm F2.8"),
    ("49220", "Zeiss Batis 40mm F2 CF"),
    ("49232", "Zeiss Loxia 50mm F2"),
    ("49233", "Zeiss Loxia 35mm F2"),
    ("49234", "Zeiss Loxia 21mm F2.8"),
    ("49235", "Zeiss Loxia 85mm F2.4"),
    ("49236", "Zeiss Loxia 25mm F2.4"),
    ("49456", "Tamron E 18-200mm F3.5-6.3 Di III VC"),
    ("49457", "Tamron 28-75mm F2.8 Di III RXD"),
    ("49458", "Tamron 17-28mm F2.8 Di III RXD"),
    ("49459", "Tamron 35mm F2.8 Di III OSD M1:2"),
    ("49460", "Tamron 24mm F2.8 Di III OSD M1:2"),
    ("49461", "Tamron 20mm F2.8 Di III OSD M1:2"),
    ("49462", "Tamron 70-180mm F2.8 Di III VXD"),
    ("49463", "Tamron 28-200mm F2.8-5.6 Di III RXD"),
    ("49464", "Tamron 70-300mm F4.5-6.3 Di III RXD"),
    ("49465", "Tamron 17-70mm F2.8 Di III-A VC RXD"),
    ("49466", "Tamron 150-500mm F5-6.7 Di III VC VXD"),
    ("49467", "Tamron 11-20mm F2.8 Di III-A RXD"),
    ("49468", "Tamron 18-300mm F3.5-6.3 Di III-A VC VXD"),
    ("49469", "Tamron 35-150mm F2-F2.8 Di III VXD"),
    ("49470", "Tamron 28-75mm F2.8 Di III VXD G2"),
    ("49471", "Tamron 50-400mm F4.5-6.3 Di III VC VXD"),
    ("49472", "Tamron 20-40mm F2.8 Di III VXD"),
    ("49473", "Tamron 17-50mm F4 Di III VXD or Tokina or Viltrox lens"),
    ("49473.1", "Tokina atx-m 85mm F1.8 FE"),
    ("49473.2", "Viltrox 23mm F1.4 E"),
    ("49473.3", "Viltrox 56mm F1.4 E"),
    ("49473.4", "Viltrox 85mm F1.8 II FE"),
    ("49474", "Tamron 70-180mm F2.8 Di III VXD G2 or Viltrox lens"),
    ("49474.1", "Viltrox 13mm F1.4 E"),
    ("49474.10", "Viltrox 20mm F2.8 FE"),
    ("49474.2", "Viltrox 16mm F1.8 FE"),
    ("49474.3", "Viltrox 23mm F1.4 E"),
    ("49474.4", "Viltrox 24mm F1.8 FE"),
    ("49474.5", "Viltrox 28mm F1.8 FE"),
    ("49474.6", "Viltrox 33mm F1.4 E"),
    ("49474.7", "Viltrox 35mm F1.8 FE"),
    ("49474.8", "Viltrox 50mm F1.8 FE"),
    ("49474.9", "Viltrox 75mm F1.2 E"),
    ("49475", "Tamron 50-300mm F4.5-6.3 Di III VC VXD"),
    ("49476", "Tamron 28-300mm F4-7.1 Di III VC VXD"),
    ("49477", "Tamron 90mm F2.8 Di III Macro VXD"),
    ("49712", "Tokina FiRIN 20mm F2 FE AF"),
    ("49713", "Tokina FiRIN 100mm F2.8 FE MACRO"),
    ("49714", "Tokina atx-m 11-18mm F2.8 E"),
    ("50480", "Sigma 30mm F1.4 DC DN | C"),
    ("50481", "Sigma 50mm F1.4 DG HSM | A"),
    ("50482", "Sigma 18-300mm F3.5-6.3 DC MACRO OS HSM | C + MC-11"),
    ("50483", "Sigma 18-35mm F1.8 DC HSM | A + MC-11"),
    ("50484", "Sigma 24-35mm F2 DG HSM | A + MC-11"),
    ("50485", "Sigma 24mm F1.4 DG HSM | A + MC-11"),
    ("50486", "Sigma 150-600mm F5-6.3 DG OS HSM | C + MC-11"),
    ("50487", "Sigma 20mm F1.4 DG HSM | A + MC-11"),
    ("50488", "Sigma 35mm F1.4 DG HSM | A"),
    ("50489", "Sigma 150-600mm F5-6.3 DG OS HSM | S + MC-11"),
    ("50490", "Sigma 120-300mm F2.8 DG OS HSM | S + MC-11"),
    ("50492", "Sigma 24-105mm F4 DG OS HSM | A + MC-11"),
    ("50493", "Sigma 17-70mm F2.8-4 DC MACRO OS HSM | C + MC-11"),
    ("50495", "Sigma 50-100mm F1.8 DC HSM | A + MC-11"),
    ("50499", "Sigma 85mm F1.4 DG HSM | A"),
    ("50501", "Sigma 100-400mm F5-6.3 DG OS HSM | C + MC-11"),
    ("50503", "Sigma 16mm F1.4 DC DN | C"),
    ("50507", "Sigma 105mm F1.4 DG HSM | A"),
    ("50508", "Sigma 56mm F1.4 DC DN | C"),
    ("50512", "Sigma 70-200mm F2.8 DG OS HSM | S + MC-11"),
    ("50513", "Sigma 70mm F2.8 DG MACRO | A"),
    ("50514", "Sigma 45mm F2.8 DG DN | C"),
    ("50515", "Sigma 35mm F1.2 DG DN | A"),
    ("50516", "Sigma 14-24mm F2.8 DG DN | A"),
    ("50517", "Sigma 24-70mm F2.8 DG DN | A"),
    ("50518", "Sigma 100-400mm F5-6.3 DG DN OS | C"),
    ("50521", "Sigma 85mm F1.4 DG DN | A"),
    ("50522", "Sigma 105mm F2.8 DG DN MACRO | A"),
    ("50523", "Sigma 65mm F2 DG DN | C"),
    ("50524", "Sigma 35mm F2 DG DN | C"),
    ("50525", "Sigma 24mm F3.5 DG DN | C"),
    ("50526", "Sigma 28-70mm F2.8 DG DN | C"),
    ("50527", "Sigma 150-600mm F5-6.3 DG DN OS | S"),
    ("50528", "Sigma 35mm F1.4 DG DN | A"),
    ("50529", "Sigma 90mm F2.8 DG DN | C"),
    ("50530", "Sigma 24mm F2 DG DN | C"),
    ("50531", "Sigma 18-50mm F2.8 DC DN | C"),
    ("50532", "Sigma 20mm F2 DG DN | C"),
    ("50533", "Sigma 16-28mm F2.8 DG DN | C"),
    ("50534", "Sigma 20mm F1.4 DG DN | A"),
    ("50535", "Sigma 24mm F1.4 DG DN | A"),
    ("50536", "Sigma 60-600mm F4.5-6.3 DG DN OS | S"),
    ("50537", "Sigma 50mm F2 DG DN | C"),
    ("50538", "Sigma 17mm F4 DG DN | C"),
    ("50539", "Sigma 50mm F1.4 DG DN | A"),
    ("50540", "Sigma 14mm F1.4 DG DN | A"),
    ("50543", "Sigma 70-200mm F2.8 DG DN OS | S"),
    ("50544", "Sigma 23mm F1.4 DC DN | C"),
    ("50545", "Sigma 24-70mm F2.8 DG DN II | A"),
    ("50546", "Sigma 500mm F5.6 DG DN OS | S"),
    ("50547", "Sigma 10-18mm F2.8 DC DN | C"),
    ("50548", "Sigma 15mm F1.4 DG DN DIAGONAL FISHEYE | A"),
    ("50549", "Sigma 50mm F1.2 DG DN | A"),
    ("50550", "Sigma 28-105mm F2.8 DG DN | A"),
    ("50551", "Sigma 28-45mm F1.8 DG DN | A"),
    ("50553", "Sigma 300-600mm F4 DG OS | S"),
    ("50992", "Voigtlander SUPER WIDE-HELIAR 15mm F4.5 III"),
    ("50993", "Voigtlander HELIAR-HYPER WIDE 10mm F5.6"),
    ("50994", "Voigtlander ULTRA WIDE-HELIAR 12mm F5.6 III"),
    ("50995", "Voigtlander MACRO APO-LANTHAR 65mm F2 Aspherical"),
    ("50996", "Voigtlander NOKTON 40mm F1.2 Aspherical"),
    ("50997", "Voigtlander NOKTON classic 35mm F1.4"),
    ("50998", "Voigtlander MACRO APO-LANTHAR 110mm F2.5"),
    ("50999", "Voigtlander COLOR-SKOPAR 21mm F3.5 Aspherical"),
    ("51000", "Voigtlander NOKTON 50mm F1.2 Aspherical"),
    ("51001", "Voigtlander NOKTON 21mm F1.4 Aspherical"),
    ("51002", "Voigtlander APO-LANTHAR 50mm F2 Aspherical"),
    ("51003", "Voigtlander NOKTON 35mm F1.2 Aspherical SE"),
    ("51006", "Voigtlander APO-LANTHAR 35mm F2 Aspherical"),
    ("51007", "Voigtlander NOKTON 50mm F1 Aspherical"),
    ("51008", "Voigtlander NOKTON 75mm F1.5 Aspherical"),
    ("51009", "Voigtlander NOKTON 28mm F1.5 Aspherical"),
    ("51072", "ZEISS Otus ML 50mm F1.4"),
    ("51073", "ZEISS Otus ML 85mm F1.4"),
    ("51504", "Samyang AF 50mm F1.4"),
    ("51505", "Samyang AF 14mm F2.8 or Samyang AF 35mm F2.8"),
    ("51505.1", "Samyang AF 35mm F2.8"),
    ("51507", "Samyang AF 35mm F1.4"),
    ("51508", "Samyang AF 45mm F1.8"),
    ("51510", "Samyang AF 18mm F2.8 or Samyang AF 35mm F1.8"),
    ("51510.1", "Samyang AF 35mm F1.8"),
    ("51512", "Samyang AF 75mm F1.8"),
    ("51513", "Samyang AF 35mm F1.8"),
    ("51514", "Samyang AF 24mm F1.8"),
    ("51515", "Samyang AF 12mm F2.0"),
    ("51516", "Samyang AF 24-70mm F2.8"),
    ("51517", "Samyang AF 50mm F1.4 II"),
    ("51518", "Samyang AF 135mm F1.8"),
    ("6", "Sony LA-EA4 Adapter"),
    ("61569", "LAOWA FFII 10mm F2.8 C&D Dreamer"),
    ("61761", "Viltrox 28mm F4.5 FE"),
    ("7", "Sony LA-EA5 Adapter"),
    ("78", "Metabones Canon EF Smart Adapter Mark III or Other Adapter"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LENS_TYPE2: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LENS_TYPE2_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__lens_type2(key: &str) -> Option<&'static str> {
    CAMERA_SETTINGS3_LENS_TYPE2.get(key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_QUALITY_DATA: &[(u8, &'static str)] = &[
    (2, "RAW"),
    (4, "RAW + JPEG"),
    (6, "Fine"),
    (7, "Standard"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_QUALITY: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_QUALITY_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__quality(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_QUALITY.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_DYNAMIC_RANGE_OPTIMIZER_SETTING_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On (Auto)"),
    (17, "On (Manual)"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_DYNAMIC_RANGE_OPTIMIZER_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_DYNAMIC_RANGE_OPTIMIZER_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__dynamic_range_optimizer_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_DYNAMIC_RANGE_OPTIMIZER_SETTING.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_DATA: &[(u8, &'static str)] = &[
    (1, "No"),
    (16, "Yes"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_A_F_BUTTON_PRESSED: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__a_f_button_pressed(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_A_F_BUTTON_PRESSED.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LIVE_VIEW_METERING_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (16, "40 Segment"),
    (32, "1200-zone Evaluative"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LIVE_VIEW_METERING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LIVE_VIEW_METERING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__live_view_metering(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LIVE_VIEW_METERING.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_VIEWING_MODE2_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (16, "Viewfinder"),
    (33, "Focus Check Live View"),
    (34, "Quick AF Live View"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_VIEWING_MODE2: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_VIEWING_MODE2_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__viewing_mode2(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_VIEWING_MODE2.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_A_E_LOCK_DATA: &[(u8, &'static str)] = &[
    (1, "On"),
    (2, "Off"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_A_E_LOCK: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_A_E_LOCK_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__a_e_lock(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_A_E_LOCK.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (2, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_status_built_in(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_DATA: &[(u8, &'static str)] = &[
    (1, "None"),
    (2, "Off"),
    (3, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_status_external(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "AF"),
    (16, "Manual"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__live_view_focus_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_COLOR_SPACE_DATA: &[(u8, &'static str)] = &[
    (1, "sRGB"),
    (2, "Adobe RGB"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_COLOR_SPACE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__color_space(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_COLOR_SPACE.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS3_CREATIVE_STYLE_SETTING_DATA: &[(u8, &'static str)] = &[
    (16, "Standard"),
    (32, "Vivid"),
    (64, "Portrait"),
    (80, "Landscape"),
    (96, "B&W"),
    (160, "Sunset"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_CREATIVE_STYLE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_CREATIVE_STYLE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__creative_style_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_CREATIVE_STYLE_SETTING.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LENS_MOUNT_DATA: &[(u8, &'static str)] = &[
    (1, "Unknown"),
    (16, "A-mount"),
    (17, "E-mount"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LENS_MOUNT: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LENS_MOUNT_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__lens_mount(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LENS_MOUNT.get(&key).copied()
}

/// Raw data (51 entries)
static CAMERA_SETTINGS3_WHITE_BALANCE_SETTING_DATA: &[(u8, &'static str)] = &[
    (16, "Auto (-3)"),
    (17, "Auto (-2)"),
    (18, "Auto (-1)"),
    (19, "Auto (0)"),
    (20, "Auto (+1)"),
    (21, "Auto (+2)"),
    (22, "Auto (+3)"),
    (32, "Daylight (-3)"),
    (33, "Daylight (-2)"),
    (34, "Daylight (-1)"),
    (35, "Daylight (0)"),
    (36, "Daylight (+1)"),
    (37, "Daylight (+2)"),
    (38, "Daylight (+3)"),
    (48, "Shade (-3)"),
    (49, "Shade (-2)"),
    (50, "Shade (-1)"),
    (51, "Shade (0)"),
    (52, "Shade (+1)"),
    (53, "Shade (+2)"),
    (54, "Shade (+3)"),
    (64, "Cloudy (-3)"),
    (65, "Cloudy (-2)"),
    (66, "Cloudy (-1)"),
    (67, "Cloudy (0)"),
    (68, "Cloudy (+1)"),
    (69, "Cloudy (+2)"),
    (70, "Cloudy (+3)"),
    (80, "Tungsten (-3)"),
    (81, "Tungsten (-2)"),
    (82, "Tungsten (-1)"),
    (83, "Tungsten (0)"),
    (84, "Tungsten (+1)"),
    (85, "Tungsten (+2)"),
    (86, "Tungsten (+3)"),
    (96, "Fluorescent (-3)"),
    (97, "Fluorescent (-2)"),
    (98, "Fluorescent (-1)"),
    (99, "Fluorescent (0)"),
    (100, "Fluorescent (+1)"),
    (101, "Fluorescent (+2)"),
    (102, "Fluorescent (+3)"),
    (112, "Flash (-3)"),
    (113, "Flash (-2)"),
    (114, "Flash (-1)"),
    (115, "Flash (0)"),
    (116, "Flash (+1)"),
    (117, "Flash (+2)"),
    (118, "Flash (+3)"),
    (163, "Custom"),
    (243, "Color Temperature/Color Filter"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_WHITE_BALANCE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_WHITE_BALANCE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__white_balance_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_WHITE_BALANCE_SETTING.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS3_FLASH_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Flash Off"),
    (16, "Autoflash"),
    (17, "Fill-flash"),
    (18, "Slow Sync"),
    (19, "Rear Sync"),
    (20, "Wireless"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_FLASH_CONTROL_DATA: &[(u8, &'static str)] = &[
    (1, "ADI Flash"),
    (2, "Pre-flash TTL"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_CONTROL: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_CONTROL_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_control(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_CONTROL.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_A_F_AREA_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Wide"),
    (2, "Spot"),
    (3, "Local"),
    (4, "Flexible"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_A_F_AREA_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_A_F_AREA_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__a_f_area_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_A_F_AREA_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_LONG_EXPOSURE_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LONG_EXPOSURE_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LONG_EXPOSURE_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__long_exposure_noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LONG_EXPOSURE_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_HIGH_I_S_O_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (16, "Low"),
    (17, "High"),
    (19, "Auto"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_HIGH_I_S_O_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_HIGH_I_S_O_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__high_i_s_o_noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_HIGH_I_S_O_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_SMILE_SHUTTER_MODE_DATA: &[(u8, &'static str)] = &[
    (17, "Slight Smile"),
    (18, "Normal Smile"),
    (19, "Big Smile"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_SMILE_SHUTTER_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_SMILE_SHUTTER_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__smile_shutter_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_SMILE_SHUTTER_MODE.get(&key).copied()
}

/// Raw data (11 entries)
static CAMERA_SETTINGS3_DRIVE_MODE_SETTING_DATA: &[(u8, &'static str)] = &[
    (16, "Single Frame"),
    (33, "Continuous High"),
    (34, "Continuous Low"),
    (48, "Speed Priority Continuous"),
    (81, "Self-timer 10 sec"),
    (82, "Self-timer 2 sec, Mirror Lock-up"),
    (113, "Continuous Bracketing 0.3 EV"),
    (117, "Continuous Bracketing 0.7 EV"),
    (145, "White Balance Bracketing Low"),
    (146, "White Balance Bracketing High"),
    (192, "Remote Commander"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_DRIVE_MODE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_DRIVE_MODE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__drive_mode_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_DRIVE_MODE_SETTING.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_RED_EYE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_RED_EYE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_RED_EYE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__red_eye_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_RED_EYE_REDUCTION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_H_D_R_SETTING_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On (Auto)"),
    (17, "On (Manual)"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_H_D_R_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_H_D_R_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__h_d_r_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_H_D_R_SETTING.get(&key).copied()
}

/// Raw data (9 entries)
static CAMERA_SETTINGS3_H_D_R_LEVEL_DATA: &[(u8, &'static str)] = &[
    (33, "1 EV"),
    (34, "1.5 EV"),
    (35, "2 EV"),
    (36, "2.5 EV"),
    (37, "3 EV"),
    (38, "3.5 EV"),
    (39, "4 EV"),
    (40, "5 EV"),
    (41, "6 EV"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_H_D_R_LEVEL: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_H_D_R_LEVEL_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__h_d_r_level(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_H_D_R_LEVEL.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_VIEWING_MODE_DATA: &[(u8, &'static str)] = &[
    (16, "ViewFinder"),
    (33, "Focus Check Live View"),
    (34, "Quick AF Live View"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_VIEWING_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_VIEWING_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__viewing_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_VIEWING_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_FACE_DETECTION_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FACE_DETECTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FACE_DETECTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__face_detection(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FACE_DETECTION.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_SMILE_SHUTTER_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (16, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_SMILE_SHUTTER: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_SMILE_SHUTTER_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__smile_shutter(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_SMILE_SHUTTER.get(&key).copied()
}

/// Raw data (32 entries)
static CAMERA_SETTINGS3_EXPOSURE_PROGRAM_DATA: &[(u8, &'static str)] = &[
    (1, "Program AE"),
    (2, "Aperture-priority AE"),
    (3, "Shutter speed priority AE"),
    (4, "Manual"),
    (5, "Cont. Priority AE"),
    (16, "Auto"),
    (17, "Auto (no flash)"),
    (18, "Auto+"),
    (49, "Portrait"),
    (50, "Landscape"),
    (51, "Macro"),
    (52, "Sports"),
    (53, "Sunset"),
    (54, "Night view"),
    (55, "Night view/portrait"),
    (56, "Handheld Night Shot"),
    (57, "3D Sweep Panorama"),
    (64, "Auto 2"),
    (65, "Auto 2 (no flash)"),
    (80, "Sweep Panorama"),
    (96, "Anti Motion Blur"),
    (128, "Toy Camera"),
    (129, "Pop Color"),
    (130, "Posterization"),
    (131, "Posterization B/W"),
    (132, "Retro Photo"),
    (133, "High-key"),
    (134, "Partial Color Red"),
    (135, "Partial Color Green"),
    (136, "Partial Color Blue"),
    (137, "Partial Color Yellow"),
    (138, "High Contrast Monochrome"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_EXPOSURE_PROGRAM: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_EXPOSURE_PROGRAM_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__exposure_program(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_EXPOSURE_PROGRAM.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_SWEEP_PANORAMA_SIZE_DATA: &[(u8, &'static str)] = &[
    (1, "Standard"),
    (2, "Wide"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_SWEEP_PANORAMA_SIZE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_SWEEP_PANORAMA_SIZE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__sweep_panorama_size(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_SWEEP_PANORAMA_SIZE.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_SWEEP_PANORAMA_DIRECTION_DATA: &[(u8, &'static str)] = &[
    (1, "Right"),
    (2, "Left"),
    (3, "Up"),
    (4, "Down"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_SWEEP_PANORAMA_DIRECTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_SWEEP_PANORAMA_DIRECTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__sweep_panorama_direction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_SWEEP_PANORAMA_DIRECTION.get(&key).copied()
}

/// Raw data (17 entries)
static CAMERA_SETTINGS3_DRIVE_MODE_DATA: &[(u8, &'static str)] = &[
    (16, "Single Frame"),
    (33, "Continuous High"),
    (34, "Continuous Low"),
    (48, "Speed Priority Continuous"),
    (81, "Self-timer 10 sec"),
    (82, "Self-timer 2 sec, Mirror Lock-up"),
    (113, "Continuous Bracketing 0.3 EV"),
    (117, "Continuous Bracketing 0.7 EV"),
    (145, "White Balance Bracketing Low"),
    (146, "White Balance Bracketing High"),
    (192, "Remote Commander"),
    (209, "Continuous - HDR"),
    (210, "Continuous - Multi Frame NR"),
    (211, "Continuous - Handheld Night Shot"),
    (212, "Continuous - Anti Motion Blur"),
    (213, "Continuous - Sweep Panorama"),
    (214, "Continuous - 3D Sweep Panorama"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_DRIVE_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_DRIVE_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__drive_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_DRIVE_MODE.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_MULTI_FRAME_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Off"),
    (16, "On"),
    (255, "None"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_MULTI_FRAME_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_MULTI_FRAME_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__multi_frame_noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_MULTI_FRAME_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LIVE_VIEW_A_F_SETTING_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Phase-detect AF"),
    (2, "Contrast AF"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LIVE_VIEW_A_F_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LIVE_VIEW_A_F_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__live_view_a_f_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LIVE_VIEW_A_F_SETTING.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_PANORAMA_SIZE3_D_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Standard"),
    (2, "Wide"),
    (3, "16:9"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_PANORAMA_SIZE3_D: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_PANORAMA_SIZE3_D_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__panorama_size3_d(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_PANORAMA_SIZE3_D.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS3_FOCUS_MODE_SETTING_DATA: &[(u8, &'static str)] = &[
    (17, "AF-S"),
    (18, "AF-C"),
    (19, "AF-A"),
    (32, "Manual"),
    (48, "DMF"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FOCUS_MODE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FOCUS_MODE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__focus_mode_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FOCUS_MODE_SETTING.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_643_DATA: &[(u8, &'static str)] = &[
    (1, "No"),
    (16, "Yes"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_643: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_643_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__a_f_button_pressed_643(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_A_F_BUTTON_PRESSED_643.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LIVE_VIEW_METERING_644_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (16, "40 Segment"),
    (32, "1200-zone Evaluative"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LIVE_VIEW_METERING_644: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LIVE_VIEW_METERING_644_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__live_view_metering_644(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LIVE_VIEW_METERING_644.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS3_VIEWING_MODE2_645_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (16, "Viewfinder"),
    (33, "Focus Check Live View"),
    (34, "Quick AF Live View"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_VIEWING_MODE2_645: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_VIEWING_MODE2_645_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__viewing_mode2_645(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_VIEWING_MODE2_645.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_A_E_LOCK_646_DATA: &[(u8, &'static str)] = &[
    (1, "On"),
    (2, "Off"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_A_E_LOCK_646: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_A_E_LOCK_646_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__a_e_lock_646(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_A_E_LOCK_646.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_647_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (2, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_647: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_647_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_status_built_in_647(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_STATUS_BUILT_IN_647.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_648_DATA: &[(u8, &'static str)] = &[
    (1, "None"),
    (2, "Off"),
    (3, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_648: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_648_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__flash_status_external_648(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_FLASH_STATUS_EXTERNAL_648.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_651_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "AF"),
    (16, "Manual"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_651: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_651_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__live_view_focus_mode_651(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_LIVE_VIEW_FOCUS_MODE_651.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS3_METERING_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Multi-segment"),
    (2, "Center-weighted average"),
    (3, "Spot"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_METERING_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_METERING_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__metering_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_METERING_MODE.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS3_SONY_IMAGE_SIZE_DATA: &[(u8, &'static str)] = &[
    (21, "Large (3:2)"),
    (22, "Medium (3:2)"),
    (23, "Small (3:2)"),
    (25, "Large (16:9)"),
    (26, "Medium (16:9)"),
    (27, "Small (16:9)"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS3_SONY_IMAGE_SIZE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS3_SONY_IMAGE_SIZE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings3__sony_image_size(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS3_SONY_IMAGE_SIZE.get(&key).copied()
}
