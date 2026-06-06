//! Platform-appropriate paths for configuration and logs.
//!
//! Two environment variables allow overrides (handy for tests and power users):
//! `RUBBERDUCK_CONFIG_DIR` and `RUBBERDUCK_DATA_DIR`.

use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

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
