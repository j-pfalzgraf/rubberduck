//! Binär-Einstiegspunkt für `rubberduck`.
//!
//! Die eigentliche Logik liegt in der Bibliothek `rubberduck_cli`, damit sie
//! unabhängig vom Terminal getestet werden kann.

fn main() {
    if let Err(err) = rubberduck_cli::run() {
        // `{err:#}` hängt die anyhow-Ursachenkette mit an.
        eprintln!("🦆 Fehler: {err:#}");
        std::process::exit(1);
    }
}
