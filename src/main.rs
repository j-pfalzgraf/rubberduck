//! Binary entry point for `rubberduck`.
//!
//! The actual logic lives in the `rubberduck_cli` library so it can be tested
//! independently of a terminal.

fn main() {
    if let Err(err) = rubberduck_cli::run() {
        // A broken pipe (e.g. `rubberduck man | head`, or a pager closed early)
        // is not a real error — exit quietly.
        let broken_pipe = err
            .chain()
            .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
            .any(|io| io.kind() == std::io::ErrorKind::BrokenPipe);
        if broken_pipe {
            std::process::exit(0);
        }
        // `{err:#}` appends the anyhow cause chain.
        eprintln!("🦆 Error: {err:#}");
        std::process::exit(1);
    }
}
