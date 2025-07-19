//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! FujiFilm model detection patterns from Main table
//! ExifTool: FujiFilm.pm %FujiFilm::Main
//! Generated at: Sat Jul 19 20:59:47 2025 GMT

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ConditionalTagEntry {
    pub condition: &'static str,
    pub name: &'static str,
}

/// Conditional tag resolution mapping
static CONDITIONAL_TAG_RESOLVER: LazyLock<HashMap<&'static str, Vec<ConditionalTagEntry>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert(
            "4352",
            vec![ConditionalTagEntry {
                condition: "$$self{Model} eq \"X-T3\"",
                name: "AutoBracketing",
            }],
        );
        map.insert(
            "4868",
            vec![ConditionalTagEntry {
                condition: "$$self{Make} =~ /^GENERAL IMAGING/",
                name: "GEImageSize",
            }],
        );
        map
    });

/// FujiFilm model detection and conditional tag resolution
/// Patterns: 0, Conditional tags: 2
#[derive(Debug, Clone)]
pub struct FujiFilmModelDetection {
    /// Model string for pattern matching
    pub model: String,
}

impl FujiFilmModelDetection {
    /// Create new model detection instance
    pub fn new(model: String) -> Self {
        Self { model }
    }

    /// Resolve conditional tag based on model and other conditions
    pub fn resolve_conditional_tag(
        &self,
        tag_id: &str,
        context: &ConditionalContext,
    ) -> Option<&'static str> {
        CONDITIONAL_TAG_RESOLVER
            .get(tag_id)?
            .iter()
            .find(|condition| self.evaluate_condition(condition.condition, context))
            .map(|condition| condition.name)
    }

    /// Evaluate a single condition against the current context
    fn evaluate_condition(&self, condition: &str, context: &ConditionalContext) -> bool {
        // Simplified condition evaluation - can be enhanced
        if condition.contains("$$self{Model}") {
            return self.evaluate_model_condition(condition);
        }
        if condition.contains("$$self{Make}") {
            if let Some(make) = &context.make {
                return self.evaluate_make_condition(condition, make);
            }
        }
        false
    }

    /// Evaluate model-specific conditions
    fn evaluate_model_condition(&self, condition: &str) -> bool {
        // Simple string matching for now - can be enhanced with regex
        if condition.contains(" eq ") {
            if let Some(quoted) = extract_quoted_string(condition) {
                return self.model == quoted;
            }
        }
        if condition.contains(" ne ") {
            if let Some(quoted) = extract_quoted_string(condition) {
                return self.model != quoted;
            }
        }
        // TODO: Implement regex matching for =~ patterns
        false
    }

    /// Evaluate make-specific conditions
    fn evaluate_make_condition(&self, condition: &str, make: &str) -> bool {
        if condition.contains(" =~ ") {
            // Simple substring matching for now
            if condition.contains("/^GENERAL IMAGING/") {
                return make.starts_with("GENERAL IMAGING");
            }
        }
        false
    }
}

/// Context for evaluating conditional tag conditions
#[derive(Debug, Clone)]
pub struct ConditionalContext {
    pub make: Option<String>,
    pub count: Option<u32>,
    pub format: Option<String>,
}

/// Extract quoted string from Perl condition
fn extract_quoted_string(condition: &str) -> Option<String> {
    if let Some(start) = condition.find('"') {
        if let Some(end) = condition[start + 1..].find('"') {
            return Some(condition[start + 1..start + 1 + end].to_string());
        }
    }
    None
}
