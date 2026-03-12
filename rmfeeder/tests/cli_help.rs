use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn fresh_cmd() -> (Command, TempDir) {
    let home = tempfile::tempdir().expect("temp HOME");
    let mut cmd = Command::cargo_bin("rmfeeder").expect("rmfeeder binary");
    cmd.env("HOME", home.path());
    cmd.env_remove("XDG_CONFIG_HOME");
    (cmd, home)
}

#[test]
fn help_includes_sections_and_examples() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Build article/notes bundles as device-friendly PDFs",
        ))
        .stdout(predicate::str::contains(
            "Source Input (choose exactly one):",
        ))
        .stdout(predicate::str::contains("Output & Rendering:"))
        .stdout(predicate::str::contains("Summarization:"))
        .stdout(predicate::str::contains("YouTube Options:"))
        .stdout(predicate::str::contains("Maintenance:"))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "rmfeeder [OPTIONS] --markdown-dir <path>",
        ));
}

#[test]
fn missing_value_for_file_is_reported() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.arg("--file");
    cmd.assert().failure().stderr(
        predicate::str::contains("--file <path>").and(predicate::str::contains("required")),
    );
}

#[test]
fn conflicting_source_flags_fail() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.args(["--feeds", "--yt-watchlist"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("conflicting source flags"))
        .stderr(predicate::str::contains("--feeds"))
        .stderr(predicate::str::contains("--yt-watchlist"));
}

#[test]
fn invalid_page_size_shows_allowed_values() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.args(["--page-size", "not-a-target", "https://example.com"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--page-size must be one of"));
}

#[test]
fn feeds_file_implies_feeds_mode() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.args(["--feeds-file", "missing-feeds.opml"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse OPML"));
}

#[test]
fn no_source_prints_help_and_fails() {
    let (mut cmd, _home) = fresh_cmd();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains(
            "Source Input (choose exactly one):",
        ));
}
