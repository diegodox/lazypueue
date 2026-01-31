use crate::app::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle key events when the log modal is open
pub fn handle_log_modal_key_event(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Close modal
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseLogs),

        // Scroll
        KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollLogDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollLogUp),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ScrollLogPageDown)
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ScrollLogPageUp)
        }
        KeyCode::PageDown => Some(Action::ScrollLogPageDown),
        KeyCode::PageUp => Some(Action::ScrollLogPageUp),
        KeyCode::Char('g') => Some(Action::ScrollLogUp), // Go to top (will saturate at 0)
        KeyCode::Char('G') => Some(Action::ScrollLogDown), // Go to bottom (large value)

        // Toggle follow mode
        KeyCode::Char('f') => Some(Action::FollowLogs),

        _ => None,
    }
}

/// Handle key events in the main view
pub fn handle_key_event(key: KeyEvent) -> Option<Action> {
    // Check for Ctrl+C first (quit)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

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
        KeyCode::Char('R') => Some(Action::RestartTask),
        KeyCode::Char('c') => Some(Action::CleanFinished),

        // Viewing
        KeyCode::Enter | KeyCode::Char('l') => Some(Action::ViewLogs),
        KeyCode::Char('f') => Some(Action::FollowLogs),

        // Quit
        KeyCode::Char('q') => Some(Action::Quit),

        _ => None,
    }
}
