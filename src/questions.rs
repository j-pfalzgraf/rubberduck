//! Laden und Verwalten des Fragen-Pools.
//!
//! Der Pool ordnet jedem Thema eine [`Topic`] (Beschreibung + Fragen) zu. Das
//! YAML-Format unterstützt zwei Schreibweisen pro Thema – eine schlanke Liste
//! (rückwärtskompatibel) und eine reiche Form mit Beschreibung:
//!
//! ```yaml
//! topics:
//!   schlank:
//!     - "Nur eine Frageliste."
//!   reich:
//!     description: "Mit Beschreibung für den Themen-Picker."
//!     questions:
//!       - "Erste Frage?"
//! ```

use crate::paths;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Name des Standardthemas.
pub const DEFAULT_TOPIC: &str = "default";

/// Ein Thema mit Beschreibung und Fragen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    /// Technischer Name (Schlüssel, via `--topic` erreichbar).
    pub name: String,
    /// Kurzbeschreibung (für den Themen-Picker); kann leer sein.
    pub description: String,
    /// Die Fragen in Reihenfolge.
    pub questions: Vec<String>,
}

/// Der gesamte Fragen-Pool, nach Themennamen sortiert.
#[derive(Debug, Clone)]
pub struct QuestionPool {
    topics: BTreeMap<String, Topic>,
}

/// Roh-Form eines Themas: schlanke Liste **oder** reiche Form (serde-untagged).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawTopic {
    Simple(Vec<String>),
    Rich {
        #[serde(default)]
        description: String,
        questions: Vec<String>,
    },
}

#[derive(Debug, Deserialize)]
struct RawPool {
    topics: BTreeMap<String, RawTopic>,
}

impl QuestionPool {
    /// Parst einen Pool aus YAML-Text (akzeptiert beide Themenschreibweisen).
    pub fn parse(yaml: &str) -> Result<Self> {
        let raw: RawPool = serde_yaml::from_str(yaml).context("Ungültiges questions.yaml")?;
        let topics: BTreeMap<String, Topic> = raw
            .topics
            .into_iter()
            .map(|(name, raw_topic)| {
                let topic = match raw_topic {
                    RawTopic::Simple(questions) => Topic {
                        name: name.clone(),
                        description: String::new(),
                        questions,
                    },
                    RawTopic::Rich {
                        description,
                        questions,
                    } => Topic {
                        name: name.clone(),
                        description,
                        questions,
                    },
                };
                (name, topic)
            })
            .collect();

        // Leere Themen früh ablehnen – sonst gäbe es eine tote Session ohne Fragen.
        for topic in topics.values() {
            if topic.questions.is_empty() {
                return Err(anyhow!("Thema '{}' enthält keine Fragen.", topic.name));
            }
        }
        Ok(Self { topics })
    }

    /// Das Thema `name`; Fehler mit Liste verfügbarer Themen, falls unbekannt.
    pub fn topic(&self, name: &str) -> Result<&Topic> {
        self.topics.get(name).ok_or_else(|| {
            let available = self.topic_names().join(", ");
            anyhow!("Unbekanntes Thema '{name}'. Verfügbar: {available}")
        })
    }

    /// Alle Themen, alphabetisch nach Name sortiert.
    pub fn topics(&self) -> impl Iterator<Item = &Topic> {
        self.topics.values()
    }

    /// Die Namen aller Themen (alphabetisch).
    #[must_use]
    pub fn topic_names(&self) -> Vec<&str> {
        self.topics.keys().map(String::as_str).collect()
    }

    /// Anzahl der Themen.
    #[must_use]
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// Ob der Pool keine Themen enthält.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }
}

/// Der im Binary eingebettete Standard-Pool als YAML-Text.
#[must_use]
pub fn embedded_yaml() -> &'static str {
    include_str!("../questions.yaml")
}

/// Parst den eingebetteten Standard-Pool.
pub fn embedded() -> Result<QuestionPool> {
    QuestionPool::parse(embedded_yaml()).context("Eingebettetes questions.yaml ist ungültig")
}

/// Lädt den Pool aus der Konfigurationsdatei; legt sie beim ersten Mal an.
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
    QuestionPool::parse(&content).with_context(|| format!("Ungültiges YAML in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_parses_and_has_topics() {
        let pool = embedded().unwrap();
        for name in ["default", "logic", "perf", "api"] {
            let topic = pool.topic(name).unwrap();
            assert!(!topic.questions.is_empty(), "Thema {name} ist leer");
            assert!(
                !topic.description.is_empty(),
                "Thema {name} ohne Beschreibung"
            );
        }
    }

    #[test]
    fn accepts_both_simple_and_rich() {
        let yaml = "topics:\n  s:\n    - eins\n    - zwei\n  r:\n    description: Hallo\n    questions:\n      - drei\n";
        let pool = QuestionPool::parse(yaml).unwrap();
        assert_eq!(pool.topic("s").unwrap().questions.len(), 2);
        assert_eq!(pool.topic("s").unwrap().description, "");
        assert_eq!(pool.topic("r").unwrap().description, "Hallo");
        assert_eq!(pool.topic("r").unwrap().questions, vec!["drei".to_string()]);
    }

    #[test]
    fn empty_topic_is_rejected() {
        let err = QuestionPool::parse("topics:\n  leer:\n    questions: []\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("keine Fragen"));
    }

    #[test]
    fn unknown_topic_lists_available() {
        let pool = embedded().unwrap();
        let err = pool.topic("gibtsnicht").unwrap_err().to_string();
        assert!(err.contains("Unbekanntes Thema"));
        assert!(err.contains("default"));
    }

    #[test]
    fn topics_are_sorted() {
        let names = embedded().unwrap().topic_names().join(",");
        assert_eq!(names, "api,default,logic,perf");
    }

    #[test]
    fn load_or_init_writes_then_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("questions.yaml");
        let first = load_or_init_at(&path).unwrap();
        assert!(path.exists());
        let second = load_or_init_at(&path).unwrap();
        assert_eq!(first.len(), second.len());
        assert!(second.topic("default").is_ok());
    }
}
