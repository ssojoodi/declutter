use ratatui::style::Color;

pub fn parse_color(hex: &str) -> Color {
    if let Some(stripped) = hex.strip_prefix('#') {
        if stripped.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&stripped[0..2], 16),
                u8::from_str_radix(&stripped[2..4], 16),
                u8::from_str_radix(&stripped[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }
    Color::White
}

pub fn interpolate_color(color1: Color, color2: Color, ratio: f64) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);

    match (color1, color2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f64 + (r2 as f64 - r1 as f64) * ratio) as u8;
            let g = (g1 as f64 + (g2 as f64 - g1 as f64) * ratio) as u8;
            let b = (b1 as f64 + (b2 as f64 - b1 as f64) * ratio) as u8;
            Color::Rgb(r, g, b)
        }
        _ => color1,
    }
}
