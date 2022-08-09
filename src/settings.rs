use anyhow::{Context, Result};
use config::Config;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvySettings {
    pub paths: Option<Vec<PathConfig>>,
    pub envs: Option<Vec<PathBuf>>,
}

impl EnvySettings {
    // Add a path to an env file to the list of allowed files
    pub fn add_env(&mut self, path: PathBuf) -> &mut Self {
        // Add directory to settings
        match self.envs.as_mut() {
            Some(envs) => {
                envs.push(path);
            }
            None => self.envs = Some(vec![path]),
        };
        self
    }

    pub fn matches(self, dir: PathBuf) -> Option<Vec<String>> {
        let path_str = dir.to_string_lossy();
        for path in self.paths? {
            if path.pattern.is_match(&path_str) {
                return Some(path.env);
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
    pub env: Vec<String>,
}

pub(crate) struct Settings {}

impl Settings {
    pub fn load(config_path: PathBuf) -> Result<EnvySettings> {
        config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .context("Cannot not read config")?
            .try_deserialize::<EnvySettings>()
            .context("Cannot deserialize config")
    }

    pub fn save(config_path: PathBuf, settings: EnvySettings) -> Result<()> {
        let serialized: &[u8] = Config::try_from(&settings)
            .context("Cannot read config")?
            .try_deserialize()
            .context("Cannot serialize config to disk")?;

        fs::write(config_path, serialized).context("Cannot write config")
    }
}
