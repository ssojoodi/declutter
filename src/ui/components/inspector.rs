use crate::ui::format::format_size;
use crate::app::{App, AppState};
use crate::models::{DirEntry, FileType};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    if let AppState::Inspecting { entry } = &app.state {
        let screen = f.area();
        let width = ((u32::from(screen.width) * 70 / 100) as u16)
            .max(32)
            .min(screen.width);
        let lines = inspector_lines(entry);
        let height =
            (lines.len() as u16 + 4).min(screen.height.saturating_sub(2).max(screen.height.min(4)));
        let area = Rect::new(
            screen.x + (screen.width - width) / 2,
            screen.y + (screen.height - height) / 2,
            width,
            height,
        );
        let block = Block::default()
            .title("Inspector")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));
        let inner = block.inner(area);
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        if inner.height == 0 {
            return;
        }
        let body = Rect {
            height: inner.height - 1,
            ..inner
        };
        f.render_widget(Paragraph::new(lines), body);
        let footer = Rect::new(inner.x, body.bottom(), inner.width, 1);
        let label = "Largest item: ";
        let value = match entry
            .children
            .as_deref()
            .and_then(|children| children.first())
        {
            Some(child) => {
                let size = format_size(child.total_size);
                let width = usize::from(inner.width).saturating_sub(label.len());
                if width > size.len() + 3 {
                    format!(
                        "{} — {size}",
                        crate::ui::fit_label(&child.name(), width - size.len() - 3)
                    )
                } else {
                    crate::ui::fit_label(&child.name(), width)
                }
            }
            None => "None (empty folder)".to_owned(),
        };
        f.render_widget(Paragraph::new(field(label, value)), footer);
    }
}

fn field(label: &'static str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Yellow)),
        Span::raw(value),
    ])
}

fn heading(title: &'static str, color: Color) -> Line<'static> {
    Line::styled(
        title,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn inspector_lines(entry: &DirEntry) -> Vec<Line<'static>> {
    let mut lines = vec![
        heading("📁 Directory Information", Color::Cyan),
        field("Name: ", entry.name()),
        field("Path: ", entry.path.display().to_string()),
        Line::from(""),
        heading("Size Information", Color::Green),
        field("Total Size: ", format_size(entry.total_size)),
    ];
    for (label, size) in [
        ("Real Files: ", entry.real_size),
        ("Temporary Files: ", entry.temp_size),
    ] {
        lines.push(Line::from(vec![
            Span::styled(label, Style::default().fg(Color::Yellow)),
            Span::raw(format_size(size)),
            Span::styled(
                format!(" ({:.1}%)", percentage(size, entry.total_size)),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }
    lines.extend([
        Line::from(""),
        heading("File Statistics", Color::Magenta),
        field("Type: ", format!("{:?}", entry.file_type)),
        field("Total Files: ", entry.file_count().to_string()),
        field(
            "Subdirectories: ",
            entry.dir_count().saturating_sub(1).to_string(),
        ),
    ]);
    if let Some(children) = entry.children.as_deref().filter(|c| !c.is_empty()) {
        lines.extend([
            Line::from(""),
            heading("Contents", Color::Blue),
            field("Direct Children: ", children.len().to_string()),
        ]);
        for (kind, color) in [
            (FileType::Real, Color::Blue),
            (FileType::Temporary, Color::LightBlue),
            (FileType::Mixed, Color::Yellow),
        ] {
            let count = children
                .iter()
                .filter(|child| child.file_type == kind)
                .count();
            if count > 0 {
                lines.push(Line::from(vec![
                    Span::raw(format!("  • {kind:?}: ")),
                    Span::styled(count.to_string(), Style::default().fg(color)),
                ]));
            }
        }
    }
    lines
}



fn percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
