use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

use dotman::{Dotman, DotmanConfig};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Link dotfiles to their respective locations
    Install {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
        /// Override existing links if they already exist
        #[clap(short, long, default_value = "false")]
        overwrite: bool,
        /// Profile to use (applies global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
    /// Validate the configuration file
    Validate {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
    },
    /// Show the configuration file in TOML format
    Show {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
    },
    /// Remove all links created by Dotman
    Remove {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
        /// Profile to use (removes global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
    /// Show the status of all configured links
    Status {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
        /// Profile to use (shows status for global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
}

impl Cli {
    /// Runs the command specified in the CLI arguments.
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Install {
                config,
                overwrite,
                profile,
            } => Self::handle_install(config, overwrite, profile),
            Command::Validate { config } => Self::handle_validate(config),
            Command::Show { config } => Self::handle_show(config),
            Command::Remove { config, profile } => Self::handle_remove(config, profile),
            Command::Status { config, profile } => Self::handle_status(config, profile),
        }
    }

    fn handle_install(
        config: PathBuf,
        overwrite: bool,
        profile: Option<String>,
    ) -> anyhow::Result<()> {
        let config = DotmanConfig::try_from(config.as_path())
            .map_err(|err| {
                eprintln!("{} {}", "Error:".red().bold(), err);
                err
            })?
            .with_overwrite(overwrite)
            .with_profile(profile);

        let dotman = Dotman::new(config);

        if let Err(e) = dotman.install() {
            eprintln!("{} {}", "Error:".red().bold(), e.message());
            return Err(e.into());
        }
        println!("{}", "Installation completed successfully.".green());
        Ok(())
    }

    fn handle_validate(config: PathBuf) -> anyhow::Result<()> {
        if let Err(e) = DotmanConfig::try_from(config.as_path()) {
            eprintln!("{} {}", "Error:".red().bold(), e);
            return Err(e.into());
        }
        println!("{}", "Configuration file is valid.".green());
        Ok(())
    }

    fn handle_show(config: PathBuf) -> anyhow::Result<()> {
        let config = DotmanConfig::try_from(config.as_path()).map_err(|err| {
            eprintln!("{} {}", "Error:".red().bold(), err);
            err
        })?;

        println!("{:#?}", config);
        Ok(())
    }

    fn handle_remove(config: PathBuf, profile: Option<String>) -> anyhow::Result<()> {
        let config = DotmanConfig::try_from(config.as_path())
            .map_err(|err| {
                eprintln!("{} {}", "Error:".red().bold(), err);
                err
            })?
            .with_profile(profile);

        let dotman = Dotman::new(config);

        if let Err(e) = dotman.remove() {
            eprintln!("{} {}", "Error:".red().bold(), e.message());
            return Err(e.into());
        }
        println!("{}", "Removal completed successfully.".green());
        Ok(())
    }

    fn handle_status(config: PathBuf, profile: Option<String>) -> anyhow::Result<()> {
        let config = DotmanConfig::try_from(config.as_path())
            .map_err(|err| {
                eprintln!("{} {}", "Error:".red().bold(), err);
                err
            })?
            .with_profile(profile);

        let dotman = Dotman::new(config);

        if let Err(e) = dotman.status() {
            eprintln!("{} {}", "Error:".red().bold(), e.message());
            return Err(e.into());
        }
        Ok(())
    }
}
