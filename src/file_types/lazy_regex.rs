//! Lazy regex compilation utility for file type detection

use regex::bytes::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::sync::RwLock;

/// A map that lazily compiles regex patterns on first use
pub struct LazyRegexMap {
    /// Static pattern strings
    patterns: HashMap<&'static str, &'static str>,
    /// Compiled regex cache, populated on demand
    compiled: RwLock<HashMap<String, Regex>>,
}

impl LazyRegexMap {
    /// Create a new lazy regex map from pattern data
    pub fn new(patterns: &[(&'static str, &'static str)]) -> Self {
        Self {
            patterns: patterns.iter().cloned().collect(),
            compiled: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a buffer matches a file type's pattern
    /// Compiles the regex on first use and caches it
    pub fn matches(&self, file_type: &str, buffer: &[u8]) -> bool {
        self.get_regex_internal(file_type)
            .map(|regex| regex.is_match(buffer))
            .unwrap_or(false)
    }

    /// Get a compiled regex for a file type (returns cloned regex)
    /// Returns None if the file type doesn't exist or regex compilation fails
    pub fn get_regex(&self, file_type: &str) -> Option<Regex> {
        self.get_regex_internal(file_type)
    }

    /// Internal method renamed to avoid confusion with public API
    fn get_regex_internal(&self, file_type: &str) -> Option<Regex> {
        // Fast path: check if already compiled
        {
            let cache = self.compiled.read().unwrap();
            if let Some(regex) = cache.get(file_type) {
                return Some(regex.clone());
            }
        }

        // Slow path: compile and cache
        let pattern = self.patterns.get(file_type)?;

        let regex = RegexBuilder::new(pattern)
            .unicode(false) // Critical for byte matching
            .build()
            .ok()?;

        // Cache the compiled regex
        {
            let mut cache = self.compiled.write().unwrap();
            cache.insert(file_type.to_string(), regex.clone());
        }

        Some(regex)
    }

    /// Get all file types with patterns
    pub fn file_types(&self) -> Vec<&'static str> {
        self.patterns.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_compilation() {
        let test_patterns = &[("JPEG", r"^\xff\xd8\xff"), ("PNG", r"^\x89PNG\r\n\x1a\n")];

        let map = LazyRegexMap::new(test_patterns);

        // Test JPEG pattern matches
        let jpeg_header = b"\xff\xd8\xff\xe0";
        assert!(map.matches("JPEG", jpeg_header));

        // Test PNG pattern matches
        let png_header = b"\x89PNG\r\n\x1a\n";
        assert!(map.matches("PNG", png_header));

        // Test non-matching data
        assert!(!map.matches("JPEG", b"not a jpeg"));
        assert!(!map.matches("PNG", b"not a png"));

        // Test unknown file type
        assert!(!map.matches("UNKNOWN", b"anything"));

        // Test file types listing
        let types = map.file_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"JPEG"));
        assert!(types.contains(&"PNG"));
    }

    #[test]
    fn test_caching_behavior() {
        let test_patterns = &[("TEST", r"^test")];

        let map = LazyRegexMap::new(test_patterns);

        // First match compiles the regex
        assert!(map.matches("TEST", b"test data"));

        // Second match should use cached regex
        assert!(map.matches("TEST", b"test another"));

        // Check cache contains the compiled regex
        {
            let cache = map.compiled.read().unwrap();
            assert!(cache.contains_key("TEST"));
        }
    }

    #[test]
    fn test_get_regex_method() {
        let test_patterns = &[("JPEG", r"^\xff\xd8\xff")];
        let map = LazyRegexMap::new(test_patterns);

        // Get regex should return the compiled pattern
        let regex = map.get_regex("JPEG").expect("Should find JPEG pattern");
        assert!(regex.is_match(b"\xff\xd8\xff\xe0"));

        // Non-existent pattern should return None
        assert!(map.get_regex("UNKNOWN").is_none());

        // Check that it's cached after first call
        {
            let cache = map.compiled.read().unwrap();
            assert!(cache.contains_key("JPEG"));
        }
    }
}
