//! Sitzungs-Logbuch im Markdown-Format unter `~/.rubberduck`.

use crate::paths;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Das Frage-Antwort-Protokoll einer Session.
#[derive(Debug, Clone)]
pub struct Transcript {
    pub topic: String,
    /// `YYYY-MM-DD` – für Dateiname und Überschrift.
    pub date: String,
    /// Menschenlesbarer Startzeitpunkt.
    pub started_at: String,
    /// Paare aus (Frage, Antwort) in Reihenfolge.
    pub entries: Vec<(String, String)>,
}

/// Rendert das Transkript als Markdown.
pub fn render_markdown(t: &Transcript) -> String {
    let mut s = String::new();
    s.push_str(&format!("# 🦆 Rubberduck-Session – {}\n\n", t.date));
    s.push_str(&format!("- **Thema:** {}\n", t.topic));
    s.push_str(&format!("- **Gestartet:** {}\n\n", t.started_at));

    for (i, (question, answer)) in t.entries.iter().enumerate() {
        s.push_str(&format!("### {}. {}\n\n", i + 1, question));
        let answer = if answer.trim().is_empty() {
            "_(keine Antwort)_"
        } else {
            answer.as_str()
        };
        s.push_str(answer);
        s.push_str("\n\n");
    }

    s.push_str("---\n\n");
    s
}

/// Schreibt das Logbuch (gleicher Tag wird angehängt) und gibt den Pfad zurück.
pub fn write_log(t: &Transcript) -> Result<PathBuf> {
    write_log_in(&paths::data_dir()?, t)
}

/// Wie [`write_log`], aber mit explizitem Zielverzeichnis (für Tests).
fn write_log_in(dir: &Path, t: &Transcript) -> Result<PathBuf> {
    fs::create_dir_all(dir)
        .with_context(|| format!("Konnte Verzeichnis {} nicht anlegen", dir.display()))?;

    let path = dir.join(format!("session-{}.md", t.date));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Konnte {} nicht öffnen", path.display()))?;
    file.write_all(render_markdown(t).as_bytes())
        .with_context(|| format!("Konnte {} nicht schreiben", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Transcript {
        Transcript {
            topic: "logic".into(),
            date: "2026-06-05".into(),
            started_at: "2026-06-05 14:32".into(),
            entries: vec![
                ("Was soll es tun?".into(), "Die Summe bilden".into()),
                ("Was passiert?".into(), "   ".into()),
            ],
        }
    }

    #[test]
    fn markdown_has_header_and_entries() {
        let md = render_markdown(&sample());
        assert!(md.contains("# 🦆 Rubberduck-Session – 2026-06-05"));
        assert!(md.contains("**Thema:** logic"));
        assert!(md.contains("### 1. Was soll es tun?"));
        assert!(md.contains("Die Summe bilden"));
        // Leere Antwort wird als Platzhalter dargestellt.
        assert!(md.contains("_(keine Antwort)_"));
    }

    #[test]
    fn write_log_creates_and_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log_in(dir.path(), &sample()).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "session-2026-06-05.md");

        // Zweiter Aufruf hängt an, statt zu überschreiben.
        write_log_in(dir.path(), &sample()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let occurrences = content.matches("Rubberduck-Session").count();
        assert_eq!(occurrences, 2);
    }
}
