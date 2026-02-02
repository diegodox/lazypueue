use crate::app::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle key events when in text input mode
pub fn handle_input_mode_key_event(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Submit
        KeyCode::Enter => Some(Action::SubmitInput),
        // Cancel
        KeyCode::Esc => Some(Action::CancelInput),
        // Text editing
        KeyCode::Backspace => Some(Action::InputBackspace),
        KeyCode::Delete => Some(Action::InputDelete),
        KeyCode::Left => Some(Action::InputLeft),
        KeyCode::Right => Some(Action::InputRight),
        KeyCode::Home => Some(Action::InputHome),
        KeyCode::End => Some(Action::InputEnd),
        // Ctrl shortcuts
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::InputHome)
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::InputEnd)
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::CancelInput)
        }
        // Regular characters
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        _ => None,
    }
}

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

/// Handle key events when a confirmation dialog is shown
pub fn handle_confirm_mode_key_event(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Confirm with y or Enter
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Action::ConfirmAction),
        // Cancel with n, Escape, or any other key
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::CancelConfirm),
        // Any other key cancels
        _ => Some(Action::CancelConfirm),
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
        KeyCode::Char(' ') => Some(Action::ToggleTaskPause),
        KeyCode::Char('r') => Some(Action::Refresh),
        KeyCode::Char('R') => Some(Action::RestartTask),
        KeyCode::Char('c') => Some(Action::CleanFinished),
        KeyCode::Char('a') => Some(Action::StartAddTask),
        KeyCode::Char('e') => Some(Action::StartEditTask),
        KeyCode::Char('d') | KeyCode::Char('x') => Some(Action::RemoveTask),

        // Stash/Enqueue
        KeyCode::Char('s') => Some(Action::StashTask),
        KeyCode::Char('S') => Some(Action::EnqueueTask),

        // Switch task order
        KeyCode::Char('<') => Some(Action::SwitchUp),
        KeyCode::Char('>') => Some(Action::SwitchDown),

        // Parallel limit
        KeyCode::Char('+') | KeyCode::Char('=') => Some(Action::IncreaseParallel),
        KeyCode::Char('-') | KeyCode::Char('_') => Some(Action::DecreaseParallel),

        // Tree navigation: h collapses / goes to parent, l expands / views logs
        KeyCode::Char('h') | KeyCode::Left => Some(Action::CollapseGroup),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => Some(Action::ExpandGroup),

        // Viewing
        KeyCode::Char('f') => Some(Action::FollowLogs),

        // Quit
        KeyCode::Char('q') => Some(Action::Quit),

        _ => None,
    }
}
