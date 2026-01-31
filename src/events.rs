use crate::app::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => Some(Action::NavigateDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::NavigateUp),
        KeyCode::Char('g') => Some(Action::NavigateTop),
        KeyCode::Char('G') => Some(Action::NavigateBottom),

        // Task management
        KeyCode::Char('K') => Some(Action::KillTask),
        KeyCode::Char('p') => Some(Action::TogglePause),
        KeyCode::Char('r') => Some(Action::Refresh),

        // Viewing
        KeyCode::Enter => Some(Action::ViewLogs),

        // Quit
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        _ => None,
    }
}
