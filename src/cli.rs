//! CLI-Definition mit `clap` (derive): Flags, Themen-Liste und Shell-Completions.

use crate::config::{ColorPref, Speed};
use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Top-Level-Argumente.
///
/// Ohne Unterbefehl startet eine Debugging-Session; die Flags steuern Thema,
/// Protokoll, Ente und Animationen.
#[derive(Parser, Debug)]
#[command(
    name = "rubberduck",
    version,
    about = "Ein offline Rubber-Duck-Debugging-Begleiter fürs Terminal.",
    long_about = "rubberduck stellt dir strukturierte Debugging-Fragen, bis du den \
                  Bug selbst findest – komplett offline, mit animierter ASCII-Ente.",
    after_help = "Beispiele:\n  \
        rubberduck                     Session starten (Themen-Auswahl, falls kein --topic)\n  \
        rubberduck --topic logic       direkt mit dem Logik-Fragenset\n  \
        rubberduck --log               Session als Markdown speichern\n  \
        rubberduck --no-anim --quiet   ohne Animation/Ente (z. B. für Logs)\n  \
        rubberduck --theme midnight    anderes Farbschema\n  \
        rubberduck topics              verfügbare Themen anzeigen\n  \
        rubberduck completions zsh     Shell-Completions ausgeben\n  \
        rubberduck self update         auf neueste Version aktualisieren\n\n\
        Tipp: Tippe während der Session !aha, sobald du den Bug gefunden hast."
)]
pub struct Cli {
    /// Themen-Fragenset; verfügbare Namen via `rubberduck topics`.
    #[arg(long, value_name = "TOPIC")]
    pub topic: Option<String>,

    /// Sitzung als Markdown-Logbuch unter ~/.rubberduck speichern.
    #[arg(long)]
    pub log: bool,

    // Darstellungs-Flags sind `global`, damit sie auch nach Unterbefehlen
    // funktionieren (z. B. `rubberduck topics --color never`).
    /// Ohne ASCII-Ente ausgeben, nur knapper Text.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Animationen abschalten (statische Ente statt Tippeffekt & Co.).
    #[arg(long = "no-anim", global = true)]
    pub no_anim: bool,

    /// Animationsgeschwindigkeit (slow, normal, fast).
    #[arg(long, value_enum, global = true)]
    pub speed: Option<Speed>,

    /// Farbmodus (auto, always, never).
    #[arg(long, value_enum, global = true)]
    pub color: Option<ColorPref>,

    /// Farbschema (classic, midnight, mono).
    #[arg(
        long,
        global = true,
        value_parser = clap::builder::PossibleValuesParser::new(crate::ui::theme::Theme::NAMES.iter().copied())
    )]
    pub theme: Option<String>,

    /// Unterbefehl (statt einer Session).
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Unterbefehle der obersten Ebene.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Verfügbare Themen mit Beschreibung auflisten.
    Topics,

    /// Shell-Completions ausgeben (bash, zsh, fish, powershell, elvish).
    Completions {
        /// Ziel-Shell.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Persistente Einstellungen verwalten (config.yaml).
    Config {
        /// Auszuführende Konfigurations-Aktion.
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Selbstverwaltung: Update und Deinstallation.
    #[command(name = "self")]
    SelfCmd {
        /// Auszuführende Selbstverwaltungs-Aktion.
        #[command(subcommand)]
        action: SelfAction,
    },
}

/// Aktionen unterhalb von `rubberduck config`.
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Eine Standard-`config.yaml` anlegen (legt nichts an, wenn sie existiert).
    Init,
    /// Effektive Einstellungen und den Dateipfad anzeigen.
    Show,
    /// Nur den Pfad der `config.yaml` ausgeben.
    Path,
}

/// Aktionen unterhalb von `rubberduck self`.
#[derive(Subcommand, Debug)]
pub enum SelfAction {
    /// Auf die neueste Version aktualisieren.
    Update {
        /// Nur prüfen, ob ein Update verfügbar ist (nichts installieren).
        #[arg(long)]
        check: bool,
    },
    /// rubberduck samt Konfiguration und Logs entfernen.
    Uninstall,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_flags() {
        let cli = Cli::parse_from([
            "rubberduck",
            "--topic",
            "logic",
            "--log",
            "--quiet",
            "--no-anim",
            "--speed",
            "fast",
            "--color",
            "never",
            "--theme",
            "midnight",
        ]);
        assert_eq!(cli.topic.as_deref(), Some("logic"));
        assert!(cli.log && cli.quiet && cli.no_anim);
        assert_eq!(cli.speed, Some(Speed::Fast));
        assert_eq!(cli.color, Some(ColorPref::Never));
        assert_eq!(cli.theme.as_deref(), Some("midnight"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_subcommands() {
        assert!(matches!(
            Cli::parse_from(["rubberduck", "topics"]).command,
            Some(Command::Topics)
        ));
        assert!(matches!(
            Cli::parse_from(["rubberduck", "completions", "bash"]).command,
            Some(Command::Completions { .. })
        ));
        assert!(matches!(
            Cli::parse_from(["rubberduck", "self", "update", "--check"]).command,
            Some(Command::SelfCmd {
                action: SelfAction::Update { check: true }
            })
        ));
        assert!(matches!(
            Cli::parse_from(["rubberduck", "self", "uninstall"]).command,
            Some(Command::SelfCmd {
                action: SelfAction::Uninstall
            })
        ));
    }

    #[test]
    fn theme_rejects_unknown_and_accepts_known() {
        assert!(Cli::try_parse_from(["rubberduck", "--theme", "bogus"]).is_err());
        let cli = Cli::try_parse_from(["rubberduck", "--theme", "midnight"]).unwrap();
        assert_eq!(cli.theme.as_deref(), Some("midnight"));
    }

    #[test]
    fn global_flags_work_after_subcommand() {
        let cli = Cli::try_parse_from(["rubberduck", "topics", "--color", "never"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Topics)));
        assert_eq!(cli.color, Some(ColorPref::Never));
    }

    #[test]
    fn parses_config_subcommands() {
        assert!(matches!(
            Cli::parse_from(["rubberduck", "config", "init"]).command,
            Some(Command::Config {
                action: ConfigAction::Init
            })
        ));
        assert!(matches!(
            Cli::parse_from(["rubberduck", "config", "path"]).command,
            Some(Command::Config {
                action: ConfigAction::Path
            })
        ));
    }
}
