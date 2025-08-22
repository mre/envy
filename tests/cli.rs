use serde_json::Value;
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

/// Test JSON export with valid .env file
#[test]
fn test_export_json_with_env_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Create a .env file with various formats
    fs::write(
        &env_file,
        "export TEST_VAR=hello_world\nAPI_KEY=secret123\n# This is a comment\nDATABASE_URL=postgres://localhost/test\nexport ANOTHER_VAR=with_export",
    )
    .expect("Failed to write .env file");

    // Allow the env file
    let allow_output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("allow")
        .arg(&env_file)
        .output()
        .expect("Failed to execute allow command");

    if !allow_output.status.success() {
        eprintln!("Allow command failed with exit code: {:?}", allow_output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&allow_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&allow_output.stderr));
        panic!("allow command failed");
    }

    // Test JSON export
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("export")
        .arg("json")
        .output()
        .expect("Failed to execute export json command");

    if !output.status.success() {
        eprintln!("Export json command failed with exit code: {:?}", output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("export json command failed");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the JSON output
    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Verify it's a JSON object
    assert!(json.is_object());
    let obj = json.as_object().unwrap();

    // Check that our variables are present
    assert_eq!(
        obj.get("TEST_VAR").unwrap().as_str().unwrap(),
        "hello_world"
    );
    assert_eq!(obj.get("API_KEY").unwrap().as_str().unwrap(), "secret123");
    assert_eq!(
        obj.get("DATABASE_URL").unwrap().as_str().unwrap(),
        "postgres://localhost/test"
    );
    assert_eq!(
        obj.get("ANOTHER_VAR").unwrap().as_str().unwrap(),
        "with_export"
    );
}

/// Test JSON export returns empty object when no env files match
#[test]
fn test_export_json_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("export")
        .arg("json")
        .output()
        .expect("Failed to execute export json command");

    if !output.status.success() {
        eprintln!("Command failed with exit code: {:?}", output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("export json command failed");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the JSON output
    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

    // Should be an empty object
    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert!(obj.is_empty());
}

/// Test JSON export produces valid JSON that can be processed by other tools
#[test]
fn test_export_json_format_compatibility() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let env_file = temp_dir.path().join(".env");

    // Create a .env file with edge cases
    fs::write(
        &env_file,
        "SIMPLE_VAR=value\nVAR_WITH_SPACES=value with spaces\nVAR_WITH_QUOTES=\"quoted value\"\nVAR_WITH_EQUALS=key=value=more\nEMPTY_VAR=",
    )
    .expect("Failed to write .env file");

    let allow_output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("allow")
        .arg(&env_file)
        .output()
        .expect("Failed to execute allow command");

    if !allow_output.status.success() {
        eprintln!("Allow command failed with exit code: {:?}", allow_output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&allow_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&allow_output.stderr));
        panic!("allow command failed");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("export")
        .arg("json")
        .output()
        .expect("Failed to execute export json command");

    if !output.status.success() {
        eprintln!("Export json command failed with exit code: {:?}", output.status.code());
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("export json command failed");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    let obj = json.as_object().unwrap();

    // Test various edge cases
    assert_eq!(obj.get("SIMPLE_VAR").unwrap().as_str().unwrap(), "value");
    assert_eq!(
        obj.get("VAR_WITH_SPACES").unwrap().as_str().unwrap(),
        "value with spaces"
    );
    assert_eq!(
        obj.get("VAR_WITH_QUOTES").unwrap().as_str().unwrap(),
        "\"quoted value\""
    );
    assert_eq!(
        obj.get("VAR_WITH_EQUALS").unwrap().as_str().unwrap(),
        "key=value=more"
    );
    assert_eq!(obj.get("EMPTY_VAR").unwrap().as_str().unwrap(), "");

    // Ensure the JSON is compact (no pretty printing)
    assert!(
        !stdout.contains('\n') || stdout.trim_end().chars().filter(|&c| c == '\n').count() <= 1
    );
}
