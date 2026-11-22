use glob::Pattern;
use std::path::Path;

pub struct Classifier {
    patterns: Vec<Pattern>,
}

impl Classifier {
    pub fn new(pattern_strings: &[String]) -> Self {
        let patterns = pattern_strings
            .iter()
            .filter_map(|s| Pattern::new(s).ok())
            .collect();

        Self { patterns }
    }

    /// Check if a path matches any temporary file pattern
    pub fn is_temporary(&self, path: &Path) -> bool {
        path.ancestors().any(|ancestor| self.matches_name(ancestor))
    }

    pub(super) fn matches_name(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| self.patterns.iter().any(|pattern| pattern.matches(name)))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/classifier.rs"]
mod tests;
