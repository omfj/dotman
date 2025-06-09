use std::fmt;
use std::path::{Path, PathBuf};

use colored::Colorize;
use serde::{Deserialize, Serialize};

pub mod hash;
pub mod utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
}

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
pub struct Config {
    version: String,
    #[serde(default = "base_config_path")]
    config_path: String,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub actions: Vec<Action>,
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

impl TryFrom<&Path> for Config {
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

pub enum DotmanError {
    SourceFileNotFound(String),
    IoError(std::io::Error),
}

impl DotmanError {
    pub fn message(&self) -> String {
        match self {
            DotmanError::SourceFileNotFound(source) => {
                format!("Source file not found: {}", source)
            }
            DotmanError::IoError(err) => format!("I/O error: {}", err),
        }
    }
}

pub struct Dotman {
    config: Config,
    os: OperatingSystem,
    should_overwrite: bool,
}

impl Dotman {
    pub fn new(config: Config, should_overwrite: bool) -> Self {
        let os = utils::get_operating_system().unwrap_or_else(|_| {
            eprintln!(
                "{} {}",
                "Error:".red().bold(),
                "Failed to determine the operating system."
            );
            std::process::exit(1);
        });

        Dotman {
            config,
            os,
            should_overwrite,
        }
    }

    pub fn install(&self) -> Result<(), DotmanError> {
        for link in &self.config.links {
            let pwd = std::env::current_dir().map_err(|e| {
                DotmanError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;

            let source = pwd.join(utils::expand_tilde(&link.source).map_err(|e| {
                DotmanError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?);
            let target = pwd.join(utils::expand_tilde(&link.target).map_err(|e| {
                DotmanError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?);

            let all_conditions_met = link.condition.as_ref().map_or(true, |cond| {
                cond.os.is_empty() || cond.os.contains(&self.os)
            });

            if !all_conditions_met {
                continue;
            }

            if !source.exists() {
                eprintln!(
                    "{} {} was not found, and will not be linked. Skipping.",
                    "Ignored:".yellow().bold(),
                    source.display()
                );
                continue;
            }

            if target.exists() {
                if self.should_overwrite {
                    std::fs::remove_file(target.clone()).map_err(|e| DotmanError::IoError(e))?;
                } else {
                    eprintln!(
                        "{} {} already exists, skipping. Use --overwrite to force linking.",
                        "Warning:".yellow().bold(),
                        target.display()
                    );
                    continue;
                }
            }

            match self.os {
                OperatingSystem::Linux | OperatingSystem::MacOS => {
                    std::os::unix::fs::symlink(source.clone(), target.clone())
                        .map_err(|e| DotmanError::IoError(e))?;
                }
                OperatingSystem::Windows => {
                    std::fs::hard_link(source.clone(), target.clone())
                        .map_err(|e| DotmanError::IoError(e))?;
                }
            };

            println!(
                "{} {} -> {}",
                "Linked:".green().bold(),
                source.display(),
                target.display()
            );
        }

        Ok(())
    }
}
