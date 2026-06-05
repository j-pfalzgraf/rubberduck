//! Persistente Nutzereinstellungen (`<config>/config.yaml`).
//!
//! Die Konfiguration ist die *persistente* Schicht. Beim Start liest die
//! Anwendung sie ein und überschreibt einzelne Felder mit CLI-Flags (siehe
//! [`crate::run`]). Fehlt die Datei, gelten die Standardwerte.

use crate::paths;
use crate::ui::theme::ColorChoice;
use crate::ui::UiSettings;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Animationsgeschwindigkeit (auch als CLI-Wert `--speed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum Speed {
    /// Gemütlich.
    Slow,
    /// Normal (Standard).
    #[default]
    Normal,
    /// Flott.
    Fast,
}

impl Speed {
    /// Geschwindigkeits-Multiplikator für Animationsverzögerungen.
    #[must_use]
    pub fn multiplier(self) -> f32 {
        match self {
            Speed::Slow => 0.55,
            Speed::Normal => 1.0,
            Speed::Fast => 2.2,
        }
    }
}

/// Farbpräferenz (auch als CLI-Wert `--color`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorPref {
    /// Automatisch (Terminal + `NO_COLOR` berücksichtigen).
    #[default]
    Auto,
    /// Immer farbig.
    Always,
    /// Nie farbig.
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

/// Die gespeicherten Nutzereinstellungen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Farbmodus.
    pub color: ColorPref,
    /// Name des Themes (siehe [`crate::ui::theme::Theme::NAMES`]).
    pub theme: String,
    /// Ob Animationen grundsätzlich an sind.
    pub animations: bool,
    /// Animationsgeschwindigkeit.
    pub speed: Speed,
    /// Ob der Tippeffekt an ist.
    pub typewriter: bool,
    /// Standardthema, wenn keines per `--topic` gewählt wird.
    pub default_topic: String,
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
        }
    }
}

impl Config {
    /// Parst eine Konfiguration aus YAML-Text.
    pub fn parse(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Ungültige config.yaml")
    }

    /// Serialisiert die Konfiguration als YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Konnte Konfiguration nicht serialisieren")
    }

    /// Lädt die Konfiguration aus der Standarddatei oder liefert die Defaults.
    ///
    /// Ein Parse-Fehler wird als Hinweis gemeldet, danach gelten die Defaults –
    /// eine kaputte Datei legt rubberduck also nicht lahm.
    #[must_use]
    pub fn load_or_default() -> Self {
        let Ok(path) = paths::config_file() else {
            return Self::default();
        };
        Self::load_or_default_at(&path)
    }

    /// Wie [`Config::load_or_default`], aber mit explizitem Pfad (für Tests).
    #[must_use]
    pub fn load_or_default_at(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => Self::parse(&content).unwrap_or_else(|err| {
                eprintln!("🦆 Hinweis: {err}; nutze Standardeinstellungen.");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Schreibt die Konfiguration in die Standarddatei und gibt deren Pfad zurück.
    pub fn save(&self) -> Result<PathBuf> {
        let path = paths::config_file()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Konnte {} nicht anlegen", dir.display()))?;
        }
        fs::write(&path, self.to_yaml()?)
            .with_context(|| format!("Konnte {} nicht schreiben", path.display()))?;
        Ok(path)
    }

    /// Basis-[`UiSettings`] aus dieser Konfiguration (vor CLI-Overrides).
    #[must_use]
    pub fn base_ui_settings(&self) -> UiSettings {
        UiSettings {
            color: self.color.into(),
            theme: self.theme.clone(),
            animations: self.animations,
            speed: self.speed.multiplier(),
            typewriter: self.typewriter,
            quiet: false,
        }
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
        assert!(back.animations && back.typewriter);
    }

    #[test]
    fn partial_yaml_fills_defaults() {
        let cfg = Config::parse("theme: midnight\nanimations: false\n").unwrap();
        assert_eq!(cfg.theme, "midnight");
        assert!(!cfg.animations);
        // Nicht gesetzte Felder kommen aus Default.
        assert_eq!(cfg.speed, Speed::Normal);
        assert_eq!(cfg.color, ColorPref::Auto);
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
}
