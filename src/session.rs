//! Sitzungs-Protokoll: Frage/Antwort mit Zeiten, Statistik, Aha-Moment und
//! Markdown-Export nach `~/.rubberduck/session-<datum>.md`.

use crate::paths;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Eine beantwortete (oder übersprungene) Frage samt Bearbeitungszeit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Die gestellte Frage.
    pub question: String,
    /// Die Antwort der Nutzerin (kann leer sein).
    pub answer: String,
    /// Wie lange bis zur Antwort gebraucht wurde (Sekunden).
    pub seconds: u64,
}

/// Der festgehaltene „Aha!“-Moment: der Bug wurde gefunden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aha {
    /// Kurze Notiz, was es war (optional).
    pub note: String,
    /// Nach wie vielen beantworteten Fragen der Groschen fiel.
    pub after: usize,
    /// Zeit von Sessionbeginn bis zur Lösung (Sekunden).
    pub seconds_to_solution: u64,
}

/// Das vollständige Protokoll einer Session.
#[derive(Debug, Clone)]
pub struct Transcript {
    /// Gewähltes Thema.
    pub topic: String,
    /// Datum `YYYY-MM-DD` (Dateiname + Überschrift).
    pub date: String,
    /// Menschenlesbarer Startzeitpunkt.
    pub started_at: String,
    /// Die Frage/Antwort-Einträge in Reihenfolge.
    pub entries: Vec<Entry>,
    /// Der Aha-Moment, falls der Bug gefunden wurde.
    pub aha: Option<Aha>,
    /// Gesamtdauer der Session (Sekunden).
    pub total_seconds: u64,
}

impl Transcript {
    /// Leeres Protokoll für `topic`, gestartet zu `date` / `started_at`.
    #[must_use]
    pub fn new(
        topic: impl Into<String>,
        date: impl Into<String>,
        started_at: impl Into<String>,
    ) -> Self {
        Self {
            topic: topic.into(),
            date: date.into(),
            started_at: started_at.into(),
            entries: Vec::new(),
            aha: None,
            total_seconds: 0,
        }
    }

    /// Hängt einen Frage/Antwort-Eintrag an.
    pub fn push(&mut self, question: impl Into<String>, answer: impl Into<String>, seconds: u64) {
        self.entries.push(Entry {
            question: question.into(),
            answer: answer.into(),
            seconds,
        });
    }

    /// Verdichtete Kennzahlen der Session.
    #[must_use]
    pub fn stats(&self) -> Stats {
        let asked = self.entries.len();
        let answered = self
            .entries
            .iter()
            .filter(|e| !e.answer.trim().is_empty())
            .count();
        let avg_seconds = if asked > 0 {
            self.total_seconds / asked as u64
        } else {
            0
        };
        Stats {
            asked,
            answered,
            total_seconds: self.total_seconds,
            avg_seconds,
            solved: self.aha.is_some(),
        }
    }
}

/// Verdichtete Kennzahlen einer Session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Anzahl gestellter Fragen.
    pub asked: usize,
    /// Anzahl tatsächlich beantworteter Fragen.
    pub answered: usize,
    /// Gesamtdauer (Sekunden).
    pub total_seconds: u64,
    /// Durchschnittliche Zeit pro Frage (Sekunden).
    pub avg_seconds: u64,
    /// Ob der Bug gefunden wurde.
    pub solved: bool,
}

/// Formatiert Sekunden menschenlesbar, z. B. `2m 05s` oder `45s`.
#[must_use]
pub fn format_duration(seconds: u64) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    if mins > 0 {
        format!("{mins}m {secs:02}s")
    } else {
        format!("{secs}s")
    }
}

/// Rendert das Protokoll als Markdown.
#[must_use]
pub fn render_markdown(t: &Transcript) -> String {
    let stats = t.stats();
    let mut s = String::new();
    s.push_str(&format!("# 🦆 Rubberduck-Session – {}\n\n", t.date));
    s.push_str(&format!("- **Thema:** {}\n", t.topic));
    s.push_str(&format!("- **Gestartet:** {}\n", t.started_at));
    s.push_str(&format!(
        "- **Fragen:** {} beantwortet / {} gestellt\n",
        stats.answered, stats.asked
    ));
    s.push_str(&format!(
        "- **Dauer:** {} (Ø {} pro Frage)\n",
        format_duration(stats.total_seconds),
        format_duration(stats.avg_seconds)
    ));
    s.push_str(&format!(
        "- **Gelöst:** {}\n\n",
        if stats.solved { "✅ ja" } else { "– offen" }
    ));

    if let Some(aha) = &t.aha {
        s.push_str(&format!(
            "> 💡 **Aha nach Frage {} ({}):** {}\n\n",
            aha.after,
            format_duration(aha.seconds_to_solution),
            if aha.note.trim().is_empty() {
                "(keine Notiz)"
            } else {
                aha.note.trim()
            }
        ));
    }

    for (i, entry) in t.entries.iter().enumerate() {
        s.push_str(&format!("### {}. {}\n\n", i + 1, entry.question));
        let answer = if entry.answer.trim().is_empty() {
            "_(keine Antwort)_"
        } else {
            entry.answer.trim()
        };
        s.push_str(answer);
        s.push_str("\n\n");
    }

    s.push_str("---\n\n");
    s
}

/// Schreibt das Protokoll (gleicher Tag wird angehängt) und gibt den Pfad zurück.
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
        let mut t = Transcript::new("logic", "2026-06-05", "2026-06-05 14:32");
        t.push("Was soll es tun?", "Summe bilden", 30);
        t.push("Was passiert?", "   ", 10);
        t.total_seconds = 40;
        t
    }

    #[test]
    fn stats_count_answers() {
        let s = sample().stats();
        assert_eq!(s.asked, 2);
        assert_eq!(s.answered, 1);
        assert_eq!(s.avg_seconds, 20);
        assert!(!s.solved);
    }

    #[test]
    fn duration_formats() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(125), "2m 05s");
    }

    #[test]
    fn markdown_contains_sections() {
        let mut t = sample();
        t.aha = Some(Aha {
            note: "Index vertauscht".into(),
            after: 2,
            seconds_to_solution: 40,
        });
        let md = render_markdown(&t);
        assert!(md.contains("# 🦆 Rubberduck-Session – 2026-06-05"));
        assert!(md.contains("**Thema:** logic"));
        assert!(md.contains("1 beantwortet / 2 gestellt"));
        assert!(md.contains("✅ ja"));
        assert!(md.contains("💡 **Aha nach Frage 2"));
        assert!(md.contains("Index vertauscht"));
        assert!(md.contains("_(keine Antwort)_"));
    }

    #[test]
    fn write_log_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log_in(dir.path(), &sample()).unwrap();
        assert_eq!(path.file_name().unwrap(), "session-2026-06-05.md");
        write_log_in(dir.path(), &sample()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.matches("Rubberduck-Session").count(), 2);
    }
}
