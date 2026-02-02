use colored::Colorize;

use crate::{
    config::{Action, condition_is_met},
    error::DotmanError,
    utils::{Absolute, ExpandTilde},
};

pub mod config;
pub mod error;
pub mod utils;

pub use crate::config::DotmanConfig;

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

        for link in self.config.get_effective_links() {
            let source = link.source.expand_tilde_path()?.absolute()?;
            let target = link.target.expand_tilde_path()?.absolute()?;

            if !link.is_met(&os, hostname.as_deref()) {
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
                        return Err(e.into());
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

            if self.config.ask {
                use std::io::{self, Write};
                print!("Link {} -> {}? [y/N] ", source.display(), target.display());
                io::stdout().flush().map_err(DotmanError::IoError)?;
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(DotmanError::IoError)?;
                let input = input.trim().to_lowercase();
                if input != "y" && input != "yes" {
                    println!("{} Skipping.", "Skipped:".yellow().bold());
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

        for action in self.config.get_effective_actions() {
            match action {
                Action::ShellCommand {
                    name,
                    run,
                    if_cond,
                    if_not_cond,
                    ..
                } => {
                    if !condition_is_met(if_cond, if_not_cond, &os, hostname.as_deref()) {
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
                        return Err(DotmanError::CommandError {
                            command: name.clone(),
                            message: String::from_utf8_lossy(&output.stderr).to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn remove(&self) -> Result<(), DotmanError> {
        for link in self.config.get_effective_links() {
            let target = link.target.expand_tilde_path()?.absolute()?;

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
                    return Err(e.into());
                }
            } else if let Err(e) = std::fs::remove_file(&target) {
                eprintln!(
                    "{} Failed to remove file {}: {}",
                    "Error:".red().bold(),
                    target.display(),
                    e
                );
                return Err(e.into());
            }

            println!(
                "{} {} removed.",
                "Removed:".green().bold(),
                target.display()
            );
        }

        Ok(())
    }

    pub fn status(&self) -> Result<(), DotmanError> {
        let os = utils::get_current_os();
        let hostname = utils::get_hostname();

        println!("{}", "Dotman Status Report".blue().bold());
        println!();

        println!("{}", "Links:".blue().bold());
        println!();

        for link in self.config.get_effective_links() {
            let source = link.source.expand_tilde_path()?.absolute()?;
            let target = link.target.expand_tilde_path()?.absolute()?;

            if !link.is_met(&os, hostname.as_deref()) {
                print!("{}", "[CONDITION NOT MET]".yellow().bold());
                continue;
            }

            if !source.exists() {
                print!("{}", "[SOURCE MISSING]".red().bold());
                continue;
            }

            if !target.exists() {
                print!("{}", "[NOT LINKED]".yellow().bold());
                continue;
            }

            if target.is_symlink() {
                match target.read_link() {
                    Ok(actual_source) => {
                        if actual_source == source {
                            print!("{}", "[OK]".green().bold());
                        } else {
                            print!(
                                "{} (points to {})",
                                "[WRONG TARGET]".red().bold(),
                                actual_source.display()
                            );
                        }
                    }
                    Err(_) => {
                        print!("{}", "[SYMLINK ERROR]".red().bold());
                    }
                }
            } else {
                print!("{}", "[EXISTS BUT NOT SYMLINK]".yellow().bold());
            }

            print!(" ");
            println!("{} -> {}", source.display(), target.display());
        }

        if !self.config.get_effective_actions().is_empty() {
            println!();
            println!("{}", "Actions:".blue().bold());
            println!();

            for action in self.config.get_effective_actions() {
                match action {
                    Action::ShellCommand {
                        name,
                        if_cond,
                        if_not_cond,
                        ..
                    } => {
                        if !condition_is_met(if_cond, if_not_cond, &os, hostname.as_deref()) {
                            print!("{}", "[CONDITION NOT MET]".yellow().bold());
                        } else {
                            print!("{}", "[READY TO RUN]".green().bold());
                        }

                        print!(" ");
                        println!("Action: {}", name);
                    }
                }
            }
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
            ask: false,
            config_path: String::new(),
            selected_profile: None,
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
            profiles: vec![],
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
                ..Default::default()
            }),
            if_not_cond: None,
            profiles: vec![],
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
                ..Default::default()
            }),
            if_not_cond: None,
            profiles: vec![],
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
            profiles: vec![],
        };

        let config = create_test_config(vec![link], vec![]);
        let dotman = Dotman::new(config);

        dotman.remove().unwrap();

        assert!(!target_file.exists());
    }
}
