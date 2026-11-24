use std::path::PathBuf;

use crate::config::Config;
use crate::events::Message;
use crate::models::selection::Direction;
use crate::models::{DirEntry, Selection};

#[derive(Debug)]
pub enum AppState {
    Loading {
        folder: PathBuf,
        tick: usize,
    },
    Exploring,
    Files {
        folder: PathBuf,
        indices: Vec<usize>,
        cursor: usize,
    },
    Inspecting {
        entry: DirEntry,
    },
    Error(String),
}

pub struct App {
    pub(crate) tiles: Vec<(usize, ratatui::layout::Rect)>,
    pub state: AppState,
    pub config: Config,
    pub current_dir: PathBuf,
    pub dir_tree: Option<DirEntry>,
    pub selection: Selection,
    pub history: Vec<PathBuf>,
}

pub enum Command {
    None,
    Quit,
}

impl App {
    pub fn new(config: Config, start_dir: PathBuf) -> Self {
        Self {
            state: AppState::Loading {
                folder: start_dir.clone(),
                tick: 0,
            },
            tiles: Vec::new(),
            config,
            current_dir: start_dir.clone(),
            dir_tree: None,
            selection: Selection::new(),
            history: vec![],
        }
    }

    pub fn update(&mut self, msg: Message) -> Command {
        match msg {
            Message::ScanComplete(tree) => {
                self.current_dir.clone_from(&tree.path);
                self.dir_tree = Some(tree);
                self.state = AppState::Exploring;
                self.select_first();
                Command::None
            }

            Message::ScanError(err) => {
                self.state = AppState::Error(err);
                Command::None
            }

            Message::ToggleFiles => {
                match &self.state {
                    AppState::Files { .. } => self.state = AppState::Exploring,
                    AppState::Exploring => {
                        if let Some(folder) = self
                            .get_selected_entry()
                            .or_else(|| self.get_current_tree())
                        {
                            // The scanner already sorted these children by descending size.
                            let indices = folder
                                .children
                                .iter()
                                .flatten()
                                .enumerate()
                                .filter_map(|(i, entry)| (!entry.is_dir).then_some(i))
                                .collect();
                            self.state = AppState::Files {
                                folder: folder.path.clone(),
                                indices,
                                cursor: 0,
                            };
                        }
                    }
                    _ => {}
                }
                Command::None
            }
            Message::Navigate(direction) => {
                if let AppState::Files {
                    indices, cursor, ..
                } = &mut self.state
                {
                    match direction {
                        Direction::Up => *cursor = cursor.saturating_sub(1),
                        Direction::Down => {
                            *cursor = (*cursor + 1).min(indices.len().saturating_sub(1))
                        }
                        _ => {}
                    }
                    return Command::None;
                }
                if matches!(self.state, AppState::Exploring) {
                    self.navigate(direction);
                }
                Command::None
            }

            Message::EnterDirectory => {
                if matches!(self.state, AppState::Exploring) {
                    let selected_path = self
                        .get_selected_entry()
                        .filter(|e| e.is_dir && e.children.is_some())
                        .map(|e| e.path.clone());

                    if let Some(path) = selected_path {
                        self.history.push(self.current_dir.clone());
                        self.current_dir = path;
                        self.select_first();
                    }
                }
                Command::None
            }

            Message::GoUp if matches!(self.state, AppState::Files { .. }) => {
                self.update(Message::ToggleFiles)
            }
            Message::GoUp if matches!(self.state, AppState::Inspecting { .. }) => {
                self.state = AppState::Exploring;
                Command::None
            }

            Message::GoUp => {
                if matches!(self.state, AppState::Exploring) {
                    if let Some(parent) = self.history.pop() {
                        let previous = std::mem::replace(&mut self.current_dir, parent);
                        self.selection.select(previous, 0);
                    }
                }
                Command::None
            }

            Message::Inspect => {
                if matches!(self.state, AppState::Exploring) {
                    if let Some(selected) = self.get_selected_entry() {
                        self.state = AppState::Inspecting {
                            entry: selected.clone(),
                        };
                    }
                }
                Command::None
            }

            Message::Quit => Command::Quit,
        }
    }

    pub fn get_selected_entry(&self) -> Option<&DirEntry> {
        let current_tree = self.get_current_tree()?;
        let children = current_tree.children.as_ref()?;

        if let Some(ref selected_path) = self.selection.path {
            children.iter().find(|e| &e.path == selected_path)
        } else {
            children.iter().find(|e| e.is_dir)
        }
    }

    pub fn get_current_tree(&self) -> Option<&DirEntry> {
        let tree = self.dir_tree.as_ref()?;

        if tree.path == self.current_dir {
            return Some(tree);
        }

        self.find_entry_by_path(&tree, &self.current_dir)
    }

    fn find_entry_by_path<'a>(&self, entry: &'a DirEntry, path: &PathBuf) -> Option<&'a DirEntry> {
        if &entry.path == path {
            return Some(entry);
        }

        if let Some(ref children) = entry.children {
            for child in children {
                if let Some(found) = self.find_entry_by_path(child, path) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn select_first(&mut self) {
        if let Some(tree) = self.get_current_tree() {
            if let Some(children) = &tree.children {
                if let Some(first) = children.first() {
                    self.selection.select(first.path.clone(), 0);
                    return;
                }
            }
        }
        self.selection.clear();
    }

    fn navigate(&mut self, direction: Direction) {
        let Some(&(_, from)) = self.tiles.iter().find(|(i, _)| *i == self.selection.index) else {
            return;
        };
        let center = |start: u16, length: u16| 2 * i32::from(start) + i32::from(length);
        let mut best = None;
        for &(i, to) in &self.tiles {
            let (a, b, c, d, forward) = match direction {
                Direction::Left | Direction::Right => (
                    from.y,
                    from.height,
                    to.y,
                    to.height,
                    if direction == Direction::Right {
                        i32::from(to.x) - i32::from(from.right())
                    } else {
                        i32::from(from.x) - i32::from(to.right())
                    },
                ),
                Direction::Up | Direction::Down => (
                    from.x,
                    from.width,
                    to.x,
                    to.width,
                    if direction == Direction::Down {
                        i32::from(to.y) - i32::from(from.bottom())
                    } else {
                        i32::from(from.y) - i32::from(to.bottom())
                    },
                ),
            };
            if forward < 0 {
                continue;
            }
            let overlap = a.max(c) < (a + b).min(c + d);
            let rank = (!overlap, forward, (center(a, b) - center(c, d)).abs(), i);
            if best.is_none_or(|prev| rank < prev) {
                best = Some(rank);
            }
        }
        if let Some((_, _, _, i)) = best {
            let path = self.get_current_tree().unwrap().children.as_ref().unwrap()[i]
                .path
                .clone();
            self.selection.select(path, i);
        }
    }
}
