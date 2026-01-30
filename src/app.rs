use anyhow::Result;
use pueue_lib::state::State;
use std::time::Instant;

use crate::pueue_client::PueueClient;

#[derive(Debug)]
pub enum Action {
    NavigateUp,
    NavigateDown,
    NavigateTop,
    NavigateBottom,
    KillTask,
    PauseDaemon,
    ResumeDaemon,
    Refresh,
    ViewLogs,
    Quit,
}

pub struct App {
    pub state: Option<State>,
    pub selected_index: usize,
    pub last_update: Instant,
    pub show_log_modal: bool,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            selected_index: 0,
            last_update: Instant::now(),
            show_log_modal: false,
            error_message: None,
        }
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

    pub async fn handle_action(&mut self, action: Action, client: &mut PueueClient) -> Result<bool> {
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
            Action::PauseDaemon => {
                // Toggle pause/resume for default group
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
            Action::ResumeDaemon => {
                client.start().await?;
                self.refresh(client).await?;
            }
            Action::Refresh => {
                self.refresh(client).await?;
            }
            Action::ViewLogs => {
                self.show_log_modal = !self.show_log_modal;
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
}
