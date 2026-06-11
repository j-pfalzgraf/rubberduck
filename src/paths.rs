//! Platform-appropriate paths for configuration and logs.
//!
//! Two environment variables allow overrides (handy for tests and power users):
//! `RUBBERDUCK_CONFIG_DIR` and `RUBBERDUCK_DATA_DIR`.

use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

/// Config directory `~/.config/rubberduck` – identical on all platforms (per the
/// spec and matching the install/uninstall scripts and the README).
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("RUBBERDUCK_CONFIG_DIR") {
        return Ok(PathBuf::from(p));
    }
    let dirs = BaseDirs::new().context("Could not determine the home directory")?;
    Ok(dirs.home_dir().join(".config").join("rubberduck"))
}

/// Path to the settings file (`<config>/config.yaml`).
pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Data directory `~/.rubberduck` (deliberately not XDG, per the spec).
pub fn data_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("RUBBERDUCK_DATA_DIR") {
        return Ok(PathBuf::from(p));
    }
    let dirs = BaseDirs::new().context("Could not determine the home directory")?;
    Ok(dirs.home_dir().join(".rubberduck"))
}

/// Reads `path`, creating it with `default_contents` first if it is missing.
///
/// This is the shared "load a user-editable file, seeding it on first run"
/// primitive behind the question pool and the tips pool (DRY): on first use the
/// bundled defaults are written to disk so a team can edit and share them, and
/// every later run simply reads what is there.
pub fn read_or_init(path: &Path, default_contents: &str) -> Result<String> {
    if !path.exists() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Could not create {}", dir.display()))?;
        }
        fs::write(path, default_contents)
            .with_context(|| format!("Could not write {}", path.display()))?;
    }
    fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))
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
            config_file().unwrap(),
            PathBuf::from("/tmp/rubberduck-test-cfg/config.yaml")
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
