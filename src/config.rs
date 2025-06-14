use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::OperatingSystem;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conditions {
    #[serde(default)]
    pub os: Vec<OperatingSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub target: String,
    pub source: String,
    pub condition: Option<Conditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitIfExistsStrategy {
    Skip,
    Overwrite,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "git_clone")]
    GitClone {
        name: String,
        repo: String,
        dest: String,
        condition: Option<Conditions>,
        if_exists: Option<GitIfExistsStrategy>,
    },
    #[serde(rename = "shell_command")]
    ShellCommand {
        name: String,
        command: String,
        condition: Option<Conditions>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotmanConfig {
    pub version: String,
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

        assert_eq!(config.version, "1");
        assert_eq!(config.links.len(), 2);
        assert_eq!(config.actions.len(), 0);
        assert!(!config.overwrite);
    }
}
