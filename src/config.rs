//! Persistent user settings (`<config>/config.yaml`).
//!
//! The configuration is the *persistent* layer. At startup the application reads
//! it and overrides individual fields with CLI flags (see [`crate::run`]). If the
//! file is missing, the defaults apply.

use crate::i18n::Lang;
use crate::paths;
use crate::ui::theme::{ColorChoice, Theme};
use crate::ui::UiSettings;
use anyhow::{anyhow, bail, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// All settable configuration keys (for `config set` help and validation).
pub const KEYS: &[&str] = &[
    "color",
    "theme",
    "animations",
    "speed",
    "typewriter",
    "default_topic",
    "language",
    "history",
];

/// Animation speed (also the `--speed` CLI value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum Speed {
    /// Leisurely.
    Slow,
    /// Normal (default).
    #[default]
    Normal,
    /// Brisk.
    Fast,
}

impl Speed {
    /// Speed multiplier for animation delays.
    #[must_use]
    pub fn multiplier(self) -> f32 {
        match self {
            Speed::Slow => 0.55,
            Speed::Normal => 1.0,
            Speed::Fast => 2.2,
        }
    }
}

/// Colour preference (also the `--color` CLI value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorPref {
    /// Automatic (respect the terminal and `NO_COLOR`).
    #[default]
    Auto,
    /// Always colour.
    Always,
    /// Never colour.
    Never,
}

impl From<ColorPref> for ColorChoice {
    fn from(pref: ColorPref) -> Self {
        match pref {
            ColorPref::Auto => ColorChoice::Auto,
            ColorPref::Always => ColorChoice::Always,
            ColorPref::Never => ColorChoice::Never,
        }
    }
}

/// The stored user settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Colour mode.
    pub color: ColorPref,
    /// Theme name (see [`crate::ui::theme::Theme::NAMES`]).
    pub theme: String,
    /// Whether animations are on at all.
    pub animations: bool,
    /// Animation speed.
    pub speed: Speed,
    /// Whether the typewriter effect is on.
    pub typewriter: bool,
    /// Default topic when none is chosen via `--topic`.
    pub default_topic: String,
    /// User-interface language (default: English).
    pub language: Lang,
    /// Whether finished sessions are recorded to the history (for `stats`).
    pub history: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color: ColorPref::Auto,
            theme: "classic".to_string(),
            animations: true,
            speed: Speed::Normal,
            typewriter: true,
            default_topic: crate::questions::DEFAULT_TOPIC.to_string(),
            language: Lang::English,
            history: true,
        }
    }
}

impl Config {
    /// Parses a configuration from YAML text.
    pub fn parse(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Invalid config.yaml")
    }

    /// Serializes the configuration as YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Could not serialize configuration")
    }

    /// Loads the configuration from the default file, or returns the defaults.
    ///
    /// A parse error is reported as a notice, then the defaults apply – a broken
    /// file therefore never takes rubberduck down.
    #[must_use]
    pub fn load_or_default() -> Self {
        let Ok(path) = paths::config_file() else {
            return Self::default();
        };
        Self::load_or_default_at(&path)
    }

    /// Like [`Config::load_or_default`], but with an explicit path (for tests).
    #[must_use]
    pub fn load_or_default_at(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => Self::parse(&content).unwrap_or_else(|err| {
                eprintln!("🦆 Note: {err}; using default settings.");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Writes the configuration to the default file and returns its path.
    pub fn save(&self) -> Result<PathBuf> {
        let path = paths::config_file()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Could not create {}", dir.display()))?;
        }
        fs::write(&path, self.to_yaml()?)
            .with_context(|| format!("Could not write {}", path.display()))?;
        Ok(path)
    }

    /// Sets a setting from a string `key`/`value` pair (used by `config set`).
    ///
    /// Validates the value against the field's type; returns an error naming the
    /// valid options on a bad key or value.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "color" => self.color = parse_enum::<ColorPref>(value, key)?,
            "theme" => {
                if !Theme::NAMES.contains(&value) {
                    bail!(
                        "Invalid theme '{value}'. Valid: {}",
                        Theme::NAMES.join(", ")
                    );
                }
                self.theme = value.to_string();
            }
            "animations" => self.animations = parse_bool(value, key)?,
            "speed" => self.speed = parse_enum::<Speed>(value, key)?,
            "typewriter" => self.typewriter = parse_bool(value, key)?,
            "default_topic" => self.default_topic = value.to_string(),
            "language" => {
                self.language = Lang::from_code(value)
                    .ok_or_else(|| anyhow!("Invalid language '{value}'. Valid: en, de"))?;
            }
            "history" => self.history = parse_bool(value, key)?,
            other => bail!("Unknown setting '{other}'. Valid keys: {}", KEYS.join(", ")),
        }
        Ok(())
    }

    /// Base [`UiSettings`] from this configuration (before CLI overrides).
    #[must_use]
    pub fn base_ui_settings(&self) -> UiSettings {
        UiSettings {
            color: self.color.into(),
            theme: self.theme.clone(),
            animations: self.animations,
            speed: self.speed.multiplier(),
            typewriter: self.typewriter,
            quiet: false,
            lang: self.language,
        }
    }
}

