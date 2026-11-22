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
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_node_modules() {
        let patterns = vec!["node_modules".to_string()];
        let classifier = Classifier::new(&patterns);

        assert!(classifier.is_temporary(Path::new("/project/node_modules")));
        assert!(classifier.is_temporary(Path::new("/project/node_modules/package")));
        assert!(classifier.is_temporary(Path::new("/project/node_modules/package/index.js")));
        assert!(!classifier.is_temporary(Path::new("/project/src/index.js")));
    }

    #[test]
    fn test_wildcards() {
        let patterns = vec!["*.tmp".to_string(), "*.log".to_string()];
        let classifier = Classifier::new(&patterns);

        assert!(classifier.is_temporary(Path::new("/project/file.tmp")));
        assert!(classifier.is_temporary(Path::new("/project/debug.log")));
        assert!(!classifier.is_temporary(Path::new("/project/file.txt")));
    }

    #[test]
    fn test_pycache() {
        let patterns = vec!["__pycache__".to_string(), ".pycache".to_string()];
        let classifier = Classifier::new(&patterns);

        assert!(classifier.is_temporary(Path::new("/project/__pycache__")));
        assert!(classifier.is_temporary(Path::new("/project/__pycache__/module.pyc")));
        assert!(classifier.is_temporary(Path::new("/project/.pycache")));
    }
}
