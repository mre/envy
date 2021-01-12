use config;
use config::{Config, ConfigError};
use regex::Regex;
use std::borrow::Cow;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub paths: Option<Vec<PathConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct PathConfig {
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
    pub env: Vec<String>,
}

impl Settings {
    pub fn new(config: Cow<str>) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(config::File::with_name(&config))?;
        s.try_into()
    }
}
