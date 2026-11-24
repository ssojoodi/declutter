use super::*;
use crate::scanner::Scanner;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "declutter-test-{}-{}-{}",
            std::process::id(),
            now(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn warm_root_and_subfolder_round_trip_without_walking() {
    let f = Fixture::new();
    let root = &f.0;
    let directory = root.join("cache");
    let sub = root.join("photos");
    fs::create_dir_all(sub.join("empty")).unwrap();
    fs::write(sub.join(OsStr::from_bytes(b"raw\xff")), b"abcd").unwrap();
    fs::write(sub.join("work.tmp"), b"temporary data").unwrap();
    let scanner = Scanner::new(Config::default());
    let expected = scanner.scan(root).unwrap();
    let expected_sub = scanner.scan(&sub).unwrap();
    assert_eq!(
        expected,
        scanner.scan_using_cache(root, &directory, |_| {}).unwrap(),
    );
    fs::write(sub.join("new"), b"added after scan").unwrap();
    for (path, tree) in [(root, expected), (&sub, expected_sub)] {
        let warm = scanner
            .scan_using_cache(path, &directory, |_| panic!("unexpected walk"))
            .unwrap();
        assert_eq!(tree, warm);
    }
}

#[test]
fn expiry_uses_requested_subtree_count_and_scan_completion() {
    let f = Fixture::new();
    let mut cache = Cache::open(&f.0, &Config::default()).unwrap();
    let mut root = DirEntry::new(f.0.clone(), true);
    let scanned = 2 * MAX_AGE;
    for (count, ttl) in [(999, MINUTE), (1000, MINUTE), (1001, MAX_AGE)] {
        let mut sub = DirEntry::new(root.path.join(count.to_string()), true);
        for n in 0..count {
            sub.add_child(DirEntry::new(sub.path.join(n.to_string()), false));
        }
        root.add_child(sub);
        cache.store(&root, scanned).unwrap();
        let path = root.path.join(count.to_string());
        assert!(cache.load(&path, scanned + ttl - 1).unwrap().is_some());
        assert!(cache.load(&path, scanned + ttl).unwrap().is_none());
    }
    assert!(cache.load(&root.path, scanned + MINUTE).unwrap().is_some());
    assert!(cache
        .load(&root.path.join("999"), scanned + MINUTE)
        .unwrap()
        .is_none());
}

#[test]
fn limited_ancestor_and_changed_patterns_require_live_scans() {
    let f = Fixture::new();
    let cache = f.0.join("cache");
    let root = f.0.join("root");
    let sub = root.join("sub");
    fs::create_dir_all(sub.join("deep")).unwrap();
    fs::write(sub.join("deep/data"), b"abc").unwrap();
    let mut config = Config::default();
    config.performance.max_depth = 2;
    let scanner = Scanner::new(config.clone());
    assert_eq!(
        scanner
            .scan_using_cache(&root, &cache, |_| {})
            .unwrap()
            .file_count(),
        0
    );
    let full = scanner.scan_using_cache(&sub, &cache, |_| {}).unwrap();
    assert_eq!(full.real_size, 3);
    config.temp_patterns.push("deep".into());
    assert_eq!(
        Scanner::new(config)
            .scan_using_cache(&sub, &cache, |_| {})
            .unwrap()
            .temp_size,
        3
    );
}

#[test]
fn newest_covering_scan_wins_and_failed_write_preserves_snapshot() {
    let f = Fixture::new();
    let mut cache = Cache::open(&f.0, &Config::default()).unwrap();
    let mut root = DirEntry::new(f.0.clone(), true);
    let mut sub = DirEntry::new(root.path.join("sub"), true);
    cache.store(&sub, 100).unwrap();
    sub.add_child(DirEntry::new(sub.path.join("new"), false));
    root.add_child(sub.clone());
    cache.store(&root, 200).unwrap();
    assert_eq!(cache.load(&sub.path, 300).unwrap().unwrap().file_count(), 1);
    sub.add_child(DirEntry::new(sub.path.join("newer"), false));
    cache.store(&sub, 400).unwrap();
    let stale = DirEntry::new(sub.path.clone(), true);
    cache.store(&stale, 250).unwrap();
    assert_eq!(cache.load(&sub.path, 401).unwrap().unwrap(), sub);
    // Duplicate names violate the unique index partway through replacement.
    sub.add_child(DirEntry::new(sub.path.join("newer"), false));
    assert!(cache.store(&sub, 450).is_err());
    assert_eq!(cache.load(&sub.path, 500).unwrap().unwrap().file_count(), 2);
}

#[test]
fn unreadable_cache_falls_back_to_live_tree() {
    let f = Fixture::new();
    let root = f.0.join("root");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("file"), b"abc").unwrap();
    let cache = f.0.join("cache");
    fs::create_dir(&cache).unwrap();
    fs::write(cache.join("trees.sqlite3"), b"invalid").unwrap();
    assert_eq!(
        Scanner::new(Config::default())
            .scan_using_cache(&root, &cache, |_| {})
            .unwrap()
            .total_size,
        3
    );
}

#[test]
fn symlinked_cache_directory_is_excluded_and_scanned_live_when_selected() {
    let f = Fixture::new();
    let root = f.0.join("project");
    let directory = root.join("cache");
    fs::create_dir_all(&directory).unwrap();
    let alias = f.0.join("alias");
    std::os::unix::fs::symlink(&directory, &alias).unwrap();
    let scanner = Scanner::new(Config::default());

    fs::write(directory.join("one"), b"a").unwrap();
    let inside = scanner
        .scan_using_cache(&directory, &alias, |_| {})
        .unwrap();
    assert_eq!(inside.file_count(), 1);
    assert!(!directory.join("trees.sqlite3").exists());

    fs::write(root.join("visible"), b"abc").unwrap();
    let parent = scanner.scan_using_cache(&root, &alias, |_| {}).unwrap();
    assert_eq!(parent.children.unwrap().len(), 1);
    assert_eq!(parent.total_size, 3);
    assert!(directory.join("trees.sqlite3").is_file());
}
