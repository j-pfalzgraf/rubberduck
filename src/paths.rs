//! Plattformkonforme Pfade für Konfiguration und Logs.
//!
//! Zwei Umgebungsvariablen erlauben Overrides (praktisch für Tests und
//! Power-User): `RUBBERDUCK_CONFIG_DIR` und `RUBBERDUCK_DATA_DIR`.

use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

/// Konfigurationsverzeichnis `~/.config/rubberduck` – auf allen Plattformen
/// gleich (gemäß Spec und identisch zu install/uninstall-Skripten und README).
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("RUBBERDUCK_CONFIG_DIR") {
        return Ok(PathBuf::from(p));
    }
    let dirs = BaseDirs::new().context("Konnte das Home-Verzeichnis nicht bestimmen")?;
    Ok(dirs.home_dir().join(".config").join("rubberduck"))
}

/// Pfad zur Fragen-Datei (`<config>/questions.yaml`).
pub fn questions_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("questions.yaml"))
}

/// Pfad zur Einstellungsdatei (`<config>/config.yaml`).
pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Datenverzeichnis `~/.rubberduck` (bewusst nicht XDG, gemäß Spec).
pub fn data_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("RUBBERDUCK_DATA_DIR") {
        return Ok(PathBuf::from(p));
    }
    let dirs = BaseDirs::new().context("Konnte das Home-Verzeichnis nicht bestimmen")?;
    Ok(dirs.home_dir().join(".rubberduck"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_respects_override() {
        std::env::set_var("RUBBERDUCK_CONFIG_DIR", "/tmp/rubberduck-test-cfg");
        assert_eq!(
            config_dir().unwrap(),
            PathBuf::from("/tmp/rubberduck-test-cfg")
        );
        assert_eq!(
            questions_file().unwrap(),
            PathBuf::from("/tmp/rubberduck-test-cfg/questions.yaml")
        );
        std::env::remove_var("RUBBERDUCK_CONFIG_DIR");
    }

    #[test]
    fn data_dir_respects_override() {
        std::env::set_var("RUBBERDUCK_DATA_DIR", "/tmp/rubberduck-test-data");
        assert_eq!(
            data_dir().unwrap(),
            PathBuf::from("/tmp/rubberduck-test-data")
        );
        std::env::remove_var("RUBBERDUCK_DATA_DIR");
    }
}
