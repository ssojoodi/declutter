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