/// Parses a clap `ValueEnum` from a string (case-insensitive).
fn parse_enum<T: ValueEnum>(value: &str, key: &str) -> Result<T> {
    T::from_str(value, true).map_err(|_| anyhow!("Invalid value '{value}' for '{key}'."))
}

/// Parses a permissive boolean (`true/false/on/off/yes/no/1/0`).
fn parse_bool(value: &str, key: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Ok(true),
        "false" | "off" | "no" | "0" => Ok(false),
        _ => Err(anyhow!(
            "Invalid boolean '{value}' for '{key}' (use true/false)."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips_through_yaml() {
        let cfg = Config::default();
        let yaml = cfg.to_yaml().unwrap();
        let back = Config::parse(&yaml).unwrap();
        assert_eq!(back.theme, "classic");
        assert_eq!(back.speed, Speed::Normal);
        assert_eq!(back.language, Lang::English);
        assert!(back.animations && back.typewriter);
    }

    #[test]
    fn partial_yaml_fills_defaults() {
        let cfg = Config::parse("theme: midnight\nanimations: false\n").unwrap();
        assert_eq!(cfg.theme, "midnight");
        assert!(!cfg.animations);
        // Unset fields come from Default.
        assert_eq!(cfg.speed, Speed::Normal);
        assert_eq!(cfg.color, ColorPref::Auto);
        assert_eq!(cfg.language, Lang::English);
    }

    #[test]
    fn language_can_be_set() {
        let cfg = Config::parse("language: de\n").unwrap();
        assert_eq!(cfg.language, Lang::German);
    }

    #[test]
    fn broken_file_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        std::fs::write(&path, "color: [this, is, wrong]\n").unwrap();
        let cfg = Config::load_or_default_at(&path);
        assert_eq!(cfg.color, ColorPref::Auto);
    }

    #[test]
    fn speed_multipliers_are_ordered() {
        assert!(Speed::Slow.multiplier() < Speed::Normal.multiplier());
        assert!(Speed::Normal.multiplier() < Speed::Fast.multiplier());
    }

    #[test]
    fn set_updates_and_validates() {
        let mut c = Config::default();
        c.set("theme", "midnight").unwrap();
        assert_eq!(c.theme, "midnight");
        c.set("history", "off").unwrap();
        assert!(!c.history);
        c.set("language", "de").unwrap();
        assert_eq!(c.language, Lang::German);
        c.set("speed", "fast").unwrap();
        assert_eq!(c.speed, Speed::Fast);
        // Bad key, bad theme, bad bool are all rejected.
        assert!(c.set("nope", "x").is_err());
        assert!(c.set("theme", "bogus").is_err());
        assert!(c.set("animations", "maybe").is_err());
    }
}
