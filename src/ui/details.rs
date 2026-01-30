use crate::app::App;
use pueue_lib::task::TaskStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_details_panel(f: &mut Frame, app: &App, area: Rect) {
    if let Some(task_id) = app.get_selected_task_id() {
        let tasks = app.get_task_list();
        if let Some((_, task)) = tasks.iter().find(|(id, _)| *id == task_id) {
            // Split into metadata and output sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(0)])
                .split(area);

            // Render metadata
            render_metadata(f, task_id, task, chunks[0]);

            // Render output
            render_output(f, task, chunks[1]);
        } else {
            render_empty(f, area);
        }
    } else {
        render_empty(f, area);
    }
}

fn render_metadata(f: &mut Frame, task_id: usize, task: &pueue_lib::task::Task, area: Rect) {
    use pueue_lib::task::TaskResult;

    let (status_text, start_time, duration, exit_code) = match &task.status {
        TaskStatus::Running { start, .. } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
            let dur = chrono::Local::now() - *start;
            let dur_str = format_duration(dur.num_seconds());
            (("Running", Color::Green), start_str, dur_str, "-".to_string())
        }
        TaskStatus::Paused { start, .. } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
            let dur = chrono::Local::now() - *start;
            let dur_str = format_duration(dur.num_seconds());
            (("Paused", Color::Cyan), start_str, dur_str, "-".to_string())
        }
        TaskStatus::Done { start, end, result, .. } => {
            let start_str = start.format("%Y-%m-%d %H:%M:%S").to_string();
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
            ((status_label, color), start_str, dur_str, exit_code_str)
        }
        TaskStatus::Queued { .. } => {
            (("Queued", Color::Yellow), "Not started".to_string(), "-".to_string(), "-".to_string())
        }
        TaskStatus::Stashed { .. } => {
            (("Stashed", Color::Gray), "Not started".to_string(), "-".to_string(), "-".to_string())
        }
        TaskStatus::Locked { .. } => {
            (("Locked", Color::Magenta), "Locked".to_string(), "-".to_string(), "-".to_string())
        }
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Task #", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(task_id.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&task.command),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(status_text.0, Style::default().fg(status_text.1)),
        ]),
        Line::from(vec![
            Span::styled("Started: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(start_time),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(duration),
        ]),
        Line::from(vec![
            Span::styled("Exit Code: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(exit_code),
        ]),
    ];

    let metadata = Paragraph::new(lines)
        .block(Block::default().title("Details").borders(Borders::ALL));

    f.render_widget(metadata, area);
}

fn render_output(f: &mut Frame, task: &pueue_lib::task::Task, area: Rect) {
    // For MVP, show a placeholder for output
    // Full log reading will be implemented in next iteration
    let output = match &task.status {
        TaskStatus::Running { .. } => "Task is running...\n(Press Enter to view full logs)".to_string(),
        TaskStatus::Done { .. } => "Task completed.\n(Press Enter to view full logs)".to_string(),
        _ => "No output available yet.".to_string(),
    };

    let output_widget = Paragraph::new(output)
        .block(
            Block::default()
                .title("Output")
                .borders(Borders::ALL)
        )
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
