use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

#[derive(Parser, Debug)]
#[command(name = "lazypueue")]
#[command(about = "A lazygit-style TUI for pueue task management", long_about = None)]
struct Args {
    /// Pueue daemon URI
    #[arg(short, long)]
    uri: Option<String>,
}

fn main() -> Result<()> {
    let _args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal);

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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(f.area());

            let main_block = Block::default()
                .title("Pueue Tasks")
                .borders(Borders::ALL);
            let main_paragraph = Paragraph::new("Welcome to lazypueue!\n\nPress 'q' to quit.")
                .block(main_block);
            f.render_widget(main_paragraph, chunks[0]);

            let help_block = Block::default()
                .title("Help")
                .borders(Borders::ALL);
            let help_paragraph = Paragraph::new("q: quit | ?: help").block(help_block);
            f.render_widget(help_paragraph, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
