//! Laden und Verwalten des Fragen-Pools.

use crate::paths;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Name des Standardthemas.
pub const DEFAULT_TOPIC: &str = "default";

/// Der gesamte Fragen-Pool: Themenname -> Liste von Fragen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPool {
    pub topics: BTreeMap<String, Vec<String>>,
}

impl QuestionPool {
    /// Fragen eines Themas; Fehler mit Liste verfügbarer Themen, falls unbekannt.
    pub fn questions_for(&self, topic: &str) -> Result<&[String]> {
        self.topics.get(topic).map(Vec::as_slice).ok_or_else(|| {
            let available = self.topics.keys().cloned().collect::<Vec<_>>().join(", ");
            anyhow!("Unbekanntes Thema '{topic}'. Verfügbar: {available}")
        })
    }
}

/// Der im Binary eingebettete Standard-Pool als YAML-Text.
pub fn embedded_yaml() -> &'static str {
    include_str!("../questions.yaml")
}

/// Den eingebetteten Standard-Pool parsen.
pub fn embedded() -> Result<QuestionPool> {
    serde_yaml::from_str(embedded_yaml()).context("Eingebettetes questions.yaml ist ungültig")
}

/// Pool aus der Konfigurationsdatei laden; beim ersten Mal die Datei mit den
/// Standardfragen anlegen, damit Teams sie bearbeiten können.
pub fn load_or_init() -> Result<QuestionPool> {
    load_or_init_at(&paths::questions_file()?)
}

/// Wie [`load_or_init`], aber mit explizitem Pfad (für Tests).
fn load_or_init_at(path: &Path) -> Result<QuestionPool> {
    if !path.exists() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Konnte {} nicht anlegen", dir.display()))?;
        }
        fs::write(path, embedded_yaml())
            .with_context(|| format!("Konnte {} nicht schreiben", path.display()))?;
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Konnte {} nicht lesen", path.display()))?;
    serde_yaml::from_str(&content).with_context(|| format!("Ungültiges YAML in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_parses_and_has_topics() {
        let pool = embedded().unwrap();
        for topic in ["default", "logic", "perf", "api"] {
            assert!(pool.topics.contains_key(topic), "Thema {topic} fehlt");
            assert!(!pool.topics[topic].is_empty(), "Thema {topic} ist leer");
        }
    }

    #[test]
    fn unknown_topic_lists_available() {
        let pool = embedded().unwrap();
        let err = pool.questions_for("gibtsnicht").unwrap_err().to_string();
        assert!(err.contains("Unbekanntes Thema"));
        assert!(err.contains("default"));
    }

    #[test]
    fn load_or_init_writes_then_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("questions.yaml");

        let first = load_or_init_at(&path).unwrap();
        assert!(path.exists(), "Datei sollte angelegt werden");

        let second = load_or_init_at(&path).unwrap();
        assert_eq!(first.topics.len(), second.topics.len());
        assert!(second.topics.contains_key("default"));
    }
}
