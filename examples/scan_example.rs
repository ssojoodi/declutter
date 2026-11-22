use declutter::config::Config;
use declutter::scanner::Scanner;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or(".");

    println!("Scanning: {}", path);
    println!("---");

    let config = Config::default();
    let scanner = Scanner::new(config);

    match scanner.scan(&std::path::PathBuf::from(path)) {
        Ok(tree) => {
            println!("✓ Scan complete!");
            println!();
            println!("Root: {}", tree.path.display());
            println!("Total Size: {}", format_size(tree.total_size));
            println!("Real Files: {} ({:.1}%)",
                format_size(tree.real_size),
                percentage(tree.real_size, tree.total_size));
            println!("Temp Files: {} ({:.1}%)",
                format_size(tree.temp_size),
                percentage(tree.temp_size, tree.total_size));
            println!("Type: {:?}", tree.file_type);
            println!();

            if let Some(children) = &tree.children {
                println!("Top-level items ({}):", children.len());
                for (i, child) in children.iter().take(10).enumerate() {
                    let type_indicator = match child.file_type {
                        declutter::models::FileType::Real => "📄",
                        declutter::models::FileType::Temporary => "🗑️ ",
                        declutter::models::FileType::Mixed => "📦",
                    };
                    println!("  {}. {} {} - {}",
                        i + 1,
                        type_indicator,
                        child.name(),
                        format_size(child.total_size));
                }
                if children.len() > 10 {
                    println!("  ... and {} more", children.len() - 10);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Error scanning: {}", e);
            std::process::exit(1);
        }
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

    format!("{:.2} {}", size, UNITS[unit_idx])
}

fn percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
