use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    error::DotmanError,
    utils::{ExpandTilde, MakeAbsolute, get_current_os},
};

pub mod config;
pub mod error;
pub mod utils;

pub use crate::config::DotmanConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
}

pub struct Dotman {
    pub config: DotmanConfig,
}

impl Dotman {
    pub fn new(config: DotmanConfig) -> Self {
        Dotman { config }
    }

    pub fn install(&self) -> Result<(), DotmanError> {
        let os = get_current_os();

        for link in &self.config.links {
            let source = link.source.expand_tilde_path()?.make_absolute()?;
            let target = link.target.expand_tilde_path()?.make_absolute()?;

            let all_conditions_met = link
                .condition
                .as_ref()
                .is_none_or(|cond| cond.os.is_empty() || cond.os.contains(&os));

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
                if self.config.overwrite {
                    if target.is_dir() {
                        if let Err(e) = std::fs::remove_dir_all(&target) {
                            eprintln!(
                                "{} Failed to remove existing target directory {}: {}",
                                "Error:".red().bold(),
                                target.display(),
                                e
                            );
                            return Err(DotmanError::IoError(e));
                        }
                    } else if let Err(e) = std::fs::remove_file(&target) {
                        eprintln!(
                            "{} Failed to remove existing target {}: {}",
                            "Error:".red().bold(),
                            target.display(),
                            e
                        );
                        return Err(DotmanError::IoError(e));
                    }
                } else {
                    eprintln!(
                        "{} {} already exists, skipping. Use --overwrite to force linking.",
                        "Warning:".yellow().bold(),
                        target.display()
                    );
                    continue;
                }
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(source.clone(), target.clone())
                    .map_err(DotmanError::IoError)?;
            }
            #[cfg(windows)]
            {
                if source.is_dir() {
                    std::os::windows::fs::symlink_dir(source.clone(), target.clone())
                        .map_err(DotmanError::IoError)?;
                } else {
                    std::os::windows::fs::symlink_file(source.clone(), target.clone())
                        .map_err(DotmanError::IoError)?;
                }
            }

            println!(
                "{} {} -> {}",
                "Linked:".green().bold(),
                source.display(),
                target.display()
            );
        }

        Ok(())
    }

    pub fn remove(&self) -> Result<(), DotmanError> {
        for link in &self.config.links {
            let target = link.target.expand_tilde_path()?.make_absolute()?;

            if !target.exists() {
                eprintln!(
                    "{} {} does not exist, skipping.",
                    "Ignored:".yellow().bold(),
                    target.display()
                );
                continue;
            }

            if target.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(&target) {
                    eprintln!(
                        "{} Failed to remove directory {}: {}",
                        "Error:".red().bold(),
                        target.display(),
                        e
                    );
                    return Err(DotmanError::IoError(e));
                }
            } else if let Err(e) = std::fs::remove_file(&target) {
                eprintln!(
                    "{} Failed to remove file {}: {}",
                    "Error:".red().bold(),
                    target.display(),
                    e
                );
                return Err(DotmanError::IoError(e));
            }

            println!(
                "{} {} removed.",
                "Removed:".green().bold(),
                target.display()
            );
        }

        Ok(())
    }
}
