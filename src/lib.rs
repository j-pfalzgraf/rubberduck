//! rubberduck – ein offline Rubber-Duck-Debugging-Begleiter.
//!
//! Die Module sind bewusst klein und (wo möglich) frei von Terminal-I/O,
//! damit sich die Logik direkt testen lässt.

pub mod cli;
pub mod dialog;
pub mod duck;
pub mod paths;
pub mod questions;
pub mod selfcmd;
pub mod session;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands, SelfAction};

/// Liest die CLI-Argumente, wählt den passenden Befehl und führt ihn aus.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::SelfCmd { action }) => match action {
            SelfAction::Update { check } => selfcmd::update(check),
            SelfAction::Uninstall => selfcmd::uninstall(),
        },
        None => run_dialog(&cli),
    }
}

/// Standard-Session: Fragenpool laden, Dialog führen, optional protokollieren.
fn run_dialog(cli: &Cli) -> Result<()> {
    let topic = cli.topic.as_deref().unwrap_or(questions::DEFAULT_TOPIC);
    let pool = questions::load_or_init()?;
    let transcript = dialog::run_session(topic, cli.quiet, &pool)?;

    if cli.log {
        let path = session::write_log(&transcript)?;
        println!("\n🦆 Logbuch gespeichert: {}", path.display());
    }
    Ok(())
}
