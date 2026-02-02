use clap::Parser;
use colored::Colorize;

use dotman::{Dotman, DotmanConfig};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Path to the configuration file
    #[clap(short, long, default_value = "dotman.toml")]
    pub config: std::path::PathBuf,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Link dotfiles to their respective locations
    Install {
        /// Override existing links if they already exist
        #[clap(short, long, default_value = "false")]
        overwrite: bool,
        /// Ask before creating each symlink
        #[clap(short, long, default_value = "false")]
        ask: bool,
        /// Profile to use (applies global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
    /// Validate the configuration file
    Validate,
    /// Remove all links created by Dotman
    Remove {
        /// Profile to use (removes global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
    /// Show the status of all configured links
    Status {
        /// Profile to use (shows status for global + profile-specific configuration)
        #[clap(short, long)]
        profile: Option<String>,
    },
}

impl Cli {
    /// Runs the command specified in the CLI arguments.
    pub fn run(self) -> anyhow::Result<()> {
        // Load and validate the config once
        let config = DotmanConfig::try_from(self.config.as_path()).map_err(|err| {
            eprintln!("{} {}", "Error:".red().bold(), err);
            err
        })?;

        match self.command {
            Command::Install {
                overwrite,
                ask,
                profile,
            } => {
                let dotman_config = config
                    .with_overwrite(overwrite)
                    .with_ask(ask)
                    .with_profile(profile);
                let dotman = Dotman::new(dotman_config);
                Self::handle_install(dotman)
            }
            Command::Validate => Self::handle_validate(),
            Command::Remove { profile } => {
                let dotman_config = config.with_profile(profile);
                let dotman = Dotman::new(dotman_config);
                Self::handle_remove(dotman)
            }
            Command::Status { profile } => {
                let dotman_config = config.with_profile(profile);
                let dotman = Dotman::new(dotman_config);
                Self::handle_status(dotman)
            }
        }
    }

    fn handle_install(dotman: Dotman) -> anyhow::Result<()> {
        if let Err(e) = dotman.install() {
            eprintln!("{} {}", "Error:".red().bold(), e);
            return Err(e.into());
        }
        println!("{}", "Installation completed successfully.".green());
        Ok(())
    }

    fn handle_validate() -> anyhow::Result<()> {
        println!("{}", "Configuration file is valid.".green());
        Ok(())
    }

    fn handle_remove(dotman: Dotman) -> anyhow::Result<()> {
        if let Err(e) = dotman.remove() {
            eprintln!("{} {}", "Error:".red().bold(), e);
            return Err(e.into());
        }
        println!("{}", "Removal completed successfully.".green());
        Ok(())
    }

    fn handle_status(dotman: Dotman) -> anyhow::Result<()> {
        if let Err(e) = dotman.status() {
            eprintln!("{} {}", "Error:".red().bold(), e);
            return Err(e.into());
        }
        Ok(())
    }
}
