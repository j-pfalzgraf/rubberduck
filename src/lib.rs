//! rubberduck – an offline rubber-duck-debugging companion for your terminal.
//!
//! An animated ASCII duck asks structured debugging questions until you find the
//! bug yourself – fully offline, no external AI. The user interface is
//! internationalized and defaults to English (see [`i18n`]).
//!
//! # Layout
//!
//! - [`ui`] – terminal layer: theme, animation engine, duck.
//! - [`app`] – controller that orchestrates a session.
//! - [`questions`] / [`session`] / [`config`] – data and state layer.
//! - [`i18n`] – languages and the translator.
//! - [`cli`] – argument parsing; [`selfcmd`] – update/uninstall.
//! - [`demo`] – the animated tour; [`history`]/[`stats`] – session insights.
//! - [`paths`] – platform-appropriate paths; [`util`] – small helpers.

#![warn(missing_docs)]

pub mod app;
pub mod cli;
pub mod config;
pub mod demo;
pub mod history;
pub mod i18n;
pub mod paths;
pub mod questions;
pub mod selfcmd;
pub mod session;
pub mod stats;
pub mod ui;
pub mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use app::App;
use cli::{Cli, Command, ConfigAction, SelfAction};
use config::Config;
use i18n::Lang;
use ui::{Ui, UiSettings};

/// Parses the CLI arguments and runs the matching command.
pub fn run() -> Result<()> {
    let mut cli = Cli::parse();
    // Restores the cursor if Ctrl-C/panic interrupts an animation.
    ui::install_terminal_guards();

    // Pull the subcommand out so `cli` stays valid for the `&cli` borrows.
    let command = cli.command.take();
    match command {
        Some(Command::SelfCmd { action }) => {
            let config = Config::load_or_default();
            let tr = resolve_lang(&cli, &config).translator();
            match action {
                SelfAction::Update { check } => selfcmd::update(check, tr),
                SelfAction::Uninstall => selfcmd::uninstall(tr),
            }
        }
        Some(Command::Topics) => list_topics(&cli),
        Some(Command::Languages) => languages_command(&cli),
        Some(Command::Demo) => demo_command(&cli),
        Some(Command::Stats { reset }) => stats_command(&cli, reset),
        Some(Command::Completions { shell }) => {
            print_completions(shell);
            Ok(())
        }
        Some(Command::Config { action }) => config_command(&cli, action),
        None => run_session(&cli),
    }
}

/// Resolves the language: `--lang` flag › `RUBBERDUCK_LANG` env › config › English.
fn resolve_lang(cli: &Cli, config: &Config) -> Lang {
    cli.lang.or_else(Lang::from_env).unwrap_or(config.language)
}

/// Starts an interactive debugging session.
fn run_session(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let settings = ui_settings(&config, cli);
    let lang = settings.lang;
    let ui = Ui::new(settings);
    let pool = questions::load_or_init(lang)?;
    let mut app = App::new(ui, pool, config.default_topic.clone(), config.history);
    app.run(cli.topic.as_deref(), cli.log)
}

/// Merges the persistent configuration with the CLI overrides.
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
    settings.lang = resolve_lang(cli, config);
    settings
}

/// Prints the available topics with their descriptions.
fn list_topics(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let settings = ui_settings(&config, cli);
    let lang = settings.lang;
    let ui = Ui::new(settings);
    let st = ui.styler();
    let tr = ui.tr();
    let pool = questions::load_or_init(lang)?;

    println!("{}", st.accent(tr.topics_header()));
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
    println!("\n{}", st.dim(tr.topics_hint()));
    Ok(())
}

/// Manages the persistent `config.yaml` (`config init|show|path`).
fn config_command(cli: &Cli, action: ConfigAction) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let st = ui.styler();
    let tr = ui.tr();
    let path = paths::config_file()?;

    match action {
        ConfigAction::Path => println!("{}", path.display()),
        ConfigAction::Show => {
            println!(
                "{}",
                st.accent(&tr.config_settings_header(&path.display().to_string()))
            );
            print!("{}", config.to_yaml()?);
        }
        ConfigAction::Init => {
            if path.exists() {
                println!("{}", st.dim(&tr.config_exists(&path.display().to_string())));
            } else {
                let written = config.save()?;
                println!(
                    "{}",
                    st.success(&tr.config_created(&written.display().to_string()))
                );
            }
        }
        ConfigAction::Set { key, value } => {
            let mut updated = config.clone();
            updated.set(&key, &value)?;
            updated.save()?;
            println!("{}", st.success(&tr.config_set_done(&key, &value)));
        }
    }
    Ok(())
}

/// Lists the available interface languages (`*` marks the active one).
fn languages_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let st = ui.styler();
    let tr = ui.tr();
    let active = tr.lang();

    println!("{}", st.accent(tr.languages_header()));
    for lang in Lang::ALL {
        let marker = if lang == active {
            st.success(" *")
        } else {
            "  ".to_string()
        };
        println!(
            "{} {}  {}",
            marker,
            st.text(lang.code()),
            st.dim(lang.label())
        );
    }
    Ok(())
}

/// Plays the animated demo tour.
fn demo_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    demo::run(&mut ui)
}

/// Shows aggregate statistics from the session history (or clears it).
fn stats_command(cli: &Cli, reset: bool) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    stats::show(&mut ui, reset)
}

/// Writes shell completions for `shell` to stdout.
fn print_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "rubberduck", &mut std::io::stdout());
}
