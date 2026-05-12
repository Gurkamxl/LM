use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;

// 1. Define our Editor Modes
enum Mode {
    Normal,
    Insert,
}

fn main() -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 2. Initial State
    let mut mode = Mode::Normal;
    let mut content = String::new(); // Our text buffer

    loop {
        terminal.draw(|f| {
            // 3. Create the Layout (Main Area vs Status Bar)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Main editor
                    Constraint::Length(1), // Status bar at the bottom
                ])
                .split(f.size());

            // 4. Style the Editor Area (The Neovim Look)
            let editor_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" LVIM - editor.md ");

            let editor_text = Paragraph::new(content.as_str()).block(editor_block);

            f.render_widget(editor_text, chunks[0]);

            // 5. Create the Status Bar
            let mode_info = match mode {
                Mode::Normal => (" NORMAL ", Color::Blue),
                Mode::Insert => (" INSERT ", Color::Green),
            };

            let status_line = Line::from(vec![
                Span::styled(
                    mode_info.0,
                    Style::default()
                        .bg(mode_info.1)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" | "),
                Span::raw("Markdown/LaTeX Editor"),
            ]);

            let status_bar =
                Paragraph::new(status_line).style(Style::default().bg(Color::Indexed(235))); // Dark charcoal color

            f.render_widget(status_bar, chunks[1]);
        })?;

        // 6. Handle Input based on Mode
        if let Event::Key(key) = event::read()? {
            // Ignore key "release" events (common on Windows)
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('i') => mode = Mode::Insert, // Switch to Insert
                    KeyCode::Char('q') => break,               // Quit
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Esc => mode = Mode::Normal, // Back to Normal
                    KeyCode::Char(c) => content.push(c), // Add characters
                    KeyCode::Backspace => {
                        content.pop();
                    } // Delete
                    KeyCode::Enter => content.push('\n'), // New line
                    _ => {}
                },
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
