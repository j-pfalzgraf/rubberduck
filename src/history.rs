//! Session history and aggregate statistics.
//!
//! Each finished session is appended as one JSON line to
//! `~/.rubberduck/history.jsonl` (when enabled via `config.history`). The
//! `stats` command reads it back and shows aggregate metrics. The format is
//! append-only and tolerant: malformed lines are skipped, so a corrupt entry
//! never breaks the stats view.

use crate::paths;
use crate::session::{format_duration, Transcript};
use crate::ui::gradient::Gradient;
use crate::ui::Ui;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// One persisted session record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// Session date `YYYY-MM-DD`.
    pub date: String,
    /// Topic name.
    pub topic: String,
    /// Questions asked.
    pub asked: usize,
    /// Questions answered.
    pub answered: usize,
    /// Total session duration (seconds).
    pub total_seconds: u64,
    /// Whether the bug was found.
    pub solved: bool,
    /// Time to solution (seconds), if solved.
    #[serde(default)]
    pub seconds_to_solution: Option<u64>,
}

impl Record {
    /// Builds a record from a finished [`Transcript`].
    #[must_use]
    pub fn from_transcript(t: &Transcript) -> Self {
        let stats = t.stats();
        Self {
            date: t.date.clone(),
            topic: t.topic.clone(),
            asked: stats.asked,
            answered: stats.answered,
            total_seconds: stats.total_seconds,
            solved: stats.solved,
            seconds_to_solution: t.aha.as_ref().map(|a| a.seconds_to_solution),
        }
    }
}

/// Path of the history file (`<data>/history.jsonl`).
fn history_file() -> Result<PathBuf> {
    Ok(paths::data_dir()?.join("history.jsonl"))
}

/// Appends `record` as one JSON line.
pub fn append(record: &Record) -> Result<()> {
    append_in(&history_file()?, record)
}

/// Like [`append`], but with an explicit path (for tests).
fn append_in(path: &Path, record: &Record) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("Could not create {}", dir.display()))?;
    }
    let line = serde_json::to_string(record).context("Could not serialize history record")?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Could not open {}", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("Could not write {}", path.display()))?;
    Ok(())
}

/// Loads all records (skipping malformed lines).
pub fn load_all() -> Result<Vec<Record>> {
    load_all_from(&history_file()?)
}

/// Like [`load_all`], but with an explicit path (for tests).
fn load_all_from(path: &Path) -> Result<Vec<Record>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Record>(l).ok())
        .collect())
}

/// Deletes the history file (if it exists).
pub fn clear() -> Result<()> {
    let path = history_file()?;
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Could not remove {}", path.display()))?;
    }
    Ok(())
}

/// Default number of recent sessions shown by `rubberduck history`.
const DEFAULT_HISTORY_LIMIT: usize = 10;

/// Shows the most recent recorded sessions (`rubberduck history`).
///
/// Human output is a compact, newest-first table; `json` emits a stable object
/// for scripts. `limit` caps how many sessions are shown (newest first).
pub fn show(ui: &mut Ui, limit: Option<usize>, json: bool) -> Result<()> {
    let records = load_all()?;
    // File order is oldest-first (append-only); present newest-first.
    let newest_first: Vec<&Record> = records.iter().rev().collect();

    if json {
        let shown: Vec<&Record> = match limit {
            Some(n) => newest_first.into_iter().take(n).collect(),
            None => newest_first,
        };
        let view = HistoryView {
            total: records.len(),
            shown: shown.len(),
            sessions: shown,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&view).context("Could not serialize history")?
        );
        return Ok(());
    }

    let tr = ui.tr();
    ui.gradient_banner(tr.history_header(), &Gradient::sunrise());

    if records.is_empty() {
        println!("{}", ui.styler().dim(tr.history_empty()));
        return Ok(());
    }

    let cap = limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    let shown: Vec<&Record> = newest_first.into_iter().take(cap).collect();
    let st = ui.styler();
    for r in &shown {
        let glyph = if r.solved {
            st.success("✓")
        } else {
            st.dim("·")
        };
        println!(
            "  {}  {} {}  {}  {}",
            st.dim(&r.date),
            glyph,
            st.text(&format!("{:<12}", r.topic)),
            st.dim(&format!("{}/{} ", r.answered, r.asked)),
            st.dim(&format_duration(r.total_seconds)),
        );
    }
    if records.len() > shown.len() {
        println!(
            "\n{}",
            st.dim(&tr.history_showing(shown.len(), records.len()))
        );
    }
    Ok(())
}

