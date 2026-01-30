use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::State;

pub struct PueueClient {
    settings: Settings,
}

impl PueueClient {
    pub fn new() -> Result<Self> {
        let (settings, _) = Settings::read(&None)?;
        Ok(Self { settings })
    }

    async fn send_message(&mut self, message: Message) -> Result<Message> {
        let mut stream = get_client_stream(&self.settings.shared).await?;
        send_message(message, &mut stream).await?;
        let response = receive_message(&mut stream).await?;
        Ok(response)
    }

    pub async fn get_state(&mut self) -> Result<State> {
        let message = Message::Status;
        let response = self.send_message(message).await?;

        match response {
            Message::StatusResponse(state) => Ok(*state),
            Message::Failure(text) => Err(anyhow::anyhow!("Daemon error: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn kill(&mut self, task_ids: Vec<usize>) -> Result<()> {
        let kill_message = KillMessage {
            tasks: TaskSelection::TaskIds(task_ids),
            signal: None,
        };
        let message = Message::Kill(kill_message);
        let response = self.send_message(message).await?;

        match response {
            Message::Success(_) => Ok(()),
            Message::Failure(text) => Err(anyhow::anyhow!("Failed to kill task: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn pause(&mut self) -> Result<()> {
        let pause_message = PauseMessage {
            tasks: TaskSelection::Group("default".to_string()),
            wait: false,
        };
        let message = Message::Pause(pause_message);
        let response = self.send_message(message).await?;

        match response {
            Message::Success(_) => Ok(()),
            Message::Failure(text) => Err(anyhow::anyhow!("Failed to pause daemon: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let start_message = StartMessage {
            tasks: TaskSelection::Group("default".to_string()),
        };
        let message = Message::Start(start_message);
        let response = self.send_message(message).await?;

        match response {
            Message::Success(_) => Ok(()),
            Message::Failure(text) => Err(anyhow::anyhow!("Failed to start daemon: {}", text)),
            _ => Err(anyhow::anyhow!("Unexpected response from daemon")),
        }
    }

    // Log viewing will be implemented in the next iteration
    // For now, users can use `pueue log <task_id>` command
}
