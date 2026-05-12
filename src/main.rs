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
    Command,
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // State Management
    let mut mode = Mode::Normal;
    let mut lines: Vec<String> = vec![String::new()]; // Store text as lines
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut command_input = String::new();
    let filename = "work.md";

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(f.size());

            let editor_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(chunks[0]);

            // 1. Line Numbers (Gutter)
            let mut line_nums = String::new();
            for i in 1..=lines.len() {
                line_nums.push_str(&format!("{:3}\n", i));
            }
            f.render_widget(
                Paragraph::new(line_nums).style(Style::default().fg(Color::DarkGray)),
                editor_chunks[0].inner(&Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
            );

            // 2. Syntax Highlighting Logic (Basic)
            let mut display_text = Vec::new();
            for line in &lines {
                if line.starts_with('#') {
                    // Markdown Header
                    display_text.push(Line::from(Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )));
                } else if line.starts_with('\\') {
                    // LaTeX Command
                    display_text.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Yellow),
                    )));
                } else {
                    display_text.push(Line::from(line.as_str()));
                }
            }

            let editor_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" {} ", filename));

            let editor_text = Paragraph::new(display_text).block(editor_block);
            f.render_widget(editor_text, editor_chunks[1]);

            // 3. Status Bar
            let (status_text, status_color) = match mode {
                Mode::Normal => (" NORMAL ", Color::Blue),
                Mode::Insert => (" INSERT ", Color::Green),
                Mode::Command => (" COMMAND ", Color::Magenta),
            };

            let bar_content = if let Mode::Command = mode {
                format!(":{}", command_input)
            } else {
                format!(
                    "{} | {} | Pos: {},{}",
                    status_text,
                    filename,
                    cursor_y + 1,
                    cursor_x
                )
            };

            f.render_widget(
                Paragraph::new(bar_content)
                    .style(Style::default().bg(Color::Indexed(235)).fg(status_color)),
                chunks[1],
            );

            // 4. Set Visual Cursor Position
            if let Mode::Insert = mode {
                // Adjust for borders and gutter
                f.set_cursor(
                    editor_chunks[1].x + cursor_x as u16 + 1,
                    editor_chunks[1].y + cursor_y as u16 + 1,
                );
            }
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
                    // Vim Navigation
                    KeyCode::Char('h') | KeyCode::Left => {
                        if cursor_x > 0 {
                            cursor_x -= 1
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if cursor_x < lines[cursor_y].len() {
                            cursor_x += 1
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if cursor_y < lines.len() - 1 {
                            cursor_y += 1;
                            cursor_x = cursor_x.min(lines[cursor_y].len());
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if cursor_y > 0 {
                            cursor_y -= 1;
                            cursor_x = cursor_x.min(lines[cursor_y].len());
                        }
                    }
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Char(c) => {
                        lines[cursor_y].insert(cursor_x, c);
                        cursor_x += 1;
                    }
                    KeyCode::Backspace => {
                        if cursor_x > 0 {
                            lines[cursor_y].remove(cursor_x - 1);
                            cursor_x -= 1;
                        } else if cursor_y > 0 {
                            // Merge with previous line
                            let current_line = lines.remove(cursor_y);
                            cursor_y -= 1;
                            cursor_x = lines[cursor_y].len();
                            lines[cursor_y].push_str(&current_line);
                        }
                    }
                    KeyCode::Enter => {
                        let remainder = lines[cursor_y].split_off(cursor_x);
                        lines.insert(cursor_y + 1, remainder);
                        cursor_y += 1;
                        cursor_x = 0;
                    }
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Enter => {
                        let content = lines.join("\n");
                        match command_input.as_str() {
                            "w" => {
                                fs::write(filename, content)?;
                            }
                            "md" => {
                                fs::write("temp.md", content)?;
                                let _ = Command::new("pandoc")
                                    .args(["temp.md", "-o", "output.html"])
                                    .spawn();
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
