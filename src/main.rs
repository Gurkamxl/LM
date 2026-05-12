use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::fs;
use std::io::stdout;

// 1. Define a Theme structure for the "LazyVim" look
struct Theme {
    background: Color,
    gutter_fg: Color,
    editor_border: Color,
    status_normal: Color,
    status_insert: Color,
    status_command: Color,
    syntax_header: Color,
    syntax_latex: Color,
    text: Color,
}

const TOKYO_NIGHT: Theme = Theme {
    background: Color::Indexed(234),     // Very dark blue/black
    gutter_fg: Color::Indexed(243),      // Grey
    editor_border: Color::Indexed(67),   // Muted Blue
    status_normal: Color::Indexed(110),  // Steel Blue
    status_insert: Color::Indexed(150),  // Soft Green
    status_command: Color::Indexed(175), // Pink/Magenta
    syntax_header: Color::Indexed(216),  // Orange/Peach
    syntax_latex: Color::Indexed(117),   // Sky Blue
    text: Color::Indexed(253),           // Off-white
};

enum Mode {
    Normal,
    Insert,
    Command,
    Search, // New Mode
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let theme = TOKYO_NIGHT;
    let mut mode = Mode::Normal;
    let mut lines: Vec<String> = vec![String::new()];
    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut command_input = String::new();
    let mut search_query = String::new();

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

            // --- Gutter ---
            let mut line_nums = String::new();
            for i in 1..=lines.len() {
                line_nums.push_str(&format!("{:3}\n", i));
            }
            f.render_widget(
                Paragraph::new(line_nums).style(Style::default().fg(theme.gutter_fg)),
                editor_chunks[0].inner(&Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
            );

            // --- Syntax Highlighting ---
            let mut display_text = Vec::new();
            for line in &lines {
                let style = if line.starts_with('#') {
                    Style::default()
                        .fg(theme.syntax_header)
                        .add_modifier(Modifier::BOLD)
                } else if line.starts_with('\\') {
                    Style::default().fg(theme.syntax_latex)
                } else {
                    Style::default().fg(theme.text)
                };
                display_text.push(Line::from(Span::styled(line.as_str(), style)));
            }

            f.render_widget(
                Paragraph::new(display_text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(theme.editor_border))
                            .title(" Rust Editor "),
                    )
                    .style(Style::default().bg(theme.background)),
                editor_chunks[1],
            );

            // --- Status/Search/Command Bar ---
            let bar_style = Style::default().bg(Color::Indexed(235));
            let bar_content = match mode {
                Mode::Normal => Line::from(vec![
                    Span::styled(
                        " NORMAL ",
                        Style::default()
                            .bg(theme.status_normal)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" | Pos: {},{}", cursor_y + 1, cursor_x)),
                ]),
                Mode::Insert => Line::from(vec![Span::styled(
                    " INSERT ",
                    Style::default()
                        .bg(theme.status_insert)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )]),
                Mode::Command => Line::from(format!(":{}", command_input)),
                Mode::Search => Line::from(format!("/{}", search_query)),
            };

            f.render_widget(Paragraph::new(bar_content).style(bar_style), chunks[1]);

            if let Mode::Insert = mode {
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
                    KeyCode::Char('/') => {
                        mode = Mode::Search;
                        search_query.clear();
                    }
                    KeyCode::Char('q') => break,
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
                            let cur = lines.remove(cursor_y);
                            cursor_y -= 1;
                            cursor_x = lines[cursor_y].len();
                            lines[cursor_y].push_str(&cur);
                        }
                    }
                    KeyCode::Enter => {
                        let rem = lines[cursor_y].split_off(cursor_x);
                        lines.insert(cursor_y + 1, rem);
                        cursor_y += 1;
                        cursor_x = 0;
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Enter => {
                        // Simple search: find the first line containing the query
                        for (i, line) in lines.iter().enumerate() {
                            if line.contains(&search_query) {
                                cursor_y = i;
                                cursor_x = line.find(&search_query).unwrap_or(0);
                                break;
                            }
                        }
                        mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => search_query.push(c),
                    KeyCode::Backspace => {
                        search_query.pop();
                    }
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Enter => {
                        if command_input == "w" {
                            fs::write("work.md", lines.join("\n"))?;
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
