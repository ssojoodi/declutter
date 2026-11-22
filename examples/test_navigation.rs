use declutter::app::{App, Command};
use declutter::config::Config;
use declutter::events::Message;
use declutter::scanner::Scanner;
use std::path::PathBuf;

fn main() {
    let test_dir = PathBuf::from("tests/fixtures");

    println!("Testing navigation in: {}", test_dir.display());
    println!("===\n");

    // Setup
    let config = Config::default();
    let scanner = Scanner::new(config.clone());

    // Scan the test directory
    let tree = scanner.scan(&test_dir).expect("Failed to scan");

    // Create app
    let mut app = App::new(config, test_dir.clone());
    let _ = app.update(Message::ScanComplete(tree));

    println!("1. Initial state - viewing root:");
    println!("   Current dir: {}", app.current_dir.display());
    if let Some(tree) = app.get_current_tree() {
        println!("   Items shown: {}", tree.children.as_ref().map(|c| c.len()).unwrap_or(0));
        if let Some(children) = &tree.children {
            for child in children.iter().take(5) {
                println!("     - {}", child.name());
            }
        }
    }
    println!();

    // Simulate entering project1
    println!("2. Pressing Enter on 'project1':");

    // First, select project1 (it should be the first item)
    if let Some(tree) = app.get_current_tree() {
        if let Some(children) = &tree.children {
            if let Some(first) = children.first() {
                println!("   Selecting: {}", first.name());
                app.selection.select(first.path.clone(), 0);
            }
        }
    }

    // Now enter the directory
    let _ = app.update(Message::EnterDirectory);

    println!("   Current dir: {}", app.current_dir.display());
    if let Some(tree) = app.get_current_tree() {
        println!("   Items shown: {}", tree.children.as_ref().map(|c| c.len()).unwrap_or(0));
        if let Some(children) = &tree.children {
            for child in children.iter().take(10) {
                let type_str = if child.is_dir { "📁" } else { "📄" };
                println!("     {} {} ({})", type_str, child.name(), format_size(child.total_size));
            }
        }
    }
    println!();

    // Go back up
    println!("3. Pressing Backspace to go back:");
    let _ = app.update(Message::GoUp);

    println!("   Current dir: {}", app.current_dir.display());
    if let Some(tree) = app.get_current_tree() {
        println!("   Items shown: {}", tree.children.as_ref().map(|c| c.len()).unwrap_or(0));
        if let Some(children) = &tree.children {
            for child in children.iter().take(5) {
                println!("     - {}", child.name());
            }
        }
    }

    println!("\n✓ Navigation test complete!");
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.0}{}", size, UNITS[unit_idx])
}
