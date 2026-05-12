use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::app::{App, Focus, Mode};
use crate::preview::render_preview;
use crate::theme::THEME;

/// Main render function — called once per frame.
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Fill background
    f.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        area,
    );

    // Top-level split: [main content] / [status bar (1 line)]
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area   = root[0];
    let status_area = root[1];

    // Horizontal split: [editor 50%] / [preview 50%]
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_area);

    let editor_area  = panes[0];
    let preview_area = panes[1];

    // ── Editor pane ────────────────────────────────────────────────────────
    render_editor(f, app, editor_area);

    // ── Preview pane ───────────────────────────────────────────────────────
    render_preview_pane(f, app, preview_area);

    // ── Status bar ────────────────────────────────────────────────────────
    render_status(f, app, status_area);

    // ── Help overlay (on top) ─────────────────────────────────────────────
    if app.show_help {
        render_help(f, area);
    }

    // ── Command bar (replaces bottom line in Command mode) ────────────────
    if app.mode == Mode::Command {
        render_command_bar(f, app, status_area);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Editor Pane
// ─────────────────────────────────────────────────────────────────────────────

fn render_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::Editor;

    let (border_color, title_color) = if is_focused {
        (THEME.border_active, THEME.fg)
    } else {
        (THEME.border_inactive, THEME.fg_dim)
    };

    // Modified indicator
    let modified_dot = if app.modified {
        Span::styled(" ● ", Style::default().fg(THEME.accent))
    } else {
        Span::raw("   ")
    };

    // File type badge
    let ft_badge = Span::styled(
        format!(" {} ", app.file_type.label()),
        Style::default().fg(THEME.fg_dim),
    );

    let title_spans = vec![
        Span::raw(" "),
        Span::styled(
            app.filename.clone(),
            Style::default().fg(title_color).add_modifier(Modifier::BOLD),
        ),
        modified_dot,
        ft_badge,
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(title_spans))
        .style(Style::default().bg(THEME.bg));

    // Apply block to textarea widget
    app.textarea.set_block(block);
    app.textarea.set_style(Style::default().fg(THEME.fg).bg(THEME.bg));
    app.textarea.set_line_number_style(
        Style::default().fg(THEME.fg_dim).bg(THEME.bg_dark),
    );

    f.render_widget(&app.textarea, area);
}

// ─────────────────────────────────────────────────────────────────────────────
// Preview Pane
// ─────────────────────────────────────────────────────────────────────────────

