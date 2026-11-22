use declutter::config::Config;
use declutter::scanner::Scanner;
use std::path::Path;

#[test]
fn test_scanner_basic() {
    let base = Path::new("tests/fixtures/project");

    // Scan the directory
    let config = Config::default();
    let scanner = Scanner::new(config);
    let result = scanner.scan(base).unwrap();

    // Verify results
    assert!(result.total_size > 0);
    assert!(result.temp_size > 0);
    assert!(result.real_size > 0);

    // Verify temp files were detected
    if let Some(children) = &result.children {
        let has_node_modules = children.iter().any(|c| {
            c.path.file_name().and_then(|n| n.to_str()) == Some("node_modules")
        });
        let has_target = children.iter().any(|c| {
            c.path.file_name().and_then(|n| n.to_str()) == Some("target")
        });

        assert!(has_node_modules, "node_modules should be detected");
        assert!(has_target, "target should be detected");
    }
}

#[test]
fn test_classifier() {
    use declutter::scanner::Classifier;
    use std::path::Path;

    let patterns = vec![
        "node_modules".to_string(),
        "*.tmp".to_string(),
        "__pycache__".to_string(),
    ];

    let classifier = Classifier::new(&patterns);

    // Should classify as temporary
    assert!(classifier.is_temporary(Path::new("/project/node_modules")));
    assert!(classifier.is_temporary(Path::new("/project/node_modules/package/index.js")));
    assert!(classifier.is_temporary(Path::new("/project/file.tmp")));
    assert!(classifier.is_temporary(Path::new("/project/__pycache__/module.pyc")));

    // Should NOT classify as temporary
    assert!(!classifier.is_temporary(Path::new("/project/src/main.rs")));
    assert!(!classifier.is_temporary(Path::new("/project/README.md")));
    assert!(!classifier.is_temporary(Path::new("/project/Cargo.toml")));
}

#[test]
fn test_treemap_algorithm() {
    use declutter::algorithms::squarify;
    use declutter::models::{DirEntry, FileType};
    use ratatui::layout::Rect;
    use std::path::PathBuf;

    // Create test entries
    let entries = vec![
        DirEntry {
            path: PathBuf::from("large"),
            total_size: 750,
            temp_size: 0,
            real_size: 750,
            file_type: FileType::Real,
            children: None,
            is_dir: false,
        },
        DirEntry {
            path: PathBuf::from("medium"),
            total_size: 250,
            temp_size: 0,
            real_size: 250,
            file_type: FileType::Real,
            children: None,
            is_dir: false,
        },
    ];

    let area = Rect::new(0, 0, 100, 100);
    let nodes = squarify(&entries, area);

    assert_eq!(nodes.len(), 2, "Should create 2 nodes");

    // Verify proportionality (roughly)
    let total_area: u32 = nodes.iter()
        .map(|n| n.rect.width as u32 * n.rect.height as u32)
        .sum();

    // Total area should be close to available area
    assert!(total_area >= 9000 && total_area <= 10100,
        "Total area should be close to 10000, got {}", total_area);
}

#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert!(!config.temp_patterns.is_empty());
    assert!(config.temp_patterns.contains(&"node_modules".to_string()));
    assert!(config.temp_patterns.contains(&"target".to_string()));
    assert_eq!(config.min_display_size, 1_048_576); // 1 MB
    assert_eq!(config.performance.max_depth, 10);
}
