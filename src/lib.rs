use std::process::Command;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Action, condition_is_met},
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
}

impl Dotman {
    pub fn new(config: DotmanConfig) -> Self {
        Dotman { config }
    }

    pub fn install(&self) -> Result<(), DotmanError> {
        let os = utils::get_current_os();
        let hostname = utils::get_hostname();

        for link in &self.config.links {
            let source = link.source.expand_tilde_path()?.make_absolute()?;
            let target = link.target.expand_tilde_path()?.make_absolute()?;

            if !link.is_met(&os, &hostname) {
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

            utils::symlink(source.clone(), target.clone())?;

            println!(
                "{} {} -> {}",
                "Linked:".green().bold(),
                source.display(),
                target.display()
            );
        }

        for action in self.config.actions.iter() {
            match action {
                Action::ShellCommand {
                    name,
                    command,
                    if_cond,
                    if_not_cond,
                } => {
                    if !condition_is_met(if_cond, if_not_cond, &os, &hostname) {
                        continue;
                    }

                    println!("{} Running action: {}", "Action:".blue().bold(), name);

                    let mut command_builder = if cfg!(target_os = "windows") {
                        let mut cmd = Command::new("cmd");
                        cmd.arg("/C").arg(command);
                        cmd
                    } else {
                        let mut cmd = Command::new("bash");
                        cmd.arg("-c").arg(command);
                        cmd
                    };

                    let output = command_builder.output()?;

                    if output.status.success() {
                        println!(
                            "{} {}",
                            "Success:".green().bold(),
                            String::from_utf8_lossy(&output.stdout)
                        );
                    } else {
                        return Err(DotmanError::CommandError(
                            name.clone(),
                            String::from_utf8_lossy(&output.stderr).to_string(),
                        ));
                    }
                }
            }
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
