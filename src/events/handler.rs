use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use super::Message;
use crate::models::selection::Direction;

pub fn handle_key_event(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Message::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Message::Quit),

        KeyCode::Up | KeyCode::Char('k') => Some(Message::Navigate(Direction::Up)),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::Navigate(Direction::Down)),
        KeyCode::Left | KeyCode::Char('h') => Some(Message::Navigate(Direction::Left)),
        KeyCode::Right | KeyCode::Char('l') => Some(Message::Navigate(Direction::Right)),

        KeyCode::Enter | KeyCode::Char(' ') => Some(Message::EnterDirectory),
        KeyCode::Backspace | KeyCode::Esc => Some(Message::GoUp),


        KeyCode::Char('f') | KeyCode::Char('F') => Some(Message::ToggleFiles),
        KeyCode::Char('i') | KeyCode::Char('I') => Some(Message::Inspect),

        _ => None,
    }
}
