//! CLI-Definition mit `clap` (derive).

use clap::{Parser, Subcommand};

/// Top-Level-Argumente.
///
/// Ohne Unterbefehl startet eine Debugging-Session; die Flags `--topic`,
/// `--log` und `--quiet` steuern diese Session.
#[derive(Parser, Debug)]
#[command(
    name = "rubberduck",
    version,
    about = "Ein offline Rubber-Duck-Debugging-Begleiter fürs Terminal.",
    long_about = "rubberduck stellt dir strukturierte Debugging-Fragen, bis du den \
                  Bug selbst findest – komplett offline, ohne externe KI.",
    after_help = "Beispiele:\n  \
        rubberduck                 Standard-Session\n  \
        rubberduck --topic logic   Themen-Fragen zu Logikfehlern\n  \
        rubberduck --log           Session als Markdown speichern\n  \
        rubberduck --quiet         ohne ASCII-Ente\n  \
        rubberduck self update     auf neueste Version aktualisieren"
)]
pub struct Cli {
    /// Themen-Fragenset (z. B. default, logic, perf, api).
    #[arg(long, value_name = "TOPIC")]
    pub topic: Option<String>,

    /// Sitzung als Markdown-Logbuch unter ~/.rubberduck speichern.
    #[arg(long)]
    pub log: bool,

    /// Ohne ASCII-Ente ausgeben, nur Text.
    #[arg(long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Unterbefehle der obersten Ebene.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Selbstverwaltung: Update und Deinstallation.
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        action: SelfAction,
    },
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
        // Erzwingt claps interne Konsistenzprüfung der Ableitung.
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_topic_and_flags() {
        let cli = Cli::parse_from(["rubberduck", "--topic", "logic", "--log", "--quiet"]);
        assert_eq!(cli.topic.as_deref(), Some("logic"));
        assert!(cli.log);
        assert!(cli.quiet);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_self_update_check() {
        let cli = Cli::parse_from(["rubberduck", "self", "update", "--check"]);
        match cli.command {
            Some(Commands::SelfCmd {
                action: SelfAction::Update { check },
            }) => assert!(check),
            other => panic!("unerwartet: {other:?}"),
        }
    }

    #[test]
    fn parses_self_uninstall() {
        let cli = Cli::parse_from(["rubberduck", "self", "uninstall"]);
        assert!(matches!(
            cli.command,
            Some(Commands::SelfCmd {
                action: SelfAction::Uninstall
            })
        ));
    }
}
