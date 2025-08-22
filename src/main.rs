use anyhow::{Context, Result, anyhow};
use serde_json::Value;

mod hooks;
mod opt;
mod settings;

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process;
use std::{env::current_dir, fs};

use directories::BaseDirs;
use hooks::zsh::Zsh;
use opt::{Command, Envy};
use settings::Settings;

/// Get the path to the envy config file
///
/// Where the config file is stored depends on the platform:
///
/// Check the following locations:
/// - Linux: `~/.config/envy/Config.toml`
/// - macOS: `~/Library/Application Support/Envy/Config.toml`
/// - Windows: `%APPDATA%/envy/Config.toml`
fn config_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Cannot get base directories")?;
    Ok(base_dirs.config_dir().join("envy").join("Config.toml"))
}

fn main() -> Result<()> {
    let opt = Envy::parse();
    match opt.cmd {
        Command::Allow { env_file } => allow(env_file),
        Command::Deny { env_file } => deny(env_file),
        Command::Edit => edit(),
        Command::Export { shell } => export(shell),
        Command::Find { variable } => find(variable),
        Command::Hook { shell } => hook(shell),
        Command::Load { env_file } => load(env_file),
        Command::Path => print_config_path(),
        Command::Show => show(),
    }
}

/// Export all environment variables from the env file into the current shell
///
/// The command is called `load` because `source` is reserved for potentially
/// showing the source of an env variable in the future.
fn load(env_file: PathBuf) -> Result<()> {
    anyhow::ensure!(
        env_file.exists(),
        "File does not exist: {}",
        env_file.display()
    );
    source(env_file)
}

/// Get all environment variables currently set and print the value of a given
/// variable
fn find(variable: String) -> Result<()> {
    match std::env::var(&variable) {
        Ok(value) => println!("{value}"),
        Err(_) => println!("Variable {variable} not found"),
    }
    Ok(())
}

/// Remove the given env file from the list of allowed paths.
///
/// This will prevent the env file from being loaded automatically
/// when entering the directory where the env file is located.
fn deny(env_file: PathBuf) -> Result<()> {
    anyhow::ensure!(
        env_file.exists(),
        "File does not exist: {}",
        env_file.display()
    );
    let mut settings = Settings::load(config_path()?)?;
    let env_file = env_file.canonicalize()?;
    settings.remove_env(env_file);
    Settings::save(config_path()?, settings)
}

/// Add the current directory to the list of allowed paths.
///
/// The `.env` file will be loaded automatically on directory enter.
fn allow(env_file: PathBuf) -> Result<()> {
    anyhow::ensure!(
        env_file.exists(),
        "File does not exist: {}",
        env_file.display()
    );
    let mut settings = Settings::load(config_path()?)?;
    let env_file = env_file.canonicalize()?;
    settings.add_env(env_file);
    Settings::save(config_path()?, settings)
}

/// Open the given file in the user's preferred editor
///
/// This function will read the `EDITOR` environment variable to determine which
/// editor to use. If the variable is not set, it will return an error.
pub fn open_editor(filename: &str) -> Result<std::process::ExitStatus> {
    let editor_name = std::env::var("EDITOR")?;
    let mut editor = process::Command::new(editor_name).arg(filename).spawn()?;
    Ok(editor.wait()?)
}

/// Open the envy config file in the user's preferred editor
///
/// This function will read the `EDITOR` environment variable to determine which
/// editor to use. If the variable is not set, it will return an error.
fn edit() -> Result<()> {
    let config = config_path()?;
    open_editor(&config.to_string_lossy())?;
    Ok(())
}

/// Print the shell hook for the given shell
///
/// The hook can be used to automatically load the envy environment variables
/// when entering a directory with an env file.
fn hook(shell: String) -> Result<()> {
    let hook = match shell.as_ref() {
        "bash" => hooks::bash::Bash::hook()?,
        "fish" => hooks::fish::Fish::hook()?,
        "zsh" => Zsh::hook()?,
        _ => return Err(anyhow!("{} is currently not supported", shell)),
    };
    println!("{hook}");
    Ok(())
}

