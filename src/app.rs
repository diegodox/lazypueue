use anyhow::Result;
use pueue_lib::message::EditableTask;
use pueue_lib::state::State;
use pueue_lib::task::TaskStatus;
use std::collections::HashSet;
use std::time::Instant;

use crate::pueue_client::PueueClient;
use crate::ui::TextInput;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NavigateUp,
    NavigateDown,
    NavigateTop,
    NavigateBottom,
    KillTask,
    TogglePause,
    ToggleTaskPause,
    Refresh,
    ViewLogs,
    CloseLogs,
    RestartTask,
    CleanFinished,
    FollowLogs,
    ScrollLogUp,
    ScrollLogDown,
    ScrollLogPageUp,
    ScrollLogPageDown,
    // Input mode actions
    StartAddTask,
    StartEditTask,
    RemoveTask,
    SubmitInput,
    CancelInput,
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
    // Phase 2: Power features
    StashTask,
    EnqueueTask,
    SwitchUp,
    SwitchDown,
    IncreaseParallel,
    DecreaseParallel,
    // Tree navigation
    CollapseGroup,
    ExpandGroup,
    // Confirmation actions
    ConfirmAction,
    CancelConfirm,
    Quit,
}

/// Mode for text input dialogs
#[derive(Debug, Clone)]
pub enum InputMode {
    AddTask,
    EditTask(EditableTask),
}

/// Tree selection - either a group header or a task within a group
#[derive(Debug, Clone, PartialEq)]
pub enum TreeSelection {
    Group(String),       // Group name selected
    Task(String, usize), // (group_name, task_id) selected
}

/// Item in the flattened tree view for navigation
#[derive(Debug, Clone)]
pub enum TreeItem {
    Group(String),       // Group header
    Task(String, usize), // (group_name, task_id)
}

pub struct App {
    pub state: Option<State>,
    pub last_update: Instant,
    pub show_log_modal: bool,
    pub log_content: Option<String>,
    pub log_scroll: usize,
    pub follow_mode: bool,
    pub error_message: Option<String>,
    // Input mode state
    pub input_mode: Option<InputMode>,
    pub text_input: TextInput,
    // Confirmation dialog state
    pub confirm_delete: Option<usize>,
    // Tree view state
    pub selection: TreeSelection,
    pub collapsed_groups: HashSet<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            last_update: Instant::now(),
            show_log_modal: false,
            log_content: None,
            log_scroll: 0,
            follow_mode: false,
            error_message: None,
            input_mode: None,
            text_input: TextInput::new(),
            confirm_delete: None,
            selection: TreeSelection::Group("default".to_string()),
            collapsed_groups: HashSet::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn refresh(&mut self, client: &mut PueueClient) -> Result<()> {
        match client.get_state().await {
            Ok(state) => {
                self.state = Some(state);
                self.error_message = None;
                self.last_update = Instant::now();

                // Validate selection is still valid
                self.validate_selection();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to connect to pueue daemon: {}", e));
            }
        }
        Ok(())
    }

    /// Ensure current selection is still valid, adjust if needed
    fn validate_selection(&mut self) {
        let tree_items = self.get_tree_items();
        if tree_items.is_empty() {
            // No items - select default group
            self.selection = TreeSelection::Group("default".to_string());
            return;
        }

        // Check if current selection exists in tree
        let is_valid = match &self.selection {
            TreeSelection::Group(name) => tree_items
                .iter()
                .any(|item| matches!(item, TreeItem::Group(g) if g == name)),
            TreeSelection::Task(group, task_id) => tree_items
                .iter()
                .any(|item| matches!(item, TreeItem::Task(g, id) if g == group && id == task_id)),
        };

        if !is_valid {
            // Selection no longer valid - select first item
            match &tree_items[0] {
                TreeItem::Group(name) => {
                    self.selection = TreeSelection::Group(name.clone());
                }
                TreeItem::Task(group, task_id) => {
                    self.selection = TreeSelection::Task(group.clone(), *task_id);
                }
            }
        }
    }

