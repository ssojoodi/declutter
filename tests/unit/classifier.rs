use super::*;

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
