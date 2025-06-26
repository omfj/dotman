use std::{
    fmt,
    ops::Not,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::OperatingSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Shell {
    #[default]
    Sh,
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn as_str(&self) -> &'static str {
        match self {
            Shell::Sh => "sh",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunCommand {
    Simple(String),
    Complex {
        command: String,
        shell: Option<Shell>,
    },
}

impl RunCommand {
    pub fn execute(&self) -> Result<std::process::Output, std::io::Error> {
        match self {
            RunCommand::Simple(cmd) => std::process::Command::new("sh").arg("-c").arg(cmd).output(),
            RunCommand::Complex { command, shell } => {
                let shell_cmd = shell.as_ref().unwrap_or(&Shell::Sh).as_str();
                std::process::Command::new(shell_cmd)
                    .arg("-c")
                    .arg(command)
                    .output()
            }
        }
    }

    pub fn is_successful(&self) -> bool {
        match self.execute() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Condition {
    #[serde(default)]
    pub os: Vec<OperatingSystem>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub run: Option<RunCommand>,
}

impl Condition {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        let os_is_met = self.os.is_empty() || self.os.iter().any(|o| o == os);
        let hostname_is_met = self.hostname.as_ref().is_none_or(|h| h == hostname);

        let command_is_met = self
            .run
            .as_ref()
            .is_none_or(|run_cmd| run_cmd.is_successful());

        os_is_met && hostname_is_met && command_is_met
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub target: String,
    pub source: String,
    #[serde(rename = "if")]
    pub if_cond: Option<Condition>,
    #[serde(rename = "if-not")]
    pub if_not_cond: Option<Condition>,
}

pub fn condition_is_met(
    if_cond: &Option<Condition>,
    if_not_cond: &Option<Condition>,
    os: &OperatingSystem,
    hostname: &str,
) -> bool {
    let if_is_met = if_cond
        .as_ref()
        .is_none_or(|cond| cond.is_met(os, hostname));
    let if_not_is_met = if_not_cond
        .as_ref()
        .is_none_or(|cond| cond.is_met(os, hostname).not());
    if_is_met && if_not_is_met
}

impl Link {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        condition_is_met(&self.if_cond, &self.if_not_cond, os, hostname)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "shell-command")]
    ShellCommand {
        name: String,
        run: RunCommand,
        #[serde(rename = "if")]
        if_cond: Option<Condition>,
        #[serde(rename = "if-not")]
        if_not_cond: Option<Condition>,
    },
}

impl Action {
    pub fn is_met(&self, os: &OperatingSystem, hostname: &str) -> bool {
        match self {
            Action::ShellCommand {
                if_cond,
                if_not_cond,
                ..
            } => condition_is_met(if_cond, if_not_cond, os, hostname),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotmanConfig {
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

        assert_eq!(config.links.len(), 2);
        assert_eq!(config.links[1].if_cond.as_ref().unwrap().os.len(), 1);
        assert_eq!(
            config.links[1]
                .if_cond
                .as_ref()
                .unwrap()
                .hostname
                .as_ref()
                .unwrap(),
            "foo"
        );

        assert_eq!(config.actions.len(), 1);

        assert!(!config.overwrite);
    }

    #[test]
    fn test_run_command_simple() {
        let cmd = RunCommand::Simple("echo test".to_string());
        let output = cmd.execute().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test");
    }

    #[test]
    fn test_run_command_complex_with_shell() {
        let cmd = RunCommand::Complex {
            command: "echo test".to_string(),
            shell: Some(Shell::Bash),
        };
        let output = cmd.execute().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test");
    }

    #[test]
    fn test_run_command_complex_default_shell() {
        let cmd = RunCommand::Complex {
            command: "echo test".to_string(),
            shell: None,
        };
        let output = cmd.execute().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test");
    }

    #[test]
    fn test_run_command_is_successful_true() {
        let cmd = RunCommand::Simple("true".to_string());
        assert!(cmd.is_successful());
    }

    #[test]
    fn test_run_command_is_successful_false() {
        let cmd = RunCommand::Simple("false".to_string());
        assert!(!cmd.is_successful());
    }

    #[test]
    fn test_condition_os_match() {
        let condition = Condition {
            os: vec![OperatingSystem::Linux],
            hostname: None,
            run: None,
        };
        assert!(condition.is_met(&OperatingSystem::Linux, "test"));
        assert!(!condition.is_met(&OperatingSystem::MacOS, "test"));
    }

    #[test]
    fn test_condition_hostname_match() {
        let condition = Condition {
            os: vec![],
            hostname: Some("test-host".to_string()),
            run: None,
        };
        assert!(condition.is_met(&OperatingSystem::Linux, "test-host"));
        assert!(!condition.is_met(&OperatingSystem::Linux, "other-host"));
    }

    #[test]
    fn test_condition_empty_matches_all() {
        let condition = Condition::default();
        assert!(condition.is_met(&OperatingSystem::Linux, "any"));
        assert!(condition.is_met(&OperatingSystem::MacOS, "any"));
    }

    #[test]
    fn test_condition_with_successful_command() {
        let condition = Condition {
            os: vec![],
            hostname: None,
            run: Some(RunCommand::Simple("true".to_string())),
        };
        assert!(condition.is_met(&OperatingSystem::Linux, "test"));
    }

    #[test]
    fn test_condition_with_failed_command() {
        let condition = Condition {
            os: vec![],
            hostname: None,
            run: Some(RunCommand::Simple("false".to_string())),
        };
        assert!(!condition.is_met(&OperatingSystem::Linux, "test"));
    }

    #[test]
    fn test_condition_all_requirements_met() {
        let condition = Condition {
            os: vec![OperatingSystem::Linux],
            hostname: Some("test-host".to_string()),
            run: Some(RunCommand::Simple("true".to_string())),
        };
        assert!(condition.is_met(&OperatingSystem::Linux, "test-host"));
        assert!(!condition.is_met(&OperatingSystem::MacOS, "test-host"));
        assert!(!condition.is_met(&OperatingSystem::Linux, "other-host"));
    }

    #[test]
    fn test_action_is_met_with_conditions() {
        let action = Action::ShellCommand {
            name: "test".to_string(),
            run: RunCommand::Simple("echo test".to_string()),
            if_cond: Some(Condition {
                os: vec![OperatingSystem::Linux],
                hostname: None,
                run: None,
            }),
            if_not_cond: None,
        };
        assert!(action.is_met(&OperatingSystem::Linux, "test"));
        assert!(!action.is_met(&OperatingSystem::MacOS, "test"));
    }

    #[test]
    fn test_action_is_met_with_if_not_condition() {
        let action = Action::ShellCommand {
            name: "test".to_string(),
            run: RunCommand::Simple("echo test".to_string()),
            if_cond: None,
            if_not_cond: Some(Condition {
                os: vec![OperatingSystem::MacOS],
                hostname: None,
                run: None,
            }),
        };
        assert!(action.is_met(&OperatingSystem::Linux, "test"));
        assert!(!action.is_met(&OperatingSystem::MacOS, "test"));
    }
}
