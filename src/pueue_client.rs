use anyhow::Result;
use pueue_lib::message::request::{
    AddRequest, CleanRequest, EnqueueRequest, KillRequest, LogRequest, ParallelRequest,
    PauseRequest, Request, RestartRequest, StartRequest, StashRequest, SwitchRequest,
    TaskSelection, TaskToRestart,
};
use pueue_lib::message::response::*;
use pueue_lib::message::EditableTask;
use pueue_lib::network::client::Client;
use pueue_lib::settings::Settings;
use pueue_lib::state::State;
use std::path::PathBuf;

pub struct PueueClient {
    client: Client,
}

impl PueueClient {
    pub async fn new() -> Result<Self> {
        let (settings, _) = Settings::read(&None)?;

        // Read shared secret before consuming settings
        let secret_path = settings.shared.shared_secret_path();
        let secret = if secret_path.exists() {
            std::fs::read(&secret_path)
                .map_err(|e| anyhow::anyhow!("Failed to read shared secret: {}", e))?
        } else {
            // Use empty secret if file doesn't exist (typically for Unix sockets without auth)
            vec![]
        };

        // Convert Shared to ConnectionSettings
        let connection_settings: pueue_lib::network::protocol::ConnectionSettings = settings
            .shared
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to create connection settings: {}", e))?;

        // Create the client
        let client = Client::new(connection_settings, &secret, false)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?;

        Ok(Self { client })
    }

    pub async fn get_state(&mut self) -> Result<State> {
        self.client.send_request(Request::Status).await?;
        let response = self
            .client
            .receive_response()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to receive response: {}", e))?;

        match response {
            Response::Status(state) => Ok(*state),
            Response::Failure(text) => Err(anyhow::anyhow!("Daemon error: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn kill(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Kill(KillRequest {
            tasks: TaskSelection::TaskIds(task_ids),
            signal: None,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to kill task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn pause(&mut self) -> Result<()> {
        let request = Request::Pause(PauseRequest {
            tasks: TaskSelection::Group("default".to_string()),
            wait: false,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to pause daemon: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let request = Request::Start(StartRequest {
            tasks: TaskSelection::Group("default".to_string()),
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to start daemon: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn get_log(&mut self, task_id: usize) -> Result<String> {
        let request = Request::Log(LogRequest {
            tasks: TaskSelection::TaskIds(vec![task_id]),
            send_logs: true,
            lines: None, // Get all lines
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Log(logs) => {
                if let Some(task_log) = logs.get(&task_id) {
                    if let Some(output) = &task_log.output {
                        // Convert bytes to string, handling potential encoding issues
                        Ok(String::from_utf8_lossy(output).to_string())
                    } else {
                        Ok("(No output)".to_string())
                    }
                } else {
                    Ok("(No log found for this task)".to_string())
                }
            }
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to get log: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn restart(&mut self, tasks_info: Vec<TaskToRestart>) -> Result<()> {
        let request = Request::Restart(RestartRequest {
            tasks: tasks_info,
            start_immediately: false,
            stashed: false,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to restart task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn clean(&mut self, successful_only: bool) -> Result<()> {
        let request = Request::Clean(CleanRequest {
            successful_only,
            group: None, // Clean all groups
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to clean tasks: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn add(&mut self, command: String) -> Result<usize> {
        let request = Request::Add(AddRequest {
            command,
            path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            envs: std::collections::HashMap::new(),
            start_immediately: false,
            stashed: false,
            group: "default".to_string(),
            enqueue_at: None,
            dependencies: vec![],
            priority: None,
            label: None,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::AddedTask(added) => Ok(added.task_id),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to add task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn remove(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Remove(task_ids);
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to remove task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn pause_tasks(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Pause(PauseRequest {
            tasks: TaskSelection::TaskIds(task_ids),
            wait: false,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to pause task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn start_tasks(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Start(StartRequest {
            tasks: TaskSelection::TaskIds(task_ids),
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to start task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Request to edit a task. Returns the editable task info if successful.
    pub async fn edit_request(&mut self, task_id: usize) -> Result<EditableTask> {
        let request = Request::EditRequest(vec![task_id]);
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Edit(mut tasks) => {
                if let Some(task) = tasks.pop() {
                    Ok(task)
                } else {
                    Err(anyhow::anyhow!("No task returned for editing"))
                }
            }
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to edit task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Restore the original task state (cancel edit).
    pub async fn edit_restore(&mut self, task_id: usize) -> Result<()> {
        let request = Request::EditRestore(vec![task_id]);
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to restore task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Submit the edited task.
    pub async fn edit_submit(&mut self, task: EditableTask) -> Result<()> {
        let request = Request::EditedTasks(vec![task]);
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to submit edit: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Stash tasks (hold them from execution).
    pub async fn stash(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Stash(StashRequest {
            tasks: TaskSelection::TaskIds(task_ids),
            enqueue_at: None,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to stash task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Enqueue stashed tasks.
    pub async fn enqueue(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let request = Request::Enqueue(EnqueueRequest {
            tasks: TaskSelection::TaskIds(task_ids),
            enqueue_at: None,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to enqueue task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Switch the position of two tasks in the queue.
    pub async fn switch(&mut self, task_id_1: usize, task_id_2: usize) -> Result<()> {
        let request = Request::Switch(SwitchRequest {
            task_id_1,
            task_id_2,
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => Err(anyhow::anyhow!("Failed to switch tasks: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    /// Set the parallel task limit for a group.
    pub async fn parallel(&mut self, group: &str, limit: usize) -> Result<()> {
        let request = Request::Parallel(ParallelRequest {
            parallel_tasks: limit,
            group: group.to_string(),
        });
        self.client.send_request(request).await?;
        let response = self.client.receive_response().await?;

        match response {
            Response::Success(_) => Ok(()),
            Response::Failure(text) => {
                Err(anyhow::anyhow!("Failed to set parallel limit: {}", text))
            }
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }
}
