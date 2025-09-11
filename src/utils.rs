use colored::Colorize;

use crate::config::OperatingSystem;

pub trait ExpandTilde {
    /// Expands a path starting with `~` to the user's home directory.
    /// If the path does not start with `~`, it is returned as is.
    fn expand_tilde_path(&self) -> Result<std::path::PathBuf, String>;
}

impl<P: AsRef<std::path::Path>> ExpandTilde for P {
    fn expand_tilde_path(&self) -> Result<std::path::PathBuf, String> {
        let path_str = self.as_ref().to_string_lossy().to_string();
        if path_str.starts_with("~") {
            if let Some(home_dir) = dirs::home_dir() {
                let relative_path = path_str.strip_prefix("~").unwrap_or(&path_str);
                Ok(home_dir.join(relative_path.trim_start_matches('/')))
            } else {
                Err("Home directory not found".to_string())
            }
        } else {
            Ok(self.as_ref().to_path_buf())
        }
    }
}

pub trait Absolute {
    /// Converts a relative path to an absolute path based on the current working directory.
    fn absolute(&self) -> Result<std::path::PathBuf, String>;
}

impl<P: AsRef<std::path::Path>> Absolute for P {
    fn absolute(&self) -> Result<std::path::PathBuf, String> {
        let path = self.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())
                .map(|current_dir| current_dir.join(path))
        }
    }
}

/// Detects the current operating system and returns an `OperatingSystem` enum.
/// Panics if the operating system is unsupported / not found.
pub fn get_current_os() -> OperatingSystem {
    match std::env::consts::OS {
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        "windows" => OperatingSystem::Windows,
        os => panic!(
            "{} Unsupported operating system '{}' for Dotman.",
            "Error:".red().bold(),
            os
        ),
    }
}

/// Wrapper for creating symbolic links that works across different operating systems.
pub fn symlink<P: AsRef<std::path::Path>>(source: P, target: P) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        if source.as_ref().is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}

pub fn get_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
