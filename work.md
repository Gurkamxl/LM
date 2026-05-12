# Welcome to LM Editor

A **LazyVim-inspired** Markdown and LaTeX editor for the terminal.

## Features

- **Split pane** — edit on the left, preview on the right
- **Vim modal editing** — Normal, Insert, and Command modes
- *Syntax highlighting* for Markdown and LaTeX
- Live preview powered by `tui-markdown`
- PDF export via `:export`

## Quick Start

Press `i` to enter **Insert mode** and start typing.

Press `Esc` to return to **Normal mode**.

Type `:w` to save, `:q` to quit, `:wq` to save and quit.

## Navigation

| Key       | Action                |
|-----------|-----------------------|
| `h/j/k/l` | Move cursor           |
| `w / b`   | Next / prev word      |
| `gg / G`  | Top / bottom of file  |
| `Ctrl+D`  | Scroll preview down   |
| `Ctrl+U`  | Scroll preview up     |

## LaTeX Example

```latex
\documentclass{article}
\begin{document}
  Hello, \textbf{world}!
\end{document}
```

Press `?` to see all keybindings.
