use crate::app::App;
use pueue_lib::task::TaskStatus;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_task_list(f: &mut Frame, app: &App, area: Rect) {
    let tasks = app.get_task_list();

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(index, (id, task))| {
            let (icon, color) = get_status_icon_and_color(&task.status);

            let duration = match &task.status {
                TaskStatus::Running { start, .. } | TaskStatus::Paused { start, .. } => {
                    let now = chrono::Local::now();
                    let duration = now - *start;
                    format!("{:>5}s", duration.num_seconds())
                }
                TaskStatus::Done { start, end, .. } => {
                    let duration = *end - *start;
                    format!("{:>5}s", duration.num_seconds())
                }
                _ => "    -".to_string(),
            };

            let command = truncate_string(&task.command, 40);

            let content = format!("{} #{:<4} {} {}", icon, id, duration, command);

            let style = if index == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
        );

    f.render_widget(list, area);
}

fn get_status_icon_and_color(status: &TaskStatus) -> (&str, Color) {
    use pueue_lib::task::TaskResult;

    match status {
        TaskStatus::Running { .. } => ("▶", Color::Green),
        TaskStatus::Queued { .. } => ("●", Color::Yellow),
        TaskStatus::Paused { .. } => ("⏸", Color::Cyan),
        TaskStatus::Stashed { .. } => ("⊡", Color::Gray),
        TaskStatus::Done { result, .. } => match result {
            TaskResult::Success => ("✓", Color::Green),
            TaskResult::Failed(_) | TaskResult::FailedToSpawn(_) | TaskResult::DependencyFailed => ("✗", Color::Red),
            TaskResult::Killed => ("⊠", Color::Magenta),
            TaskResult::Errored => ("⚠", Color::Red),
        },
        TaskStatus::Locked { .. } => ("🔒", Color::Magenta),
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