    pub async fn handle_action(
        &mut self,
        action: Action,
        client: &mut PueueClient,
    ) -> Result<bool> {
        match action {
            Action::NavigateUp => {
                let tree_items = self.get_tree_items();
                if let Some(current_pos) = self.get_selection_position(&tree_items) {
                    if current_pos > 0 {
                        self.select_tree_item(&tree_items[current_pos - 1]);
                    }
                }
            }
            Action::NavigateDown => {
                let tree_items = self.get_tree_items();
                if let Some(current_pos) = self.get_selection_position(&tree_items) {
                    if current_pos + 1 < tree_items.len() {
                        self.select_tree_item(&tree_items[current_pos + 1]);
                    }
                }
            }
            Action::NavigateTop => {
                let tree_items = self.get_tree_items();
                if let Some(first) = tree_items.first() {
                    self.select_tree_item(first);
                }
            }
            Action::NavigateBottom => {
                let tree_items = self.get_tree_items();
                if let Some(last) = tree_items.last() {
                    self.select_tree_item(last);
                }
            }
            Action::KillTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    client.kill(vec![task_id]).await?;
                    self.refresh(client).await?;
                }
            }
            Action::TogglePause => {
                // When group is selected, pause/resume the group
                // When task is selected, pause/resume that task's group
                let group_name = match &self.selection {
                    TreeSelection::Group(name) => name.clone(),
                    TreeSelection::Task(group, _) => group.clone(),
                };
                if let Some(state) = &self.state {
                    if let Some(group) = state.groups.get(&group_name) {
                        match group.status {
                            pueue_lib::state::GroupStatus::Paused => {
                                client.start_group(&group_name).await?;
                            }
                            _ => {
                                client.pause_group(&group_name).await?;
                            }
                        }
                    }
                    self.refresh(client).await?;
                }
            }
            Action::Refresh => {
                self.refresh(client).await?;
            }
            Action::ViewLogs => {
                if !self.show_log_modal {
                    // Opening logs - fetch the content
                    if let Some(task_id) = self.get_selected_task_id() {
                        match client.get_log(task_id).await {
                            Ok(content) => {
                                self.log_content = Some(content);
                                self.log_scroll = 0;
                                self.show_log_modal = true;
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to get logs: {}", e));
                            }
                        }
                    }
                } else {
                    // Closing logs
                    self.show_log_modal = false;
                    self.log_content = None;
                    self.follow_mode = false;
                }
            }
            Action::CloseLogs => {
                self.show_log_modal = false;
                self.log_content = None;
                self.log_scroll = 0;
                self.follow_mode = false;
            }
            Action::ScrollLogUp => {
                if self.log_scroll > 0 {
                    self.log_scroll = self.log_scroll.saturating_sub(1);
                }
            }
            Action::ScrollLogDown => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            Action::ScrollLogPageUp => {
                self.log_scroll = self.log_scroll.saturating_sub(20);
            }
            Action::ScrollLogPageDown => {
                self.log_scroll = self.log_scroll.saturating_add(20);
            }
            Action::RestartTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        if let Some(task) = state.tasks.get(&task_id) {
                            // Restart by creating a new task copy at end of queue (default pueue behavior)
                            use crate::pueue_client::RestartOptions;
                            let opts = RestartOptions {
                                command: task.command.clone(),
                                path: task.path.clone(),
                                envs: task.envs.clone(),
                                group: task.group.clone(),
                                priority: Some(task.priority),
                                label: task.label.clone(),
                            };
                            if let Err(e) = client.restart(opts).await {
                                self.error_message = Some(format!("Failed to restart task: {}", e));
                            } else {
                                self.refresh(client).await?;
                            }
                        }
                    }
                }
            }
            Action::CleanFinished => {
                // Clean currently selected group (or task's group)
                let group_name = match &self.selection {
                    TreeSelection::Group(name) => Some(name.as_str()),
                    TreeSelection::Task(group, _) => Some(group.as_str()),
                };
                if let Err(e) = client.clean(false, group_name).await {
                    self.error_message = Some(format!("Failed to clean tasks: {}", e));
                } else {
                    self.refresh(client).await?;
                }
            }
            Action::FollowLogs => {
                if let Some(task_id) = self.get_selected_task_id() {
                    // Toggle follow mode or open logs in follow mode
                    if self.show_log_modal {
                        self.follow_mode = !self.follow_mode;
                    } else {
                        match client.get_log(task_id).await {
                            Ok(content) => {
                                self.log_content = Some(content);
                                // Start at the end for follow mode
                                self.log_scroll = usize::MAX;
                                self.show_log_modal = true;
                                self.follow_mode = true;
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to get logs: {}", e));
                            }
                        }
                    }
                }
            }
            Action::ToggleTaskPause => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        if let Some(task) = state.tasks.get(&task_id) {
                            match &task.status {
                                TaskStatus::Paused { .. } => {
                                    if let Err(e) = client.start_tasks(vec![task_id]).await {
                                        self.error_message =
                                            Some(format!("Failed to resume task: {}", e));
                                    }
                                }
                                TaskStatus::Running { .. } => {
                                    if let Err(e) = client.pause_tasks(vec![task_id]).await {
                                        self.error_message =
                                            Some(format!("Failed to pause task: {}", e));
                                    }
                                }
                                TaskStatus::Queued { .. } => {
                                    // Start queued task immediately
                                    if let Err(e) = client.start_tasks(vec![task_id]).await {
                                        self.error_message =
                                            Some(format!("Failed to start task: {}", e));
                                    }
                                }
                                TaskStatus::Stashed { .. } => {
                                    // Force-start stashed task (like 'pueue start <id>')
                                    if let Err(e) = client.start_tasks(vec![task_id]).await {
                                        self.error_message =
                                            Some(format!("Failed to start task: {}", e));
                                    }
                                }
                                _ => {
                                    // Can't pause/resume completed tasks
                                }
                            }
                            self.refresh(client).await?;
                        }
                    }
                }
            }
            Action::StartAddTask => {
                self.text_input.clear();
                self.input_mode = Some(InputMode::AddTask);
            }
            Action::StartEditTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    match client.edit_request(task_id).await {
                        Ok(editable) => {
                            self.text_input =
                                TextInput::with_value(editable.original_command.clone());
                            self.input_mode = Some(InputMode::EditTask(editable));
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to edit task: {}", e));
                        }
                    }
                }
            }
            Action::RemoveTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        if let Some(task) = state.tasks.get(&task_id) {
                            // Only allow removing non-running tasks
                            if !matches!(task.status, TaskStatus::Running { .. }) {
                                // Set confirmation state instead of immediate delete
                                self.confirm_delete = Some(task_id);
                            }
                        }
                    }
                }
            }
            Action::ConfirmAction => {
                if let Some(task_id) = self.confirm_delete.take() {
                    if let Err(e) = client.remove(vec![task_id]).await {
                        self.error_message = Some(format!("Failed to remove task: {}", e));
                    } else {
                        self.refresh(client).await?;
                    }
                }
            }
            Action::CancelConfirm => {
                self.confirm_delete = None;
            }
            Action::SubmitInput => {
                if let Some(mode) = self.input_mode.take() {
                    let command = self.text_input.value.clone();
                    if !command.trim().is_empty() {
                        match mode {
                            InputMode::AddTask => {
                                // Add to currently selected group (or task's group)
                                let group = match &self.selection {
                                    TreeSelection::Group(name) => name.as_str(),
                                    TreeSelection::Task(group, _) => group.as_str(),
                                };
                                match client.add(command, group).await {
                                    Ok(_task_id) => {
                                        self.refresh(client).await?;
                                    }
                                    Err(e) => {
                                        self.error_message =
                                            Some(format!("Failed to add task: {}", e));
                                    }
                                }
                            }
                            InputMode::EditTask(mut editable) => {
                                editable.original_command = command;
                                if let Err(e) = client.edit_submit(editable).await {
                                    self.error_message =
                                        Some(format!("Failed to save edit: {}", e));
                                } else {
                                    self.refresh(client).await?;
                                }
                            }
                        }
                    }
                    self.text_input.clear();
                }
            }
            Action::CancelInput => {
                if let Some(mode) = self.input_mode.take() {
                    // If editing, restore the original task state
                    if let InputMode::EditTask(editable) = mode {
                        let _ = client.edit_restore(editable.id).await;
                    }
                    self.text_input.clear();
                }
            }
            Action::InputChar(c) => {
                self.text_input.insert(c);
            }
            Action::InputBackspace => {
                self.text_input.delete_char();
            }
            Action::InputDelete => {
                self.text_input.delete_forward();
            }
            Action::InputLeft => {
                self.text_input.move_left();
            }
            Action::InputRight => {
                self.text_input.move_right();
            }
            Action::InputHome => {
                self.text_input.move_start();
            }
            Action::InputEnd => {
                self.text_input.move_end();
            }
            Action::StashTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        if let Some(task) = state.tasks.get(&task_id) {
                            // Can only stash queued tasks
                            if matches!(task.status, TaskStatus::Queued { .. }) {
                                if let Err(e) = client.stash(vec![task_id]).await {
                                    self.error_message =
                                        Some(format!("Failed to stash task: {}", e));
                                } else {
                                    self.refresh(client).await?;
                                }
                            }
                        }
                    }
                }
            }
            Action::EnqueueTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        if let Some(task) = state.tasks.get(&task_id) {
                            // Can only enqueue stashed tasks
                            if matches!(task.status, TaskStatus::Stashed { .. }) {
                                if let Err(e) = client.enqueue(vec![task_id]).await {
                                    self.error_message =
                                        Some(format!("Failed to enqueue task: {}", e));
                                } else {
                                    self.refresh(client).await?;
                                }
                            }
                        }
                    }
                }
            }
            Action::SwitchUp => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        let tasks = self.get_task_list();
                        // Find the task before this one that can be switched
                        if let Some(pos) = tasks.iter().position(|(id, _)| *id == task_id) {
                            if pos > 0 {
                                let other_id = tasks[pos - 1].0;
                                // Validate both tasks can be switched (queued or stashed only)
                                let task1 = state.tasks.get(&task_id);
                                let task2 = state.tasks.get(&other_id);
                                if let (Some(t1), Some(t2)) = (task1, task2) {
                                    let can_switch = |s: &TaskStatus| {
                                        matches!(
                                            s,
                                            TaskStatus::Queued { .. } | TaskStatus::Stashed { .. }
                                        )
                                    };
                                    if can_switch(&t1.status) && can_switch(&t2.status) {
                                        if let Err(e) = client.switch(task_id, other_id).await {
                                            self.error_message =
                                                Some(format!("Failed to switch tasks: {}", e));
                                        } else {
                                            self.refresh(client).await?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Action::SwitchDown => {
                if let Some(task_id) = self.get_selected_task_id() {
                    if let Some(state) = &self.state {
                        let tasks = self.get_task_list();
                        // Find the task after this one that can be switched
                        if let Some(pos) = tasks.iter().position(|(id, _)| *id == task_id) {
                            if pos < tasks.len() - 1 {
                                let other_id = tasks[pos + 1].0;
                                // Validate both tasks can be switched (queued or stashed only)
                                let task1 = state.tasks.get(&task_id);
                                let task2 = state.tasks.get(&other_id);
                                if let (Some(t1), Some(t2)) = (task1, task2) {
                                    let can_switch = |s: &TaskStatus| {
                                        matches!(
                                            s,
                                            TaskStatus::Queued { .. } | TaskStatus::Stashed { .. }
                                        )
                                    };
                                    if can_switch(&t1.status) && can_switch(&t2.status) {
                                        if let Err(e) = client.switch(task_id, other_id).await {
                                            self.error_message =
                                                Some(format!("Failed to switch tasks: {}", e));
                                        } else {
                                            self.refresh(client).await?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Action::IncreaseParallel => {
                let group_name = match &self.selection {
                    TreeSelection::Group(name) => name.clone(),
                    TreeSelection::Task(group, _) => group.clone(),
                };
                if let Some(state) = &self.state {
                    if let Some(group) = state.groups.get(&group_name) {
                        let new_limit = group.parallel_tasks + 1;
                        if let Err(e) = client.parallel(&group_name, new_limit).await {
                            self.error_message =
                                Some(format!("Failed to increase parallel: {}", e));
                        } else {
                            self.refresh(client).await?;
                        }
                    }
                }
            }
            Action::DecreaseParallel => {
                let group_name = match &self.selection {
                    TreeSelection::Group(name) => name.clone(),
                    TreeSelection::Task(group, _) => group.clone(),
                };
                if let Some(state) = &self.state {
                    if let Some(group) = state.groups.get(&group_name) {
                        if group.parallel_tasks > 1 {
                            let new_limit = group.parallel_tasks - 1;
                            if let Err(e) = client.parallel(&group_name, new_limit).await {
                                self.error_message =
                                    Some(format!("Failed to decrease parallel: {}", e));
                            } else {
                                self.refresh(client).await?;
                            }
                        }
                    }
                }
            }
            Action::CollapseGroup => {
                match &self.selection {
                    TreeSelection::Group(name) => {
                        // If group is selected, toggle collapse
                        if self.collapsed_groups.contains(name) {
                            self.collapsed_groups.remove(name);
                        } else {
                            self.collapsed_groups.insert(name.clone());
                        }
                    }
                    TreeSelection::Task(group, _) => {
                        // If task is selected, go to parent group
                        self.selection = TreeSelection::Group(group.clone());
                    }
                }
            }
            Action::ExpandGroup => {
                match &self.selection {
                    TreeSelection::Group(name) => {
                        if self.collapsed_groups.contains(name) {
                            // Expand the group
                            self.collapsed_groups.remove(name);
                        } else {
                            // Already expanded - select first task if any
                            if let Some(state) = &self.state {
                                let mut tasks_in_group: Vec<_> = state
                                    .tasks
                                    .iter()
                                    .filter(|(_, t)| &t.group == name)
                                    .map(|(id, _)| *id)
                                    .collect();
                                tasks_in_group.sort();
                                if let Some(first_task_id) = tasks_in_group.first() {
                                    self.selection =
                                        TreeSelection::Task(name.clone(), *first_task_id);
                                }
                            }
                        }
                    }
                    TreeSelection::Task(_, task_id) => {
                        // Task selected - view logs
                        match client.get_log(*task_id).await {
                            Ok(content) => {
                                self.log_content = Some(content);
                                self.log_scroll = 0;
                                self.show_log_modal = true;
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to get logs: {}", e));
                            }
                        }
                    }
                }
            }
            Action::Quit => {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_selected_task_id(&self) -> Option<usize> {
        match &self.selection {
            TreeSelection::Task(_, task_id) => Some(*task_id),
            TreeSelection::Group(_) => None,
        }
    }

    /// Get the group name of the current selection
    pub fn get_selected_group(&self) -> &str {
        match &self.selection {
            TreeSelection::Group(name) => name,
            TreeSelection::Task(group, _) => group,
        }
    }

    /// Build the flattened tree of visible items for navigation
    pub fn get_tree_items(&self) -> Vec<TreeItem> {
        let mut items = Vec::new();
        let groups = self.get_group_list();

        if let Some(state) = &self.state {
            for group_name in groups {
                // Add the group header
                items.push(TreeItem::Group(group_name.clone()));

                // If not collapsed, add tasks in this group
                if !self.collapsed_groups.contains(&group_name) {
                    let mut tasks_in_group: Vec<_> = state
                        .tasks
                        .iter()
                        .filter(|(_, t)| t.group == group_name)
                        .map(|(id, _)| *id)
                        .collect();
                    tasks_in_group.sort();
                    for task_id in tasks_in_group {
                        items.push(TreeItem::Task(group_name.clone(), task_id));
                    }
                }
            }
        }

        items
    }

    /// Find the position of current selection in tree items
    fn get_selection_position(&self, items: &[TreeItem]) -> Option<usize> {
        items.iter().position(|item| match (&self.selection, item) {
            (TreeSelection::Group(a), TreeItem::Group(b)) => a == b,
            (TreeSelection::Task(g1, t1), TreeItem::Task(g2, t2)) => g1 == g2 && t1 == t2,
            _ => false,
        })
    }

    /// Select a tree item
    fn select_tree_item(&mut self, item: &TreeItem) {
        self.selection = match item {
            TreeItem::Group(name) => TreeSelection::Group(name.clone()),
            TreeItem::Task(group, task_id) => TreeSelection::Task(group.clone(), *task_id),
        };
    }

    pub fn get_task_list(&self) -> Vec<(usize, &pueue_lib::task::Task)> {
        if let Some(state) = &self.state {
            let mut tasks: Vec<_> = state.tasks.iter().map(|(id, task)| (*id, task)).collect();
            tasks.sort_by_key(|(id, _)| *id);
            tasks
        } else {
            Vec::new()
        }
    }

    /// Get list of all group names, sorted alphabetically with "default" first
    pub fn get_group_list(&self) -> Vec<String> {
        if let Some(state) = &self.state {
            let mut groups: Vec<_> = state.groups.keys().cloned().collect();
            groups.sort();
            // Move "default" to front if present
            if let Some(pos) = groups.iter().position(|g| g == "default") {
                groups.remove(pos);
                groups.insert(0, "default".to_string());
            }
            groups
        } else {
            Vec::new()
        }
    }

    pub async fn refresh_logs(&mut self, client: &mut PueueClient) -> Result<()> {
        if self.follow_mode {
            if let Some(task_id) = self.get_selected_task_id() {
                match client.get_log(task_id).await {
                    Ok(content) => {
                        self.log_content = Some(content);
                        // Keep scroll at the end for follow mode
                        self.log_scroll = usize::MAX;
                    }
                    Err(_) => {
                        // Silently ignore errors during follow refresh
                    }
                }
            }
        }
        Ok(())
    }
}
