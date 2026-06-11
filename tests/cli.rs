//! Integration tests against the built `rubberduck` binary.

use assert_cmd::Command;
use predicates::str::contains;

/// Builds a command with isolated config/data dirs and no colour.
fn duck(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("rubberduck").unwrap();
    cmd.env("RUBBERDUCK_CONFIG_DIR", home.join("config"))
        .env("RUBBERDUCK_DATA_DIR", home.join("data"))
        .env_remove("RUBBERDUCK_LANG")
        .env("NO_COLOR", "1");
    cmd
}

#[test]
fn prints_version() {
    let tmp = tempfile::tempdir().unwrap();
    // Assert against the compile-time crate version so this never needs a manual
    // bump on a release (the test crate shares the package's CARGO_PKG_VERSION).
    duck(tmp.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_lists_flags_and_commands() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--topic"))
        .stdout(contains("--no-anim"))
        .stdout(contains("--speed"))
        .stdout(contains("--theme"))
        .stdout(contains("--lang"))
        .stdout(contains("topics"))
        .stdout(contains("completions"))
        .stdout(contains("self"));
}

#[test]
fn self_help_lists_subcommands() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["self", "--help"])
        .assert()
        .success()
        .stdout(contains("update"))
        .stdout(contains("uninstall"));
}

#[test]
fn topics_lists_all_topics_in_english() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("topics")
        .assert()
        .success()
        .stdout(contains("Available topics"))
        .stdout(contains("default"))
        .stdout(contains("logic"))
        .stdout(contains("perf"))
        .stdout(contains("api"));
}

#[test]
fn topics_localizes_with_lang_flag() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["topics", "--lang", "de"])
        .assert()
        .success()
        .stdout(contains("Verfügbare Themen"));
}

#[test]
fn completions_generate_for_bash() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("rubberduck"));
}

#[test]
fn unknown_topic_fails_with_helpful_message() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["--topic", "does-not-exist"])
        .assert()
        .failure()
        .stderr(contains("Unknown topic"))
        .stderr(contains("default"));
}

#[test]
fn first_run_creates_english_questions_file() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path()).args(["--topic", "x"]).assert().failure();
    assert!(tmp.path().join("config").join("questions.en.yaml").exists());
}

#[test]
fn first_run_with_german_creates_german_questions_file() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["--lang", "de", "--topic", "x"])
        .assert()
        .failure();
    assert!(tmp.path().join("config").join("questions.de.yaml").exists());
}

#[test]
fn quiet_non_tty_session_runs_and_exits_cleanly() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["--topic", "default", "--quiet"])
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn man_page_renders() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("man")
        .assert()
        .success()
        .stdout(contains("rubberduck"));
}

#[test]
fn stats_json_is_valid_when_empty() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["stats", "--json"])
        .assert()
        .success()
        .stdout(contains("\"sessions\": 0"))
        .stdout(contains("\"per_topic\""));
}

#[test]
fn config_set_then_reset() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["config", "set", "theme", "ocean"])
        .assert()
        .success();
    duck(tmp.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("theme: ocean"));
    duck(tmp.path())
        .args(["config", "reset"])
        .assert()
        .success();
    duck(tmp.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("theme: classic"));
}

#[test]
fn config_set_rejects_bad_value() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["config", "set", "theme", "bogus"])
        .assert()
        .failure();
}

#[test]
fn help_lists_the_new_commands() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("themes"))
        .stdout(contains("tip"))
        .stdout(contains("history"))
        .stdout(contains("doctor"));
}

#[test]
fn topics_includes_the_added_topics() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("topics")
        .assert()
        .success()
        .stdout(contains("build"))
        .stdout(contains("concurrency"))
        .stdout(contains("memory"))
        .stdout(contains("network"));
}

#[test]
fn languages_lists_spanish() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("languages")
        .assert()
        .success()
        .stdout(contains("Español"));
}

#[test]
fn topics_localizes_to_spanish() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["topics", "--lang", "es"])
        .assert()
        .success()
        .stdout(contains("Temas disponibles"));
    assert!(tmp.path().join("config").join("questions.es.yaml").exists());
}

#[test]
fn themes_lists_every_theme() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("themes")
        .assert()
        .success()
        .stdout(contains("classic"))
        .stdout(contains("dracula"))
        .stdout(contains("nord"))
        .stdout(contains("solarized"));
}

#[test]
fn tip_prints_a_tip_and_seeds_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["tip", "--no-anim", "--quiet"])
        .assert()
        .success();
    assert!(tmp.path().join("config").join("tips.en.yaml").exists());
}

#[test]
fn tips_lists_and_localizes() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("tips")
        .assert()
        .success()
        .stdout(contains("Debugging tips"));
    duck(tmp.path())
        .args(["tips", "--lang", "es"])
        .assert()
        .success()
        .stdout(contains("Consejos de depuración"));
}

#[test]
fn doctor_reports_the_environment() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(contains("Version"))
        .stdout(contains("8 topics (en)"))
        .stdout(contains("tips (en)"));
}

#[test]
fn history_is_empty_initially() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .args(["history", "--json"])
        .assert()
        .success()
        .stdout(contains("\"total\": 0"))
        .stdout(contains("\"sessions\""));
}

#[test]
fn history_shows_seeded_sessions_newest_first() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(
        data.join("history.jsonl"),
        "{\"date\":\"2026-06-08\",\"topic\":\"logic\",\"asked\":7,\"answered\":6,\"total_seconds\":120,\"solved\":true,\"seconds_to_solution\":90}\n\
         {\"date\":\"2026-06-09\",\"topic\":\"memory\",\"asked\":7,\"answered\":3,\"total_seconds\":60,\"solved\":false,\"seconds_to_solution\":null}\n",
    )
    .unwrap();
    duck(tmp.path())
        .args(["history", "--json"])
        .assert()
        .success()
        .stdout(contains("\"total\": 2"))
        // Newest first: memory (2026-06-09) precedes logic (2026-06-08).
        .stdout(predicates::str::is_match(r#"(?s)"memory".*"logic""#).unwrap());
    // Animated/table view renders both topics.
    duck(tmp.path())
        .args(["history", "--no-anim"])
        .assert()
        .success()
        .stdout(contains("logic"))
        .stdout(contains("memory"));
}
