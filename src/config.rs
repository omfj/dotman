use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::OperatingSystem;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Condition {
    #[serde(default)]
    pub os: Vec<OperatingSystem>,
    #[serde(default)]
    pub hostname: Option<String>,
}

impl Condition {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        (self.os.is_empty() || self.os.contains(os))
            && (self.hostname.as_ref().is_none() || self.hostname.as_ref().unwrap() == hostname)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub target: String,
    pub source: String,
    #[serde(rename = "if")]
    pub if_cond: Option<Condition>,
    #[serde(rename = "if-not")]
    pub if_not_cond: Option<Condition>,
}

pub fn condition_is_met(
    if_cond: &Option<Condition>,
    if_not_cond: &Option<Condition>,
    os: &OperatingSystem,
    hostname: &str,
) -> bool {
    if_cond
        .as_ref()
        .map_or(true, |cond| cond.is_met(os, hostname))
        && if_not_cond
            .as_ref()
            .map_or(true, |cond| !cond.is_met(os, hostname))
}

impl Link {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        condition_is_met(&self.if_cond, &self.if_not_cond, os, hostname)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "shell-command")]
    ShellCommand {
        name: String,
        command: String,
        #[serde(rename = "if")]
        if_cond: Option<Condition>,
        #[serde(rename = "if-not")]
        if_not_cond: Option<Condition>,
    },
}

impl Action {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        match self {
            Action::ShellCommand {
                if_cond,
                if_not_cond,
                ..
            } => condition_is_met(if_cond, if_not_cond, os, hostname),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotmanConfig {
    #[serde(default = "base_config_path")]
    pub config_path: String,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default = "default_false")]
    pub overwrite: bool,
}

impl DotmanConfig {
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

fn default_false() -> bool {
    false
}

fn base_config_path() -> String {
    "dotman.toml".to_string()
}

#[derive(Debug)]
pub enum DotmanConfigError {
    ConfigFileDoesNotExist(PathBuf),
    ConfigFileReadError(PathBuf, std::io::Error),
    ConfigFileParseError(PathBuf, toml::de::Error),
}

impl fmt::Display for DotmanConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotmanConfigError::ConfigFileDoesNotExist(path) => {
                write!(f, "Configuration file does not exist: {}", path.display())
            }
            DotmanConfigError::ConfigFileReadError(path, err) => {
                write!(
                    f,
                    "Failed to read configuration file '{}': {}",
                    path.display(),
                    err
                )
            }
            DotmanConfigError::ConfigFileParseError(path, err) => {
                write!(
                    f,
                    "Failed to parse configuration file '{}': {}",
                    path.display(),
                    err
                )
            }
        }
    }
}

impl std::error::Error for DotmanConfigError {}

impl TryFrom<&Path> for DotmanConfig {
    type Error = DotmanConfigError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if !path.exists() {
            return Err(DotmanConfigError::ConfigFileDoesNotExist(
                path.to_path_buf(),
            ));
        }

        let file_str = std::fs::read_to_string(path)
            .map_err(|e| DotmanConfigError::ConfigFileReadError(path.to_path_buf(), e))?;

        let config = toml::from_str(&file_str)
            .map_err(|e| DotmanConfigError::ConfigFileParseError(path.to_path_buf(), e))?;

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_config_fail() {
        let config_file = PathBuf::from("tests/config/fail_to_parse.toml");
        let config = DotmanConfig::try_from(config_file.as_path());
        assert!(config.is_ok(), "Failed to parse config: {:?}", config.err());
    }

    #[test]
    fn test_parse_config_success() {
        let config_file = PathBuf::from("tests/config/working.toml");
        let config = DotmanConfig::try_from(config_file.as_path());

        assert!(config.is_ok(), "Failed to parse config: {:?}", config.err());

        let config = config.unwrap();

        assert_eq!(config.links.len(), 2);
        assert_eq!(config.links[1].if_cond.as_ref().unwrap().os.len(), 1);
        assert_eq!(
            config.links[1]
                .if_cond
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "foo"
        );

        assert_eq!(config.actions.len(), 1);

        assert!(!config.overwrite);
    }
}
