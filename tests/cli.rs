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

    assert!(allow_output.status.success());

    // Test JSON export
    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("export")
        .arg("json")
        .output()
        .expect("Failed to execute export json command");

    assert!(output.status.success());
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

    assert!(output.status.success());
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

    assert!(allow_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_envy"))
        .current_dir(&temp_dir)
        .arg("export")
        .arg("json")
        .output()
        .expect("Failed to execute export json command");

    assert!(output.status.success());
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

/// Test bash support functionality (requires bash-support feature)
#[cfg(all(feature = "bash-support", unix))]
mod bash_tests {
    use super::*;

    /// Test that .envrc files can be processed when bash support is enabled
    #[test]
    fn test_envrc_basic_functionality() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let envrc_file = temp_dir.path().join(".envrc");

        // Create a simple .envrc file
        fs::write(
            &envrc_file,
            "export TEST_VAR=hello\nexport ANOTHER_VAR=world\n",
        )
        .expect("Failed to write .envrc file");

        // Allow the .envrc file
        let allow_output = Command::new(env!("CARGO_BIN_EXE_envy"))
            .current_dir(&temp_dir)
            .arg("allow")
            .arg(&envrc_file)
            .output()
            .expect("Failed to execute allow command");

        assert!(allow_output.status.success());

        // Test bash export
        let output = Command::new(env!("CARGO_BIN_EXE_envy"))
            .current_dir(&temp_dir)
            .arg("export")
            .arg("bash")
            .output()
            .expect("Failed to execute export bash command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("export TEST_VAR=hello"));
        assert!(stdout.contains("export ANOTHER_VAR=world"));
    }

    /// Test PATH_add function in .envrc files
    #[test]
    fn test_envrc_path_add() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let envrc_file = temp_dir.path().join(".envrc");
        let bin_dir = temp_dir.path().join("bin");

        // Create bin directory
        fs::create_dir(&bin_dir).expect("Failed to create bin dir");

        // Create .envrc with PATH_add
        fs::write(
            &envrc_file,
            format!("PATH_add {}", bin_dir.to_string_lossy()),
        )
        .expect("Failed to write .envrc file");

        let allow_output = Command::new(env!("CARGO_BIN_EXE_envy"))
            .current_dir(&temp_dir)
            .arg("allow")
            .arg(&envrc_file)
            .output()
            .expect("Failed to execute allow command");

        assert!(allow_output.status.success());

        // Test JSON export to verify PATH was modified
        let output = Command::new(env!("CARGO_BIN_EXE_envy"))
            .current_dir(&temp_dir)
            .arg("export")
            .arg("json")
            .output()
            .expect("Failed to execute export json command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");

        if let Some(path_value) = json.get("PATH").and_then(|v| v.as_str()) {
            assert!(path_value.contains(&bin_dir.to_string_lossy().to_string()));
        }
    }

    /// Test that .envrc files are only executed when allowed
    #[test]
    fn test_envrc_security_model() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let envrc_file = temp_dir.path().join(".envrc");

        // Create .envrc file without allowing it
        fs::write(&envrc_file, "export SECURITY_TEST=should_not_appear")
            .expect("Failed to write .envrc file");

        // Try to export without allowing - should not execute the .envrc
        let output = Command::new(env!("CARGO_BIN_EXE_envy"))
            .current_dir(&temp_dir)
            .arg("export")
            .arg("json")
            .output()
            .expect("Failed to execute export json command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
        let obj = json.as_object().unwrap();

        // Should not contain the variable from .envrc since it wasn't allowed
        assert!(!obj.contains_key("SECURITY_TEST"));
    }
}
