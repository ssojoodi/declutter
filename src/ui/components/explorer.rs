use crate::app::{App, AppState};
use crate::ui::format::compact_size;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::models::{DirEntry, FileType};
use crate::ui::colors::{interpolate_color, parse_color};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    match &app.state {
        AppState::Loading { folder, tick } => {
            const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let relative = folder.strip_prefix(&app.current_dir).unwrap_or(folder);
            let path = if relative.as_os_str().is_empty() {
                folder
            } else {
                relative
            };
            let label = crate::ui::fit_label(
                &path.to_string_lossy(),
                usize::from(area.width.saturating_sub(2)),
            );
            let line = format!("{} {label}", SPINNER[tick % SPINNER.len()]);
            f.render_widget(
                Paragraph::new(line).style(Style::default().fg(Color::Cyan)),
                area,
            );
        }
        AppState::Error(msg) => f.render_widget(Paragraph::new(msg.as_str()), area),
        _ => {
            if let Some(tree) = app.get_current_tree() {
                render_treemap(f, area, app, tree);
            }
        }
    }
}

fn render_treemap(f: &mut Frame, area: Rect, app: &App, tree: &crate::models::DirEntry) {
    if app.tiles.is_empty() {
        let message = if tree.children.as_ref().is_none_or(Vec::is_empty) {
            "Empty folder"
        } else {
            "No subfolders. Press F to show files."
        };
        f.render_widget(Paragraph::new(message), area);
        return;
    }
    if let Some(children) = &tree.children {
        for &(index, rect) in &app.tiles {
            render_node(f, &children[index], rect, app);
        }
    }
}

fn render_node(f: &mut Frame, entry: &crate::models::DirEntry, rect: Rect, app: &App) {
    // Determine color based on file type
    let real_color = parse_color(&app.config.colors.real_file_fg);
    let temp_color = parse_color(&app.config.colors.temp_file_fg);

    let bg_color = match entry.file_type {
        FileType::Real => real_color,
        FileType::Temporary => temp_color,
        FileType::Mixed => {
            let ratio = FileType::temp_ratio(entry.real_size, entry.temp_size);
            interpolate_color(real_color, temp_color, ratio)
        }
    };

    // Check if selected
    let is_selected = app.selection.is_selected(&entry.path);
    let border_color = if is_selected {
        parse_color(&app.config.colors.selected_border)
    } else {
        Color::DarkGray
    };

    let mut style = Style::default().bg(bg_color);
    if is_selected {
        style = style.add_modifier(Modifier::BOLD);
    }

    // Create block with border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(style);

    let text_area = if rect.width >= 4 && rect.height >= 3 {
        let inner = block.inner(rect);
        f.render_widget(block, rect);
        inner
    } else {
        rect
    };
    let name = crate::ui::fit_label(&entry.name(), usize::from(text_area.width));
    let mut lines = vec![Line::from(name)];
    if text_area.height > 1 {
        lines.push(Line::from(compact_size(entry.total_size)));
    }
    if let Some(files) = fitting_file_list(entry, text_area) {
        lines.push(Line::default());
        lines.extend(files);
    }
    let foreground = match bg_color {
        Color::Rgb(r, g, b)
            if 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b) > 135000 =>
        {
            Color::Black
        }
        _ => Color::White,
    };
    let mut label_style = style.fg(foreground);
    if text_area == rect && is_selected {
        label_style = label_style.add_modifier(Modifier::REVERSED);
    }
    f.render_widget(Paragraph::new(lines).style(label_style), text_area);
}

fn fitting_file_list(entry: &DirEntry, area: Rect) -> Option<Vec<Line<'static>>> {
    let available_rows = usize::from(area.height.saturating_sub(3));
    if available_rows == 0 {
        return None;
    }

    let mut lines = Vec::new();
    for file in entry.children.as_ref()?.iter().filter(|child| !child.is_dir) {
        if lines.len() == available_rows {
            return None;
        }
        let name = file.path.file_name()?.to_string_lossy();
        let size = compact_size(file.total_size);
        let gap = usize::from(area.width).saturating_sub(2 + name.width() + size.len());
        if gap < 2 || name.contains(char::is_control) {
            return None;
        }
        let line = Line::from(format!("  {name}{}{size}", " ".repeat(gap)));
        lines.push(line.style(Style::default().remove_modifier(Modifier::BOLD)));
    }
    (!lines.is_empty()).then_some(lines)
}

pub fn render_files(
    f: &mut Frame,
    area: Rect,
    folder: &std::path::Path,
    children: &[crate::models::DirEntry],
    indices: &[usize],
    cursor: usize,
) {
    let title = format!("Files in {}", folder.display());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(crate::ui::fit_label(
            &title,
            area.width.saturating_sub(2).into(),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if indices.is_empty() {
        f.render_widget(Paragraph::new("No files directly in this folder."), inner);
        return;
    }
    let height = usize::from(inner.height);
    let start = (cursor + 1).saturating_sub(height);
    let mut lines = Vec::new();
    for (row, &index) in indices.iter().enumerate().skip(start).take(height) {
        let entry = &children[index];
        let size = compact_size(entry.total_size);
        let width = usize::from(inner.width);
        let name_width = width.saturating_sub(size.len() + 2);
        let label = if name_width > 0 {
            let name = crate::ui::fit_label(&entry.name(), name_width);
            let spaces = width - name.width() - size.len();
            format!("{name}{}{size}", " ".repeat(spaces))
        } else {
            crate::ui::fit_label(&entry.name(), width)
        };
        let line = Line::from(label);
        lines.push(if row == cursor {
            line.style(Style::default().add_modifier(Modifier::REVERSED))
        } else {
            line
        });
    }
    f.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_list_requires_space_for_every_indented_name_and_the_gap() {
        let mut folder = DirEntry::new("folder".into(), true);
        folder.add_child(DirEntry::new("nested".into(), true));
        folder.add_child(DirEntry::new("猫.txt".into(), false).with_size(12, false));
        folder.add_child(DirEntry::new("b".into(), false).with_size(2048, false));
        folder.finalize();

        let lines = fitting_file_list(&folder, Rect::new(0, 0, 15, 5)).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].to_string(), "  b      2.00KB");
        assert_eq!(lines[1].to_string(), "  猫.txt  12.0B");
        assert!(fitting_file_list(&folder, Rect::new(0, 0, 14, 5)).is_none());
        assert!(fitting_file_list(&folder, Rect::new(0, 0, 15, 4)).is_none());
    }
}
