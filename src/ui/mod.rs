mod format;
use format::fit_label;
pub mod colors;
pub mod components;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Breadcrumb
            Constraint::Min(0),    // Main content
            Constraint::Length(1 + components::status_bar::help_height(app, f.area().width)), // Status bar
        ])
        .split(f.area());

    components::breadcrumb::render(f, chunks[0], app);
    if let crate::app::AppState::Files {
        folder,
        indices,
        cursor,
    } = &app.state
    {
        if let Some(source) = app.get_selected_entry().or_else(|| app.get_current_tree()) {
            components::explorer::render_files(
                f,
                chunks[1],
                folder,
                source.children.as_deref().unwrap_or_default(),
                indices,
                *cursor,
            );
        }
        components::status_bar::render(f, chunks[2], app);
        return;
    }
    app.tiles = app
        .get_current_tree()
        .and_then(|tree| tree.children.as_ref())
        .map(|children| {
            crate::algorithms::treemap::tiles(
                children
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.is_dir)
                    .map(|(i, e)| (i, e.total_size)),
                chunks[1],
            )
        })
        .unwrap_or_default();
    if let Some(children) = app
        .get_current_tree()
        .and_then(|tree| tree.children.as_ref())
    {
        let selected = app
            .tiles
            .iter()
            .find(|(i, _)| app.selection.is_selected(&children[*i].path))
            .or(app.tiles.first())
            .map(|(i, _)| (*i, children[*i].path.clone()));
        if let Some((index, path)) = selected {
            app.selection.select(path, index);
        } else {
            app.selection.clear();
        }
    }
    components::explorer::render(f, chunks[1], app);
    components::status_bar::render(f, chunks[2], app);

    // Render overlays
    if let crate::app::AppState::Inspecting { .. } = &app.state {
        components::inspector::render(f, app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{events::Message, models::DirEntry};
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn file_preview_only_changes_spare_cells_inside_its_folder() {
        let mut app = App::new(Default::default(), "/root".into());
        let render = |app: &mut App, filename: &str| {
            let mut root = DirEntry::new(app.current_dir.clone(), true);
            let mut folder = DirEntry::new(root.path.join("photos"), true);
            folder.add_child(DirEntry::new(folder.path.join(filename), false).with_size(512, false));
            root.add_child(folder);
            root.add_child(DirEntry::new(root.path.join("music"), true).with_size(256, false));
            app.update(Message::ScanComplete(root));
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|frame| render_ui(frame, app)).unwrap();
            terminal.backend().buffer().clone()
        };

        let hidden = render(&mut app, &"a".repeat(100));
        let tiles = app.tiles.clone();
        let visible = render(&mut app, "photo.jpg");
        assert_eq!(app.tiles, tiles);
        assert_eq!(tiles.len(), 2);

        let folder = tiles[0].1;
        let preview_y = folder.y + 4; // Top border, folder name, size, blank row.
        let mut preview = String::new();
        for y in 0..visible.area.height {
            for x in 0..visible.area.width {
                if y == preview_y && x > folder.x && x < folder.right() - 1 {
                    preview.push_str(visible[(x, y)].symbol());
                } else {
                    assert_eq!(visible[(x, y)], hidden[(x, y)], "changed cell ({x}, {y})");
                }
            }
        }
        assert!(preview.starts_with("  photo.jpg"));
        assert!(preview.ends_with("512B"));
    }
}
