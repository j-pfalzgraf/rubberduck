//! CLI definition with `clap` (derive): flags, topic listing and shell completions.

use crate::config::{ColorPref, Speed};
use crate::i18n::Lang;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Top-level arguments.
///
/// Without a subcommand a debugging session starts; the flags control the topic,
/// the log, the duck and the animations.
#[derive(Parser, Debug)]
#[command(
    name = "rubberduck",
    version,
    about = "An offline rubber-duck-debugging companion for your terminal.",
    long_about = "rubberduck asks you structured debugging questions until you find the \
                  bug yourself — fully offline, with an animated ASCII duck.",
    after_help = "Examples:\n  \
        rubberduck                     start a session (topic picker if no --topic)\n  \
        rubberduck --topic logic       jump straight into the logic question set\n  \
        rubberduck --log               save the session as Markdown\n  \
        rubberduck --no-anim --quiet   no animation/duck (e.g. for logs)\n  \
        rubberduck --theme midnight    a different colour scheme\n  \
        rubberduck --lang de           switch the language to German\n  \
        rubberduck topics              show the available topics\n  \
        rubberduck completions zsh     print shell completions\n  \
        rubberduck self update         update to the latest version\n\n\
        Tip: type !aha during a session as soon as you have found the bug."
)]
pub struct Cli {
    /// Topic question set; see `rubberduck topics` for the available names.
    #[arg(long, value_name = "TOPIC")]
    pub topic: Option<String>,

    /// Save the session as a Markdown log under ~/.rubberduck.
    #[arg(long)]
    pub log: bool,

    // Presentation flags are `global` so they also work after a subcommand
    // (e.g. `rubberduck topics --color never`).
    /// Print without the ASCII duck, just concise text.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Disable animations (a static duck instead of the typewriter & co.).
    #[arg(long = "no-anim", global = true)]
    pub no_anim: bool,

    /// Animation speed (slow, normal, fast).
    #[arg(long, value_enum, global = true)]
    pub speed: Option<Speed>,

    /// Colour mode (auto, always, never).
    #[arg(long, value_enum, global = true)]
    pub color: Option<ColorPref>,

    /// Colour scheme (classic, midnight, mono).
    #[arg(
        long,
        global = true,
        value_parser = clap::builder::PossibleValuesParser::new(crate::ui::theme::Theme::NAMES.iter().copied())
    )]
    pub theme: Option<String>,

    /// User-interface language (en, de).
    #[arg(long, value_enum, global = true)]
    pub lang: Option<Lang>,

    /// Subcommand (instead of a session).
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// List the available topics with their descriptions.
    Topics,

    /// Print shell completions (bash, zsh, fish, powershell, elvish).
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Manage persistent settings (config.yaml).
    Config {
        /// Configuration action to run.
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Self management: update and uninstall.
    #[command(name = "self")]
    SelfCmd {
        /// Self-management action to run.
        #[command(subcommand)]
        action: SelfAction,
    },
}

/// Actions under `rubberduck config`.
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Create a default `config.yaml` (does nothing if it already exists).
    Init,
    /// Show the effective settings and the file path.
    Show,
    /// Print only the path of `config.yaml`.
    Path,
}

/// Actions under `rubberduck self`.
#[derive(Subcommand, Debug)]
pub enum SelfAction {
    /// Update to the latest version.
    Update {
        /// Only check whether an update is available (install nothing).
        #[arg(long)]
        check: bool,
    },
    /// Remove rubberduck along with its configuration and logs.
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
            "--lang",
            "de",
        ]);
        assert_eq!(cli.topic.as_deref(), Some("logic"));
        assert!(cli.log && cli.quiet && cli.no_anim);
        assert_eq!(cli.speed, Some(Speed::Fast));
        assert_eq!(cli.color, Some(ColorPref::Never));
        assert_eq!(cli.theme.as_deref(), Some("midnight"));
        assert_eq!(cli.lang, Some(Lang::German));
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
        let cli = Cli::try_parse_from(["rubberduck", "topics", "--color", "never", "--lang", "de"])
            .unwrap();
        assert!(matches!(cli.command, Some(Command::Topics)));
        assert_eq!(cli.color, Some(ColorPref::Never));
        assert_eq!(cli.lang, Some(Lang::German));
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
