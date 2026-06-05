//! Integrationstests gegen das gebaute `rubberduck`-Binary.

use assert_cmd::Command;
use predicates::str::contains;

/// Baut einen Command mit isolierten Config/Daten-Verzeichnissen und ohne Farbe.
fn duck(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("rubberduck").unwrap();
    cmd.env("RUBBERDUCK_CONFIG_DIR", home.join("config"))
        .env("RUBBERDUCK_DATA_DIR", home.join("data"))
        .env("NO_COLOR", "1");
    cmd
}

#[test]
fn prints_version() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("0.1.0"));
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
fn topics_lists_all_topics() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("topics")
        .assert()
        .success()
        .stdout(contains("default"))
        .stdout(contains("logic"))
        .stdout(contains("perf"))
        .stdout(contains("api"));
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
        .args(["--topic", "gibtsnicht"])
        .assert()
        .failure()
        .stderr(contains("Unbekanntes Thema"))
        .stderr(contains("default"));
}

#[test]
fn first_run_creates_questions_file() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path()).args(["--topic", "x"]).assert().failure();
    assert!(tmp.path().join("config").join("questions.yaml").exists());
}

#[test]
fn quiet_non_tty_session_runs_and_exits_cleanly() {
    let tmp = tempfile::tempdir().unwrap();
    // Kein TTY (assert_cmd) + leere Eingabe: Session begrüßt und endet sauber.
    duck(tmp.path())
        .args(["--topic", "default", "--quiet"])
        .write_stdin("")
        .assert()
        .success();
}
