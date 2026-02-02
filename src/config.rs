use anyhow::Context;
use serde::{Deserialize, Serialize};

const fn default_false() -> bool {
    false
}

fn base_config_path() -> String {
    "dotman.toml".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Hostname {
    Single(String),
    Multiple(Vec<String>),
}

impl RunCommand {
    pub fn execute(&self) -> Result<std::process::Output, std::io::Error> {
        match self {
            RunCommand::Simple(cmd) => std::process::Command::new("sh").arg("-c").arg(cmd).output(),
            RunCommand::Complex { command, shell } => {
                let shell = shell.as_ref().unwrap_or(&Shell::Sh).as_str();
                std::process::Command::new(shell)
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
    pub hostname: Option<Hostname>,
    #[serde(default)]
    pub run: Option<RunCommand>,
    #[serde(default)]
    pub file_exists: Vec<String>,
}

fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{}", home.display(), stripped);
    }
    path.to_string()
}

impl Condition {
    pub fn is_met(&self, os: &OperatingSystem, hostname: Option<&str>) -> bool {
        let os_matches = self.os.is_empty() || self.os.contains(os);
        let hostname_matches = self.hostname.as_ref().is_none_or(|h| match hostname {
            None => false,
            Some(hostname) => match h {
                Hostname::Single(h) => h == hostname,
                Hostname::Multiple(hosts) => hosts.iter().any(|h| h == hostname),
            },
        });
        let command_succeeds = self.run.as_ref().is_none_or(|cmd| cmd.is_successful());
        let files_exist = self
            .file_exists
            .iter()
            .all(|path| std::path::Path::new(&expand_tilde(path)).exists());

        os_matches && hostname_matches && command_succeeds && files_exist
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
    #[serde(default)]
    pub profiles: Vec<String>,
}

pub fn condition_is_met(
    if_cond: &Option<Condition>,
    if_not_cond: &Option<Condition>,
    os: &OperatingSystem,
    hostname: Option<&str>,
) -> bool {
    let if_condition_passes = if_cond
        .as_ref()
        .is_none_or(|cond| cond.is_met(os, hostname));
    let if_not_condition_passes = if_not_cond
        .as_ref()
        .is_none_or(|cond| !cond.is_met(os, hostname));

    if_condition_passes && if_not_condition_passes
}

impl Link {
    pub fn is_met(&self, os: &OperatingSystem, hostname: Option<&str>) -> bool {
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
        #[serde(default)]
        profiles: Vec<String>,
    },
}

impl Action {
    pub fn is_met(&self, os: &OperatingSystem, hostname: Option<&str>) -> bool {
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
    #[serde(skip)]
    pub ask: bool,
    #[serde(skip)]
    pub selected_profile: Option<String>,
}

impl DotmanConfig {
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    pub fn with_ask(mut self, ask: bool) -> Self {
        self.ask = ask;
        self
    }

    pub fn with_profile(mut self, profile: Option<String>) -> Self {
        self.selected_profile = profile;
        self
    }

    pub fn get_effective_links(&self) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|link| self.profile_matches(&link.profiles))
            .collect()
    }

    pub fn get_effective_actions(&self) -> Vec<&Action> {
        self.actions
            .iter()
            .filter(|action| match action {
                Action::ShellCommand { profiles, .. } => self.profile_matches(profiles),
            })
            .collect()
    }

    fn profile_matches(&self, profiles: &[String]) -> bool {
        profiles.is_empty()
            || self
                .selected_profile
                .as_ref()
                .is_some_and(|selected| profiles.contains(selected))
    }
}

impl TryFrom<&std::path::Path> for DotmanConfig {
    type Error = anyhow::Error;

    fn try_from(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file does not exist: {}",
                path.display()
            ));
        }

        let file_str = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file '{}'", path.display()))?;

        let config: DotmanConfig = toml::from_str(&file_str)
            .with_context(|| format!("Failed to parse configuration file '{}'", path.display()))?;

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

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
        match config.links[1]
            .if_cond
            .as_ref()
            .unwrap()
            .hostname
            .as_ref()
            .unwrap()
        {
            Hostname::Single(h) => assert_eq!(h, "foo"),
            Hostname::Multiple(_) => panic!("Expected single hostname"),
        }

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
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test")));
        assert!(!condition.is_met(&OperatingSystem::MacOS, Some("test")));
    }

    #[test]
    fn test_condition_hostname_match() {
        let condition = Condition {
            os: vec![],
            hostname: Some(Hostname::Single("test-host".to_string())),
            run: None,
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test-host")));
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("other-host")));
        // Should not match when hostname is None
        assert!(!condition.is_met(&OperatingSystem::Linux, None));
    }

    #[test]
    fn test_condition_hostname_multiple_match() {
        let condition = Condition {
            os: vec![],
            hostname: Some(Hostname::Multiple(vec![
                "host1".to_string(),
                "host2".to_string(),
                "host3".to_string(),
            ])),
            run: None,
            ..Default::default()
        };
        // Should match any of the hostnames in the list
        assert!(condition.is_met(&OperatingSystem::Linux, Some("host1")));
        assert!(condition.is_met(&OperatingSystem::Linux, Some("host2")));
        assert!(condition.is_met(&OperatingSystem::Linux, Some("host3")));
        // Should not match hostnames not in the list
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("other-host")));
        // Should not match when hostname is None
        assert!(!condition.is_met(&OperatingSystem::Linux, None));
    }

    #[test]
    fn test_condition_empty_matches_all() {
        let condition = Condition::default();
        assert!(condition.is_met(&OperatingSystem::Linux, Some("any")));
        assert!(condition.is_met(&OperatingSystem::MacOS, Some("any")));
        // Should also match when hostname is None
        assert!(condition.is_met(&OperatingSystem::Linux, None));
    }

    #[test]
    fn test_condition_with_successful_command() {
        let condition = Condition {
            os: vec![],
            hostname: None,
            run: Some(RunCommand::Simple("true".to_string())),
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test")));
    }

    #[test]
    fn test_condition_with_failed_command() {
        let condition = Condition {
            os: vec![],
            hostname: None,
            run: Some(RunCommand::Simple("false".to_string())),
            ..Default::default()
        };
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("test")));
    }

    #[test]
    fn test_condition_all_requirements_met() {
        let condition = Condition {
            os: vec![OperatingSystem::Linux],
            hostname: Some(Hostname::Single("test-host".to_string())),
            run: Some(RunCommand::Simple("true".to_string())),
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test-host")));
        assert!(!condition.is_met(&OperatingSystem::MacOS, Some("test-host")));
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("other-host")));
        assert!(!condition.is_met(&OperatingSystem::Linux, None));
    }

    #[test]
    fn test_condition_file_exists() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        // File exists - should match
        let condition = Condition {
            file_exists: vec![temp_path.clone()],
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test")));

        // File doesn't exist - should not match
        let non_existent = "/tmp/non_existent_file_12345.txt".to_string();
        let condition = Condition {
            file_exists: vec![non_existent],
            ..Default::default()
        };
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("test")));
    }

    #[test]
    fn test_condition_file_exists_with_tilde() {
        let home = dirs::home_dir().unwrap();

        // Create a temp file in home directory to test tilde expansion
        let temp_file = tempfile::NamedTempFile::new_in(&home).unwrap();
        let file_name = temp_file.path().file_name().unwrap().to_str().unwrap();
        let tilde_path = format!("~/{}", file_name);

        // File exists with tilde - should match
        let condition = Condition {
            file_exists: vec![tilde_path],
            ..Default::default()
        };
        assert!(condition.is_met(&OperatingSystem::Linux, Some("test")));

        // Non-existent file with tilde - should not match
        let condition = Condition {
            file_exists: vec!["~/non_existent_file_12345.txt".to_string()],
            ..Default::default()
        };
        assert!(!condition.is_met(&OperatingSystem::Linux, Some("test")));
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
                ..Default::default()
            }),
            if_not_cond: None,
            profiles: vec![],
        };
        assert!(action.is_met(&OperatingSystem::Linux, Some("test")));
        assert!(!action.is_met(&OperatingSystem::MacOS, Some("test")));
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
                ..Default::default()
            }),
            profiles: vec![],
        };
        assert!(action.is_met(&OperatingSystem::Linux, Some("test")));
        assert!(!action.is_met(&OperatingSystem::MacOS, Some("test")));
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
                ..Default::default()
            }),
            if_not_cond: None,
            profiles: vec![],
        };

        assert!(action_met.is_met(&OperatingSystem::Linux, Some("test")));

        let action_not_met = Action::ShellCommand {
            name: "Test action".to_string(),
            run: RunCommand::Simple("echo test".to_string()),
            if_cond: Some(Condition {
                os: vec![],
                hostname: None,
                run: Some(RunCommand::Simple("false".to_string())),
                ..Default::default()
            }),
            if_not_cond: None,
            profiles: vec![],
        };

        assert!(!action_not_met.is_met(&OperatingSystem::Linux, Some("test")));
    }
}
