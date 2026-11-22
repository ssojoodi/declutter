use super::*;
use crate::models::FileType;
use std::path::PathBuf;

fn create_test_entry(name: &str, size: u64) -> DirEntry {
    DirEntry {
        path: PathBuf::from(name),
        total_size: size,
        temp_size: 0,
        real_size: size,
        file_type: FileType::Real,
        children: None,
        is_dir: false,
    }
}

#[test]
fn test_empty_entries() {
    let entries = vec![];
    let area = Rect::new(0, 0, 100, 100);
    let result = squarify(&entries, area);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_single_entry() {
    let entries = vec![create_test_entry("a", 100)];
    let area = Rect::new(0, 0, 100, 100);
    let result = squarify(&entries, area);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].rect.width * result[0].rect.height, 100 * 100);
}

#[test]
fn test_proportional_sizes() {
    let entries = vec![
        create_test_entry("large", 75),
        create_test_entry("small", 25),
    ];
    let area = Rect::new(0, 0, 100, 100);
    let result = squarify(&entries, area);

    assert_eq!(result.len(), 2);

    let total_area: u32 = result
        .iter()
        .map(|n| n.rect.width as u32 * n.rect.height as u32)
        .sum();

    // Total area should approximately equal the available area
    assert!((total_area as i32 - 10000).abs() < 1000);
}
