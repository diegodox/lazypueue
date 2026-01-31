use anyhow::Result;
use lazypueue::app::App;
use lazypueue::events;
use lazypueue::pueue_client::PueueClient;
use lazypueue::ui;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "lazypueue")]
#[command(about = "A lazygit-style TUI for pueue task management", long_about = None)]
struct Args {
    /// Pueue daemon URI
    #[arg(short, long)]
    uri: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();
    let mut client = PueueClient::new().await?;

    // Initial fetch
    app.refresh(&mut client).await?;

    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events with timeout for periodic refresh
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = events::handle_key_event(key) {
                    let should_quit = app.handle_action(action, &mut client).await?;
                    if should_quit {
                        break;
                    }
                }
            }
        } else {
            // Timeout - refresh task state
            app.refresh(&mut client).await?;
        }
    }

    Ok(())
}
