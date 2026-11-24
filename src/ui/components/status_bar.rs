use crate::app::{App, AppState};
use crate::ui::format::format_size;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

fn hints(app: &App) -> &[&str] {
    match app.state {
        AppState::Inspecting { .. } => &["Esc close", "Q quit"],
        AppState::Files { .. } => &["↑↓ select file", "F/Esc close", "Backspace back", "Q quit"],
        AppState::Exploring => &[
            "↑↓←→ move",
            "Space open",
            "Esc/Backspace up",
            "Q quit",
            "F files",
            "c open shell",
            "I inspect",
        ],
        _ => &["Q quit"],
    }
}

fn legend(app: &App, width: u16) -> Vec<String> {
    let mut rows = vec![String::new()];
    for hint in hints(app) {
        let row = rows.last_mut().unwrap();
        if row.is_empty() {
            row.push_str(hint);
        } else if row.chars().count() + 3 + hint.chars().count() <= usize::from(width) {
            row.push_str("   ");
            row.push_str(hint);
        } else {
            rows.push(hint.to_string());
        }
    }
    rows
}

pub fn help_height(app: &App, width: u16) -> u16 {
    legend(app, width).len() as u16
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let status = match &app.state {
        AppState::Loading { .. } => "Scanning directory…".to_owned(),
        AppState::Error(msg) => msg.clone(),
        AppState::Inspecting { .. } => "Inspector".to_owned(),
        AppState::Files {
            indices, cursor, ..
        } if !indices.is_empty() => {
            format!("File {} of {} • largest first", cursor + 1, indices.len())
        }
        AppState::Files { .. } => "0 files".to_owned(),
        AppState::Exploring => app
            .get_selected_entry()
            .map(|e| {
                format!(
                    "{} | {} | {:?}",
                    e.name(),
                    format_size(e.total_size),
                    e.file_type
                )
            })
            .unwrap_or_else(|| "No selection".to_owned()),
    };
    let mut lines = vec![Line::from(Span::styled(
        crate::ui::fit_label(&status, area.width.into()),
        Style::default().fg(Color::Yellow),
    ))];
    lines.extend(legend(app, area.width).into_iter().map(Line::from));
    f.render_widget(Paragraph::new(lines), area);
}
