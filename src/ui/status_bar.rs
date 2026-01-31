use crate::app::App;
use pueue_lib::task::TaskStatus;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(state) = &app.state {
        let tasks = state.tasks.values();

        let running_count = tasks
            .clone()
            .filter(|t| matches!(t.status, TaskStatus::Running { .. }))
            .count();

        let queued_count = tasks
            .clone()
            .filter(|t| matches!(t.status, TaskStatus::Queued { .. }))
            .count();

        let total_count = state.tasks.len();

        // Get default group status
        let default_group = state.groups.get("default");
        let parallel_limit = default_group.map(|g| g.parallel_tasks).unwrap_or(1);

        let pause_status = if let Some(group) = default_group {
            match group.status {
                pueue_lib::state::GroupStatus::Paused => Span::styled(
                    " [PAUSED]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                _ => Span::raw(""),
            }
        } else {
            Span::raw("")
        };

        Line::from(vec![
            Span::styled("Tasks: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} running", running_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("{} queued", queued_count),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::raw(format!("{} total", total_count)),
            Span::raw(" | "),
            Span::styled("Parallel: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}/{}", running_count, parallel_limit)),
            pause_status,
        ])
    } else {
        Line::from("Connecting to pueue daemon...")
    };

    let status =
        Paragraph::new(status_text).block(Block::default().title("Status").borders(Borders::ALL));

    f.render_widget(status, area);
}

pub fn render_help_bar(f: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":navigate | "),
        Span::styled("K", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":kill | "),
        Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":pause/resume | "),
        Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":refresh | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":logs | "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":quit"),
    ]);

    let help =
        Paragraph::new(help_text).block(Block::default().title("Help").borders(Borders::ALL));

    f.render_widget(help, area);
}
