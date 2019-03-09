#[macro_use]
extern crate serde_derive;

mod errors;
mod hooks;
mod opt;
mod settings;

use std::env::current_dir;
use std::path::PathBuf;
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

fn main() -> Result<(), EnvyError> {
    let opt = Envy::from_args();
    match opt.cmd {
        Command::Hook { shell } => hook(shell),
        Command::Export { shell } => export(shell),
    }
}

fn hook(shell: String) -> Result<(), EnvyError> {
    let hook = match shell.as_ref() {
        "zsh" => Zsh::hook()?,
        _ => return Err(EnvyError::InvalidShell(shell)),
    };
    println!("{}", hook);
    Ok(())
}

// TODO: We don't support different shells yet. Fix that.
fn export(_shell: String) -> Result<(), EnvyError> {
    let mut config = get_app_root(AppDataType::UserConfig, &APP_INFO)?;
    config = config.join("Config.toml");
    let settings = Settings::new(config.to_string_lossy())?;
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
