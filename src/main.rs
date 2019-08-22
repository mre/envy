#[macro_use]
extern crate serde_derive;

mod errors;
mod hooks;
mod opt;
mod settings;

use std::env::current_dir;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

use app_dirs::*;

use errors::EnvyError;
use hooks::zsh::Zsh;
use opt::{Command, Envy};
use settings::Settings;

const APP_INFO: AppInfo = AppInfo {
    name: "Envy",
    author: "Matthias Endler",
};

fn get_config() -> Result<PathBuf, EnvyError> {
    let config = get_app_root(AppDataType::UserConfig, &APP_INFO)?;
    Ok(config.join("Config.toml"))
}

fn main() -> Result<(), EnvyError> {
    let opt = Envy::from_args();
    match opt.cmd {
        Command::Hook { shell } => hook(shell),
        Command::Export { shell } => export(shell),
        Command::Edit {} => edit(),
        Command::Show {} => show(),
    }
}

pub fn edit_file(filename: &str) -> Result<std::process::ExitStatus, EnvyError> {
    let editor_name = std::env::var("EDITOR").map_err(EnvyError::InvalidEditor)?;
    let mut editor = process::Command::new(editor_name).arg(filename).spawn()?;
    Ok(editor.wait()?)
}

fn edit() -> Result<(), EnvyError> {
    let config = get_config()?;
    edit_file(&config.to_string_lossy())?;
    Ok(())
}

fn hook(shell: String) -> Result<(), EnvyError> {
    let hook = match shell.as_ref() {
        "zsh" => Zsh::hook()?,
        _ => return Err(EnvyError::InvalidShell(shell)),
    };
    println!("{}", hook);
    Ok(())
}

fn show() -> Result<(), EnvyError> {
    let settings = Settings::new(get_config()?.to_string_lossy())?;
    let dir = current_dir()?;
    if let Some(env) = find_matching(dir, settings) {
        println!("{}", env.join("\n"));
    } else {
        println!("envy found no matches for this directory.");
    }
    Ok(())
}

// TODO: We don't support different shells yet. Fix that.
fn export(_shell: String) -> Result<(), EnvyError> {
    let settings = Settings::new(get_config()?.to_string_lossy())?;
    let dir = current_dir()?;
    if let Some(env) = find_matching(dir, settings) {
        println!("export {}", env.join(" "));
    }
    Ok(())
}

fn find_matching(dir: PathBuf, settings: Settings) -> Option<Vec<String>> {
    let path_str = dir.to_string_lossy();
    for path in settings.paths? {
        if path.pattern.is_match(&path_str) {
            return Some(path.env);
        }
    }
    None
}
