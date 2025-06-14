use clap::Parser;
use colored::Colorize;
use dotman::{Dotman, DotmanConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(styles = get_styles())]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Link dotfiles to their respective locations
    Install {
        /// Path to the configuration file
        #[clap(short, long, default_value = "dotman.toml")]
        config: PathBuf,
        /// Override existing links if they already exist
        #[clap(short, long, default_value = "false")]
        overwrite: bool,
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
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Install { config, overwrite } => {
            let config = DotmanConfig::try_from(config.as_path())
                .unwrap_or_else(|err| {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                    std::process::exit(1);
                })
                .with_overwrite(overwrite);

            let dotman = Dotman::new(config);

            if let Err(e) = dotman.install() {
                eprintln!("{} {}", "Error:".red().bold(), e.message());
                std::process::exit(1);
            } else {
                println!("{}", "Installation completed successfully.".green());
            }
        }
        Command::Validate { config } => {
            if let Err(e) = DotmanConfig::try_from(config.as_path()) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            } else {
                println!("{}", "Configuration file is valid.".green());
            }
        }
        Command::Show { config } => {
            let config = DotmanConfig::try_from(config.as_path()).unwrap_or_else(|err| {
                eprintln!("{} {}", "Error:".red().bold(), err);
                std::process::exit(1);
            });

            println!("{:#?}", config);
        }
        Command::Remove { config } => {
            let config = DotmanConfig::try_from(config.as_path()).unwrap_or_else(|err| {
                eprintln!("{} {}", "Error:".red().bold(), err);
                std::process::exit(1);
            });

            let dotman = Dotman::new(config);

            if let Err(e) = dotman.remove() {
                eprintln!("{} {}", "Error:".red().bold(), e.message());
                std::process::exit(1);
            } else {
                println!("{}", "Removal completed successfully.".green());
            }
        }
    }
}

// Thank you to:
// https://stackoverflow.com/a/76916424/8653870
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}
