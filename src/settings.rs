use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Settings for environment variables management
///
/// Holds configuration for environment files and directory patterns that
/// determine which environment variables to load based on the current
/// directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvySettings {
    /// List of allowed environment files which will be loaded
    pub envs: Option<Vec<PathBuf>>,

    /// List of regex patterns, each with associated environment variables
    ///
    /// If a directory matches a pattern, the associated environment variables
    /// will be loaded automatically when entering that directory.
    pub paths: Option<Vec<PathConfig>>,
}

impl EnvySettings {
    /// Add a path to an env file to the list of allowed files
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

    /// Remove a path to an env file from the list of allowed files
    pub fn remove_env(&mut self, path: PathBuf) -> &mut Self {
        if let Some(envs) = self.envs.as_mut() {
            envs.retain(|p| p != &path);
        };
        self
    }

    /// Check if a directory matches any of the configured patterns
    ///
    /// If a match is found, return the associated environment variables
    /// for the first matching pattern.
    pub fn matching_patterns(&self, dir: &Path) -> Option<Vec<String>> {
        let path_str = dir.to_string_lossy();
        self.paths
            .as_ref()?
            .iter()
            .find(|path| path.pattern.is_match(&path_str))
            .map(|path| path.env.clone())
    }

    /// Get all env files in dir and parent directory
    pub fn matching_env_files(&self, dir: &Path) -> Vec<PathBuf> {
        self.envs
            .iter()
            .flatten()
            .filter(|env| env.parent().is_some_and(|env_dir| dir.starts_with(env_dir)))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathConfig {
    /// The regex pattern to match against the directory path
    #[serde(with = "serde_regex")]
    pub pattern: Regex,

    /// The environment variables to load if the pattern matches
    pub env: Vec<String>,
}

/// Settings management for envy
///
/// Handles loading and saving configuration settings
/// from a TOML file.
pub(crate) struct Settings;

impl Settings {
    /// Load settings from the given config path
    /// Returns default empty settings if the config file doesn't exist
    pub fn load(config_path: PathBuf) -> Result<EnvySettings> {
        // If config file doesn't exist, return default empty settings
        if !config_path.exists() {
            return Ok(EnvySettings {
                envs: None,
                paths: None,
            });
        }

        config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .context("Cannot read config")?
            .try_deserialize::<EnvySettings>()
            .context("Cannot deserialize config")
    }

    /// Save settings to the given config path as TOML
    pub fn save(config_path: PathBuf, settings: EnvySettings) -> Result<()> {
        let toml = toml::to_string_pretty(&settings).context("Cannot serialize config")?;
        fs::write(config_path, toml).context("Cannot write config")
    }
}
