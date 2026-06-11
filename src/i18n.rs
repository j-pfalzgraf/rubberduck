//! Internationalization (i18n).
//!
//! English is the default language; German, French and Spanish ship bundled.
//! [`Lang`] selects the language and [`Tr`] is a small, `Copy` translator value
//! object that produces every user-facing string.
//!
//! # How translations are stored
//!
//! Every user-facing string lives in a `Catalog` — a plain data struct with one
//! field per message. There is exactly one `const` catalog per language (`EN`,
//! `DE`, `FR`, `ES`). The [`Tr`] methods are thin accessors over the active catalog;
//! they never branch on the language themselves. The payoff is DRY scalability:
//! **adding a language is one more `Catalog` literal**, and because a struct
//! literal must set every field, the compiler refuses to let a message be
//! forgotten.
//!
//! Messages that interpolate runtime values use `{name}` placeholders which the
//! `fill` helper expands, e.g. `"Topic: {topic}"`.
//!
//! Language resolution at startup is: `--lang` flag › `RUBBERDUCK_LANG` env ›
//! `config.yaml` › English. The system locale is intentionally **not** read, so
//! the default stays deterministically English.

use crate::ui::Mood;
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
#[non_exhaustive]
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
    /// French / Français.
    #[value(name = "fr", alias = "french", alias = "francais", alias = "français")]
    #[serde(rename = "fr")]
    French,
    /// Spanish / Español.
    #[value(name = "es", alias = "spanish", alias = "espanol", alias = "español")]
    #[serde(rename = "es")]
    Spanish,
}

impl Lang {
    /// All supported languages, in display order.
    pub const ALL: [Lang; 4] = [Lang::English, Lang::German, Lang::French, Lang::Spanish];

