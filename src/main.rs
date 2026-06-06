//! Binary entry point for `rubberduck`.
//!
//! The actual logic lives in the `rubberduck_cli` library so it can be tested
//! independently of a terminal.

fn main() {
    if let Err(err) = rubberduck_cli::run() {
        // `{err:#}` appends the anyhow cause chain.
        eprintln!("🦆 Error: {err:#}");
        std::process::exit(1);
    }
}
