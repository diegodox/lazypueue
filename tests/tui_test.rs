use anyhow::Result;
use lazypueue::{app::App, pueue_client::PueueClient, ui};
use ratatui::{backend::TestBackend, Terminal};

#[tokio::test]
async fn test_tui_renders() -> Result<()> {
    // Set up project-local pueue config (use env var if already set by nix shell)
    if std::env::var("PUEUE_CONFIG_PATH").is_err() {
        let config_path = std::path::Path::new(".pueue/pueue.yml");
        if config_path.exists() {
            std::env::set_var("PUEUE_CONFIG_PATH", config_path.canonicalize()?);
        }
    }

    println!("Using config: {:?}", std::env::var("PUEUE_CONFIG_PATH"));

    // Give daemon a moment to be fully ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create a test backend
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // Create app and client
    let mut app = App::new();

    println!("Creating pueue client...");
    let mut client = PueueClient::new().await?;
    println!("Client created successfully");

    // Try to refresh state from daemon
    println!("Attempting to connect to pueue daemon...");
    app.refresh(&mut client).await?;

    if app.error_message.is_none() {
        println!("✓ Connected to daemon on first try");
    } else {
        println!("⚠ First connection failed, retrying...");

        // Wait a moment for connection to stabilize
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Refresh again to ensure we have latest state
        app.refresh(&mut client).await?;

        if app.error_message.is_none() {
            println!("✓ Connected to daemon on retry");
        } else {
            println!("⚠ Daemon not available: {}", app.error_message.as_ref().unwrap());
        }
    }

    // Draw the UI
    terminal.draw(|f| ui::render(f, &app))?;

    // Get the buffer to inspect
    let buffer = terminal.backend().buffer();

    println!("\n=== TUI Buffer (80x24) ===");
    for y in 0..24 {
        let mut line = String::new();
        for x in 0..80 {
            let cell = &buffer[( x, y)];
            line.push_str(cell.symbol());
        }
        println!("{}", line);
    }
    println!("=========================\n");

    // Verify UI rendering based on connection state
    let buffer_string = format!("{:?}", buffer);

    if app.error_message.is_none() {
        // Successfully connected - should show task list
        println!("✓ Daemon connected successfully");

        assert!(app.state.is_some(), "State should be loaded when connected");
        let state = app.state.as_ref().unwrap();
        println!("✓ State loaded: {} tasks", state.tasks.len());

        // UI should show task-related elements
        assert!(
            buffer_string.contains("Tasks") || buffer_string.contains("Status"),
            "UI should render task list or status when connected"
        );
        println!("✓ TUI renders successfully with daemon connection");
    } else {
        // Daemon not available - should show error
        println!("✓ Daemon not available (expected in some environments)");
        println!("   Error: {}", app.error_message.as_ref().unwrap());

        // UI should render error message
        assert!(
            buffer_string.contains("Error") || buffer_string.contains("Failed"),
            "UI should show error message when daemon unavailable"
        );
        println!("✓ TUI renders error message correctly");
    }

    Ok(())
}

#[test]
fn test_app_state_management() {
    let mut app = App::new();

    // Test initial state
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.show_log_modal, false);
    assert!(app.state.is_none());

    println!("✓ App initializes correctly");
}
