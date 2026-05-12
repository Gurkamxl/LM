use std::path::PathBuf;
use ratatui_textarea::TextArea;

/// Which kind of document is loaded.
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Markdown,
    Latex,
    Plain,
}

impl FileType {
    pub fn from_path(path: &str) -> Self {
        let p = path.to_lowercase();
        if p.ends_with(".md") || p.ends_with(".markdown") {
            FileType::Markdown
        } else if p.ends_with(".tex") {
            FileType::Latex
        } else {
            FileType::Plain
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FileType::Markdown => "MD",
            FileType::Latex    => "TEX",
            FileType::Plain    => "TXT",
        }
    }
}

/// Vim-style editing modes.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Normal  => " NORMAL ",
            Mode::Insert  => " INSERT ",
            Mode::Visual  => " VISUAL ",
            Mode::Command => " COMMAND ",
        }
    }
}

/// Status of the last PDF export attempt.
#[derive(Debug, Clone)]
pub enum ExportStatus {
    Success(String), // path to generated PDF
    Failure(String), // error message
    Running,
}

/// Which pane has keyboard focus.
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Editor,
    Preview,
}

/// All application state lives here.
pub struct App<'a> {
    // Editor
    pub textarea:  TextArea<'a>,
    pub mode:      Mode,
    pub filename:  String,
    pub file_type: FileType,
    pub modified:  bool,

    // Command bar
    pub command_input: String,
    pub command_msg:   Option<String>, // last status / error message

    // Preview
    pub preview_scroll: u16,
    pub focus:          Focus,

    // Export
    pub export_status: Option<ExportStatus>,

    // UI flags
    pub show_help: bool,
    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(filename: String) -> Self {
        let file_type = FileType::from_path(&filename);
        App {
            textarea:      TextArea::default(),
            mode:          Mode::Normal,
            filename,
            file_type,
            modified:      false,
            command_input: String::new(),
            command_msg:   None,
            preview_scroll: 0,
            focus:          Focus::Editor,
            export_status:  None,
            show_help:      false,
            should_quit:    false,
        }
    }

    /// Return all text in the editor as a single String.
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Return (line, col) 1-indexed for display.
    pub fn cursor_pos(&self) -> (usize, usize) {
        let dc = self.textarea.cursor();
        (dc.0 + 1, dc.1 + 1)
    }

    /// Path to the file as a PathBuf.
    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.filename)
    }
}
