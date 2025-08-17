impl ValueConvEvaluator {
    pub fn new() -> Self {
        Self {
            strategy_cache: HashMap::new(),
        }
    }

    /// Stub implementation while composite tags are not generated
    pub fn evaluate_composite(
        &mut self,
        _composite_def: &str,
        _resolved_dependencies: &HashMap<String, TagValue>,
    ) -> Option<TagValue> {
        None
    }
}

impl Default for ValueConvEvaluator {
    fn default() -> Self {
        Self::new()
    }
}