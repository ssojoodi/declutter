use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Selection {
    /// Currently selected path
    pub path: Option<PathBuf>,
    /// Index in the current children list
    pub index: usize,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            path: None,
            index: 0,
        }
    }

    pub fn select(&mut self, path: PathBuf, index: usize) {
        self.path = Some(path);
        self.index = index;
    }

    pub fn clear(&mut self) {
        self.path = None;
        self.index = 0;
    }

    pub fn is_selected(&self, path: &PathBuf) -> bool {
        self.path.as_ref() == Some(path)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}
