use anyhow::{anyhow, Context, Result};

mod hooks;
mod opt;
mod settings;

use std::path::{Path, PathBuf};
use std::process;
use std::{env::current_dir, fs};
use structopt::StructOpt;

use directories::BaseDirs;
use hooks::zsh::Zsh;
use opt::{Command, Envy};
use settings::Settings;

fn config_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Cannot get base directories")?;
    Ok(base_dirs.config_dir().join("envy").join("Config.toml"))
}

fn main() -> Result<()> {
    let opt = Envy::from_args();
    match opt.cmd {
        Command::Hook { shell } => hook(shell),
        Command::Export { shell } => export(shell),
        Command::Edit {} => edit(),
        Command::Show {} => show(),
        Command::Find { variable } => find(variable),
        Command::Load { env_file } => load(env_file),
        Command::Allow { env_file } => allow(env_file),
        Command::Deny { env_file } => deny(env_file),
        Command::Path {} => path(),
    }
}

/// Export all environment variables from the env file into the current shell
/// The command is called load because `source` is reserved for potentially
/// showing the source of an env variable in the future.
fn load(env_file: PathBuf) -> Result<(), anyhow::Error> {
    if !env_file.exists() {
        return Err(anyhow!("File does not exist: {}", env_file.display()));
    };
    source(env_file)
}

/// Get all environment variables currently set
/// and return the value of the given variable
fn find(variable: String) -> Result<(), anyhow::Error> {
    let value = std::env::vars()
        .find(|(key, _)| key == &variable)
        .map(|(_, value)| value);

    match value {
        Some(value) => println!("{}", value),
        None => println!("Variable {} not found", variable),
    }

    Ok(())
}

fn deny(env_file: PathBuf) -> Result<()> {
    if !env_file.exists() {
        return Err(anyhow!("File does not exist: {}", env_file.display()));
    };
    let mut settings = Settings::load(config_path()?)?;
    // Get full path to env file
    let env_file = env_file.canonicalize()?;
    settings.remove_env(env_file);
    Settings::save(config_path()?, settings)
}

// Add the current directory to the list of allowed paths.
// The `.env` file will be loaded automatically on dir enter.
fn allow(env_file: PathBuf) -> Result<()> {
    if !env_file.exists() {
        return Err(anyhow!("File does not exist: {}", env_file.display()));
    };
    let mut settings = Settings::load(config_path()?)?;
    // Get full path to env file
    let env_file = env_file.canonicalize()?;
    settings.add_env(env_file);
    Settings::save(config_path()?, settings)
}

pub fn open_editor(filename: &str) -> Result<std::process::ExitStatus> {
    let editor_name = std::env::var("EDITOR")?;
    let mut editor = process::Command::new(editor_name).arg(filename).spawn()?;
    Ok(editor.wait()?)
}

fn edit() -> Result<()> {
    let config = config_path()?;
    open_editor(&config.to_string_lossy())?;
    Ok(())
}

fn hook(shell: String) -> Result<()> {
    let hook = match shell.as_ref() {
        "zsh" => Zsh::hook()?,
        _ => return Err(anyhow!("{} is currently not supported", shell)),
    };
    println!("{}", hook);
    Ok(())
}

/// Get all environment variables from the given file
fn get_env_vars_from_file(env: &Path) -> Result<Vec<String>> {
    let mut env_vars = Vec::new();
    let env = fs::read_to_string(env).context("Cannot read env file")?;
    for line in env.lines() {
        // Ignore comments
        if line.starts_with('#') {
            continue;
        }
        env_vars.push(line.to_string())
    }
    Ok(env_vars)
}

fn show() -> Result<()> {
    let settings = Settings::load(config_path()?)?;
    let dir = current_dir()?;
    let env_files = settings.matching_env_files(&dir);
    for file in &env_files {
        println!("Loaded from `{}`:", file.display());
        let vars = get_env_vars_from_file(file).context("Cannot read env file")?;
        for var in vars {
            println!("{}", var);
        }
        println!();
    }
    match settings.matching_patterns(&dir) {
        Some(env) => println!("{}", env.join("\n")),
        None => {
            if env_files.is_empty() {
                println!("envy found no pattern matches for this directory.");
            }
        }
    };

    Ok(())
}

fn path() -> Result<()> {
    println!(
        "{}",
        config_path().context("Cannot read config path")?.display()
    );
    Ok(())
}

/// Source the given env file
/// This will print the commands to stdout that need to be executed to source
/// the file
///
/// This is used by the `envy export` command to source all matching env files
/// and by `envy load` to source the given env file directly (for the current
/// session)
fn source(env_file: PathBuf) -> Result<()> {
    for var in get_env_vars_from_file(&env_file)? {
        if var.starts_with("export") {
            println!("{}", var);
            continue;
        }

        println!("export {}", var);
    }
    Ok(())
}

fn export(shell: String) -> Result<()> {
    if shell != *"zsh" {
        todo!("{} not supported yet. You could add support for it!", shell);
    }
    let settings = Settings::load(config_path()?)?;
    if let Some(patterns) = settings.matching_patterns(&current_dir()?) {
        println!("export {}", patterns.join(" "));
    }
    for env_file in settings.matching_env_files(&current_dir()?) {
        source(env_file)?
    }
    Ok(())
}
