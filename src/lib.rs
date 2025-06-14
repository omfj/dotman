use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    error::DotmanError,
    utils::{ExpandTilde, MakeAbsolute},
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
    os: OperatingSystem,
}

impl Dotman {
    pub fn new(config: DotmanConfig) -> Self {
        let os = utils::get_operating_system().unwrap_or_else(|_| {
            panic!(
                "{} Failed to determine the operating system.",
                "Error:".red().bold()
            );
        });

        Dotman { config, os }
    }

    pub fn install(&self) -> Result<(), DotmanError> {
        for link in &self.config.links {
            let source = link
                .source
                .expand_tilde_path()?
                .canonicalize()?
                .make_absolute()?;

            let target = link
                .target
                .expand_tilde_path()?
                .canonicalize()?
                .make_absolute()?;

            let all_conditions_met = link
                .condition
                .as_ref()
                .is_none_or(|cond| cond.os.is_empty() || cond.os.contains(&self.os));

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
                    if let Err(e) = std::fs::remove_file(&target) {
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

            match self.os {
                OperatingSystem::Linux | OperatingSystem::MacOS => {
                    std::os::unix::fs::symlink(source.clone(), target.clone())
                        .map_err(DotmanError::IoError)?;
                }
                OperatingSystem::Windows => {
                    std::fs::hard_link(source.clone(), target.clone())
                        .map_err(DotmanError::IoError)?;
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
