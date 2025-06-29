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
                println!(
                    "{} {} failed condition check, skipping.",
                    "Ignored:".yellow().bold(),
                    source.display()
                );
                continue;
            }

            if !source.exists() {
                println!(
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
                            println!(
                                "{} Failed to remove existing target directory {}: {}",
                                "Error:".red().bold(),
                                target.display(),
                                e
                            );
                            return Err(DotmanError::IoError(e));
                        }
                    } else if let Err(e) = std::fs::remove_file(&target) {
                        println!(
                            "{} Failed to remove existing target {}: {}",
                            "Error:".red().bold(),
                            target.display(),
                            e
                        );
                        return Err(DotmanError::IoError(e));
                    }
                } else {
                    println!(
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
                    run,
                    if_cond,
                    if_not_cond,
                } => {
                    if !condition_is_met(if_cond, if_not_cond, &os, &hostname) {
                        println!(
                            "{} {} failed condition check, skipping.",
                            "Ignored:".yellow().bold(),
                            name
                        );
                        continue;
                    }

                    println!("{} Running action: {}", "Action:".blue().bold(), name);

                    let output = run.execute()?;

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
                println!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Action, Condition, DotmanConfig, Link, RunCommand};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(links: Vec<Link>, actions: Vec<Action>) -> DotmanConfig {
        DotmanConfig {
            links,
            actions,
            overwrite: false,
            config_path: String::new(),
        }
    }

    #[test]
    fn test_dotman_install_basic_link() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&source_file, "test content").unwrap();

        let link = Link {
            source: source_file.to_string_lossy().to_string(),
            target: target_file.to_string_lossy().to_string(),
            if_cond: None,
            if_not_cond: None,
        };

        let config = create_test_config(vec![link], vec![]);
        let dotman = Dotman::new(config);

        dotman.install().unwrap();

        assert!(target_file.exists());
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "test content");
    }

    #[test]
    fn test_dotman_install_with_condition_met() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&source_file, "test content").unwrap();

        let link = Link {
            source: source_file.to_string_lossy().to_string(),
            target: target_file.to_string_lossy().to_string(),
            if_cond: Some(Condition {
                os: vec![],
                hostname: None,
                run: Some(RunCommand::Simple("true".to_string())),
            }),
            if_not_cond: None,
        };

        let config = create_test_config(vec![link], vec![]);
        let dotman = Dotman::new(config);

        dotman.install().unwrap();

        assert!(target_file.exists());
    }

    #[test]
    fn test_dotman_install_with_condition_not_met() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&source_file, "test content").unwrap();

        let link = Link {
            source: source_file.to_string_lossy().to_string(),
            target: target_file.to_string_lossy().to_string(),
            if_cond: Some(Condition {
                os: vec![],
                hostname: None,
                run: Some(RunCommand::Simple("false".to_string())),
            }),
            if_not_cond: None,
        };

        let config = create_test_config(vec![link], vec![]);
        let dotman = Dotman::new(config);

        dotman.install().unwrap();

        assert!(!target_file.exists());
    }

    #[test]
    fn test_dotman_remove_existing_link() {
        let temp_dir = TempDir::new().unwrap();
        let target_file = temp_dir.path().join("target.txt");

        fs::write(&target_file, "test content").unwrap();

        let link = Link {
            source: "source.txt".to_string(),
            target: target_file.to_string_lossy().to_string(),
            if_cond: None,
            if_not_cond: None,
        };

        let config = create_test_config(vec![link], vec![]);
        let dotman = Dotman::new(config);

        dotman.remove().unwrap();

        assert!(!target_file.exists());
    }

    #[test]
    fn test_action_is_met_conditions() {
        let action_met = Action::ShellCommand {
            name: "Test action".to_string(),
            run: RunCommand::Simple("echo test".to_string()),
            if_cond: Some(Condition {
                os: vec![],
                hostname: None,
                run: Some(RunCommand::Simple("true".to_string())),
            }),
            if_not_cond: None,
        };

        assert!(action_met.is_met(&OperatingSystem::Linux, "test"));

        let action_not_met = Action::ShellCommand {
            name: "Test action".to_string(),
            run: RunCommand::Simple("echo test".to_string()),
            if_cond: Some(Condition {
                os: vec![],
                hostname: None,
                run: Some(RunCommand::Simple("false".to_string())),
            }),
            if_not_cond: None,
        };

        assert!(!action_not_met.is_met(&OperatingSystem::Linux, "test"));
    }
}
