use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicates::str::contains(
        "A modern, ergonomic command runner",
    ));
}

#[test]
fn test_list_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path()).arg("list");
    // Should succeed even if only global scripts are found
    // The important thing is that it doesn't crash or error
    cmd.assert().success();
}

#[test]
fn test_which_nonexistent_script() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("which")
        .arg("nonexistent");
    cmd.assert().failure().code(127);
}

#[test]
fn test_run_dry_run() {
    let temp_dir = TempDir::new().unwrap();

    // Create a simple Python script
    let script_dir = temp_dir.path().join(".pyst");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("hello.py");
    fs::write(&script_path, "print('Hello, world!')").unwrap();

    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("run")
        .arg("--dry-run")
        .arg("hello");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Would execute"));
}

#[test]
fn test_project_root_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a .git directory
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();

    // Create a script in .pyst directory
    let script_dir = temp_dir.path().join(".pyst");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("test_script.py");
    fs::write(&script_path, "#!/usr/bin/env python3\nprint('test')").unwrap();

    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path()).arg("list");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("test_script"));
}

#[test]
fn test_json_output() {
    let temp_dir = TempDir::new().unwrap();

    // Create a script
    let script_dir = temp_dir.path().join(".pyst");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("test.py");
    fs::write(&script_path, "print('test')").unwrap();

    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("list")
        .arg("--format")
        .arg("json");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"name\": \"test\""));
}

#[test]
fn test_explicit_selector() {
    let temp_dir = TempDir::new().unwrap();

    // Create a script
    let script_dir = temp_dir.path().join(".pyst");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("test.py");
    fs::write(&script_path, "print('test')").unwrap();

    let mut cmd = Command::cargo_bin("pyst").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("which")
        .arg("project:test");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("test.py"));
}
