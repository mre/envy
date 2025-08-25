//! Bash environment file support for envy
//!
//! This module provides support for executing `.envrc` files (bash scripts)
//! similar to direnv, enabling full bash scripting capabilities for environment
//! variable management.
//!
//! # Features
//!
//! - Execute `.envrc` files as bash scripts
//! - Support for direnv stdlib functions (PATH_add, dotenv, etc.)
//! - Subprocess-based execution for security and compatibility
//! - Environment variable extraction and export
//!
//! # Security
//!
//! Like direnv, `.envrc` files must be explicitly allowed before execution to
//! prevent malicious code execution.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Execute a .envrc file and extract environment variables
///
/// This function runs the .envrc file in a bash subprocess and captures
/// the resulting environment variables.
///
/// # Arguments
///
/// * `envrc_path` - Path to the .envrc file to execute
/// * `current_dir` - Working directory for script execution
///
/// # Returns
///
/// A HashMap containing the environment variables set by the script
pub fn process_envrc(envrc_path: &Path, current_dir: &Path) -> Result<HashMap<String, String>> {
    // Create a bash script that:
    // 1. Sources the .envrc file
    // 2. Prints all environment variables in a parseable format
    let script = create_extraction_script(envrc_path)?;

    // Execute the script and capture environment variables
    let output = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .current_dir(current_dir)
        .output()
        .context("Failed to execute bash subprocess")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bash script execution failed: {}", stderr);
    }

    // Parse environment variables from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_env_output(&stdout)
}

/// Create a bash script for environment variable extraction
fn create_extraction_script(envrc_path: &Path) -> Result<String> {
    let envrc_str = envrc_path
        .to_str()
        .context("Invalid path encoding for .envrc file")?;

    // Create a script that includes direnv stdlib functions and executes the .envrc
    Ok(format!(
        r#"
# Direnv stdlib functions (simplified versions)
PATH_add() {{
    if [[ ":$PATH:" != *":$1:"* ]]; then
        export PATH="$1:$PATH"
    fi
}}

path_add() {{
    local var_name="$1"
    local new_path="$2"
    local current_value
    current_value=$(eval echo \$$var_name)
    if [[ ":$current_value:" != *":$new_path:"* ]]; then
        export "$var_name"="$new_path:$current_value"
    fi
}}

dotenv() {{
    local env_file="${{1:-.env}}"
    if [[ -f "$env_file" ]]; then
        set -o allexport
        source "$env_file"
        set +o allexport
    fi
}}

dotenv_if_exists() {{
    local env_file="${{1:-.env}}"
    if [[ -f "$env_file" ]]; then
        dotenv "$env_file"
    fi
}}

expand_path() {{
    echo "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
}}

has() {{
    command -v "$1" >/dev/null 2>&1
}}

find_up() {{
    local file="$1"
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/$file" ]]; then
            echo "$dir/$file"
            return 0
        fi
        dir=$(dirname "$dir")
    done
    return 1
}}

source_up() {{
    local envrc_file
    envrc_file=$(find_up .envrc)
    if [[ -n "$envrc_file" && "$envrc_file" != "$PWD/.envrc" ]]; then
        source "$envrc_file"
    fi
}}

source_env() {{
    if [[ -f "$1" ]]; then
        source "$1"
    fi
}}

# Layout functions (basic versions)
layout() {{
    case "$1" in
        python|python3)
            if has python3; then
                export VIRTUAL_ENV="$PWD/.venv"
                export PATH="$VIRTUAL_ENV/bin:$PATH"
            fi
            ;;
        node|nodejs)
            if [[ -d node_modules/.bin ]]; then
                PATH_add node_modules/.bin
            fi
            ;;
        go)
            if [[ -f go.mod ]]; then
                export GOPATH="$PWD/.go"
                PATH_add "$GOPATH/bin"
            fi
            ;;
    esac
}}

use() {{
    # Simplified version - in real implementation would handle version managers
    case "$1" in
        python|python3)
            layout python3
            ;;
        node|nodejs)
            layout node
            ;;
        go)
            layout go
            ;;
    esac
}}

# Store initial environment in a temporary file
initial_env_file=$(mktemp)
env > "$initial_env_file"

# Source the .envrc file
source "{}"

# Output environment variables that changed
echo "=== ENVY_ENV_START ==="
while IFS='=' read -r key value; do
    # Check if this variable existed before and had a different value
    if ! grep -q "^$key=" "$initial_env_file" || [ "$(grep "^$key=" "$initial_env_file" | cut -d'=' -f2-)" != "$value" ]; then
        printf '%s=%s\n' "$key" "$value"
    fi
done < <(env)
echo "=== ENVY_ENV_END ==="

# Clean up
rm -f "$initial_env_file"
"#,
        envrc_str
    ))
}

/// Parse environment variables from bash script output
fn parse_env_output(output: &str) -> Result<HashMap<String, String>> {
    let mut env_vars = HashMap::new();
    let mut in_env_section = false;

    for line in output.lines() {
        let line = line.trim();

        if line == "=== ENVY_ENV_START ===" {
            in_env_section = true;
            continue;
        }

        if line == "=== ENVY_ENV_END ===" {
            break;
        }

        if in_env_section && !line.is_empty() {
            if let Some((key, value)) = line.split_once('=') {
                env_vars.insert(key.to_string(), value.to_string());
            }
        }
    }

    Ok(env_vars)
}

/// Check if a file is an .envrc file
pub fn is_envrc_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == ".envrc")
        .unwrap_or(false)
}

/// Validate that bash is available on the system
pub fn is_bash_available() -> bool {
    Command::new("bash")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_envrc_file() {
        assert!(is_envrc_file(Path::new(".envrc")));
        assert!(is_envrc_file(Path::new("/path/to/.envrc")));
        assert!(!is_envrc_file(Path::new(".env")));
        assert!(!is_envrc_file(Path::new("envrc")));
    }

    #[test]
    fn test_bash_available() {
        // This should pass on most Unix systems
        assert!(is_bash_available());
    }

    #[test]
    fn test_parse_env_output() {
        let output = r#"
Some other output
=== ENVY_ENV_START ===
FOO=bar
BAZ=qux with spaces
=== ENVY_ENV_END ===
More output
"#;

        let result = parse_env_output(output).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(result.get("BAZ"), Some(&"qux with spaces".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn test_process_envrc_basic() {
        let temp_dir = TempDir::new().unwrap();
        let envrc_path = temp_dir.path().join(".envrc");

        fs::write(
            &envrc_path,
            "export TEST_VAR=hello\nexport ANOTHER_VAR=world",
        )
        .unwrap();

        let result = process_envrc(&envrc_path, temp_dir.path()).unwrap();

        assert_eq!(result.get("TEST_VAR"), Some(&"hello".to_string()));
        assert_eq!(result.get("ANOTHER_VAR"), Some(&"world".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn test_path_add_function() {
        let temp_dir = TempDir::new().unwrap();
        let envrc_path = temp_dir.path().join(".envrc");

        fs::write(&envrc_path, "PATH_add /custom/bin").unwrap();

        let result = process_envrc(&envrc_path, temp_dir.path()).unwrap();

        assert!(result.get("PATH").unwrap().starts_with("/custom/bin:"));
    }
}
