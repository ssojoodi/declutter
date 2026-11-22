use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let path_str = app.current_dir.display().to_string();
    let text = Span::styled(
        crate::ui::fit_label(&path_str, area.width.into()),
        Style::default().fg(Color::Cyan),
    );
    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}