/// Get all environment variables from the given file
///
/// This function reads the file line by line, ignoring comments (lines starting
/// with `#`), and returns a vector of strings containing the environment
/// variables in the format `KEY=value`.
fn get_env_vars_from_file(env: &Path) -> Result<Vec<String>> {
    let env = fs::read_to_string(env).context("Cannot read env file")?;
    Ok(env
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(String::from)
        .collect())
}

/// Split environment variable string into key-value pair
///
/// Handles formats:
/// - `KEY=value`
/// - `export KEY=value`
fn split_env_var(var: &str) -> Option<(String, String)> {
    // Remove "export " prefix if present
    let trimmed = var.trim();
    let var = trimmed.strip_prefix("export ").unwrap_or(trimmed);

    // Split on first '=' to get key and value
    var.split_once('=')
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
}

/// Print the environment variables loaded from the config file and the env
/// files for the current directory.
fn show() -> Result<()> {
    let settings = Settings::load(config_path()?)?;
    let dir = current_dir()?;
    let env_files = settings.matching_env_files(&dir);

    for file in &env_files {
        println!("Loaded from `{}`:", file.display());
        let vars = get_env_vars_from_file(file).context("Cannot read env file")?;
        for var in vars {
            println!("{var}");
        }
        println!();
    }

    match settings.matching_patterns(&dir) {
        Some(env) => println!("{}", env.join("\n")),
        None => {
            if env_files.is_empty() {
                println!(
                    "{}: envy found no pattern matches for this directory",
                    dir.display()
                );
            }
        }
    };

    Ok(())
}

/// Print the path to the envy config file
fn print_config_path() -> Result<()> {
    let path = config_path().context("Cannot read config path")?;
    println!("{}", path.display());
    Ok(())
}

/// Source the given env file
///
/// This will print the commands to stdout that need to be executed to source
/// the file
///
/// This is used by the `envy export` command to source all matching env files
/// and by `envy load` to source the given env file directly (for the current
/// session)
fn source(env_file: PathBuf) -> Result<()> {
    for var in get_env_vars_from_file(&env_file)? {
        match var {
            var if var.starts_with("export ") => {
                println!("{var}");
            }
            var => {
                println!("export {var}");
            }
        }
    }
    Ok(())
}

/// Export environment variables for the current shell
fn export(shell: String) -> Result<()> {
    let settings = Settings::load(config_path()?)?;
    let current_dir = current_dir()?;

    // Collect all environment variables from patterns and files
    let mut all_env_vars = Vec::new();

    // Add variables from patterns
    if let Some(patterns) = settings.matching_patterns(&current_dir) {
        all_env_vars.extend(patterns);
    }

    // Add variables from env files
    for env_file in settings.matching_env_files(&current_dir) {
        let file_env_vars = get_env_vars_from_file(&env_file)?;
        all_env_vars.extend(file_env_vars);
    }

    match shell.as_ref() {
        "bash" | "zsh" => export_bash_zsh(&all_env_vars),
        "fish" => export_fish(&all_env_vars),
        "json" => export_json(&all_env_vars),
        _ => Err(anyhow!("{} is currently not supported", shell)),
    }
}

/// Export variables for bash/zsh shells
fn export_bash_zsh(env_vars: &[String]) -> Result<()> {
    for env_var in env_vars {
        if env_var.starts_with("export ") {
            println!("{env_var}");
        } else {
            println!("export {env_var}");
        }
    }
    Ok(())
}

/// Export variables for fish shell
fn export_fish(env_vars: &[String]) -> Result<()> {
    for env_var in env_vars {
        if let Some((key, value)) = split_env_var(env_var) {
            println!("set -gx {key} {value}");
        }
    }
    Ok(())
}

/// Export variables as JSON
fn export_json(env_vars: &[String]) -> Result<()> {
    let env_vars: serde_json::Map<String, Value> = env_vars
        .iter()
        .filter_map(|var| split_env_var(var).map(|(key, value)| (key, Value::String(value))))
        .collect();

    let json = serde_json::to_string(&env_vars).context("Failed to serialize to JSON")?;
    println!("{json}");

    Ok(())
}
