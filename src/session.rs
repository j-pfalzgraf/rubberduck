//! Session log: question/answer with timings, statistics, the aha moment and a
//! Markdown export to `~/.rubberduck/session-<date>.md`.

use crate::i18n::Tr;
use crate::paths;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// One answered (or skipped) question with the time spent on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The question that was asked.
    pub question: String,
    /// The user's answer (may be empty).
    pub answer: String,
    /// How long until the answer was given (seconds).
    pub seconds: u64,
}

/// The recorded "aha!" moment: the bug was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aha {
    /// Short note on what it was (optional).
    pub note: String,
    /// After how many answered questions the penny dropped.
    pub after: usize,
    /// Time from session start to the solution (seconds).
    pub seconds_to_solution: u64,
}

/// The complete log of a session.
#[derive(Debug, Clone)]
pub struct Transcript {
    /// Chosen topic.
    pub topic: String,
    /// Date `YYYY-MM-DD` (file name + heading).
    pub date: String,
    /// Human-readable start time.
    pub started_at: String,
    /// The question/answer entries, in order.
    pub entries: Vec<Entry>,
    /// The aha moment, if the bug was found.
    pub aha: Option<Aha>,
    /// Total session duration (seconds).
    pub total_seconds: u64,
}

impl Transcript {
    /// Empty log for `topic`, started at `date` / `started_at`.
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

    /// Appends a question/answer entry.
    pub fn push(&mut self, question: impl Into<String>, answer: impl Into<String>, seconds: u64) {
        self.entries.push(Entry {
            question: question.into(),
            answer: answer.into(),
            seconds,
        });
    }

    /// Condensed metrics for the session.
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

/// Condensed metrics for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Number of questions asked.
    pub asked: usize,
    /// Number of questions actually answered.
    pub answered: usize,
    /// Total duration (seconds).
    pub total_seconds: u64,
    /// Average time per question (seconds).
    pub avg_seconds: u64,
    /// Whether the bug was found.
    pub solved: bool,
}

/// Formats seconds for humans, e.g. `2m 05s` or `45s`.
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

/// Renders the log as Markdown in the language of `tr`.
#[must_use]
pub fn render_markdown(t: &Transcript, tr: Tr) -> String {
    let stats = t.stats();
    let mut s = String::new();
    s.push_str(&tr.md_title(&t.date));
    s.push_str("\n\n");
    s.push_str(&format!("- {}\n", tr.md_topic(&t.topic)));
    s.push_str(&format!("- {}\n", tr.md_started(&t.started_at)));
    s.push_str(&format!(
        "- {}\n",
        tr.md_questions(stats.answered, stats.asked)
    ));
    s.push_str(&format!(
        "- {}\n",
        tr.md_duration(
            &format_duration(stats.total_seconds),
            &format_duration(stats.avg_seconds)
        )
    ));
    s.push_str(&format!("- {}\n\n", tr.md_solved(stats.solved)));

    if let Some(aha) = &t.aha {
        let note = if aha.note.trim().is_empty() {
            tr.md_no_note()
        } else {
            aha.note.trim()
        };
        s.push_str(&tr.md_aha(aha.after, &format_duration(aha.seconds_to_solution), note));
        s.push_str("\n\n");
    }

    for (i, entry) in t.entries.iter().enumerate() {
        s.push_str(&format!("### {}. {}\n\n", i + 1, entry.question));
        let answer = if entry.answer.trim().is_empty() {
            tr.md_no_answer()
        } else {
            entry.answer.trim()
        };
        s.push_str(answer);
        s.push_str("\n\n");
    }

    s.push_str("---\n\n");
    s
}

/// Writes the log (same day is appended) and returns its path.
pub fn write_log(t: &Transcript, tr: Tr) -> Result<PathBuf> {
    write_log_in(&paths::data_dir()?, t, tr)
}

/// Like [`write_log`], but with an explicit target directory (for tests).
fn write_log_in(dir: &Path, t: &Transcript, tr: Tr) -> Result<PathBuf> {
    fs::create_dir_all(dir)
        .with_context(|| format!("Could not create directory {}", dir.display()))?;
    let path = dir.join(format!("session-{}.md", t.date));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Could not open {}", path.display()))?;
    file.write_all(render_markdown(t, tr).as_bytes())
        .with_context(|| format!("Could not write {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    fn sample() -> Transcript {
        let mut t = Transcript::new("logic", "2026-06-05", "2026-06-05 14:32");
        t.push("What should it do?", "Compute the sum", 30);
        t.push("What happens?", "   ", 10);
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
    fn markdown_contains_sections_in_english() {
        let mut t = sample();
        t.aha = Some(Aha {
            note: "swapped index".into(),
            after: 2,
            seconds_to_solution: 40,
        });
        let md = render_markdown(&t, Tr::new(Lang::English));
        assert!(md.contains("# 🦆 Rubberduck session – 2026-06-05"));
        assert!(md.contains("**Topic:** logic"));
        assert!(md.contains("1 answered / 2 asked"));
        assert!(md.contains("✅ yes"));
        assert!(md.contains("💡 **Aha after question 2"));
        assert!(md.contains("swapped index"));
        assert!(md.contains("_(no answer)_"));
    }

    #[test]
    fn markdown_localizes_to_german() {
        let md = render_markdown(&sample(), Tr::new(Lang::German));
        assert!(md.contains("**Thema:** logic"));
        assert!(md.contains("1 beantwortet / 2 gestellt"));
    }

    #[test]
    fn write_log_appends() {
        let dir = tempfile::tempdir().unwrap();
        let tr = Tr::new(Lang::English);
        let path = write_log_in(dir.path(), &sample(), tr).unwrap();
        assert_eq!(path.file_name().unwrap(), "session-2026-06-05.md");
        write_log_in(dir.path(), &sample(), tr).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.matches("Rubberduck session").count(), 2);
    }
}
