mod app;
mod commands;
mod editor;
mod preview;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;
// ratatui 0.30 bundles crossterm — always use ratatui's re-export
// to ensure type compatibility with ratatui-textarea 0.9
use ratatui::crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use ratatui_textarea::{CursorMove, Input};
use std::io::stdout;

use app::{App, Focus, Mode};
use editor::{load_file, save_file, set_insert_cursor, set_normal_cursor};

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "lm", about = "LazyVim-inspired Markdown/LaTeX CLI editor")]
struct Cli {
    /// File to open (optional — defaults to scratch.md)
    file: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry Point
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    let filename = cli.file.unwrap_or_else(|| "scratch.md".to_string());

    // Build initial app state
    let mut app = App::new(filename);
    load_file(&mut app)?;

    // Apply initial cursor style (Normal mode)
    set_normal_cursor(&mut app.textarea);

    // Terminal setup
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_loop(&mut terminal, &mut app);

    // Always clean up the terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Event Loop
// ─────────────────────────────────────────────────────────────────────────────

fn run_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.mode {
                Mode::Normal  => handle_normal(app, key)?,
                Mode::Insert  => handle_insert(app, key),
                Mode::Command => handle_command_mode(app, key)?,
                Mode::Visual  => handle_visual(app, key),
            }

            if app.should_quit {
                break;
            }
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Normal Mode
// ─────────────────────────────────────────────────────────────────────────────

fn handle_normal(
    app: &mut App,
    key: ratatui::crossterm::event::KeyEvent,
) -> Result<()> {
    // Clear status message on any keypress
    app.command_msg = None;

    match key.code {
        // ── Enter Insert mode ────────────────────────────────────────────
        KeyCode::Char('i') => {
            app.mode = Mode::Insert;
            set_insert_cursor(&mut app.textarea);
        }
        KeyCode::Char('a') => {
            app.mode = Mode::Insert;
            set_insert_cursor(&mut app.textarea);
            app.textarea.move_cursor(CursorMove::Forward);
        }
        KeyCode::Char('o') => {
            app.mode = Mode::Insert;
            set_insert_cursor(&mut app.textarea);
            app.textarea.move_cursor(CursorMove::End);
            app.textarea.insert_newline();
        }
        KeyCode::Char('O') => {
            app.mode = Mode::Insert;
            set_insert_cursor(&mut app.textarea);
            app.textarea.move_cursor(CursorMove::Head);
            app.textarea.insert_newline();
            app.textarea.move_cursor(CursorMove::Up);
        }

        // ── Enter Command mode ────────────────────────────────────────────
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_input.clear();
        }

        // ── Help overlay ──────────────────────────────────────────────────
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }

        // ── Focus toggle ──────────────────────────────────────────────────
        KeyCode::Tab => {
            app.focus = match app.focus {
                Focus::Editor  => Focus::Preview,
                Focus::Preview => Focus::Editor,
            };
        }

        // ── Vim motions ───────────────────────────────────────────────────
        KeyCode::Char('h') | KeyCode::Left => {
            app.textarea.move_cursor(CursorMove::Back);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.textarea.move_cursor(CursorMove::Forward);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.textarea.move_cursor(CursorMove::Down);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.textarea.move_cursor(CursorMove::Up);
        }
        KeyCode::Char('w') => {
            app.textarea.move_cursor(CursorMove::WordForward);
        }
        KeyCode::Char('b') => {
            app.textarea.move_cursor(CursorMove::WordBack);
        }
        KeyCode::Char('e') => {
            app.textarea.move_cursor(CursorMove::WordEnd);
        }
        KeyCode::Char('0') | KeyCode::Home => {
            app.textarea.move_cursor(CursorMove::Head);
        }
        KeyCode::Char('$') | KeyCode::End => {
            app.textarea.move_cursor(CursorMove::End);
        }
        KeyCode::Char('g') => {
            // gg — go to top
            app.textarea.move_cursor(CursorMove::Top);
        }
        KeyCode::Char('G') => {
            app.textarea.move_cursor(CursorMove::Bottom);
        }

        // ── Editing ───────────────────────────────────────────────────────
        KeyCode::Char('u') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.textarea.undo();
            app.modified = true;
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.textarea.redo();
            app.modified = true;
        }
        // Scroll preview down
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.preview_scroll = app.preview_scroll.saturating_add(5);
        }
        // Scroll preview up
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.preview_scroll = app.preview_scroll.saturating_sub(5);
        }
        // Delete char under cursor (x)
        KeyCode::Char('x') => {
            app.textarea.delete_next_char();
            app.modified = true;
        }

        // ── Quick PDF export ──────────────────────────────────────────────
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = save_file(app);
            commands::export_pdf(app);
        }

        _ => {}
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Insert Mode
// ─────────────────────────────────────────────────────────────────────────────

fn handle_insert(app: &mut App, key: ratatui::crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            set_normal_cursor(&mut app.textarea);
            app.textarea.move_cursor(CursorMove::Back);
        }
        _ => {
            // Wrap the crossterm KeyEvent in ratatui's event enum
            let evt = ratatui::crossterm::event::Event::Key(key);
            let input = Input::from(evt);
            if app.textarea.input(input) {
                app.modified = true;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Command Mode
// ─────────────────────────────────────────────────────────────────────────────

fn handle_command_mode(
    app: &mut App,
    key: ratatui::crossterm::event::KeyEvent,
) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            app.command_input.clear();
            app.mode = Mode::Normal;

            let quit = commands::handle_command(app, &cmd)?;
            if quit {
                app.should_quit = true;
            }
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        _ => {}
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Visual Mode
// ─────────────────────────────────────────────────────────────────────────────

fn handle_visual(app: &mut App, key: ratatui::crossterm::event::KeyEvent) {
    if key.code == KeyCode::Esc {
        app.mode = Mode::Normal;
        set_normal_cursor(&mut app.textarea);
    } else {
        let evt = ratatui::crossterm::event::Event::Key(key);
        let input = Input::from(evt);
        app.textarea.input(input);
    }
}
