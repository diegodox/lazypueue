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
        // Get tasks filtered by current group for counts
        let filtered_tasks = app.get_task_list();
        let running_count = filtered_tasks
            .iter()
            .filter(|(_, t)| matches!(t.status, TaskStatus::Running { .. }))
            .count();
        let queued_count = filtered_tasks
            .iter()
            .filter(|(_, t)| matches!(t.status, TaskStatus::Queued { .. }))
            .count();
        let total_count = filtered_tasks.len();

        // Get current group info
        let group_name = app.current_group_display();
        let current_group = app.current_group.as_ref().and_then(|g| state.groups.get(g));

        // For "All" view, show aggregate; for specific group, show that group's info
        let (parallel_limit, pause_status) = if let Some(group) = current_group {
            let pause = match group.status {
                pueue_lib::state::GroupStatus::Paused => Span::styled(
                    " [PAUSED]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                _ => Span::raw(""),
            };
            (group.parallel_tasks, pause)
        } else {
            // "All" view - show default group's parallel limit, no pause indicator
            let default_group = state.groups.get("default");
            let limit = default_group.map(|g| g.parallel_tasks).unwrap_or(1);
            (limit, Span::raw(""))
        };

        Line::from(vec![
            Span::styled("Group: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("[{}]", group_name),
                Style::default().fg(Color::Cyan),
            ),
            pause_status,
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
            Span::raw(" | "),
            Span::styled("Parallel: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", parallel_limit)),
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
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":group "),
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
        Span::styled("l", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":logs "),
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