/// Machine-readable view for `history --json`.
#[derive(Serialize)]
struct HistoryView<'a> {
    total: usize,
    shown: usize,
    sessions: Vec<&'a Record>,
}

/// Aggregate metrics over a topic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TopicAggregate {
    /// Number of sessions on this topic.
    pub sessions: usize,
    /// Number that were solved.
    pub solved: usize,
}

impl TopicAggregate {
    /// Percentage of this topic's sessions that were solved (0–100).
    #[must_use]
    pub fn solve_rate(&self) -> u32 {
        (self.solved * 100).checked_div(self.sessions).unwrap_or(0) as u32
    }
}

/// Aggregate metrics over a set of [`Record`]s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Aggregate {
    /// Total number of sessions.
    pub sessions: usize,
    /// Number of solved sessions.
    pub solved: usize,
    /// Sum of all session durations (seconds).
    pub total_seconds: u64,
    /// Sum of time-to-solution over solved sessions (seconds).
    pub solution_seconds_sum: u64,
    /// Number of sessions that contributed a time-to-solution.
    pub solution_count: usize,
    /// Per-topic breakdown, sorted by topic name.
    pub per_topic: BTreeMap<String, TopicAggregate>,
}

impl Aggregate {
    /// Whether there are no sessions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions == 0
    }

    /// Percentage of sessions that were solved (0–100).
    #[must_use]
    pub fn solve_rate(&self) -> u32 {
        (self.solved * 100).checked_div(self.sessions).unwrap_or(0) as u32
    }

    /// Average session duration (seconds).
    #[must_use]
    pub fn avg_total_seconds(&self) -> u64 {
        self.total_seconds
            .checked_div(self.sessions as u64)
            .unwrap_or(0)
    }

    /// Average time to solution over solved sessions (seconds).
    #[must_use]
    pub fn avg_solution_seconds(&self) -> u64 {
        self.solution_seconds_sum
            .checked_div(self.solution_count as u64)
            .unwrap_or(0)
    }
}

/// Aggregates a slice of records.
#[must_use]
pub fn aggregate(records: &[Record]) -> Aggregate {
    let mut agg = Aggregate::default();
    for r in records {
        agg.sessions += 1;
        agg.total_seconds += r.total_seconds;
        if r.solved {
            agg.solved += 1;
        }
        if let Some(secs) = r.seconds_to_solution {
            agg.solution_seconds_sum += secs;
            agg.solution_count += 1;
        }
        let topic = agg.per_topic.entry(r.topic.clone()).or_default();
        topic.sessions += 1;
        if r.solved {
            topic.solved += 1;
        }
    }
    agg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(topic: &str, solved: bool, total: u64, sol: Option<u64>) -> Record {
        Record {
            date: "2026-06-06".into(),
            topic: topic.into(),
            asked: 3,
            answered: 2,
            total_seconds: total,
            solved,
            seconds_to_solution: sol,
        }
    }

    #[test]
    fn aggregate_computes_metrics() {
        let records = vec![
            rec("logic", true, 60, Some(40)),
            rec("logic", false, 120, None),
            rec("api", true, 30, Some(20)),
        ];
        let agg = aggregate(&records);
        assert_eq!(agg.sessions, 3);
        assert_eq!(agg.solved, 2);
        assert_eq!(agg.solve_rate(), 66);
        assert_eq!(agg.avg_total_seconds(), 70);
        assert_eq!(agg.avg_solution_seconds(), 30);
        assert_eq!(agg.per_topic["logic"].sessions, 2);
        assert_eq!(agg.per_topic["logic"].solved, 1);
        assert_eq!(agg.per_topic["api"].solved, 1);
    }

    #[test]
    fn empty_aggregate_is_safe() {
        let agg = aggregate(&[]);
        assert!(agg.is_empty());
        assert_eq!(agg.solve_rate(), 0);
        assert_eq!(agg.avg_total_seconds(), 0);
        assert_eq!(agg.avg_solution_seconds(), 0);
    }

    #[test]
    fn append_and_load_round_trip_skipping_garbage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        append_in(&path, &rec("logic", true, 60, Some(40))).unwrap();
        append_in(&path, &rec("api", false, 30, None)).unwrap();
        // A malformed line must be skipped, not fatal.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"not json\n")
            .unwrap();
        let loaded = load_all_from(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].topic, "logic");
    }

    #[test]
    fn missing_history_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_all_from(&dir.path().join("nope.jsonl")).unwrap();
        assert!(loaded.is_empty());
    }
}
