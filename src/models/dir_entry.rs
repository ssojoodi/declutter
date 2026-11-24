use std::path::PathBuf;
use super::FileType;

#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    pub path: PathBuf,
    pub total_size: u64,
    pub temp_size: u64,
    pub real_size: u64,
    pub file_type: FileType,
    pub children: Option<Vec<DirEntry>>,
    pub is_dir: bool,
}

impl DirEntry {
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        Self {
            path,
            total_size: 0,
            temp_size: 0,
            real_size: 0,
            file_type: FileType::Real,
            children: if is_dir { Some(Vec::new()) } else { None },
            is_dir,
        }
    }

    pub fn with_size(mut self, size: u64, is_temp: bool) -> Self {
        self.total_size = size;
        if is_temp {
            self.temp_size = size;
            self.file_type = FileType::Temporary;
        } else {
            self.real_size = size;
            self.file_type = FileType::Real;
        }
        self
    }

    pub fn add_child(&mut self, child: DirEntry) {
        if let Some(ref mut children) = self.children {
            self.total_size += child.total_size;
            self.temp_size += child.temp_size;
            self.real_size += child.real_size;
            children.push(child);
        }
    }

    pub fn finalize(&mut self) {
        self.file_type = FileType::from_sizes(self.real_size, self.temp_size);
        if let Some(ref mut children) = self.children {
            // Sort children by size (largest first)
            children.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        }
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string()
    }

    pub fn file_count(&self) -> usize {
        if !self.is_dir {
            return 1;
        }
        self.children
            .as_ref()
            .map(|c| c.iter().map(|e| e.file_count()).sum())
            .unwrap_or(0)
    }

    pub fn dir_count(&self) -> usize {
        if !self.is_dir {
            return 0;
        }
        1 + self.children
            .as_ref()
            .map(|c| c.iter().map(|e| e.dir_count()).sum())
            .unwrap_or(0)
    }
}
