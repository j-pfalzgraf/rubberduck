//! Application orchestration: topic selection, the question dialog, the aha
//! moment and the summary.
//!
//! [`App`] is the controller: it wires [`Ui`](crate::ui::Ui),
//! [`QuestionPool`](crate::questions::QuestionPool) and the
//! [`Transcript`](crate::session::Transcript) into one session.

use crate::i18n::Tr;
use crate::questions::{QuestionPool, Topic, DEFAULT_TOPIC};
use crate::session::{self, Aha, Transcript};
use crate::ui::{Mood, Ui};
use anyhow::{Context, Result};
use chrono::Local;
use dialoguer::{Confirm, Input, Select};
use std::io::ErrorKind;
use std::time::Instant;

/// Special input that triggers the aha moment immediately.
const AHA_TRIGGER: &str = "!aha";

/// Moods cycled through, one per question.
const QUESTION_MOODS: [Mood; 3] = [Mood::Thinking, Mood::Curious, Mood::Listening];

/// Orchestrates a complete debugging session.
pub struct App {
    ui: Ui,
    pool: QuestionPool,
    default_topic: String,
    record_history: bool,
    tr: Tr,
}

impl App {
    /// New app from the UI, the question pool, the default topic and whether to
    /// record finished sessions to the history (for `stats`).
    #[must_use]
    pub fn new(ui: Ui, pool: QuestionPool, default_topic: String, record_history: bool) -> Self {
        let tr = ui.tr();
        Self {
            ui,
            pool,
            default_topic,
            record_history,
            tr,
        }
    }

    /// Runs a session for the (optional) topic and logs it when `log` is set.
    pub fn run(&mut self, requested_topic: Option<&str>, log: bool) -> Result<()> {
        let Some(topic_name) = self.resolve_topic(requested_topic)? else {
            println!("{}", self.ui.styler().dim(self.tr.aborted_no_topic()));
            return Ok(());
        };
        // Clone so no borrow of `self.pool` lives while `&mut self.ui` is in use.
        let topic = self.pool.topic(&topic_name)?.clone();

        let transcript = self.run_dialog(&topic)?;
        self.print_summary(&transcript);

        if log {
            let path = session::write_log(&transcript, self.tr)?;
            let msg = self.tr.log_saved(&path.display().to_string());
            println!("{}", self.ui.styler().dim(&msg));
        }

        // Record the session for `stats` (best-effort; skip trivial no-op runs).
        if self.record_history && (!transcript.entries.is_empty() || transcript.aha.is_some()) {
            let _ = crate::history::append(&crate::history::Record::from_transcript(&transcript));
        }
        Ok(())
    }

    /// Picks the topic: via `--topic`, the interactive picker, or the default.
    fn resolve_topic(&mut self, requested: Option<&str>) -> Result<Option<String>> {
        if let Some(name) = requested {
            self.pool.topic(name)?; // validate (error lists available topics)
            return Ok(Some(name.to_string()));
        }
        if self.ui.is_interactive() && !self.ui.is_quiet() {
            // Interactive picker; `None` means a deliberate cancel.
            return self.pick_topic();
        }
        if self.pool.topic(&self.default_topic).is_ok() {
            return Ok(Some(self.default_topic.clone()));
        }
        if self.pool.topic(DEFAULT_TOPIC).is_ok() {
            return Ok(Some(DEFAULT_TOPIC.to_string()));
        }
        let first = self
            .pool
            .topic_names()
            .first()
            .map(ToString::to_string)
            .context("The question pool has no topics.")?;
        Ok(Some(first))
    }

    /// Interactive topic picker; `None` if the selection was cancelled.
    fn pick_topic(&mut self) -> Result<Option<String>> {
        let topics: Vec<&Topic> = self.pool.topics().collect();
        let labels: Vec<String> = topics
            .iter()
            .map(|t| {
                if t.description.is_empty() {
                    t.name.clone()
                } else {
                    format!("{}  – {}", t.name, t.description)
                }
            })
            .collect();
        let default_idx = topics
            .iter()
            .position(|t| t.name == self.default_topic)
            .unwrap_or(0);

        let selection = Select::new()
            .with_prompt(self.tr.pick_topic_prompt())
            .items(&labels)
            .default(default_idx)
            .interact_opt()
            .context("Topic selection failed")?;
        Ok(selection.map(|i| topics[i].name.clone()))
    }

