//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! FujiFilm model detection patterns from Main table
//! ExifTool: FujiFilm.pm %FujiFilm::Main

use crate::expressions::ExpressionEvaluator;
use crate::processor_registry::ProcessorContext;
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
            .find(|condition| self.evaluate_condition(&condition.condition, context))
            .map(|condition| condition.name)
    }

    /// Evaluate a condition using the unified expression system
    fn evaluate_condition(&self, condition: &str, context: &ConditionalContext) -> bool {
        let mut evaluator = ExpressionEvaluator::new();

        // Build ProcessorContext from ConditionalContext
        let mut processor_context = ProcessorContext::default();
        if let Some(model) = &context.model {
            processor_context = processor_context.with_model(model.clone());
        }
        if let Some(make) = &context.make {
            processor_context = processor_context.with_manufacturer(make.clone());
        }

        // Try context-based evaluation
        evaluator
            .evaluate_context_condition(&processor_context, condition)
            .unwrap_or(false)
    }
}

/// Context for evaluating conditional tag conditions
#[derive(Debug, Clone)]
pub struct ConditionalContext {
    pub make: Option<String>,
    pub model: Option<String>,
    pub count: Option<u32>,
    pub format: Option<String>,
}
