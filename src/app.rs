use anyhow::Result;
use pueue_lib::state::State;
use std::time::Instant;

use crate::pueue_client::PueueClient;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NavigateUp,
    NavigateDown,
    NavigateTop,
    NavigateBottom,
    KillTask,
    TogglePause,
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
    Quit,
}

pub struct App {
    pub state: Option<State>,
    pub selected_index: usize,
    pub last_update: Instant,
    pub show_log_modal: bool,
    pub log_content: Option<String>,
    pub log_scroll: usize,
    pub follow_mode: bool,
    pub error_message: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            selected_index: 0,
            last_update: Instant::now(),
            show_log_modal: false,
            log_content: None,
            log_scroll: 0,
            follow_mode: false,
            error_message: None,
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

                // Adjust selected index if out of bounds
                if let Some(state) = &self.state {
                    let task_count = state.tasks.len();
                    if task_count == 0 {
                        self.selected_index = 0;
                    } else if self.selected_index >= task_count {
                        self.selected_index = task_count.saturating_sub(1);
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to connect to pueue daemon: {}", e));
            }
        }
        Ok(())
    }

    pub async fn handle_action(
        &mut self,
        action: Action,
        client: &mut PueueClient,
    ) -> Result<bool> {
        match action {
            Action::NavigateUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Action::NavigateDown => {
                if let Some(state) = &self.state {
                    let task_count = state.tasks.len();
                    if task_count > 0 && self.selected_index < task_count - 1 {
                        self.selected_index += 1;
                    }
                }
            }
            Action::NavigateTop => {
                self.selected_index = 0;
            }
            Action::NavigateBottom => {
                if let Some(state) = &self.state {
                    let task_count = state.tasks.len();
                    if task_count > 0 {
                        self.selected_index = task_count - 1;
                    }
                }
            }
            Action::KillTask => {
                if let Some(task_id) = self.get_selected_task_id() {
                    client.kill(vec![task_id]).await?;
                    self.refresh(client).await?;
                }
            }
            Action::TogglePause => {
                if let Some(state) = &self.state {
                    if let Some(group) = state.groups.get("default") {
                        match group.status {
                            pueue_lib::state::GroupStatus::Paused => {
                                client.start().await?;
                            }
                            _ => {
                                client.pause().await?;
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
                            use pueue_lib::message::request::TaskToRestart;
                            let task_to_restart = TaskToRestart {
                                task_id,
                                original_command: task.command.clone(),
                                path: task.path.clone(),
                                label: task.label.clone(),
                                priority: task.priority,
                            };
                            if let Err(e) = client.restart(vec![task_to_restart]).await {
                                self.error_message = Some(format!("Failed to restart task: {}", e));
                            } else {
                                self.refresh(client).await?;
                            }
                        }
                    }
                }
            }
            Action::CleanFinished => {
                if let Err(e) = client.clean(false).await {
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
            Action::Quit => {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_selected_task_id(&self) -> Option<usize> {
        self.state.as_ref().and_then(|state| {
            let task_ids: Vec<usize> = state.tasks.keys().copied().collect();
            task_ids.get(self.selected_index).copied()
        })
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
