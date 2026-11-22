pub(crate) fn fit_label(text: &str, width: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    if text.width() <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut left = String::new();
    let mut used = 0;
    for g in text.graphemes(true) {
        if used + g.width() > width / 2 {
            break;
        }
        left.push_str(g);
        used += g.width();
    }
    let mut suffix = text.len();
    for (i, g) in text.grapheme_indices(true).rev() {
        if used + 1 + g.width() > width {
            break;
        }
        suffix = i;
        used += g.width();
    }
    format!("{left}…{}", &text[suffix..])
}

fn scaled_size(bytes: u64) -> (f64, &'static str) {
    let mut size = bytes as f64;
    for unit in ["B", "KB", "MB", "GB"] {
        if size < 1024.0 {
            return (size, unit);
        }
        size /= 1024.0;
    }
    (size, "TB")
}

pub(super) fn format_size(bytes: u64) -> String {
    let (size, unit) = scaled_size(bytes);
    format!("{size:.2} {unit}")
}

pub(super) fn compact_size(bytes: u64) -> String {
    let (size, unit) = scaled_size(bytes);
    let precision = if size >= 100.0 {
        0
    } else if size >= 10.0 {
        1
    } else {
        2
    };
    format!("{size:.precision$}{unit}")
}
