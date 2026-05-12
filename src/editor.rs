use anyhow::{Context, Result};
use ratatui::style::{Color, Modifier, Style};
use ratatui_textarea::TextArea;
use std::fs;

use crate::app::{App, FileType};
use crate::theme::THEME;

/// Load a file from disk into the app's TextArea.
/// Creates an empty buffer if the file doesn't exist.
pub fn load_file(app: &mut App) -> Result<()> {
    let path = app.path();

    let content = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {:?}", path))?
    } else {
        String::new()
    };

    // Split into lines; ratatui-textarea expects Vec<String>
    let lines: Vec<String> = if content.is_empty() {
        vec![String::new()]
    } else {
        content.lines().map(|l| l.to_string()).collect()
    };

    app.textarea = build_textarea(lines, &app.file_type);
    app.modified = false;
    Ok(())
}

/// Save the TextArea contents back to disk.
pub fn save_file(app: &mut App) -> Result<()> {
    let text = app.text();
    fs::write(app.path(), &text)
        .with_context(|| format!("Failed to write {:?}", app.path()))?;
    app.modified = false;
    Ok(())
}

/// Build a styled TextArea from lines, applying Tokyo Night theme.
pub fn build_textarea(lines: Vec<String>, file_type: &FileType) -> TextArea<'static> {
    let mut ta = TextArea::from(lines);

    // Cursor style — block in Normal, bar in Insert
    ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    // Line numbers
    ta.set_line_number_style(Style::default().fg(THEME.fg_dim));

    // Tab width
    ta.set_tab_length(4);

    // Hard-wrap at 0 means no wrap (scroll horizontally)
    // We'll let the Paragraph handle wrapping via the block width

    // Block styling is applied by ui.rs when rendering

    ta
}

/// Apply vim Normal-mode styling to the textarea cursor.
pub fn set_normal_cursor(ta: &mut TextArea) {
    ta.set_cursor_style(
        Style::default()
            .bg(Color::Indexed(110))
            .fg(Color::Indexed(234))
    );
}

/// Apply vim Insert-mode styling to the textarea cursor.
pub fn set_insert_cursor(ta: &mut TextArea) {
    ta.set_cursor_style(
        Style::default()
            .bg(Color::Indexed(150))
            .fg(Color::Indexed(234))
    );
}

/// Set cursor back to default (dim reversed block) for command mode.
pub fn set_command_cursor(ta: &mut TextArea) {
    ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
}

/// Detect file type from a filename string.
pub fn detect_file_type(filename: &str) -> FileType {
    FileType::from_path(filename)
}
