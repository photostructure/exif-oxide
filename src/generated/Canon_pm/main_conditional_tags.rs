//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon conditional tag definitions from Main table
//! ExifTool: Canon.pm %Canon::Main
use std::collections::HashMap;
use std::sync::LazyLock;

/// Context for evaluating conditional tag conditions
#[derive(Debug, Clone)]
pub struct ConditionalContext {
    pub model: Option<String>,
    pub make: Option<String>,
    pub count: Option<u32>,
    pub format: Option<String>,
    pub binary_data: Option<Vec<u8>>,
}

/// Resolved tag information
#[derive(Debug, Clone)]
pub struct ResolvedTag {
    pub name: String,
    pub subdirectory: bool,
    pub writable: bool,
    pub format: Option<String>,
}

/// Conditional entry for resolution
#[derive(Debug, Clone)]
pub struct ConditionalEntry {
    pub condition: &'static str,
    pub name: &'static str,
    pub subdirectory: bool,
    pub writable: bool,
    pub format: Option<&'static str>,
}

/// Conditional array resolution mapping
static CONDITIONAL_ARRAYS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(
    || {
        let mut map = HashMap::new();
        map.insert(
            "12",
            vec![
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS D30\\b/",
                    name: "SerialNumber",
                    subdirectory: false,
                    writable: true,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS-1D/",
                    name: "SerialNumber",
                    subdirectory: false,
                    writable: true,
                    format: None,
                },
            ],
        );
        map.insert("13", vec![
        ConditionalEntry {
            condition: "($$self{CameraInfoCount} = $count) and $$self{Model} =~ /\\b1DS?$/",
            name: "CanonCameraInfo1D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b1Ds? Mark II$/",
            name: "CanonCameraInfo1DmkII",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b1Ds? Mark II N$/",
            name: "CanonCameraInfo1DmkIIN",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b1Ds? Mark III$/",
            name: "CanonCameraInfo1DmkIII",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b1D Mark IV$/",
            name: "CanonCameraInfo1DmkIV",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS-1D X$/",
            name: "CanonCameraInfo1DX",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 5D$/",
            name: "CanonCameraInfo5D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 5D Mark II$/",
            name: "CanonCameraInfo5DmkII",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 5D Mark III$/",
            name: "CanonCameraInfo5DmkIII",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 6D$/",
            name: "CanonCameraInfo6D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 7D$/",
            name: "CanonCameraInfo7D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 40D$/",
            name: "CanonCameraInfo40D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 50D$/",
            name: "CanonCameraInfo50D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 60D$/",
            name: "CanonCameraInfo60D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 70D$/",
            name: "CanonCameraInfo70D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /EOS 80D$/",
            name: "CanonCameraInfo80D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(450D|REBEL XSi|Kiss X2)\\b/",
            name: "CanonCameraInfo450D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(500D|REBEL T1i|Kiss X3)\\b/",
            name: "CanonCameraInfo500D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(550D|REBEL T2i|Kiss X4)\\b/",
            name: "CanonCameraInfo550D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(600D|REBEL T3i|Kiss X5)\\b/",
            name: "CanonCameraInfo600D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(650D|REBEL T4i|Kiss X6i)\\b/",
            name: "CanonCameraInfo650D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(700D|REBEL T5i|Kiss X7i)\\b/",
            name: "CanonCameraInfo700D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(750D|Rebel T6i|Kiss X8i)\\b/",
            name: "CanonCameraInfo750D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(760D|Rebel T6s|8000D)\\b/",
            name: "CanonCameraInfo760D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(1000D|REBEL XS|Kiss F)\\b/",
            name: "CanonCameraInfo1000D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(1100D|REBEL T3|Kiss X50)\\b/",
            name: "CanonCameraInfo1100D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\b(1200D|REBEL T5|Kiss X70)\\b/",
            name: "CanonCameraInfo1200D",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\bEOS R[56]$/",
            name: "CanonCameraInfoR6",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\bEOS (R6m2|R8|R50)$/",
            name: "CanonCameraInfoR6m2",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$self{Model} =~ /\\bG5 X Mark II$/",
            name: "CanonCameraInfoG5XII",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$format eq \"int32u\" and ($count == 138 or $count == 148)",
            name: "CanonCameraInfoPowerShot",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "\n                $format eq \"int32u\" and ($count == 156 or $count == 162 or\n                $count == 167 or $count == 171 or $count == 264)\n            ",
            name: "CanonCameraInfoPowerShot2",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$format =~ /^int32/",
            name: "CanonCameraInfoUnknown32",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$format =~ /^int16/",
            name: "CanonCameraInfoUnknown16",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map.insert(
            "15",
            vec![
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS-1D/",
                    name: "CustomFunctions1D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS 5D/",
                    name: "CustomFunctions5D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS 10D/",
                    name: "CustomFunctions10D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS 20D/",
                    name: "CustomFunctions20D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS 30D/",
                    name: "CustomFunctions30D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /\\b(350D|REBEL XT|Kiss Digital N)\\b/",
                    name: "CustomFunctions350D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /\\b(400D|REBEL XTi|Kiss Digital X|K236)\\b/",
                    name: "CustomFunctions400D",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS D30\\b/",
                    name: "CustomFunctionsD30",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
                ConditionalEntry {
                    condition: "$$self{Model} =~ /EOS D60\\b/",
                    name: "CustomFunctionsD60",
                    subdirectory: true,
                    writable: false,
                    format: None,
                },
            ],
        );
        map.insert(
            "150",
            vec![ConditionalEntry {
                condition: "$$self{Model} =~ /EOS 5D/",
                name: "SerialInfo",
                subdirectory: true,
                writable: false,
                format: None,
            }],
        );
        map.insert("16385", vec![
        ConditionalEntry {
            condition: "$count == 582",
            name: "ColorData1",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 653",
            name: "ColorData2",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 796",
            name: "ColorData3",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "\n                $count == 692  or $count == 674  or $count == 702 or\n                $count == 1227 or $count == 1250 or $count == 1251 or\n                $count == 1337 or $count == 1338 or $count == 1346\n            ",
            name: "ColorData4",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 5120",
            name: "ColorData5",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1273 or $count == 1275",
            name: "ColorData6",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1312 or $count == 1313 or $count == 1316 or\n                          $count == 1506",
            name: "ColorData7",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1560 or $count == 1592 or $count == 1353 or $count == 1602",
            name: "ColorData8",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1816 or $count == 1820 or $count == 1824",
            name: "ColorData9",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 2024 or $count == 3656",
            name: "ColorData10",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 3973 or $count == 3778",
            name: "ColorData11",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 4528",
            name: "ColorData12",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map.insert("16405", vec![
        ConditionalEntry {
            condition: "$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/",
            name: "VignettingCorr",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$valPt =~ /^[\\x01\\x02\\x10\\x20]/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x02\\x50\\x7c\\x04)/",
            name: "VignettingCorrUnknown1",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$valPt !~ /^\\0\\0\\0\\0/",
            name: "VignettingCorrUnknown2",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map
    },
);

/// Count-based condition resolution mapping
static COUNT_CONDITIONS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(
    || {
        let mut map = HashMap::new();
        map.insert("13", vec![
        ConditionalEntry {
            condition: "$format eq \"int32u\" and ($count == 138 or $count == 148)",
            name: "CanonCameraInfoPowerShot",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "\n                $format eq \"int32u\" and ($count == 156 or $count == 162 or\n                $count == 167 or $count == 171 or $count == 264)\n            ",
            name: "CanonCameraInfoPowerShot2",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map.insert("16385", vec![
        ConditionalEntry {
            condition: "$count == 582",
            name: "ColorData1",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 653",
            name: "ColorData2",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 796",
            name: "ColorData3",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "\n                $count == 692  or $count == 674  or $count == 702 or\n                $count == 1227 or $count == 1250 or $count == 1251 or\n                $count == 1337 or $count == 1338 or $count == 1346\n            ",
            name: "ColorData4",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 5120",
            name: "ColorData5",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1273 or $count == 1275",
            name: "ColorData6",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1312 or $count == 1313 or $count == 1316 or\n                          $count == 1506",
            name: "ColorData7",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1560 or $count == 1592 or $count == 1353 or $count == 1602",
            name: "ColorData8",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 1816 or $count == 1820 or $count == 1824",
            name: "ColorData9",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 2024 or $count == 3656",
            name: "ColorData10",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 3973 or $count == 3778",
            name: "ColorData11",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$count == 4528",
            name: "ColorData12",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map.insert(
            "56",
            vec![ConditionalEntry {
                condition: "$count == 76",
                name: "BatteryType",
                subdirectory: false,
                writable: true,
                format: None,
            }],
        );
        map
    },
);

/// Binary pattern condition resolution mapping
static BINARY_PATTERNS: LazyLock<HashMap<&'static str, Vec<ConditionalEntry>>> = LazyLock::new(
    || {
        let mut map = HashMap::new();
        map.insert(
            "39",
            vec![ConditionalEntry {
                condition: "$$valPt =~ /^\\x0a\\0/",
                name: "ContrastInfo",
                subdirectory: false,
                writable: false,
                format: None,
            }],
        );
        map.insert("16405", vec![
        ConditionalEntry {
            condition: "$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/",
            name: "VignettingCorr",
            subdirectory: true,
            writable: false,
            format: None,
        },
        ConditionalEntry {
            condition: "$$valPt =~ /^[\\x01\\x02\\x10\\x20]/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x02\\x50\\x7c\\x04)/",
            name: "VignettingCorrUnknown1",
            subdirectory: true,
            writable: false,
            format: None,
        },
    ]);
        map.insert(
            "35",
            vec![ConditionalEntry {
                condition: "$$valPt =~ /^\\x08\\0\\0\\0/",
                name: "Categories",
                subdirectory: false,
                writable: true,
                format: Some("int32u"),
            }],
        );
        map
    },
);

/// Canon conditional tag resolution engine
/// Arrays: 6, Count: 15, Binary: 4, Format: 2, Dependencies: 45
#[derive(Debug, Clone)]
pub struct CanonConditionalTags {}

impl Default for CanonConditionalTags {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonConditionalTags {
    /// Create new conditional tag processor
    pub fn new() -> Self {
        Self {}
    }

    /// Resolve conditional tag based on context
    pub fn resolve_tag(&self, tag_id: &str, context: &ConditionalContext) -> Option<ResolvedTag> {
        // Try conditional arrays first
        if let Some(resolved) = self.resolve_conditional_array(tag_id, context) {
            return Some(resolved);
        }

        // Try count-based conditions
        if let Some(resolved) = self.resolve_count_condition(tag_id, context) {
            return Some(resolved);
        }

        // Try binary pattern matching
        if let Some(resolved) = self.resolve_binary_pattern(tag_id, context) {
            return Some(resolved);
        }

        None
    }

    /// Resolve using conditional arrays
    fn resolve_conditional_array(
        &self,
        tag_id: &str,
        context: &ConditionalContext,
    ) -> Option<ResolvedTag> {
        CONDITIONAL_ARRAYS
            .get(tag_id)?
            .iter()
            .find(|entry| self.evaluate_condition(entry.condition, context))
            .map(|entry| ResolvedTag {
                name: entry.name.to_string(),
                subdirectory: entry.subdirectory,
                writable: entry.writable,
                format: entry.format.map(|s| s.to_string()),
            })
    }

    /// Resolve using count conditions
    fn resolve_count_condition(
        &self,
        tag_id: &str,
        context: &ConditionalContext,
    ) -> Option<ResolvedTag> {
        COUNT_CONDITIONS
            .get(tag_id)?
            .iter()
            .find(|entry| self.evaluate_count_condition(entry.condition, context.count))
            .map(|entry| ResolvedTag {
                name: entry.name.to_string(),
                subdirectory: entry.subdirectory,
                writable: entry.writable,
                format: entry.format.map(|s| s.to_string()),
            })
    }

    /// Resolve using binary pattern matching
    fn resolve_binary_pattern(
        &self,
        tag_id: &str,
        context: &ConditionalContext,
    ) -> Option<ResolvedTag> {
        if let Some(binary_data) = &context.binary_data {
            BINARY_PATTERNS
                .get(tag_id)?
                .iter()
                .find(|entry| self.evaluate_binary_pattern(entry.condition, binary_data))
                .map(|entry| ResolvedTag {
                    name: entry.name.to_string(),
                    subdirectory: entry.subdirectory,
                    writable: entry.writable,
                    format: entry.format.map(|s| s.to_string()),
                })
        } else {
            None
        }
    }

    /// Evaluate a general condition
    fn evaluate_condition(&self, condition: &str, context: &ConditionalContext) -> bool {
        // Model conditions
        if condition.contains("$$self{Model}") {
            if let Some(model) = &context.model {
                return self.evaluate_model_condition(condition, model);
            }
        }

        // Count conditions
        if condition.contains("$count") {
            return self.evaluate_count_condition(condition, context.count);
        }

        // Format conditions
        if condition.contains("$format") {
            if let Some(format) = &context.format {
                return self.evaluate_format_condition(condition, format);
            }
        }

        false
    }

    /// Evaluate model-specific conditions
    fn evaluate_model_condition(&self, condition: &str, model: &str) -> bool {
        // Simple regex matching for now - can be enhanced
        if condition.contains("=~") {
            // Extract regex pattern and evaluate
            // TODO: Implement full regex pattern evaluation
            return model.contains("EOS"); // Simplified for demo
        }
        false
    }

    /// Evaluate count-based conditions
    fn evaluate_count_condition(&self, condition: &str, count: Option<u32>) -> bool {
        if let Some(count_val) = count {
            // Parse simple count conditions like '$count == 582'
            if let Some(expected) = extract_count_value(condition) {
                return count_val == expected;
            }
        }
        false
    }

    /// Evaluate format-based conditions
    fn evaluate_format_condition(&self, condition: &str, format: &str) -> bool {
        if condition.contains("eq") {
            if let Some(expected_format) = extract_quoted_string(condition) {
                return format == expected_format;
            }
        }
        false
    }

    /// Evaluate binary pattern conditions
    fn evaluate_binary_pattern(&self, condition: &str, binary_data: &[u8]) -> bool {
        // Simple binary pattern matching
        if condition.contains("$$valPt =~ /^\\0/") {
            return !binary_data.is_empty() && binary_data[0] == 0;
        }
        // TODO: Implement full binary pattern evaluation
        false
    }
}

/// Extract count value from condition string
fn extract_count_value(condition: &str) -> Option<u32> {
    // Simple parser for conditions like '$count == 582'
    if let Some(start) = condition.find("== ") {
        let number_part = &condition[start + 3..];
        if let Some(end) = number_part.find(|c: char| !c.is_ascii_digit()) {
            number_part[..end].parse().ok()
        } else {
            number_part.parse().ok()
        }
    } else {
        None
    }
}

/// Extract quoted string from condition
fn extract_quoted_string(condition: &str) -> Option<String> {
    if let Some(start) = condition.find('"') {
        if let Some(end) = condition[start + 1..].find('"') {
            return Some(condition[start + 1..start + 1 + end].to_string());
        }
    }
    None
}
