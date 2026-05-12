use anyhow::Result;
use std::process::Command;

use crate::app::{App, ExportStatus, FileType};
use crate::editor::save_file;

/// Process a command-line input (everything the user typed after `:`).
/// Returns Ok(true) if the app should quit.
pub fn handle_command(app: &mut App, cmd: &str) -> Result<bool> {
    let cmd = cmd.trim();

    match cmd {
        // ── Save ─────────────────────────────────────────────────────────────
        "w" => {
            save_file(app)?;
            app.command_msg = Some(format!("\"{}\" written", app.filename));
        }

        // ── Quit ─────────────────────────────────────────────────────────────
        "q" => {
            if app.modified {
                app.command_msg = Some(
                    "No write since last change. Use :q! to force quit or :wq to save.".into(),
                );
            } else {
                return Ok(true); // signal quit
            }
        }

        "q!" => {
            return Ok(true); // force quit without saving
        }

        // ── Save & quit ───────────────────────────────────────────────────────
        "wq" | "x" => {
            save_file(app)?;
            return Ok(true);
        }

        // ── Export to PDF ────────────────────────────────────────────────────
        "export" | "pdf" | "e" => {
            export_pdf(app);
        }

        // ── Help overlay ─────────────────────────────────────────────────────
        "help" | "h" => {
            app.show_help = true;
        }

        // ── Change file type ─────────────────────────────────────────────────
        "set md" | "set markdown" => {
            app.file_type = FileType::Markdown;
            app.command_msg = Some("File type set to Markdown".into());
        }
        "set tex" | "set latex" => {
            app.file_type = FileType::Latex;
            app.command_msg = Some("File type set to LaTeX".into());
        }
        "set txt" | "set plain" => {
            app.file_type = FileType::Plain;
            app.command_msg = Some("File type set to Plain Text".into());
        }

        // ── Open a file ──────────────────────────────────────────────────────
        s if s.starts_with("open ") || s.starts_with("e ") || s.starts_with("edit ") => {
            let parts: Vec<&str> = s.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let new_name = parts[1].trim().to_string();
                app.filename  = new_name.clone();
                app.file_type = FileType::from_path(&new_name);
                crate::editor::load_file(app)?;
                app.command_msg = Some(format!("Opened \"{}\"", new_name));
            }
        }

        // ── Write to a different file ─────────────────────────────────────────
        s if s.starts_with("w ") => {
            let new_name = s[2..].trim().to_string();
            let old = app.filename.clone();
            app.filename = new_name.clone();
            save_file(app)?;
            app.filename = old; // keep current name unless user wants to switch
            app.command_msg = Some(format!("Written to \"{}\"", new_name));
        }

        // ── Unknown command ───────────────────────────────────────────────────
        other => {
            app.command_msg = Some(format!("Unknown command: :{}", other));
        }
    }

    Ok(false)
}

/// Spawn a background PDF compile process.
/// Sets `app.export_status` to Running, then to Success/Failure.
/// NOTE: This is a blocking call for simplicity — in the future this could
/// be made async with a thread channel.
pub fn export_pdf(app: &mut App) {
    app.export_status = Some(ExportStatus::Running);

    let result = match app.file_type {
        FileType::Latex => compile_latex(&app.filename),
        FileType::Markdown | FileType::Plain => compile_markdown(&app.filename),
    };

    match result {
        Ok(path) => {
            app.export_status = Some(ExportStatus::Success(path));
            app.command_msg   = Some("PDF exported successfully!".into());
        }
        Err(e) => {
            let msg = e.to_string();
            app.export_status = Some(ExportStatus::Failure(msg.clone()));
            app.command_msg   = Some(format!("Export failed: {}", msg));
        }
    }
}

/// Compile a .tex file to PDF using `tectonic`.
fn compile_latex(filename: &str) -> Result<String> {
    // Try tectonic first
    let tectonic = Command::new("tectonic")
        .arg(filename)
        .output();

    match tectonic {
        Ok(out) if out.status.success() => {
            let pdf = filename.replace(".tex", ".pdf");
            Ok(pdf)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            anyhow::bail!("tectonic: {}", stderr.lines().last().unwrap_or("unknown error"))
        }
        Err(_) => {
            // tectonic not found — try pdflatex
            let pdftex = Command::new("pdflatex")
                .args(["-interaction=nonstopmode", filename])
                .output();

            match pdftex {
                Ok(out) if out.status.success() => {
                    let pdf = filename.replace(".tex", ".pdf");
                    Ok(pdf)
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    anyhow::bail!("{}", stderr.lines().last().unwrap_or("pdflatex failed"))
                }
                Err(_) => {
                    anyhow::bail!("No LaTeX engine found. Install: brew install tectonic")
                }
            }
        }
    }
}

/// Compile a .md file to PDF using `pandoc`.
fn compile_markdown(filename: &str) -> Result<String> {
    let pdf_name = filename
        .trim_end_matches(".md")
        .trim_end_matches(".markdown")
        .to_string() + ".pdf";

    let pandoc = Command::new("pandoc")
        .args([filename, "-o", &pdf_name, "--pdf-engine=tectonic"])
        .output();

    match pandoc {
        Ok(out) if out.status.success() => Ok(pdf_name),
        Ok(out) => {
            // Try without specifying pdf-engine
            let pandoc2 = Command::new("pandoc")
                .args([filename, "-o", &pdf_name])
                .output();
            match pandoc2 {
                Ok(o) if o.status.success() => Ok(pdf_name),
                _ => {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    anyhow::bail!("{}", stderr.lines().last().unwrap_or("pandoc failed"))
                }
            }
        }
        Err(_) => {
            anyhow::bail!("pandoc not found. Install: brew install pandoc")
        }
    }
}
