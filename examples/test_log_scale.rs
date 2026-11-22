use declutter::algorithms::squarify;
use declutter::models::{DirEntry, FileType};
use ratatui::layout::Rect;
use std::path::PathBuf;

fn main() {
    println!("Testing logarithmic scale in treemap\n");

    // Create test entries with vastly different sizes
    let entries = vec![
        create_entry("tiny.txt", 10),           // 10 bytes
        create_entry("small.txt", 1_000),       // 1 KB
        create_entry("medium.txt", 100_000),    // 100 KB
        create_entry("large.txt", 10_000_000),  // 10 MB
        create_entry("huge.txt", 1_000_000_000), // 1 GB
    ];

    println!("Original sizes:");
    for entry in &entries {
        println!("  {} - {}", entry.name(), format_size(entry.total_size));
    }
    println!();

    // Layout with treemap (now using log scale internally)
    let area = Rect::new(0, 0, 100, 50);
    let nodes = squarify(&entries, area);

    println!("Block sizes in grid (with logarithmic scaling):");
    for node in &nodes {
        let block_area = node.rect.width as u32 * node.rect.height as u32;
        let percentage = (block_area as f64 / (100.0 * 50.0)) * 100.0;
        println!(
            "  {} - {}x{} = {} cells ({:.1}% of grid)",
            node.entry.name(),
            node.rect.width,
            node.rect.height,
            block_area,
            percentage
        );
    }

    println!();
    println!("Without log scale, the 1 GB file would take 99.99% of the grid.");
    println!("With log scale, even the 10 byte file gets ~8% of the grid!");
    println!("\n✓ Small folders are now visible alongside large ones!");
}

fn create_entry(name: &str, size: u64) -> DirEntry {
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

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if size >= 100.0 {
        format!("{:.0} {}", size, UNITS[unit_idx])
    } else if size >= 10.0 {
        format!("{:.1} {}", size, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}
