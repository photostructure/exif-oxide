//! Canon camera model identification
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (354 entries)
static CANON_MODEL_ID_DATA: &[(u32, &'static str)] = &[
    (1042, "EOS M50 / Kiss M"),
    (2049, "PowerShot SX740 HS"),
    (2052, "PowerShot G5 X Mark II"),
    (2053, "PowerShot SX70 HS"),
    (2056, "PowerShot G7 X Mark III"),
    (2065, "EOS M6 Mark II"),
    (2066, "EOS M200"),
    (16842752, "PowerShot A30"),
    (
        17039360,
        "PowerShot S300 / Digital IXUS 300 / IXY Digital 300",
    ),
    (17170432, "PowerShot A20"),
    (17301504, "PowerShot A10"),
    (
        17367040,
        "PowerShot S110 / Digital IXUS v / IXY Digital 200",
    ),
    (17825792, "PowerShot G2"),
    (17891328, "PowerShot S40"),
    (17956864, "PowerShot S30"),
    (18022400, "PowerShot A40"),
    (18087936, "EOS D30"),
    (18153472, "PowerShot A100"),
    (
        18219008,
        "PowerShot S200 / Digital IXUS v2 / IXY Digital 200a",
    ),
    (18284544, "PowerShot A200"),
    (
        18350080,
        "PowerShot S330 / Digital IXUS 330 / IXY Digital 300a",
    ),
    (18415616, "PowerShot G3"),
    (18939904, "PowerShot S45"),
    (
        19070976,
        "PowerShot SD100 / Digital IXUS II / IXY Digital 30",
    ),
    (
        19136512,
        "PowerShot S230 / Digital IXUS v3 / IXY Digital 320",
    ),
    (19202048, "PowerShot A70"),
    (19267584, "PowerShot A60"),
    (
        19333120,
        "PowerShot S400 / Digital IXUS 400 / IXY Digital 400",
    ),
    (19464192, "PowerShot G5"),
    (19922944, "PowerShot A300"),
    (19988480, "PowerShot S50"),
    (20185088, "PowerShot A80"),
    (20250624, "PowerShot SD10 / Digital IXUS i / IXY Digital L"),
    (20316160, "PowerShot S1 IS"),
    (20381696, "PowerShot Pro1"),
    (20447232, "PowerShot S70"),
    (20512768, "PowerShot S60"),
    (20971520, "PowerShot G6"),
    (
        21037056,
        "PowerShot S500 / Digital IXUS 500 / IXY Digital 500",
    ),
    (21102592, "PowerShot A75"),
    (
        21233664,
        "PowerShot SD110 / Digital IXUS IIs / IXY Digital 30a",
    ),
    (21299200, "PowerShot A400"),
    (21430272, "PowerShot A310"),
    (21561344, "PowerShot A85"),
    (
        22151168,
        "PowerShot S410 / Digital IXUS 430 / IXY Digital 450",
    ),
    (22216704, "PowerShot A95"),
    (
        22282240,
        "PowerShot SD300 / Digital IXUS 40 / IXY Digital 50",
    ),
    (
        22347776,
        "PowerShot SD200 / Digital IXUS 30 / IXY Digital 40",
    ),
    (22413312, "PowerShot A520"),
    (22478848, "PowerShot A510"),
    (
        22609920,
        "PowerShot SD20 / Digital IXUS i5 / IXY Digital L2",
    ),
    (23330816, "PowerShot S2 IS"),
    (
        23396352,
        "PowerShot SD430 / Digital IXUS Wireless / IXY Digital Wireless",
    ),
    (
        23461888,
        "PowerShot SD500 / Digital IXUS 700 / IXY Digital 600",
    ),
    (23494656, "EOS D60"),
    (
        24117248,
        "PowerShot SD30 / Digital IXUS i Zoom / IXY Digital L3",
    ),
    (24379392, "PowerShot A430"),
    (24444928, "PowerShot A410"),
    (24510464, "PowerShot S80"),
    (24641536, "PowerShot A620"),
    (24707072, "PowerShot A610"),
    (
        25165824,
        "PowerShot SD630 / Digital IXUS 65 / IXY Digital 80",
    ),
    (
        25231360,
        "PowerShot SD450 / Digital IXUS 55 / IXY Digital 60",
    ),
    (25296896, "PowerShot TX1"),
    (
        25624576,
        "PowerShot SD400 / Digital IXUS 50 / IXY Digital 55",
    ),
    (25690112, "PowerShot A420"),
    (
        25755648,
        "PowerShot SD900 / Digital IXUS 900 Ti / IXY Digital 1000",
    ),
    (
        26214400,
        "PowerShot SD550 / Digital IXUS 750 / IXY Digital 700",
    ),
    (26345472, "PowerShot A700"),
    (
        26476544,
        "PowerShot SD700 IS / Digital IXUS 800 IS / IXY Digital 800 IS",
    ),
    (26542080, "PowerShot S3 IS"),
    (26607616, "PowerShot A540"),
    (
        26673152,
        "PowerShot SD600 / Digital IXUS 60 / IXY Digital 70",
    ),
    (26738688, "PowerShot G7"),
    (26804224, "PowerShot A530"),
    (
        33554432,
        "PowerShot SD800 IS / Digital IXUS 850 IS / IXY Digital 900 IS",
    ),
    (
        33619968,
        "PowerShot SD40 / Digital IXUS i7 / IXY Digital L4",
    ),
    (33685504, "PowerShot A710 IS"),
    (33751040, "PowerShot A640"),
    (33816576, "PowerShot A630"),
    (34144256, "PowerShot S5 IS"),
    (34603008, "PowerShot A460"),
    (
        34734080,
        "PowerShot SD850 IS / Digital IXUS 950 IS / IXY Digital 810 IS",
    ),
    (34799616, "PowerShot A570 IS"),
    (34865152, "PowerShot A560"),
    (
        34930688,
        "PowerShot SD750 / Digital IXUS 75 / IXY Digital 90",
    ),
    (
        34996224,
        "PowerShot SD1000 / Digital IXUS 70 / IXY Digital 10",
    ),
    (35127296, "PowerShot A550"),
    (35192832, "PowerShot A450"),
    (35848192, "PowerShot G9"),
    (35913728, "PowerShot A650 IS"),
    (36044800, "PowerShot A720 IS"),
    (36241408, "PowerShot SX100 IS"),
    (
        36700160,
        "PowerShot SD950 IS / Digital IXUS 960 IS / IXY Digital 2000 IS",
    ),
    (
        36765696,
        "PowerShot SD870 IS / Digital IXUS 860 IS / IXY Digital 910 IS",
    ),
    (
        36831232,
        "PowerShot SD890 IS / Digital IXUS 970 IS / IXY Digital 820 IS",
    ),
    (
        37093376,
        "PowerShot SD790 IS / Digital IXUS 90 IS / IXY Digital 95 IS",
    ),
    (
        37158912,
        "PowerShot SD770 IS / Digital IXUS 85 IS / IXY Digital 25 IS",
    ),
    (37224448, "PowerShot A590 IS"),
    (37289984, "PowerShot A580"),
    (37879808, "PowerShot A470"),
    (
        37945344,
        "PowerShot SD1100 IS / Digital IXUS 80 IS / IXY Digital 20 IS",
    ),
    (38141952, "PowerShot SX1 IS"),
    (38207488, "PowerShot SX10 IS"),
    (38273024, "PowerShot A1000 IS"),
    (38338560, "PowerShot G10"),
    (38862848, "PowerShot A2000 IS"),
    (38928384, "PowerShot SX110 IS"),
    (
        38993920,
        "PowerShot SD990 IS / Digital IXUS 980 IS / IXY Digital 3000 IS",
    ),
    (
        39059456,
        "PowerShot SD880 IS / Digital IXUS 870 IS / IXY Digital 920 IS",
    ),
    (39124992, "PowerShot E1"),
    (39190528, "PowerShot D10"),
    (
        39256064,
        "PowerShot SD960 IS / Digital IXUS 110 IS / IXY Digital 510 IS",
    ),
    (39321600, "PowerShot A2100 IS"),
    (39387136, "PowerShot A480"),
    (39845888, "PowerShot SX200 IS"),
    (
        39911424,
        "PowerShot SD970 IS / Digital IXUS 990 IS / IXY Digital 830 IS",
    ),
    (
        39976960,
        "PowerShot SD780 IS / Digital IXUS 100 IS / IXY Digital 210 IS",
    ),
    (40042496, "PowerShot A1100 IS"),
    (
        40108032,
        "PowerShot SD1200 IS / Digital IXUS 95 IS / IXY Digital 110 IS",
    ),
    (40894464, "PowerShot G11"),
    (40960000, "PowerShot SX120 IS"),
    (41025536, "PowerShot S90"),
    (41222144, "PowerShot SX20 IS"),
    (
        41287680,
        "PowerShot SD980 IS / Digital IXUS 200 IS / IXY Digital 930 IS",
    ),
    (
        41353216,
        "PowerShot SD940 IS / Digital IXUS 120 IS / IXY Digital 220 IS",
    ),
    (41943040, "PowerShot A495"),
    (42008576, "PowerShot A490"),
    (42074112, "PowerShot A3100/A3150 IS"),
    (42139648, "PowerShot A3000 IS"),
    (42205184, "PowerShot SD1400 IS / IXUS 130 / IXY 400F"),
    (42270720, "PowerShot SD1300 IS / IXUS 105 / IXY 200F"),
    (42336256, "PowerShot SD3500 IS / IXUS 210 / IXY 10S"),
    (42401792, "PowerShot SX210 IS"),
    (42467328, "PowerShot SD4000 IS / IXUS 300 HS / IXY 30S"),
    (42532864, "PowerShot SD4500 IS / IXUS 1000 HS / IXY 50S"),
    (43122688, "PowerShot G12"),
    (43188224, "PowerShot SX30 IS"),
    (43253760, "PowerShot SX130 IS"),
    (43319296, "PowerShot S95"),
    (43515904, "PowerShot A3300 IS"),
    (43581440, "PowerShot A3200 IS"),
    (50331648, "PowerShot ELPH 500 HS / IXUS 310 HS / IXY 31S"),
    (50397184, "PowerShot Pro90 IS"),
    (50397185, "PowerShot A800"),
    (50462720, "PowerShot ELPH 100 HS / IXUS 115 HS / IXY 210F"),
    (50528256, "PowerShot SX230 HS"),
    (50593792, "PowerShot ELPH 300 HS / IXUS 220 HS / IXY 410F"),
    (50659328, "PowerShot A2200"),
    (50724864, "PowerShot A1200"),
    (50790400, "PowerShot SX220 HS"),
    (50855936, "PowerShot G1 X"),
    (50921472, "PowerShot SX150 IS"),
    (51380224, "PowerShot ELPH 510 HS / IXUS 1100 HS / IXY 51S"),
    (51445760, "PowerShot S100 (new)"),
    (51511296, "PowerShot ELPH 310 HS / IXUS 230 HS / IXY 600F"),
    (51576832, "PowerShot SX40 HS"),
    (51642368, "IXY 32S"),
    (51773440, "PowerShot A1300"),
    (51838976, "PowerShot A810"),
    (51904512, "PowerShot ELPH 320 HS / IXUS 240 HS / IXY 420F"),
    (51970048, "PowerShot ELPH 110 HS / IXUS 125 HS / IXY 220F"),
    (52428800, "PowerShot D20"),
    (52494336, "PowerShot A4000 IS"),
    (52559872, "PowerShot SX260 HS"),
    (52625408, "PowerShot SX240 HS"),
    (52690944, "PowerShot ELPH 530 HS / IXUS 510 HS / IXY 1"),
    (52756480, "PowerShot ELPH 520 HS / IXUS 500 HS / IXY 3"),
    (52822016, "PowerShot A3400 IS"),
    (52887552, "PowerShot A2400 IS"),
    (52953088, "PowerShot A2300"),
    (53608448, "PowerShot S100V"),
    (53673984, "PowerShot G15"),
    (53739520, "PowerShot SX50 HS"),
    (53805056, "PowerShot SX160 IS"),
    (53870592, "PowerShot S110 (new)"),
    (53936128, "PowerShot SX500 IS"),
    (54001664, "PowerShot N"),
    (54067200, "IXUS 245 HS / IXY 430F"),
    (54525952, "PowerShot SX280 HS"),
    (54591488, "PowerShot SX270 HS"),
    (54657024, "PowerShot A3500 IS"),
    (54722560, "PowerShot A2600"),
    (54788096, "PowerShot SX275 HS"),
    (54853632, "PowerShot A1400"),
    (54919168, "PowerShot ELPH 130 IS / IXUS 140 / IXY 110F"),
    (
        54984704,
        "PowerShot ELPH 115/120 IS / IXUS 132/135 / IXY 90F/100F",
    ),
    (55115776, "PowerShot ELPH 330 HS / IXUS 255 HS / IXY 610F"),
    (55640064, "PowerShot A2500"),
    (55836672, "PowerShot G16"),
    (55902208, "PowerShot S120"),
    (55967744, "PowerShot SX170 IS"),
    (56098816, "PowerShot SX510 HS"),
    (56164352, "PowerShot S200 (new)"),
    (56623104, "IXY 620F"),
    (56688640, "PowerShot N100"),
    (56885248, "PowerShot G1 X Mark II"),
    (56950784, "PowerShot D30"),
    (57016320, "PowerShot SX700 HS"),
    (57081856, "PowerShot SX600 HS"),
    (57147392, "PowerShot ELPH 140 IS / IXUS 150 / IXY 130"),
    (57212928, "PowerShot ELPH 135 / IXUS 145 / IXY 120"),
    (57671680, "PowerShot ELPH 340 HS / IXUS 265 HS / IXY 630"),
    (57737216, "PowerShot ELPH 150 IS / IXUS 155 / IXY 140"),
    (57933824, "EOS M3"),
    (57999360, "PowerShot SX60 HS"),
    (58064896, "PowerShot SX520 HS"),
    (58130432, "PowerShot SX400 IS"),
    (58195968, "PowerShot G7 X"),
    (58261504, "PowerShot N2"),
    (58720256, "PowerShot SX530 HS"),
    (58851328, "PowerShot SX710 HS"),
    (58916864, "PowerShot SX610 HS"),
    (58982400, "EOS M10"),
    (59047936, "PowerShot G3 X"),
    (59113472, "PowerShot ELPH 165 HS / IXUS 165 / IXY 160"),
    (59179008, "PowerShot ELPH 160 / IXUS 160"),
    (59244544, "PowerShot ELPH 350 HS / IXUS 275 HS / IXY 640"),
    (59310080, "PowerShot ELPH 170 IS / IXUS 170"),
    (59834368, "PowerShot SX410 IS"),
    (59965440, "PowerShot G9 X"),
    (60030976, "EOS M5"),
    (60096512, "PowerShot G5 X"),
    (60227584, "PowerShot G7 X Mark II"),
    (60293120, "EOS M100"),
    (60358656, "PowerShot ELPH 360 HS / IXUS 285 HS / IXY 650"),
    (67174400, "PowerShot SX540 HS"),
    (67239936, "PowerShot SX420 IS"),
    (67305472, "PowerShot ELPH 190 IS / IXUS 180 / IXY 190"),
    (67371008, "PowerShot G1"),
    (67371009, "PowerShot ELPH 180 IS / IXUS 175 / IXY 180"),
    (67436544, "PowerShot SX720 HS"),
    (67502080, "PowerShot SX620 HS"),
    (67567616, "EOS M6"),
    (68157440, "PowerShot G9 X Mark II"),
    (68485120, "PowerShot ELPH 185 / IXUS 185 / IXY 200"),
    (68550656, "PowerShot SX430 IS"),
    (68616192, "PowerShot SX730 HS"),
    (68681728, "PowerShot G1 X Mark III"),
    (100925440, "PowerShot S100 / Digital IXUS / IXY Digital"),
    (1074255475, "DC19/DC21/DC22"),
    (1074255476, "XH A1"),
    (1074255477, "HV10"),
    (1074255478, "MD130/MD140/MD150/MD160/ZR850"),
    (1074255735, "DC50"),
    (1074255736, "HV20"),
    (1074255737, "DC211"),
    (1074255738, "HG10"),
    (1074255739, "HR10"),
    (1074255741, "MD255/ZR950"),
    (1074255900, "HF11"),
    (1074255992, "HV30"),
    (1074255996, "XH A1S"),
    (1074255998, "DC301/DC310/DC311/DC320/DC330"),
    (1074255999, "FS100"),
    (1074256000, "HF10"),
    (1074256002, "HG20/HG21"),
    (1074256165, "HF21"),
    (1074256166, "HF S11"),
    (1074256248, "HV40"),
    (1074256263, "DC410/DC411/DC420"),
    (1074256264, "FS19/FS20/FS21/FS22/FS200"),
    (1074256265, "HF20/HF200"),
    (1074256266, "HF S10/S100"),
    (1074256526, "HF R10/R16/R17/R18/R100/R106"),
    (1074256527, "HF M30/M31/M36/M300/M306"),
    (1074256528, "HF S20/S21/S200"),
    (1074256530, "FS31/FS36/FS37/FS300/FS305/FS306/FS307"),
    (1074257056, "EOS C300"),
    (1074257321, "HF G25"),
    (1074257844, "XC10"),
    (1074258371, "EOS C200"),
    (2147483649, "EOS-1D"),
    (2147484007, "EOS-1DS"),
    (2147484008, "EOS 10D"),
    (2147484009, "EOS-1D Mark III"),
    (2147484016, "EOS Digital Rebel / 300D / Kiss Digital"),
    (2147484020, "EOS-1D Mark II"),
    (2147484021, "EOS 20D"),
    (2147484022, "EOS Digital Rebel XSi / 450D / Kiss X2"),
    (2147484040, "EOS-1Ds Mark II"),
    (2147484041, "EOS Digital Rebel XT / 350D / Kiss Digital N"),
    (2147484048, "EOS 40D"),
    (2147484179, "EOS 5D"),
    (2147484181, "EOS-1Ds Mark III"),
    (2147484184, "EOS 5D Mark II"),
    (2147484185, "WFT-E1"),
    (2147484210, "EOS-1D Mark II N"),
    (2147484212, "EOS 30D"),
    (2147484214, "EOS Digital Rebel XTi / 400D / Kiss Digital X"),
    (2147484225, "WFT-E2"),
    (2147484230, "WFT-E3"),
    (2147484240, "EOS 7D"),
    (2147484242, "EOS Rebel T1i / 500D / Kiss X3"),
    (2147484244, "EOS Rebel XS / 1000D / Kiss F"),
    (2147484257, "EOS 50D"),
    (2147484265, "EOS-1D X"),
    (2147484272, "EOS Rebel T2i / 550D / Kiss X4"),
    (2147484273, "WFT-E4"),
    (2147484275, "WFT-E5"),
    (2147484289, "EOS-1D Mark IV"),
    (2147484293, "EOS 5D Mark III"),
    (2147484294, "EOS Rebel T3i / 600D / Kiss X5"),
    (2147484295, "EOS 60D"),
    (2147484296, "EOS Rebel T3 / 1100D / Kiss X50"),
    (2147484297, "EOS 7D Mark II"),
    (2147484311, "WFT-E2 II"),
    (2147484312, "WFT-E4 II"),
    (2147484417, "EOS Rebel T4i / 650D / Kiss X6i"),
    (2147484418, "EOS 6D"),
    (2147484452, "EOS-1D C"),
    (2147484453, "EOS 70D"),
    (2147484454, "EOS Rebel T5i / 700D / Kiss X7i"),
    (2147484455, "EOS Rebel T5 / 1200D / Kiss X70 / Hi"),
    (2147484456, "EOS-1D X Mark II"),
    (2147484465, "EOS M"),
    (2147484486, "EOS Rebel SL1 / 100D / Kiss X7"),
    (2147484487, "EOS Rebel T6s / 760D / 8000D"),
    (2147484489, "EOS 5D Mark IV"),
    (2147484496, "EOS 80D"),
    (2147484501, "EOS M2"),
    (2147484546, "EOS 5DS"),
    (2147484563, "EOS Rebel T6i / 750D / Kiss X8i"),
    (2147484673, "EOS 5DS R"),
    (2147484676, "EOS Rebel T6 / 1300D / Kiss X80"),
    (2147484677, "EOS Rebel T7i / 800D / Kiss X9i"),
    (2147484678, "EOS 6D Mark II"),
    (2147484680, "EOS 77D / 9000D"),
    (2147484695, "EOS Rebel SL2 / 200D / Kiss X9"),
    (2147484705, "EOS R5"),
    (2147484706, "EOS Rebel T100 / 4000D / 3000D"),
    (2147484708, "EOS R"),
    (2147484712, "EOS-1D X Mark III"),
    (2147484722, "EOS Rebel T7 / 2000D / 1500D / Kiss X90"),
    (2147484723, "EOS RP"),
    (2147484725, "EOS Rebel T8i / 850D / X10i"),
    (2147484726, "EOS SL3 / 250D / Kiss X10"),
    (2147484727, "EOS 90D"),
    (2147484752, "EOS R3"),
    (2147484755, "EOS R6"),
    (2147484772, "EOS R7"),
    (2147484773, "EOS R10"),
    (2147484775, "PowerShot ZOOM"),
    (2147484776, "EOS M50 Mark II / Kiss M2"),
    (2147484800, "EOS R50"),
    (2147484801, "EOS R6 Mark II"),
    (2147484807, "EOS R8"),
    (2147484817, "PowerShot V10"),
    (2147484821, "EOS R1"),
    (2147484822, "R5 Mark II"),
    (2147484824, "EOS R100"),
    (2147484950, "EOS R50 V"),
    (2147484960, "EOS D2000C"),
    (2147485024, "EOS D6000C"),
];

/// Lookup table (lazy-initialized)
pub static CANON_MODEL_ID: LazyLock<HashMap<u32, &'static str>> =
    LazyLock::new(|| CANON_MODEL_ID_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_canon_model_id(key: u32) -> Option<&'static str> {
    CANON_MODEL_ID.get(&key).copied()
}
