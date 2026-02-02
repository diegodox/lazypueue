use crate::app::{App, TreeItem, TreeSelection};
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_task_list(f: &mut Frame, app: &App, area: Rect) {
    let tree_items = app.get_tree_items();
    let state = match &app.state {
        Some(s) => s,
        None => {
            let list = List::new::<Vec<ListItem>>(vec![])
                .block(Block::default().title("Tasks").borders(Borders::ALL));
            f.render_widget(list, area);
            return;
        }
    };

    let items: Vec<ListItem> = tree_items
        .iter()
        .map(|item| {
            let is_selected = match (&app.selection, item) {
                (TreeSelection::Group(a), TreeItem::Group(b)) => a == b,
                (TreeSelection::Task(g1, t1), TreeItem::Task(g2, t2)) => g1 == g2 && t1 == t2,
                _ => false,
            };

            match item {
                TreeItem::Group(name) => render_group_item(
                    state,
                    name,
                    app.collapsed_groups.contains(name),
                    is_selected,
                ),
                TreeItem::Task(_group, task_id) => {
                    if let Some(task) = state.tasks.get(task_id) {
                        render_task_item(*task_id, task, is_selected)
                    } else {
                        ListItem::new(Line::from(format!("  ? #{} (unknown)", task_id)))
                    }
                }
            }
        })
        .collect();

    let list = List::new(items).block(Block::default().title("Tasks").borders(Borders::ALL));

    f.render_widget(list, area);
}

fn render_group_item(
    state: &pueue_lib::state::State,
    name: &str,
    is_collapsed: bool,
    is_selected: bool,
) -> ListItem<'static> {
    let group = state.groups.get(name);

    // Count tasks in group
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

    // Collapse indicator
    let indicator = if is_collapsed { "▶" } else { "▼" };

    // Group status indicator
    let (status_indicator, status_color) = match group.map(|g| &g.status) {
        Some(GroupStatus::Paused) => (" [PAUSED]", Color::Red),
        _ => ("", Color::Reset),
    };

    // Parallel limit
    let parallel = group.map(|g| g.parallel_tasks).unwrap_or(1);

    let style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let mut spans = vec![Span::styled(
        format!(
            "{} {} ({}/{}) ∥{}",
            indicator, name, running, total, parallel
        ),
        style,
    )];

    if !status_indicator.is_empty() && !is_selected {
        spans.push(Span::styled(
            status_indicator.to_string(),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ));
    } else if !status_indicator.is_empty() {
        spans.push(Span::styled(status_indicator.to_string(), style));
    }

    ListItem::new(Line::from(spans))
}

fn render_task_item(
    task_id: usize,
    task: &pueue_lib::task::Task,
    is_selected: bool,
) -> ListItem<'static> {
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

    let command = truncate_string(&task.command, 35);

    // Indent with 2 spaces for tasks under groups
    let content = format!("  {} #{:<4} {} {}", icon, task_id, duration, command);

    let style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    ListItem::new(Line::from(Span::styled(content, style)))
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
            TaskResult::Failed(_) | TaskResult::FailedToSpawn(_) | TaskResult::DependencyFailed => {
                ("✗", Color::Red)
            }
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
