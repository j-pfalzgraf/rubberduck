//! Loading and managing the debugging-tips pool.
//!
//! Tips are short, language-specific one-liners shown by `rubberduck tip`
//! (a single random one) and `rubberduck tips` (the whole list), and as a
//! gentle closing nudge after an interactive session. Like the question pool,
//! each language ships a bundled default file (`tips.<code>.yaml`) and a
//! user-editable copy is seeded under `<config>/tips.<code>.yaml` on first run,
//! so teams can curate their own. The format is intentionally minimal:
//!
//! ```yaml
//! tips:
//!   - "Explain the problem out loud, line by line."
//!   - "Read the error message again, slowly."
//! ```

use crate::i18n::Lang;
use crate::paths;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Raw deserialization shape of a tips file.
#[derive(Debug, Deserialize)]
struct RawTips {
    tips: Vec<String>,
}

/// A non-empty, ordered collection of debugging tips for one language.
#[derive(Debug, Clone)]
pub struct TipPool {
    tips: Vec<String>,
}

impl TipPool {
    /// Parses a pool from YAML text. Blank/whitespace-only entries are dropped and
    /// an entirely empty pool is rejected, so callers can rely on
    /// [`TipPool::random`] always returning a non-empty tip.
    pub fn parse(yaml: &str) -> Result<Self> {
        let raw: RawTips = serde_yaml::from_str(yaml).context("Invalid tips YAML")?;
        let tips: Vec<String> = raw
            .tips
            .into_iter()
            .filter(|t| !t.trim().is_empty())
            .collect();
        if tips.is_empty() {
            return Err(anyhow!("The tips file contains no tips."));
        }
        Ok(Self { tips })
    }

    /// All tips, in file order.
    #[must_use]
    pub fn all(&self) -> &[String] {
        &self.tips
    }

    /// Number of tips.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tips.len()
    }

    /// Whether the pool has no tips (never true for a parsed pool).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tips.is_empty()
    }

    /// The tip at `seed` modulo the pool size — a deterministic selector that
    /// keeps [`TipPool::random`] testable.
    #[must_use]
    pub fn pick(&self, seed: usize) -> &str {
        &self.tips[seed % self.tips.len()]
    }

    /// A pseudo-random tip, seeded from the current time.
    ///
    /// No RNG dependency is pulled in for something this trivial; the
    /// sub-second clock jitter is plenty of entropy for "show me a tip".
    #[must_use]
    pub fn random(&self) -> &str {
        self.pick(time_seed())
    }
}

/// A time-derived seed (sub-second nanoseconds); `0` if the clock is unavailable.
fn time_seed() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
}

/// The bundled default tips for `lang`, as embedded YAML text.
#[must_use]
pub fn embedded_yaml(lang: Lang) -> &'static str {
    match lang {
        Lang::English => include_str!("../tips.en.yaml"),
        Lang::German => include_str!("../tips.de.yaml"),
        Lang::French => include_str!("../tips.fr.yaml"),
        Lang::Spanish => include_str!("../tips.es.yaml"),
    }
}

/// Parses the bundled default tips for `lang`.
pub fn embedded(lang: Lang) -> Result<TipPool> {
    TipPool::parse(embedded_yaml(lang)).context("Bundled tips are invalid")
}

/// Path of the per-language tips file, e.g. `<config>/tips.en.yaml`.
fn tips_file(lang: Lang) -> Result<PathBuf> {
    Ok(paths::config_dir()?.join(format!("tips.{}.yaml", lang.code())))
}

/// Loads the tips for `lang`; creates the file on first run so teams can edit it.
pub fn load_or_init(lang: Lang) -> Result<TipPool> {
    load_or_init_at(&tips_file(lang)?, lang)
}

/// Like [`load_or_init`], but with an explicit path (for tests).
fn load_or_init_at(path: &Path, lang: Lang) -> Result<TipPool> {
    let content = paths::read_or_init(path, embedded_yaml(lang))?;
    TipPool::parse(&content).with_context(|| format!("Invalid YAML in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_parses_for_every_language_with_matching_counts() {
        let reference = embedded(Lang::English).unwrap().len();
        assert!(reference >= 10, "expected a decent set of tips");
        for lang in Lang::ALL {
            let pool = embedded(lang).unwrap();
            assert!(!pool.is_empty(), "{lang} has no tips");
            assert_eq!(pool.len(), reference, "tip count differs in {lang}");
        }
    }

    #[test]
    fn pick_wraps_around_and_is_deterministic() {
        let pool = embedded(Lang::English).unwrap();
        assert_eq!(pool.pick(0), pool.pick(pool.len()));
        assert_eq!(pool.pick(1), pool.pick(pool.len() + 1));
        assert_eq!(pool.pick(0), &pool.all()[0]);
    }

    #[test]
    fn empty_pool_is_rejected() {
        assert!(TipPool::parse("tips: []\n").is_err());
        assert!(TipPool::parse("tips:\n  - \"  \"\n").is_err());
    }

    #[test]
    fn blank_entries_are_dropped_but_real_ones_kept() {
        let pool =
            TipPool::parse("tips:\n  - \"Real tip.\"\n  - \"   \"\n  - \"Another.\"\n").unwrap();
        assert_eq!(pool.len(), 2);
        assert!(pool.all().iter().all(|t| !t.trim().is_empty()));
    }

    #[test]
    fn random_returns_a_known_tip() {
        let pool = embedded(Lang::English).unwrap();
        let tip = pool.random();
        assert!(pool.all().iter().any(|t| t == tip));
    }

    #[test]
    fn load_or_init_writes_then_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tips.en.yaml");
        let first = load_or_init_at(&path, Lang::English).unwrap();
        assert!(path.exists());
        let second = load_or_init_at(&path, Lang::English).unwrap();
        assert_eq!(first.len(), second.len());
    }
}
