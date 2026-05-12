use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::fs;
use std::io::stdout;
use std::process::Command;

enum Mode {
    Normal,
    Insert,
    Command, // New Mode for exporting
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut mode = Mode::Normal;
    let mut content = String::new();
    let mut command_input = String::new(); // For typing :commands
    let filename = "work.md";

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(f.size());

            // 1. Editor Rendering (Gutter + Text)
            let editor_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(chunks[0]);

            let line_count = content.lines().count().max(1);
            let mut line_nums = String::new();
            for i in 1..=line_count {
                line_nums.push_str(&format!("{:3}\n", i));
            }

            f.render_widget(
                Paragraph::new(line_nums).style(Style::default().fg(Color::DarkGray)),
                editor_chunks[0].inner(&Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
            );
            f.render_widget(
                Paragraph::new(content.as_str()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" {} ", filename)),
                ),
                editor_chunks[1],
            );

            // 2. Status Bar & Command Line
            let (status_text, status_color) = match mode {
                Mode::Normal => (" NORMAL ", Color::Blue),
                Mode::Insert => (" INSERT ", Color::Green),
                Mode::Command => (" COMMAND ", Color::Magenta),
            };

            let bar_content = if let Mode::Command = mode {
                format!(":{}", command_input)
            } else {
                format!("{} | {} | Lines: {}", status_text, filename, line_count)
            };

            f.render_widget(
                Paragraph::new(bar_content)
                    .style(Style::default().bg(Color::Indexed(235)).fg(status_color)),
                chunks[1],
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('i') => mode = Mode::Insert,
                    KeyCode::Char(':') => {
                        mode = Mode::Command;
                        command_input.clear();
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Char(c) => content.push(c),
                    KeyCode::Backspace => {
                        content.pop();
                    }
                    KeyCode::Enter => content.push('\n'),
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Enter => {
                        match command_input.as_str() {
                            "w" => {
                                fs::write(filename, &content)?;
                            }
                            "md" => {
                                // Convert Markdown to HTML using Pandoc
                                fs::write("temp.md", &content)?;
                                Command::new("pandoc")
                                    .args(["temp.md", "-o", "output.html"])
                                    .spawn()?;
                            }
                            "tex" => {
                                // Compile LaTeX to PDF
                                fs::write("temp.tex", &content)?;
                                Command::new("pdflatex").arg("temp.tex").spawn()?;
                            }
                            _ => {}
                        }
                        mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => command_input.push(c),
                    KeyCode::Backspace => {
                        command_input.pop();
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
