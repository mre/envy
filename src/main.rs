#[macro_use]
extern crate serde_derive;

mod errors;

use app_dirs::*;
use config;
use config::{Config, ConfigError, Environment, File};
use errors::EnvyError;
use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo {
    name: "Envy",
    author: "Matthias Endler",
};

#[derive(Debug, Deserialize)]
pub struct Settings {
    paths: Option<Vec<PathConfig>>,
}

#[derive(Debug, Deserialize)]
struct PathConfig {
    #[serde(with = "serde_regex")]
    pattern: Regex,
    env: Vec<String>,
}

impl Settings {
    pub fn new(config: Cow<str>) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(config::File::with_name(&config))?;
        s.try_into()
    }
}

fn main() -> Result<(), EnvyError> {
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
