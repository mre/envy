use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvySettings {
    pub envs: Option<Vec<PathBuf>>,
    pub paths: Option<Vec<PathConfig>>,
}

impl EnvySettings {
    // Add a path to an env file to the list of allowed files
    pub fn add_env(&mut self, path: PathBuf) -> &mut Self {
        // Add directory to settings
        match self.envs.as_mut() {
            Some(envs) => {
                if !envs.contains(&path) {
                    envs.push(path);
                }
            }
            None => self.envs = Some(vec![path]),
        };
        self
    }

    // Remove a path to an env file from the list of allowed files
    pub fn remove_env(&mut self, path: PathBuf) -> &mut Self {
        if let Some(envs) = self.envs.as_mut() {
            envs.retain(|p| p != &path);
        };
        self
    }

    pub fn matching_patterns(&self, dir: &Path) -> Option<Vec<String>> {
        let path_str = dir.to_string_lossy();
        for path in self.paths.as_ref()? {
            if path.pattern.is_match(&path_str) {
                return Some(path.env.clone());
            }
        }
        None
    }

    // get all env files in dir and parent directory
    pub fn matching_env_files(&self, dir: &Path) -> Vec<PathBuf> {
        self.envs.iter().flatten().filter(|env|
            // check if env file is in dir
            if let Some(env_dir) = env.parent() {
                dir.starts_with(env_dir)
            } else {
                false
            }
        ).cloned().collect()
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
        let toml = toml::to_string_pretty(&settings).context("Cannot serialize config")?;
        fs::write(config_path, toml).context("Cannot write config")
    }
}
