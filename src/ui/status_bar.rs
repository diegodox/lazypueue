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
        // Get all tasks for overall stats
        let all_tasks = app.get_task_list();
        let running_count = all_tasks
            .iter()
            .filter(|(_, t)| matches!(t.status, TaskStatus::Running { .. }))
            .count();
        let queued_count = all_tasks
            .iter()
            .filter(|(_, t)| matches!(t.status, TaskStatus::Queued { .. }))
            .count();
        let total_count = all_tasks.len();
        let group_count = state.groups.len();

        Line::from(vec![
            Span::styled("Groups: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", group_count)),
            Span::raw(" | "),
            Span::styled("Tasks: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{} run", running_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw("/"),
            Span::styled(
                format!("{} queue", queued_count),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("/"),
            Span::raw(format!("{} total", total_count)),
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
        Span::raw(":nav "),
        Span::styled("h/l", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":fold "),
        Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":add "),
        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":del "),
        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":edit "),
        Span::styled("s/S", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":stash/enq "),
        Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":pause "),
        Span::styled("K", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":kill "),
        Span::styled("R", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":restart "),
        Span::styled("+/-", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":parallel "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":quit"),
    ]);

    let help =
        Paragraph::new(help_text).block(Block::default().title("Help").borders(Borders::ALL));

    f.render_widget(help, area);
}
