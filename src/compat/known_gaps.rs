//! Known-gap allowlist for the ExifTool compatibility oracle.
//!
//! `test_exiftool_compatibility` (`tests/exiftool_compatibility_tests.rs`) is a
//! hard gate: any tag whose value diverges from the committed ExifTool snapshot
//! must be listed here with a `reason` and a `reference` (a `_todo/…` TPP path or
//! an ExifTool source citation), or the test fails. Conversely, if an allowlisted
//! tag starts matching ExifTool, the test fails so the entry gets removed — the
//! ratchet that keeps this file from becoming a silent dumping ground.
//!
//! The backing data lives in `config/compat_known_gaps.json`.

use serde::Deserialize;
use std::collections::HashMap;

/// One group of allowlisted tags sharing a common root cause.
#[derive(Debug, Clone, Deserialize)]
pub struct KnownGapGroup {
    /// Why these tags legitimately diverge (video read support not implemented,
    /// binary extraction pending, MakerNotes gaps, etc.). Must be non-empty.
    pub reason: String,
    /// Where the gap is tracked: a `_todo/…` TPP path or an ExifTool `file:line`
    /// citation. Must be non-empty.
    pub reference: String,
    /// The `Group:Tag` names covered by this group.
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct KnownGapsConfig {
    groups: Vec<KnownGapGroup>,
}

/// Parsed, validated allowlist plus a flattened `tag -> group-index` map.
#[derive(Debug, Clone)]
pub struct KnownGaps {
    groups: Vec<KnownGapGroup>,
    tag_to_group: HashMap<String, usize>,
}

impl KnownGaps {
    /// Parse and validate an allowlist from JSON text.
    ///
    /// Errors on: an empty `reason` or `reference` (after trimming), and any tag
    /// listed in more than one group (a config mistake that hides which reason
    /// actually applies).
    pub fn parse(json: &str) -> Result<Self, String> {
        let config: KnownGapsConfig = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse compat_known_gaps.json: {e}"))?;

        let mut tag_to_group: HashMap<String, usize> = HashMap::new();
        for (idx, group) in config.groups.iter().enumerate() {
            if group.reason.trim().is_empty() {
                return Err(format!(
                    "known-gaps group #{idx} (reference {:?}) has an empty `reason`",
                    group.reference
                ));
            }
            if group.reference.trim().is_empty() {
                return Err(format!(
                    "known-gaps group #{idx} (reason {:?}) has an empty `reference`",
                    group.reason
                ));
            }
            for tag in &group.tags {
                if let Some(prev) = tag_to_group.insert(tag.clone(), idx) {
                    return Err(format!(
                        "tag {tag:?} appears in two known-gaps groups (#{prev} and #{idx}); \
                         each tag must belong to exactly one group"
                    ));
                }
            }
        }

        Ok(Self {
            groups: config.groups,
            tag_to_group,
        })
    }

    /// Load and validate `config/compat_known_gaps.json` from the crate root.
    pub fn load() -> Result<Self, String> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/config/compat_known_gaps.json");
        let json =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
        Self::parse(&json)
    }

    /// True if `tag` is on the allowlist.
    pub fn contains(&self, tag: &str) -> bool {
        self.tag_to_group.contains_key(tag)
    }

    /// Every allowlisted tag name.
    pub fn tags(&self) -> impl Iterator<Item = &str> {
        self.tag_to_group.keys().map(String::as_str)
    }

    /// The group a tag belongs to, if any.
    pub fn group_for(&self, tag: &str) -> Option<&KnownGapGroup> {
        self.tag_to_group.get(tag).map(|&idx| &self.groups[idx])
    }

    /// All groups, in declaration order.
    pub fn groups(&self) -> &[KnownGapGroup] {
        &self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_happy_path_and_flattens_tags() {
        let json = r#"{
            "groups": [
                {
                    "reason": "video read support not implemented",
                    "reference": "_todo/20260701-P0-strategic-review-program.md item #5",
                    "tags": ["QuickTime:Make", "QuickTime:Model"]
                },
                {
                    "reason": "binary extraction pending",
                    "reference": "_todo/P1-BINARY-EXTRACTION-ALL-FORMATS.md",
                    "tags": ["EXIF:JpgFromRaw"]
                }
            ]
        }"#;

        let gaps = KnownGaps::parse(json).expect("should parse");
        assert!(gaps.contains("QuickTime:Make"));
        assert!(gaps.contains("QuickTime:Model"));
        assert!(gaps.contains("EXIF:JpgFromRaw"));
        assert!(!gaps.contains("EXIF:Make"));
        assert_eq!(gaps.groups().len(), 2);
        assert_eq!(gaps.tags().count(), 3);
        assert_eq!(
            gaps.group_for("EXIF:JpgFromRaw").unwrap().reason,
            "binary extraction pending"
        );
    }

    #[test]
    fn rejects_duplicate_tag_across_groups() {
        let json = r#"{
            "groups": [
                {
                    "reason": "reason one",
                    "reference": "ref one",
                    "tags": ["EXIF:PreviewImage"]
                },
                {
                    "reason": "reason two",
                    "reference": "ref two",
                    "tags": ["EXIF:PreviewImage"]
                }
            ]
        }"#;

        let err = KnownGaps::parse(json).expect_err("duplicate tag must be rejected");
        assert!(
            err.contains("EXIF:PreviewImage") && err.contains("two known-gaps groups"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn rejects_empty_reason() {
        let json = r#"{
            "groups": [
                { "reason": "   ", "reference": "some ref", "tags": ["EXIF:Foo"] }
            ]
        }"#;

        let err = KnownGaps::parse(json).expect_err("empty reason must be rejected");
        assert!(err.contains("empty `reason`"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_empty_reference() {
        let json = r#"{
            "groups": [
                { "reason": "some reason", "reference": "", "tags": ["EXIF:Foo"] }
            ]
        }"#;

        let err = KnownGaps::parse(json).expect_err("empty reference must be rejected");
        assert!(err.contains("empty `reference`"), "unexpected error: {err}");
    }
}
