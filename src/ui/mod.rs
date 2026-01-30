mod details;
mod status_bar;
mod task_list;

pub use details::render_details_panel;
pub use status_bar::{render_help_bar, render_status_bar};
pub use task_list::render_task_list;

use crate::app::App;
use crate::pueue_client::PueueClient;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
            Constraint::Length(3),  // Status bar
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Help bar
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
        let log_block = Block::default()
            .title(format!("Logs - Task #{} (Press Enter to close)", task_id))
            .borders(Borders::ALL)
            .style(Style::default());

        // For MVP, show a message that log viewing will be implemented
        // This requires async file reading which we'll add in the next iteration
        let output = "Log viewing will be implemented in the next version.\n\nFor now, use 'pueue log <task_id>' to view logs.";

        let log_text = Paragraph::new(output)
            .block(log_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(90, 90, f.area());
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
