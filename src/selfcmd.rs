//! Selbstverwaltung: `self update` und `self uninstall`.
//!
//! `self update` lädt das passende Release-Archiv über GitHub (HTTPS/TLS) und
//! ersetzt das laufende Binary. Eine zusätzliche SHA256-Prüfung übernehmen die
//! Install-Skripte (`install.sh`/`install.ps1`); eine Signatur-Verifikation für
//! `self update` ist als Härtung vorgesehen (siehe README → „Geplant").

use crate::paths;
use anyhow::{bail, Context, Result};
use dialoguer::Confirm;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

/// GitHub-Repository, aus dem Releases bezogen werden. **Vor dem ersten Release
/// auf den echten Owner/Repo-Namen anpassen.**
pub const REPO_OWNER: &str = "j-pfalzgraf";
/// Repository-Name innerhalb von [`REPO_OWNER`].
pub const REPO_NAME: &str = "rubberduck";
/// Name des Binärassets innerhalb der Release-Archive.
pub const BIN_NAME: &str = "rubberduck";

/// Aktualisiert das Binary – oder prüft mit `check_only` nur auf Updates.
pub fn update(check_only: bool) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    if check_only {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .build()
            .context("Konnte die Release-Abfrage nicht konfigurieren")?
            .fetch()
            .context("Konnte Releases nicht abrufen")?;

        match releases.first() {
            Some(latest) => {
                let newer = self_update::version::bump_is_greater(current, &latest.version)
                    .unwrap_or(false);
                if newer {
                    println!("🦆 Update verfügbar: {current} → {}", latest.version);
                } else {
                    println!("🦆 rubberduck ist aktuell (Version {current}).");
                }
            }
            None => println!("Keine Releases gefunden."),
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
        .context("Update-Konfiguration fehlgeschlagen")?
        .update()
        .context("Update fehlgeschlagen")?;

    if status.updated() {
        println!("🦆 Aktualisiert auf Version {}.", status.version());
    } else {
        println!("🦆 Bereits aktuell (Version {}).", status.version());
    }
    Ok(())
}

/// Entfernt nach Rückfrage Konfiguration, Logs und das Binary selbst.
pub fn uninstall() -> Result<()> {
    let config = paths::config_dir()?;
    let data = paths::data_dir()?;
    let exe = std::env::current_exe().context("Konnte den eigenen Pfad nicht bestimmen")?;

    // Sicherheitsnetz: niemals Home- oder Wurzelverzeichnis löschen. Schützt vor
    // versehentlich gesetztem RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR.
    for dir in [&config, &data] {
        if is_unsafe_target(dir) {
            bail!(
                "Abbruch: '{}' sieht nach einem Home-/System-Verzeichnis aus und wird \
                 nicht gelöscht. Bitte RUBBERDUCK_CONFIG_DIR/RUBBERDUCK_DATA_DIR prüfen.",
                dir.display()
            );
        }
    }

    println!("Folgendes wird entfernt:");
    println!("  • Binary:        {}", exe.display());
    println!("  • Konfiguration: {}", config.display());
    println!("  • Logs:          {}", data.display());

    // Bestätigung braucht ein Terminal (analog zum TTY-Guard in uninstall.sh).
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        bail!(
            "Deinstallation braucht eine interaktive Bestätigung und ist ohne Terminal \
             nicht möglich. Nutze stattdessen 'uninstall.sh --yes'."
        );
    }

    let confirmed = Confirm::new()
        .with_prompt("Wirklich alles entfernen?")
        .default(false)
        .interact()
        .context("Abfrage fehlgeschlagen")?;

    if !confirmed {
        println!("Abgebrochen – nichts wurde entfernt.");
        return Ok(());
    }

    for dir in [&config, &data] {
        if dir.exists() {
            fs::remove_dir_all(dir)
                .with_context(|| format!("Konnte {} nicht entfernen", dir.display()))?;
            println!("Entfernt: {}", dir.display());
        }
    }

    // Das laufende Binary plattformübergreifend löschen – ganz zuletzt.
    if let Err(e) = self_replace::self_delete() {
        eprintln!(
            "Konfiguration und Logs wurden entfernt, aber das Binary unter {} konnte \
             nicht gelöscht werden. Bitte manuell entfernen.",
            exe.display()
        );
        return Err(e).context("Konnte das Binary nicht entfernen");
    }
    println!("rubberduck wurde entfernt. Danke fürs Quaken! 🦆");
    Ok(())
}

/// Prüft, ob `dir` ein gefährliches Löschziel ist: Wurzelverzeichnis, das
/// Home-Verzeichnis selbst oder ein Vorfahre davon.
fn is_unsafe_target(dir: &Path) -> bool {
    let resolved = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    // Ein Wurzelverzeichnis ("/" bzw. Laufwerks-Root) hat keinen Parent.
    if resolved.parent().is_none() {
        return true;
    }

    if let Some(base) = directories::BaseDirs::new() {
        let home = base
            .home_dir()
            .canonicalize()
            .unwrap_or_else(|_| base.home_dir().to_path_buf());
        // Exakt das Home-Verzeichnis oder ein Vorfahre davon -> gefährlich.
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
