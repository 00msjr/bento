use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn test_fuzzy_match_exact() {
    assert_eq!(bento::fuzzy_match("git", "git"), 6); // contains rule (len * 2)
}

#[test]
fn test_fuzzy_match_partial() {
    assert!(bento::fuzzy_match("gt", "git") > 0);
}

#[test]
fn test_fuzzy_match_none() {
    assert_eq!(bento::fuzzy_match("xyz", "git"), 0);
}

#[test]
fn test_get_commands_from_path() {
    let dir = tempdir().unwrap();
    let bin_path = dir.path().join("dummy");
    fs::write(&bin_path, "#!/bin/sh\necho hi").unwrap();

    // make it executable
    let mut perms = fs::metadata(&bin_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&bin_path, perms).unwrap();

    // override PATH
    env::set_var("PATH", dir.path());

    let cmds = bento::get_commands();
    let names: Vec<String> = cmds.into_iter().map(|c| c.name).collect();
    assert!(names.contains(&"dummy".to_string()));
}

#[test]
fn test_cli_runs_without_query() {
    let mut cmd = Command::cargo_bin("bento").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🍱 Bento - Command Organizer"));
}

#[test]
fn test_cli_with_query_alias_filter() {
    let mut cmd = Command::cargo_bin("bento").unwrap();
    cmd.arg("--alias").arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("alias").or(predicate::str::contains("ls")));
}
