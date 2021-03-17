#[macro_use]
extern crate serde_derive;
use anyhow::{anyhow, Context, Result};

mod hooks;
mod opt;
mod settings;

use std::env::current_dir;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

use app_dirs::*;

use hooks::zsh::Zsh;
use opt::{Command, Envy};
use settings::Settings;

const APP_INFO: AppInfo = AppInfo {
    name: "Envy",
    author: "Matthias Endler",
};

fn config_path() -> Result<PathBuf> {
    let config = get_app_root(AppDataType::UserConfig, &APP_INFO)?;
    Ok(config.join("Config.toml"))
}

fn main() -> Result<()> {
    let opt = Envy::from_args();
    match opt.cmd {
        Command::Hook { shell } => hook(shell),
        Command::Export { shell } => export(shell),
        Command::Edit {} => edit(),
        Command::Show {} => show(),
        Command::Allow { env_file } => allow(env_file),
        Command::Deny {} => deny(),
        Command::Path {} => path(),
    }
}

fn deny() -> Result<()> {
    todo!()
}

// Add the current directory to the list of allowed paths.
// The `.env` file will be loaded automatically on dir enter.
fn allow(env_file: PathBuf) -> Result<()> {
    if !env_file.exists() {
        return Err(anyhow!("File does not exist: {}", env_file.display()));
    };
    let mut settings = Settings::load(config_path()?)?;
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
    match settings.matches(dir) {
        Some(env) => println!("{}", env.join("\n")),
        None => println!("envy found no matches for this directory."),
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
    if let Some(env) = settings.matches(current_dir()?) {
        println!("export {}", env.join(" "));
    }
    Ok(())
}
