use crate::models::DirEntry;
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct TreemapNode {
    pub entry: DirEntry,
    pub rect: Rect,
}

/// Split contiguous, size-sorted groups near half their combined log weight.
pub(crate) fn tiles(entries: impl Iterator<Item = (usize, u64)>, area: Rect) -> Vec<(usize, Rect)> {
    let mut total = 0.0;
    let items: Vec<_> = entries
        .map(|(index, size)| {
            total += 1.0 + 10.0 * (size as f64 + 1.0).log10();
            (index, total)
        })
        .collect();
    let mut result = Vec::with_capacity(items.len());
    partition(&items, 0.0, area, &mut result);
    result
}

fn partition(items: &[(usize, f64)], start: f64, area: Rect, out: &mut Vec<(usize, Rect)>) {
    if items.is_empty() || area.is_empty() {
        return;
    }
    if items.len() == 1 || (area.width == 1 && area.height == 1) {
        out.push((items[0].0, area));
        return;
    }
    let weight = items.last().unwrap().1 - start;
    let mid = start + weight / 2.0;
    let mut split = items.partition_point(|item| item.1 < mid);
    if split == 0 || (split + 1 < items.len() && items[split].1 - mid < mid - items[split - 1].1) {
        split += 1;
    }
    let boundary = items[split - 1].1;
    // Terminal cells are roughly twice as tall as they are wide.
    let across = area.height == 1 || u32::from(area.width) >= 2 * u32::from(area.height);
    let extent = if across { area.width } else { area.height };
    let cut =
        (((boundary - start) / weight * f64::from(extent)).round() as u16).clamp(1, extent - 1);
    let (mut first, mut second) = (area, area);
    if across {
        first.width = cut;
        second.x += cut;
        second.width -= cut;
    } else {
        first.height = cut;
        second.y += cut;
        second.height -= cut;
    }
    partition(&items[..split], start, first, out);
    partition(&items[split..], boundary, second, out);
}

pub fn squarify(entries: &[DirEntry], area: Rect) -> Vec<TreemapNode> {
    tiles(
        entries.iter().enumerate().map(|(i, e)| (i, e.total_size)),
        area,
    )
    .into_iter()
    .map(|(i, rect)| TreemapNode {
        entry: entries[i].clone(),
        rect,
    })
    .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/treemap.rs"]
mod tests;
