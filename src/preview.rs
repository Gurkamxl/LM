use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use crate::app::FileType;
use crate::theme::THEME;

/// Render Markdown content into a ratatui `Text` for display in the preview pane.
/// Uses `tui-markdown` for proper styled rendering.
pub fn render_markdown(source: &str) -> Text<'_> {
    tui_markdown::from_str(source)
}

/// For LaTeX files, we can't render a PDF in the terminal.
/// Instead, display syntax-highlighted LaTeX source in the preview.
pub fn render_latex(source: &str) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled("  LaTeX Preview", Style::default().fg(THEME.syn_latex).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(Span::styled(
        "  Use :export to compile PDF",
        Style::default().fg(THEME.fg_dim),
    )));
    lines.push(Line::from(""));

    for raw_line in source.lines() {
        let line = raw_line.to_string();
        let styled = colorize_latex_line(line);
        lines.push(styled);
    }

    Text::from(lines)
}

/// Apply simple single-pass syntax coloring for a LaTeX source line.
fn colorize_latex_line(line: String) -> Line<'static> {
    let trimmed = line.trim_start();

    // Comment lines
    if trimmed.starts_with('%') {
        return Line::from(Span::styled(line, Style::default().fg(THEME.syn_comment)));
    }

    // \begin{...} / \end{...} — environment markers
    if trimmed.starts_with("\\begin") || trimmed.starts_with("\\end") {
        return Line::from(Span::styled(line, Style::default().fg(THEME.syn_bold).add_modifier(Modifier::BOLD)));
    }

    // \section, \subsection, \title, \author, \chapter
    if trimmed.starts_with("\\section")
        || trimmed.starts_with("\\subsection")
        || trimmed.starts_with("\\subsubsection")
        || trimmed.starts_with("\\title")
        || trimmed.starts_with("\\author")
        || trimmed.starts_with("\\chapter")
    {
        return Line::from(Span::styled(
            line,
            Style::default().fg(THEME.syn_header).add_modifier(Modifier::BOLD),
        ));
    }

    // Any other \command — colour the whole line cyan
    if trimmed.contains('\\') {
        return Line::from(colorize_latex_inline(line));
    }

    // Plain text
    Line::from(Span::styled(line, Style::default().fg(THEME.fg)))
}

/// Walk a line character-by-character and colour \commands vs plain text.
fn colorize_latex_inline(line: String) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut plain = String::new();
    let mut cmd   = String::new();
    let mut in_cmd = false;

    for ch in line.chars() {
        if ch == '\\' {
            if !plain.is_empty() {
                spans.push(Span::styled(plain.clone(), Style::default().fg(THEME.fg)));
                plain.clear();
            }
            in_cmd = true;
            cmd.push(ch);
        } else if in_cmd {
            if ch.is_alphanumeric() || ch == '*' {
                cmd.push(ch);
            } else {
                // End of command
                spans.push(Span::styled(cmd.clone(), Style::default().fg(THEME.syn_latex)));
                cmd.clear();
                in_cmd = false;
                plain.push(ch);
            }
        } else {
            plain.push(ch);
        }
    }

    if in_cmd && !cmd.is_empty() {
        spans.push(Span::styled(cmd, Style::default().fg(THEME.syn_latex)));
    } else if !plain.is_empty() {
        spans.push(Span::styled(plain, Style::default().fg(THEME.fg)));
    }

    spans
}

/// Build the export status banner shown at the top of the preview for LaTeX.
pub fn export_status_lines(status: Option<&crate::app::ExportStatus>) -> Vec<Line<'static>> {
    match status {
        None => vec![],
        Some(crate::app::ExportStatus::Running) => vec![
            Line::from(Span::styled(
                " ⠋ Compiling PDF…",
                Style::default().fg(THEME.warning),
            )),
            Line::from(""),
        ],
        Some(crate::app::ExportStatus::Success(path)) => vec![
            Line::from(Span::styled(
                format!(" ✓ PDF saved: {}", path),
                Style::default().fg(THEME.success),
            )),
            Line::from(""),
        ],
        Some(crate::app::ExportStatus::Failure(msg)) => vec![
            Line::from(Span::styled(
                format!(" ✗ Export failed: {}", msg),
                Style::default().fg(THEME.error),
            )),
            Line::from(""),
        ],
    }
}

/// Dispatch preview rendering based on file type.
pub fn render_preview<'a>(source: &'a str, file_type: &FileType, export_status: Option<&crate::app::ExportStatus>) -> Text<'a> {
    match file_type {
        FileType::Markdown | FileType::Plain => render_markdown(source),
        FileType::Latex => {
            let status_lines = export_status_lines(export_status);
            let rendered = render_latex(source);
            let mut all_lines = status_lines;
            all_lines.extend(rendered.lines);
            Text::from(all_lines)
        }
    }
}
