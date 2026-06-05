//! Anwendungs-Orchestrierung: Themenwahl, Frage-Dialog, Aha-Moment und Statistik.
//!
//! [`App`] ist der Controller: Er verbindet [`Ui`](crate::ui::Ui),
//! [`QuestionPool`](crate::questions::QuestionPool) und das
//! [`Transcript`](crate::session::Transcript) zu einer Session.

use crate::questions::{QuestionPool, Topic, DEFAULT_TOPIC};
use crate::session::{self, Aha, Transcript};
use crate::ui::{Mood, Ui};
use anyhow::{Context, Result};
use chrono::Local;
use dialoguer::{Confirm, Input, Select};
use std::io::ErrorKind;
use std::time::Instant;

/// Spezialeingabe, mit der man den Aha-Moment sofort auslöst.
const AHA_TRIGGER: &str = "!aha";

/// Stimmungen, die der Reihe nach für die Fragen verwendet werden.
const QUESTION_MOODS: [Mood; 3] = [Mood::Thinking, Mood::Curious, Mood::Listening];

/// Orchestriert eine komplette Debugging-Session.
pub struct App {
    ui: Ui,
    pool: QuestionPool,
    default_topic: String,
}

impl App {
    /// Neue App aus Oberfläche, Fragenpool und konfiguriertem Standardthema.
    #[must_use]
    pub fn new(ui: Ui, pool: QuestionPool, default_topic: String) -> Self {
        Self {
            ui,
            pool,
            default_topic,
        }
    }

    /// Führt eine Session zum (optionalen) Thema und protokolliert bei `log`.
    pub fn run(&mut self, requested_topic: Option<&str>, log: bool) -> Result<()> {
        let Some(topic_name) = self.resolve_topic(requested_topic)? else {
            println!(
                "{}",
                self.ui.styler().dim("Abgebrochen – kein Thema gewählt.")
            );
            return Ok(());
        };
        // Klonen, damit kein Borrow auf `self.pool` während `&mut self.ui` lebt.
        let topic = self.pool.topic(&topic_name)?.clone();

        let transcript = self.run_dialog(&topic)?;
        self.print_summary(&transcript);

        if log {
            let path = session::write_log(&transcript)?;
            println!(
                "{}",
                self.ui
                    .styler()
                    .dim(&format!("Logbuch gespeichert: {}", path.display()))
            );
        }
        Ok(())
    }

    /// Wählt das Thema: per `--topic`, interaktivem Picker oder Standard.
    fn resolve_topic(&mut self, requested: Option<&str>) -> Result<Option<String>> {
        if let Some(name) = requested {
            self.pool.topic(name)?; // validieren (Fehler listet verfügbare Themen)
            return Ok(Some(name.to_string()));
        }
        if self.ui.is_interactive() && !self.ui.is_quiet() {
            // Interaktiver Picker; `None` heißt: bewusst abgebrochen.
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
            .context("Der Fragenpool enthält keine Themen.")?;
        Ok(Some(first))
    }

    /// Interaktiver Themen-Picker; `None`, wenn die Auswahl abgebrochen wurde.
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
            .with_prompt("Welches Thema möchtest du durchgehen?")
            .items(&labels)
            .default(default_idx)
            .interact_opt()
            .context("Themen-Auswahl fehlgeschlagen")?;
        Ok(selection.map(|i| topics[i].name.clone()))
    }

    /// Der eigentliche Frage-Dialog.
    fn run_dialog(&mut self, topic: &Topic) -> Result<Transcript> {
        let now = Local::now();
        let mut transcript = Transcript::new(
            topic.name.clone(),
            now.format("%Y-%m-%d").to_string(),
            now.format("%Y-%m-%d %H:%M").to_string(),
        );
        let session_start = Instant::now();

        self.ui.swim_in(Mood::Idle)?;
        self.ui.quack(Mood::Idle)?;
        self.ui.duck_says(
            &format!(
                "Hi! Thema: {}. Erklär mir dein Problem – Schritt für Schritt. \
                 (Tippe !aha, sobald der Groschen fällt.)",
                topic.name
            ),
            Mood::Listening,
        )?;
        self.ui
            .thinking("Die Ente überlegt sich gute Fragen …", 10)?;

        for (i, question) in topic.questions.iter().enumerate() {
            self.ui
                .duck_says(question, QUESTION_MOODS[i % QUESTION_MOODS.len()])?;

            let asked_at = Instant::now();
            match self.read_answer()? {
                Some(answer) => {
                    let secs = asked_at.elapsed().as_secs();
                    if let Some(note) = aha_note(&answer) {
                        // Die !aha-Frage wird NICHT als (leere) Antwort gezählt.
                        self.trigger_aha(&mut transcript, note, session_start)?;
                        transcript.total_seconds = session_start.elapsed().as_secs();
                        return Ok(transcript);
                    }
                    transcript.push(question.clone(), answer, secs);
                }
                None => {
                    self.ui
                        .duck_says("Abgebrochen – bis zum nächsten Quaken!", Mood::Idle)
                        .ok();
                    transcript.total_seconds = session_start.elapsed().as_secs();
                    return Ok(transcript);
                }
            }
        }
        transcript.total_seconds = session_start.elapsed().as_secs();

        // Abschluss-Frage nach dem Aha-Moment (nur interaktiv).
        if transcript.aha.is_none() && self.ui.is_interactive() && !self.ui.is_quiet() {
            let solved = Confirm::new()
                .with_prompt("Aha – hast du den Bug gefunden?")
                .default(false)
                .interact_opt()
                .context("Abfrage fehlgeschlagen")?
                .unwrap_or(false);
            if solved {
                let note: String = Input::new()
                    .with_prompt("Was war's? (kurze Notiz, Enter überspringt)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                self.trigger_aha(&mut transcript, note, session_start)?;
            }
        }
        Ok(transcript)
    }

    /// Hält den Aha-Moment fest und feiert ihn.
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
        self.ui.duck_says(
            "Stark! Erklären hilft – genau dafür bin ich da.",
            Mood::Happy,
        )?;
        Ok(())
    }

    /// Liest eine Antwort; `None` bei Abbruch (Strg-C/ESC) oder ohne Terminal.
    fn read_answer(&self) -> Result<Option<String>> {
        match Input::<String>::new()
            .with_prompt("  Du")
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
            Err(e) => Err(e).context("Eingabe fehlgeschlagen"),
        }
    }

    /// Druckt die Abschluss-Statistik.
    fn print_summary(&self, t: &Transcript) {
        let s = t.stats();
        let st = self.ui.styler();
        println!();
        println!("{}", st.dim("──────── Zusammenfassung ────────"));
        println!(
            "  {} {} / {} Fragen beantwortet",
            st.accent("•"),
            s.answered,
            s.asked
        );
        println!(
            "  {} Dauer: {} (Ø {} pro Frage)",
            st.accent("•"),
            session::format_duration(s.total_seconds),
            session::format_duration(s.avg_seconds)
        );
        let solved = if s.solved {
            st.success("✅ Bug gefunden")
        } else {
            st.dim("– noch offen")
        };
        println!("  {} {}", st.accent("•"), solved);
    }
}

/// Erkennt die `!aha`-Spezialeingabe und liefert die (ggf. leere) Notiz dahinter.
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
            aha_note("  !aha Index vertauscht "),
            Some("Index vertauscht".into())
        );
        assert_eq!(aha_note("eine normale Antwort"), None);
        assert_eq!(aha_note("aha"), None); // nur mit '!'
    }
}
