use std::{
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;

use crate::OperatingSystem;

pub trait ExpandTilde {
    fn expand_tilde_path(&self) -> Result<PathBuf, String>;
}

impl<P: AsRef<Path>> ExpandTilde for P {
    fn expand_tilde_path(&self) -> Result<PathBuf, String> {
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

pub trait MakeAbsolute {
    fn make_absolute(&self) -> Result<PathBuf, String>;
}

impl<P: AsRef<Path>> MakeAbsolute for P {
    fn make_absolute(&self) -> Result<PathBuf, String> {
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

pub fn get_current_os() -> OperatingSystem {
    match std::env::consts::OS {
        "linux" => OperatingSystem::Linux,
        "macos" => OperatingSystem::MacOS,
        "windows" => OperatingSystem::Windows,
        os => panic!("{} Unsupported operating system '{}' for Dotman.", "Error:".red().bold(), os),
    }
}

pub fn symlink<P: AsRef<Path>>(source: P, target: P) -> std::io::Result<()> {
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
    Command::new("hostname")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
