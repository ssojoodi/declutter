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
