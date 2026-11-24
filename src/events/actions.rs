use crate::models::DirEntry;
use crate::models::selection::Direction;

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Direction),
    EnterDirectory,
    GoUp,
    ToggleFiles,
    Inspect,
    ScanComplete(DirEntry),
    ScanError(String),
    OpenShell,
    Quit,
}