    /// The short ISO-639-1 code (`"en"` / `"de"` / `"fr"` / `"es"`).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Lang::English => "en",
            Lang::German => "de",
            Lang::French => "fr",
            Lang::Spanish => "es",
        }
    }

    /// The endonym shown in pickers (`"English"` / `"Deutsch"` / `"Français"` /
    /// `"Español"`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Lang::English => "English",
            Lang::German => "Deutsch",
            Lang::French => "Français",
            Lang::Spanish => "Español",
        }
    }

    /// Parses a language code/name (`"en"`, `"english"`, `"fr"`, `"español"` …).
    #[must_use]
    pub fn from_code(s: &str) -> Option<Lang> {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(Lang::English),
            "de" | "german" | "deutsch" => Some(Lang::German),
            "fr" | "french" | "francais" | "français" => Some(Lang::French),
            "es" | "spanish" | "espanol" | "español" => Some(Lang::Spanish),
            _ => None,
        }
    }

    /// A comma-separated list of every language code (`"en, de, fr"`).
    ///
    /// Derived from [`Lang::ALL`] so error messages and help stay in sync as
    /// languages are added.
    #[must_use]
    pub fn code_list() -> String {
        Self::ALL
            .iter()
            .map(|lang| lang.code())
            .collect::<Vec<_>>()
            .join(", ")
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

/// Expands `{name}` placeholders in a message template.
///
/// Each `(name, value)` pair replaces every `{name}` occurrence. Templates are
/// short and rendered infrequently, so this simple replace-based pass is plenty
/// fast and keeps the call sites readable.
fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// All user-facing strings for one language.
///
/// One `const` instance exists per language (`EN`, `DE`, `FR`, `ES`). Because
/// every field must be set in the literal, the compiler guarantees a new language
/// translates every message. Fields whose name maps to a templated message hold
/// a template with `{placeholder}` markers (see `fill`).
struct Catalog {
    // ----- session greeting & dialog --------------------------------------
    greeting: &'static str, // {topic}
    pondering: &'static str,
    answer_prompt: &'static str,
    aborted_session: &'static str,
    aborted_no_topic: &'static str,
    pick_topic_prompt: &'static str,
    end_confirm: &'static str,
    aha_note_prompt: &'static str,
    aha_closing: &'static str,
    celebrate_quiet: &'static str,
    quack_word: &'static str,
    eureka: &'static str,
    // ----- summary --------------------------------------------------------
    summary_header: &'static str,
    summary_answered: &'static str, // {answered} {asked}
    summary_duration: &'static str, // {total} {avg}
    summary_solved: &'static str,
    summary_open: &'static str,
    log_saved: &'static str, // {path}
    // ----- topics & config ------------------------------------------------
    topics_header: &'static str,
    topics_hint: &'static str,
    config_settings_header: &'static str, // {path}
    config_exists: &'static str,          // {path}
    config_created: &'static str,         // {path}
    config_set_done: &'static str,        // {key} {value}
    config_reset_done: &'static str,      // {path}
    // ----- demo -----------------------------------------------------------
    demo_title: &'static str,
    demo_intro: &'static str,
    demo_section_moods: &'static str,
    demo_section_themes: &'static str,
    demo_section_spinners: &'static str,
    demo_section_gradients: &'static str,
    demo_done: &'static str,
    // ----- moods ----------------------------------------------------------
    mood_idle: &'static str,
    mood_thinking: &'static str,
    mood_listening: &'static str,
    mood_happy: &'static str,
    mood_curious: &'static str,
    mood_surprised: &'static str,
    mood_celebrating: &'static str,
    mood_sleeping: &'static str,
    mood_confused: &'static str,
    mood_proud: &'static str,
    mood_reading: &'static str,
    // ----- stats ----------------------------------------------------------
    stats_header: &'static str,
    stats_empty: &'static str,
    stats_sessions: &'static str,     // {n}
    stats_solved: &'static str,       // {solved} {total} {pct}
    stats_avg_session: &'static str,  // {d}
    stats_avg_solution: &'static str, // {d}
    stats_by_topic: &'static str,
    stats_cleared: &'static str,
    // ----- tips -----------------------------------------------------------
    tip_label: &'static str,
    tips_header: &'static str,
    // ----- themes ---------------------------------------------------------
    themes_header: &'static str,
    themes_hint: &'static str,
    // ----- history --------------------------------------------------------
    history_header: &'static str,
    history_empty: &'static str,
    history_showing: &'static str, // {shown} {total}
    // ----- doctor ---------------------------------------------------------
    doctor_header: &'static str,
    doctor_version: &'static str,
    doctor_language: &'static str,
    doctor_theme: &'static str,
    doctor_color: &'static str,
    doctor_animations: &'static str,
    doctor_terminal: &'static str,
    doctor_config: &'static str,
    doctor_data: &'static str,
    doctor_questions: &'static str, // {topics} {lang}
    doctor_tips: &'static str,      // {count} {lang}
    doctor_enabled: &'static str,
    doctor_disabled: &'static str,
    doctor_interactive: &'static str,
    doctor_plain: &'static str,
    doctor_config_ok: &'static str,
    doctor_config_missing: &'static str,
    // ----- languages ------------------------------------------------------
    languages_header: &'static str,
    // ----- self update / uninstall ----------------------------------------
    update_available: &'static str, // {current} {latest}
    up_to_date: &'static str,       // {current}
    no_releases: &'static str,
    updated_to: &'static str,      // {version}
    already_current: &'static str, // {version}
    uninstall_header: &'static str,
    uninstall_label_binary: &'static str,
    uninstall_label_config: &'static str,
    uninstall_label_logs: &'static str,
    uninstall_confirm: &'static str,
    uninstall_cancelled: &'static str,
    uninstall_removed: &'static str, // {path}
    uninstall_needs_tty: &'static str,
    uninstall_unsafe: &'static str, // {path}
    uninstall_done: &'static str,
    uninstall_binary_failed: &'static str, // {path}
    // ----- markdown log ---------------------------------------------------
    md_title: &'static str,     // {date}
    md_topic: &'static str,     // {topic}
    md_started: &'static str,   // {started_at}
    md_questions: &'static str, // {answered} {asked}
    md_duration: &'static str,  // {total} {avg}
    md_solved: &'static str,    // {value}
    md_solved_yes: &'static str,
    md_solved_no: &'static str,
    md_aha: &'static str, // {after} {time} {note}
    md_no_note: &'static str,
    md_no_answer: &'static str,
}

/// English catalog (the default language).
const EN: Catalog = Catalog {
    greeting: "Hi! Topic: {topic}. Walk me through your problem — step by step. \
               (Type !aha the moment it clicks.)",
    pondering: "The duck is mulling over good questions …",
    answer_prompt: "  You",
    aborted_session: "Cancelled — see you next quack!",
    aborted_no_topic: "Cancelled — no topic chosen.",
    pick_topic_prompt: "Which topic would you like to go through?",
    end_confirm: "Aha — did you find the bug?",
    aha_note_prompt: "What was it? (short note, press Enter to skip)",
    aha_closing: "Nice! Explaining helps — that's exactly what I'm here for.",
    celebrate_quiet: "✨ Nice — found it!",
    quack_word: "Quack!",
    eureka: "EUREKA!",
    summary_header: "──────── Summary ────────",
    summary_answered: "{answered} / {asked} questions answered",
    summary_duration: "Duration: {total} ({avg} avg per question)",
    summary_solved: "✅ Bug found",
    summary_open: "– still open",
    log_saved: "Log saved: {path}",
    topics_header: "Available topics:",
    topics_hint: "Start with:  rubberduck --topic <name>   (* = default)",
    config_settings_header: "Settings ({path})",
    config_exists: "Already exists: {path}",
    config_created: "Created: {path}",
    config_set_done: "Set {key} = {value}",
    config_reset_done: "Reset to defaults: {path}",
    demo_title: "rubberduck — animation demo",
    demo_intro: "Watch closely — I type, I swim, I quack, and I celebrate when you win.",
    demo_section_moods: "Moods",
    demo_section_themes: "Themes",
    demo_section_spinners: "Spinners",
    demo_section_gradients: "Gradients",
    demo_done: "That's the tour — happy debugging!",
    mood_idle: "Idle",
    mood_thinking: "Thinking",
    mood_listening: "Listening",
    mood_happy: "Happy",
    mood_curious: "Curious",
    mood_surprised: "Surprised",
    mood_celebrating: "Celebrating",
    mood_sleeping: "Sleeping",
    mood_confused: "Confused",
    mood_proud: "Proud",
    mood_reading: "Reading",
    stats_header: "Your debugging stats",
    stats_empty: "No sessions recorded yet — run one to build your history.",
    stats_sessions: "Sessions: {n}",
    stats_solved: "Solved: {solved}/{total} ({pct}%)",
    stats_avg_session: "Avg session: {d}",
    stats_avg_solution: "Avg time to solution: {d}",
    stats_by_topic: "By topic",
    stats_cleared: "History cleared.",
    tip_label: "Tip",
    tips_header: "Debugging tips:",
    themes_header: "Available themes:",
    themes_hint: "Use with:  rubberduck --theme <name>   (or config set theme <name>)",
    history_header: "Recent sessions:",
    history_empty: "No sessions recorded yet — run one to build your history.",
    history_showing: "Showing the {shown} most recent of {total} sessions.",
    doctor_header: "rubberduck doctor — environment check",
    doctor_version: "Version",
    doctor_language: "Language",
    doctor_theme: "Theme",
    doctor_color: "Colour",
    doctor_animations: "Animations",
    doctor_terminal: "Terminal",
    doctor_config: "Config",
    doctor_data: "Data",
    doctor_questions: "{topics} topics ({lang})",
    doctor_tips: "{count} tips ({lang})",
    doctor_enabled: "enabled",
    doctor_disabled: "disabled",
    doctor_interactive: "interactive",
    doctor_plain: "not a terminal",
    doctor_config_ok: "ok",
    doctor_config_missing: "not created yet (using defaults)",
    languages_header: "Available languages:",
    update_available: "🦆 Update available: {current} → {latest}",
    up_to_date: "🦆 rubberduck is up to date (version {current}).",
    no_releases: "No releases found.",
    updated_to: "🦆 Updated to version {version}.",
    already_current: "🦆 Already up to date (version {version}).",
    uninstall_header: "The following will be removed:",
    uninstall_label_binary: "Binary",
    uninstall_label_config: "Config",
    uninstall_label_logs: "Logs",
    uninstall_confirm: "Really remove everything?",
    uninstall_cancelled: "Cancelled — nothing was removed.",
    uninstall_removed: "Removed: {path}",
    uninstall_needs_tty:
        "Uninstall needs an interactive confirmation and is not possible without a \
                          terminal. Use 'uninstall.sh --yes' instead.",
    uninstall_unsafe: "Aborting: '{path}' looks like a home/system directory and will not be \
                       deleted. Please check RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR.",
    uninstall_done: "rubberduck has been removed. Thanks for the quacks! 🦆",
    uninstall_binary_failed: "Config and logs were removed, but the binary at {path} could not be \
                              deleted. Please remove it manually.",
    md_title: "# 🦆 Rubberduck session – {date}",
    md_topic: "**Topic:** {topic}",
    md_started: "**Started:** {started_at}",
    md_questions: "**Questions:** {answered} answered / {asked} asked",
    md_duration: "**Duration:** {total} ({avg} avg per question)",
    md_solved: "**Solved:** {value}",
    md_solved_yes: "✅ yes",
    md_solved_no: "– open",
    md_aha: "> 💡 **Aha after question {after} ({time}):** {note}",
    md_no_note: "(no note)",
    md_no_answer: "_(no answer)_",
};

/// German catalog.
const DE: Catalog = Catalog {
    greeting: "Hi! Thema: {topic}. Erklär mir dein Problem – Schritt für Schritt. \
               (Tippe !aha, sobald der Groschen fällt.)",
    pondering: "Die Ente überlegt sich gute Fragen …",
    answer_prompt: "  Du",
    aborted_session: "Abgebrochen – bis zum nächsten Quaken!",
    aborted_no_topic: "Abgebrochen – kein Thema gewählt.",
    pick_topic_prompt: "Welches Thema möchtest du durchgehen?",
    end_confirm: "Aha – hast du den Bug gefunden?",
    aha_note_prompt: "Was war's? (kurze Notiz, Enter überspringt)",
    aha_closing: "Stark! Erklären hilft – genau dafür bin ich da.",
    celebrate_quiet: "✨ Stark – gefunden!",
    quack_word: "Quak!",
    eureka: "HEUREKA!",
    summary_header: "──────── Zusammenfassung ────────",
    summary_answered: "{answered} / {asked} Fragen beantwortet",
    summary_duration: "Dauer: {total} (Ø {avg} pro Frage)",
    summary_solved: "✅ Bug gefunden",
    summary_open: "– noch offen",
    log_saved: "Logbuch gespeichert: {path}",
    topics_header: "Verfügbare Themen:",
    topics_hint: "Start mit:  rubberduck --topic <name>   (* = Standard)",
    config_settings_header: "Einstellungen ({path})",
    config_exists: "Existiert bereits: {path}",
    config_created: "Angelegt: {path}",
    config_set_done: "Gesetzt: {key} = {value}",
    config_reset_done: "Auf Standard zurückgesetzt: {path}",
    demo_title: "rubberduck — Animations-Demo",
    demo_intro: "Schau genau hin – ich tippe, schwimme, quake und feiere, wenn du gewinnst.",
    demo_section_moods: "Stimmungen",
    demo_section_themes: "Themes",
    demo_section_spinners: "Spinner",
    demo_section_gradients: "Farbverläufe",
    demo_done: "Das war die Tour – frohes Debuggen!",
    mood_idle: "Ruhig",
    mood_thinking: "Nachdenklich",
    mood_listening: "Zuhörend",
    mood_happy: "Glücklich",
    mood_curious: "Neugierig",
    mood_surprised: "Überrascht",
    mood_celebrating: "Feiernd",
    mood_sleeping: "Schläft",
    mood_confused: "Verwirrt",
    mood_proud: "Stolz",
    mood_reading: "Lesend",
    stats_header: "Deine Debugging-Statistik",
    stats_empty: "Noch keine Sessions aufgezeichnet – starte eine, um deine Historie aufzubauen.",
    stats_sessions: "Sessions: {n}",
    stats_solved: "Gelöst: {solved}/{total} ({pct}%)",
    stats_avg_session: "Ø Session: {d}",
    stats_avg_solution: "Ø Zeit bis zur Lösung: {d}",
    stats_by_topic: "Nach Thema",
    stats_cleared: "Historie gelöscht.",
    tip_label: "Tipp",
    tips_header: "Debugging-Tipps:",
    themes_header: "Verfügbare Themes:",
    themes_hint: "Nutze mit:  rubberduck --theme <name>   (oder config set theme <name>)",
    history_header: "Letzte Sessions:",
    history_empty: "Noch keine Sessions aufgezeichnet – starte eine, um deine Historie aufzubauen.",
    history_showing: "Zeige die {shown} neuesten von {total} Sessions.",
    doctor_header: "rubberduck doctor — Umgebungs-Check",
    doctor_version: "Version",
    doctor_language: "Sprache",
    doctor_theme: "Theme",
    doctor_color: "Farbe",
    doctor_animations: "Animationen",
    doctor_terminal: "Terminal",
    doctor_config: "Konfiguration",
    doctor_data: "Daten",
    doctor_questions: "{topics} Themen ({lang})",
    doctor_tips: "{count} Tipps ({lang})",
    doctor_enabled: "aktiv",
    doctor_disabled: "aus",
    doctor_interactive: "interaktiv",
    doctor_plain: "kein Terminal",
    doctor_config_ok: "ok",
    doctor_config_missing: "noch nicht angelegt (Standardwerte)",
    languages_header: "Verfügbare Sprachen:",
    update_available: "🦆 Update verfügbar: {current} → {latest}",
    up_to_date: "🦆 rubberduck ist aktuell (Version {current}).",
    no_releases: "Keine Releases gefunden.",
    updated_to: "🦆 Aktualisiert auf Version {version}.",
    already_current: "🦆 Bereits aktuell (Version {version}).",
    uninstall_header: "Folgendes wird entfernt:",
    uninstall_label_binary: "Binary",
    uninstall_label_config: "Konfiguration",
    uninstall_label_logs: "Logs",
    uninstall_confirm: "Wirklich alles entfernen?",
    uninstall_cancelled: "Abgebrochen – nichts wurde entfernt.",
    uninstall_removed: "Entfernt: {path}",
    uninstall_needs_tty: "Deinstallation braucht eine interaktive Bestätigung und ist ohne \
                          Terminal nicht möglich. Nutze stattdessen 'uninstall.sh --yes'.",
    uninstall_unsafe: "Abbruch: '{path}' sieht nach einem Home-/System-Verzeichnis aus und wird \
                       nicht gelöscht. Bitte RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR prüfen.",
    uninstall_done: "rubberduck wurde entfernt. Danke fürs Quaken! 🦆",
    uninstall_binary_failed:
        "Konfiguration und Logs wurden entfernt, aber das Binary unter {path} \
                              konnte nicht gelöscht werden. Bitte manuell entfernen.",
    md_title: "# 🦆 Rubberduck-Session – {date}",
    md_topic: "**Thema:** {topic}",
    md_started: "**Gestartet:** {started_at}",
    md_questions: "**Fragen:** {answered} beantwortet / {asked} gestellt",
    md_duration: "**Dauer:** {total} (Ø {avg} pro Frage)",
    md_solved: "**Gelöst:** {value}",
    md_solved_yes: "✅ ja",
    md_solved_no: "– offen",
    md_aha: "> 💡 **Aha nach Frage {after} ({time}):** {note}",
    md_no_note: "(keine Notiz)",
    md_no_answer: "_(keine Antwort)_",
};

/// French catalog.
const FR: Catalog = Catalog {
    greeting: "Salut ! Sujet : {topic}. Explique-moi ton problème – étape par étape. \
               (Tape !aha dès que le déclic se produit.)",
    pondering: "Le canard réfléchit à de bonnes questions …",
    answer_prompt: "  Toi",
    aborted_session: "Annulé – à la prochaine, coin coin !",
    aborted_no_topic: "Annulé – aucun sujet choisi.",
    pick_topic_prompt: "Quel sujet veux-tu parcourir ?",
    end_confirm: "Aha – tu as trouvé le bug ?",
    aha_note_prompt: "C'était quoi ? (note courte, Entrée pour passer)",
    aha_closing: "Bien joué ! Expliquer aide – c'est exactement pour ça que je suis là.",
    celebrate_quiet: "✨ Bien joué – trouvé !",
    quack_word: "Coin !",
    eureka: "EURÊKA !",
    summary_header: "──────── Résumé ────────",
    summary_answered: "{answered} / {asked} questions répondues",
    summary_duration: "Durée : {total} ({avg} en moyenne par question)",
    summary_solved: "✅ Bug trouvé",
    summary_open: "– encore ouvert",
    log_saved: "Journal enregistré : {path}",
    topics_header: "Sujets disponibles :",
    topics_hint: "Commence avec :  rubberduck --topic <nom>   (* = défaut)",
    config_settings_header: "Paramètres ({path})",
    config_exists: "Existe déjà : {path}",
    config_created: "Créé : {path}",
    config_set_done: "Défini : {key} = {value}",
    config_reset_done: "Réinitialisé aux valeurs par défaut : {path}",
    demo_title: "rubberduck — démo des animations",
    demo_intro: "Regarde bien – je tape, je nage, je cancane et je fête ta victoire.",
    demo_section_moods: "Humeurs",
    demo_section_themes: "Thèmes",
    demo_section_spinners: "Indicateurs",
    demo_section_gradients: "Dégradés",
    demo_done: "Fin de la visite – bon débogage !",
    mood_idle: "Au repos",
    mood_thinking: "Pensif",
    mood_listening: "À l'écoute",
    mood_happy: "Heureux",
    mood_curious: "Curieux",
    mood_surprised: "Surpris",
    mood_celebrating: "En fête",
    mood_sleeping: "Endormi",
    mood_confused: "Perplexe",
    mood_proud: "Fier",
    mood_reading: "En lecture",
    stats_header: "Tes statistiques de débogage",
    stats_empty: "Aucune session enregistrée – lances-en une pour bâtir ton historique.",
    stats_sessions: "Sessions : {n}",
    stats_solved: "Résolus : {solved}/{total} ({pct} %)",
    stats_avg_session: "Session moyenne : {d}",
    stats_avg_solution: "Temps moyen jusqu'à la solution : {d}",
    stats_by_topic: "Par sujet",
    stats_cleared: "Historique effacé.",
    tip_label: "Astuce",
    tips_header: "Astuces de débogage :",
    themes_header: "Thèmes disponibles :",
    themes_hint: "À utiliser avec :  rubberduck --theme <nom>   (ou config set theme <nom>)",
    history_header: "Sessions récentes :",
    history_empty: "Aucune session enregistrée – lances-en une pour bâtir ton historique.",
    history_showing: "Affichage des {shown} sessions les plus récentes sur {total}.",
    doctor_header: "rubberduck doctor — vérification de l'environnement",
    doctor_version: "Version",
    doctor_language: "Langue",
    doctor_theme: "Thème",
    doctor_color: "Couleur",
    doctor_animations: "Animations",
    doctor_terminal: "Terminal",
    doctor_config: "Configuration",
    doctor_data: "Données",
    doctor_questions: "{topics} sujets ({lang})",
    doctor_tips: "{count} astuces ({lang})",
    doctor_enabled: "activé",
    doctor_disabled: "désactivé",
    doctor_interactive: "interactif",
    doctor_plain: "pas un terminal",
    doctor_config_ok: "ok",
    doctor_config_missing: "pas encore créé (valeurs par défaut)",
    languages_header: "Langues disponibles :",
    update_available: "🦆 Mise à jour disponible : {current} → {latest}",
    up_to_date: "🦆 rubberduck est à jour (version {current}).",
    no_releases: "Aucune version trouvée.",
    updated_to: "🦆 Mis à jour vers la version {version}.",
    already_current: "🦆 Déjà à jour (version {version}).",
    uninstall_header: "Les éléments suivants seront supprimés :",
    uninstall_label_binary: "Binaire",
    uninstall_label_config: "Configuration",
    uninstall_label_logs: "Journaux",
    uninstall_confirm: "Vraiment tout supprimer ?",
    uninstall_cancelled: "Annulé – rien n'a été supprimé.",
    uninstall_removed: "Supprimé : {path}",
    uninstall_needs_tty: "La désinstallation nécessite une confirmation interactive et n'est pas \
                          possible sans terminal. Utilise plutôt 'uninstall.sh --yes'.",
    uninstall_unsafe: "Abandon : '{path}' ressemble à un répertoire personnel/système et ne sera \
                       pas supprimé. Vérifie RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR.",
    uninstall_done: "rubberduck a été supprimé. Merci pour les coin coin ! 🦆",
    uninstall_binary_failed: "La configuration et les journaux ont été supprimés, mais le binaire \
                              à {path} n'a pas pu être supprimé. Supprime-le manuellement.",
    md_title: "# 🦆 Session Rubberduck – {date}",
    md_topic: "**Sujet :** {topic}",
    md_started: "**Démarré :** {started_at}",
    md_questions: "**Questions :** {answered} répondues / {asked} posées",
    md_duration: "**Durée :** {total} ({avg} en moyenne par question)",
    md_solved: "**Résolu :** {value}",
    md_solved_yes: "✅ oui",
    md_solved_no: "– ouvert",
    md_aha: "> 💡 **Aha après la question {after} ({time}) :** {note}",
    md_no_note: "(aucune note)",
    md_no_answer: "_(aucune réponse)_",
};

/// Spanish catalog.
const ES: Catalog = Catalog {
    greeting: "¡Hola! Tema: {topic}. Explícame tu problema paso a paso. \
               (Escribe !aha en cuanto se te encienda la bombilla.)",
    pondering: "El pato está pensando buenas preguntas …",
    answer_prompt: "  Tú",
    aborted_session: "Cancelado: ¡hasta el próximo cuac!",
    aborted_no_topic: "Cancelado: no se eligió ningún tema.",
    pick_topic_prompt: "¿Qué tema quieres repasar?",
    end_confirm: "Aha, ¿encontraste el bug?",
    aha_note_prompt: "¿Qué era? (nota breve, Enter para omitir)",
    aha_closing: "¡Bien! Explicar ayuda: justo para eso estoy aquí.",
    celebrate_quiet: "✨ ¡Bien, lo encontraste!",
    quack_word: "¡Cuac!",
    eureka: "¡EUREKA!",
    summary_header: "──────── Resumen ────────",
    summary_answered: "{answered} / {asked} preguntas respondidas",
    summary_duration: "Duración: {total} ({avg} de media por pregunta)",
    summary_solved: "✅ Bug encontrado",
    summary_open: "– aún abierto",
    log_saved: "Registro guardado: {path}",
    topics_header: "Temas disponibles:",
    topics_hint: "Empieza con:  rubberduck --topic <nombre>   (* = predeterminado)",
    config_settings_header: "Ajustes ({path})",
    config_exists: "Ya existe: {path}",
    config_created: "Creado: {path}",
    config_set_done: "Definido: {key} = {value}",
    config_reset_done: "Restablecido a los valores predeterminados: {path}",
    demo_title: "rubberduck — demo de animaciones",
    demo_intro: "Mira bien: escribo, nado, hago cuac y celebro cuando ganas.",
    demo_section_moods: "Estados de ánimo",
    demo_section_themes: "Temas de color",
    demo_section_spinners: "Indicadores",
    demo_section_gradients: "Degradados",
    demo_done: "Fin del recorrido: ¡feliz depuración!",
    mood_idle: "Tranquilo",
    mood_thinking: "Pensativo",
    mood_listening: "Atento",
    mood_happy: "Feliz",
    mood_curious: "Curioso",
    mood_surprised: "Sorprendido",
    mood_celebrating: "Celebrando",
    mood_sleeping: "Durmiendo",
    mood_confused: "Confundido",
    mood_proud: "Orgulloso",
    mood_reading: "Leyendo",
    stats_header: "Tus estadísticas de depuración",
    stats_empty: "Aún no hay sesiones registradas: ejecuta una para construir tu historial.",
    stats_sessions: "Sesiones: {n}",
    stats_solved: "Resueltos: {solved}/{total} ({pct} %)",
    stats_avg_session: "Sesión media: {d}",
    stats_avg_solution: "Tiempo medio hasta la solución: {d}",
    stats_by_topic: "Por tema",
    stats_cleared: "Historial borrado.",
    tip_label: "Consejo",
    tips_header: "Consejos de depuración:",
    themes_header: "Temas disponibles:",
    themes_hint: "Úsalo con:  rubberduck --theme <nombre>   (o config set theme <nombre>)",
    history_header: "Sesiones recientes:",
    history_empty: "Aún no hay sesiones registradas: ejecuta una para construir tu historial.",
    history_showing: "Mostrando las {shown} sesiones más recientes de {total}.",
    doctor_header: "rubberduck doctor — comprobación del entorno",
    doctor_version: "Versión",
    doctor_language: "Idioma",
    doctor_theme: "Tema",
    doctor_color: "Color",
    doctor_animations: "Animaciones",
    doctor_terminal: "Terminal",
    doctor_config: "Configuración",
    doctor_data: "Datos",
    doctor_questions: "{topics} temas ({lang})",
    doctor_tips: "{count} consejos ({lang})",
    doctor_enabled: "activado",
    doctor_disabled: "desactivado",
    doctor_interactive: "interactivo",
    doctor_plain: "no es un terminal",
    doctor_config_ok: "ok",
    doctor_config_missing: "aún no creado (valores predeterminados)",
    languages_header: "Idiomas disponibles:",
    update_available: "🦆 Actualización disponible: {current} → {latest}",
    up_to_date: "🦆 rubberduck está actualizado (versión {current}).",
    no_releases: "No se encontraron versiones.",
    updated_to: "🦆 Actualizado a la versión {version}.",
    already_current: "🦆 Ya está actualizado (versión {version}).",
    uninstall_header: "Se eliminará lo siguiente:",
    uninstall_label_binary: "Binario",
    uninstall_label_config: "Configuración",
    uninstall_label_logs: "Registros",
    uninstall_confirm: "¿Eliminar todo de verdad?",
    uninstall_cancelled: "Cancelado: no se eliminó nada.",
    uninstall_removed: "Eliminado: {path}",
    uninstall_needs_tty: "La desinstalación necesita una confirmación interactiva y no es posible \
                          sin terminal. Usa 'uninstall.sh --yes' en su lugar.",
    uninstall_unsafe: "Cancelado: '{path}' parece un directorio personal o del sistema y no se \
                       eliminará. Revisa RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR.",
    uninstall_done: "rubberduck se ha eliminado. ¡Gracias por los cuacs! 🦆",
    uninstall_binary_failed:
        "Se eliminaron la configuración y los registros, pero no se pudo borrar \
                              el binario en {path}. Elimínalo manualmente.",
    md_title: "# 🦆 Sesión de Rubberduck – {date}",
    md_topic: "**Tema:** {topic}",
    md_started: "**Inicio:** {started_at}",
    md_questions: "**Preguntas:** {answered} respondidas / {asked} formuladas",
    md_duration: "**Duración:** {total} ({avg} de media por pregunta)",
    md_solved: "**Resuelto:** {value}",
    md_solved_yes: "✅ sí",
    md_solved_no: "– abierto",
    md_aha: "> 💡 **Aha tras la pregunta {after} ({time}):** {note}",
    md_no_note: "(sin nota)",
    md_no_answer: "_(sin respuesta)_",
};

/// A translator: turns message keys into localized, user-facing strings.
///
/// `Tr` is `Copy`, so it can be threaded cheaply through the UI, the controller
/// and the session writer. Every method reads from the catalog of [`Tr::lang`];
/// none of them branch on the language directly (see the module docs).
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

    /// The string catalog backing this translator.
    fn cat(self) -> &'static Catalog {
        match self.lang {
            Lang::English => &EN,
            Lang::German => &DE,
            Lang::French => &FR,
            Lang::Spanish => &ES,
        }
    }

    // ----- session greeting & dialog --------------------------------------

    /// Opening line the duck greets the user with.
    #[must_use]
    pub fn greeting(self, topic: &str) -> String {
        fill(self.cat().greeting, &[("topic", topic)])
    }

    /// Label of the "thinking" spinner shown before the questions.
    #[must_use]
    pub fn pondering(self) -> &'static str {
        self.cat().pondering
    }

    /// Prompt prefix for the user's answer.
    #[must_use]
    pub fn answer_prompt(self) -> &'static str {
        self.cat().answer_prompt
    }

    /// Shown when the session is cancelled mid-way.
    #[must_use]
    pub fn aborted_session(self) -> &'static str {
        self.cat().aborted_session
    }

    /// Shown when the topic picker is cancelled.
    #[must_use]
    pub fn aborted_no_topic(self) -> &'static str {
        self.cat().aborted_no_topic
    }

    /// Prompt of the interactive topic picker.
    #[must_use]
    pub fn pick_topic_prompt(self) -> &'static str {
        self.cat().pick_topic_prompt
    }

    /// End-of-session confirmation: was the bug found?
    #[must_use]
    pub fn end_confirm(self) -> &'static str {
        self.cat().end_confirm
    }

    /// Prompt for the optional aha note.
    #[must_use]
    pub fn aha_note_prompt(self) -> &'static str {
        self.cat().aha_note_prompt
    }

    /// Closing line after a celebrated aha moment.
    #[must_use]
    pub fn aha_closing(self) -> &'static str {
        self.cat().aha_closing
    }

    /// Quiet-mode replacement for the celebration animation.
    #[must_use]
    pub fn celebrate_quiet(self) -> &'static str {
        self.cat().celebrate_quiet
    }

    /// The word the duck quacks in its animation.
    #[must_use]
    pub fn quack_word(self) -> &'static str {
        self.cat().quack_word
    }

    /// The celebration banner word.
    #[must_use]
    pub fn eureka(self) -> &'static str {
        self.cat().eureka
    }

    // ----- summary --------------------------------------------------------

    /// Heading of the end-of-session summary block.
    #[must_use]
    pub fn summary_header(self) -> &'static str {
        self.cat().summary_header
    }

    /// Summary line: questions answered.
    #[must_use]
    pub fn summary_answered(self, answered: usize, asked: usize) -> String {
        fill(
            self.cat().summary_answered,
            &[
                ("answered", &answered.to_string()),
                ("asked", &asked.to_string()),
            ],
        )
    }

    /// Summary line: total and average duration.
    #[must_use]
    pub fn summary_duration(self, total: &str, avg: &str) -> String {
        fill(
            self.cat().summary_duration,
            &[("total", total), ("avg", avg)],
        )
    }

    /// Summary line: bug found.
    #[must_use]
    pub fn summary_solved(self) -> &'static str {
        self.cat().summary_solved
    }

    /// Summary line: not solved yet.
    #[must_use]
    pub fn summary_open(self) -> &'static str {
        self.cat().summary_open
    }

    /// Confirmation that the log file was written.
    #[must_use]
    pub fn log_saved(self, path: &str) -> String {
        fill(self.cat().log_saved, &[("path", path)])
    }

    // ----- topics & config commands ---------------------------------------

    /// Heading of `rubberduck topics`.
    #[must_use]
    pub fn topics_header(self) -> &'static str {
        self.cat().topics_header
    }

    /// Hint printed after the topic list.
    #[must_use]
    pub fn topics_hint(self) -> &'static str {
        self.cat().topics_hint
    }

    /// `config show` heading with the file path.
    #[must_use]
    pub fn config_settings_header(self, path: &str) -> String {
        fill(self.cat().config_settings_header, &[("path", path)])
    }

    /// `config init` when the file already exists.
    #[must_use]
    pub fn config_exists(self, path: &str) -> String {
        fill(self.cat().config_exists, &[("path", path)])
    }

    /// `config init` after creating the file.
    #[must_use]
    pub fn config_created(self, path: &str) -> String {
        fill(self.cat().config_created, &[("path", path)])
    }

    /// `config set` confirmation.
    #[must_use]
    pub fn config_set_done(self, key: &str, value: &str) -> String {
        fill(
            self.cat().config_set_done,
            &[("key", key), ("value", value)],
        )
    }

    /// `config reset` confirmation.
    #[must_use]
    pub fn config_reset_done(self, path: &str) -> String {
        fill(self.cat().config_reset_done, &[("path", path)])
    }

    // ----- demo -----------------------------------------------------------

    /// Title banner of `rubberduck demo`.
    #[must_use]
    pub fn demo_title(self) -> &'static str {
        self.cat().demo_title
    }

    /// Typewriter intro line of the demo.
    #[must_use]
    pub fn demo_intro(self) -> &'static str {
        self.cat().demo_intro
    }

    /// Demo section heading: moods.
    #[must_use]
    pub fn demo_section_moods(self) -> &'static str {
        self.cat().demo_section_moods
    }

    /// Demo section heading: themes.
    #[must_use]
    pub fn demo_section_themes(self) -> &'static str {
        self.cat().demo_section_themes
    }

    /// Demo section heading: spinner styles.
    #[must_use]
    pub fn demo_section_spinners(self) -> &'static str {
        self.cat().demo_section_spinners
    }

    /// Demo section heading: colour gradients.
    #[must_use]
    pub fn demo_section_gradients(self) -> &'static str {
        self.cat().demo_section_gradients
    }

    /// Closing line of the demo.
    #[must_use]
    pub fn demo_done(self) -> &'static str {
        self.cat().demo_done
    }

    /// Localized label for a duck [`Mood`] (used by the demo showcase).
    #[must_use]
    pub fn mood_label(self, mood: Mood) -> &'static str {
        let c = self.cat();
        match mood {
            Mood::Idle => c.mood_idle,
            Mood::Thinking => c.mood_thinking,
            Mood::Listening => c.mood_listening,
            Mood::Happy => c.mood_happy,
            Mood::Curious => c.mood_curious,
            Mood::Surprised => c.mood_surprised,
            Mood::Celebrating => c.mood_celebrating,
            Mood::Sleeping => c.mood_sleeping,
            Mood::Confused => c.mood_confused,
            Mood::Proud => c.mood_proud,
            Mood::Reading => c.mood_reading,
        }
    }

    // ----- stats ----------------------------------------------------------

    /// Heading of `rubberduck stats`.
    #[must_use]
    pub fn stats_header(self) -> &'static str {
        self.cat().stats_header
    }

    /// `stats` with no history yet.
    #[must_use]
    pub fn stats_empty(self) -> &'static str {
        self.cat().stats_empty
    }

    /// Stats line: number of sessions.
    #[must_use]
    pub fn stats_sessions(self, n: usize) -> String {
        fill(self.cat().stats_sessions, &[("n", &n.to_string())])
    }

    /// Stats line: solved count and rate.
    #[must_use]
    pub fn stats_solved(self, solved: usize, total: usize, pct: u32) -> String {
        fill(
            self.cat().stats_solved,
            &[
                ("solved", &solved.to_string()),
                ("total", &total.to_string()),
                ("pct", &pct.to_string()),
            ],
        )
    }

    /// Stats line: average session length.
    #[must_use]
    pub fn stats_avg_session(self, d: &str) -> String {
        fill(self.cat().stats_avg_session, &[("d", d)])
    }

    /// Stats line: average time to solution.
    #[must_use]
    pub fn stats_avg_solution(self, d: &str) -> String {
        fill(self.cat().stats_avg_solution, &[("d", d)])
    }

    /// Stats sub-heading: per-topic breakdown.
    #[must_use]
    pub fn stats_by_topic(self) -> &'static str {
        self.cat().stats_by_topic
    }

    /// Confirmation that the history was cleared.
    #[must_use]
    pub fn stats_cleared(self) -> &'static str {
        self.cat().stats_cleared
    }

    // ----- tips -----------------------------------------------------------

    /// Inline label prefix for a single tip (e.g. `Tip`).
    #[must_use]
    pub fn tip_label(self) -> &'static str {
        self.cat().tip_label
    }

    /// Heading of `rubberduck tips`.
    #[must_use]
    pub fn tips_header(self) -> &'static str {
        self.cat().tips_header
    }

    // ----- themes ---------------------------------------------------------

    /// Heading of `rubberduck themes`.
    #[must_use]
    pub fn themes_header(self) -> &'static str {
        self.cat().themes_header
    }

    /// Hint printed after the theme previews.
    #[must_use]
    pub fn themes_hint(self) -> &'static str {
        self.cat().themes_hint
    }

    // ----- history --------------------------------------------------------

    /// Heading of `rubberduck history`.
    #[must_use]
    pub fn history_header(self) -> &'static str {
        self.cat().history_header
    }

    /// `history` with no recorded sessions.
    #[must_use]
    pub fn history_empty(self) -> &'static str {
        self.cat().history_empty
    }

    /// Footer noting how many of the total sessions are shown.
    #[must_use]
    pub fn history_showing(self, shown: usize, total: usize) -> String {
        fill(
            self.cat().history_showing,
            &[("shown", &shown.to_string()), ("total", &total.to_string())],
        )
    }

    // ----- doctor ---------------------------------------------------------

    /// Heading of `rubberduck doctor`.
    #[must_use]
    pub fn doctor_header(self) -> &'static str {
        self.cat().doctor_header
    }

    /// Doctor row label: version.
    #[must_use]
    pub fn doctor_version(self) -> &'static str {
        self.cat().doctor_version
    }

    /// Doctor row label: language.
    #[must_use]
    pub fn doctor_language(self) -> &'static str {
        self.cat().doctor_language
    }

    /// Doctor row label: theme.
    #[must_use]
    pub fn doctor_theme(self) -> &'static str {
        self.cat().doctor_theme
    }

    /// Doctor row label: colour.
    #[must_use]
    pub fn doctor_color(self) -> &'static str {
        self.cat().doctor_color
    }

    /// Doctor row label: animations.
    #[must_use]
    pub fn doctor_animations(self) -> &'static str {
        self.cat().doctor_animations
    }

    /// Doctor row label: terminal.
    #[must_use]
    pub fn doctor_terminal(self) -> &'static str {
        self.cat().doctor_terminal
    }

    /// Doctor row label: config file.
    #[must_use]
    pub fn doctor_config(self) -> &'static str {
        self.cat().doctor_config
    }

    /// Doctor row label: data directory.
    #[must_use]
    pub fn doctor_data(self) -> &'static str {
        self.cat().doctor_data
    }

    /// Doctor value: number of question topics for a language.
    #[must_use]
    pub fn doctor_questions(self, topics: usize, lang: &str) -> String {
        fill(
            self.cat().doctor_questions,
            &[("topics", &topics.to_string()), ("lang", lang)],
        )
    }

    /// Doctor value: number of tips for a language.
    #[must_use]
    pub fn doctor_tips(self, count: usize, lang: &str) -> String {
        fill(
            self.cat().doctor_tips,
            &[("count", &count.to_string()), ("lang", lang)],
        )
    }

    /// Doctor status: a feature is enabled.
    #[must_use]
    pub fn doctor_enabled(self) -> &'static str {
        self.cat().doctor_enabled
    }

    /// Doctor status: a feature is disabled.
    #[must_use]
    pub fn doctor_disabled(self) -> &'static str {
        self.cat().doctor_disabled
    }

    /// Doctor status: stdout/stdin form an interactive terminal.
    #[must_use]
    pub fn doctor_interactive(self) -> &'static str {
        self.cat().doctor_interactive
    }

    /// Doctor status: output is not a terminal.
    #[must_use]
    pub fn doctor_plain(self) -> &'static str {
        self.cat().doctor_plain
    }

    /// Doctor status: the config file exists and parsed.
    #[must_use]
    pub fn doctor_config_ok(self) -> &'static str {
        self.cat().doctor_config_ok
    }

    /// Doctor status: no config file yet (defaults in use).
    #[must_use]
    pub fn doctor_config_missing(self) -> &'static str {
        self.cat().doctor_config_missing
    }

    // ----- languages ------------------------------------------------------

    /// Heading of `rubberduck languages`.
    #[must_use]
    pub fn languages_header(self) -> &'static str {
        self.cat().languages_header
    }

    // ----- self update / uninstall ----------------------------------------

    /// `self update --check`: an update is available.
    #[must_use]
    pub fn update_available(self, current: &str, latest: &str) -> String {
        fill(
            self.cat().update_available,
            &[("current", current), ("latest", latest)],
        )
    }

    /// `self update --check`: already up to date.
    #[must_use]
    pub fn up_to_date(self, current: &str) -> String {
        fill(self.cat().up_to_date, &[("current", current)])
    }

    /// `self update`: no releases found.
    #[must_use]
    pub fn no_releases(self) -> &'static str {
        self.cat().no_releases
    }

    /// `self update`: binary was updated.
    #[must_use]
    pub fn updated_to(self, version: &str) -> String {
        fill(self.cat().updated_to, &[("version", version)])
    }

    /// `self update`: already on the latest version.
    #[must_use]
    pub fn already_current(self, version: &str) -> String {
        fill(self.cat().already_current, &[("version", version)])
    }

    /// Uninstall: header listing what will be removed.
    #[must_use]
    pub fn uninstall_header(self) -> &'static str {
        self.cat().uninstall_header
    }

    /// Uninstall: label for the binary.
    #[must_use]
    pub fn uninstall_label_binary(self) -> &'static str {
        self.cat().uninstall_label_binary
    }

    /// Uninstall: label for the configuration directory.
    #[must_use]
    pub fn uninstall_label_config(self) -> &'static str {
        self.cat().uninstall_label_config
    }

    /// Uninstall: label for the logs directory.
    #[must_use]
    pub fn uninstall_label_logs(self) -> &'static str {
        self.cat().uninstall_label_logs
    }

    /// Uninstall: confirmation prompt.
    #[must_use]
    pub fn uninstall_confirm(self) -> &'static str {
        self.cat().uninstall_confirm
    }

    /// Uninstall: cancelled by the user.
    #[must_use]
    pub fn uninstall_cancelled(self) -> &'static str {
        self.cat().uninstall_cancelled
    }

    /// Uninstall: one path was removed.
    #[must_use]
    pub fn uninstall_removed(self, path: &str) -> String {
        fill(self.cat().uninstall_removed, &[("path", path)])
    }

    /// Uninstall: needs an interactive terminal.
    #[must_use]
    pub fn uninstall_needs_tty(self) -> &'static str {
        self.cat().uninstall_needs_tty
    }

    /// Uninstall: refusing to delete a home/system directory.
    #[must_use]
    pub fn uninstall_unsafe(self, path: &str) -> String {
        fill(self.cat().uninstall_unsafe, &[("path", path)])
    }

    /// Uninstall: success message.
    #[must_use]
    pub fn uninstall_done(self) -> &'static str {
        self.cat().uninstall_done
    }

    /// Uninstall: dirs removed but the binary could not be deleted.
    #[must_use]
    pub fn uninstall_binary_failed(self, path: &str) -> String {
        fill(self.cat().uninstall_binary_failed, &[("path", path)])
    }

    // ----- markdown log ---------------------------------------------------

    /// Log: top-level title.
    #[must_use]
    pub fn md_title(self, date: &str) -> String {
        fill(self.cat().md_title, &[("date", date)])
    }

    /// Log: topic line (without the leading `- `).
    #[must_use]
    pub fn md_topic(self, topic: &str) -> String {
        fill(self.cat().md_topic, &[("topic", topic)])
    }

    /// Log: started-at line.
    #[must_use]
    pub fn md_started(self, started_at: &str) -> String {
        fill(self.cat().md_started, &[("started_at", started_at)])
    }

    /// Log: questions answered/asked line.
    #[must_use]
    pub fn md_questions(self, answered: usize, asked: usize) -> String {
        fill(
            self.cat().md_questions,
            &[
                ("answered", &answered.to_string()),
                ("asked", &asked.to_string()),
            ],
        )
    }

    /// Log: duration line.
    #[must_use]
    pub fn md_duration(self, total: &str, avg: &str) -> String {
        fill(self.cat().md_duration, &[("total", total), ("avg", avg)])
    }

    /// Log: solved line (`✅ yes` / `– open`).
    #[must_use]
    pub fn md_solved(self, solved: bool) -> String {
        let c = self.cat();
        let value = if solved {
            c.md_solved_yes
        } else {
            c.md_solved_no
        };
        fill(c.md_solved, &[("value", value)])
    }

    /// Log: the aha-moment blockquote.
    #[must_use]
    pub fn md_aha(self, after: usize, time: &str, note: &str) -> String {
        fill(
            self.cat().md_aha,
            &[
                ("after", &after.to_string()),
                ("time", time),
                ("note", note),
            ],
        )
    }

    /// Log: placeholder for a missing aha note.
    #[must_use]
    pub fn md_no_note(self) -> &'static str {
        self.cat().md_no_note
    }

    /// Log: placeholder for an unanswered question.
    #[must_use]
    pub fn md_no_answer(self) -> &'static str {
        self.cat().md_no_answer
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
        assert_eq!(Lang::from_code("french"), Some(Lang::French));
        assert_eq!(Lang::from_code("français"), Some(Lang::French));
        assert_eq!(Lang::from_code("spanish"), Some(Lang::Spanish));
        assert_eq!(Lang::from_code("español"), Some(Lang::Spanish));
        assert_eq!(Lang::from_code("xx"), None);
    }

    #[test]
    fn code_list_lists_every_language() {
        assert_eq!(Lang::code_list(), "en, de, fr, es");
    }

    #[test]
    fn languages_differ_and_interpolate() {
        let en = Tr::new(Lang::English);
        let de = Tr::new(Lang::German);
        let fr = Tr::new(Lang::French);
        let es = Tr::new(Lang::Spanish);
        assert!(en.greeting("logic").contains("Topic: logic"));
        assert!(de.greeting("logic").contains("Thema: logic"));
        assert!(fr.greeting("logic").contains("Sujet : logic"));
        assert!(es.greeting("logic").contains("Tema: logic"));
        // No two languages share the "solved" summary.
        assert_ne!(en.summary_solved(), de.summary_solved());
        assert_ne!(en.summary_solved(), fr.summary_solved());
        assert_ne!(de.summary_solved(), fr.summary_solved());
        assert!(en.md_questions(1, 2).contains("1 answered / 2 asked"));
    }

    #[test]
    fn serde_uses_short_codes() {
        let yaml = serde_yaml::to_string(&Lang::French).unwrap();
        assert!(yaml.contains("fr"));
        let back: Lang = serde_yaml::from_str("en").unwrap();
        assert_eq!(back, Lang::English);
    }

    #[test]
    fn mood_labels_localize() {
        assert_eq!(Tr::new(Lang::English).mood_label(Mood::Happy), "Happy");
        assert_eq!(Tr::new(Lang::German).mood_label(Mood::Happy), "Glücklich");
        assert_eq!(Tr::new(Lang::French).mood_label(Mood::Happy), "Heureux");
        assert_eq!(Tr::new(Lang::Spanish).mood_label(Mood::Happy), "Feliz");
        // The moods added later are localized too (no fallback to a default).
        assert_eq!(
            Tr::new(Lang::English).mood_label(Mood::Confused),
            "Confused"
        );
        assert_eq!(Tr::new(Lang::German).mood_label(Mood::Proud), "Stolz");
    }

    /// Every templated message must interpolate for every language: no result is
    /// empty and none still contains an unfilled `{placeholder}`. This guards a
    /// new language against a typo'd or missing marker.
    #[test]
    fn placeholders_are_filled_for_every_language() {
        for lang in Lang::ALL {
            let tr = Tr::new(lang);
            let samples = [
                tr.greeting("topicX"),
                tr.summary_answered(1, 2),
                tr.summary_duration("1m", "2s"),
                tr.log_saved("/p"),
                tr.config_settings_header("/p"),
                tr.config_set_done("k", "v"),
                tr.config_reset_done("/p"),
                tr.stats_sessions(3),
                tr.stats_solved(1, 2, 50),
                tr.stats_avg_session("1m"),
                tr.history_showing(5, 12),
                tr.doctor_questions(8, "en"),
                tr.doctor_tips(12, "en"),
                tr.update_available("1", "2"),
                tr.up_to_date("1"),
                tr.uninstall_unsafe("/home"),
                tr.uninstall_binary_failed("/bin"),
                tr.md_title("today"),
                tr.md_aha(3, "1m", "note"),
                tr.md_solved(true),
                tr.md_solved(false),
            ];
            for s in samples {
                assert!(!s.is_empty(), "empty message in {lang}");
                assert!(!s.contains('{'), "unfilled placeholder in {lang}: {s}");
            }
            assert!(
                tr.greeting("topicX").contains("topicX"),
                "lost arg in {lang}"
            );
        }
    }

    #[test]
    fn fill_replaces_named_placeholders() {
        assert_eq!(fill("a {x} b {y}", &[("x", "1"), ("y", "2")]), "a 1 b 2");
        // A placeholder with no matching arg is left untouched (caught by the
        // completeness test above for real messages).
        assert_eq!(fill("{missing}", &[]), "{missing}");
    }
}
