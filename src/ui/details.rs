use crate::app::{App, TreeSelection};
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    match &app.selection {
        TreeSelection::Group(name) => {
            render_group_details(f, app, name, area);
        }
        TreeSelection::Task(_, task_id) => {
            render_task_details(f, app, *task_id, area);
        }
    }
}

fn render_group_details(f: &mut Frame, app: &App, name: &str, area: Rect) {
    let state = match &app.state {
        Some(s) => s,
        None => {
            render_empty(f, area);
            return;
        }
    };

    let group = match state.groups.get(name) {
        Some(g) => g,
        None => {
            render_empty(f, area);
            return;
        }
    };

    // Count tasks in group by status
    let tasks_in_group: Vec<_> = state
        .tasks
        .iter()
        .filter(|(_, t)| t.group == name)
        .collect();
    let total = tasks_in_group.len();
    let running = tasks_in_group
        .iter()
        .filter(|(_, t)| matches!(t.status, TaskStatus::Running { .. }))
        .count();
    let queued = tasks_in_group
        .iter()
        .filter(|(_, t)| matches!(t.status, TaskStatus::Queued { .. }))
        .count();
    let paused = tasks_in_group
        .iter()
        .filter(|(_, t)| matches!(t.status, TaskStatus::Paused { .. }))
        .count();
    let stashed = tasks_in_group
        .iter()
        .filter(|(_, t)| matches!(t.status, TaskStatus::Stashed { .. }))
        .count();
    let done = tasks_in_group
        .iter()
        .filter(|(_, t)| matches!(t.status, TaskStatus::Done { .. }))
        .count();

    let (status_text, status_color) = match group.status {
        GroupStatus::Running => ("Running", Color::Green),
        GroupStatus::Paused => ("Paused", Color::Red),
        GroupStatus::Reset => ("Reset", Color::Yellow),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Group: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "Parallel Tasks: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(group.parallel_tasks.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Task Summary:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("  Total: "), Span::raw(total.to_string())]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("▶ Running: {}", running),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("● Queued: {}", queued),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("⏸ Paused: {}", paused),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("⊡ Stashed: {}", stashed),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("✓ Done: {}", done),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keybinds:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  p      Pause/resume group"),
        Line::from("  +/-    Change parallel limit"),
        Line::from("  a      Add task to group"),
        Line::from("  c      Clean finished tasks"),
        Line::from("  l/→    Expand / select first task"),
        Line::from("  h/←    Collapse group"),
    ];

    let details = Paragraph::new(lines).block(
        Block::default()
            .title("Group Details")
            .borders(Borders::ALL),
    );

    f.render_widget(details, area);
}

fn render_task_details(f: &mut Frame, app: &App, task_id: usize, area: Rect) {
    let state = match &app.state {
        Some(s) => s,
        None => {
            render_empty(f, area);
            return;
        }
    };

    let task = match state.tasks.get(&task_id) {
        Some(t) => t,
        None => {
            render_empty(f, area);
            return;
        }
    };

    // Split into metadata and output sections (11 lines + 2 for borders = 13)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(13), Constraint::Min(0)])
        .split(area);

    // Render metadata
    render_metadata(f, task_id, task, chunks[0]);

    // Render output
    render_output(f, task, chunks[1]);
}

fn render_metadata(f: &mut Frame, task_id: usize, task: &pueue_lib::task::Task, area: Rect) {
    use pueue_lib::task::TaskResult;

    let (status_text, start_time, end_time, duration, exit_code) = match &task.status {
        TaskStatus::Running { start, .. } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
            let dur = chrono::Local::now() - *start;
            let dur_str = format_duration(dur.num_seconds());
            (
                ("Running", Color::Green),
                start_str,
                "-".to_string(),
                dur_str,
                "-".to_string(),
            )
        }
        TaskStatus::Paused { start, .. } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
            let dur = chrono::Local::now() - *start;
            let dur_str = format_duration(dur.num_seconds());
            (
                ("Paused", Color::Cyan),
                start_str,
                "-".to_string(),
                dur_str,
                "-".to_string(),
            )
        }
        TaskStatus::Done {
            start, end, result, ..
        } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
            let end_str = end.format("%Y-%m-%d %H:%M:%S").to_string();
            let dur = *end - *start;
            let dur_str = format_duration(dur.num_seconds());
            let (status_label, color) = match result {
                TaskResult::Success => ("Success", Color::Green),
                TaskResult::Failed(_) => ("Failed", Color::Red),
                TaskResult::FailedToSpawn(_) => ("Failed to spawn", Color::Red),
                TaskResult::Killed => ("Killed", Color::Magenta),
                TaskResult::Errored => ("Errored", Color::Red),
                TaskResult::DependencyFailed => ("Dependency failed", Color::Red),
            };
            let exit_code_str = match result {
                TaskResult::Failed(code) => code.to_string(),
                TaskResult::Success => "0".to_string(),
                _ => "-".to_string(),
            };
            (
                (status_label, color),
                start_str,
                end_str,
                dur_str,
                exit_code_str,
            )
        }
        TaskStatus::Queued { .. } => (
            ("Queued", Color::Yellow),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ),
        TaskStatus::Stashed { .. } => (
            ("Stashed", Color::Gray),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ),
        TaskStatus::Locked { .. } => (
            ("Locked", Color::Magenta),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ),
    };

    // Format label
    let label_str = task.label.as_deref().unwrap_or("-");

    // Format dependencies
    let deps_str = if task.dependencies.is_empty() {
        "-".to_string()
    } else {
        task.dependencies
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Format created_at
    let created_str = task.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

    // Format path (truncate if too long)
    let path_str = task.path.to_string_lossy();
    let path_display = if path_str.len() > 50 {
        format!("...{}", &path_str[path_str.len() - 47..])
    } else {
        path_str.to_string()
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Task #", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}  ", task_id)),
            Span::styled("Group: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&task.group),
        ]),
        Line::from(vec![
            Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&task.command),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(path_display, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(status_text.0, Style::default().fg(status_text.1)),
            Span::raw("  "),
            Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(task.priority.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Label: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(label_str),
            Span::raw("  "),
            Span::styled(
                "Dependencies: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(deps_str),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(created_str),
        ]),
        Line::from(vec![
            Span::styled("Started: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(start_time),
        ]),
        Line::from(vec![
            Span::styled("Ended: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(end_time),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(duration),
            Span::raw("  "),
            Span::styled("Exit Code: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(exit_code),
        ]),
    ];

    let metadata =
        Paragraph::new(lines).block(Block::default().title("Details").borders(Borders::ALL));

    f.render_widget(metadata, area);
}

fn render_output(f: &mut Frame, task: &pueue_lib::task::Task, area: Rect) {
    // For MVP, show a placeholder for output
    // Full log reading will be implemented in next iteration
    let output = match &task.status {
        TaskStatus::Running { .. } => {
            "Task is running...\n(Press Enter to view full logs)".to_string()
        }
        TaskStatus::Done { .. } => "Task completed.\n(Press Enter to view full logs)".to_string(),
        _ => "No output available yet.".to_string(),
    };

    let output_widget = Paragraph::new(output)
        .block(Block::default().title("Output").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    f.render_widget(output_widget, area);
}

fn render_empty(f: &mut Frame, area: Rect) {
    let empty = Paragraph::new("No task selected")
        .block(Block::default().title("Details").borders(Borders::ALL));
    f.render_widget(empty, area);
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
