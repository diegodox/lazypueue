use anyhow::Result;
use pueue_lib::message::request::*;
use pueue_lib::message::response::*;
use pueue_lib::network::client::Client;
use pueue_lib::settings::Settings;
use pueue_lib::state::State;

pub struct PueueClient {
    client: Client,
}

impl PueueClient {
    pub async fn new() -> Result<Self> {
        let (settings, _) = Settings::read(&None)?;

        eprintln!("Socket path: {}", settings.shared.unix_socket_path().display());
        eprintln!("Use unix socket: {}", settings.shared.use_unix_socket);

        // Convert Shared to ConnectionSettings
        let connection_settings: pueue_lib::network::protocol::ConnectionSettings = settings
            .shared
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to create connection settings: {}", e))?;

        // Use empty secret for Unix socket connections (no TLS)
        let secret = vec![];

        eprintln!("Creating client...");
        // Create the client
        let client = Client::new(connection_settings, &secret, false)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?;

        eprintln!("Client created!");
        Ok(Self { client })
    }

    pub async fn get_state(&mut self) -> Result<State> {
        self.client.send_request(Request::Status).await?;
        let response = self.client.receive_response()
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

    // Log viewing will be implemented in the next iteration
    // For now, users can use `pueue log <task_id>` command
}
