use anyhow::{anyhow, Context, Result};

mod hooks;
mod opt;
mod settings;

use std::env::current_dir;
use std::path::PathBuf;
use std::process;
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
        Command::Allow { env_file } => allow(env_file),
        Command::Deny { env_file } => deny(env_file),
        Command::Path {} => path(),
    }
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

fn show() -> Result<()> {
    let settings = Settings::load(config_path()?)?;
    let dir = current_dir()?;
    settings.matching_env_files(&dir).iter().for_each(|env| {
        println!("Loading {}", env.display());
    });
    match settings.matching_patterns(&dir) {
        Some(env) => println!("{}", env.join("\n")),
        None => println!("envy found no pattern matches for this directory."),
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

fn export(shell: String) -> Result<()> {
    if shell != *"zsh" {
        todo!("{} not supported yet. You could add support for it!", shell);
    }
    let settings = Settings::load(config_path()?)?;
    if let Some(patterns) = settings.matching_patterns(&current_dir()?) {
        println!("export {}", patterns.join(" "));
    }
    for env_file in settings.matching_env_files(&current_dir()?) {
        println!("source {}", env_file.display());
    }
    Ok(())
}
