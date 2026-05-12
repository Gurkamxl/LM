use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;

fn main() -> Result<()> {
    // 1. SETUP: Take over the terminal screen
    enable_raw_mode()?; // This allows us to read keys one by one
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 2. THE APP LOOP
    loop {
        // Draw the UI
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title(" My Rust Editor (Vim Style) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
            f.render_widget(block, size);
        })?;

        // Handle Input
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break; // Quit if 'q' is pressed
            }
        }
    }

    // 3. CLEANUP: Give the terminal back to the user
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
