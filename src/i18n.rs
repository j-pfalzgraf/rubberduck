//! Internationalization (i18n).
//!
//! English is the default language; German ships bundled. [`Lang`] selects the
//! language and [`Tr`] is a small, `Copy` translator value object that produces
//! every user-facing string. All translatable text lives here, so adding a
//! language means implementing the `match self.lang { … }` arms in one file.
//!
//! Language resolution at startup is: `--lang` flag › `RUBBERDUCK_LANG` env ›
//! `config.yaml` › English. The system locale is intentionally **not** read, so
//! the default stays deterministically English.

use std::fmt;

/// A supported user-interface language.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    clap::ValueEnum,
    Default,
)]
pub enum Lang {
    /// English (default).
    #[default]
    #[value(name = "en", alias = "english")]
    #[serde(rename = "en")]
    English,
    /// German / Deutsch.
    #[value(name = "de", alias = "german", alias = "deutsch")]
    #[serde(rename = "de")]
    German,
}

impl Lang {
    /// All supported languages.
    pub const ALL: [Lang; 2] = [Lang::English, Lang::German];

    /// The short ISO-639-1 code (`"en"` / `"de"`).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Lang::English => "en",
            Lang::German => "de",
        }
    }

    /// The endonym shown in pickers (`"English"` / `"Deutsch"`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Lang::English => "English",
            Lang::German => "Deutsch",
        }
    }

    /// Parses a language code/name (`"en"`, `"english"`, `"de"`, `"deutsch"` …).
    #[must_use]
    pub fn from_code(s: &str) -> Option<Lang> {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(Lang::English),
            "de" | "german" | "deutsch" => Some(Lang::German),
            _ => None,
        }
    }

    /// Reads the `RUBBERDUCK_LANG` environment variable; `None` if unset/unknown.
    #[must_use]
    pub fn from_env() -> Option<Lang> {
        std::env::var("RUBBERDUCK_LANG")
            .ok()
            .and_then(|v| Lang::from_code(&v))
    }

    /// A translator bound to this language.
    #[must_use]
    pub fn translator(self) -> Tr {
        Tr { lang: self }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// A translator: turns message keys into localized, user-facing strings.
///
/// `Tr` is `Copy`, so it can be threaded cheaply through the UI, the controller
/// and the session writer.
#[derive(Debug, Clone, Copy)]
pub struct Tr {
    lang: Lang,
}

impl Tr {
    /// Creates a translator for `lang`.
    #[must_use]
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// The language this translator renders.
    #[must_use]
    pub fn lang(self) -> Lang {
        self.lang
    }

    // ----- session greeting & dialog --------------------------------------

    /// Opening line the duck greets the user with.
    #[must_use]
    pub fn greeting(self, topic: &str) -> String {
        match self.lang {
            Lang::English => format!(
                "Hi! Topic: {topic}. Walk me through your problem — step by step. \
                 (Type !aha the moment it clicks.)"
            ),
            Lang::German => format!(
                "Hi! Thema: {topic}. Erklär mir dein Problem – Schritt für Schritt. \
                 (Tippe !aha, sobald der Groschen fällt.)"
            ),
        }
    }

    /// Label of the "thinking" spinner shown before the questions.
    #[must_use]
    pub fn pondering(self) -> &'static str {
        match self.lang {
            Lang::English => "The duck is mulling over good questions …",
            Lang::German => "Die Ente überlegt sich gute Fragen …",
        }
    }

    /// Prompt prefix for the user's answer.
    #[must_use]
    pub fn answer_prompt(self) -> &'static str {
        match self.lang {
            Lang::English => "  You",
            Lang::German => "  Du",
        }
    }

    /// Shown when the session is cancelled mid-way.
    #[must_use]
    pub fn aborted_session(self) -> &'static str {
        match self.lang {
            Lang::English => "Cancelled — see you next quack!",
            Lang::German => "Abgebrochen – bis zum nächsten Quaken!",
        }
    }

    /// Shown when the topic picker is cancelled.
    #[must_use]
    pub fn aborted_no_topic(self) -> &'static str {
        match self.lang {
            Lang::English => "Cancelled — no topic chosen.",
            Lang::German => "Abgebrochen – kein Thema gewählt.",
        }
    }

    /// Prompt of the interactive topic picker.
    #[must_use]
    pub fn pick_topic_prompt(self) -> &'static str {
        match self.lang {
            Lang::English => "Which topic would you like to go through?",
            Lang::German => "Welches Thema möchtest du durchgehen?",
        }
    }

    /// End-of-session confirmation: was the bug found?
    #[must_use]
    pub fn end_confirm(self) -> &'static str {
        match self.lang {
            Lang::English => "Aha — did you find the bug?",
            Lang::German => "Aha – hast du den Bug gefunden?",
        }
    }

    /// Prompt for the optional aha note.
    #[must_use]
    pub fn aha_note_prompt(self) -> &'static str {
        match self.lang {
            Lang::English => "What was it? (short note, press Enter to skip)",
            Lang::German => "Was war's? (kurze Notiz, Enter überspringt)",
        }
    }

    /// Closing line after a celebrated aha moment.
    #[must_use]
    pub fn aha_closing(self) -> &'static str {
        match self.lang {
            Lang::English => "Nice! Explaining helps — that's exactly what I'm here for.",
            Lang::German => "Stark! Erklären hilft – genau dafür bin ich da.",
        }
    }

    /// Quiet-mode replacement for the celebration animation.
    #[must_use]
    pub fn celebrate_quiet(self) -> &'static str {
        match self.lang {
            Lang::English => "✨ Nice — found it!",
            Lang::German => "✨ Stark – gefunden!",
        }
    }

    /// The word the duck quacks in its animation.
    #[must_use]
    pub fn quack_word(self) -> &'static str {
        match self.lang {
            Lang::English => "Quack!",
            Lang::German => "Quak!",
        }
    }

    /// The celebration banner word.
    #[must_use]
    pub fn eureka(self) -> &'static str {
        match self.lang {
            Lang::English => "EUREKA!",
            Lang::German => "HEUREKA!",
        }
    }

    // ----- summary --------------------------------------------------------

    /// Heading of the end-of-session summary block.
    #[must_use]
    pub fn summary_header(self) -> &'static str {
        match self.lang {
            Lang::English => "──────── Summary ────────",
            Lang::German => "──────── Zusammenfassung ────────",
        }
    }

    /// Summary line: questions answered.
    #[must_use]
    pub fn summary_answered(self, answered: usize, asked: usize) -> String {
        match self.lang {
            Lang::English => format!("{answered} / {asked} questions answered"),
            Lang::German => format!("{answered} / {asked} Fragen beantwortet"),
        }
    }

    /// Summary line: total and average duration.
    #[must_use]
    pub fn summary_duration(self, total: &str, avg: &str) -> String {
        match self.lang {
            Lang::English => format!("Duration: {total} ({avg} avg per question)"),
            Lang::German => format!("Dauer: {total} (Ø {avg} pro Frage)"),
        }
    }

    /// Summary line: bug found.
    #[must_use]
    pub fn summary_solved(self) -> &'static str {
        match self.lang {
            Lang::English => "✅ Bug found",
            Lang::German => "✅ Bug gefunden",
        }
    }

    /// Summary line: not solved yet.
    #[must_use]
    pub fn summary_open(self) -> &'static str {
        match self.lang {
            Lang::English => "– still open",
            Lang::German => "– noch offen",
        }
    }

    /// Confirmation that the log file was written.
    #[must_use]
    pub fn log_saved(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!("Log saved: {path}"),
            Lang::German => format!("Logbuch gespeichert: {path}"),
        }
    }

    // ----- topics & config commands ---------------------------------------

    /// Heading of `rubberduck topics`.
    #[must_use]
    pub fn topics_header(self) -> &'static str {
        match self.lang {
            Lang::English => "Available topics:",
            Lang::German => "Verfügbare Themen:",
        }
    }

    /// Hint printed after the topic list.
    #[must_use]
    pub fn topics_hint(self) -> &'static str {
        match self.lang {
            Lang::English => "Start with:  rubberduck --topic <name>   (* = default)",
            Lang::German => "Start mit:  rubberduck --topic <name>   (* = Standard)",
        }
    }

    /// `config show` heading with the file path.
    #[must_use]
    pub fn config_settings_header(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!("Settings ({path})"),
            Lang::German => format!("Einstellungen ({path})"),
        }
    }

    /// `config init` when the file already exists.
    #[must_use]
    pub fn config_exists(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!("Already exists: {path}"),
            Lang::German => format!("Existiert bereits: {path}"),
        }
    }

    /// `config init` after creating the file.
    #[must_use]
    pub fn config_created(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!("Created: {path}"),
            Lang::German => format!("Angelegt: {path}"),
        }
    }

    // ----- self update / uninstall ----------------------------------------

    /// `self update --check`: an update is available.
    #[must_use]
    pub fn update_available(self, current: &str, latest: &str) -> String {
        match self.lang {
            Lang::English => format!("🦆 Update available: {current} → {latest}"),
            Lang::German => format!("🦆 Update verfügbar: {current} → {latest}"),
        }
    }

    /// `self update --check`: already up to date.
    #[must_use]
    pub fn up_to_date(self, current: &str) -> String {
        match self.lang {
            Lang::English => format!("🦆 rubberduck is up to date (version {current})."),
            Lang::German => format!("🦆 rubberduck ist aktuell (Version {current})."),
        }
    }

    /// `self update`: no releases found.
    #[must_use]
    pub fn no_releases(self) -> &'static str {
        match self.lang {
            Lang::English => "No releases found.",
            Lang::German => "Keine Releases gefunden.",
        }
    }

    /// `self update`: binary was updated.
    #[must_use]
    pub fn updated_to(self, version: &str) -> String {
        match self.lang {
            Lang::English => format!("🦆 Updated to version {version}."),
            Lang::German => format!("🦆 Aktualisiert auf Version {version}."),
        }
    }

    /// `self update`: already on the latest version.
    #[must_use]
    pub fn already_current(self, version: &str) -> String {
        match self.lang {
            Lang::English => format!("🦆 Already up to date (version {version})."),
            Lang::German => format!("🦆 Bereits aktuell (Version {version})."),
        }
    }

    /// Uninstall: header listing what will be removed.
    #[must_use]
    pub fn uninstall_header(self) -> &'static str {
        match self.lang {
            Lang::English => "The following will be removed:",
            Lang::German => "Folgendes wird entfernt:",
        }
    }

    /// Uninstall: label for the binary.
    #[must_use]
    pub fn uninstall_label_binary(self) -> &'static str {
        match self.lang {
            Lang::English => "Binary",
            Lang::German => "Binary",
        }
    }

    /// Uninstall: label for the configuration directory.
    #[must_use]
    pub fn uninstall_label_config(self) -> &'static str {
        match self.lang {
            Lang::English => "Config",
            Lang::German => "Konfiguration",
        }
    }

    /// Uninstall: label for the logs directory.
    #[must_use]
    pub fn uninstall_label_logs(self) -> &'static str {
        match self.lang {
            Lang::English => "Logs",
            Lang::German => "Logs",
        }
    }

    /// Uninstall: confirmation prompt.
    #[must_use]
    pub fn uninstall_confirm(self) -> &'static str {
        match self.lang {
            Lang::English => "Really remove everything?",
            Lang::German => "Wirklich alles entfernen?",
        }
    }

    /// Uninstall: cancelled by the user.
    #[must_use]
    pub fn uninstall_cancelled(self) -> &'static str {
        match self.lang {
            Lang::English => "Cancelled — nothing was removed.",
            Lang::German => "Abgebrochen – nichts wurde entfernt.",
        }
    }

    /// Uninstall: one path was removed.
    #[must_use]
    pub fn uninstall_removed(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!("Removed: {path}"),
            Lang::German => format!("Entfernt: {path}"),
        }
    }

    /// Uninstall: needs an interactive terminal.
    #[must_use]
    pub fn uninstall_needs_tty(self) -> &'static str {
        match self.lang {
            Lang::English => {
                "Uninstall needs an interactive confirmation and is not possible without a \
                 terminal. Use 'uninstall.sh --yes' instead."
            }
            Lang::German => {
                "Deinstallation braucht eine interaktive Bestätigung und ist ohne Terminal \
                 nicht möglich. Nutze stattdessen 'uninstall.sh --yes'."
            }
        }
    }

    /// Uninstall: refusing to delete a home/system directory.
    #[must_use]
    pub fn uninstall_unsafe(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!(
                "Aborting: '{path}' looks like a home/system directory and will not be deleted. \
                 Please check RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR."
            ),
            Lang::German => format!(
                "Abbruch: '{path}' sieht nach einem Home-/System-Verzeichnis aus und wird nicht \
                 gelöscht. Bitte RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR prüfen."
            ),
        }
    }

    /// Uninstall: success message.
    #[must_use]
    pub fn uninstall_done(self) -> &'static str {
        match self.lang {
            Lang::English => "rubberduck has been removed. Thanks for the quacks! 🦆",
            Lang::German => "rubberduck wurde entfernt. Danke fürs Quaken! 🦆",
        }
    }

    /// Uninstall: dirs removed but the binary could not be deleted.
    #[must_use]
    pub fn uninstall_binary_failed(self, path: &str) -> String {
        match self.lang {
            Lang::English => format!(
                "Config and logs were removed, but the binary at {path} could not be deleted. \
                 Please remove it manually."
            ),
            Lang::German => format!(
                "Konfiguration und Logs wurden entfernt, aber das Binary unter {path} konnte \
                 nicht gelöscht werden. Bitte manuell entfernen."
            ),
        }
    }

    // ----- markdown log ---------------------------------------------------

    /// Log: top-level title.
    #[must_use]
    pub fn md_title(self, date: &str) -> String {
        match self.lang {
            Lang::English => format!("# 🦆 Rubberduck session – {date}"),
            Lang::German => format!("# 🦆 Rubberduck-Session – {date}"),
        }
    }

    /// Log: topic line (without the leading `- `).
    #[must_use]
    pub fn md_topic(self, topic: &str) -> String {
        match self.lang {
            Lang::English => format!("**Topic:** {topic}"),
            Lang::German => format!("**Thema:** {topic}"),
        }
    }

    /// Log: started-at line.
    #[must_use]
    pub fn md_started(self, started_at: &str) -> String {
        match self.lang {
            Lang::English => format!("**Started:** {started_at}"),
            Lang::German => format!("**Gestartet:** {started_at}"),
        }
    }

    /// Log: questions answered/asked line.
    #[must_use]
    pub fn md_questions(self, answered: usize, asked: usize) -> String {
        match self.lang {
            Lang::English => format!("**Questions:** {answered} answered / {asked} asked"),
            Lang::German => format!("**Fragen:** {answered} beantwortet / {asked} gestellt"),
        }
    }

    /// Log: duration line.
    #[must_use]
    pub fn md_duration(self, total: &str, avg: &str) -> String {
        match self.lang {
            Lang::English => format!("**Duration:** {total} ({avg} avg per question)"),
            Lang::German => format!("**Dauer:** {total} (Ø {avg} pro Frage)"),
        }
    }

    /// Log: solved line (`✅ yes` / `– open`).
    #[must_use]
    pub fn md_solved(self, solved: bool) -> String {
        let value = match (self.lang, solved) {
            (Lang::English, true) => "✅ yes",
            (Lang::English, false) => "– open",
            (Lang::German, true) => "✅ ja",
            (Lang::German, false) => "– offen",
        };
        match self.lang {
            Lang::English => format!("**Solved:** {value}"),
            Lang::German => format!("**Gelöst:** {value}"),
        }
    }

    /// Log: the aha-moment blockquote.
    #[must_use]
    pub fn md_aha(self, after: usize, time: &str, note: &str) -> String {
        match self.lang {
            Lang::English => format!("> 💡 **Aha after question {after} ({time}):** {note}"),
            Lang::German => format!("> 💡 **Aha nach Frage {after} ({time}):** {note}"),
        }
    }

    /// Log: placeholder for a missing aha note.
    #[must_use]
    pub fn md_no_note(self) -> &'static str {
        match self.lang {
            Lang::English => "(no note)",
            Lang::German => "(keine Notiz)",
        }
    }

    /// Log: placeholder for an unanswered question.
    #[must_use]
    pub fn md_no_answer(self) -> &'static str {
        match self.lang {
            Lang::English => "_(no answer)_",
            Lang::German => "_(keine Antwort)_",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_english() {
        assert_eq!(Lang::default(), Lang::English);
    }

    #[test]
    fn codes_round_trip() {
        for lang in Lang::ALL {
            assert_eq!(Lang::from_code(lang.code()), Some(lang));
        }
        assert_eq!(Lang::from_code("English"), Some(Lang::English));
        assert_eq!(Lang::from_code("deutsch"), Some(Lang::German));
        assert_eq!(Lang::from_code("xx"), None);
    }

    #[test]
    fn english_and_german_differ_and_interpolate() {
        let en = Tr::new(Lang::English);
        let de = Tr::new(Lang::German);
        assert!(en.greeting("logic").contains("Topic: logic"));
        assert!(de.greeting("logic").contains("Thema: logic"));
        assert_ne!(en.summary_solved(), de.summary_solved());
        assert!(en.md_questions(1, 2).contains("1 answered / 2 asked"));
    }

    #[test]
    fn serde_uses_short_codes() {
        let yaml = serde_yaml::to_string(&Lang::German).unwrap();
        assert!(yaml.contains("de"));
        let back: Lang = serde_yaml::from_str("en").unwrap();
        assert_eq!(back, Lang::English);
    }
}
