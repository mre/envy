use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that the path command works
#[test]
fn test_path_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("path")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Config.toml"));
}

/// Test that find command works for existing environment variable
#[test]
fn test_find_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("find")
        .arg("PATH")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // PATH should exist and contain some directory separator
    assert!(stdout.contains("/") || stdout.contains("\\"));
}

/// Test that find command handles non-existent variables
#[test]
fn test_find_nonexistent_variable() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("find")
        .arg("DEFINITELY_DOES_NOT_EXIST_12345")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not found"));
}

/// Test that load command fails for non-existent file
#[test]
fn test_load_nonexistent_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("load")
        .arg("/path/that/definitely/does/not/exist.env")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File does not exist"));
}

/// Test that load command works with valid .env file
#[test]
fn test_load_valid_env_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Create a simple .env file
    fs::write(
        &env_file,
        "TEST_VAR=test_value\n# This is a comment\nANOTHER_VAR=another_value",
    )
    .expect("Failed to write .env file");

    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("load")
        .arg(&env_file)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export TEST_VAR=test_value"));
    assert!(stdout.contains("export ANOTHER_VAR=another_value"));
    // Comments should be filtered out
    assert!(!stdout.contains("# This is a comment"));
}

/// Test hook generation for bash
#[test]
fn test_hook_bash() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("hook")
        .arg("bash")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_envy_hook"));
    assert!(stdout.contains("PROMPT_COMMAND"));
    assert!(stdout.contains("export bash"));
}

/// Test hook generation for unsupported shell
#[test]
fn test_hook_unsupported_shell() {
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .arg("hook")
        .arg("unsupported_shell")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("currently not supported"));
}
