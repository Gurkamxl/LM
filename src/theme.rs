use ratatui::style::Color;

/// Tokyo Night color palette — every color used in the UI lives here.
pub struct Theme {
    // Background layers
    pub bg:        Color,   // main background
    pub bg_dark:   Color,   // slightly darker (gutter, status)
    pub bg_popup:  Color,   // help overlay background

    // Foreground / text
    pub fg:        Color,   // normal text
    pub fg_dim:    Color,   // line numbers, dim text
    pub fg_dark:   Color,   // very dim (inactive borders)

    // Borders
    pub border_active:   Color, // focused pane border
    pub border_inactive: Color, // unfocused pane border

    // Status bar mode pills
    pub mode_normal:  Color, // NORMAL  — blue
    pub mode_insert:  Color, // INSERT  — green
    pub mode_visual:  Color, // VISUAL  — magenta
    pub mode_command: Color, // COMMAND — orange

    // Syntax colours (editor)
    pub syn_header:  Color, // # headings
    pub syn_latex:   Color, // \commands
    pub syn_bold:    Color, // **bold** markers
    pub syn_italic:  Color, // _italic_ markers
    pub syn_code:    Color, // `code` / code blocks
    pub syn_comment: Color, // <!-- comments --> / % latex comments
    pub syn_link:    Color, // [link]() text
    pub syn_keyword: Color, // misc keywords

    // Preview colours
    pub preview_h1:     Color,
    pub preview_h2:     Color,
    pub preview_h3:     Color,
    pub preview_bullet: Color,
    pub preview_code:   Color,
    pub preview_quote:  Color,

    // Decorative
    pub accent:  Color, // unsaved dot, export spinner
    pub success: Color, // compile success
    pub error:   Color, // compile error
    pub warning: Color, // warnings
}

/// The single global theme instance used throughout the app.
pub const THEME: Theme = Theme {
    // ── Backgrounds ──────────────────────────────────────────────
    bg:         Color::Indexed(234), // #1a1b26  (Tokyo Night bg)
    bg_dark:    Color::Indexed(232), // #16161e  (darker panel)
    bg_popup:   Color::Indexed(237), // #283457  (popup bg)

    // ── Text ──────────────────────────────────────────────────────
    fg:         Color::Indexed(253), // #c0caf5  (main text)
    fg_dim:     Color::Indexed(243), // #565f89  (comments / line nums)
    fg_dark:    Color::Indexed(238), // #3b4261  (very dim)

    // ── Borders ───────────────────────────────────────────────────
    border_active:   Color::Indexed(111), // #7aa2f7  (blue)
    border_inactive: Color::Indexed(238), // #3b4261  (dim)

    // ── Mode pills ────────────────────────────────────────────────
    mode_normal:  Color::Indexed(110), // #7dcfff  (cyan-blue)
    mode_insert:  Color::Indexed(150), // #9ece6a  (green)
    mode_visual:  Color::Indexed(175), // #bb9af7  (purple)
    mode_command: Color::Indexed(215), // #ff9e64  (orange)

    // ── Editor syntax ─────────────────────────────────────────────
    syn_header:  Color::Indexed(216), // #ff9e64  orange headings
    syn_latex:   Color::Indexed(117), // #7dcfff  cyan LaTeX cmds
    syn_bold:    Color::Indexed(222), // #e0af68  gold bold markers
    syn_italic:  Color::Indexed(183), // #bb9af7  purple italics
    syn_code:    Color::Indexed(141), // #9ece6a-ish  inline code
    syn_comment: Color::Indexed(59),  // #414868  muted grey
    syn_link:    Color::Indexed(117), // #73daca  teal links
    syn_keyword: Color::Indexed(204), // #f7768e  red keywords

    // ── Preview ───────────────────────────────────────────────────
    preview_h1:     Color::Indexed(216),
    preview_h2:     Color::Indexed(222),
    preview_h3:     Color::Indexed(215),
    preview_bullet: Color::Indexed(110),
    preview_code:   Color::Indexed(150),
    preview_quote:  Color::Indexed(243),

    // ── Status / alerts ───────────────────────────────────────────
    accent:  Color::Indexed(204), // #f7768e  red — unsaved dot
    success: Color::Indexed(150), // #9ece6a  green
    error:   Color::Indexed(204), // #f7768e  red
    warning: Color::Indexed(215), // #ff9e64  orange
};
