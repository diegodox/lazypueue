mod details;
pub mod input;
mod status_bar;
mod task_list;

pub use details::render_details_panel;
pub use input::{render_input_dialog, TextInput};
pub use status_bar::{render_help_bar, render_status_bar};
pub use task_list::render_task_list;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    // Check for error message
    if let Some(error) = &app.error_message {
        render_error(f, error);
        return;
    }

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Help bar
        ])
        .split(f.area());

    // Render status bar
    render_status_bar(f, app, chunks[0]);

    // Split main content into task list and details
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Render task list and details
    render_task_list(f, app, main_chunks[0]);
    render_details_panel(f, app, main_chunks[1]);

    // Render help bar
    render_help_bar(f, chunks[2]);

    // Render log modal if active
    if app.show_log_modal {
        render_log_modal(f, app);
    }

    // Render input dialog if in input mode
    if let Some(input_mode) = &app.input_mode {
        let title = match input_mode {
            crate::app::InputMode::AddTask => "Add Task (Enter: submit, Esc: cancel)",
            crate::app::InputMode::EditTask(_) => "Edit Task (Enter: submit, Esc: cancel)",
        };
        let area = input_dialog_rect(f.area());
        render_input_dialog(f, title, &app.text_input, area);
    }
}

fn render_error(f: &mut Frame, error: &str) {
    let error_block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let error_text = Paragraph::new(error)
        .block(error_block)
        .wrap(Wrap { trim: true });

    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(error_text, area);
}

fn render_log_modal(f: &mut Frame, app: &App) {
    if let Some(task_id) = app.get_selected_task_id() {
        let follow_indicator = if app.follow_mode { " [FOLLOW]" } else { "" };
        let title = format!(
            "Logs - Task #{}{} (q/Enter:close, j/k:scroll, f:follow)",
            task_id, follow_indicator
        );

        let log_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if app.follow_mode {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            });

        let output = app.log_content.as_deref().unwrap_or("(Loading logs...)");

        // Calculate area for the log content
        let area = centered_rect(90, 90, f.area());
        let inner_height = area.height.saturating_sub(2) as usize; // Account for borders

        // Split output into lines for scrolling
        let lines: Vec<&str> = output.lines().collect();
        let total_lines = lines.len();

        // Calculate scroll position
        let scroll = if app.follow_mode || app.log_scroll == usize::MAX {
            // Follow mode: show the last lines
            total_lines.saturating_sub(inner_height)
        } else {
            app.log_scroll.min(total_lines.saturating_sub(inner_height))
        };

        // Get visible lines
        let visible_lines: String = lines
            .iter()
            .skip(scroll)
            .take(inner_height)
            .copied()
            .collect::<Vec<&str>>()
            .join("\n");

        let log_text = Paragraph::new(visible_lines).block(log_block);

        f.render_widget(Clear, area);
        f.render_widget(log_text, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn input_dialog_rect(r: Rect) -> Rect {
    // Create a centered dialog that's 80% wide and 3 lines tall
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(60),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(popup_layout[1])[1]
}
