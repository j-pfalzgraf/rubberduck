//! Self management: `self update` and `self uninstall`.
//!
//! `self update` downloads the matching release archive over GitHub (HTTPS/TLS)
//! and replaces the running binary. An additional SHA256 check is done by the
//! install scripts (`install.sh`/`install.ps1`); signature verification for
//! `self update` is a planned hardening (see the README → "Planned").

use crate::i18n::Tr;
use crate::paths;
use anyhow::{bail, Context, Result};
use dialoguer::Confirm;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

/// GitHub repository releases are fetched from. **Set this to the real
/// owner/repo before the first release.**
pub const REPO_OWNER: &str = "j-pfalzgraf";
/// Repository name within [`REPO_OWNER`].
pub const REPO_NAME: &str = "rubberduck";
/// Name of the binary asset inside the release archives.
pub const BIN_NAME: &str = "rubberduck";

/// Updates the binary – or, with `check_only`, only checks for updates.
pub fn update(check_only: bool, tr: Tr) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    if check_only {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .build()
            .context("Could not configure the release query")?
            .fetch()
            .context("Could not fetch releases")?;

        match releases.first() {
            Some(latest) => {
                let newer = self_update::version::bump_is_greater(current, &latest.version)
                    .unwrap_or(false);
                if newer {
                    println!("{}", tr.update_available(current, &latest.version));
                } else {
                    println!("{}", tr.up_to_date(current));
                }
            }
            None => println!("{}", tr.no_releases()),
        }
        return Ok(());
    }

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .show_download_progress(true)
        .current_version(current)
        .build()
        .context("Update configuration failed")?
        .update()
        .context("Update failed")?;

    if status.updated() {
        println!("{}", tr.updated_to(status.version()));
    } else {
        println!("{}", tr.already_current(status.version()));
    }
    Ok(())
}

/// Removes configuration, logs and the binary itself after confirmation.
pub fn uninstall(tr: Tr) -> Result<()> {
    let config = paths::config_dir()?;
    let data = paths::data_dir()?;
    let exe = std::env::current_exe().context("Could not determine own path")?;

    // Safety net: never delete the home or root directory. Guards against an
    // accidentally set RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR.
    for dir in [&config, &data] {
        if is_unsafe_target(dir) {
            bail!("{}", tr.uninstall_unsafe(&dir.display().to_string()));
        }
    }

    println!("{}", tr.uninstall_header());
    println!("  • {}: {}", tr.uninstall_label_binary(), exe.display());
    println!("  • {}: {}", tr.uninstall_label_config(), config.display());
    println!("  • {}: {}", tr.uninstall_label_logs(), data.display());

    // Confirmation needs a terminal (mirrors the TTY guard in uninstall.sh).
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        bail!("{}", tr.uninstall_needs_tty());
    }

    let confirmed = Confirm::new()
        .with_prompt(tr.uninstall_confirm())
        .default(false)
        .interact()
        .context("Confirmation failed")?;

    if !confirmed {
        println!("{}", tr.uninstall_cancelled());
        return Ok(());
    }

    for dir in [&config, &data] {
        if dir.exists() {
            fs::remove_dir_all(dir)
                .with_context(|| format!("Could not remove {}", dir.display()))?;
            println!("{}", tr.uninstall_removed(&dir.display().to_string()));
        }
    }

    // Delete the running binary cross-platform – last of all.
    if let Err(e) = self_replace::self_delete() {
        eprintln!("{}", tr.uninstall_binary_failed(&exe.display().to_string()));
        return Err(e).context("Could not delete the binary");
    }
    println!("{}", tr.uninstall_done());
    Ok(())
}

/// Whether `dir` is a dangerous deletion target: the root directory, the home
/// directory itself, or an ancestor of it.
fn is_unsafe_target(dir: &Path) -> bool {
    let resolved = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    // A root directory ("/" or a drive root) has no parent.
    if resolved.parent().is_none() {
        return true;
    }

    if let Some(base) = directories::BaseDirs::new() {
        let home = base
            .home_dir()
            .canonicalize()
            .unwrap_or_else(|_| base.home_dir().to_path_buf());
        // Exactly the home directory, or an ancestor of it -> dangerous.
        if resolved == home || home.starts_with(&resolved) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn root_is_unsafe() {
        assert!(is_unsafe_target(Path::new("/")));
    }

    #[test]
    fn deep_non_home_subdir_is_safe() {
        let p = PathBuf::from("/tmp/rubberduck-uninstall-xyz/config/rubberduck");
        assert!(!is_unsafe_target(&p));
    }
}
