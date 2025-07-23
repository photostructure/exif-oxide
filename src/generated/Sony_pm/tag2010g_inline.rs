//! Inline PrintConv tables for Tag2010g table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: Tag2010g)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (23 entries)
static TAG2010G_RELEASE_MODE2_DATA: &[(u8, &'static str)] = &[
    (0, "Normal"),
    (1, "Continuous"),
    (2, "Continuous - Exposure Bracketing"),
    (3, "DRO or White Balance Bracketing"),
    (5, "Continuous - Burst"),
    (6, "Single Frame - Capture During Movie"),
    (7, "Continuous - Sweep Panorama"),
    (8, "Continuous - Anti-Motion Blur, Hand-held Twilight"),
    (9, "Continuous - HDR"),
    (10, "Continuous - Background defocus"),
    (13, "Continuous - 3D Sweep Panorama"),
    (15, "Continuous - High Resolution Sweep Panorama"),
    (16, "Continuous - 3D Image"),
    (17, "Continuous - Burst 2"),
    (18, "Normal - iAuto+"),
    (19, "Continuous - Speed/Advance Priority"),
    (20, "Continuous - Multi Frame NR"),
    (23, "Single-frame - Exposure Bracketing"),
    (26, "Continuous Low"),
    (27, "Continuous - High Sensitivity"),
    (28, "Smile Shutter"),
    (29, "Continuous - Tele-zoom Advance Priority"),
    (146, "Single Frame - Movie Capture"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_RELEASE_MODE2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_RELEASE_MODE2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__release_mode2(key: u8) -> Option<&'static str> {
    TAG2010G_RELEASE_MODE2.get(&key).copied()
}

/// Raw data (7 entries)
static TAG2010G_RELEASE_MODE3_DATA: &[(u8, &'static str)] = &[
    (0, "Normal"),
    (1, "Continuous"),
    (2, "Bracketing"),
    (4, "Continuous - Burst"),
    (5, "Continuous - Speed/Advance Priority"),
    (6, "Normal - Self-timer"),
    (9, "Single Burst Shooting"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_RELEASE_MODE3: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_RELEASE_MODE3_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__release_mode3(key: u8) -> Option<&'static str> {
    TAG2010G_RELEASE_MODE3.get(&key).copied()
}

/// Raw data (23 entries)
static TAG2010G_RELEASE_MODE2_528_DATA: &[(u8, &'static str)] = &[
    (0, "Normal"),
    (1, "Continuous"),
    (2, "Continuous - Exposure Bracketing"),
    (3, "DRO or White Balance Bracketing"),
    (5, "Continuous - Burst"),
    (6, "Single Frame - Capture During Movie"),
    (7, "Continuous - Sweep Panorama"),
    (8, "Continuous - Anti-Motion Blur, Hand-held Twilight"),
    (9, "Continuous - HDR"),
    (10, "Continuous - Background defocus"),
    (13, "Continuous - 3D Sweep Panorama"),
    (15, "Continuous - High Resolution Sweep Panorama"),
    (16, "Continuous - 3D Image"),
    (17, "Continuous - Burst 2"),
    (18, "Normal - iAuto+"),
    (19, "Continuous - Speed/Advance Priority"),
    (20, "Continuous - Multi Frame NR"),
    (23, "Single-frame - Exposure Bracketing"),
    (26, "Continuous Low"),
    (27, "Continuous - High Sensitivity"),
    (28, "Smile Shutter"),
    (29, "Continuous - Tele-zoom Advance Priority"),
    (146, "Single Frame - Movie Capture"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_RELEASE_MODE2_528: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_RELEASE_MODE2_528_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__release_mode2_528(key: u8) -> Option<&'static str> {
    TAG2010G_RELEASE_MODE2_528.get(&key).copied()
}

/// Raw data (3 entries)
static TAG2010G_SELF_TIMER_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "Self-timer 10 s"), (2, "Self-timer 2 s")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_SELF_TIMER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_SELF_TIMER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__self_timer(key: u8) -> Option<&'static str> {
    TAG2010G_SELF_TIMER.get(&key).copied()
}

/// Raw data (6 entries)
static TAG2010G_FLASH_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Autoflash"),
    (1, "Fill-flash"),
    (2, "Flash Off"),
    (3, "Slow Sync"),
    (4, "Rear Sync"),
    (6, "Wireless"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_FLASH_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_FLASH_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__flash_mode(key: u8) -> Option<&'static str> {
    TAG2010G_FLASH_MODE.get(&key).copied()
}

/// Raw data (8 entries)
static TAG2010G_DYNAMIC_RANGE_OPTIMIZER_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Auto"),
    (3, "Lv1"),
    (4, "Lv2"),
    (5, "Lv3"),
    (6, "Lv4"),
    (7, "Lv5"),
    (8, "n/a"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_DYNAMIC_RANGE_OPTIMIZER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        TAG2010G_DYNAMIC_RANGE_OPTIMIZER_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_tag2010g__dynamic_range_optimizer(key: u8) -> Option<&'static str> {
    TAG2010G_DYNAMIC_RANGE_OPTIMIZER.get(&key).copied()
}

/// Raw data (8 entries)
static TAG2010G_H_D_R_SETTING_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "HDR Auto"),
    (3, "HDR 1 EV"),
    (5, "HDR 2 EV"),
    (7, "HDR 3 EV"),
    (9, "HDR 4 EV"),
    (11, "HDR 5 EV"),
    (13, "HDR 6 EV"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_H_D_R_SETTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_H_D_R_SETTING_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__h_d_r_setting(key: u8) -> Option<&'static str> {
    TAG2010G_H_D_R_SETTING.get(&key).copied()
}

/// Raw data (14 entries)
static TAG2010G_PICTURE_EFFECT2_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Toy Camera"),
    (2, "Pop Color"),
    (3, "Posterization"),
    (4, "Retro Photo"),
    (5, "Soft High Key"),
    (6, "Partial Color"),
    (7, "High Contrast Monochrome"),
    (8, "Soft Focus"),
    (9, "HDR Painting"),
    (10, "Rich-tone Monochrome"),
    (11, "Miniature"),
    (12, "Water Color"),
    (13, "Illustration"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_PICTURE_EFFECT2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_PICTURE_EFFECT2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__picture_effect2(key: u8) -> Option<&'static str> {
    TAG2010G_PICTURE_EFFECT2.get(&key).copied()
}

/// Raw data (3 entries)
static TAG2010G_QUALITY2_DATA: &[(u8, &'static str)] =
    &[(0, "JPEG"), (1, "RAW"), (2, "RAW + JPEG")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_QUALITY2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_QUALITY2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__quality2(key: u8) -> Option<&'static str> {
    TAG2010G_QUALITY2.get(&key).copied()
}

/// Raw data (5 entries)
static TAG2010G_METERING_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Multi-segment"),
    (2, "Center-weighted average"),
    (3, "Spot"),
    (4, "Average"),
    (5, "Highlight"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_METERING_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_METERING_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__metering_mode(key: u8) -> Option<&'static str> {
    TAG2010G_METERING_MODE.get(&key).copied()
}

/// Raw data (33 entries)
static TAG2010G_EXPOSURE_PROGRAM_DATA: &[(u8, &'static str)] = &[
    (0, "Program AE"),
    (1, "Aperture-priority AE"),
    (2, "Shutter speed priority AE"),
    (3, "Manual"),
    (4, "Auto"),
    (5, "iAuto"),
    (6, "Superior Auto"),
    (7, "iAuto+"),
    (8, "Portrait"),
    (9, "Landscape"),
    (10, "Twilight"),
    (11, "Twilight Portrait"),
    (12, "Sunset"),
    (14, "Action (High speed)"),
    (16, "Sports"),
    (17, "Handheld Night Shot"),
    (18, "Anti Motion Blur"),
    (19, "High Sensitivity"),
    (21, "Beach"),
    (22, "Snow"),
    (23, "Fireworks"),
    (26, "Underwater"),
    (27, "Gourmet"),
    (28, "Pet"),
    (29, "Macro"),
    (30, "Backlight Correction HDR"),
    (33, "Sweep Panorama"),
    (36, "Background Defocus"),
    (37, "Soft Skin"),
    (42, "3D Image"),
    (43, "Cont. Priority AE"),
    (45, "Document"),
    (46, "Party"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_EXPOSURE_PROGRAM: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_EXPOSURE_PROGRAM_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__exposure_program(key: u8) -> Option<&'static str> {
    TAG2010G_EXPOSURE_PROGRAM.get(&key).copied()
}

/// Raw data (3 entries)
static TAG2010G_LENS_FORMAT_DATA: &[(u8, &'static str)] =
    &[(0, "Unknown"), (1, "APS-C"), (2, "Full-frame")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_LENS_FORMAT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_LENS_FORMAT_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__lens_format(key: u8) -> Option<&'static str> {
    TAG2010G_LENS_FORMAT.get(&key).copied()
}

/// Raw data (3 entries)
static TAG2010G_LENS_MOUNT_DATA: &[(u8, &'static str)] =
    &[(0, "Unknown"), (1, "A-mount"), (2, "E-mount")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_LENS_MOUNT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_LENS_MOUNT_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__lens_mount(key: u8) -> Option<&'static str> {
    TAG2010G_LENS_MOUNT.get(&key).copied()
}

/// Raw data (276 entries)
static TAG2010G_LENS_TYPE2_DATA: &[(&'static str, &'static str)] = &[
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
    (
        "33077",
        "Sony FE 100-400mm F4.5-5.6 GM OSS + 1.4X Teleconverter",
    ),
    (
        "33078",
        "Sony FE 100-400mm F4.5-5.6 GM OSS + 2X Teleconverter",
    ),
    ("33079", "Sony FE 400mm F2.8 GM OSS + 1.4X Teleconverter"),
    ("33080", "Sony FE 400mm F2.8 GM OSS + 2X Teleconverter"),
    (
        "33081",
        "Sony FE 200-600mm F5.6-6.3 G OSS + 1.4X Teleconverter",
    ),
    (
        "33082",
        "Sony FE 200-600mm F5.6-6.3 G OSS + 2X Teleconverter",
    ),
    ("33083", "Sony FE 600mm F4 GM OSS + 1.4X Teleconverter"),
    ("33084", "Sony FE 600mm F4 GM OSS + 2X Teleconverter"),
    (
        "33085",
        "Sony FE 70-200mm F2.8 GM OSS II + 1.4X Teleconverter",
    ),
    (
        "33086",
        "Sony FE 70-200mm F2.8 GM OSS II + 2X Teleconverter",
    ),
    (
        "33087",
        "Sony FE 70-200mm F4 Macro G OSS II + 1.4X Teleconverter",
    ),
    (
        "33088",
        "Sony FE 70-200mm F4 Macro G OSS II + 2X Teleconverter",
    ),
    ("33089", "Sony FE 300mm F2.8 GM OSS + 1.4X Teleconverter"),
    ("33090", "Sony FE 300mm F2.8 GM OSS + 2X Teleconverter"),
    (
        "33091",
        "Sony FE 400-800mm F6.3-8 G OSS + 1.4X Teleconverter",
    ),
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
    (
        "49473",
        "Tamron 17-50mm F4 Di III VXD or Tokina or Viltrox lens",
    ),
    ("49473.1", "Tokina atx-m 85mm F1.8 FE"),
    ("49473.2", "Viltrox 23mm F1.4 E"),
    ("49473.3", "Viltrox 56mm F1.4 E"),
    ("49473.4", "Viltrox 85mm F1.8 II FE"),
    (
        "49474",
        "Tamron 70-180mm F2.8 Di III VXD G2 or Viltrox lens",
    ),
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
    (
        "50482",
        "Sigma 18-300mm F3.5-6.3 DC MACRO OS HSM | C + MC-11",
    ),
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
    (
        "78",
        "Metabones Canon EF Smart Adapter Mark III or Other Adapter",
    ),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_LENS_TYPE2: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| TAG2010G_LENS_TYPE2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__lens_type2(key: &str) -> Option<&'static str> {
    TAG2010G_LENS_TYPE2.get(key).copied()
}

/// Raw data (2 entries)
static TAG2010G_DISTORTION_CORR_PARAMS_PRESENT_DATA: &[(u8, &'static str)] =
    &[(0, "No"), (1, "Yes")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_DISTORTION_CORR_PARAMS_PRESENT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        TAG2010G_DISTORTION_CORR_PARAMS_PRESENT_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_tag2010g__distortion_corr_params_present(key: u8) -> Option<&'static str> {
    TAG2010G_DISTORTION_CORR_PARAMS_PRESENT.get(&key).copied()
}

/// Raw data (2 entries)
static TAG2010G_DISTORTION_CORR_PARAMS_NUMBER_DATA: &[(u8, &'static str)] =
    &[(11, "11 (APS-C)"), (16, "16 (Full-frame)")];

/// Lookup table (lazy-initialized)
pub static TAG2010G_DISTORTION_CORR_PARAMS_NUMBER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        TAG2010G_DISTORTION_CORR_PARAMS_NUMBER_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_tag2010g__distortion_corr_params_number(key: u8) -> Option<&'static str> {
    TAG2010G_DISTORTION_CORR_PARAMS_NUMBER.get(&key).copied()
}

/// Raw data (5 entries)
static TAG2010G_ASPECT_RATIO_DATA: &[(u8, &'static str)] = &[
    (0, "16:9"),
    (1, "4:3"),
    (2, "3:2"),
    (3, "1:1"),
    (5, "Panorama"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_ASPECT_RATIO: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TAG2010G_ASPECT_RATIO_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_tag2010g__aspect_ratio(key: u8) -> Option<&'static str> {
    TAG2010G_ASPECT_RATIO.get(&key).copied()
}

/// Raw data (8 entries)
static TAG2010G_DYNAMIC_RANGE_OPTIMIZER_80_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Auto"),
    (3, "Lv1"),
    (4, "Lv2"),
    (5, "Lv3"),
    (6, "Lv4"),
    (7, "Lv5"),
    (8, "n/a"),
];

/// Lookup table (lazy-initialized)
pub static TAG2010G_DYNAMIC_RANGE_OPTIMIZER_80: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        TAG2010G_DYNAMIC_RANGE_OPTIMIZER_80_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_tag2010g__dynamic_range_optimizer_80(key: u8) -> Option<&'static str> {
    TAG2010G_DYNAMIC_RANGE_OPTIMIZER_80.get(&key).copied()
}
