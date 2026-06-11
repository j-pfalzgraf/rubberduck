//! rubberduck – an offline rubber-duck-debugging companion for your terminal.
//!
//! An animated ASCII duck asks structured debugging questions until you find the
//! bug yourself – fully offline, no external AI. The user interface is
//! internationalized and defaults to English (see [`i18n`]).
//!
//! # Layout
//!
//! - [`ui`] – terminal layer: theme, animation engine, duck, charts.
//! - [`app`] – controller that orchestrates a session.
//! - [`questions`] / [`tips`] / [`session`] / [`config`] – data and state layer.
//! - [`i18n`] – languages and the translator.
//! - [`cli`] – argument parsing; [`selfcmd`] – update/uninstall.
//! - [`demo`] – the animated tour; [`history`]/[`stats`] – session insights.
//! - [`doctor`] – environment diagnostics.
//! - [`paths`] – platform-appropriate paths; [`util`] – small helpers.

#![warn(missing_docs)]

pub mod app;
pub mod cli;
pub mod config;
pub mod demo;
pub mod doctor;
pub mod history;
pub mod i18n;
pub mod paths;
pub mod questions;
pub mod selfcmd;
pub mod session;
pub mod stats;
pub mod tips;
pub mod ui;
pub mod util;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};

use app::App;
use cli::{Cli, Command, ConfigAction, SelfAction};
use config::Config;
use i18n::Lang;
use ui::{Mood, Ui, UiSettings};

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
        Some(Command::Themes) => themes_command(&cli),
        Some(Command::Demo) => demo_command(&cli),
        Some(Command::Tip) => tip_command(&cli),
        Some(Command::Tips) => tips_command(&cli),
        Some(Command::Stats { reset, json }) => stats_command(&cli, reset, json),
        Some(Command::History { limit, json }) => history_command(&cli, limit, json),
        Some(Command::Doctor) => doctor_command(&cli),
        Some(Command::Completions { shell }) => {
            print_completions(shell);
            Ok(())
        }
        Some(Command::Man) => print_man(),
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
        ConfigAction::Reset => {
            let written = Config::default().save()?;
            println!(
                "{}",
                st.success(&tr.config_reset_done(&written.display().to_string()))
            );
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

/// Lists the available colour themes with a live preview.
fn themes_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let st = ui.styler();
    let tr = ui.tr();
    println!("{}", st.accent(tr.themes_header()));
    ui.theme_previews();
    println!("\n{}", st.dim(tr.themes_hint()));
    Ok(())
}

/// Shows a single random debugging tip, delivered by the duck.
fn tip_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    let lang = ui.tr().lang();
    let pool = tips::load_or_init(lang)?;
    let tip = pool.random().to_string();
    ui.swim_in(Mood::Reading)?;
    ui.duck_says(&tip, Mood::Reading)?;
    Ok(())
}

/// Lists every bundled debugging tip for the active language.
fn tips_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let ui = Ui::new(ui_settings(&config, cli));
    let lang = ui.tr().lang();
    let pool = tips::load_or_init(lang)?;
    let st = ui.styler();
    println!("{}", st.accent(ui.tr().tips_header()));
    for (i, tip) in pool.all().iter().enumerate() {
        println!("  {} {}", st.dim(&format!("{:>2}.", i + 1)), st.text(tip));
    }
    Ok(())
}

/// Shows aggregate statistics from the session history (or clears it).
fn stats_command(cli: &Cli, reset: bool, json: bool) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    stats::show(&mut ui, reset, json)
}

/// Lists the most recent recorded sessions.
fn history_command(cli: &Cli, limit: Option<usize>, json: bool) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    history::show(&mut ui, limit, json)
}

/// Runs the environment / configuration diagnostics.
fn doctor_command(cli: &Cli) -> Result<()> {
    let config = Config::load_or_default();
    let mut ui = Ui::new(ui_settings(&config, cli));
    doctor::run(&mut ui)
}

/// Writes shell completions for `shell` to stdout.
fn print_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "rubberduck", &mut std::io::stdout());
}

/// Writes a man page (roff) to stdout.
fn print_man() -> Result<()> {
    clap_mangen::Man::new(Cli::command())
        .render(&mut std::io::stdout())
        .context("Could not render man page")?;
    Ok(())
}
