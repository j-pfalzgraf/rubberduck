//! Integrationstests gegen das gebaute `rubberduck`-Binary.

use assert_cmd::Command;
use predicates::str::contains;

/// Baut einen Command mit isolierten Config/Daten-Verzeichnissen.
fn duck(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("rubberduck").unwrap();
    cmd.env("RUBBERDUCK_CONFIG_DIR", home.join("config"))
        .env("RUBBERDUCK_DATA_DIR", home.join("data"));
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
fn help_lists_flags_and_self() {
    let tmp = tempfile::tempdir().unwrap();
    duck(tmp.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--topic"))
        .stdout(contains("--quiet"))
        .stdout(contains("--log"))
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
    // Ungültiges Thema bricht ab, nachdem die Default-Datei angelegt wurde.
    duck(tmp.path()).args(["--topic", "x"]).assert().failure();
    assert!(tmp.path().join("config").join("questions.yaml").exists());
}
