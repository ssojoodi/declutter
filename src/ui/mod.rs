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
#[path = "../../tests/unit/ui.rs"]
mod tests;
