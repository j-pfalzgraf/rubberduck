//! rubberduck – ein offline Rubber-Duck-Debugging-Begleiter fürs Terminal.
//!
//! Eine animierte ASCII-Ente stellt strukturierte Debugging-Fragen, bis der Bug
//! selbst gefunden ist – komplett offline, ohne externe KI.
//!
//! # Aufbau
//!
//! - [`ui`] – Terminal-Oberfläche: Theme, Animations-Engine, Ente.
//! - [`app`] – Controller, der eine Session orchestriert.
//! - [`questions`] / [`session`] / [`config`] – Daten- und Zustandsschicht.
//! - [`cli`] – Argument-Parsing; [`selfcmd`] – Update/Deinstallation.
//! - [`paths`] – plattformkonforme Pfade.

#![warn(missing_docs)]

pub mod app;
pub mod cli;
pub mod config;
pub mod paths;
pub mod questions;
pub mod selfcmd;
pub mod session;
pub mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use app::App;
use cli::{Cli, Command, ConfigAction, SelfAction};
use config::Config;
use ui::{Ui, UiSettings};

/// Liest die CLI-Argumente und führt den passenden Befehl aus.
pub fn run() -> Result<()> {
    let mut cli = Cli::parse();
    // Stellt den Cursor wieder her, falls Strg-C/Panic eine Animation unterbricht.
    ui::install_terminal_guards();

    // Unterbefehl herauslösen, damit `cli` für die `&cli`-Übergaben gültig bleibt.
    let command = cli.command.take();
    match command {
        Some(Command::SelfCmd { action }) => match action {
            SelfAction::Update { check } => selfcmd::update(check),
            SelfAction::Uninstall => selfcmd::uninstall(),
        },
        Some(Command::Topics) => list_topics(&cli),
        Some(Command::Completions { shell }) => {
            print_completions(shell);
            Ok(())
        }
        Some(Command::Config { action }) => config_command(&cli, action),
        None => run_session(&cli),
    }
}

/// Verwaltet die persistente `config.yaml` (`config init|show|path`).
fn config_command(cli: &Cli, action: ConfigAction) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let st = ui.styler();
    let path = paths::config_file()?;

    match action {
        ConfigAction::Path => println!("{}", path.display()),
        ConfigAction::Show => {
            println!(
                "{}",
                st.accent(&format!("Einstellungen ({})", path.display()))
            );
            print!("{}", config.to_yaml()?);
        }
        ConfigAction::Init => {
            if path.exists() {
                println!(
                    "{}",
                    st.dim(&format!("Existiert bereits: {}", path.display()))
                );
            } else {
                let written = config.save()?;
                println!("{} {}", st.success("Angelegt:"), written.display());
            }
        }
    }
    Ok(())
}

/// Startet eine interaktive Debugging-Session.
fn run_session(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let pool = questions::load_or_init()?;
    let mut app = App::new(ui, pool, config.default_topic.clone());
    app.run(cli.topic.as_deref(), cli.log)
}

/// Verschmilzt die persistente Konfiguration mit den CLI-Overrides.
fn ui_settings(config: &Config, cli: &Cli) -> UiSettings {
    let mut settings = config.base_ui_settings();
    settings.quiet = cli.quiet;
    if cli.no_anim {
        settings.animations = false;
    }
    if let Some(speed) = cli.speed {
        settings.speed = speed.multiplier();
    }
    if let Some(color) = cli.color {
        settings.color = color.into();
    }
    if let Some(theme) = &cli.theme {
        settings.theme = theme.clone();
    }
    settings
}

/// Gibt die verfügbaren Themen mit Beschreibung aus.
fn list_topics(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let st = ui.styler();
    let pool = questions::load_or_init()?;

    println!("{}", st.accent("Verfügbare Themen:"));
    for topic in pool.topics() {
        let marker = if topic.name == config.default_topic {
            st.success(" *")
        } else {
            "  ".to_string()
        };
        let desc = if topic.description.is_empty() {
            String::new()
        } else {
            st.dim(&format!("  – {}", topic.description))
        };
        println!("{} {}{}", marker, st.text(&topic.name), desc);
    }
    println!(
        "\n{}",
        st.dim("Start mit:  rubberduck --topic <name>   (* = Standard)")
    );
    Ok(())
}

/// Schreibt Shell-Completions für `shell` nach stdout.
fn print_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "rubberduck", &mut std::io::stdout());
}
