//! Loading and managing the question pool.
//!
//! The pool maps each topic name to a [`Topic`] (description + questions). The
//! YAML format supports two shapes per topic — a lean list (backwards
//! compatible) and a rich `{description, questions}` form:
//!
//! ```yaml
//! topics:
//!   lean:
//!     - "Just a list of questions."
//!   rich:
//!     description: "With a description for the topic picker."
//!     questions:
//!       - "First question?"
//! ```
//!
//! Each language has its own bundled default file (`questions.en.yaml`,
//! `questions.de.yaml`, `questions.fr.yaml`, `questions.es.yaml`) and its own
//! user file (`<config>/questions.<code>.yaml`).

use crate::i18n::Lang;
use crate::paths;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Name of the default topic (stable across languages).
pub const DEFAULT_TOPIC: &str = "default";

/// A topic with a description and its questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    /// Technical name (the key; reachable via `--topic`).
    pub name: String,
    /// Short description (for the topic picker); may be empty.
    pub description: String,
    /// The questions, in order.
    pub questions: Vec<String>,
}

/// The whole question pool, sorted by topic name.
#[derive(Debug, Clone)]
pub struct QuestionPool {
    topics: BTreeMap<String, Topic>,
}

/// Raw form of a topic: a lean list **or** the rich form (serde-untagged).
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
    /// Parses a pool from YAML text (accepts both topic shapes).
    pub fn parse(yaml: &str) -> Result<Self> {
        let raw: RawPool = serde_yaml::from_str(yaml).context("Invalid questions YAML")?;
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

        // Reject empty topics early — otherwise they'd be a dead, question-less session.
        for topic in topics.values() {
            if topic.questions.is_empty() {
                return Err(anyhow!("Topic '{}' has no questions.", topic.name));
            }
        }
        Ok(Self { topics })
    }

    /// The topic `name`; error listing available topics (and a "did you mean?"
    /// suggestion) if unknown.
    pub fn topic(&self, name: &str) -> Result<&Topic> {
        self.topics.get(name).ok_or_else(|| {
            let names = self.topic_names();
            let suggestion = crate::util::closest(name, &names, 2)
                .map(|s| format!(" Did you mean '{s}'?"))
                .unwrap_or_default();
            anyhow!(
                "Unknown topic '{name}'.{suggestion} Available: {}",
                names.join(", ")
            )
        })
    }

    /// All topics, alphabetically by name.
    pub fn topics(&self) -> impl Iterator<Item = &Topic> {
        self.topics.values()
    }

    /// The names of all topics (alphabetical).
    #[must_use]
    pub fn topic_names(&self) -> Vec<&str> {
        self.topics.keys().map(String::as_str).collect()
    }

    /// Number of topics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// Whether the pool has no topics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }
}

/// The bundled default pool for `lang`, as embedded YAML text.
#[must_use]
pub fn embedded_yaml(lang: Lang) -> &'static str {
    match lang {
        Lang::English => include_str!("../questions.en.yaml"),
        Lang::German => include_str!("../questions.de.yaml"),
        Lang::French => include_str!("../questions.fr.yaml"),
        Lang::Spanish => include_str!("../questions.es.yaml"),
    }
}

/// Parses the bundled default pool for `lang`.
pub fn embedded(lang: Lang) -> Result<QuestionPool> {
    QuestionPool::parse(embedded_yaml(lang)).context("Bundled question pool is invalid")
}

/// Path of the per-language question file, e.g. `<config>/questions.en.yaml`.
fn questions_file(lang: Lang) -> Result<PathBuf> {
    Ok(paths::config_dir()?.join(format!("questions.{}.yaml", lang.code())))
}

/// Loads the pool for `lang`; creates the file on first run so teams can edit it.
pub fn load_or_init(lang: Lang) -> Result<QuestionPool> {
    load_or_init_at(&questions_file(lang)?, lang)
}

/// Like [`load_or_init`], but with an explicit path (for tests).
fn load_or_init_at(path: &Path, lang: Lang) -> Result<QuestionPool> {
    let content = paths::read_or_init(path, embedded_yaml(lang))?;
    QuestionPool::parse(&content).with_context(|| format!("Invalid YAML in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_parses_for_every_language() {
        for lang in Lang::ALL {
            let pool = embedded(lang).unwrap();
            for name in ["default", "logic", "perf", "api"] {
                let topic = pool.topic(name).unwrap();
                assert!(!topic.questions.is_empty(), "{name} empty in {lang}");
                assert!(
                    !topic.description.is_empty(),
                    "{name} has no description in {lang}"
                );
            }
        }
    }

    #[test]
    fn accepts_both_simple_and_rich() {
        let yaml = "topics:\n  s:\n    - one\n    - two\n  r:\n    description: Hi\n    questions:\n      - three\n";
        let pool = QuestionPool::parse(yaml).unwrap();
        assert_eq!(pool.topic("s").unwrap().questions.len(), 2);
        assert_eq!(pool.topic("s").unwrap().description, "");
        assert_eq!(pool.topic("r").unwrap().description, "Hi");
        assert_eq!(
            pool.topic("r").unwrap().questions,
            vec!["three".to_string()]
        );
    }

    #[test]
    fn empty_topic_is_rejected() {
        let err = QuestionPool::parse("topics:\n  empty:\n    questions: []\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no questions"));
    }

    #[test]
    fn unknown_topic_lists_available() {
        let pool = embedded(Lang::English).unwrap();
        let err = pool.topic("zzzz").unwrap_err().to_string();
        assert!(err.contains("Unknown topic"));
        assert!(err.contains("default"));
    }

    #[test]
    fn unknown_topic_suggests_closest() {
        let pool = embedded(Lang::English).unwrap();
        let err = pool.topic("logc").unwrap_err().to_string();
        assert!(err.contains("Did you mean 'logic'?"), "got: {err}");
    }

    #[test]
    fn topics_are_sorted() {
        let names = embedded(Lang::English).unwrap().topic_names().join(",");
        assert_eq!(
            names,
            "api,build,concurrency,default,logic,memory,network,perf"
        );
    }

    #[test]
    fn every_language_ships_the_same_topic_set() {
        let reference: Vec<String> = embedded(Lang::English)
            .unwrap()
            .topic_names()
            .iter()
            .map(ToString::to_string)
            .collect();
        for lang in Lang::ALL {
            let names: Vec<String> = embedded(lang)
                .unwrap()
                .topic_names()
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(names, reference, "topic set differs in {lang}");
        }
    }

    #[test]
    fn load_or_init_writes_then_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("questions.en.yaml");
        let first = load_or_init_at(&path, Lang::English).unwrap();
        assert!(path.exists());
        let second = load_or_init_at(&path, Lang::English).unwrap();
        assert_eq!(first.len(), second.len());
        assert!(second.topic("default").is_ok());
    }
}