    /// The actual question dialog.
    fn run_dialog(&mut self, topic: &Topic) -> Result<Transcript> {
        let tr = self.tr;
        let now = Local::now();
        let mut transcript = Transcript::new(
            topic.name.clone(),
            now.format("%Y-%m-%d").to_string(),
            now.format("%Y-%m-%d %H:%M").to_string(),
        );
        let session_start = Instant::now();

        self.ui.swim_in(Mood::Idle)?;
        self.ui.quack(Mood::Idle)?;
        let greeting = tr.greeting(&topic.name);
        self.ui.duck_says(&greeting, Mood::Listening)?;
        self.ui.thinking(tr.pondering(), 10)?;

        for (i, question) in topic.questions.iter().enumerate() {
            self.ui
                .duck_says(question, QUESTION_MOODS[i % QUESTION_MOODS.len()])?;

            let asked_at = Instant::now();
            match self.read_answer()? {
                Some(answer) => {
                    let secs = asked_at.elapsed().as_secs();
                    if let Some(note) = aha_note(&answer) {
                        // The !aha question is NOT counted as an (empty) answer.
                        self.trigger_aha(&mut transcript, note, session_start)?;
                        transcript.total_seconds = session_start.elapsed().as_secs();
                        return Ok(transcript);
                    }
                    transcript.push(question.clone(), answer, secs);
                }
                None => {
                    self.ui.duck_says(tr.aborted_session(), Mood::Idle).ok();
                    transcript.total_seconds = session_start.elapsed().as_secs();
                    return Ok(transcript);
                }
            }
        }
        transcript.total_seconds = session_start.elapsed().as_secs();

        // Closing aha question (interactive only).
        if transcript.aha.is_none() && self.ui.is_interactive() && !self.ui.is_quiet() {
            let solved = Confirm::new()
                .with_prompt(tr.end_confirm())
                .default(false)
                .interact_opt()
                .context("Confirmation failed")?
                .unwrap_or(false);
            if solved {
                let note: String = Input::new()
                    .with_prompt(tr.aha_note_prompt())
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                self.trigger_aha(&mut transcript, note, session_start)?;
            }
        }
        Ok(transcript)
    }

    /// Records the aha moment and celebrates it.
    fn trigger_aha(
        &mut self,
        transcript: &mut Transcript,
        note: String,
        session_start: Instant,
    ) -> Result<()> {
        transcript.aha = Some(Aha {
            note,
            after: transcript.entries.len(),
            seconds_to_solution: session_start.elapsed().as_secs(),
        });
        self.ui.celebrate()?;
        self.ui.duck_says(self.tr.aha_closing(), Mood::Happy)?;
        Ok(())
    }

    /// Reads an answer; `None` on cancel (Ctrl-C/ESC) or without a terminal.
    fn read_answer(&self) -> Result<Option<String>> {
        match Input::<String>::new()
            .with_prompt(self.tr.answer_prompt())
            .allow_empty(true)
            .interact_text()
        {
            Ok(answer) => Ok(Some(answer)),
            Err(dialoguer::Error::IO(e))
                if matches!(
                    e.kind(),
                    ErrorKind::Interrupted | ErrorKind::NotConnected | ErrorKind::UnexpectedEof
                ) =>
            {
                Ok(None)
            }
            Err(e) => Err(e).context("Input failed"),
        }
    }

    /// Prints the closing statistics.
    fn print_summary(&self, t: &Transcript) {
        let tr = self.tr;
        let s = t.stats();
        let st = self.ui.styler();
        println!();
        println!("{}", st.dim(tr.summary_header()));
        println!(
            "  {} {}",
            st.accent("•"),
            tr.summary_answered(s.answered, s.asked)
        );
        println!(
            "  {} {}",
            st.accent("•"),
            tr.summary_duration(
                &session::format_duration(s.total_seconds),
                &session::format_duration(s.avg_seconds)
            )
        );
        let solved = if s.solved {
            st.success(tr.summary_solved())
        } else {
            st.dim(tr.summary_open())
        };
        println!("  {} {}", st.accent("•"), solved);
    }
}

/// Detects the `!aha` special input and returns the (possibly empty) note.
fn aha_note(answer: &str) -> Option<String> {
    let trimmed = answer.trim();
    let rest = trimmed.strip_prefix(AHA_TRIGGER)?;
    Some(rest.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aha_trigger_detected_with_and_without_note() {
        assert_eq!(aha_note("!aha"), Some(String::new()));
        assert_eq!(
            aha_note("  !aha index was off "),
            Some("index was off".into())
        );
        assert_eq!(aha_note("a normal answer"), None);
        assert_eq!(aha_note("aha"), None); // only with '!'
    }
}
