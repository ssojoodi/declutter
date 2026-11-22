use anyhow::Result;
use jwalk::WalkDirGeneric;
use std::path::Path;

use super::classifier::Classifier;
use super::size_calculator::calculate_file_size;
use crate::config::Config;
use crate::models::DirEntry;

pub struct Scanner {
    config: Config,
    classifier: Classifier,
}

impl Scanner {
    pub fn new(config: Config) -> Self {
        let classifier = Classifier::new(&config.temp_patterns);
        Self { config, classifier }
    }

    pub fn scan(&self, root: &Path) -> Result<DirEntry> {
        self.scan_with_progress(root, |_| {})
    }

    /// Report the first two directory levels as the index advances through them.
    pub fn scan_with_progress(
        &self,
        root: &Path,
        mut on_folder: impl FnMut(&Path),
    ) -> Result<DirEntry> {
        let walk = WalkDirGeneric::<((), u64)>::new(root)
            .max_depth(self.config.performance.max_depth)
            .parallelism(jwalk::Parallelism::RayonNewPool(
                self.config.performance.parallelism,
            ))
            .skip_hidden(false)
            .process_read_dir(|_, _, _, children| {
                for entry in children.iter_mut().flatten() {
                    if !entry.file_type().is_dir() {
                        entry.client_state = calculate_file_size(&entry.path());
                    }
                }
            });

        // jwalk yields depth-first, even when workers read directories concurrently.
        // Keep only unfinished ancestors alongside the tree itself.
        let mut open: Vec<(DirEntry, bool)> = Vec::new();
        for entry in walk.into_iter().flatten() {
            while open.len() > entry.depth() {
                let (mut child, _) = open.pop().unwrap();
                child.finalize();
                open.last_mut().unwrap().0.add_child(child);
            }

            let path = entry.path();
            let is_temp = match open.last() {
                Some((_, inherited)) => *inherited || self.classifier.matches_name(&path),
                None => self.classifier.is_temporary(&path),
            };
            let mut node = DirEntry::new(path, entry.file_type().is_dir());
            if node.is_dir {
                if (1..=2).contains(&entry.depth()) {
                    on_folder(&node.path);
                }
                open.push((node, is_temp));
            } else {
                node = node.with_size(entry.client_state, is_temp);
                node.finalize();
                match open.last_mut() {
                    Some((parent, _)) => parent.add_child(node),
                    None => return Ok(node),
                }
            }
        }

        while let Some((mut node, _)) = open.pop() {
            node.finalize();
            match open.last_mut() {
                Some((parent, _)) => parent.add_child(node),
                None => return Ok(node),
            }
        }
        anyhow::bail!("Failed to scan root directory {}", root.display())
    }
}