fn render_preview_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::Preview;

    let border_color = if is_focused {
        THEME.border_active
    } else {
        THEME.border_inactive
    };

    let source = app.text();
    let text   = render_preview(&source, &app.file_type, app.export_status.as_ref());

    let total_lines = text.lines.len() as u16;
    // Clamp scroll
    let inner_h = area.height.saturating_sub(2);
    if app.preview_scroll + inner_h > total_lines {
        app.preview_scroll = total_lines.saturating_sub(inner_h);
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("Preview", Style::default().fg(THEME.fg_dim).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]))
        .style(Style::default().bg(THEME.bg));

    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    f.render_widget(para, area);

    // Scrollbar
    if total_lines > inner_h {
        let mut sb_state = ScrollbarState::new(total_lines as usize)
            .position(app.preview_scroll as usize);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut sb_state,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Status Bar
// ─────────────────────────────────────────────────────────────────────────────

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let (mode_str, mode_color) = match app.mode {
        Mode::Normal  => (app.mode.label(), THEME.mode_normal),
        Mode::Insert  => (app.mode.label(), THEME.mode_insert),
        Mode::Visual  => (app.mode.label(), THEME.mode_visual),
        Mode::Command => (app.mode.label(), THEME.mode_command),
    };

    let (ln, col) = app.cursor_pos();

    // Build status message — prefer command_msg if present
    let msg = app.command_msg.clone().unwrap_or_default();

    // Right side hint
    let hint = "  ?:help  Ctrl+P:PDF  Tab:focus";

    let spans = vec![
        // Mode pill
        Span::styled(
            mode_str,
            Style::default()
                .fg(THEME.bg_dark)
                .bg(mode_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().bg(THEME.bg_dark)),
        // Filename
        Span::styled(
            app.filename.clone(),
            Style::default().fg(THEME.fg).bg(THEME.bg_dark),
        ),
        Span::styled("  ", Style::default().bg(THEME.bg_dark)),
        // Message / status
        Span::styled(msg, Style::default().fg(THEME.fg_dim).bg(THEME.bg_dark)),
        // Spacer (filled with bg_dark)
        Span::styled(
            " ".repeat(area.width.saturating_sub(
                mode_str.len() as u16 + 2
                    + app.filename.len() as u16 + 2
                    + hint.len() as u16
                    + 20 // Ln/Col
            ) as usize),
            Style::default().bg(THEME.bg_dark),
        ),
        // Right: cursor position
        Span::styled(
            format!("  Ln {}, Col {}  ", ln, col),
            Style::default().fg(THEME.fg_dim).bg(THEME.bg_dark),
        ),
        // Hint
        Span::styled(hint, Style::default().fg(THEME.fg_dark).bg(THEME.bg_dark)),
    ];

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(THEME.bg_dark)),
        area,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Command Bar
// ─────────────────────────────────────────────────────────────────────────────

fn render_command_bar(f: &mut Frame, app: &App, area: Rect) {
    let content = format!(":{}", app.command_input);
    let para = Paragraph::new(content)
        .style(Style::default().fg(THEME.fg).bg(THEME.bg_dark));
    f.render_widget(para, area);

    // Place cursor at end of command input
    let cx = area.x + 1 + app.command_input.len() as u16;
    f.set_cursor_position((cx, area.y));
}

// ─────────────────────────────────────────────────────────────────────────────
// Help Overlay
// ─────────────────────────────────────────────────────────────────────────────

fn render_help(f: &mut Frame, area: Rect) {
    // Centre the overlay
    let popup_w = 60u16.min(area.width.saturating_sub(4));
    let popup_h = 34u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_w)) / 2;
    let y = (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(x, y, popup_w, popup_h);

    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled("  LM — Keybindings", Style::default().fg(THEME.syn_header).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  ── Navigation ──────────────────────────────────", Style::default().fg(THEME.fg_dim))),
        help_line("h/j/k/l", "Move cursor left/down/up/right"),
        help_line("w / b",   "Next / prev word"),
        help_line("0 / $",   "Start / end of line"),
        help_line("gg / G",  "Top / bottom of file"),
        help_line("Ctrl+D",  "Scroll preview down"),
        help_line("Ctrl+U",  "Scroll preview up"),
        Line::from(""),
        Line::from(Span::styled("  ── Modes ───────────────────────────────────────", Style::default().fg(THEME.fg_dim))),
        help_line("i",       "Enter Insert mode"),
        help_line("Esc",     "Return to Normal mode"),
        help_line(":",       "Enter Command mode"),
        help_line("Tab",     "Toggle focus (editor ↔ preview)"),
        Line::from(""),
        Line::from(Span::styled("  ── Editing ─────────────────────────────────────", Style::default().fg(THEME.fg_dim))),
        help_line("u",       "Undo"),
        help_line("Ctrl+R",  "Redo"),
        help_line("dd",      "Delete line"),
        help_line("yy",      "Yank (copy) line"),
        help_line("p",       "Paste"),
        Line::from(""),
        Line::from(Span::styled("  ── Commands ────────────────────────────────────", Style::default().fg(THEME.fg_dim))),
        help_line(":w",      "Save file"),
        help_line(":q",      "Quit (fails if unsaved)"),
        help_line(":q!",     "Force quit"),
        help_line(":wq / :x","Save and quit"),
        help_line(":export", "Export to PDF"),
        help_line(":open f", "Open a different file"),
        help_line(":help",   "Show this help"),
        Line::from(""),
        Line::from(Span::styled("  ── Shortcuts ───────────────────────────────────", Style::default().fg(THEME.fg_dim))),
        help_line("Ctrl+P",  "Quick export to PDF"),
        help_line("?",       "Toggle this help"),
        Line::from(""),
        Line::from(Span::styled("  Press ? or Esc to close", Style::default().fg(THEME.fg_dim).add_modifier(Modifier::ITALIC))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.border_active))
        .style(Style::default().bg(THEME.bg_popup));

    f.render_widget(
        Paragraph::new(help_text).block(block),
        popup_area,
    );
}

fn help_line(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<12}", key),
            Style::default().fg(THEME.syn_bold).add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(THEME.fg)),
    ])
}
